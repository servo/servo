/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%! from data import Keyword, to_rust_ident, to_camel_case, LOGICAL_SIDES, PHYSICAL_SIDES, LOGICAL_SIZES %>

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

<%def name="predefined_type(name, type, initial_value, parse_method='parse', needs_context=True, **kwargs)">
    <%call expr="longhand(name, predefined_type=type, **kwargs)">
        #[allow(unused_imports)]
        use app_units::Au;
        use cssparser::{Color as CSSParserColor, RGBA};
        pub use values::specified::${type} as SpecifiedValue;
        pub mod computed_value {
            pub use values::computed::${type} as T;
        }
        #[inline] pub fn get_initial_value() -> computed_value::T { ${initial_value} }
        #[allow(unused_variables)]
        #[inline] pub fn parse(context: &ParserContext, input: &mut Parser)
                               -> Result<SpecifiedValue, ()> {
            % if needs_context:
            specified::${type}::${parse_method}(context, input)
            % else:
            specified::${type}::${parse_method}(input)
            % endif
        }
    </%call>
</%def>

// FIXME (Manishearth): Add computed_value_as_specified argument
// and handle the empty case correctly
<%doc>
    To be used in cases where we have a grammar like
    "<thing> [ , <thing> ]*". `gecko_only` should be set
    to True for cases where Servo takes a single value
    and Stylo supports vector values.

    Setting allow_empty to False allows for cases where the vector
    is empty. The grammar for these is usually "none | <thing> [ , <thing> ]*".
    We assume that the default/initial value is an empty vector for these.
    `initial_value` need not be defined for these.
</%doc>
<%def name="vector_longhand(name, gecko_only=False, allow_empty=False, **kwargs)">
    <%call expr="longhand(name, **kwargs)">
        % if not gecko_only:
            use std::fmt;
            use values::HasViewportPercentage;
            use style_traits::ToCss;

            impl HasViewportPercentage for SpecifiedValue {
                fn has_viewport_percentage(&self) -> bool {
                    let &SpecifiedValue(ref vec) = self;
                    vec.iter().any(|ref x| x.has_viewport_percentage())
                }
            }

            pub mod single_value {
                use cssparser::Parser;
                use parser::{Parse, ParserContext, ParserContextExtraData};
                use properties::{CSSWideKeyword, DeclaredValue, ShorthandId};
                use values::computed::{Context, ToComputedValue};
                use values::{computed, specified};
                ${caller.body()}
            }

            /// The definition of the computed value for ${name}.
            pub mod computed_value {
                pub use super::single_value::computed_value as single_value;
                pub use self::single_value::T as SingleComputedValue;
                /// The computed value, effectively a list of single values.
                #[derive(Debug, Clone, PartialEq)]
                #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
                pub struct T(pub Vec<single_value::T>);
            }

            impl ToCss for computed_value::T {
                fn to_css<W>(&self, dest: &mut W) -> fmt::Result
                    where W: fmt::Write,
                {
                    let mut iter = self.0.iter();
                    if let Some(val) = iter.next() {
                        try!(val.to_css(dest));
                    } else {
                        % if allow_empty:
                            try!(dest.write_str("none"));
                        % else:
                            error!("Found empty value for property ${name}");
                        % endif
                    }
                    for i in iter {
                        try!(dest.write_str(", "));
                        try!(i.to_css(dest));
                    }
                    Ok(())
                }
            }

            /// The specified value of ${name}.
            #[derive(Debug, Clone, PartialEq)]
            #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
            pub struct SpecifiedValue(pub Vec<single_value::SpecifiedValue>);

            impl ToCss for SpecifiedValue {
                fn to_css<W>(&self, dest: &mut W) -> fmt::Result
                    where W: fmt::Write,
                {
                    let mut iter = self.0.iter();
                    if let Some(val) = iter.next() {
                        try!(val.to_css(dest));
                    } else {
                        % if allow_empty:
                            try!(dest.write_str("none"));
                        % else:
                            error!("Found empty value for property ${name}");
                        % endif
                    }
                    for i in iter {
                        try!(dest.write_str(", "));
                        try!(i.to_css(dest));
                    }
                    Ok(())
                }
            }

            pub fn get_initial_value() -> computed_value::T {
                % if allow_empty:
                    computed_value::T(vec![])
                % else:
                    computed_value::T(vec![single_value::get_initial_value()])
                % endif
            }

            pub fn parse(context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
                % if allow_empty:
                    if input.try(|input| input.expect_ident_matching("none")).is_ok() {
                        Ok(SpecifiedValue(Vec::new()))
                    } else {
                        input.parse_comma_separated(|parser| {
                            single_value::parse(context, parser)
                        }).map(SpecifiedValue)
                    }
                % else:
                    input.parse_comma_separated(|parser| {
                        single_value::parse(context, parser)
                    }).map(SpecifiedValue)
                % endif
            }

            pub use self::single_value::computed_value::T as SingleSpecifiedValue;

            impl ToComputedValue for SpecifiedValue {
                type ComputedValue = computed_value::T;

                #[inline]
                fn to_computed_value(&self, context: &Context) -> computed_value::T {
                    computed_value::T(self.0.iter().map(|x| x.to_computed_value(context)).collect())
                }
                #[inline]
                fn from_computed_value(computed: &computed_value::T) -> Self {
                    SpecifiedValue(computed.0.iter()
                                       .map(|x| ToComputedValue::from_computed_value(x))
                                       .collect())
                }
            }
        % else:
            ${caller.body()}
        % endif
    </%call>
