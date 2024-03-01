use std::{collections::HashMap, mem};

use proc_macro2::Span;
use quote::quote;
use syn::{
    parse::Parse, parse_quote, token::Comma, Attribute, DataEnum, Error, GenericArgument,
    GenericParam, Generics, Ident, Lifetime, LifetimeParam, Path, PathArguments, Token, Type,
    TypePath, TypeReference,
};

use crate::{get_unnamed_single_non_ignored_variants, Unique, VariantFieldEntry};

pub(super) fn derive_enum_into(
    r#enum: DataEnum,
    name: Ident,
    top_attributes: Vec<Attribute>,
    generics: Generics,
) -> proc_macro2::TokenStream {
    let mut unique_typed_fields = HashMap::<Type, Unique<VariantFieldEntry>>::new();

    for entry in get_unnamed_single_non_ignored_variants(&r#enum, "try_into_ignore") {
        unique_typed_fields
            .entry(entry.1.ty.clone())
            .and_modify(|entry| *entry = Unique::NonUnique)
            .or_insert(Unique::Unique(entry));
    }

    let references: ReferenceConfig = top_attributes
        .iter()
        .find_map(|attr| {
            attr.path()
                .is_ident("try_into_references")
                .then(|| attr.parse_args().unwrap())
        })
        .unwrap_or_default();

    let Generics {
        params,
        where_clause,
        ..
    } = generics;

    let mut enum_path: Path = parse_quote!(#name);
    if !params.is_empty() {
        let args: syn::punctuated::Punctuated<GenericArgument, Comma> =
            super::type_args_from_parameters(params.iter()).collect();

        enum_path.segments.last_mut().unwrap().arguments =
            PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
                colon2_token: None,
                lt_token: Token![<](proc_macro2::Span::call_site()),
                args,
                gt_token: Token![>](proc_macro2::Span::call_site()),
            });
    }
    let enum_self_ty = Type::Path(TypePath {
        path: enum_path,
        qself: None,
    });

    let lifetime = Lifetime::new("'try_into_ref", Span::call_site());

    unique_typed_fields
        .into_values()
        .filter_map(|possibly_unique_field| match possibly_unique_field {
            Unique::Unique(entry) => Some(entry),
            Unique::NonUnique => None,
        })
        .flat_map(|(variant_name, variant_type)| {
            let where_clause = where_clause.clone();
            let params = params.clone();
            let enum_self_ty = enum_self_ty.clone();
            let name = name.clone();

            references.to_decorators(lifetime.clone()).map(move |decorator| {
                let mut params = params.clone();
                if let Some(param) = decorator.as_parameter() {
                    params.push(param);
                }
                let into_arg = decorator.wrap_argument(variant_type.ty.clone());
                let over_arg = decorator.wrap_argument(enum_self_ty.clone());

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
                    impl #lt_token #params #gt_token ::core::convert::TryInto<#into_arg> for #over_arg #where_clause {
                        type Error = Self;

                        #[inline]
                        fn try_into(self) -> Result<#into_arg, Self::Error> {
                            if let #name::#variant_name(item) = self {
                                Ok(item)
                            } else {
                                Err(self)
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
enum ReferenceConfig {
    Owned = 0b0001,
    Shared = 0b0010,
    SharedMut = 0b0100,
    None = 0b0000,
    All = 0b0111,
}

impl Default for ReferenceConfig {
    fn default() -> Self {
        Self::Owned
    }
}

impl Parse for ReferenceConfig {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let flags = input
            .parse_terminated(
                |input| {
                    if input.peek(Token![ref]) || input.peek(Token![&]) {
                        input.parse::<proc_macro2::TokenTree>().unwrap();
                        if input.peek(Token![mut]) {
                            input.parse::<Token![mut]>().unwrap();
                            Ok(ReferenceConfig::SharedMut)
                        } else {
                            Ok(ReferenceConfig::Shared)
                        }
                    } else if input.peek(Ident) && input.parse::<Ident>().unwrap() == "owned" {
                        Ok(ReferenceConfig::Owned)
                    } else {
                        Err(Error::new(input.span(), "expected 'ref', '&' or 'owned'"))
                    }
                },
                Token![,],
            )?
            .into_iter()
            .map(|member| member as u8)
            .reduce(std::ops::BitOr::bitor)
            .unwrap_or_default();

        Ok(unsafe { mem::transmute(flags) })
    }
}

impl ReferenceConfig {
    /// Decorators = `&'a` and `&'a mut`
    pub fn to_decorators(
        self,
        lifetime_name: Lifetime,
    ) -> impl Iterator<Item = ReferenceDecorator> {
        (0..3)
            .map(|bit_offset| 0b0001_u8 << bit_offset)
            .filter_map(move |power_of_two| {
                let flag = self as u8 & power_of_two;
                match unsafe { mem::transmute(flag) } {
                    ReferenceConfig::Owned => Some(ReferenceDecorator::Owned),
                    ReferenceConfig::Shared => {
                        Some(ReferenceDecorator::Shared(lifetime_name.clone()))
                    }
                    ReferenceConfig::SharedMut => {
                        Some(ReferenceDecorator::SharedMut(lifetime_name.clone()))
                    }
                    ReferenceConfig::None => None,
                    _ => unreachable!(),
                }
            })
    }
}

enum ReferenceDecorator {
    Owned,
    Shared(Lifetime),
    SharedMut(Lifetime),
}

impl ReferenceDecorator {
    fn as_parameter(&self) -> Option<GenericParam> {
        match self {
            ReferenceDecorator::Owned => None,
            ReferenceDecorator::Shared(lifetime_name)
            | ReferenceDecorator::SharedMut(lifetime_name) => Some(GenericParam::Lifetime(
                LifetimeParam::new(lifetime_name.clone()),
            )),
        }
    }

    fn wrap_argument(&self, over: Type) -> Type {
        match self {
            ReferenceDecorator::Owned => over,
            ref_dec @ ReferenceDecorator::Shared(lifetime_name)
            | ref_dec @ ReferenceDecorator::SharedMut(lifetime_name) => {
                TypeReference {
                    and_token: Token![&](Span::call_site()),
                    lifetime: Some(lifetime_name.clone()),
                    mutability: matches!(ref_dec, ReferenceDecorator::SharedMut(_))
                        .then_some(Token![mut](Span::call_site())),
                    elem: Box::new(over),
                }
                .into()
            }
        }
    }
}
