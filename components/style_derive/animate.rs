/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cg;
use quote::Tokens;
use syn::{DeriveInput, Path};
use synstructure;

pub fn derive(input: DeriveInput) -> Tokens {
    let name = &input.ident;
    let trait_path = parse_quote!(values::animated::Animate);
    let (impl_generics, ty_generics, mut where_clause) =
        cg::trait_parts(&input, &trait_path);

    let input_attrs = cg::parse_input_attrs::<AnimateInputAttrs>(&input);
    let s = synstructure::Structure::new(&input);
    let mut append_error_clause = s.variants().len() > 1;

    let mut match_body = s.variants().iter().fold(quote!(), |body, variant| {
        let variant_attrs = cg::parse_variant_attrs::<AnimationVariantAttrs>(&variant.ast());
        if variant_attrs.error {
            append_error_clause = true;
            return body;
        }
        let (this_pattern, this_info) = cg::ref_pattern(&variant, "this");
        let (other_pattern, other_info) = cg::ref_pattern(&variant, "other");
        let (result_value, result_info) = cg::value(&variant, "result");
        let mut computations = quote!();
        let iter = result_info.iter().zip(this_info.iter().zip(&other_info));
        computations.append_all(iter.map(|(result, (this, other))| {
            let field_attrs = cg::parse_field_attrs::<AnimationFieldAttrs>(&result.ast());
            if field_attrs.constant {
                if cg::is_parameterized(&result.ast().ty, &where_clause.params, None) {
                    where_clause.add_predicate(cg::where_predicate(
                        result.ast().ty.clone(),
                        &parse_quote!(std::cmp::PartialEq),
                        None,
                    ));
                    where_clause.add_predicate(cg::where_predicate(
                        result.ast().ty.clone(),
                        &parse_quote!(std::clone::Clone),
                        None,
                    ));
                }
                quote! {
                    if #this != #other {
                        return Err(());
                    }
                    let #result = ::std::clone::Clone::clone(#this);
                }
            } else {
                where_clause.add_trait_bound(&result.ast().ty);
                quote! {
                    let #result =
                        ::values::animated::Animate::animate(#this, #other, procedure)?;
                }
            }
        }));
        quote! {
            #body
            (&#this_pattern, &#other_pattern) => {
                #computations
                Ok(#result_value)
            }
        }
    });

    if append_error_clause {
        if let Some(fallback) = input_attrs.fallback {
            match_body.append_all(quote! {
                (this, other) => #fallback(this, other, procedure)
            });
        } else {
            match_body.append_all(quote! { _ => Err(()) });
        }
    }

    quote! {
        impl #impl_generics ::values::animated::Animate for #name #ty_generics #where_clause {
            #[allow(unused_variables, unused_imports)]
            #[inline]
            fn animate(
                &self,
                other: &Self,
                procedure: ::values::animated::Procedure,
            ) -> Result<Self, ()> {
                match (self, other) {
                    #match_body
                }
            }
        }
    }
}

#[darling(attributes(animate), default)]
#[derive(Default, FromDeriveInput)]
struct AnimateInputAttrs {
    fallback: Option<Path>,
}

#[darling(attributes(animation), default)]
#[derive(Default, FromVariant)]
pub struct AnimationVariantAttrs {
    pub error: bool,
}

#[darling(attributes(animation), default)]
#[derive(Default, FromField)]
pub struct AnimationFieldAttrs {
    pub constant: bool,
}
