/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cg;
use darling::{Error, FromMetaItem};
use quote::Tokens;
use syn::{self, DeriveInput, Ident};
use synstructure;

pub fn derive(input: DeriveInput) -> Tokens {
    let name = &input.ident;
    let trait_path = &["style_traits", "ToCss"];
    let (impl_generics, ty_generics, mut where_clause) =
        cg::trait_parts(&input, trait_path);

    let input_attrs = cg::parse_input_attrs::<CssInputAttrs>(&input);
    let style = synstructure::BindStyle::Ref.into();
    let match_body = synstructure::each_variant(&input, &style, |bindings, variant| {
        let identifier = cg::to_css_identifier(variant.ident.as_ref());
        let variant_attrs = cg::parse_variant_attrs::<CssVariantAttrs>(variant);
        let separator = if variant_attrs.comma { ", " } else { " " };

        if variant_attrs.dimension {
            assert_eq!(bindings.len(), 1);
            assert!(variant_attrs.function.is_none(), "That makes no sense");
        }

        let mut expr = if !bindings.is_empty() {
            let mut expr = quote! {};
            if variant_attrs.iterable {
                assert_eq!(bindings.len(), 1);
                let binding = &bindings[0];
                expr = quote! {
                    #expr

                    for item in #binding.iter() {
                        writer.item(item)?;
                    }
                };
            } else {
                for binding in bindings {
                    let attrs = cg::parse_field_attrs::<CssFieldAttrs>(&binding.field);
                    if !attrs.ignore_bound {
                        where_clause.add_trait_bound(&binding.field.ty);
                    }
                    expr = quote! {
                        #expr
                        writer.item(#binding)?;
                    };
                }
            }

            quote! {{
                let mut writer = ::style_traits::values::SequenceWriter::new(dest, #separator);
                #expr
                Ok(())
            }}
        } else {
            quote! {
                ::std::fmt::Write::write_str(dest, #identifier)
            }
        };

        if variant_attrs.dimension {
            expr = quote! {
                #expr?;
                ::std::fmt::Write::write_str(dest, #identifier)
            }
        } else if let Some(function) = variant_attrs.function {
            let mut identifier = function.name.map_or(identifier, |name| name.to_string());
            identifier.push_str("(");
            expr = quote! {
                ::std::fmt::Write::write_str(dest, #identifier)?;
                #expr?;
                ::std::fmt::Write::write_str(dest, ")")
            }
        }
        Some(expr)
    });

    let mut impls = quote! {
        impl #impl_generics ::style_traits::ToCss for #name #ty_generics #where_clause {
            #[allow(unused_variables)]
            #[inline]
            fn to_css<W>(
                &self,
                dest: &mut ::style_traits::CssWriter<W>,
            ) -> ::std::fmt::Result
            where
                W: ::std::fmt::Write
            {
                match *self {
                    #match_body
                }
            }
        }
    };

    if input_attrs.derive_debug {
        impls.append(quote! {
            impl #impl_generics ::std::fmt::Debug for #name #ty_generics #where_clause {
                fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                    ::style_traits::ToCss::to_css(
                        self,
                        &mut ::style_traits::CssWriter::new(f),
                    )
                }
            }
        });
    }

    impls
}

#[darling(attributes(css), default)]
#[derive(Default, FromDeriveInput)]
struct CssInputAttrs {
    derive_debug: bool,
    function: Option<Function>,
    comma: bool,
}

#[darling(attributes(css), default)]
#[derive(Default, FromVariant)]
struct CssVariantAttrs {
    function: Option<Function>,
    iterable: bool,
    comma: bool,
    dimension: bool,
}

#[darling(attributes(css), default)]
#[derive(Default, FromField)]
struct CssFieldAttrs {
    ignore_bound: bool,
}

struct Function {
    name: Option<Ident>,
}

impl FromMetaItem for Function {
    fn from_word() -> Result<Self, Error> {
        Ok(Self { name: None })
    }

    fn from_string(name: &str) -> Result<Self, Error> {
        let name = syn::parse_ident(name).map_err(Error::custom)?;
        Ok(Self { name: Some(name) })
    }
}
