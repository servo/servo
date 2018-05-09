/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use animate::{AnimationFieldAttrs, AnimationInputAttrs, AnimationVariantAttrs};
use cg;
use quote;
use syn;
use synstructure;

pub fn derive(mut input: syn::DeriveInput) -> quote::Tokens {
    let animation_input_attrs = cg::parse_input_attrs::<AnimationInputAttrs>(&input);
    let no_bound = animation_input_attrs.no_bound.unwrap_or_default();
    let mut where_clause = input.generics.where_clause.take();
    for param in input.generics.type_params() {
        if !no_bound.contains(&param.ident) {
            cg::add_predicate(
                &mut where_clause,
                parse_quote!(#param: ::values::animated::ToAnimatedZero),
            );
        }
    }

    let to_body = synstructure::Structure::new(&input).each_variant(|variant| {
        let attrs = cg::parse_variant_attrs_from_ast::<AnimationVariantAttrs>(&variant.ast());
        if attrs.error {
            return Some(quote! { Err(()) });
        }
        let (mapped, mapped_bindings) = cg::value(variant, "mapped");
        let bindings_pairs = variant.bindings().into_iter().zip(mapped_bindings);
        let mut computations = quote!();
        computations.append_all(bindings_pairs.map(|(binding, mapped_binding)| {
            let field_attrs = cg::parse_field_attrs::<AnimationFieldAttrs>(&binding.ast());
            if field_attrs.constant {
                quote! {
                    let #mapped_binding = ::std::clone::Clone::clone(#binding);
                }
            } else {
                quote! {
                    let #mapped_binding =
                        ::values::animated::ToAnimatedZero::to_animated_zero(#binding)?;
                }
            }
        }));
        computations.append_all(quote! { Ok(#mapped) });
        Some(computations)
    });
    input.generics.where_clause = where_clause;

    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    quote! {
        impl #impl_generics ::values::animated::ToAnimatedZero for #name #ty_generics #where_clause {
            #[allow(unused_variables)]
            #[inline]
            fn to_animated_zero(&self) -> Result<Self, ()> {
                match *self {
                    #to_body
                }
            }
        }
    }
}
