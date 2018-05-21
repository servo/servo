/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![recursion_limit = "128"]

extern crate proc_macro;
#[macro_use] extern crate quote;
#[macro_use] extern crate syn;

#[proc_macro_derive(DomObject)]
pub fn expand_token_stream(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse(input).unwrap();
    expand_dom_object(input).into()
}

fn expand_dom_object(input: syn::DeriveInput) -> quote::Tokens {
    let fields = if let syn::Data::Struct(syn::DataStruct { ref fields, .. }) = input.data {
        fields.iter().collect::<Vec<&syn::Field>>()
    } else {
        panic!("#[derive(DomObject)] should only be applied on proper structs")
    };

    let (first_field, fields) = fields
        .split_first()
        .expect("#[derive(DomObject)] should not be applied on empty structs");

    let first_field_name = first_field.ident.as_ref().unwrap();
    let mut field_types = vec![];
    for field in fields {
        if !field_types.contains(&&field.ty) {
            field_types.push(&field.ty);
        }
    }

    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let mut items = quote! {
        impl #impl_generics ::js::conversions::ToJSValConvertible for #name #ty_generics #where_clause {
            #[allow(unsafe_code)]
            unsafe fn to_jsval(&self,
                                cx: *mut ::js::jsapi::JSContext,
                                rval: ::js::rust::MutableHandleValue) {
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

    let mut params = quote::Tokens::new();
    params.append_separated(input.generics.type_params().map(|param| param.ident), ", ");

    // For each field in the struct, we implement ShouldNotImplDomObject for a
    // pair of all the type parameters of the DomObject and and the field type.
    // This allows us to support parameterized DOM objects
    // such as IteratorIterable<T>.
    items.append_all(field_types.iter().map(|ty| {
        quote! {
            impl #impl_generics ShouldNotImplDomObject for ((#params), #ty) #where_clause {}
        }
    }));

    let mut generics = input.generics.clone();
    generics.params.push(parse_quote!(__T: ::dom::bindings::reflector::DomObject));

    let (impl_generics, _, where_clause) = generics.split_for_impl();

    items.append_all(quote! {
        trait ShouldNotImplDomObject {}
        impl #impl_generics ShouldNotImplDomObject for ((#params), __T) #where_clause {}
    });

    let dummy_const = syn::Ident::from(format!("_IMPL_DOMOBJECT_FOR_{}", name));
    let tokens = quote! {
        #[allow(non_upper_case_globals)]
        const #dummy_const: () = { #items };
    };

    tokens
}
