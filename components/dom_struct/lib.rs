/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![recursion_limit = "128"]

use proc_macro::TokenStream;
use quote::quote;
use syn::*;
mod domobject;
use crate::domobject::expand_dom_object;

#[proc_macro_attribute]
pub fn dom_struct(args: TokenStream, input: TokenStream) -> TokenStream {
    let args2 = proc_macro2::TokenStream::from(args);
    let input2 = proc_macro2::TokenStream::from(input);

    TokenStream::from(dom_struct_impl(args2, input2))
}

fn dom_struct_impl(
    args: proc_macro2::TokenStream,
    input: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    let associated_memory = args.to_string().contains("associated_memory");
    if !associated_memory && !args.is_empty() {
        panic!("#[dom_struct] only takes 'associated_memory' as an argument");
    }
    let attributes = quote! {
        #[derive(deny_public_fields::DenyPublicFields, JSTraceable, MallocSizeOf)]
        #[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
        #[repr(C)]
    };

    // Work around https://github.com/rust-lang/rust/issues/46489
    let attributes: proc_macro2::TokenStream = attributes.to_string().parse().unwrap();

    let output: proc_macro2::TokenStream = attributes.into_iter().chain(input).collect();

    let item: Item = syn::parse2(output).unwrap();

    if let Item::Struct(s) = item {
        let expanded_dom_object = expand_dom_object(s.clone(), associated_memory);
        let s2 = quote! { #s #expanded_dom_object };
        if !s.generics.params.is_empty() {
            return s2;
        }
        if let Fields::Named(ref f) = s.fields {
            let f = f.named.first().expect("Must have at least one field");
            let ident = f.ident.as_ref().expect("Must have named fields");
            let name = &s.ident;
            let ty = &f.ty;

            quote! (
                #s2

                impl crate::HasParent for #name {
                    type Parent = #ty;
                    /// This is used in a type assertion to ensure that
                    /// the source and webidls agree as to what the parent type is
                    fn as_parent(&self) -> &#ty {
                        &self.#ident
                    }
                }
            )
        } else {
            panic!("#[dom_struct] only applies to structs with named fields");
        }
    } else {
        panic!("#[dom_struct] only applies to structs");
    }
}

#[test]
fn test_valid_dom_struct_generation() {
    let args = quote! { associated_memory };
    let reflector_type: syn::Type = parse_quote!(Reflector);
    let input = quote! {
        struct DomElement {
            reflector: #reflector_type,
        }
    };

    let output = dom_struct_impl(args, input);
    let output_str = output.to_string();

    println!("{}", output_str);

    assert!(output_str.contains("DenyPublicFields"));
    assert!(output_str.contains("repr (C)"));

    assert!(output_str.contains("impl crate :: HasParent for DomElement"));
    assert!(output_str.contains("type Parent = Reflector"));
    assert!(output_str.contains("& self . reflector"));
}

#[test]
#[should_panic(expected = "#[dom_struct] only takes 'associated_memory'")]
fn test_invalid_arguments_panic() {
    let args = quote! { invalid_flag_here };
    let input = quote! { struct MockStruct { first_field: i32 } };

    dom_struct_impl(args, input);
}

#[test]
#[should_panic]
fn test_tuple_struct_panic() {
    let args = quote! {};
    let input = quote! { struct UnnamedFields{}; };

    dom_struct_impl(args, input);
}
