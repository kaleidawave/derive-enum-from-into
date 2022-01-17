#![doc = include_str!("../README.md")]
mod from;
mod into;

use proc_macro2::Ident;
use quote::quote;
use syn::{Data, DataEnum, DeriveInput, Field, Fields};

type VariantFieldEntry = (Ident, Field);

#[proc_macro_derive(EnumFrom, attributes(from_ignore))]
pub fn derive_from(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let derive_input = syn::parse::<DeriveInput>(input).unwrap();
    if let Data::Enum(r#enum) = derive_input.data {
        from::derive_enum_from(r#enum, derive_input.ident, derive_input.attrs).into()
    } else {
        quote!( compile_error!("Can only derive EnumFrom on enums"); ).into()
    }
}

#[proc_macro_derive(EnumTryInto, attributes(try_into_references, try_into_ignore))]
pub fn derive_enum_into(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let derive_input = syn::parse::<DeriveInput>(input).unwrap();
    if let Data::Enum(r#enum) = derive_input.data {
        into::derive_enum_into(r#enum, derive_input.ident, derive_input.attrs).into()
    } else {
        quote!( compile_error!("Can only derive EnumTryInto on enums"); ).into()
    }
}

pub(crate) enum Unique<T> {
    Unique(T),
    NonUnique,
}

/// Gets variants with a single field without a specified *ignore* attribute
fn get_unnamed_single_non_ignored_variants<'a>(
    r#enum: &'a DataEnum,
    ignore_ident: &'a str,
) -> impl Iterator<Item = VariantFieldEntry> + 'a {
    r#enum.variants.iter().filter_map(move |variant| {
        if variant
            .attrs
            .iter()
            .any(|attr| attr.path.is_ident(ignore_ident))
        {
            return None;
        }
        if let Fields::Unnamed(unnamed_fields) = &variant.fields {
            if unnamed_fields.unnamed.len() == 1 {
                return Some((
                    variant.ident.clone(),
                    unnamed_fields.unnamed.first().unwrap().clone(),
                ));
            }
        }
        None
    })
}

#[cfg(test)]
mod test {
    use quote::quote;
    use syn::{parse_quote, DataEnum, DeriveInput};

    fn dissect_input(input: DeriveInput) -> (DataEnum, proc_macro2::Ident, Vec<syn::Attribute>) {
        if let syn::Data::Enum(data_enum) = input.data {
            (data_enum, input.ident, input.attrs)
        } else {
            unreachable!();
        }
    }

    #[test]
    fn duplicate_type_fields_detection() {
        let input: DeriveInput = parse_quote! {
            enum X {
                A(i32),
                B(String),
                C(String),
            }
        };
        let (enum1, name, attributes) = dissect_input(input);

        let result = crate::from::derive_enum_from(enum1, name, attributes);
        assert_eq!(
            result.to_string(),
            quote! {
                #[automatically_derived]
                impl ::core::convert::From<i32> for X {
                    #[inline]
                    fn from(item: i32) -> X {
                        X::A(item)
                    }
                }
            }
            .to_string()
        )
    }
}
