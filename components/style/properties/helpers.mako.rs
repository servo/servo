/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%!
    from data import Keyword, to_rust_ident, to_camel_case
    from data import LOGICAL_SIDES, PHYSICAL_SIDES, LOGICAL_SIZES, SYSTEM_FONT_LONGHANDS
%>

<%def name="predefined_type(name, type, initial_value, parse_method='parse',
            needs_context=True, vector=False, computed_type=None, initial_specified_value=None,
            allow_quirks=False, allow_empty=False, **kwargs)">
    <%def name="predefined_type_inner(name, type, initial_value, parse_method)">
        #[allow(unused_imports)]
        use app_units::Au;
        #[allow(unused_imports)]
        use cssparser::{Color as CSSParserColor, RGBA};
        #[allow(unused_imports)]
        use values::specified::AllowQuirks;
        #[allow(unused_imports)]
        use smallvec::SmallVec;
        pub use values::specified::${type} as SpecifiedValue;
        pub mod computed_value {
            % if computed_type:
            pub use ${computed_type} as T;
            % else:
            pub use values::computed::${type} as T;
            % endif
        }
        % if initial_value:
        #[inline] pub fn get_initial_value() -> computed_value::T { ${initial_value} }
        % endif
        % if initial_specified_value:
        #[inline] pub fn get_initial_specified_value() -> SpecifiedValue { ${initial_specified_value} }
        % endif
        #[allow(unused_variables)]
        #[inline]
        pub fn parse<'i, 't>(context: &ParserContext,
                             input: &mut Parser<'i, 't>)
                             -> Result<SpecifiedValue, ParseError<'i>> {
            % if allow_quirks:
            specified::${type}::${parse_method}_quirky(context, input, AllowQuirks::Yes)
            % elif needs_context:
            specified::${type}::${parse_method}(context, input)
            % else:
            specified::${type}::${parse_method}(input)
            % endif
        }
    </%def>
    % if vector:
        <%call
            expr="vector_longhand(name, predefined_type=type, allow_empty=allow_empty or not initial_value, **kwargs)"
        >
            ${predefined_type_inner(name, type, initial_value, parse_method)}
            % if caller:
            ${caller.body()}
            % endif
        </%call>
    % else:
        <%call expr="longhand(name, predefined_type=type, **kwargs)">
            ${predefined_type_inner(name, type, initial_value, parse_method)}
            % if caller:
            ${caller.body()}
            % endif
        </%call>
    % endif
</%def>

// FIXME (Manishearth): Add computed_value_as_specified argument
// and handle the empty case correctly
<%doc>
    To be used in cases where we have a grammar like "<thing> [ , <thing> ]*".

    Setting allow_empty to False allows for cases where the vector
    is empty. The grammar for these is usually "none | <thing> [ , <thing> ]*".
    We assume that the default/initial value is an empty vector for these.
    `initial_value` need not be defined for these.
