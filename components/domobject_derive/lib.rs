/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![recursion_limit = "128"]

use quote::{TokenStreamExt, quote};

/// First field of DomObject must be either reflector or another dom_struct,
/// all other fields must not implement DomObject
#[proc_macro_derive(DomObject)]
pub fn expand_token_stream(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse(input).unwrap();
    expand_dom_object(input).into()
}

fn expand_dom_object(input: syn::DeriveInput) -> proc_macro2::TokenStream {
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
    let items = quote! {
        impl #impl_generics ::js::conversions::ToJSValConvertible for #name #ty_generics #where_clause {
            #[allow(unsafe_code)]
            unsafe fn to_jsval(&self,
                                cx: *mut js::jsapi::JSContext,
                                rval: js::rust::MutableHandleValue) {
                let object = crate::DomObject::reflector(self).get_jsobject();
                object.to_jsval(cx, rval)
            }
        }

        impl #impl_generics crate::DomObject for #name #ty_generics #where_clause {
            #[inline]
            fn reflector(&self) -> &crate::Reflector {
                self.#first_field_name.reflector()
            }
        }

        impl #impl_generics crate::MutDomObject for #name #ty_generics #where_clause {
            unsafe fn init_reflector(&self, obj: *mut js::jsapi::JSObject) {
                self.#first_field_name.init_reflector(obj);
            }
        }

        impl #impl_generics Eq for #name #ty_generics #where_clause {}

        impl #impl_generics PartialEq for #name #ty_generics #where_clause {
            fn eq(&self, other: &Self) -> bool {
                crate::DomObject::reflector(self) == crate::DomObject::reflector(other)
            }
        }
    };

    let mut params = proc_macro2::TokenStream::new();
    params.append_separated(
        input.generics.type_params().map(|param| &param.ident),
        quote! {,},
    );

    let mut dummy_items = quote! {
        // Generic trait with a blanket impl over `()` for all types.
        // becomes ambiguous if impl
        trait NoDomObjectInDomObject<A> {
            // Required for actually being able to reference the trait.
            fn some_item() {}
        }

        impl<T: ?Sized> NoDomObjectInDomObject<()> for T {}

        // Used for the specialized impl when DomObject is implemented.
        #[allow(dead_code)]
        struct Invalid;
        // forbids DomObject
        impl<T> NoDomObjectInDomObject<Invalid> for T where T: ?Sized + crate::DomObject {}
    };

    dummy_items.append_all(field_types.iter().enumerate().map(|(i, ty)| {
        let s = syn::Ident::new(&format!("S{i}"), proc_macro2::Span::call_site());
        quote! {
            struct #s<#params>(#params);

            impl #impl_generics #s<#params> #where_clause {
                fn f() {
                    // If there is only one specialized trait impl, type inference with
                    // `_` can be resolved and this can compile. Fails to compile if
                    // ty implements `NoDomObjectInDomObject<Invalid>`.
                    let _ = <#ty as NoDomObjectInDomObject<_>>::some_item;
                }
            }
        }
    }));

    let dummy_const = syn::Ident::new(
        &format!("_IMPL_DOMOBJECT_FOR_{}", name),
        proc_macro2::Span::call_site(),
    );
    let tokens = quote! {
        #[allow(non_upper_case_globals)]
        const #dummy_const: () = { #dummy_items };
        #items
    };

    tokens
}
