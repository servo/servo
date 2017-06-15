/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />

// Box-shadow, etc.
<% data.new_style_struct("Effects", inherited=False) %>

${helpers.predefined_type("opacity",
                          "Opacity",
                          "1.0",
                          animation_value_type="ComputedValue",
                          flags="CREATES_STACKING_CONTEXT",
                          spec="https://drafts.csswg.org/css-color/#opacity")}

<%helpers:vector_longhand name="box-shadow" allow_empty="True"
                          animation_value_type="IntermediateShadowList"
                          extra_prefixes="webkit"
                          ignored_when_colors_disabled="True"
                          spec="https://drafts.csswg.org/css-backgrounds/#box-shadow">
    pub type SpecifiedValue = specified::Shadow;

    pub mod computed_value {
        use values::computed::Shadow;

        pub type T = Shadow;
    }

    pub fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>)
                         -> Result<specified::Shadow, ParseError<'i>> {
        specified::Shadow::parse(context, input, false)
    }
</%helpers:vector_longhand>

${helpers.predefined_type("clip",
                          "ClipRectOrAuto",
                          "computed::ClipRectOrAuto::auto()",
                          animation_value_type="ComputedValue",
                          boxed="True",
                          allow_quirks=True,
                          spec="https://drafts.fxtf.org/css-masking/#clip-property")}

