/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use derive_common::cg;
use proc_macro2::TokenStream;
use syn;
use synstructure::{BindStyle, Structure};

pub fn derive(mut input: syn::DeriveInput) -> TokenStream {
    let mut where_clause = input.generics.where_clause.take();
    let attrs = cg::parse_input_attrs::<ShmemInputAttrs>(&input);
    if !attrs.no_bounds {
        for param in input.generics.type_params() {
            cg::add_predicate(&mut where_clause, parse_quote!(#param: ::to_shmem::ToShmem));
        }
    }
    for variant in Structure::new(&input).variants() {
        for binding in variant.bindings() {
            let attrs = cg::parse_field_attrs::<ShmemFieldAttrs>(&binding.ast());
            if attrs.field_bound {
                let ty = &binding.ast().ty;
                cg::add_predicate(&mut where_clause, parse_quote!(#ty: ::to_shmem::ToShmem))
            }
        }
    }

    input.generics.where_clause = where_clause;

    // Do all of the `to_shmem()?` calls before the `ManuallyDrop::into_inner()`
    // calls, so that we don't drop a value in the shared memory buffer if one
    // of the `to_shmem`s fails.
    let match_body = cg::fmap2_match(
        &input,
        BindStyle::Ref,
        |binding| {
            quote! {
                ::to_shmem::ToShmem::to_shmem(#binding, builder)?
            }
        },
        |binding| {
            Some(quote! {
                ::std::mem::ManuallyDrop::into_inner(#binding)
            })
        },
    );

    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    quote! {
        impl #impl_generics ::to_shmem::ToShmem for #name #ty_generics #where_clause {
            #[allow(unused_variables, unreachable_code)]
            fn to_shmem(
                &self,
                builder: &mut ::to_shmem::SharedMemoryBuilder,
            ) -> ::to_shmem::Result<Self> {
                Ok(::std::mem::ManuallyDrop::new(
                    match *self {
                        #match_body
                    }
                ))
            }
        }
    }
}

#[derive(Default, FromDeriveInput)]
#[darling(attributes(shmem), default)]
pub struct ShmemInputAttrs {
    pub no_bounds: bool,
}

#[derive(Default, FromField)]
#[darling(attributes(shmem), default)]
pub struct ShmemFieldAttrs {
    pub field_bound: bool,
}
