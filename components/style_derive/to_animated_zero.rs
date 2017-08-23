/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cg;
use quote;
use syn;
use synstructure::BindStyle;

pub fn derive(input: syn::DeriveInput) -> quote::Tokens {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = cg::trait_parts(
        &input,
        &["values", "animated", "ToAnimatedZero"],
    );

    let to_body = cg::fmap_match(&input, BindStyle::Ref, |field| {
        quote! { ::values::animated::ToAnimatedZero::to_animated_zero(#field)? }
    });

    quote! {
        impl #impl_generics ::values::animated::ToAnimatedZero for #name #ty_generics #where_clause {
            #[allow(unused_variables)]
            #[inline]
            fn to_animated_zero(&self) -> Result<Self, ()> {
                Ok(match *self {
                    #to_body
                })
            }
        }
    }
}
