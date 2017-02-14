/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern crate proc_macro;
#[macro_use] extern crate quote;
extern crate syn;

#[proc_macro_derive(DomObject)]
pub fn expand_token_stream(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    expand_string(&input.to_string()).parse().unwrap()
}

fn expand_string(input: &str) -> String {
    let type_ = syn::parse_macro_input(input).unwrap();

    let first_field_name = if let syn::Body::Struct(syn::VariantData::Struct(ref fields)) = type_.body {
        let first_field = fields.first().expect("#[derive(DomObject)] should not be applied on empty structs");
        first_field.ident.as_ref().unwrap()
    } else {
        panic!("#[derive(DomObject)] should only be applied on proper structs")
    };

    let name = &type_.ident;
    let (impl_generics, ty_generics, where_clause) = type_.generics.split_for_impl();

    let tokens = quote! {
        impl #impl_generics ::js::conversions::ToJSValConvertible for #name #ty_generics #where_clause {
            #[allow(unsafe_code)]
            unsafe fn to_jsval(&self,
                                cx: *mut ::js::jsapi::JSContext,
                                rval: ::js::jsapi::MutableHandleValue) {
                let object = ::dom::bindings::reflector::DomObject::reflector(self).get_jsobject();
                object.to_jsval(cx, rval)
            }
        }

        impl #impl_generics ::dom::bindings::reflector::DomObject for #name #ty_generics #where_clause {
            #[inline]
            fn reflector(&self) -> &::dom::bindings::reflector::Reflector {
                self.#first_field_name.reflector()
            }
        }

        impl #impl_generics ::dom::bindings::reflector::MutDomObject for #name #ty_generics #where_clause {
            fn init_reflector(&mut self, obj: *mut ::js::jsapi::JSObject) {
                self.#first_field_name.init_reflector(obj);
            }
        }
    };

    tokens.to_string()
}