</%def>

<%def name="raw_longhand(*args, **kwargs)">
    <%
        property = data.declare_longhand(*args, **kwargs)
        if property is None:
            return ""
    %>
    /// ${property.spec}
    pub mod ${property.ident} {
        #![allow(unused_imports)]
        % if not property.derived_from:
            use cssparser::Parser;
            use parser::{Parse, ParserContext, ParserContextExtraData};
            use properties::{CSSWideKeyword, DeclaredValue, ShorthandId};
        % endif
        use values::{Auto, Either, None_, Normal};
        use cascade_info::CascadeInfo;
        use error_reporting::ParseErrorReporter;
        use properties::longhands;
        use properties::property_bit_field::PropertyBitField;
        use properties::{ComputedValues, PropertyDeclaration};
        use properties::style_structs;
        use std::boxed::Box as StdBox;
        use std::collections::HashMap;
        use std::sync::Arc;
        use values::computed::{Context, ToComputedValue};
        use values::{computed, specified};
        use Atom;
        ${caller.body()}
        #[allow(unused_variables)]
        pub fn cascade_property(declaration: &PropertyDeclaration,
                                inherited_style: &ComputedValues,
                                default_style: &Arc<ComputedValues>,
                                context: &mut computed::Context,
                                seen: &mut PropertyBitField,
                                cacheable: &mut bool,
                                cascade_info: &mut Option<<&mut CascadeInfo>,
                                error_reporter: &mut StdBox<ParseErrorReporter + Send>) {
            let declared_value = match *declaration {
                PropertyDeclaration::${property.camel_case}(ref declared_value) => {
                    declared_value
                }
                _ => panic!("entered the wrong cascade_property() implementation"),
            };

            % if property.logical:
                let wm = context.style.writing_mode;
            % endif
            <% maybe_wm = "wm" if property.logical else "" %>
            <% maybe_physical = "_physical" if property.logical else "" %>
            % if not property.derived_from:
                if seen.get${maybe_physical}_${property.ident}(${maybe_wm}) {
                    return
                }
                seen.set${maybe_physical}_${property.ident}(${maybe_wm});
                {
                    let custom_props = context.style().custom_properties();
                    ::properties::substitute_variables_${property.ident}(
                        declared_value, &custom_props,
                    |value| {
                        if let Some(ref mut cascade_info) = *cascade_info {
                            cascade_info.on_cascade_property(&declaration,
                                                             &value);
                        }
                        <% maybe_wm = ", wm" if property.logical else "" %>
                        match *value {
                            DeclaredValue::Value(ref specified_value) => {
                                let computed = specified_value.to_computed_value(context);
                                % if property.has_uncacheable_values:
                                context.mutate_style().mutate_${data.current_style_struct.name_lower}()
                                                      .set_${property.ident}(computed, cacheable ${maybe_wm});
                                % else:
                                context.mutate_style().mutate_${data.current_style_struct.name_lower}()
                                                      .set_${property.ident}(computed ${maybe_wm});
                                % endif
                            }
                            DeclaredValue::WithVariables { .. } => unreachable!(),
                            % if not data.current_style_struct.inherited:
                            DeclaredValue::Unset |
                            % endif
                            DeclaredValue::Initial => {
                                // We assume that it's faster to use copy_*_from rather than
                                // set_*(get_initial_value());
                                let initial_struct = default_style
                                                      .get_${data.current_style_struct.name_lower}();
                                context.mutate_style().mutate_${data.current_style_struct.name_lower}()
                                                      .copy_${property.ident}_from(initial_struct ${maybe_wm});
                            },
                            % if data.current_style_struct.inherited:
                            DeclaredValue::Unset |
                            % endif
                            DeclaredValue::Inherit => {
                                // This is a bit slow, but this is rare so it shouldn't
                                // matter.
                                //
                                // FIXME: is it still?
                                *cacheable = false;
                                let inherited_struct =
                                    inherited_style.get_${data.current_style_struct.name_lower}();
                                context.mutate_style().mutate_${data.current_style_struct.name_lower}()
                                       .copy_${property.ident}_from(inherited_struct ${maybe_wm});
                            }
                        }
                    }, error_reporter);
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
                match input.try(|i| CSSWideKeyword::parse(context, i)) {
                    Ok(CSSWideKeyword::InheritKeyword) => Ok(DeclaredValue::Inherit),
                    Ok(CSSWideKeyword::InitialKeyword) => Ok(DeclaredValue::Initial),
                    Ok(CSSWideKeyword::UnsetKeyword) => Ok(DeclaredValue::Unset),
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

<%def name="single_keyword(name, values, vector=False, **kwargs)">
    <%call expr="single_keyword_computed(name, values, vector, **kwargs)">
        use values::computed::ComputedValueAsSpecified;
        use values::NoViewportPercentage;
        impl ComputedValueAsSpecified for SpecifiedValue {}
        impl NoViewportPercentage for SpecifiedValue {}
    </%call>
</%def>

<%def name="single_keyword_computed(name, values, vector=False, extra_specified=None, **kwargs)">
    <%
        keyword_kwargs = {a: kwargs.pop(a, None) for a in [
            'gecko_constant_prefix', 'gecko_enum_prefix',
            'extra_gecko_values', 'extra_servo_values',
            'custom_consts', 'gecko_inexhaustive',
        ]}
    %>

    <%def name="inner_body()">
        % if extra_specified:
            use style_traits::ToCss;
            define_css_keyword_enum! { SpecifiedValue:
                % for value in data.longhands_by_name[name].keyword.values_for(product) + extra_specified.split():
                    "${value}" => ${to_rust_ident(value)},
                % endfor
            }
        % else:
            pub use self::computed_value::T as SpecifiedValue;
        % endif
        pub mod computed_value {
            use style_traits::ToCss;
            define_css_keyword_enum! { T:
                % for value in data.longhands_by_name[name].keyword.values_for(product):
                    "${value}" => ${to_rust_ident(value)},
                % endfor
            }
        }
        #[inline]
        pub fn get_initial_value() -> computed_value::T {
            computed_value::T::${to_rust_ident(values.split()[0])}
        }
        #[inline]
        pub fn get_initial_specified_value() -> SpecifiedValue {
            SpecifiedValue::${to_rust_ident(values.split()[0])}
        }
        #[inline]
        pub fn parse(_context: &ParserContext, input: &mut Parser)
                     -> Result<SpecifiedValue, ()> {
            SpecifiedValue::parse(input)
        }
    </%def>
    % if vector:
        <%call expr="vector_longhand(name, keyword=Keyword(name, values, **keyword_kwargs), **kwargs)">
            ${inner_body()}
            ${caller.body()}
        </%call>
    % else:
        <%call expr="longhand(name, keyword=Keyword(name, values, **keyword_kwargs), **kwargs)">
            ${inner_body()}
            ${caller.body()}
        </%call>
    % endif
</%def>

<%def name="shorthand(name, sub_properties, experimental=False, **kwargs)">
<%
    shorthand = data.declare_shorthand(name, sub_properties.split(), experimental=experimental,
                                       **kwargs)
%>
    % if shorthand:
    /// ${shorthand.spec}
    pub mod ${shorthand.ident} {
        #[allow(unused_imports)]
        use cssparser::Parser;
        use parser::ParserContext;
        use properties::{longhands, PropertyDeclaration, DeclaredValue, ShorthandId};
        use std::fmt;
        use style_traits::ToCss;
        use super::{SerializeFlags, ALL_INHERIT, ALL_INITIAL, ALL_UNSET};

        pub struct Longhands {
            % for sub_property in shorthand.sub_properties:
                pub ${sub_property.ident}:
                    Option<longhands::${sub_property.ident}::SpecifiedValue>,
            % endfor
        }

        /// Represents a serializable set of all of the longhand properties that
        /// correspond to a shorthand.
        pub struct LonghandsToSerialize<'a> {
            % for sub_property in shorthand.sub_properties:
                pub ${sub_property.ident}: &'a DeclaredValue<longhands::${sub_property.ident}::SpecifiedValue>,
            % endfor
        }

        impl<'a> LonghandsToSerialize<'a> {
            /// Tries to get a serializable set of longhands given a set of
            /// property declarations.
            pub fn from_iter<I>(iter: I) -> Result<Self, ()>
                where I: Iterator<Item=&'a PropertyDeclaration>,
            {
                // Define all of the expected variables that correspond to the shorthand
                % for sub_property in shorthand.sub_properties:
                    let mut ${sub_property.ident} = None;
                % endfor

                // Attempt to assign the incoming declarations to the expected variables
                for longhand in iter {
                    match *longhand {
                        % for sub_property in shorthand.sub_properties:
                            PropertyDeclaration::${sub_property.camel_case}(ref value) => {
                                ${sub_property.ident} = Some(value)
                            },
                        % endfor
                        _ => {}
                    };
                }

                // If any of the expected variables are missing, return an error
                match (
                    % for sub_property in shorthand.sub_properties:
                        ${sub_property.ident},
                    % endfor
                ) {

                    (
                    % for sub_property in shorthand.sub_properties:
                        Some(${sub_property.ident}),
                    % endfor
                    ) =>
                    Ok(LonghandsToSerialize {
                        % for sub_property in shorthand.sub_properties:
                            ${sub_property.ident}: ${sub_property.ident},
                        % endfor
                    }),
                    _ => Err(())
                }
            }
        }

        impl<'a> ToCss for LonghandsToSerialize<'a> {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result
                where W: fmt::Write,
            {
                let mut all_flags = SerializeFlags::all();
                let mut with_variables = false;
                % for sub_property in shorthand.sub_properties:
                    match *self.${sub_property.ident} {
                        DeclaredValue::Initial => all_flags &= ALL_INITIAL,
                        DeclaredValue::Inherit => all_flags &= ALL_INHERIT,
                        DeclaredValue::Unset => all_flags &= ALL_UNSET,
                        DeclaredValue::WithVariables {..} => with_variables = true,
                        DeclaredValue::Value(..) => {
                            all_flags = SerializeFlags::empty();
                        }
                    }
                % endfor

                if with_variables {
                    // We don't serialize shorthands with variables
                    dest.write_str("")
                } else if all_flags == ALL_INHERIT {
                    dest.write_str("inherit")
                } else if all_flags == ALL_INITIAL {
                    dest.write_str("initial")
                } else if all_flags == ALL_UNSET {
                    dest.write_str("unset")
                } else {
                    self.to_css_declared(dest)
                }
            }
        }


        /// Parse the given shorthand and fill the result into the
        /// `declarations` vector.
        pub fn parse(context: &ParserContext,
                     input: &mut Parser,
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
                            from_shorthand: Some(ShorthandId::${shorthand.camel_case}),
                        }
                    ));
                % endfor
                Ok(())
            } else {
                Err(())
            }
        }

        ${caller.body()}
    }
    % endif
