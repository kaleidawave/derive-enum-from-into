use std::{collections::HashMap, mem};

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{parse::Parse, Attribute, DataEnum, Error, Ident, Lifetime, Token, Type};

use crate::{get_unnamed_single_non_ignored_variants, Unique, VariantFieldEntry};

pub(super) fn derive_enum_into(
    r#enum: DataEnum,
    enum_name: Ident,
    top_attributes: Vec<Attribute>,
) -> proc_macro2::TokenStream {
    let mut unique_typed_fields = HashMap::<Type, Unique<VariantFieldEntry>>::new();
    for entry in get_unnamed_single_non_ignored_variants(&r#enum, "try_into_ignore") {
        unique_typed_fields
            .entry(entry.1.ty.clone())
            .and_modify(|entry| *entry = Unique::NonUnique)
            .or_insert(Unique::Unique(entry));
    }

    let references: References = top_attributes
        .iter()
        .find_map(|attr| {
            attr.path
                .is_ident("try_into_references")
                .then(|| attr.parse_args().unwrap())
        })
        .unwrap_or_default();

    // TODO generics
    unique_typed_fields
        .into_values()
        .filter_map(|possibly_unique_field| match possibly_unique_field {
            Unique::Unique(entry) => Some(entry),
            Unique::NonUnique => None,
        })
        .flat_map(|(variant_name, variant_type)| {
            let enum_name = enum_name.clone();
            let lifetime = Lifetime::new("'try_into_ref", Span::call_site());
            references
                .to_decorators(Some(lifetime.clone()))
                .map(move |reference| {
                    let lifetime_binding = if !reference.is_empty() {
                        Some(quote!(<#lifetime>))
                    } else {
                        None
                    };
                    quote! {
                        #[automatically_derived]
                        impl #lifetime_binding ::core::convert::TryInto<#reference #variant_type>
                        for #reference #enum_name {
                            type Error = ();

                            #[inline]
                            fn try_into(self) -> Result<#reference #variant_type, Self::Error> {
                                if let #enum_name::#variant_name(item) = self {
                                    Ok(item)
                                } else {
                                    Err(())
                                }
                            }
                        }
                    }
                })
        })
        .collect::<proc_macro2::TokenStream>()
}

#[repr(u8)]
#[derive(Copy, Clone)]
#[allow(dead_code)]
enum References {
    Owned = 0b0001,
    Shared = 0b0010,
    SharedMut = 0b0100,
    None = 0b0000,
    All = 0b0111,
}

impl Default for References {
    fn default() -> Self {
        Self::Owned
    }
}

impl Parse for References {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let flags = input
            .parse_terminated::<u8, Token![,]>(|input| {
                if input.peek(Token![ref]) || input.peek(Token![&]) {
                    input.parse::<proc_macro2::TokenTree>().unwrap();
                    if input.peek(Token![mut]) {
                        input.parse::<Token![mut]>().unwrap();
                        Ok(References::SharedMut as u8)
                    } else {
                        Ok(References::Shared as u8)
                    }
                } else {
                    if input.peek(Ident) && input.parse::<Ident>().unwrap().to_string() == "owned" {
                        Ok(References::Owned as u8)
                    } else {
                        Err(Error::new(input.span(), "expected 'ref' or '&'"))
                    }
                }
            })?
            .into_iter()
            .reduce(std::ops::BitOr::bitor)
            .unwrap_or_default();

        Ok(unsafe { mem::transmute(flags) })
    }
}

impl References {
    pub fn to_decorators(self, lifetime: Option<Lifetime>) -> impl Iterator<Item = TokenStream> {
        (0..3)
            .map(|bit_offset| 0b0001_u8 << bit_offset)
            .filter_map(move |power_of_two| {
                let flag = self as u8 & power_of_two;
                match unsafe { mem::transmute(flag) } {
                    References::Owned => Some(TokenStream::default()),
                    References::Shared => Some(quote!(&#lifetime)),
                    References::SharedMut => Some(quote!(&#lifetime mut)),
                    References::None => None,
                    _ => unreachable!(),
                }
            })
    }
}
