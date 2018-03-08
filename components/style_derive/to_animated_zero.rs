/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use animate::{AnimationVariantAttrs, AnimationFieldAttrs};
use cg;
use quote;
use syn;
use synstructure;

pub fn derive(input: syn::DeriveInput) -> quote::Tokens {
    let name = &input.ident;
    let trait_path = parse_quote!(values::animated::ToAnimatedZero);
    let (impl_generics, ty_generics, mut where_clause) =
        cg::trait_parts(&input, &trait_path);

    let s = synstructure::Structure::new(&input);
    let to_body = s.each_variant(|variant| {
        let attrs = cg::parse_variant_attrs::<AnimationVariantAttrs>(&variant.ast());
        if attrs.error {
            return Some(quote! { Err(()) });
        }
        let (mapped, mapped_bindings) = cg::value(variant, "mapped");
        let bindings_pairs = variant.bindings().into_iter().zip(mapped_bindings);
        let mut computations = quote!();
        computations.append_all(bindings_pairs.map(|(binding, mapped_binding)| {
            let field_attrs = cg::parse_field_attrs::<AnimationFieldAttrs>(&binding.ast());
            if field_attrs.constant {
                if cg::is_parameterized(&binding.ast().ty, &where_clause.params, None) {
                    cg::add_predicate(
                        &mut where_clause.inner,
                        cg::where_predicate(
                            binding.ast().ty.clone(),
                            &parse_quote!(std::clone::Clone),
                            None,
                        ),
                    );
                }
                quote! {
                    let #mapped_binding = ::std::clone::Clone::clone(#binding);
                }
            } else {
                where_clause.add_trait_bound(&binding.ast().ty);
                quote! {
                    let #mapped_binding =
                        ::values::animated::ToAnimatedZero::to_animated_zero(#binding)?;
                }
            }
        }));
        computations.append_all(quote! { Ok(#mapped) });
        Some(computations)
    });

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
