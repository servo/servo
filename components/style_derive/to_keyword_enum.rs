/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use quote::Tokens;
use syn::DeriveInput;
use synstructure;

pub fn derive(input: DeriveInput) -> Tokens {
    let name = &input.ident;
    let mut match_body = quote! {};

    let style = synstructure::BindStyle::Ref.into();
    synstructure::each_variant(&input, &style, |bindings, variant| {
        assert!(
            bindings.is_empty(),
            "ToKeywordEnum is only supported for single-variant enums"
        );

        let ident = &variant.ident;
        let discriminant = &variant.discriminant;

        // TODO(canaltinova): We might also want this work with implicit discriminants.
        assert!(
            discriminant.is_some(),
            "ToKeywordEnum requires discriminant values for all variants to implement from_u32 for now"
        );
        match_body = quote! {
            #match_body
            #discriminant => Some(#name::#ident),
        }
    });

    match_body = quote! {
        #match_body
        _ => None
    };

    // TODO(canaltinova): We can create a trait for that.
    quote! {
        impl #name {
            /// Construct a keyword from u32.
            pub fn from_u32(discriminant: u32) -> Option<Self> {
                match discriminant {
                    #match_body
                }
            }
        }
    }
}
