/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::Punct;
use quote::{ToTokens, TokenStreamExt, quote};
use syn::parse::{Parse, Parser};
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{Expr, ItemFn, Meta, MetaList, Token, parse_quote, parse2};

struct Fields(MetaList);
impl From<MetaList> for Fields {
    fn from(value: MetaList) -> Self {
        Fields(value)
    }
}

impl Fields {
    fn create_with_servo_profiling() -> Self {
        Fields(parse_quote! { fields(servo_profiling = true) })
    }

    fn inject_servo_profiling(&mut self) -> syn::Result<()> {
        let metalist = std::mem::replace(&mut self.0, parse_quote! {field()});

        let arguments: Punctuated<Meta, Comma> =
            Punctuated::parse_terminated.parse2(metalist.tokens)?;

        let servo_profile_given = arguments
            .iter()
            .any(|arg| arg.path().is_ident("servo_profiling"));

        let metalist = if servo_profile_given {
            parse_quote! {
                fields(#arguments)
            }
        } else {
            parse_quote! {
                fields(servo_profiling=true, #arguments)
            }
        };

        let _ = std::mem::replace(&mut self.0, metalist);

        Ok(())
    }
}

impl ToTokens for Fields {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let items = &self.0;
        tokens.append_all(quote! { #items });
    }
}
enum Directive {
    Passthrough(Meta),
    Level(Expr),
    Fields(Fields),
}

impl From<Fields> for Directive {
    fn from(value: Fields) -> Self {
        Directive::Fields(value)
    }
}

impl Directive {
    fn is_level(&self) -> bool {
        matches!(self, Directive::Level(..))
    }

    fn fields_mut(&mut self) -> Option<&mut Fields> {
        match self {
            Directive::Fields(fields) => Some(fields),
            _ => None,
        }
    }
}

impl ToTokens for Directive {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Directive::Passthrough(meta) => tokens.append_all(quote! { #meta }),
            Directive::Level(level) => tokens.append_all(quote! { level = #level }),
            Directive::Fields(fields) => tokens.append_all(quote! { #fields }),
        };
    }
}

impl ToTokens for InstrumentConfiguration {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.append_terminated(&self.0, Punct::new(',', proc_macro2::Spacing::Joint));
    }
}

struct InstrumentConfiguration(Vec<Directive>);

impl InstrumentConfiguration {
    fn inject_servo_profiling(&mut self) -> syn::Result<()> {
        let fields = self.0.iter_mut().find_map(Directive::fields_mut);
        match fields {
            None => {
                self.0
                    .push(Directive::from(Fields::create_with_servo_profiling()));
                Ok(())
            },
            Some(fields) => fields.inject_servo_profiling(),
        }
    }

    fn inject_level(&mut self) {
        if self.0.iter().any(|a| a.is_level()) {
            return;
        }
        self.0.push(Directive::Level(parse_quote! { "trace" }));
    }
}

impl Parse for InstrumentConfiguration {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let args = Punctuated::<Meta, Token![,]>::parse_terminated(input)?;
        let mut components = vec![];

        for arg in args {
            match arg {
                Meta::List(meta_list) if meta_list.path.is_ident("fields") => {
                    components.push(Directive::Fields(meta_list.into()));
                },
                Meta::NameValue(meta_name_value) if meta_name_value.path.is_ident("level") => {
                    components.push(Directive::Level(meta_name_value.value));
                },
                _ => {
                    components.push(Directive::Passthrough(arg));
                },
            }
        }
        Ok(InstrumentConfiguration(components))
    }
}

fn instrument_internal(
    attr: proc_macro2::TokenStream,
    item: proc_macro2::TokenStream,
) -> syn::Result<proc_macro2::TokenStream> {
    // Prepare passthrough arguments for tracing::instrument
    let mut configuration: InstrumentConfiguration = parse2(attr)?;
    let input_fn: ItemFn = parse2(item)?;

    configuration.inject_servo_profiling()?;
    configuration.inject_level();

    let output = quote! {
        #[cfg_attr(
            feature = "tracing",
            tracing::instrument(
                #configuration
            )
        )]
        #input_fn
    };

    Ok(output)
}

#[proc_macro_attribute]
/// Instruments a function with some sane defaults by automatically:
///  - setting the attribute behind the "tracing" flag
///  - adding `servo_profiling = true` in the `tracing::instrument(fields(...))` argument.
///  - setting `level = "trace"` if it is not given.
///
/// This macro assumes the consuming crate has a `tracing` feature flag.
///
/// We need to be able to set the following
/// ```
/// #[cfg_attr(
///         feature = "tracing",
///         tracing::instrument(
///             name = "MyCustomName",
///             skip_all,
///             fields(servo_profiling = true),
///             level = "trace",
///         )
///     )]
/// fn my_fn() { /* .... */ }
/// ```
/// from a simpler macro, such as:
///
/// ```
/// #[servo_tracing::instrument(name = "MyCustomName", skip_all)]
/// fn my_fn() { /* .... */ }
/// ```
pub fn instrument(attr: TokenStream, item: TokenStream) -> TokenStream {
    match instrument_internal(attr.into(), item.into()) {
        Ok(stream) => stream.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[cfg(test)]
mod test {
    use proc_macro2::TokenStream;
    use quote::{ToTokens, quote};
    use syn::{Attribute, ItemFn};

    use crate::instrument_internal;

    fn extract_instrument_attribute(item_fn: &mut ItemFn) -> TokenStream {
        let attr: &Attribute = item_fn
            .attrs
            .iter()
            .find(|attr| {
                // because this is a very nested structure, it is easier to check
                // by constructing the full path, and then doing a string comparison.
                let p = attr.path().to_token_stream().to_string();
                p == "servo_tracing :: instrument"
            })
            .expect("Attribute `servo_tracing::instrument` not found");

        // we create a tokenstream of the actual internal contents of the attribute
        let attr_args = attr
            .parse_args::<TokenStream>()
            .expect("Failed to parse attribute args");

        // we remove the tracing attribute, this is to avoid passing it as an actual attribute to itself.
        item_fn.attrs.retain(|attr| {
            attr.path().to_token_stream().to_string() != "servo_tracing :: instrument"
        });

        attr_args
    }

    /// To make test case generation easy, we parse a test_case as a function item
    /// with its own attributes, including [`servo_tracing::instrument`].
    ///
    /// We extract the [`servo_tracing::instrument`] attribute, and pass it as the first argument to
    /// [`servo_tracing::instrument_internal`],
    fn evaluate(function: TokenStream, test_case: TokenStream, expected: TokenStream) {
        let test_case = quote! {
            #test_case
            #function
        };
        let expected = quote! {
            #expected
            #function
        };
        let function_str = function.to_string();
        let function_str = syn::parse_file(&function_str).expect("function to have valid syntax");
        let function_str = prettyplease::unparse(&function_str);

        let mut item_fn: ItemFn =
            syn::parse2(test_case).expect("Failed to parse input as function");

        let attr_args = extract_instrument_attribute(&mut item_fn);
        let item_fn = item_fn.to_token_stream();

        let generated = instrument_internal(attr_args, item_fn).expect("Generation to not fail.");

        let generated = syn::parse_file(generated.to_string().as_str())
            .expect("to have generated a valid function");
        let generated = prettyplease::unparse(&generated);
        let expected = syn::parse_file(expected.to_string().as_str())
            .expect("to have been given a valid expected function");
        let expected = prettyplease::unparse(&expected);

        eprintln!(
            "Generated:---------:\n{}--------\nExpected:----------\n{}",
            &generated, &expected
        );
        assert_eq!(generated, expected);
        assert!(
            generated.contains(&function_str),
            "Expected generated code: {generated} to contain the function code: {function_str}"
        );
    }

    fn function1() -> TokenStream {
        quote! {
            pub fn start(
                state: (),
                layout_factory: (),
                random_pipeline_closure_probability: (),
                random_pipeline_closure_seed: (),
                hard_fail: (),
                canvas_create_sender: (),
                canvas_ipc_sender: (),
            ) {
            }
        }
    }

    fn function2() -> TokenStream {
        quote! {
            fn layout(
                mut self,
                layout_context: &LayoutContext,
                positioning_context: &mut PositioningContext,
                containing_block_for_children: &ContainingBlock,
                containing_block_for_table: &ContainingBlock,
                depends_on_block_constraints: bool,
            ) {
            }
        }
    }

    #[test]
    fn passing_servo_profiling_and_level_and_aux() {
        let function = function1();
        let expected = quote! {
            #[cfg_attr(
                feature = "tracing",
                tracing::instrument(skip(state, layout_factory), fields(servo_profiling = true), level = "trace",)
            )]
        };

        let test_case = quote! {
            #[servo_tracing::instrument(skip(state, layout_factory),fields(servo_profiling = true),level = "trace",)]
        };

        evaluate(function, test_case, expected);
    }

    #[test]
    fn passing_servo_profiling_and_level() {
        let function = function1();
        let expected = quote! {
            #[cfg_attr(
                feature = "tracing",
                tracing::instrument( fields(servo_profiling = true), level = "trace",)
            )]
        };

        let test_case = quote! {
            #[servo_tracing::instrument(fields(servo_profiling = true),level = "trace",)]
        };
        evaluate(function, test_case, expected);
    }

    #[test]
    fn passing_servo_profiling() {
        let function = function1();
        let expected = quote! {
            #[cfg_attr(
                feature = "tracing",
                tracing::instrument( fields(servo_profiling = true), level = "trace",)
            )]
        };

        let test_case = quote! {
            #[servo_tracing::instrument(fields(servo_profiling = true))]
        };
        evaluate(function, test_case, expected);
    }

    #[test]
    fn inject_level_and_servo_profiling() {
        let function = function1();
        let expected = quote! {
            #[cfg_attr(
                feature = "tracing",
                tracing::instrument(fields(servo_profiling = true), level = "trace",)
            )]
        };

        let test_case = quote! {
            #[servo_tracing::instrument()]
        };
        evaluate(function, test_case, expected);
    }

    #[test]
    fn instrument_with_name() {
        let function = function2();
        let expected = quote! {
            #[cfg_attr(
                feature = "tracing",
                tracing::instrument(
                    name = "Table::layout",
                    skip_all,
                    fields(servo_profiling = true),
                    level = "trace",
                )
            )]
        };

        let test_case = quote! {
            #[servo_tracing::instrument(name="Table::layout", skip_all)]
        };

        evaluate(function, test_case, expected);
    }
}