</%doc>
<%def name="vector_longhand(name, animation_value_type=None, allow_empty=False, separator='Comma', **kwargs)">
    <%call expr="longhand(name, animation_value_type=animation_value_type, vector=True, **kwargs)">
        #[allow(unused_imports)]
        use smallvec::SmallVec;
        use std::fmt;
        #[allow(unused_imports)]
        use style_traits::HasViewportPercentage;
        use style_traits::{Separator, ToCss};

        pub mod single_value {
            #[allow(unused_imports)]
            use cssparser::{Parser, BasicParseError};
            #[allow(unused_imports)]
            use parser::{Parse, ParserContext};
            #[allow(unused_imports)]
            use properties::ShorthandId;
            #[allow(unused_imports)]
            use selectors::parser::SelectorParseError;
            #[allow(unused_imports)]
            use style_traits::{ParseError, StyleParseError};
            #[allow(unused_imports)]
            use values::computed::{Context, ToComputedValue};
            #[allow(unused_imports)]
            use values::{computed, specified};
            #[allow(unused_imports)]
            use values::{Auto, Either, None_, Normal};
            ${caller.body()}
        }

        /// The definition of the computed value for ${name}.
        pub mod computed_value {
            pub use super::single_value::computed_value as single_value;
            pub use self::single_value::T as SingleComputedValue;
            % if allow_empty and allow_empty != "NotInitial":
            use std::vec::IntoIter;
            % else:
            use smallvec::{IntoIter, SmallVec};
            % endif
            use values::computed::ComputedVecIter;

            /// The computed value, effectively a list of single values.
            #[derive(Debug, Clone, PartialEq)]
            #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
            pub struct T(
                % if allow_empty and allow_empty != "NotInitial":
                pub Vec<single_value::T>,
                % else:
                pub SmallVec<[single_value::T; 1]>,
                % endif
            );

            % if animation_value_type == "ComputedValue":
                use properties::animated_properties::Animatable;
                use values::animated::ToAnimatedZero;

                impl Animatable for T {
                    fn add_weighted(&self, other: &Self, self_portion: f64, other_portion: f64)
                        -> Result<Self, ()> {
                        self.0.add_weighted(&other.0, self_portion, other_portion).map(T)
                    }

                    fn add(&self, other: &Self) -> Result<Self, ()> {
                        self.0.add(&other.0).map(T)
                    }

                    #[inline]
                    fn compute_distance(&self, other: &Self) -> Result<f64, ()> {
                        self.0.compute_distance(&other.0)
                    }

                    #[inline]
                    fn compute_squared_distance(&self, other: &Self) -> Result<f64, ()> {
                        self.0.compute_squared_distance(&other.0)
                    }
                }

                impl ToAnimatedZero for T {
                    #[inline]
                    fn to_animated_zero(&self) -> Result<Self, ()> { Err(()) }
                }
            % endif

            pub type Iter<'a, 'cx, 'cx_a> = ComputedVecIter<'a, 'cx, 'cx_a, super::single_value::SpecifiedValue>;

            impl IntoIterator for T {
                type Item = single_value::T;
                % if allow_empty and allow_empty != "NotInitial":
                type IntoIter = IntoIter<single_value::T>;
                % else:
                type IntoIter = IntoIter<[single_value::T; 1]>;
                % endif
                fn into_iter(self) -> Self::IntoIter {
                    self.0.into_iter()
                }
            }
        }

        impl ToCss for computed_value::T {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result
                where W: fmt::Write,
            {
                let mut iter = self.0.iter();
                if let Some(val) = iter.next() {
                    val.to_css(dest)?;
                } else {
                    % if allow_empty:
                        dest.write_str("none")?;
                    % else:
                        warn!("Found empty value for property ${name}");
                    % endif
                }
                for i in iter {
                    dest.write_str(::style_traits::${separator}::separator())?;
                    i.to_css(dest)?;
                }
                Ok(())
            }
        }

        /// The specified value of ${name}.
        #[derive(Clone, Debug, HasViewportPercentage, PartialEq)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub struct SpecifiedValue(pub Vec<single_value::SpecifiedValue>);

        impl ToCss for SpecifiedValue {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result
                where W: fmt::Write,
            {
                let mut iter = self.0.iter();
                if let Some(val) = iter.next() {
                    val.to_css(dest)?;
                } else {
                    % if allow_empty:
                        dest.write_str("none")?;
                    % else:
                        warn!("Found empty value for property ${name}");
                    % endif
                }
                for i in iter {
                    dest.write_str(::style_traits::${separator}::separator())?;
                    i.to_css(dest)?;
                }
                Ok(())
            }
        }

        pub fn get_initial_value() -> computed_value::T {
            % if allow_empty and allow_empty != "NotInitial":
                computed_value::T(vec![])
            % else:
                let mut v = SmallVec::new();
                v.push(single_value::get_initial_value());
                computed_value::T(v)
            % endif
        }

        pub fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>)
                                -> Result<SpecifiedValue, ParseError<'i>> {
            use style_traits::Separator;

            % if allow_empty:
                if input.try(|input| input.expect_ident_matching("none")).is_ok() {
                    return Ok(SpecifiedValue(Vec::new()))
                }
            % endif

            ::style_traits::${separator}::parse(input, |parser| {
                single_value::parse(context, parser)
            }).map(SpecifiedValue)
        }

        pub use self::single_value::SpecifiedValue as SingleSpecifiedValue;

        impl SpecifiedValue {
            pub fn compute_iter<'a, 'cx, 'cx_a>(&'a self, context: &'cx Context<'cx_a>)
                -> computed_value::Iter<'a, 'cx, 'cx_a> {
                computed_value::Iter::new(context, &self.0)
            }
        }

        impl ToComputedValue for SpecifiedValue {
            type ComputedValue = computed_value::T;

            #[inline]
            fn to_computed_value(&self, context: &Context) -> computed_value::T {
                computed_value::T(self.compute_iter(context).collect())
            }

            #[inline]
            fn from_computed_value(computed: &computed_value::T) -> Self {
                SpecifiedValue(computed.0.iter()
                                    .map(ToComputedValue::from_computed_value)
                                    .collect())
            }
        }
    </%call>
