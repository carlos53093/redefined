use proc_macro2::TokenStream;
use quote::quote;
use syn::{self, Attribute, DataEnum, Fields, Generics, Ident, Variant, Visibility};

use super::parse_attributes;
use crate::new_types::r#struct::parse_field;

pub fn parse_new_enum(
    data_enum: &DataEnum,
    enum_name: &Ident,
    new_enum_name: &Ident,
    generics: &Generics,
    visibility: &Visibility,
    attributes: &[Attribute],
    generics_skip_remote: &[Ident],
) -> syn::Result<TokenStream> {
    let enum_fields = data_enum
        .variants
        .iter()
        .map(|variant| parse_enum_variant(variant, generics_skip_remote))
        .collect::<syn::Result<Vec<_>>>()?;

    let (derive_attrs, container_attrs, new_attrs) = parse_attributes(attributes, enum_name.span())?;

    let tokens = quote! {
        #[derive(#(#derive_attrs),*)]
        #[redefined(#enum_name)]
        #(#container_attrs)*
        #(#new_attrs)*
        #visibility enum #new_enum_name #generics {
            #(#enum_fields),*
        }
    };

    //panic!("TOKENS: {:?}", tokens.to_string());

    Ok(tokens)
}

fn parse_enum_variant(variant: &Variant, generics_skip_remote: &[Ident]) -> syn::Result<TokenStream> {
    let discriminant = &variant.discriminant;
    let ident = &variant.ident;
    let mut copied_field_attrs = Vec::new();

    let fields = match &variant.fields {
        Fields::Named(fields) => {
            let f = fields
                .named
                .iter()
                .map(|f| parse_field(f, generics_skip_remote))
                .collect::<Result<Vec<_>, _>>()?;
            quote! { {#(#f),* }}
        }
        Fields::Unnamed(fields) => {
            let f = fields
                .unnamed
                .iter()
                .map(|f| parse_field(f, generics_skip_remote))
                .collect::<Result<Vec<_>, _>>()?;
            quote! { (#(#f),*)}
        }
        Fields::Unit => Default::default(),
    };

    for attr in &variant.attrs {
        if !attr.path().is_ident("redefined") {
            copied_field_attrs.push(attr)
        }
    }

    let tokens = if let Some((eq, expr)) = discriminant {
        quote! {
            #(#copied_field_attrs)*
            #ident #eq #expr,
        }
    } else {
        quote! {
            #(#copied_field_attrs)*
            #ident #fields
        }
    };

    Ok(tokens)
}
