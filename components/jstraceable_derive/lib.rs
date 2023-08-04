/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#[macro_use]
extern crate syn;
#[macro_use]
extern crate synstructure;

decl_derive!([JSTraceable, attributes(no_trace)] =>
/// Implements `JSTraceable` on structs and enums
///
/// Example:
/// ```rust
/// #[derive(JSTraceable)]
/// struct S {
///   js_managed: JSManagedType,
///   #[no_trace]
///   non_js: NonJSManagedType,
/// }
/// ```
///
/// creates:
///
/// ```rust
/// unsafe impl JSTraceable for S {
///     #[inline]
///     unsafe fn trace(&self, tracer: *mut js::jsapi::JSTracer) {
///         match *self {
///             S {
///                 js_managed: ref __binding_0,
///                 non_js: ref __binding_1,
///             } => {
///                 {
///                     __binding_0.trace(tracer);
///                 }
///                 {
///                     // __binding_1 is not traceable so we do not need to trace it
///                 }
///             },
///         }
///     }
/// }
/// ```
///
/// In cases where there is a need to make type (empty) traceable (`HashMap<NoTraceable, Traceable>`),
/// NoTrace wrapper can be used, because it implements empty traceble:
/// ```rust
/// unsafe impl<T> JSTraceable for NoTrace<T> {
///     unsafe fn trace(&self, _: *mut ::js::jsapi::JSTracer) { /* nop */}
/// }
/// ```
///
/// ## SAFETY
/// Puting `#[no_trace]` on fields is safe if there are no types that are JS managed in that field.
/// `#[no_trace]` should NOT be put on field that does implement (non-empty) `JSTraceable` (is JS managed).
/// There are safeguards in place to prevent such mistakes. Example error:
///
/// ```console
/// error[E0282]: type annotations needed
/// |
/// | #[derive(JSTraceable, MallocSizeOf)]
/// |          ^^^^^^^^^^^ cannot infer type of the type parameter `Self` declared on the trait `NoTraceOnJSTraceable`
/// |
/// = note: this error originates in the derive macro `JSTraceable`
/// ```
///
/// If you can assure that type has empty JSTraceable impl, you can bypass guards, providing your reasoning:
/// ```rust
/// #[derive(JSTraceable)]
/// struct S {
///   #[no_trace = "Safe because both u32 and u64 are empty traceable"]
///   field: HashMap<u32, u64>,
/// }
/// ```
js_traceable_derive);

// based off https://docs.rs/static_assertions/latest/src/static_assertions/assert_impl.rs.html#263
fn assert_not_impl_traceable(ty: &syn::Type) -> proc_macro2::TokenStream {
    quote!(
        const _: fn() = || {
            // Generic trait with a blanket impl over `()` for all types.
            // becomes ambiguous if impl
            trait NoTraceOnJSTraceable<A> {
                // Required for actually being able to reference the trait.
                fn some_item() {}
            }

            impl<T: ?Sized> NoTraceOnJSTraceable<()> for T {}

            // Used for the specialized impl when JSTraceable is implemented.
            #[allow(dead_code)]
            struct Invalid0;
            // forbids JSTraceable
            impl<T> NoTraceOnJSTraceable<Invalid0> for T where T: ?Sized + crate::dom::bindings::trace::JSTraceable {}

            #[allow(dead_code)]
            struct Invalid2;
            // forbids HashMap<JSTraceble, _>
            impl<K, V, S> NoTraceOnJSTraceable<Invalid2> for std::collections::HashMap<K, V, S>
            where
                K: crate::dom::bindings::trace::JSTraceable + std::cmp::Eq + std::hash::Hash,
                S: std::hash::BuildHasher,
            {
            }

            #[allow(dead_code)]
            struct Invalid3;
            // forbids HashMap<_, JSTraceble>
            impl<K, V, S> NoTraceOnJSTraceable<Invalid3> for std::collections::HashMap<K, V, S>
            where
                K: std::cmp::Eq + std::hash::Hash,
                V: crate::dom::bindings::trace::JSTraceable,
                S: std::hash::BuildHasher,
            {
            }

            #[allow(dead_code)]
            struct Invalid4;
            // forbids BTreeMap<_, JSTraceble>
            impl<K, V> NoTraceOnJSTraceable<Invalid4> for std::collections::BTreeMap<K, V> where
                K: crate::dom::bindings::trace::JSTraceable + std::cmp::Eq + std::hash::Hash
            {
            }

            #[allow(dead_code)]
            struct Invalid5;
            // forbids BTreeMap<_, JSTraceble>
            impl<K, V> NoTraceOnJSTraceable<Invalid5> for std::collections::BTreeMap<K, V>
            where
                K: std::cmp::Eq + std::hash::Hash,
                V: crate::dom::bindings::trace::JSTraceable,
            {
            }

            // If there is only one specialized trait impl, type inference with
            // `_` can be resolved and this can compile. Fails to compile if
            // ty implements `NoTraceOnJSTraceable<InvalidX>`.
            let _ = <#ty as NoTraceOnJSTraceable<_>>::some_item;
        };
    )
}

fn js_traceable_derive(s: synstructure::Structure) -> proc_macro2::TokenStream {
    let mut asserts = quote!();
    let match_body = s.each(|binding| {
        for attr in binding.ast().attrs.iter() {
            match attr.parse_meta().unwrap() {
                syn::Meta::Path(ref path) | syn::Meta::List(syn::MetaList { ref path, .. }) => {
                    if path.is_ident("no_trace") {
                        asserts.extend(assert_not_impl_traceable(&binding.ast().ty));
                        return None;
                    }
                },
                syn::Meta::NameValue(syn::MetaNameValue { ref path, .. }) => {
                    if path.is_ident("no_trace") {
                        return None;
                    }
                },
            }
        }
        Some(quote!(#binding.trace(tracer);))
    });

    let ast = s.ast();
    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    let mut where_clause = where_clause.unwrap_or(&parse_quote!(where)).clone();
    for param in ast.generics.type_params() {
        let ident = &param.ident;
        where_clause
            .predicates
            .push(parse_quote!(#ident: crate::dom::bindings::trace::JSTraceable))
    }

    let tokens = quote! {
        #asserts

        #[allow(unsafe_code)]
        unsafe impl #impl_generics crate::dom::bindings::trace::JSTraceable for #name #ty_generics #where_clause {
            #[inline]
            #[allow(unused_variables, unused_imports)]
            unsafe fn trace(&self, tracer: *mut js::jsapi::JSTracer) {
                use crate::dom::bindings::trace::JSTraceable;
                match *self {
                    #match_body
                }
            }
        }
    };

    tokens
}
