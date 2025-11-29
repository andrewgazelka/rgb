use std::{collections::HashMap, env, fs, path::PathBuf};

use heck::{ToSnakeCase, ToUpperCamelCase};
use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote};
use serde::Deserialize;

/// Known types that can be derived with Encode/Decode
const KNOWN_TYPES: &[&str] = &[
    "i8",
    "i16",
    "i32",
    "i64",
    "f32",
    "f64",
    "bool",
    "String",
    "Uuid",
    "Position",
    "Nbt",
    "BlockState",
];

/// Packet ID info from Mojang's data generator
#[derive(Debug, Deserialize)]
struct PacketIdInfo {
    protocol_id: i32,
}

/// State -> Direction -> PacketName -> PacketIdInfo
type PacketIds = HashMap<String, HashMap<String, HashMap<String, PacketIdInfo>>>;

/// Field info from reflection extraction
#[derive(Debug, Clone, Deserialize)]
struct FieldInfo {
    name: String,
    #[serde(rename = "rustType")]
    rust_type: String,
}

/// Packet info from reflection extraction
#[derive(Debug, Deserialize)]
struct PacketInfo {
    #[serde(rename = "className")]
    class_name: String,
    fields: Vec<FieldInfo>,
}

/// State -> Direction -> Vec<PacketInfo>
type PacketFields = HashMap<String, HashMap<String, Vec<PacketInfo>>>;

/// Protocol metadata
#[derive(Debug, Deserialize)]
struct ProtocolInfo {
    version: String,
    protocol_version: i32,
}

fn is_known_type(t: &str) -> bool {
    if KNOWN_TYPES.contains(&t) {
        return true;
    }
    if let Some(inner) = t.strip_prefix("Vec<").and_then(|s| s.strip_suffix('>')) {
        return is_known_type(inner);
    }
    if let Some(inner) = t.strip_prefix("Option<").and_then(|s| s.strip_suffix('>')) {
        return is_known_type(inner);
    }
    false
}

fn rust_type_tokens(t: &str) -> TokenStream {
    if KNOWN_TYPES.contains(&t) {
        let ident = format_ident!("{}", t);
        return quote! { #ident };
    }

    if let Some(inner) = t.strip_prefix("Vec<").and_then(|s| s.strip_suffix('>')) {
        if !inner.contains("Unknown") {
            let inner_tokens = rust_type_tokens(inner);
            return quote! { Vec<#inner_tokens> };
        }
    }

    if let Some(inner) = t.strip_prefix("Option<").and_then(|s| s.strip_suffix('>')) {
        if !inner.contains("Unknown") {
            let inner_tokens = rust_type_tokens(inner);
            return quote! { Option<#inner_tokens> };
        }
    }

    // Unknown types as raw bytes with comment
    let comment = format!(" {t} ");
    quote! { #[doc = #comment] Cow<'a, [u8]> }
}

fn needs_lifetime(fields: &[FieldInfo]) -> bool {
    fields.iter().any(|f| {
        let t = &f.rust_type;
        !is_known_type(t) && !KNOWN_TYPES.contains(&t.as_str())
    })
}

fn to_pascal_case(s: &str) -> String {
    s.replace("minecraft:", "")
        .replace('/', "_")
        .to_upper_camel_case()
}

fn sanitize_field_name(name: &str) -> Ident {
    let snake = name.to_snake_case();
    match snake.as_str() {
        "type" | "move" | "match" | "ref" | "self" | "super" | "mod" | "fn" | "let" | "const"
        | "static" | "use" | "impl" | "trait" | "struct" | "enum" | "pub" | "mut" | "if"
        | "else" | "for" | "while" | "loop" | "return" | "break" | "continue" | "async"
        | "await" | "dyn" | "in" | "where" | "crate" | "extern" | "unsafe" | "as" => {
            format_ident!("r#{}", snake)
        }
        _ => format_ident!("{}", snake),
    }
}

fn gen_struct(name: &str, fields: &[FieldInfo], packet_id: i32) -> TokenStream {
    let struct_name = format_ident!("{}", name);
    let has_lifetime = needs_lifetime(fields);
    let all_known = fields.iter().all(|f| is_known_type(&f.rust_type));

    let field_tokens: Vec<TokenStream> = fields
        .iter()
        .map(|f| {
            let fname = sanitize_field_name(&f.name);
            let ftype = rust_type_tokens(&f.rust_type);
            quote! { pub #fname: #ftype }
        })
        .collect();

    let doc = format!("Packet ID: {packet_id}");

    if all_known {
        if has_lifetime {
            quote! {
                #[doc = #doc]
                #[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize)]
                pub struct #struct_name<'a> {
                    #(#field_tokens,)*
                }
            }
        } else {
            quote! {
                #[doc = #doc]
                #[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize)]
                pub struct #struct_name {
                    #(#field_tokens,)*
                }
            }
        }
    } else if has_lifetime {
        quote! {
            #[doc = #doc]
            #[derive(Debug, Clone, Serialize, Deserialize)]
            pub struct #struct_name<'a> {
                #(#field_tokens,)*
            }
        }
    } else {
        quote! {
            #[doc = #doc]
            #[derive(Debug, Clone, Serialize, Deserialize)]
            pub struct #struct_name {
                #(#field_tokens,)*
            }
        }
    }
}

fn gen_packet_impl(struct_name: &str, packet_id: i32, state: &str, direction: &str) -> TokenStream {
    let struct_ident = format_ident!("{}", struct_name);

    let state_enum = match state {
        "handshake" => quote! { State::Handshaking },
        "status" => quote! { State::Status },
        "login" => quote! { State::Login },
        "configuration" => quote! { State::Configuration },
        "play" => quote! { State::Play },
        _ => quote! { State::Play },
    };

    let dir_enum = if direction == "clientbound" {
        quote! { Direction::Clientbound }
    } else {
        quote! { Direction::Serverbound }
    };

    quote! {
        impl Packet for #struct_ident {
            const ID: i32 = #packet_id;
            const STATE: State = #state_enum;
            const DIRECTION: Direction = #dir_enum;
        }
    }
}

