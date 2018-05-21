/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern crate quote;
#[macro_use] extern crate syn;
#[macro_use] extern crate synstructure;

decl_derive!([JSTraceable] => js_traceable_derive);

fn js_traceable_derive(s: synstructure::Structure) -> quote::Tokens {
    let match_body = s.each(|binding| {
        Some(quote!(#binding.trace(tracer);))
    });

    let ast = s.ast();
    let name = ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    let mut where_clause = where_clause.unwrap_or(&parse_quote!(where)).clone();
    for param in ast.generics.type_params() {
        let ident = param.ident;
        where_clause.predicates.push(parse_quote!(#ident: ::dom::bindings::trace::JSTraceable))
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

    tokens
}
