/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern crate proc_macro;
#[macro_use] extern crate quote;
extern crate syn;
extern crate synstructure;

#[cfg(not(test))]
#[proc_macro_derive(JSTraceable)]
pub fn expand_token_stream(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    expand_string(&input.to_string()).parse().unwrap()
}

fn expand_string(input: &str) -> String {
    let mut type_ = syn::parse_macro_input(input).unwrap();

    let style = synstructure::BindStyle::Ref.into();
    let match_body = synstructure::each_field(&mut type_, &style, |binding| {
        Some(quote! { #binding.trace(tracer); })
    });

    let name = &type_.ident;
    let (impl_generics, ty_generics, where_clause) = type_.generics.split_for_impl();
    let mut where_clause = where_clause.clone();
    for param in &type_.generics.ty_params {
        where_clause.predicates.push(syn::WherePredicate::BoundPredicate(syn::WhereBoundPredicate {
            bound_lifetimes: Vec::new(),
            bounded_ty: syn::Ty::Path(None, param.ident.clone().into()),
            bounds: vec![syn::TyParamBound::Trait(
                syn::PolyTraitRef {
                    bound_lifetimes: Vec::new(),
                    trait_ref: syn::parse_path("::dom::bindings::trace::JSTraceable").unwrap(),
                },
                syn::TraitBoundModifier::None
            )],
        }))
    }

    let tokens = quote! {
        #[allow(unsafe_code)]
        unsafe impl #impl_generics ::dom::bindings::trace::JSTraceable for #name #ty_generics #where_clause {
            #[inline]
            #[allow(unused_variables, unused_imports)]
            unsafe fn trace(&self, tracer: *mut ::js::jsapi::JSTracer) {
                use ::dom::bindings::trace::JSTraceable;
                match *self {
                    #match_body
                }
            }
        }
    };

    tokens.to_string()
}
