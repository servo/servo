// Copyright 2019 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Ident, Index, TraitBound};
use synstructure::{decl_derive, Structure, BindStyle, AddBounds};
use unicode_xid::UnicodeXID;

// Internal method for sanitizing an identifier for hygiene purposes.
fn sanitize_ident(s: &str) -> Ident {
    let mut res = String::with_capacity(s.len());
    for mut c in s.chars() {
        if !UnicodeXID::is_xid_continue(c) {
            c = '_'
        }
        // Deduplicate consecutive _ characters.
        if res.ends_with('_') && c == '_' {
            continue;
        }
        res.push(c);
    }
    Ident::new(&res, Span::call_site())
}

/// Calculates size type for number of variants (used for enums)
fn get_discriminant_size_type(len: usize) -> TokenStream {
    if len <= <u8>::max_value() as usize {
        quote! { u8 }
    } else if len <= <u16>::max_value() as usize {
        quote! { u16 }
    } else {
        quote! { u32 }
    }
}

fn is_struct(s: &Structure) -> bool {
    // a single variant with no prefix is 'struct'
    match &s.variants()[..] {
        [v] if v.prefix.is_none() => true,
        _ => false,
    }
}

fn derive_max_size(s: &Structure) -> TokenStream {
    let max_size = s.variants().iter().fold(quote!(0), |acc, vi| {
        let variant_size = vi.bindings().iter().fold(quote!(0), |acc, bi| {
            // compute size of each variant by summing the sizes of its bindings
            let ty = &bi.ast().ty;
            quote!(#acc + <#ty>::max_size())
        });

        // find the maximum of each variant
        quote! {
            max(#acc, #variant_size)
        }
    });

    let body = if is_struct(s) {
        max_size
    } else {
        let discriminant_size_type = get_discriminant_size_type(s.variants().len());
        quote! {
            #discriminant_size_type ::max_size() + #max_size
        }
    };

    quote! {
        #[inline(always)]
        fn max_size() -> usize {
            use std::cmp::max;
            #body
        }
    }
}

fn derive_peek_from_for_enum(s: &mut Structure) -> TokenStream {
    assert!(!is_struct(s));
    s.bind_with(|_| BindStyle::Move);

    let num_variants = s.variants().len();
    let discriminant_size_type = get_discriminant_size_type(num_variants);
    let body = s
        .variants()
        .iter()
        .enumerate()
        .fold(quote!(), |acc, (i, vi)| {
            let bindings = vi
                .bindings()
                .iter()
                .map(|bi| quote!(#bi))
                .collect::<Vec<_>>();

            let variant_pat = Index::from(i);
            let poke_exprs = bindings.iter().fold(quote!(), |acc, bi| {
                quote! {
                    #acc
                    let (#bi, bytes) = peek_poke::peek_from_default(bytes);
                }
            });
            let construct = vi.construct(|_, i| {
                let bi = &bindings[i];
                quote!(#bi)
            });

            quote! {
                #acc
                #variant_pat => {
                    #poke_exprs
                    *output = #construct;
                    bytes
                }
            }
        });

    let type_name = s.ast().ident.to_string();
    let max_tag_value = num_variants - 1;

    quote! {
        #[inline(always)]
        unsafe fn peek_from(bytes: *const u8, output: *mut Self) -> *const u8 {
            let (variant, bytes) = peek_poke::peek_from_default::<#discriminant_size_type>(bytes);
            match variant {
                #body
                out_of_range_tag => {
                    panic!("WRDL: memory corruption detected while parsing {} - enum tag should be <= {}, but was {}",
                        #type_name, #max_tag_value, out_of_range_tag);
                }
            }
        }
    }
}

fn derive_peek_from_for_struct(s: &mut Structure) -> TokenStream {
    assert!(is_struct(&s));

    s.variants_mut()[0].bind_with(|_| BindStyle::RefMut);
    let pat = s.variants()[0].pat();
    let peek_exprs = s.variants()[0].bindings().iter().fold(quote!(), |acc, bi| {
        let ty = &bi.ast().ty;
        quote! {
            #acc
            let bytes = <#ty>::peek_from(bytes, #bi);
        }
    });

    let body = quote! {
        #pat => {
            #peek_exprs
            bytes
        }
    };

    quote! {
        #[inline(always)]
        unsafe fn peek_from(bytes: *const u8, output: *mut Self) -> *const u8 {
            match &mut (*output) {
                #body
            }
        }
    }
}

fn derive_poke_into(s: &Structure) -> TokenStream {
    let is_struct = is_struct(&s);
    let discriminant_size_type = get_discriminant_size_type(s.variants().len());
    let body = s
        .variants()
        .iter()
        .enumerate()
        .fold(quote!(), |acc, (i, vi)| {
            let init = if !is_struct {
                let index = Index::from(i);
                quote! {
                    let bytes = #discriminant_size_type::poke_into(&#index, bytes);
                }
            } else {
                quote!()
            };
            let variant_pat = vi.pat();
            let poke_exprs = vi.bindings().iter().fold(init, |acc, bi| {
                quote! {
                    #acc
                    let bytes = #bi.poke_into(bytes);
                }
            });

            quote! {
                #acc
                #variant_pat => {
                    #poke_exprs
                    bytes
                }
            }
        });

    quote! {
        #[inline(always)]
        unsafe fn poke_into(&self, bytes: *mut u8) -> *mut u8 {
            match &*self {
                #body
            }
        }
    }
}

fn peek_poke_derive(mut s: Structure) -> TokenStream {
    s.binding_name(|_, i| Ident::new(&format!("__self_{}", i), Span::call_site()));

    let max_size_fn = derive_max_size(&s);
    let poke_into_fn = derive_poke_into(&s);
    let peek_from_fn = if is_struct(&s) {
        derive_peek_from_for_struct(&mut s)
    } else {
        derive_peek_from_for_enum(&mut s)
    };

    let poke_impl = s.gen_impl(quote! {
        extern crate peek_poke;

        gen unsafe impl peek_poke::Poke for @Self {
            #max_size_fn
            #poke_into_fn
        }
    });

    // To implement `fn peek_from` we require that types implement `Default`
    // trait to create temporary values. This code does the addition all
    // manually until https://github.com/mystor/synstructure/issues/24 is fixed.
    let default_trait = syn::parse_str::<TraitBound>("::std::default::Default").unwrap();
    let peek_trait = syn::parse_str::<TraitBound>("peek_poke::Peek").unwrap();

    let ast = s.ast();
    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    let mut where_clause = where_clause.cloned();
    s.add_trait_bounds(&default_trait, &mut where_clause, AddBounds::Generics);
    s.add_trait_bounds(&peek_trait, &mut where_clause, AddBounds::Generics);

    let dummy_const: Ident = sanitize_ident(&format!("_DERIVE_peek_poke_Peek_FOR_{}", name));

    let peek_impl = quote! {
        #[allow(non_upper_case_globals)]
        const #dummy_const: () = {
            extern crate peek_poke;

            impl #impl_generics peek_poke::Peek for #name #ty_generics #where_clause {
                #peek_from_fn
            }
        };
    };

    quote! {
        #poke_impl
        #peek_impl
    }
}

decl_derive!([PeekPoke] => peek_poke_derive);
