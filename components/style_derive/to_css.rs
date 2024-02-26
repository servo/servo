/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use darling::util::Override;
use derive_common::cg;
use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, TokenStreamExt};
use syn::{self, Data, Ident, Path, WhereClause};
use synstructure::{BindingInfo, Structure, VariantInfo};

fn derive_bitflags(input: &syn::DeriveInput, bitflags: &CssBitflagAttrs) -> TokenStream {
    let name = &input.ident;
    let mut body = TokenStream::new();
    for (rust_name, css_name) in bitflags.single_flags() {
        let rust_ident = Ident::new(&rust_name, Span::call_site());
        body.append_all(quote! {
            if *self == Self::#rust_ident {
                return dest.write_str(#css_name);
            }
        });
    }

    body.append_all(quote! {
        let mut has_any = false;
    });

    if bitflags.overlapping_bits {
        body.append_all(quote! {
            let mut serialized = Self::empty();
        });
    }

    for (rust_name, css_name) in bitflags.mixed_flags() {
        let rust_ident = Ident::new(&rust_name, Span::call_site());
        let serialize = quote! {
            if has_any {
                dest.write_char(' ')?;
            }
            has_any = true;
            dest.write_str(#css_name)?;
        };
        if bitflags.overlapping_bits {
            body.append_all(quote! {
                if self.contains(Self::#rust_ident) && !serialized.intersects(Self::#rust_ident) {
                    #serialize
                    serialized.insert(Self::#rust_ident);
                }
            });
        } else {
            body.append_all(quote! {
                if self.intersects(Self::#rust_ident) {
                    #serialize
                }
            });
        }
    }

    body.append_all(quote! {
        Ok(())
    });

    quote! {
        impl style_traits::ToCss for #name {
            #[allow(unused_variables)]
            #[inline]
            fn to_css<W>(
                &self,
                dest: &mut style_traits::CssWriter<W>,
            ) -> std::fmt::Result
            where
                W: std::fmt::Write,
            {
                #body
            }
        }
    }
}

pub fn derive(mut input: syn::DeriveInput) -> TokenStream {
    let mut where_clause = input.generics.where_clause.take();
    for param in input.generics.type_params() {
        cg::add_predicate(&mut where_clause, parse_quote!(#param: style_traits::ToCss));
    }

    let input_attrs = cg::parse_input_attrs::<CssInputAttrs>(&input);
    if matches!(input.data, Data::Enum(..)) || input_attrs.bitflags.is_some() {
        assert!(
            input_attrs.function.is_none(),
            "#[css(function)] is not allowed on enums or bitflags"
        );
        assert!(
            !input_attrs.comma,
            "#[css(comma)] is not allowed on enums or bitflags"
        );
    }

    if let Some(ref bitflags) = input_attrs.bitflags {
        assert!(
            !input_attrs.derive_debug,
            "Bitflags can derive debug on their own"
        );
        assert!(where_clause.is_none(), "Generic bitflags?");
        return derive_bitflags(&input, bitflags);
    }

    let match_body = {
        let s = Structure::new(&input);
        s.each_variant(|variant| derive_variant_arm(variant, &mut where_clause))
    };
    input.generics.where_clause = where_clause;

    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let mut impls = quote! {
        impl #impl_generics style_traits::ToCss for #name #ty_generics #where_clause {
            #[allow(unused_variables)]
            #[inline]
            fn to_css<W>(
                &self,
                dest: &mut style_traits::CssWriter<W>,
            ) -> std::fmt::Result
            where
                W: std::fmt::Write,
            {
                match *self {
                    #match_body
                }
            }
        }
    };

    if input_attrs.derive_debug {
        impls.append_all(quote! {
            impl #impl_generics std::fmt::Debug for #name #ty_generics #where_clause {
                fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                    style_traits::ToCss::to_css(
                        self,
                        &mut style_traits::CssWriter::new(f),
                    )
                }
            }
        });
    }

    impls
}

