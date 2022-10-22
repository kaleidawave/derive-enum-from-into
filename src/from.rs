use std::collections::HashMap;

use proc_macro2::Span;
use quote::quote;
use syn::{
    parse_quote, token::Comma, DataEnum, Field, GenericArgument, Generics, Ident, Path,
    PathArguments, Token, Type,
};

use crate::{get_unnamed_single_non_ignored_variants, Unique, VariantFieldEntry};

pub(crate) fn derive_enum_from(
    r#enum: DataEnum,
    name: Ident,
    generics: Generics,
) -> proc_macro2::TokenStream {
    let mut unique_typed_fields = HashMap::<Type, Unique<VariantFieldEntry>>::new();

    for entry in get_unnamed_single_non_ignored_variants(&r#enum, "from_ignore") {
        unique_typed_fields
            .entry(entry.1.ty.clone())
            .and_modify(|entry| *entry = Unique::NonUnique)
            .or_insert(Unique::Unique(entry));
    }

    let Generics {
        params,
        where_clause,
        ..
    } = generics;

    let mut enum_self: Path = parse_quote!(#name);
    if !params.is_empty() {
        let args: syn::punctuated::Punctuated<GenericArgument, Comma> =
            super::type_args_from_parameters(params.iter()).collect();

        enum_self.segments.last_mut().unwrap().arguments =
            PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
                colon2_token: None,
                lt_token: Token![<](proc_macro2::Span::call_site()),
                args,
                gt_token: Token![>](proc_macro2::Span::call_site()),
            });
    }

    unique_typed_fields
        .into_values()
        .filter_map(|possibly_unique_field| match possibly_unique_field {
            Unique::Unique(entry) => Some(entry),
            Unique::NonUnique => None,
        })
        .map(|(variant, Field { ty, .. })| {
            let (lt_token, gt_token) = if params.is_empty() {
                (None, None)
            } else {
                (
                    Some(Token![<](Span::call_site())),
                    Some(Token![>](Span::call_site())),
                )
            };
            quote! {
                #[automatically_derived]
                impl #lt_token #params #gt_token ::core::convert::From<#ty> for #enum_self #where_clause {
                    #[inline]
                    fn from(item: #ty) -> Self {
                        Self::#variant(item)
                    }
                }
            }
        })
        .collect::<proc_macro2::TokenStream>()
}
