use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, parse_macro_input};

#[proc_macro_derive(Encode)]
pub fn derive_encode(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let encode_body = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => {
                let field_encodes = fields.named.iter().map(|f| {
                    let field_name = &f.ident;
                    quote! {
                        mc_protocol::Encode::encode(&self.#field_name, writer)?;
                    }
                });
                quote! {
                    #(#field_encodes)*
                    Ok(())
                }
            }
            Fields::Unnamed(fields) => {
                let field_encodes = (0..fields.unnamed.len()).map(|i| {
                    let index = syn::Index::from(i);
                    quote! {
                        mc_protocol::Encode::encode(&self.#index, writer)?;
                    }
                });
                quote! {
                    #(#field_encodes)*
                    Ok(())
                }
            }
            Fields::Unit => {
                quote! { Ok(()) }
            }
        },
        Data::Enum(_) => {
            quote! {
                compile_error!("Encode derive does not support enums yet")
            }
        }
        Data::Union(_) => {
            quote! {
                compile_error!("Encode derive does not support unions")
            }
        }
    };

    let expanded = quote! {
        impl #impl_generics mc_protocol::Encode for #name #ty_generics #where_clause {
            fn encode<W: std::io::Write>(&self, writer: &mut W) -> mc_protocol::Result<()> {
                #encode_body
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(Decode)]
pub fn derive_decode(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // For Decode, we need a lifetime. Check if there's one already.
    let has_lifetime = generics.lifetimes().count() > 0;

    let decode_body = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => {
                let field_decodes = fields.named.iter().map(|f| {
                    let field_name = &f.ident;
                    let field_ty = &f.ty;
                    quote! {
                        #field_name: <#field_ty as mc_protocol::Decode>::decode(reader)?,
                    }
                });
                quote! {
                    Ok(Self {
                        #(#field_decodes)*
                    })
                }
            }
            Fields::Unnamed(fields) => {
                let field_decodes = fields.unnamed.iter().map(|f| {
                    let field_ty = &f.ty;
                    quote! {
                        <#field_ty as mc_protocol::Decode>::decode(reader)?,
                    }
                });
                quote! {
                    Ok(Self(#(#field_decodes)*))
                }
            }
            Fields::Unit => {
                quote! { Ok(Self) }
            }
        },
        Data::Enum(_) => {
            quote! {
                compile_error!("Decode derive does not support enums yet")
            }
        }
        Data::Union(_) => {
            quote! {
                compile_error!("Decode derive does not support unions")
            }
        }
    };

    let expanded = if has_lifetime {
        quote! {
            impl #impl_generics mc_protocol::Decode<'a> for #name #ty_generics #where_clause {
                fn decode<R: std::io::Read>(reader: &mut R) -> mc_protocol::Result<Self> {
                    #decode_body
                }
            }
        }
    } else {
        quote! {
            impl mc_protocol::Decode<'_> for #name {
                fn decode<R: std::io::Read>(reader: &mut R) -> mc_protocol::Result<Self> {
                    #decode_body
                }
            }
        }
    };

    TokenStream::from(expanded)
}
