/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use animate::AnimateAttrs;
use cg;
use quote;
use syn;
use synstructure::{self, BindStyle};

pub fn derive(input: syn::DeriveInput) -> quote::Tokens {
    let name = &input.ident;
    let trait_path = &["values", "animated", "ToAnimatedZero"];
    let (impl_generics, ty_generics, mut where_clause) =
        cg::trait_parts(&input, trait_path);

    let bind_opts = BindStyle::Ref.into();
    let to_body = synstructure::each_variant(&input, &bind_opts, |bindings, variant| {
        let attrs = cg::parse_variant_attrs::<AnimateAttrs>(variant);
        if attrs.error {
            return Some(quote! { Err(()) });
        }
        let name = cg::variant_ctor(&input, variant);
        let (mapped, mapped_bindings) = cg::value(&name, variant, "mapped");
        let bindings_pairs = bindings.into_iter().zip(mapped_bindings);
        let mut computations = quote!();
        computations.append_all(bindings_pairs.map(|(binding, mapped_binding)| {
            where_clause.add_trait_bound(binding.field.ty.clone());
            quote! {
                let #mapped_binding =
                    ::values::animated::ToAnimatedZero::to_animated_zero(#binding)?;
            }
        }));
        computations.append(quote! { Ok(#mapped) });
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