</%def>
<%def name="longhand(*args, **kwargs)">
    <%
        property = data.declare_longhand(*args, **kwargs)
        if property is None:
            return ""
    %>
    /// ${property.spec}
    pub mod ${property.ident} {
        % if not property.derived_from:
            #[allow(unused_imports)]
            use cssparser::{Parser, BasicParseError, Token};
            #[allow(unused_imports)]
            use parser::{Parse, ParserContext};
            #[allow(unused_imports)]
            use properties::{UnparsedValue, ShorthandId};
        % endif
        #[allow(unused_imports)]
        use values::{Auto, Either, None_, Normal};
        #[allow(unused_imports)]
        use cascade_info::CascadeInfo;
        #[allow(unused_imports)]
        use error_reporting::ParseErrorReporter;
        #[allow(unused_imports)]
        use properties::longhands;
        #[allow(unused_imports)]
        use properties::{DeclaredValue, LonghandId, LonghandIdSet};
        #[allow(unused_imports)]
        use properties::{CSSWideKeyword, ComputedValues, PropertyDeclaration};
        #[allow(unused_imports)]
        use properties::style_structs;
        #[allow(unused_imports)]
        use selectors::parser::SelectorParseError;
        #[allow(unused_imports)]
        use servo_arc::Arc;
        #[allow(unused_imports)]
        use style_traits::{ParseError, StyleParseError};
        #[allow(unused_imports)]
        use values::computed::{Context, ToComputedValue};
        #[allow(unused_imports)]
        use values::{computed, generics, specified};
        #[allow(unused_imports)]
        use Atom;
        ${caller.body()}
        #[allow(unused_variables)]
        pub fn cascade_property(
            declaration: &PropertyDeclaration,
            context: &mut computed::Context,
            cascade_info: &mut Option<<&mut CascadeInfo>,
        ) {
            let value = match *declaration {
                PropertyDeclaration::${property.camel_case}(ref value) => {
                    DeclaredValue::Value(value)
                },
                PropertyDeclaration::CSSWideKeyword(id, value) => {
                    debug_assert!(id == LonghandId::${property.camel_case});
                    DeclaredValue::CSSWideKeyword(value)
                },
                PropertyDeclaration::WithVariables(..) => {
                    panic!("variables should already have been substituted")
                }
                _ => panic!("entered the wrong cascade_property() implementation"),
            };

            % if not property.derived_from:
                if let Some(ref mut cascade_info) = *cascade_info {
                    cascade_info.on_cascade_property(&declaration, &value);
                }
                match value {
                    DeclaredValue::Value(ref specified_value) => {
                        % if property.ident in SYSTEM_FONT_LONGHANDS and product == "gecko":
                            if let Some(sf) = specified_value.get_system() {
                                longhands::system_font::resolve_system_font(sf, context);
                            }
                        % endif
                        % if property.is_vector:
                            // In the case of a vector property we want to pass
                            // down an iterator so that this can be computed
                            // without allocation
                            //
                            // However, computing requires a context, but the
                            // style struct being mutated is on the context. We
                            // temporarily remove it, mutate it, and then put it
                            // back. Vector longhands cannot touch their own
                            // style struct whilst computing, else this will
                            // panic.
                            let mut s =
                                context.builder.take_${data.current_style_struct.name_lower}();
                            {
                                let iter = specified_value.compute_iter(context);
                                s.set_${property.ident}(iter);
                            }
                            context.builder.put_${data.current_style_struct.name_lower}(s);
                        % else:
                            let computed = specified_value.to_computed_value(context);
                            % if property.ident == "font_size":
                                 longhands::font_size::cascade_specified_font_size(
                                     context,
                                     &specified_value,
                                     computed,
                                 );
                            % else:
                                context.builder.set_${property.ident}(computed)
                            % endif
                        % endif
                    }
                    DeclaredValue::WithVariables(_) => unreachable!(),
                    DeclaredValue::CSSWideKeyword(keyword) => match keyword {
                        % if not data.current_style_struct.inherited:
                        CSSWideKeyword::Unset |
                        % endif
                        CSSWideKeyword::Initial => {
                            % if property.ident == "font_size":
                                longhands::font_size::cascade_initial_font_size(context);
                            % else:
                                context.builder.reset_${property.ident}();
                            % endif
                        },
                        % if data.current_style_struct.inherited:
                        CSSWideKeyword::Unset |
                        % endif
                        CSSWideKeyword::Inherit => {
                            % if property.ident == "font_size":
                                longhands::font_size::cascade_inherit_font_size(context);
                            % else:
                                context.builder.inherit_${property.ident}();
                            % endif
                        }
                    }
                }

                % if property.custom_cascade:
                    cascade_property_custom(declaration, context);
                % endif
            % else:
                // Do not allow stylesheets to set derived properties.
            % endif
        }
        % if not property.derived_from:
            pub fn parse_specified<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>)
                % if property.boxed:
                                   -> Result<Box<SpecifiedValue>, ParseError<'i>> {
                    parse(context, input).map(|result| Box::new(result))
                % else:
                                   -> Result<SpecifiedValue, ParseError<'i>> {
                    % if property.allow_quirks:
                        parse_quirky(context, input, specified::AllowQuirks::Yes)
                    % else:
                        parse(context, input)
                    % endif
                % endif
            }
            pub fn parse_declared<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>)
                                          -> Result<PropertyDeclaration, ParseError<'i>> {
                parse_specified(context, input).map(PropertyDeclaration::${property.camel_case})
            }
        % endif
    }
