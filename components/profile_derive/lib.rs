/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern crate proc_macro;
#[macro_use]
extern crate quote;

use proc_macro2::Span;
use syn::{Data, DeriveInput, Fields, Ident, Lit, Meta, MetaNameValue};

#[proc_macro_derive(ToProfile, attributes(profile_prefix))]
pub fn derive_to_profile(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let item: DeriveInput = syn::parse(item.into()).unwrap();
    let ident = item.ident;

    let prefix: String = item
        .attrs
        .iter()
        .filter_map(|attr| attr.parse_meta().ok())
        .find(|meta| meta.name() == "profile_prefix")
        .and_then(|meta| {
            if let Meta::NameValue(MetaNameValue {
                lit: Lit::Str(name),
                ..
            }) = meta
            {
                Some(name.value())
            } else {
                None
            }
        })
        .unwrap_or_else(|| ident.to_string());

    let (impl_generics, ty_generics, where_clause) = item.generics.split_for_impl();

    let data_enum = match item.data {
        Data::Enum(data_enum) => data_enum,
        _ => panic!("Can only derive ToProfile on an enum."),
    };

    let arms = data_enum.variants.iter().map(|variant| {
        let variant_ident = &variant.ident;
        let new_ident = Ident::new(&format!("{}{}", prefix, variant_ident), Span::call_site());
        let fields = match variant.fields {
            Fields::Named(..) => quote! { { .. } },
            Fields::Unnamed(..) => quote! { (..) },
            Fields::Unit => quote!{},
        };
        quote! {
            #ident::#variant_ident #fields => profile_traits::time::ProfilerCategory::#new_ident
        }
    });

    let output = quote! {
        impl #impl_generics Into<profile_traits::time::ProfilerCategory> for &#ident #ty_generics #where_clause {
            fn into(self) -> profile_traits::time::ProfilerCategory {
                match *self {
                    #(#arms),*
                }
            }
        }
    };

    output.into()
}