</%def>

<%def name="four_sides_shorthand(name, sub_property_pattern, parser_function, needs_context=True, **kwargs)">
    <% sub_properties=' '.join(sub_property_pattern % side for side in ['top', 'right', 'bottom', 'left']) %>
    <%call expr="self.shorthand(name, sub_properties=sub_properties, **kwargs)">
        #[allow(unused_imports)]
        use parser::Parse;
        use super::parse_four_sides;
        use values::specified;

        pub fn parse_value(context: &ParserContext, input: &mut Parser) -> Result<Longhands, ()> {
            let (top, right, bottom, left) =
            % if needs_context:
                try!(parse_four_sides(input, |i| ${parser_function}(context, i)));
            % else:
                try!(parse_four_sides(input, ${parser_function}));
                let _unused = context;
            % endif
            Ok(Longhands {
                % for side in ["top", "right", "bottom", "left"]:
                    ${to_rust_ident(sub_property_pattern % side)}: Some(${side}),
                % endfor
            })
        }

        impl<'a> LonghandsToSerialize<'a> {
            fn to_css_declared<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                super::serialize_four_sides(
                    dest,
                    self.${to_rust_ident(sub_property_pattern % 'top')},
                    self.${to_rust_ident(sub_property_pattern % 'right')},
                    self.${to_rust_ident(sub_property_pattern % 'bottom')},
                    self.${to_rust_ident(sub_property_pattern % 'left')}
                )
            }
        }
    </%call>
