// Copyright 2016-2017 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! A crate for deriving the MallocSizeOf trait.

#[cfg(not(test))] extern crate proc_macro;
#[macro_use] extern crate quote;
extern crate syn;
extern crate synstructure;

#[cfg(not(test))]
#[proc_macro_derive(MallocSizeOf, attributes(ignore_malloc_size_of))]
pub fn expand_token_stream(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    expand_string(&input.to_string()).parse().unwrap()
}

fn expand_string(input: &str) -> String {
    let mut type_ = syn::parse_macro_input(input).unwrap();

    let style = synstructure::BindStyle::Ref.into();
    let match_body = synstructure::each_field(&mut type_, &style, |binding| {
        let ignore = binding.field.attrs.iter().any(|attr| match attr.value {
            syn::MetaItem::Word(ref ident) |
            syn::MetaItem::List(ref ident, _) if ident == "ignore_malloc_size_of" => {
                panic!("#[ignore_malloc_size_of] should have an explanation, \
                        e.g. #[ignore_malloc_size_of = \"because reasons\"]");
            }
            syn::MetaItem::NameValue(ref ident, _) if ident == "ignore_malloc_size_of" => {
                true
            }
            _ => false,
        });
        if ignore {
            None
        } else if let syn::Ty::Array(..) = binding.field.ty {
            Some(quote! {
                for item in #binding.iter() {
                    sum += ::malloc_size_of::MallocSizeOf::size_of(item, ops);
                }
            })
        } else {
            Some(quote! {
                sum += ::malloc_size_of::MallocSizeOf::size_of(#binding, ops);
            })
        }
    });

    let name = &type_.ident;
    let (impl_generics, ty_generics, where_clause) = type_.generics.split_for_impl();
    let mut where_clause = where_clause.clone();
    for param in &type_.generics.ty_params {
        where_clause.predicates.push(syn::WherePredicate::BoundPredicate(syn::WhereBoundPredicate {
            bound_lifetimes: Vec::new(),
            bounded_ty: syn::Ty::Path(None, param.ident.clone().into()),
            bounds: vec![syn::TyParamBound::Trait(
                syn::PolyTraitRef {
                    bound_lifetimes: Vec::new(),
                    trait_ref: syn::parse_path("::malloc_size_of::MallocSizeOf").unwrap(),
                },
                syn::TraitBoundModifier::None
            )],
        }))
    }

    let tokens = quote! {
        impl #impl_generics ::malloc_size_of::MallocSizeOf for #name #ty_generics #where_clause {
            #[inline]
            #[allow(unused_variables, unused_mut, unreachable_code)]
            fn size_of(&self, ops: &mut ::malloc_size_of::MallocSizeOfOps) -> usize {
                let mut sum = 0;
                match *self {
                    #match_body
                }
                sum
            }
        }
    };

    tokens.to_string()
}

#[test]
fn test_struct() {
    let mut source = "struct Foo<T> { bar: Bar, baz: T, #[ignore_malloc_size_of = \"\"] z: Arc<T> }";
    let mut expanded = expand_string(source);
    let mut no_space = expanded.replace(" ", "");
    macro_rules! match_count {
        ($e: expr, $count: expr) => {
            assert_eq!(no_space.matches(&$e.replace(" ", "")).count(), $count,
                       "counting occurences of {:?} in {:?} (whitespace-insensitive)",
                       $e, expanded)
        }
    }
    match_count!("struct", 0);
    match_count!("ignore_malloc_size_of", 0);
    match_count!("impl<T> ::malloc_size_of::MallocSizeOf for Foo<T> where T: ::malloc_size_of::MallocSizeOf {", 1);
    match_count!("sum += ::malloc_size_of::MallocSizeOf::size_of(", 2);

    source = "struct Bar([Baz; 3]);";
    expanded = expand_string(source);
    no_space = expanded.replace(" ", "");
    match_count!("for item in", 1);
}

#[should_panic(expected = "should have an explanation")]
#[test]
fn test_no_reason() {
    expand_string("struct A { #[ignore_malloc_size_of] b: C }");
}