</%def>

<%def name="single_keyword_system(name, values, **kwargs)">
    <%
        keyword_kwargs = {a: kwargs.pop(a, None) for a in [
            'gecko_constant_prefix', 'gecko_enum_prefix',
            'extra_gecko_values', 'extra_servo_values',
            'custom_consts', 'gecko_inexhaustive',
        ]}
        keyword = keyword=Keyword(name, values, **keyword_kwargs)
    %>
    <%call expr="longhand(name, keyword=Keyword(name, values, **keyword_kwargs), **kwargs)">
        use properties::longhands::system_font::SystemFont;
        no_viewport_percentage!(SpecifiedValue);

        pub mod computed_value {
            use cssparser::Parser;
            use parser::{Parse, ParserContext};

            use style_traits::{ToCss, ParseError};
            define_css_keyword_enum! { T:
                % for value in keyword.values_for(product):
                    "${value}" => ${to_rust_ident(value)},
                % endfor
            }

            impl Parse for T {
                fn parse<'i, 't>(_: &ParserContext, input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
                    T::parse(input)
                }
            }

            ${gecko_keyword_conversion(keyword, keyword.values_for(product), type="T", cast_to="i32")}
        }

        #[derive(Debug, Clone, PartialEq, Eq, Copy, ToCss)]
        pub enum SpecifiedValue {
            Keyword(computed_value::T),
            System(SystemFont),
        }

        pub fn parse<'i, 't>(_: &ParserContext, input: &mut Parser<'i, 't>) -> Result<SpecifiedValue, ParseError<'i>> {
            Ok(SpecifiedValue::Keyword(computed_value::T::parse(input)?))
        }

        impl ToComputedValue for SpecifiedValue {
            type ComputedValue = computed_value::T;
            fn to_computed_value(&self, _cx: &Context) -> Self::ComputedValue {
                match *self {
                    SpecifiedValue::Keyword(v) => v,
                    SpecifiedValue::System(_) => {
                        % if product == "gecko":
                            _cx.cached_system_font.as_ref().unwrap().${to_rust_ident(name)}
                        % else:
                            unreachable!()
                        % endif
                    }
                }
            }
            fn from_computed_value(other: &computed_value::T) -> Self {
                SpecifiedValue::Keyword(*other)
            }
        }

        impl From<computed_value::T> for SpecifiedValue {
            fn from(other: computed_value::T) -> Self {
                SpecifiedValue::Keyword(other)
            }
        }

        #[inline]
        pub fn get_initial_value() -> computed_value::T {
            computed_value::T::${to_rust_ident(values.split()[0])}
        }
        #[inline]
        pub fn get_initial_specified_value() -> SpecifiedValue {
            SpecifiedValue::Keyword(computed_value::T::${to_rust_ident(values.split()[0])})
        }

        impl SpecifiedValue {
            pub fn system_font(f: SystemFont) -> Self {
                SpecifiedValue::System(f)
            }
            pub fn get_system(&self) -> Option<SystemFont> {
                if let SpecifiedValue::System(s) = *self {
                    Some(s)
                } else {
                    None
                }
            }
        }
    </%call>