// FIXME: This prop should be animatable
<%helpers:longhand name="filter" animation_value_type="none" extra_prefixes="webkit"
                   flags="CREATES_STACKING_CONTEXT FIXPOS_CB"
                   spec="https://drafts.fxtf.org/filters/#propdef-filter">
    //pub use self::computed_value::T as SpecifiedValue;
    use std::fmt;
    use style_traits::{HasViewportPercentage, ToCss};
    use values::CSSFloat;
    use values::specified::{Angle, Length};
    #[cfg(feature = "gecko")]
    use values::specified::Shadow;
    #[cfg(feature = "gecko")]
    use values::specified::url::SpecifiedUrl;

    #[derive(Clone, Debug, HasViewportPercentage, PartialEq)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub struct SpecifiedValue(pub Vec<SpecifiedFilter>);

    impl HasViewportPercentage for SpecifiedFilter {
        fn has_viewport_percentage(&self) -> bool {
            match *self {
                SpecifiedFilter::Blur(ref length) => length.has_viewport_percentage(),
                _ => false
            }
        }
    }

    // TODO(pcwalton): `drop-shadow`
    #[derive(Clone, PartialEq, Debug)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub enum SpecifiedFilter {
        Blur(Length),
        Brightness(CSSFloat),
        Contrast(CSSFloat),
        Grayscale(CSSFloat),
        HueRotate(Angle),
        Invert(CSSFloat),
        Opacity(CSSFloat),
        Saturate(CSSFloat),
        Sepia(CSSFloat),
        % if product == "gecko":
        DropShadow(Shadow),
        Url(SpecifiedUrl),
        % endif
    }

    pub mod computed_value {
        use app_units::Au;
        use values::CSSFloat;
        #[cfg(feature = "gecko")]
        use values::computed::Shadow;
        use values::computed::Angle;
        #[cfg(feature = "gecko")]
        use values::specified::url::SpecifiedUrl;

        #[derive(Clone, PartialEq, Debug)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf, Deserialize, Serialize))]
        pub enum Filter {
            Blur(Au),
            Brightness(CSSFloat),
            Contrast(CSSFloat),
            Grayscale(CSSFloat),
            HueRotate(Angle),
            Invert(CSSFloat),
            Opacity(CSSFloat),
            Saturate(CSSFloat),
            Sepia(CSSFloat),
            % if product == "gecko":
            DropShadow(Shadow),
            Url(SpecifiedUrl),
            % endif
        }

        #[derive(Clone, PartialEq, Debug)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf, Deserialize, Serialize))]
        pub struct T { pub filters: Vec<Filter> }

        impl T {
            /// Creates a new filter pipeline.
            #[inline]
            pub fn new(filters: Vec<Filter>) -> T {
                T
                {
                   filters: filters,
                }
            }

            /// Adds a new filter to the filter pipeline.
            #[inline]
            pub fn push(&mut self, filter: Filter) {
                self.filters.push(filter)
            }

            /// Returns true if this filter pipeline is empty and false otherwise.
            #[inline]
            pub fn is_empty(&self) -> bool {
                self.filters.is_empty()
            }

            /// Returns the resulting opacity of this filter pipeline.
            #[inline]
            pub fn opacity(&self) -> CSSFloat {
                let mut opacity = 1.0;

                for filter in &self.filters {
                    if let Filter::Opacity(ref opacity_value) = *filter {
                        opacity *= *opacity_value
                    }
                }
                opacity
            }
        }
    }

    impl ToCss for computed_value::T {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            let mut iter = self.filters.iter();
            if let Some(filter) = iter.next() {
                try!(filter.to_css(dest));
            } else {
                try!(dest.write_str("none"));
                return Ok(())
            }
            for filter in iter {
                try!(dest.write_str(" "));
                try!(filter.to_css(dest));
            }
            Ok(())
        }
    }

    impl ToCss for SpecifiedValue {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            let mut iter = self.0.iter();
            if let Some(filter) = iter.next() {
                try!(filter.to_css(dest));
            } else {
                try!(dest.write_str("none"));
                return Ok(())
            }
            for filter in iter {
                try!(dest.write_str(" "));
                try!(filter.to_css(dest));
            }
            Ok(())
        }
    }

    impl ToCss for computed_value::Filter {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match *self {
                computed_value::Filter::Blur(ref value) => {
                    try!(dest.write_str("blur("));
                    try!(value.to_css(dest));
                    try!(dest.write_str(")"));
                }
                computed_value::Filter::Brightness(value) => try!(write!(dest, "brightness({})", value)),
                computed_value::Filter::Contrast(value) => try!(write!(dest, "contrast({})", value)),
                computed_value::Filter::Grayscale(value) => try!(write!(dest, "grayscale({})", value)),
                computed_value::Filter::HueRotate(value) => {
                    try!(dest.write_str("hue-rotate("));
                    try!(value.to_css(dest));
                    try!(dest.write_str(")"));
                }
                computed_value::Filter::Invert(value) => try!(write!(dest, "invert({})", value)),
                computed_value::Filter::Opacity(value) => try!(write!(dest, "opacity({})", value)),
                computed_value::Filter::Saturate(value) => try!(write!(dest, "saturate({})", value)),
                computed_value::Filter::Sepia(value) => try!(write!(dest, "sepia({})", value)),
                % if product == "gecko":
                computed_value::Filter::DropShadow(shadow) => {
                    dest.write_str("drop-shadow(")?;
                    shadow.to_css(dest)?;
                    dest.write_str(")")?;
                }
                computed_value::Filter::Url(ref url) => {
                    url.to_css(dest)?;
                }
                % endif
            }
            Ok(())
        }
    }

    impl ToCss for SpecifiedFilter {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match *self {
                SpecifiedFilter::Blur(ref value) => {
                    try!(dest.write_str("blur("));
                    try!(value.to_css(dest));
                    try!(dest.write_str(")"));
                }
                SpecifiedFilter::Brightness(value) => try!(write!(dest, "brightness({})", value)),
                SpecifiedFilter::Contrast(value) => try!(write!(dest, "contrast({})", value)),
                SpecifiedFilter::Grayscale(value) => try!(write!(dest, "grayscale({})", value)),
                SpecifiedFilter::HueRotate(value) => {
                    try!(dest.write_str("hue-rotate("));
                    try!(value.to_css(dest));
                    try!(dest.write_str(")"));
                }
                SpecifiedFilter::Invert(value) => try!(write!(dest, "invert({})", value)),
                SpecifiedFilter::Opacity(value) => try!(write!(dest, "opacity({})", value)),
                SpecifiedFilter::Saturate(value) => try!(write!(dest, "saturate({})", value)),
                SpecifiedFilter::Sepia(value) => try!(write!(dest, "sepia({})", value)),
                % if product == "gecko":
                SpecifiedFilter::DropShadow(ref shadow) => {
                    dest.write_str("drop-shadow(")?;
                    shadow.to_css(dest)?;
                    dest.write_str(")")?;
                }
                SpecifiedFilter::Url(ref url) => {
                    url.to_css(dest)?;
                }
                % endif
            }
            Ok(())
        }
    }

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        computed_value::T::new(Vec::new())
    }

    pub fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>)
                         -> Result<SpecifiedValue, ParseError<'i>> {
        let mut filters = Vec::new();
        if input.try(|input| input.expect_ident_matching("none")).is_ok() {
            return Ok(SpecifiedValue(filters))
        }
        loop {
            % if product == "gecko":
                if let Ok(url) = input.try(|i| SpecifiedUrl::parse(context, i)) {
                    filters.push(SpecifiedFilter::Url(url));
                } else
            % endif
            if let Ok(function_name) = input.try(|input| input.expect_function()) {
                filters.push(try!(input.parse_nested_block(|input| {
                    match_ignore_ascii_case! { &function_name,
                        "blur" => specified::Length::parse_non_negative(context, input).map(SpecifiedFilter::Blur),
                        "brightness" => parse_factor(input).map(SpecifiedFilter::Brightness),
                        "contrast" => parse_factor(input).map(SpecifiedFilter::Contrast),
                        "grayscale" => parse_factor(input).map(SpecifiedFilter::Grayscale),
                        "hue-rotate" => Angle::parse(context, input).map(SpecifiedFilter::HueRotate),
                        "invert" => parse_factor(input).map(SpecifiedFilter::Invert),
                        "opacity" => parse_factor(input).map(SpecifiedFilter::Opacity),
                        "saturate" => parse_factor(input).map(SpecifiedFilter::Saturate),
                        "sepia" => parse_factor(input).map(SpecifiedFilter::Sepia),
                        % if product == "gecko":
                        "drop-shadow" => specified::Shadow::parse(context, input, true)
                                             .map(SpecifiedFilter::DropShadow),
                        % endif
                        _ => Err(StyleParseError::UnexpectedFunction(function_name.clone()).into())
                    }
                })));
            } else if filters.is_empty() {
                return Err(StyleParseError::UnspecifiedError.into())
            } else {
                return Ok(SpecifiedValue(filters))
            }
        }
    }

    fn parse_factor<'i, 't>(input: &mut Parser<'i, 't>) -> Result<::values::CSSFloat, ParseError<'i>> {
        use cssparser::Token;
        match input.next() {
            Ok(Token::Number { value, .. }) if value.is_sign_positive() => Ok(value),
            Ok(Token::Percentage { unit_value, .. }) if unit_value.is_sign_positive() => Ok(unit_value),
            Ok(t) => Err(BasicParseError::UnexpectedToken(t).into()),
            Err(e) => Err(e.into())
        }
    }

    impl ToComputedValue for SpecifiedValue {
        type ComputedValue = computed_value::T;

        fn to_computed_value(&self, context: &Context) -> computed_value::T {
            computed_value::T{ filters: self.0.iter().map(|value| {
                match *value {
                    SpecifiedFilter::Blur(ref factor) =>
                        computed_value::Filter::Blur(factor.to_computed_value(context)),
                    SpecifiedFilter::Brightness(factor) => computed_value::Filter::Brightness(factor),
                    SpecifiedFilter::Contrast(factor) => computed_value::Filter::Contrast(factor),
                    SpecifiedFilter::Grayscale(factor) => computed_value::Filter::Grayscale(factor),
                    SpecifiedFilter::HueRotate(ref factor) => {
                        computed_value::Filter::HueRotate(factor.to_computed_value(context))
                    },
                    SpecifiedFilter::Invert(factor) => computed_value::Filter::Invert(factor),
                    SpecifiedFilter::Opacity(factor) => computed_value::Filter::Opacity(factor),
                    SpecifiedFilter::Saturate(factor) => computed_value::Filter::Saturate(factor),
                    SpecifiedFilter::Sepia(factor) => computed_value::Filter::Sepia(factor),
                    % if product == "gecko":
                    SpecifiedFilter::DropShadow(ref shadow) => {
                        computed_value::Filter::DropShadow(shadow.to_computed_value(context))
                    },
                    SpecifiedFilter::Url(ref url) => {
                        computed_value::Filter::Url(url.to_computed_value(context))
                    }
                    % endif
                }
            }).collect() }
        }

        fn from_computed_value(computed: &computed_value::T) -> Self {
            SpecifiedValue(computed.filters.iter().map(|value| {
                match *value {
                    computed_value::Filter::Blur(factor) =>
                        SpecifiedFilter::Blur(ToComputedValue::from_computed_value(&factor)),
                    computed_value::Filter::Brightness(factor) => SpecifiedFilter::Brightness(factor),
                    computed_value::Filter::Contrast(factor) => SpecifiedFilter::Contrast(factor),
                    computed_value::Filter::Grayscale(factor) => SpecifiedFilter::Grayscale(factor),
                    computed_value::Filter::HueRotate(ref factor) => {
                        SpecifiedFilter::HueRotate(ToComputedValue::from_computed_value(factor))
                    },
                    computed_value::Filter::Invert(factor) => SpecifiedFilter::Invert(factor),
                    computed_value::Filter::Opacity(factor) => SpecifiedFilter::Opacity(factor),
                    computed_value::Filter::Saturate(factor) => SpecifiedFilter::Saturate(factor),
                    computed_value::Filter::Sepia(factor) => SpecifiedFilter::Sepia(factor),
                    % if product == "gecko":
                    computed_value::Filter::DropShadow(ref shadow) => {
                        SpecifiedFilter::DropShadow(
                            ToComputedValue::from_computed_value(shadow),
                        )
                    }
                    computed_value::Filter::Url(ref url) => {
                        SpecifiedFilter::Url(
                            ToComputedValue::from_computed_value(url),
                        )
                    }
                    % endif
                }
            }).collect())
        }
    }
</%helpers:longhand>

${helpers.single_keyword("mix-blend-mode",
                         """normal multiply screen overlay darken lighten color-dodge
                            color-burn hard-light soft-light difference exclusion hue
                            saturation color luminosity""", gecko_constant_prefix="NS_STYLE_BLEND",
                         animation_value_type="discrete",
                         flags="CREATES_STACKING_CONTEXT",
                         spec="https://drafts.fxtf.org/compositing/#propdef-mix-blend-mode")}