</%def>

<%def name="logical_setter_helper(name)">
    <%
        side = None
        size = None
        maybe_side = [s for s in LOGICAL_SIDES if s in name]
        maybe_size = [s for s in LOGICAL_SIZES if s in name]
        if len(maybe_side) == 1:
            side = maybe_side[0]
        elif len(maybe_size) == 1:
            size = maybe_size[0]
        def phys_ident(side, phy_side):
            return to_rust_ident(name.replace(side, phy_side).replace("offset-", ""))
    %>
    % if side is not None:
        use logical_geometry::PhysicalSide;
        match wm.${to_rust_ident(side)}_physical_side() {
            % for phy_side in PHYSICAL_SIDES:
                PhysicalSide::${phy_side.title()} => {
                    ${caller.inner(physical_ident=phys_ident(side, phy_side))}
                }
            % endfor
        }
    % elif size is not None:
        <%
            # (horizontal, vertical)
            physical_size = ("height", "width")
            if size == "inline-size":
                physical_size = ("width", "height")
        %>
        if wm.is_vertical() {
            ${caller.inner(physical_ident=phys_ident(size, physical_size[1]))}
        } else {
            ${caller.inner(physical_ident=phys_ident(size, physical_size[0]))}
        }
    % else:
        <% raise Exception("Don't know what to do with logical property %s" % name) %>
    % endif
