use std::collections::HashMap;

use quote::quote;
use syn::{Attribute, DataEnum, Field, Ident, Type};

use crate::{get_unnamed_single_non_ignored_variants, Unique, VariantFieldEntry};

pub(crate) fn derive_enum_from(
    r#enum: DataEnum,
    enum_name: Ident,
    _top_level_attributes: Vec<Attribute>,
) -> proc_macro2::TokenStream {
    let mut unique_typed_fields = HashMap::<Type, Unique<VariantFieldEntry>>::new();
    for entry in get_unnamed_single_non_ignored_variants(&r#enum, "from_ignore") {
        unique_typed_fields
            .entry(entry.1.ty.clone())
            .and_modify(|entry| *entry = Unique::NonUnique)
            .or_insert(Unique::Unique(entry));
    }
    // TODO generics
    unique_typed_fields
        .into_values()
        .filter_map(|possibly_unique_field| match possibly_unique_field {
            Unique::Unique(entry) => Some(entry),
            Unique::NonUnique => None,
        })
        .map(|(variant, Field { ty, .. })| {
            quote! {
                #[automatically_derived]
                impl ::core::convert::From<#ty> for #enum_name {
                    #[inline]
                    fn from(item: #ty) -> #enum_name {
                        #enum_name::#variant(item)
                    }
                }
            }
        })
        .collect::<proc_macro2::TokenStream>()
}
