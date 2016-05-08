/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />
<% from data import Method %>

<% data.new_style_struct("Outline",
                         inherited=False,
                         additional_methods=[Method("outline_has_nonzero_width", "bool")]) %>

// TODO(pcwalton): `invert`
${helpers.predefined_type("outline-color", "CSSColor", "::cssparser::Color::CurrentColor")}

<%helpers:longhand name="outline-style" need_clone="True">
    pub use values::specified::BorderStyle as SpecifiedValue;
    pub fn get_initial_value() -> SpecifiedValue { SpecifiedValue::none }
    pub mod computed_value {
        pub use values::specified::BorderStyle as T;
    }
    pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
        match SpecifiedValue::parse(input) {
            Ok(SpecifiedValue::hidden) => Err(()),
            result => result
        }
    }
</%helpers:longhand>

<%helpers:longhand name="outline-width" products="servo">
    use app_units::Au;
    use cssparser::ToCss;
    use std::fmt;
    use values::AuExtensionMethods;

    impl ToCss for SpecifiedValue {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            self.0.to_css(dest)
        }
    }

    pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
        specified::parse_border_width(input).map(SpecifiedValue)
    }
    #[derive(Debug, Clone, PartialEq, HeapSizeOf)]
    pub struct SpecifiedValue(pub specified::Length);
    pub mod computed_value {
        use app_units::Au;
        pub type T = Au;
        pub fn zero() -> T { Au(0) }
    }
    pub use super::border_top_width::get_initial_value;
    impl ToComputedValue for SpecifiedValue {
        type ComputedValue = computed_value::T;

        #[inline]
        fn to_computed_value<Cx: TContext>(&self, context: &Cx) -> computed_value::T {
            self.0.to_computed_value(context)
        }
    }
</%helpers:longhand>

<%helpers:longhand name="outline-width" products="gecko">
    use animation::Interpolate;
    use app_units::Au;
    use cssparser::ToCss;
    use std::fmt;
    use values::AuExtensionMethods;
    use values::specified::Length;

    impl ToCss for SpecifiedValue {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match *self {
                SpecifiedValue::Length(length) => length.to_css(dest),
                SpecifiedValue::Thin => dest.write_str("thin"),
                SpecifiedValue::Medium => dest.write_str("medium"),
                SpecifiedValue::Thick => dest.write_str("thick"),
            }
        }
    }

    impl ToCss for computed_value::T {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match *self {
                computed_value::T::Length(au) => au.to_css(dest),
                computed_value::T::Thin => dest.write_str("thin"),
                computed_value::T::Medium => dest.write_str("medium"),
                computed_value::T::Thick => dest.write_str("thick"),
            }
        }
    }

    pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
        input.try(Length::parse_non_negative).map(SpecifiedValue::Length).or_else(|()| {
            match_ignore_ascii_case! { try!(input.expect_ident()),
                "thin" => Ok(SpecifiedValue::Thin),
                "medium" => Ok(SpecifiedValue::Medium),
                "thick" => Ok(SpecifiedValue::Thick),
                _ => Err(())
            }
        })
    }
    #[derive(Debug, Clone, PartialEq, HeapSizeOf)]
    pub enum SpecifiedValue {
        Length(specified::Length),
        Thin,
        Medium,
        Thick,
    }
    pub mod computed_value {
        use app_units::Au;
        #[derive(Debug, Clone, Copy, PartialEq, HeapSizeOf)]
        pub enum T {
            Length(Au),
            Thin,
            Medium,
            Thick,
        }
        pub fn zero() -> T { T::Length(Au(0)) }
    }
    pub fn get_initial_value() -> computed_value::T { computed_value::T::Medium }
    impl ToComputedValue for SpecifiedValue {
        type ComputedValue = computed_value::T;

        #[inline]
        fn to_computed_value<Cx: TContext>(&self, context: &Cx) -> computed_value::T {
            match *self {
                SpecifiedValue::Length(au) => computed_value::T::Length(au.to_computed_value(context)),
                SpecifiedValue::Thin => computed_value::T::Thin,
                SpecifiedValue::Medium => computed_value::T::Medium,
                SpecifiedValue::Thick => computed_value::T::Thick,
            }
        }
    }

    impl Interpolate for computed_value::T {
        #[inline]
        fn interpolate(&self, other: &computed_value::T, time: f64)
                       -> Option<computed_value::T> {
            match (*self, *other) {
                (computed_value::T::Length(ref a),
                 computed_value::T::Length(ref b)) => {
                    a.interpolate(b, time).and_then(|value| {
                        Some(computed_value::T::Length(value))
                    })
                },
                _ => None,
            }
        }
    }
</%helpers:longhand>

% for corner in ["topleft", "topright", "bottomright", "bottomleft"]:
    ${helpers.predefined_type("-moz-outline-radius-" + corner, "BorderRadiusSize",
                              "computed::BorderRadiusSize::zero()",
                              "parse", products="gecko")}
% endfor

${helpers.predefined_type("outline-offset", "Length", "Au(0)")}
