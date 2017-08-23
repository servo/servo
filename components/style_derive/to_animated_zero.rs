/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cg;
use quote;
use syn;
use synstructure;

pub fn derive(input: syn::DeriveInput) -> quote::Tokens {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = cg::trait_parts(
        &input,
        &["values", "animated", "ToAnimatedZero"],
    );

    let to_body = match_body(&input);

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

fn match_body(input: &syn::DeriveInput) -> quote::Tokens {
    synstructure::each_variant(&input, &synstructure::BindStyle::Ref.into(), |fields, variant| {
        let name = cg::variant_ctor(input, variant);
        let (zero, computed_fields) = synstructure::match_pattern(
            &name,
            &variant.data,
            &synstructure::BindStyle::Move.into(),
        );
        let fields_pairs = fields.iter().zip(computed_fields.iter());
        let mut computations = quote!();
        computations.append_all(fields_pairs.map(|(field, computed_field)| {
            quote! {
                let #computed_field = ::values::animated::ToAnimatedZero::to_animated_zero(#field)?;
            }
        }));
        Some(quote!(
            #computations
            Ok(#zero)
        ))
    })
}
