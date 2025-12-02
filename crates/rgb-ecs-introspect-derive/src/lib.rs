//! Derive macro for the `Introspectable` trait.
//!
//! # Usage
//!
//! ```ignore
//! use rgb_ecs_introspect::Introspectable;
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(Clone, Serialize, Deserialize, Introspectable)]
//! pub struct Position {
//!     pub x: f64,
//!     pub y: f64,
//!     pub z: f64,
//! }
//!
//! // For opaque components (won't serialize internals)
//! #[derive(Clone, Introspectable)]
//! #[introspectable(opaque)]
//! pub struct PacketBuffer {
//!     data: Vec<u8>,
//! }
//! ```

use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

/// Derive macro for the `Introspectable` trait.
///
/// By default, generates implementations that use serde for JSON serialization.
/// The type must implement `Serialize` and `Deserialize`.
///
/// # Attributes
///
/// - `#[introspectable(opaque)]` - Marks the type as opaque, meaning it won't
///   serialize its internals. Instead, it returns `null` for JSON and cannot
///   be deserialized from the dashboard.
#[proc_macro_derive(Introspectable, attributes(introspectable))]
pub fn derive_introspectable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    // Check for #[introspectable(opaque)] attribute
    let is_opaque = input.attrs.iter().any(|attr| {
        if !attr.path().is_ident("introspectable") {
            return false;
        }
        let Ok(nested) = attr.parse_args::<syn::Ident>() else {
            return false;
        };
        nested == "opaque"
    });

    let type_name_str = name.to_string();

    let expanded = if is_opaque {
        // Opaque implementation - no serialization
        quote! {
            impl #impl_generics rgb_ecs_introspect::Introspectable for #name #ty_generics #where_clause {
                fn to_json(&self) -> serde_json::Value {
                    serde_json::Value::Null
                }

                fn from_json(_value: serde_json::Value) -> Result<Self, rgb_ecs_introspect::IntrospectError>
                where
                    Self: Sized,
                {
                    Err(rgb_ecs_introspect::IntrospectError::OpaqueComponent(
                        Self::type_name().to_string(),
                    ))
                }

                fn type_name() -> &'static str {
                    #type_name_str
                }

                fn full_type_name() -> &'static str {
                    core::any::type_name::<Self>()
                }

                fn is_opaque() -> bool {
                    true
                }

                fn schema() -> Option<serde_json::Value> {
                    None
                }
            }
        }
    } else {
        // Normal implementation - uses serde
        quote! {
            impl #impl_generics rgb_ecs_introspect::Introspectable for #name #ty_generics #where_clause {
                fn to_json(&self) -> serde_json::Value {
                    serde_json::to_value(self).unwrap_or(serde_json::Value::Null)
                }

                fn from_json(value: serde_json::Value) -> Result<Self, rgb_ecs_introspect::IntrospectError>
                where
                    Self: Sized,
                {
                    serde_json::from_value(value).map_err(|e| {
                        rgb_ecs_introspect::IntrospectError::DeserializationFailed {
                            component: Self::type_name().to_string(),
                            error: e.to_string(),
                        }
                    })
                }

                fn type_name() -> &'static str {
                    #type_name_str
                }

                fn full_type_name() -> &'static str {
                    core::any::type_name::<Self>()
                }

                fn is_opaque() -> bool {
                    false
                }

                fn schema() -> Option<serde_json::Value> {
                    // TODO: Could generate JSON schema from struct fields
                    None
                }
            }
        }
    };

    TokenStream::from(expanded)
}