</%def>

<%def name="single_keyword(name, values, vector=False, **kwargs)">
    <%call expr="single_keyword_computed(name, values, vector, **kwargs)">
        % if not "extra_specified" in kwargs and ("aliases" in kwargs or (("extra_%s_aliases" % product) in kwargs)):
            impl ToComputedValue for SpecifiedValue {
                type ComputedValue = computed_value::T;

                #[inline]
                fn to_computed_value(&self, _context: &Context) -> computed_value::T {
                    match *self {
                        % for value in data.longhands_by_name[name].keyword.values_for(product):
                            SpecifiedValue::${to_rust_ident(value)} => computed_value::T::${to_rust_ident(value)},
                        % endfor
                    }
                }
                #[inline]
                fn from_computed_value(computed: &computed_value::T) -> Self {
                    match *computed {
                        % for value in data.longhands_by_name[name].keyword.values_for(product):
                            computed_value::T::${to_rust_ident(value)} => SpecifiedValue::${to_rust_ident(value)},
                        % endfor
                    }
                }
            }
        % else:
            use values::computed::ComputedValueAsSpecified;
            impl ComputedValueAsSpecified for SpecifiedValue {}
        % endif

        no_viewport_percentage!(SpecifiedValue);
    </%call>
</%def>

<%def name="gecko_keyword_conversion(keyword, values=None, type='SpecifiedValue', cast_to=None)">
    <%
        if not values:
            values = keyword.values_for(product)
        maybe_cast = "as %s" % cast_to if cast_to else ""
        const_type = cast_to if cast_to else "u32"
    %>
    #[cfg(feature = "gecko")]
    impl ${type} {
        /// Obtain a specified value from a Gecko keyword value
        ///
        /// Intended for use with presentation attributes, not style structs
        pub fn from_gecko_keyword(kw: u32) -> Self {
            use gecko_bindings::structs;
            % for value in values:
                // We can't match on enum values if we're matching on a u32
                const ${to_rust_ident(value).upper()}: ${const_type}
                    = structs::${keyword.gecko_constant(value)} as ${const_type};
            % endfor
            match kw ${maybe_cast} {
                % for value in values:
                    ${to_rust_ident(value).upper()} => ${type}::${to_rust_ident(value)},
                % endfor
                x => panic!("Found unexpected value in style struct for ${keyword.name} property: {:?}", x),
            }
        }
    }
</%def>

<%def name="gecko_bitflags_conversion(bit_map, gecko_bit_prefix, type, kw_type='u8')">
    #[cfg(feature = "gecko")]
    impl ${type} {
        /// Obtain a specified value from a Gecko keyword value
        ///
        /// Intended for use with presentation attributes, not style structs
        pub fn from_gecko_keyword(kw: ${kw_type}) -> Self {
            % for gecko_bit in bit_map.values():
            use gecko_bindings::structs::${gecko_bit_prefix}${gecko_bit};
            % endfor

            let mut bits = ${type}::empty();
            % for servo_bit, gecko_bit in bit_map.iteritems():
                if kw & (${gecko_bit_prefix}${gecko_bit} as ${kw_type}) != 0 {
                    bits |= ${servo_bit};
                }
            % endfor
            bits
        }

        pub fn to_gecko_keyword(self) -> ${kw_type} {
            % for gecko_bit in bit_map.values():
            use gecko_bindings::structs::${gecko_bit_prefix}${gecko_bit};
            % endfor

            let mut bits: ${kw_type} = 0;
            // FIXME: if we ensure that the Servo bitflags storage is the same
            // as Gecko's one, we can just copy it.
            % for servo_bit, gecko_bit in bit_map.iteritems():
                if self.contains(${servo_bit}) {
                    bits |= ${gecko_bit_prefix}${gecko_bit} as ${kw_type};
                }
            % endfor
            bits
        }
    }
