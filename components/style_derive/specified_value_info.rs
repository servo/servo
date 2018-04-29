/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cg;
use quote::Tokens;
use syn::{Data, DeriveInput, Fields, Ident, Type};
use to_css::{CssFieldAttrs, CssInputAttrs, CssVariantAttrs};

pub fn derive(mut input: DeriveInput) -> Tokens {
    let css_attrs = cg::parse_input_attrs::<CssInputAttrs>(&input);
    let mut types = vec![];
    let mut values = vec![];

    let input_ident = input.ident;
    let input_name = || cg::to_css_identifier(input_ident.as_ref());
    if let Some(function) = css_attrs.function {
        values.push(function.explicit().unwrap_or_else(input_name));
        // If the whole value is wrapped in a function, value types of
        // its fields should not be propagated.
    } else {
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
                    let css_attrs = cg::parse_variant_attrs::<CssVariantAttrs>(&v);
                    let info_attrs = cg::parse_variant_attrs::<ValueInfoVariantAttrs>(&v);
                    if css_attrs.skip {
                        continue;
                    }
                    if let Some(aliases) = css_attrs.aliases {
                        for alias in aliases.split(",") {
                            values.push(alias.to_string());
                        }
                    }
                    if let Some(other_values) = info_attrs.other_values {
                        for value in other_values.split(",") {
                            values.push(value.to_string());
                        }
                    }
                    let ident = &v.ident;
                    let variant_name = || cg::to_css_identifier(ident.as_ref());
                    if info_attrs.starts_with_keyword {
                        values.push(variant_name());
                        continue;
                    }
                    if let Some(keyword) = css_attrs.keyword {
                        values.push(keyword);
                        continue;
                    }
                    if let Some(function) = css_attrs.function {
                        values.push(function.explicit().unwrap_or_else(variant_name));
                    } else {
                        if !derive_struct_fields(&v.fields, &mut types, &mut values) {
                            values.push(variant_name());
                        }
                    }
                }
            }
            Data::Struct(ref s) => {
                if !derive_struct_fields(&s.fields, &mut types, &mut values) {
                    values.push(input_name());
                }
            }
            Data::Union(_) => unreachable!("union is not supported"),
        }
    }

    let info_attrs = cg::parse_input_attrs::<ValueInfoInputAttrs>(&input);
    if let Some(other_values) = info_attrs.other_values {
        for value in other_values.split(",") {
            values.push(value.to_string());
        }
    }

    let mut types_value = quote!(0);
    types_value.append_all(types.iter().map(|ty| quote! {
        | <#ty as ::style_traits::SpecifiedValueInfo>::SUPPORTED_TYPES
    }));

    let mut nested_collects = quote!();
    nested_collects.append_all(types.iter().map(|ty| quote! {
        <#ty as ::style_traits::SpecifiedValueInfo>::collect_completion_keywords(_f);
    }));

    if let Some(ty) = info_attrs.ty {
        types_value.append_all(quote! {
            | ::style_traits::CssType::#ty
        });
    }

    let append_values = if values.is_empty() {
        quote!()
    } else {
        let mut value_list = quote!();
        value_list.append_separated(values.iter(), quote! { , });
        quote! { _f(&[#value_list]); }
    };

    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    quote! {
        impl #impl_generics ::style_traits::SpecifiedValueInfo for #name #ty_generics
        #where_clause
        {
            const SUPPORTED_TYPES: u8 = #types_value;

            fn collect_completion_keywords(_f: &mut FnMut(&[&'static str])) {
                #nested_collects
                #append_values
            }
        }
    }
}

/// Derive from the given fields. Return false if the fields is a Unit,
/// true otherwise.
fn derive_struct_fields<'a>(
    fields: &'a Fields,
    types: &mut Vec<&'a Type>,
    values: &mut Vec<String>,
) -> bool {
    let fields = match *fields {
        Fields::Unit => return false,
        Fields::Named(ref fields) => fields.named.iter(),
        Fields::Unnamed(ref fields) => fields.unnamed.iter(),
    };
    types.extend(fields.filter_map(|field| {
        let info_attrs = cg::parse_field_attrs::<ValueInfoFieldAttrs>(field);
        if let Some(other_values) = info_attrs.other_values {
            for value in other_values.split(",") {
                values.push(value.to_string());
            }
        }
        let css_attrs = cg::parse_field_attrs::<CssFieldAttrs>(field);
        if css_attrs.represents_keyword {
            let ident = field.ident.as_ref()
                .expect("only named field should use represents_keyword");
            values.push(cg::to_css_identifier(ident.as_ref()));
            return None;
        }
        if let Some(if_empty) = css_attrs.if_empty {
            values.push(if_empty);
        }
        if !css_attrs.skip {
            Some(&field.ty)
        } else {
            None
        }
    }));
    true
}

#[darling(attributes(value_info), default)]
#[derive(Default, FromDeriveInput)]
struct ValueInfoInputAttrs {
    ty: Option<Ident>,
    other_values: Option<String>,
}

#[darling(attributes(value_info), default)]
#[derive(Default, FromVariant)]
struct ValueInfoVariantAttrs {
    starts_with_keyword: bool,
    other_values: Option<String>,
}

#[darling(attributes(value_info), default)]
#[derive(Default, FromField)]
struct ValueInfoFieldAttrs {
    other_values: Option<String>,
}
