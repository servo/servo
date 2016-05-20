/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%! from data import Keyword, to_rust_ident %>

<%def name="longhand(name, **kwargs)">
    <%call expr="raw_longhand(name, **kwargs)">
        ${caller.body()}
        % if not data.longhands_by_name[name].derived_from:
            pub fn parse_specified(context: &ParserContext, input: &mut Parser)
                               -> Result<DeclaredValue<SpecifiedValue>, ()> {
                parse(context, input).map(DeclaredValue::Value)
            }
        % endif
    </%call>
</%def>

<%def name="predefined_type(name, type, initial_value, parse_method='parse', **kwargs)">
    <%call expr="longhand(name, predefined_type=type, **kwargs)">
        #[allow(unused_imports)]
        use app_units::Au;
        use cssparser::{Color as CSSParserColor, RGBA};
        pub type SpecifiedValue = specified::${type};
        pub mod computed_value {
            pub use values::computed::${type} as T;
        }
        #[inline] pub fn get_initial_value() -> computed_value::T { ${initial_value} }
        #[inline] pub fn parse(_context: &ParserContext, input: &mut Parser)
                               -> Result<SpecifiedValue, ()> {
            specified::${type}::${parse_method}(input)
        }
    </%call>
</%def>

<%def name="raw_longhand(*args, **kwargs)">
    <%
        property = data.declare_longhand(*args, **kwargs)
        if property is None:
            return ""
    %>
    pub mod ${property.ident} {
        #![allow(unused_imports)]
        % if not property.derived_from:
            use cssparser::Parser;
            use parser::{ParserContext, ParserContextExtraData};
            use properties::{CSSWideKeyword, DeclaredValue, Shorthand};
        % endif
        use error_reporting::ParseErrorReporter;
        use properties::longhands;
        use properties::property_bit_field::PropertyBitField;
        use properties::{ComputedValues, ServoComputedValues, PropertyDeclaration};
        use properties::style_struct_traits::${data.current_style_struct.trait_name};
        use properties::style_structs;
        use std::boxed::Box as StdBox;
        use std::collections::HashMap;
        use std::sync::Arc;
        use values::computed::{TContext, ToComputedValue};
        use values::{computed, specified};
        use string_cache::Atom;
        ${caller.body()}
        #[allow(unused_variables)]
        pub fn cascade_property<C: ComputedValues>(
                                declaration: &PropertyDeclaration,
                                inherited_style: &C,
                                context: &mut computed::Context<C>,
                                seen: &mut PropertyBitField,
                                cacheable: &mut bool,
                                error_reporter: &mut StdBox<ParseErrorReporter + Send>) {
            let declared_value = match *declaration {
                PropertyDeclaration::${property.camel_case}(ref declared_value) => {
                    declared_value
                }
                _ => panic!("entered the wrong cascade_property() implementation"),
            };
            % if not property.derived_from:
                if seen.get_${property.ident}() {
                    return
                }
                seen.set_${property.ident}();
                {
                    let custom_props = context.style().custom_properties();
                    ::properties::substitute_variables_${property.ident}(
                        declared_value, &custom_props, |value| match *value {
                            DeclaredValue::Value(ref specified_value) => {
                                let computed = specified_value.to_computed_value(context);
                                context.mutate_style().mutate_${data.current_style_struct.trait_name_lower}()
                                                      .set_${property.ident}(computed);
                            }
                            DeclaredValue::WithVariables { .. } => unreachable!(),
                            DeclaredValue::Initial => {
                                // We assume that it's faster to use copy_*_from rather than
                                // set_*(get_initial_value());
                                let initial_struct = C::initial_values()
                                                      .get_${data.current_style_struct.trait_name_lower}();
                                context.mutate_style().mutate_${data.current_style_struct.trait_name_lower}()
                                                      .copy_${property.ident}_from(initial_struct);
                            },
                            DeclaredValue::Inherit => {
                                // This is a bit slow, but this is rare so it shouldn't
                                // matter.
                                //
                                // FIXME: is it still?
                                *cacheable = false;
                                let inherited_struct =
                                    inherited_style.get_${data.current_style_struct.trait_name_lower}();
                                context.mutate_style().mutate_${data.current_style_struct.trait_name_lower}()
                                       .copy_${property.ident}_from(inherited_struct);
                            }
                        }, error_reporter
                    );
                }

                % if property.custom_cascade:
                    cascade_property_custom(declaration,
                                            inherited_style,
                                            context,
                                            seen,
                                            cacheable,
                                            error_reporter);
                % endif
            % else:
                // Do not allow stylesheets to set derived properties.
            % endif
        }
        % if not property.derived_from:
            pub fn parse_declared(context: &ParserContext, input: &mut Parser)
                               -> Result<DeclaredValue<SpecifiedValue>, ()> {
                match input.try(CSSWideKeyword::parse) {
                    Ok(CSSWideKeyword::InheritKeyword) => Ok(DeclaredValue::Inherit),
                    Ok(CSSWideKeyword::InitialKeyword) => Ok(DeclaredValue::Initial),
                    Ok(CSSWideKeyword::UnsetKeyword) => Ok(DeclaredValue::${
                        "Inherit" if data.current_style_struct.inherited else "Initial"}),
                    Err(()) => {
                        input.look_for_var_functions();
                        let start = input.position();
                        let specified = parse_specified(context, input);
                        if specified.is_err() {
                            while let Ok(_) = input.next() {}  // Look for var() after the error.
                        }
                        let var = input.seen_var_functions();
                        if specified.is_err() && var {
                            input.reset(start);
                            let (first_token_type, css) = try!(
                                ::custom_properties::parse_non_custom_with_var(input));
                            return Ok(DeclaredValue::WithVariables {
                                css: css.into_owned(),
                                first_token_type: first_token_type,
                                base_url: context.base_url.clone(),
                                from_shorthand: None,
                            })
                        }
                        specified
                    }
                }
            }
        % endif
    }
