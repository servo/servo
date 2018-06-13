/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cg;
use darling::util::Override;
use quote::{ToTokens, Tokens};
use syn::{self, Data, Path, WhereClause};
use synstructure::{BindingInfo, Structure, VariantInfo};

pub fn derive(mut input: syn::DeriveInput) -> Tokens {
    let mut where_clause = input.generics.where_clause.take();
    for param in input.generics.type_params() {
        cg::add_predicate(
            &mut where_clause,
            parse_quote!(#param: ::style_traits::ToCss),
        );
    }

    let input_attrs = cg::parse_input_attrs::<CssInputAttrs>(&input);
    if let Data::Enum(_) = input.data {
        assert!(input_attrs.function.is_none(), "#[css(function)] is not allowed on enums");
        assert!(!input_attrs.comma, "#[css(comma)] is not allowed on enums");
    }

    let match_body = {
        let s = Structure::new(&input);
        s.each_variant(|variant| {
            derive_variant_arm(variant, &mut where_clause)
        })
    };
    input.generics.where_clause = where_clause;

    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let mut impls = quote! {
        impl #impl_generics ::style_traits::ToCss for #name #ty_generics #where_clause {
            #[allow(unused_variables)]
            #[inline]
            fn to_css<W>(
                &self,
                dest: &mut ::style_traits::CssWriter<W>,
            ) -> ::std::fmt::Result
            where
                W: ::std::fmt::Write,
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

fn derive_variant_arm(
    variant: &VariantInfo,
    generics: &mut Option<WhereClause>,
) -> Tokens {
    let bindings = variant.bindings();
    let identifier = cg::to_css_identifier(variant.ast().ident.as_ref());
    let ast = variant.ast();
    let variant_attrs = cg::parse_variant_attrs_from_ast::<CssVariantAttrs>(&ast);
    let separator = if variant_attrs.comma { ", " } else { " " };

    if variant_attrs.skip {
        return quote!(Ok(()));
    }
    if variant_attrs.dimension {
        assert_eq!(bindings.len(), 1);
        assert!(
            variant_attrs.function.is_none() && variant_attrs.keyword.is_none(),
            "That makes no sense"
        );
    }

    let mut expr = if let Some(keyword) = variant_attrs.keyword {
        assert!(bindings.is_empty());
        quote! {
            ::std::fmt::Write::write_str(dest, #keyword)
        }
    } else if !bindings.is_empty() {
        derive_variant_fields_expr(bindings, generics, separator)
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
        let mut identifier = function.explicit().map_or(identifier, |name| name);
        identifier.push_str("(");
        expr = quote! {
            ::std::fmt::Write::write_str(dest, #identifier)?;
            #expr?;
            ::std::fmt::Write::write_str(dest, ")")
        }
    }
    expr
}

fn derive_variant_fields_expr(
    bindings: &[BindingInfo],
    where_clause: &mut Option<WhereClause>,
    separator: &str,
) -> Tokens {
    let mut iter = bindings.iter().filter_map(|binding| {
        let attrs = cg::parse_field_attrs::<CssFieldAttrs>(&binding.ast());
        if attrs.skip {
            return None;
        }
        Some((binding, attrs))
    }).peekable();

    let (first, attrs) = match iter.next() {
        Some(pair) => pair,
        None => return quote! { Ok(()) },
    };
    if !attrs.iterable && iter.peek().is_none() {
        if attrs.field_bound {
            let ty = &first.ast().ty;
            cg::add_predicate(where_clause, parse_quote!(#ty: ::style_traits::ToCss));
        }
        let mut expr = quote! { ::style_traits::ToCss::to_css(#first, dest) };
        if let Some(condition) = attrs.skip_if {
            expr = quote! {
                if !#condition(#first) {
                    #expr
                }
            }
        }
        return expr;
    }

    let mut expr = derive_single_field_expr(first, attrs, where_clause);
    for (binding, attrs) in iter {
        derive_single_field_expr(binding, attrs, where_clause).to_tokens(&mut expr)
    }

    quote! {{
        let mut writer = ::style_traits::values::SequenceWriter::new(dest, #separator);
        #expr
        Ok(())
    }}
}

fn derive_single_field_expr(
    field: &BindingInfo,
    attrs: CssFieldAttrs,
    where_clause: &mut Option<WhereClause>,
) -> Tokens {
    let mut expr = if attrs.iterable {
        if let Some(if_empty) = attrs.if_empty {
            return quote! {
                {
                    let mut iter = #field.iter().peekable();
                    if iter.peek().is_none() {
                        writer.raw_item(#if_empty)?;
                    } else {
                        for item in iter {
                            writer.item(&item)?;
                        }
                    }
                }
            };
        }
        quote! {
            for item in #field.iter() {
                writer.item(&item)?;
            }
        }
    } else if attrs.represents_keyword {
        let ident =
            field.ast().ident.as_ref().expect("Unnamed field with represents_keyword?");
        let ident = cg::to_css_identifier(ident.as_ref());
        quote! {
            if *#field {
                writer.raw_item(#ident)?;
            }
        }
    } else {
        if attrs.field_bound {
            let ty = &field.ast().ty;
            cg::add_predicate(where_clause, parse_quote!(#ty: ::style_traits::ToCss));
        }
        quote! { writer.item(#field)?; }
    };

    if let Some(condition) = attrs.skip_if {
        expr = quote! {
            if !#condition(#field) {
                #expr
            }
        }
    }

    expr
}

#[darling(attributes(css), default)]
#[derive(Default, FromDeriveInput)]
pub struct CssInputAttrs {
    pub derive_debug: bool,
    // Here because structs variants are also their whole type definition.
    pub function: Option<Override<String>>,
    // Here because structs variants are also their whole type definition.
    pub comma: bool,
}

#[darling(attributes(css), default)]
#[derive(Default, FromVariant)]
pub struct CssVariantAttrs {
    pub function: Option<Override<String>>,
    pub comma: bool,
    pub dimension: bool,
    pub keyword: Option<String>,
    pub skip: bool,
}

#[darling(attributes(css), default)]
#[derive(Default, FromField)]
pub struct CssFieldAttrs {
    pub if_empty: Option<String>,
    pub field_bound: bool,
    pub iterable: bool,
    pub skip: bool,
    pub represents_keyword: bool,
    pub skip_if: Option<Path>,
}
