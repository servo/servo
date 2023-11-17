/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::{hash_map, HashMap};
use std::fmt::Write;
use std::iter;

use itertools::Itertools;
use proc_macro2::{Span, TokenStream};
use quote::*;
use syn::parse::Result;
use syn::spanned::Spanned;
use syn::{parse_macro_input, Attribute, Ident, LitStr, Path};

mod parse;
use parse::*;

#[proc_macro]
pub fn build_structs(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: MacroInput = parse_macro_input!(tokens);
    let out = Build::new(&input)
        .build(&input.type_def)
        .unwrap_or_else(|e| syn::Error::new(e.span(), e).to_compile_error());
    out.into()
}

struct Build {
    root_type_name: Ident,
    gen_accessors: Ident,
    accessor_type: Path,
    output: TokenStream,
    path_stack: Vec<Ident>,
    path_map: HashMap<String, Vec<Ident>>,
}

impl Build {
    fn new(input: &MacroInput) -> Self {
        Build {
            root_type_name: input.type_def.type_name.clone(),
            gen_accessors: input.gen_accessors.clone(),
            accessor_type: input.accessor_type.clone(),
            output: TokenStream::new(),
            path_stack: Vec::new(),
            path_map: HashMap::new(),
        }
    }

    fn build(mut self, type_def: &RootTypeDef) -> Result<TokenStream> {
        self.walk(&type_def.type_def)?;
        self.build_accessors();
        Ok(self.output)
    }

    fn walk(&mut self, type_def: &NewTypeDef) -> Result<()> {
        self.define_pref_struct(type_def)?;

        for field in type_def.fields.iter() {
            self.path_stack.push(field.name.clone());

            if let FieldType::NewTypeDef(new_def) = &field.field_type {
                self.walk(new_def)?;
            } else {
                let pref_name =
                    self.pref_name(field, &self.path_stack[..self.path_stack.len() - 1]);
                if let hash_map::Entry::Vacant(slot) = self.path_map.entry(pref_name) {
                    slot.insert(self.path_stack.clone());
                } else {
                    return Err(err(&field.name, "duplicate preference name"));
                }
            }

            self.path_stack.pop();
        }
        Ok(())
    }

    fn define_pref_struct(&mut self, type_def: &NewTypeDef) -> Result<()> {
        let struct_name = self.path_to_name(self.path_stack.iter());
        let field_defs = type_def
            .fields
            .iter()
            .map(|field| self.field_to_tokens(field, &self.path_stack))
            .collect::<Result<Vec<_>>>()?;
        self.output.extend(quote! {
            #[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
            pub struct #struct_name {
                #(#field_defs), *
            }
        });
        Ok(())
    }

    fn build_accessors(&mut self) {
        let accessor_type = &self.accessor_type;
        let values = self.path_map.iter().map(|(key, path)| {
            quote! {
                map.insert(String::from(#key),
                    #accessor_type::new(
                        |prefs| prefs #(.#path)*.clone().into(),
                        |prefs, value| prefs #(.#path)* = value.into()
                    )
                 );
            }
        });

        let gen_accessors = &self.gen_accessors;
        let num_prefs = self.path_map.len();

        self.output.extend(quote! {
            lazy_static::lazy_static! {
                pub static ref #gen_accessors: std::collections::HashMap<String, #accessor_type> = {
                    let mut map = std::collections::HashMap::with_capacity(#num_prefs);
                    #(#values)*
                    map
                };
            }
        });
    }

    fn pref_name(&self, field: &Field, path_stack: &[Ident]) -> String {
        field
            .get_field_name_mapping()
            .map(|pref_attr| pref_attr.value())
            .unwrap_or_else(|| {
                Itertools::intersperse(
                    path_stack
                        .iter()
                        .chain(iter::once(&field.name))
                        .map(Ident::to_string),
                    String::from("."),
                )
                .collect()
            })
    }

    fn field_to_tokens(&self, field: &Field, path_stack: &[Ident]) -> Result<TokenStream> {
        let name = &field.name;
        Ok(match &field.field_type {
            FieldType::NewTypeDef(_) => {
                let type_name = self.path_to_name(path_stack.iter().chain(iter::once(name)));
                quote! {
                    #[serde(flatten)]
                    pub #name: #type_name
                }
            },
            FieldType::Existing(type_name) => {
                let pref_name = self.pref_name(field, path_stack);
                let attributes = field.get_attributes(&pref_name);
                quote! {
                    #attributes
                    pub #name: #type_name
                }
            },
        })
    }

    fn path_to_name<'p, P: Iterator<Item = &'p Ident> + 'p>(&self, path: P) -> Ident {
        let mut name = format!("{}", self.root_type_name);
        for part in path {
            name.write_fmt(format_args!("__{}", part)).unwrap();
        }
        Ident::new(&name, Span::call_site())
    }
}

impl Field {
    fn get_attributes(&self, pref_name: &str) -> TokenStream {
        let mut tokens = TokenStream::new();
        for attr in self
            .attributes
            .iter()
            .filter(|attr| attr_to_pref_name(attr).is_none())
        {
            attr.to_tokens(&mut tokens);
        }
        tokens.extend(quote! {
            #[serde(rename = #pref_name)]
        });
        tokens
    }

    fn get_field_name_mapping(&self) -> Option<LitStr> {
        self.attributes.iter().filter_map(attr_to_pref_name).next()
    }
}

fn attr_to_pref_name(attr: &Attribute) -> Option<LitStr> {
    if attr.path().is_ident("serde") {
        // If `parse_nested_meta()` fails, `result` will remain None.
        let mut result = None;
        let _ = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("rename") {
                result = Some(meta.value()?.parse()?);
            }
            Ok(())
        });
        result
    } else {
        None
    }
}

fn err<S: Spanned>(s: S, msg: &str) -> syn::Error {
    syn::Error::new(s.span(), msg)
}