</%def>

<%def name="single_keyword_computed(name, values, vector=False,
            extra_specified=None, needs_conversion=False, **kwargs)">
    <%
        keyword_kwargs = {a: kwargs.pop(a, None) for a in [
            'gecko_constant_prefix', 'gecko_enum_prefix',
            'extra_gecko_values', 'extra_servo_values',
            'aliases', 'extra_gecko_aliases', 'extra_servo_aliases',
            'custom_consts', 'gecko_inexhaustive', 'gecko_strip_moz_prefix',
        ]}
    %>

    <%def name="inner_body(keyword, extra_specified=None, needs_conversion=False)">
        % if extra_specified or keyword.aliases_for(product):
            use style_traits::ToCss;
            define_css_keyword_enum! { SpecifiedValue:
                values {
                    % for value in keyword.values_for(product) + (extra_specified or "").split():
                        "${value}" => ${to_rust_ident(value)},
                    % endfor
                }
                aliases {
                    % for alias, value in keyword.aliases_for(product).iteritems():
                        "${alias}" => ${to_rust_ident(value)},
                    % endfor
                }
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
        pub fn parse<'i, 't>(_context: &ParserContext, input: &mut Parser<'i, 't>)
                             -> Result<SpecifiedValue, ParseError<'i>> {
            SpecifiedValue::parse(input)
        }
        impl Parse for SpecifiedValue {
            #[inline]
            fn parse<'i, 't>(_context: &ParserContext, input: &mut Parser<'i, 't>)
                             -> Result<SpecifiedValue, ParseError<'i>> {
                SpecifiedValue::parse(input)
            }
        }

        % if needs_conversion:
            <%
                conversion_values = keyword.values_for(product)
                if extra_specified:
                    conversion_values += extra_specified.split()
                conversion_values += keyword.aliases_for(product).keys()
            %>
            ${gecko_keyword_conversion(keyword, values=conversion_values)}
        % endif
    </%def>
    % if vector:
        <%call expr="vector_longhand(name, keyword=Keyword(name, values, **keyword_kwargs), **kwargs)">
            ${inner_body(Keyword(name, values, **keyword_kwargs))}
            ${caller.body()}
        </%call>
    % else:
        <%call expr="longhand(name, keyword=Keyword(name, values, **keyword_kwargs), **kwargs)">
            ${inner_body(Keyword(name, values, **keyword_kwargs),
                         extra_specified=extra_specified, needs_conversion=needs_conversion)}
            ${caller.body()}
        </%call>
    % endif
</%def>

<%def name="shorthand(name, sub_properties, experimental=False, derive_serialize=False, **kwargs)">
<%
    shorthand = data.declare_shorthand(name, sub_properties.split(), experimental=experimental,
                                       **kwargs)
