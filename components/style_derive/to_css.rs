/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cg;
use darling::util::Override;
use quote::Tokens;
use syn::{self, Ident};
use synstructure;

pub fn derive(input: syn::DeriveInput) -> Tokens {
    let name = &input.ident;
    let trait_path = parse_quote!(style_traits::ToCss);
    let (impl_generics, ty_generics, mut where_clause) =
        cg::trait_parts(&input, &trait_path);

    let input_attrs = cg::parse_input_attrs::<CssInputAttrs>(&input);
    let s = synstructure::Structure::new(&input);

    let match_body = s.each_variant(|variant| {
        let bindings = variant.bindings().into_iter().filter(|binding| {
            !cg::parse_field_attrs::<CssFieldAttrs>(&binding.ast()).skip
        }).collect::<Vec<_>>();
        let identifier = cg::to_css_identifier(variant.ast().ident.as_ref());
        let ast = variant.ast();
        let variant_attrs = cg::parse_variant_attrs::<CssVariantAttrs>(&ast);
        let separator = if variant_attrs.comma { ", " } else { " " };

        if variant_attrs.dimension {
            assert_eq!(bindings.len(), 1);
            assert!(
                variant_attrs.function.is_none() && variant_attrs.keyword.is_none(),
                "That makes no sense"
            );
        }

        let mut expr = if let Some(keyword) = variant_attrs.keyword {
            assert!(bindings.is_empty());
            let keyword = keyword.to_string();
            quote! {
                ::std::fmt::Write::write_str(dest, #keyword)
            }
        } else if !bindings.is_empty() {
            let mut expr = quote! {};
            if variant_attrs.iterable {
                assert_eq!(bindings.len(), 1);
                let binding = &bindings[0];
                expr = quote! {
                    #expr

                    for item in #binding.iter() {
                        writer.item(&item)?;
                    }
                };
            } else {
                for binding in bindings {
                    let attrs = cg::parse_field_attrs::<CssFieldAttrs>(&binding.ast());
                    if !attrs.ignore_bound {
                        where_clause.add_trait_bound(&binding.ast().ty);
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
            let mut identifier = function.explicit().map_or(identifier, |name| name.to_string());
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
        impls.append_all(quote! {
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
    // Here because structs variants are also their whole type definition.
    function: Option<Override<Ident>>,
    // Here because structs variants are also their whole type definition.
    comma: bool,
    // Here because structs variants are also their whole type definition.
    iterable: bool,
}

#[darling(attributes(css), default)]
#[derive(Default, FromVariant)]
pub struct CssVariantAttrs {
    pub function: Option<Override<Ident>>,
    pub iterable: bool,
    pub comma: bool,
    pub dimension: bool,
    pub keyword: Option<String>,
    pub aliases: Option<String>,
}

#[darling(attributes(css), default)]
#[derive(Default, FromField)]
struct CssFieldAttrs {
    ignore_bound: bool,
    skip: bool,
}
