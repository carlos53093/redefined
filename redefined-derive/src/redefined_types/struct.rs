use proc_macro2::TokenStream;
use quote::quote;
use syn::{self, parse::Parse, DataStruct, Field, Fields, Ident};

use crate::attributes::{symbol::FIELD_FN, type_attr::TypeAttribute, ContainerAttributes};

pub struct StructContainer {
    pub fields:   Vec<StructField>,
    pub is_named: bool,
}

impl StructContainer {
    pub fn parse_new(data_struct: &DataStruct) -> syn::Result<Self> {
        let (fields, is_named) = match &data_struct.fields {
            Fields::Named(fields_named) => (&fields_named.named, true),
            Fields::Unnamed(fields_unnamed) => (&fields_unnamed.unnamed, false),
            _ => return Err(syn::Error::new_spanned(&data_struct.fields, "Expected a struct with named fields")),
        };

        let struct_fields = if is_named {
            fields
                .iter()
                .flat_map(|field| {
                    field.ident.as_ref().map(|idt| {
                        let mut this = StructField::new(Some(idt.clone()), field.clone(), None);
                        this.parse_attributes_for_field()?;
                        Ok(this)
                    })
                })
                .collect::<syn::Result<Vec<_>>>()?
        } else {
            fields
                .iter()
                .enumerate()
                .map(|(unnamed_idx, field)| {
                    let mut this = StructField::new(None, field.clone(), Some(unnamed_idx));
                    this.parse_attributes_for_field()?;
                    Ok(this)
                })
                .collect::<syn::Result<Vec<_>>>()?
        };

        Ok(Self { fields: struct_fields, is_named })
    }

    pub fn from_source_tokens(&self) -> syn::Result<TokenStream> {
        let tokens = self
            .fields
            .iter()
            .map(|field| field.from_source_tokens())
            .collect::<syn::Result<Vec<_>>>()?;

        let gen = if self.is_named {
            quote! {
                Self {
                    #(#tokens)*
                }
            }
        } else {
            let tokens_combined = quote! { #(#tokens),* };
            quote! {
                Self(#tokens_combined)
            }
        };

        Ok(gen)
    }

    pub fn to_source_tokens(&self, source_type: &Ident) -> syn::Result<TokenStream> {
        let tokens = self
            .fields
            .iter()
            .map(|field| field.to_source_tokens())
            .collect::<syn::Result<Vec<_>>>()?;

        let gen = if self.is_named {
            quote! {
                #source_type {
                    #(#tokens)*
                }
            }
        } else {
            let tokens_combined = quote! { #(#tokens),* };
            quote! {
                #source_type(#tokens_combined)
            }
        };

        Ok(gen)
    }
}

pub struct StructField {
    pub ident:          Option<Ident>,
    pub field:          Field,
    pub field_attrs:    Vec<TypeAttribute>,
    pub is_unnamed_idx: Option<usize>,
}

impl StructField {
    pub fn new(ident: Option<Ident>, field: Field, is_unnamed_idx: Option<usize>) -> Self {
        Self { ident, field, field_attrs: Vec::new(), is_unnamed_idx }
    }

    pub fn parse_attributes_for_field(&mut self) -> syn::Result<()> {
        let mut attrs = Vec::new();
        for attr in &self.field.attrs {
            if attr.path().is_ident("redefined") {
                attrs.extend(attr.parse_args_with(ContainerAttributes::parse)?.0);
            }
        }
        self.field_attrs = attrs;

        Ok(())
    }

    pub fn from_source_tokens(&self) -> syn::Result<TokenStream> {
        let fields_attrs = &self.field_attrs;
        let ident = &self.ident;

        let gen = if FIELD_FN.contained_in(fields_attrs) {
            let attr = FIELD_FN
                .find_type_attr(fields_attrs)
                .ok_or(syn::Error::new_spanned(&self.ident, "FIELD FN ERROR"))?;

            let func_name = attr.nv_tokens.unwrap();

            quote! { #ident: redefined::RedefinedConvert::from_source(#func_name), }
        } else if fields_attrs.is_empty() {
            if let Some(idx) = self.is_unnamed_idx {
                let index = syn::Index::from(idx);
                quote! { redefined::RedefinedConvert::from_source(src.#index)}
            } else {
                quote! { #ident: redefined::RedefinedConvert::from_source(src.#ident),}
            }
        } else {
            unreachable!("cannot reach - should be no more field attrs: {:?}", fields_attrs);
        };

        Ok(gen)
    }

    pub fn to_source_tokens(&self) -> syn::Result<TokenStream> {
        let matched_field = &self.ident;

        let gen = if let Some(idx) = self.is_unnamed_idx {
            let index = syn::Index::from(idx);
            quote! { redefined::RedefinedConvert::to_source(self.#index) }
        } else {
            quote! { #matched_field: redefined::RedefinedConvert::to_source(self.#matched_field), }
        };

        Ok(gen)
    }
}