fn derive_variant_arm(variant: &VariantInfo, generics: &mut Option<WhereClause>) -> TokenStream {
    let bindings = variant.bindings();
    let identifier = cg::to_css_identifier(&variant.ast().ident.to_string());
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
            std::fmt::Write::write_str(dest, #keyword)
        }
    } else if !bindings.is_empty() {
        derive_variant_fields_expr(bindings, generics, separator)
    } else {
        quote! {
            std::fmt::Write::write_str(dest, #identifier)
        }
    };

    if variant_attrs.dimension {
        expr = quote! {
            #expr?;
            std::fmt::Write::write_str(dest, #identifier)
        }
    } else if let Some(function) = variant_attrs.function {
        let mut identifier = function.explicit().map_or(identifier, |name| name);
        identifier.push('(');
        expr = quote! {
            std::fmt::Write::write_str(dest, #identifier)?;
            #expr?;
            std::fmt::Write::write_str(dest, ")")
        }
    }
    expr
}

fn derive_variant_fields_expr(
    bindings: &[BindingInfo],
    where_clause: &mut Option<WhereClause>,
    separator: &str,
) -> TokenStream {
    let mut iter = bindings
        .iter()
        .filter_map(|binding| {
            let attrs = cg::parse_field_attrs::<CssFieldAttrs>(&binding.ast());
            if attrs.skip {
                return None;
            }
            Some((binding, attrs))
        })
        .peekable();

    let (first, attrs) = match iter.next() {
        Some(pair) => pair,
        None => return quote! { Ok(()) },
    };
    if attrs.field_bound {
        let ty = &first.ast().ty;
        // TODO(emilio): IntoIterator might not be enough for every type of
        // iterable thing (like ArcSlice<> or what not). We might want to expose
        // an `item = "T"` attribute to handle that in the future.
        let predicate = if attrs.iterable {
            parse_quote!(<#ty as IntoIterator>::Item: style_traits::ToCss)
        } else {
            parse_quote!(#ty: style_traits::ToCss)
        };
        cg::add_predicate(where_clause, predicate);
    }
    if !attrs.iterable && iter.peek().is_none() {
        let mut expr = quote! { style_traits::ToCss::to_css(#first, dest) };
        if let Some(condition) = attrs.skip_if {
            expr = quote! {
                if !#condition(#first) {
                    #expr
                }
            }
        }

        if let Some(condition) = attrs.contextual_skip_if {
            expr = quote! {
                if !#condition(#(#bindings), *) {
                    #expr
                }
            }
        }
        return expr;
    }

    let mut expr = derive_single_field_expr(first, attrs, where_clause, bindings);
    for (binding, attrs) in iter {
        derive_single_field_expr(binding, attrs, where_clause, bindings).to_tokens(&mut expr)
    }

    quote! {{
        let mut writer = style_traits::values::SequenceWriter::new(dest, #separator);
        #expr
        Ok(())
    }}
}

fn derive_single_field_expr(
    field: &BindingInfo,
    attrs: CssFieldAttrs,
    where_clause: &mut Option<WhereClause>,
    bindings: &[BindingInfo],
) -> TokenStream {
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
        let ident = field
            .ast()
            .ident
            .as_ref()
            .expect("Unnamed field with represents_keyword?");
        let ident = cg::to_css_identifier(&ident.to_string()).replace("_", "-");
        quote! {
            if *#field {
                writer.raw_item(#ident)?;
            }
        }
    } else {
        if attrs.field_bound {
            let ty = &field.ast().ty;
            cg::add_predicate(where_clause, parse_quote!(#ty: style_traits::ToCss));
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

    if let Some(condition) = attrs.contextual_skip_if {
        expr = quote! {
            if !#condition(#(#bindings), *) {
                #expr
            }
        }
    }

    expr
}

#[derive(Default, FromMeta)]
#[darling(default)]
pub struct CssBitflagAttrs {
    /// Flags that can only go on their own, comma-separated.
    pub single: Option<String>,
    /// Flags that can go mixed with each other, comma-separated.
    pub mixed: Option<String>,
    /// Extra validation of the resulting mixed flags.
    pub validate_mixed: Option<Path>,
    /// Whether there are overlapping bits we need to take care of when
    /// serializing.
    pub overlapping_bits: bool,
}

impl CssBitflagAttrs {
    /// Returns a vector of (rust_name, css_name) of a given flag list.
    fn names(s: &Option<String>) -> Vec<(String, String)> {
        let s = match s {
            Some(s) => s,
            None => return vec![],
        };
        s.split(',')
            .map(|css_name| (cg::to_scream_case(css_name), css_name.to_owned()))
            .collect()
    }

    pub fn single_flags(&self) -> Vec<(String, String)> {
        Self::names(&self.single)
    }

    pub fn mixed_flags(&self) -> Vec<(String, String)> {
        Self::names(&self.mixed)
    }
}

#[derive(Default, FromDeriveInput)]
#[darling(attributes(css), default)]
pub struct CssInputAttrs {
    pub derive_debug: bool,
    // Here because structs variants are also their whole type definition.
    pub function: Option<Override<String>>,
    // Here because structs variants are also their whole type definition.
    pub comma: bool,
    pub bitflags: Option<CssBitflagAttrs>,
}

#[derive(Default, FromVariant)]
#[darling(attributes(css), default)]
pub struct CssVariantAttrs {
    pub function: Option<Override<String>>,
    // Here because structs variants are also their whole type definition.
    pub derive_debug: bool,
    pub comma: bool,
    pub bitflags: Option<CssBitflagAttrs>,
    pub dimension: bool,
    pub keyword: Option<String>,
    pub skip: bool,
}

#[derive(Default, FromField)]
#[darling(attributes(css), default)]
pub struct CssFieldAttrs {
    pub if_empty: Option<String>,
    pub field_bound: bool,
    pub iterable: bool,
    pub skip: bool,
    pub represents_keyword: bool,
    pub contextual_skip_if: Option<Path>,
    pub skip_if: Option<Path>,
}
