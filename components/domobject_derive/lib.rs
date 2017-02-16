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

    let fields = if let syn::Body::Struct(syn::VariantData::Struct(ref fields)) = type_.body {
        fields
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

    let name = &type_.ident;
    let (impl_generics, ty_generics, where_clause) = type_.generics.split_for_impl();

    let mut items = quote! {
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

    let mut params = quote::Tokens::new();
    params.append_separated(type_.generics.ty_params.iter().map(|param| &param.ident), ", ");

    // For each field in the struct, we implement ShouldNotImplDomObject for a
    // pair of all the type parameters of the DomObject and and the field type.
    // This allows us to support parameterized DOM objects
    // such as IteratorIterable<T>.
    items.append_all(field_types.iter().map(|ty| {
        quote! {
            impl #impl_generics ShouldNotImplDomObject for ((#params), #ty) #where_clause {}
        }
    }));

    let bound = syn::TyParamBound::Trait(
        syn::PolyTraitRef {
            bound_lifetimes: vec![],
            trait_ref: syn::parse_path("::dom::bindings::reflector::DomObject").unwrap(),
        },
        syn::TraitBoundModifier::None
    );

    let mut generics = type_.generics.clone();
    generics.ty_params.push(syn::TyParam {
        attrs: vec![],
        ident: "__T".into(),
        bounds: vec![bound],
        default: None,
    });
    let (impl_generics, _, where_clause) = generics.split_for_impl();

    items.append(quote! {
        trait ShouldNotImplDomObject {}
        impl #impl_generics ShouldNotImplDomObject for ((#params), __T) #where_clause {}
    }.as_str());

    let dummy_const = syn::Ident::new(format!("_IMPL_DOMOBJECT_FOR_{}", name));
    let tokens = quote! {
        #[allow(non_upper_case_globals)]
        const #dummy_const: () = { #items };
    };

    tokens.to_string()
}
