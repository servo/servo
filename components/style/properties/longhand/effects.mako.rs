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
                          animation_value_type="IntermediateBoxShadowList"
                          extra_prefixes="webkit"
                          ignored_when_colors_disabled="True"
                          spec="https://drafts.csswg.org/css-backgrounds/#box-shadow">
    use std::fmt;
    use style_traits::ToCss;

    pub type SpecifiedValue = specified::Shadow;

    impl ToCss for SpecifiedValue {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            if self.inset {
                try!(dest.write_str("inset "));
            }
            try!(self.offset_x.to_css(dest));
            try!(dest.write_str(" "));
            try!(self.offset_y.to_css(dest));
            try!(dest.write_str(" "));
            try!(self.blur_radius.to_css(dest));
            try!(dest.write_str(" "));
            try!(self.spread_radius.to_css(dest));

            if let Some(ref color) = self.color {
                try!(dest.write_str(" "));
                try!(color.to_css(dest));
            }
            Ok(())
        }
    }

    pub mod computed_value {
        use values::computed::Shadow;

        pub type T = Shadow;
    }

    impl ToCss for computed_value::T {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            if self.inset {
                try!(dest.write_str("inset "));
            }
            try!(self.blur_radius.to_css(dest));
            try!(dest.write_str(" "));
            try!(self.spread_radius.to_css(dest));
            try!(dest.write_str(" "));
            try!(self.offset_x.to_css(dest));
            try!(dest.write_str(" "));
            try!(self.offset_y.to_css(dest));
            try!(dest.write_str(" "));
            try!(self.color.to_css(dest));
            Ok(())
        }
    }

    pub fn parse(context: &ParserContext, input: &mut Parser) -> Result<specified::Shadow, ()> {
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
                    try!(dest.write_str("drop-shadow("));
                    try!(shadow.offset_x.to_css(dest));
                    try!(dest.write_str(" "));
                    try!(shadow.offset_y.to_css(dest));
                    try!(dest.write_str(" "));
                    try!(shadow.blur_radius.to_css(dest));
                    try!(dest.write_str(" "));
                    try!(shadow.color.to_css(dest));
                    try!(dest.write_str(")"));
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
                    try!(dest.write_str("drop-shadow("));
                    try!(shadow.offset_x.to_css(dest));
                    try!(dest.write_str(" "));
                    try!(shadow.offset_y.to_css(dest));
                    try!(dest.write_str(" "));
                    try!(shadow.blur_radius.to_css(dest));
                    if let Some(ref color) = shadow.color {
                        try!(dest.write_str(" "));
                        try!(color.to_css(dest));
                    }
                    try!(dest.write_str(")"));
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

    pub fn parse(context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
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
                        _ => Err(())
                    }
                })));
            } else if filters.is_empty() {
                return Err(())
            } else {
                return Ok(SpecifiedValue(filters))
            }
        }
    }

    fn parse_factor(input: &mut Parser) -> Result<::values::CSSFloat, ()> {
        use cssparser::Token;
        match input.next() {
            Ok(Token::Number(value)) if value.value.is_sign_positive() => Ok(value.value),
            Ok(Token::Percentage(value)) if value.unit_value.is_sign_positive() => Ok(value.unit_value),
            _ => Err(())
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

pub struct OriginParseResult {
    pub horizontal: Option<specified::LengthOrPercentage>,
    pub vertical: Option<specified::LengthOrPercentage>,
    pub depth: Option<specified::NoCalcLength>
}

pub fn parse_origin(context: &ParserContext, input: &mut Parser) -> Result<OriginParseResult,()> {
    use values::specified::{LengthOrPercentage, Percentage};
    let (mut horizontal, mut vertical, mut depth, mut horizontal_is_center) = (None, None, None, false);
    loop {
        if let Err(_) = input.try(|input| {
            let token = try!(input.expect_ident());
            match_ignore_ascii_case! {
                &token,
                "left" => {
                    if horizontal.is_none() {
                        horizontal = Some(LengthOrPercentage::Percentage(Percentage(0.0)))
                    } else if horizontal_is_center && vertical.is_none() {
                        vertical = Some(LengthOrPercentage::Percentage(Percentage(0.5)));
                        horizontal = Some(LengthOrPercentage::Percentage(Percentage(0.0)));
                    } else {
                        return Err(())
                    }
                },
                "center" => {
                    if horizontal.is_none() {
                        horizontal_is_center = true;
                        horizontal = Some(LengthOrPercentage::Percentage(Percentage(0.5)))
                    } else if vertical.is_none() {
                        vertical = Some(LengthOrPercentage::Percentage(Percentage(0.5)))
                    } else {
                        return Err(())
                    }
                },
                "right" => {
                    if horizontal.is_none() {
                        horizontal = Some(LengthOrPercentage::Percentage(Percentage(1.0)))
                    } else if horizontal_is_center && vertical.is_none() {
                        vertical = Some(LengthOrPercentage::Percentage(Percentage(0.5)));
                        horizontal = Some(LengthOrPercentage::Percentage(Percentage(1.0)));
                    } else {
                        return Err(())
                    }
                },
                "top" => {
                    if vertical.is_none() {
                        vertical = Some(LengthOrPercentage::Percentage(Percentage(0.0)))
                    } else {
                        return Err(())
                    }
                },
                "bottom" => {
                    if vertical.is_none() {
                        vertical = Some(LengthOrPercentage::Percentage(Percentage(1.0)))
                    } else {
                        return Err(())
                    }
                },
                _ => return Err(())
            }
            Ok(())
        }) {
            match input.try(|input| LengthOrPercentage::parse(context, input)) {
                Ok(value) => {
                    if horizontal.is_none() {
                        horizontal = Some(value);
                    } else if vertical.is_none() {
                        vertical = Some(value);
                    } else if let LengthOrPercentage::Length(length) = value {
                        depth = Some(length);
                    } else {
                        break;
                    }
                }
                _ => break,
            }
        }
    }

    if horizontal.is_some() || vertical.is_some() {
        Ok(OriginParseResult {
            horizontal: horizontal,
            vertical: vertical,
            depth: depth,
        })
    } else {
        Err(())
    }
}

${helpers.single_keyword("mix-blend-mode",
                         """normal multiply screen overlay darken lighten color-dodge
                            color-burn hard-light soft-light difference exclusion hue
                            saturation color luminosity""", gecko_constant_prefix="NS_STYLE_BLEND",
                         animation_value_type="discrete",
                         flags="CREATES_STACKING_CONTEXT",
                         spec="https://drafts.fxtf.org/compositing/#propdef-mix-blend-mode")}