%>
    % if shorthand:
    /// ${shorthand.spec}
    pub mod ${shorthand.ident} {
        use cssparser::Parser;
        use parser::ParserContext;
        use properties::{PropertyDeclaration, SourcePropertyDeclaration, MaybeBoxed, longhands};
        #[allow(unused_imports)]
        use selectors::parser::SelectorParseError;
        #[allow(unused_imports)]
        use std::fmt;
        #[allow(unused_imports)]
        use style_traits::{ParseError, StyleParseError};
        #[allow(unused_imports)]
        use style_traits::ToCss;

        pub struct Longhands {
            % for sub_property in shorthand.sub_properties:
                pub ${sub_property.ident}:
                    % if sub_property.boxed:
                        Box<
                    % endif
                    longhands::${sub_property.ident}::SpecifiedValue
                    % if sub_property.boxed:
                        >
                    % endif
                    ,
            % endfor
        }

        /// Represents a serializable set of all of the longhand properties that
        /// correspond to a shorthand.
        % if derive_serialize:
        #[derive(ToCss)]
        % endif
        pub struct LonghandsToSerialize<'a> {
            % for sub_property in shorthand.sub_properties:
                pub ${sub_property.ident}:
                    &'a longhands::${sub_property.ident}::SpecifiedValue,
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

        /// Parse the given shorthand and fill the result into the
        /// `declarations` vector.
        pub fn parse_into<'i, 't>(declarations: &mut SourcePropertyDeclaration,
                                  context: &ParserContext, input: &mut Parser<'i, 't>)
                                  -> Result<(), ParseError<'i>> {
            input.parse_entirely(|input| parse_value(context, input)).map(|longhands| {
                % for sub_property in shorthand.sub_properties:
                    declarations.push(PropertyDeclaration::${sub_property.camel_case}(
                        longhands.${sub_property.ident}
                    ));
                % endfor
            })
        }

        ${caller.body()}
    }
    % endif
</%def>

