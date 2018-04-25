/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cg;
use quote::Tokens;
use syn::{Data, DeriveInput, Fields};
use to_css::{CssFieldAttrs, CssInputAttrs, CssVariantAttrs};

pub fn derive(mut input: DeriveInput) -> Tokens {
    let attrs = cg::parse_input_attrs::<CssInputAttrs>(&input);
    let mut types_value = quote!(0);
    // If the whole value is wrapped in a function, value types of its
    // fields should not be propagated.
    if attrs.function.is_none() {
        let mut where_clause = input.generics.where_clause.take();
        for param in input.generics.type_params() {
            cg::add_predicate(
                &mut where_clause,
                parse_quote!(#param: ::style_traits::SpecifiedValueInfo),
            );
        }
        input.generics.where_clause = where_clause;

        match input.data {
            Data::Enum(ref e) => {
                for v in e.variants.iter() {
                    let attrs = cg::parse_variant_attrs::<CssVariantAttrs>(&v);
                    if attrs.function.is_none() {
                        derive_struct_fields(&v.fields, &mut types_value);
                    }
                }
            }
            Data::Struct(ref s) => {
                derive_struct_fields(&s.fields, &mut types_value)
            }
            Data::Union(_) => unreachable!("union is not supported"),
        }
    }

    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    quote! {
        impl #impl_generics ::style_traits::SpecifiedValueInfo for #name #ty_generics
        #where_clause
        {
            const SUPPORTED_TYPES: u8 = #types_value;
        }
    }
}

fn derive_struct_fields(fields: &Fields, supports_body: &mut Tokens) {
    let fields = match *fields {
        Fields::Unit => return,
        Fields::Named(ref fields) => fields.named.iter(),
        Fields::Unnamed(ref fields) => fields.unnamed.iter(),
    };
    supports_body.append_all(fields.map(|field| {
        let attrs = cg::parse_field_attrs::<CssFieldAttrs>(field);
        if attrs.skip {
            return quote!();
        }
        let ty = &field.ty;
        quote! {
            | <#ty as ::style_traits::SpecifiedValueInfo>::SUPPORTED_TYPES
        }
    }));
}