fn gen_empty_struct(name: &str, packet_id: i32) -> TokenStream {
    let struct_name = format_ident!("{}", name);
    let doc = format!("Packet ID: {packet_id}");

    quote! {
        #[doc = #doc]
        #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
        pub struct #struct_name;
    }
}

fn generate_state_module(
    state: &str,
    ids_data: &PacketIds,
    fields_by_class: &HashMap<String, Vec<FieldInfo>>,
) -> String {
    let empty = HashMap::new();
    let state_ids = ids_data.get(state).unwrap_or(&empty);

    let mut direction_modules = Vec::new();

    for direction in ["clientbound", "serverbound"] {
        let Some(dir_ids) = state_ids.get(direction) else {
            continue;
        };

        let mut packets: Vec<(&String, &PacketIdInfo)> = dir_ids.iter().collect();
        packets.sort_by_key(|(_, info)| info.protocol_id);

        let mut packet_tokens = Vec::new();

        for (pkt_name, pkt_info) in packets {
            let pkt_id = pkt_info.protocol_id;
            let clean_name = pkt_name.replace("minecraft:", "").replace('/', "_");
            let struct_name = to_pascal_case(&clean_name);

            // Try to find fields
            let dir_prefix = if direction == "clientbound" {
                "Clientbound"
            } else {
                "Serverbound"
            };
            let class_patterns = [
                format!("{dir_prefix}{struct_name}Packet"),
                format!("{struct_name}Packet"),
                struct_name.clone(),
            ];

            let fields: Option<&Vec<FieldInfo>> = class_patterns
                .iter()
                .find_map(|pattern| fields_by_class.get(pattern));

            let struct_tokens = if let Some(flds) = fields {
                if flds.is_empty() {
                    gen_empty_struct(&struct_name, pkt_id)
                } else {
                    gen_struct(&struct_name, flds, pkt_id)
                }
            } else {
                gen_empty_struct(&struct_name, pkt_id)
            };

            let impl_tokens = gen_packet_impl(&struct_name, pkt_id, state, direction);

            packet_tokens.push(quote! {
                #struct_tokens

                #impl_tokens
            });
        }

        let dir_ident = Ident::new(direction, Span::call_site());
        let dir_module = quote! {
            pub mod #dir_ident {
                use super::*;

                #(#packet_tokens)*
            }
        };
        direction_modules.push(dir_module);
    }

    let header = quote! {
        use std::borrow::Cow;
        use mc_protocol::{Encode, Decode, Packet, State, Direction, VarInt, Uuid, Position, Nbt, BlockState};
        use serde::{Serialize, Deserialize};

        #(#direction_modules)*
    };

    prettyplease::unparse(&syn::parse2(header).expect("failed to parse generated code"))
}

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let data_dir = manifest_dir.join("data");

    // Tell cargo to rerun if data files change
    println!("cargo:rerun-if-changed=data/packets-ids.json");
    println!("cargo:rerun-if-changed=data/packets-fields.json");
    println!("cargo:rerun-if-changed=data/protocol.json");

    // Load JSON files
    let ids_json = fs::read_to_string(data_dir.join("packets-ids.json"))
        .expect("failed to read packets-ids.json");
    let ids_data: PacketIds =
        serde_json::from_str(&ids_json).expect("failed to parse packets-ids.json");

    // Fields file may be empty or missing - that's okay
    let fields_json = fs::read_to_string(data_dir.join("packets-fields.json")).unwrap_or_default();
    let fields_data: PacketFields = serde_json::from_str(&fields_json).unwrap_or_default();

    // Protocol info
    let protocol_json =
        fs::read_to_string(data_dir.join("protocol.json")).expect("failed to read protocol.json");
    let protocol_info: ProtocolInfo =
        serde_json::from_str(&protocol_json).expect("failed to parse protocol.json");

    // Build lookup for fields by class name
    let mut fields_by_class: HashMap<String, Vec<FieldInfo>> = HashMap::new();
    for dirs in fields_data.values() {
        for packets in dirs.values() {
            for p in packets {
                fields_by_class.insert(p.class_name.clone(), p.fields.clone());
            }
        }
    }

    let states = ["handshake", "status", "login", "configuration", "play"];

    // Generate each state module
    for state in &states {
        let content = generate_state_module(state, &ids_data, &fields_by_class);
        let file_path = out_dir.join(format!("{state}.rs"));
        fs::write(&file_path, content).expect("failed to write state module");
    }

    // Generate constants
    let protocol_version = protocol_info.protocol_version;
    let protocol_name = &protocol_info.version;
    let constants = quote! {
        /// Protocol version for this build
        pub const PROTOCOL_VERSION: i32 = #protocol_version;

        /// Minecraft version name for this build
        pub const PROTOCOL_NAME: &str = #protocol_name;
    };
    let constants_content =
        prettyplease::unparse(&syn::parse2(constants).expect("failed to parse constants"));
    fs::write(out_dir.join("constants.rs"), constants_content).expect("failed to write constants");
}