<%def name="four_sides_shorthand(name, sub_property_pattern, parser_function,
                                 needs_context=True, allow_quirks=False, **kwargs)">
    <% sub_properties=' '.join(sub_property_pattern % side for side in ['top', 'right', 'bottom', 'left']) %>
    <%call expr="self.shorthand(name, sub_properties=sub_properties, **kwargs)">
        #[allow(unused_imports)]
        use parser::Parse;
        use values::generics::rect::Rect;
        use values::specified;

        pub fn parse_value<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>)
                                   -> Result<Longhands, ParseError<'i>> {
            let rect = Rect::parse_with(context, input, |_c, i| {
            % if allow_quirks:
                ${parser_function}_quirky(_c, i, specified::AllowQuirks::Yes)
            % elif needs_context:
                ${parser_function}(_c, i)
            % else:
                ${parser_function}(i)
            % endif
            })?;
            Ok(expanded! {
                % for index, side in enumerate(["top", "right", "bottom", "left"]):
                    ${to_rust_ident(sub_property_pattern % side)}: rect.${index},
                % endfor
            })
        }

        impl<'a> ToCss for LonghandsToSerialize<'a> {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                let rect = Rect::new(
                    % for side in ["top", "right", "bottom", "left"]:
                    &self.${to_rust_ident(sub_property_pattern % side)},
                    % endfor
                );
                rect.to_css(dest)
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

    /// Copy the appropriate physical property from another struct for ${name}
    /// given a writing mode.
    pub fn reset_${to_rust_ident(name)}(&mut self,
                                        other: &Self,
                                        wm: WritingMode) {
        self.copy_${to_rust_ident(name)}_from(other, wm)
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

<%def name="alias_to_nscsspropertyid(alias)">
    <%
        return "nsCSSPropertyID::eCSSPropertyAlias_%s" % to_camel_case(alias)
    %>
</%def>

<%def name="to_nscsspropertyid(ident)">
    <%
        if ident == "float":
            ident = "float_"
        return "nsCSSPropertyID::eCSSProperty_%s" % ident
    %>
</%def>

/// Macro for defining Animatable trait for tuple struct which has Option<T>,
/// e.g. struct T(pub Option<Au>).
<%def name="impl_animatable_for_option_tuple(value_for_none)">
    impl Animatable for T {
        #[inline]
        fn add_weighted(&self, other: &Self, self_portion: f64, other_portion: f64)
            -> Result<Self, ()> {
            match (self, other) {
                (&T(Some(ref this)), &T(Some(ref other))) => {
                    Ok(T(this.add_weighted(other, self_portion, other_portion).ok()))
                },
                (&T(Some(ref this)), &T(None)) => {
                    Ok(T(this.add_weighted(&${value_for_none}, self_portion, other_portion).ok()))
                },
                (&T(None), &T(Some(ref other))) => {
                    Ok(T(${value_for_none}.add_weighted(other, self_portion, other_portion).ok()))
                },
                (&T(None), &T(None)) => {
                    Ok(T(None))
                },
            }
        }

        #[inline]
        fn compute_distance(&self, other: &Self) -> Result<f64, ()> {
            match (self, other) {
                (&T(Some(ref this)), &T(Some(ref other))) => {
                    this.compute_distance(other)
                },
                (&T(Some(ref value)), &T(None)) |
                (&T(None), &T(Some(ref value)))=> {
                    value.compute_distance(&${value_for_none})
                },
                (&T(None), &T(None)) => {
                    Ok(0.0)
                },
            }
        }

        #[inline]
        fn compute_squared_distance(&self, other: &Self) -> Result<f64, ()> {
            match (self, other) {
                (&T(Some(ref this)), &T(Some(ref other))) => {
                    this.compute_squared_distance(other)
                },
                (&T(Some(ref value)), &T(None)) |
                (&T(None), &T(Some(ref value))) => {
                    value.compute_squared_distance(&${value_for_none})
                },
                (&T(None), &T(None)) => {
                    Ok(0.0)
                },
            }
        }
    }
</%def>

// Define property that supports prefixed intrinsic size keyword values for gecko.
// E.g. -moz-max-content, -moz-min-content, etc.
<%def name="gecko_size_type(name, length_type, initial_value, logical, **kwargs)">
    <%call expr="longhand(name,
                          predefined_type=length_type,
                          logical=logical,
                          **kwargs)">
        % if not logical:
            use values::specified::AllowQuirks;
        % endif
        use values::specified::${length_type};

        pub mod computed_value {
            pub type T = ::values::computed::${length_type};
        }

        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        #[derive(Clone, Debug, HasViewportPercentage, PartialEq, ToCss)]
        pub struct SpecifiedValue(pub ${length_type});

        % if length_type == "MozLength":
        impl SpecifiedValue {
            /// Returns the `auto` value.
            pub fn auto() -> Self {
                use values::specified::length::LengthOrPercentageOrAuto;
                SpecifiedValue(MozLength::LengthOrPercentageOrAuto(LengthOrPercentageOrAuto::Auto))
            }

            /// Returns a value representing a `0` length.
            pub fn zero() -> Self {
                use values::specified::length::LengthOrPercentageOrAuto;
                SpecifiedValue(MozLength::LengthOrPercentageOrAuto(LengthOrPercentageOrAuto::zero()))
            }

            /// Returns a value representing a `0%` length.
            pub fn zero_percent() -> Self {
                use values::specified::length::LengthOrPercentageOrAuto;
                SpecifiedValue(MozLength::LengthOrPercentageOrAuto(LengthOrPercentageOrAuto::zero_percent()))
            }
        }
        % endif

        #[inline]
        pub fn get_initial_value() -> computed_value::T {
            use values::computed::${length_type};
            ${length_type}::${initial_value}
        }
        fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>)
                         -> Result<SpecifiedValue, ParseError<'i>> {
            % if logical:
            let ret = ${length_type}::parse(context, input);
            % else:
            let ret = ${length_type}::parse_quirky(context, input, AllowQuirks::Yes);
            % endif
            // Keyword values don't make sense in the block direction; don't parse them
            % if "block" in name:
                if let Ok(${length_type}::ExtremumLength(..)) = ret {
                    return Err(StyleParseError::UnspecifiedError.into())
                }
            % endif
            ret.map(SpecifiedValue)
        }

        impl ToComputedValue for SpecifiedValue {
            type ComputedValue = computed_value::T;
            #[inline]
            fn to_computed_value(&self, context: &Context) -> computed_value::T {
                % if not logical or "block" in name:
                    use values::computed::${length_type};
                % endif
                let computed = self.0.to_computed_value(context);

                // filter out keyword values in the block direction
                % if logical:
                    % if "block" in name:
                        if let ${length_type}::ExtremumLength(..) = computed {
                            return get_initial_value()
                        }
                    % endif
                % else:
                    if let ${length_type}::ExtremumLength(..) = computed {
                        <% is_height = "true" if "height" in name else "false" %>
                        if ${is_height} != context.builder.writing_mode.is_vertical() {
                            return get_initial_value()
                        }
                    }
                % endif
                computed
            }

            #[inline]
            fn from_computed_value(computed: &computed_value::T) -> Self {
                SpecifiedValue(ToComputedValue::from_computed_value(computed))
            }
        }
    </%call>
</%def>