</%def>

<%def name="single_keyword(name, values, **kwargs)">
    <%call expr="single_keyword_computed(name, values, **kwargs)">
        use values::computed::ComputedValueAsSpecified;
        impl ComputedValueAsSpecified for SpecifiedValue {}
    </%call>
</%def>

<%def name="single_keyword_computed(name, values, **kwargs)">
    <%
        keyword_kwargs = {a: kwargs.pop(a, None) for a in [
            'gecko_constant_prefix', 'extra_gecko_values', 'extra_servo_values'
        ]}
    %>
    <%call expr="longhand(name, keyword=Keyword(name, values, **keyword_kwargs), **kwargs)">
        pub use self::computed_value::T as SpecifiedValue;
        ${caller.body()}
        pub mod computed_value {
            define_css_keyword_enum! { T:
                % for value in data.longhands_by_name[name].keyword.values_for(product):
                    "${value}" => ${to_rust_ident(value)},
                % endfor
            }
        }
        #[inline] pub fn get_initial_value() -> computed_value::T {
            computed_value::T::${to_rust_ident(values.split()[0])}
        }
        pub fn parse(_context: &ParserContext, input: &mut Parser)
                     -> Result<SpecifiedValue, ()> {
            computed_value::T::parse(input)
        }
    </%call>
</%def>

<%def name="shorthand(name, sub_properties, experimental=False, **kwargs)">
<%
    shorthand = data.declare_shorthand(name, sub_properties.split(), experimental=experimental,
                                       **kwargs)
%>
    % if shorthand:
    pub mod ${shorthand.ident} {
        use cssparser::Parser;
        use parser::ParserContext;
        use properties::{longhands, PropertyDeclaration, DeclaredValue, Shorthand};

        pub struct Longhands {
            % for sub_property in shorthand.sub_properties:
                pub ${sub_property.ident}:
                    Option<longhands::${sub_property.ident}::SpecifiedValue>,
            % endfor
        }

        pub fn parse(context: &ParserContext, input: &mut Parser,
                     declarations: &mut Vec<PropertyDeclaration>)
                     -> Result<(), ()> {
            input.look_for_var_functions();
            let start = input.position();
            let value = input.parse_entirely(|input| parse_value(context, input));
            if value.is_err() {
                while let Ok(_) = input.next() {}  // Look for var() after the error.
            }
            let var = input.seen_var_functions();
            if let Ok(value) = value {
                % for sub_property in shorthand.sub_properties:
                    declarations.push(PropertyDeclaration::${sub_property.camel_case}(
                        match value.${sub_property.ident} {
                            Some(value) => DeclaredValue::Value(value),
                            None => DeclaredValue::Initial,
                        }
                    ));
                % endfor
                Ok(())
            } else if var {
                input.reset(start);
                let (first_token_type, css) = try!(
                    ::custom_properties::parse_non_custom_with_var(input));
                % for sub_property in shorthand.sub_properties:
                    declarations.push(PropertyDeclaration::${sub_property.camel_case}(
                        DeclaredValue::WithVariables {
                            css: css.clone().into_owned(),
                            first_token_type: first_token_type,
                            base_url: context.base_url.clone(),
                            from_shorthand: Some(Shorthand::${shorthand.camel_case}),
                        }
                    ));
                % endfor
                Ok(())
            } else {
                Err(())
            }
        }

        #[allow(unused_variables)]
        pub fn parse_value(context: &ParserContext, input: &mut Parser) -> Result<Longhands, ()> {
            ${caller.body()}
        }
    }
    % endif
</%def>

<%def name="four_sides_shorthand(name, sub_property_pattern, parser_function)">
    <%self:shorthand name="${name}" sub_properties="${
            ' '.join(sub_property_pattern % side
                     for side in ['top', 'right', 'bottom', 'left'])}">
        use super::parse_four_sides;
        use values::specified;
        let _unused = context;
        let (top, right, bottom, left) = try!(parse_four_sides(input, ${parser_function}));
        Ok(Longhands {
            % for side in ["top", "right", "bottom", "left"]:
                ${to_rust_ident(sub_property_pattern % side)}: Some(${side}),
            % endfor
        })
    </%self:shorthand>
</%def>