</%def>

<%def name="logical_setter(name, need_clone=False)">
    /// Set the appropriate physical property for ${name} given a writing mode.
    pub fn set_${to_rust_ident(name)}(&mut self,
                                      v: longhands::${to_rust_ident(name)}::computed_value::T,
                                      wm: WritingMode) {
        <%self:logical_setter_helper name="${name}">
            <%def name="inner(physical_ident)">
                self.set_${physical_ident}(v)
            </%def>
        </%self:logical_setter_helper>
    }

    /// Copy the appropriate physical property from another struct for ${name}
    /// given a writing mode.
    pub fn copy_${to_rust_ident(name)}_from(&mut self,
                                            other: &Self,
                                            wm: WritingMode) {
        <%self:logical_setter_helper name="${name}">
            <%def name="inner(physical_ident)">
                self.copy_${physical_ident}_from(other)
            </%def>
        </%self:logical_setter_helper>
    }
    % if need_clone:
        /// Get the computed value for the appropriate physical property for
        /// ${name} given a writing mode.
        pub fn clone_${to_rust_ident(name)}(&self, wm: WritingMode)
            -> longhands::${to_rust_ident(name)}::computed_value::T {
        <%self:logical_setter_helper name="${name}">
            <%def name="inner(physical_ident)">
                self.clone_${physical_ident}()
            </%def>
        </%self:logical_setter_helper>
        }
    % endif
</%def>
