use std::{collections::HashMap, fs, path::PathBuf};

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
        // Auto-generated from Minecraft
        // Do not edit manually

        #![allow(dead_code)]
        #![allow(unused_imports)]

        use std::borrow::Cow;
        use mc_protocol::{Encode, Decode, Packet, State, Direction, VarInt, Uuid, Position, Nbt, BlockState};
        use serde::{Serialize, Deserialize};

        #(#direction_modules)*
    };

    prettyplease::unparse(&syn::parse2(header).expect("failed to parse generated code"))
}

fn generate_lib_rs(
    states: &[&str],
    protocol_version: Option<i32>,
    protocol_name: Option<&str>,
) -> String {
    let mut tokens = vec![quote! {
        // Auto-generated Minecraft packet definitions
        // Run `nix run .#mc-gen` to regenerate
    }];

    if let Some(version) = protocol_version {
        tokens.push(quote! {
            /// Protocol version for this build
            pub const PROTOCOL_VERSION: i32 = #version;
        });
    }

    if let Some(name) = protocol_name {
        tokens.push(quote! {
            /// Minecraft version name for this build
            pub const PROTOCOL_NAME: &str = #name;
        });
    }

    tokens.push(quote! {
        // Re-export protocol types
        pub use mc_protocol::{State, Direction, Packet};
    });

    for state in states {
        let state_ident = Ident::new(state, Span::call_site());
        tokens.push(quote! {
            pub mod #state_ident;
        });
    }

    let all_tokens = quote! { #(#tokens)* };
    prettyplease::unparse(&syn::parse2(all_tokens).expect("failed to parse lib.rs"))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 4 {
        eprintln!(
            "Usage: {} <ids_file> <fields_file> <out_dir> [protocol_version] [protocol_name]",
            args[0]
        );
        std::process::exit(1);
    }

    let ids_file = &args[1];
    let fields_file = &args[2];
    let out_dir = PathBuf::from(&args[3]);
    let protocol_version: Option<i32> = args.get(4).and_then(|s| s.parse().ok());
    let protocol_name: Option<&str> = args.get(5).map(String::as_str);

    // Load JSON files
    let ids_data: PacketIds = serde_json::from_str(&fs::read_to_string(ids_file)?)?;
    let fields_data: PacketFields = serde_json::from_str(&fs::read_to_string(fields_file)?)?;

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

    // Create output directory if needed
    fs::create_dir_all(&out_dir)?;

    // Generate each state module
    for state in &states {
        let content = generate_state_module(state, &ids_data, &fields_by_class);
        let file_path = out_dir.join(format!("{state}.rs"));
        fs::write(&file_path, content)?;
        eprintln!("Generated {}", file_path.display());
    }

    // Generate lib.rs
    let lib_content = generate_lib_rs(&states, protocol_version, protocol_name);
    let lib_path = out_dir.join("lib.rs");
    fs::write(&lib_path, lib_content)?;
    eprintln!("Generated {}", lib_path.display());

    Ok(())
}
