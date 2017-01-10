/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />

// Box-shadow, etc.
<% data.new_style_struct("Effects", inherited=False) %>

${helpers.predefined_type("opacity",
                          "Opacity",
                          "1.0",
                          animatable=True,
                          spec="https://drafts.csswg.org/css-color/#opacity")}

<%helpers:vector_longhand name="box-shadow" allow_empty="True"
                          animatable="True" extra_prefixes="webkit"
                          spec="https://drafts.csswg.org/css-backgrounds/#box-shadow">
    use cssparser;
    use std::fmt;
    use style_traits::ToCss;
    use values::HasViewportPercentage;

    pub type SpecifiedValue = specified::Shadow;

    impl ToCss for SpecifiedValue {
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

            if let Some(ref color) = self.color {
                try!(dest.write_str(" "));
                try!(color.to_css(dest));
            }
            Ok(())
        }
    }

    pub mod computed_value {
        use app_units::Au;
        use std::fmt;
        use values::computed;
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

// FIXME: This prop should be animatable
<%helpers:longhand name="clip" products="servo" animatable="False"
                   spec="https://drafts.fxtf.org/css-masking/#clip-property">
    use std::fmt;
    use style_traits::ToCss;
    use values::HasViewportPercentage;

    // NB: `top` and `left` are 0 if `auto` per CSS 2.1 11.1.2.

    pub mod computed_value {
        use app_units::Au;
        use properties::animated_properties::Interpolate;

        #[derive(Clone, PartialEq, Eq, Copy, Debug)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub struct ClipRect {
            pub top: Au,
            pub right: Option<Au>,
            pub bottom: Option<Au>,
            pub left: Au,
        }

        #[derive(Debug, Clone, PartialEq)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub struct T(pub Option<ClipRect>);


        /// https://drafts.csswg.org/css-transitions/#animtype-rect
        impl Interpolate for ClipRect {
            #[inline]
            fn interpolate(&self, other: &Self, time: f64) -> Result<Self, ()> {
                Ok(ClipRect {
                    top: try!(self.top.interpolate(&other.top, time)),
                    right: try!(self.right.interpolate(&other.right, time)),
                    bottom: try!(self.bottom.interpolate(&other.bottom, time)),
                    left: try!(self.left.interpolate(&other.left, time)),
                })
            }
        }
    }

    impl ToCss for computed_value::T {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match self.0 {
                None => dest.write_str("auto"),
                Some(rect) => {
                    try!(dest.write_str("rect("));
                    try!(rect.top.to_css(dest));
                    try!(dest.write_str(", "));
                    if let Some(right) = rect.right {
                        try!(right.to_css(dest));
                        try!(dest.write_str(", "));
                    } else {
                        try!(dest.write_str("auto, "));
                    }

                    if let Some(bottom) = rect.bottom {
                        try!(bottom.to_css(dest));
                        try!(dest.write_str(", "));
                    } else {
                        try!(dest.write_str("auto, "));
                    }

                    try!(rect.left.to_css(dest));
                    try!(dest.write_str(")"));
                    Ok(())
                }
            }
        }
    }

    impl HasViewportPercentage for SpecifiedClipRect {
        fn has_viewport_percentage(&self) -> bool {
            self.top.has_viewport_percentage() ||
            self.right.map_or(false, |x| x.has_viewport_percentage()) ||
            self.bottom.map_or(false, |x| x.has_viewport_percentage()) ||
            self.left.has_viewport_percentage()
        }
    }

    #[derive(Clone, Debug, PartialEq, Copy)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub struct SpecifiedClipRect {
        pub top: specified::Length,
        pub right: Option<specified::Length>,
        pub bottom: Option<specified::Length>,
        pub left: specified::Length,
    }

    impl HasViewportPercentage for SpecifiedValue {
        fn has_viewport_percentage(&self) -> bool {
            let &SpecifiedValue(clip) = self;
            clip.map_or(false, |x| x.has_viewport_percentage())
        }
    }

    #[derive(Clone, Debug, PartialEq, Copy)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub struct SpecifiedValue(Option<SpecifiedClipRect>);

    impl ToCss for SpecifiedClipRect {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            try!(dest.write_str("rect("));

            try!(self.top.to_css(dest));
            try!(dest.write_str(", "));

            if let Some(right) = self.right {
                try!(right.to_css(dest));
                try!(dest.write_str(", "));
            } else {
                try!(dest.write_str("auto, "));
            }

            if let Some(bottom) = self.bottom {
                try!(bottom.to_css(dest));
                try!(dest.write_str(", "));
            } else {
                try!(dest.write_str("auto, "));
            }

            try!(self.left.to_css(dest));

            try!(dest.write_str(")"));
            Ok(())
        }
    }

    impl ToCss for SpecifiedValue {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            if let Some(ref rect) = self.0 {
                rect.to_css(dest)
            } else {
                dest.write_str("auto")
            }
        }
    }

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        computed_value::T(None)
    }

    impl ToComputedValue for SpecifiedValue {
        type ComputedValue = computed_value::T;

        #[inline]
        fn to_computed_value(&self, context: &Context) -> computed_value::T {
            computed_value::T(self.0.map(|value| computed_value::ClipRect {
                top: value.top.to_computed_value(context),
                right: value.right.map(|right| right.to_computed_value(context)),
                bottom: value.bottom.map(|bottom| bottom.to_computed_value(context)),
                left: value.left.to_computed_value(context),
            }))
        }

        #[inline]
        fn from_computed_value(computed: &computed_value::T) -> Self {
            SpecifiedValue(computed.0.map(|value| SpecifiedClipRect {
                top: ToComputedValue::from_computed_value(&value.top),
                right: value.right.map(|right| ToComputedValue::from_computed_value(&right)),
                bottom: value.bottom.map(|bottom| ToComputedValue::from_computed_value(&bottom)),
                left: ToComputedValue::from_computed_value(&value.left),
            }))
        }
    }

    pub fn parse(context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
        use app_units::Au;
        use std::ascii::AsciiExt;
        use values::specified::Length;

        fn parse_argument(context: &ParserContext, input: &mut Parser) -> Result<Option<Length>, ()> {
            if input.try(|input| input.expect_ident_matching("auto")).is_ok() {
                Ok(None)
            } else {
                Length::parse(context, input).map(Some)
            }
        }

        if input.try(|input| input.expect_ident_matching("auto")).is_ok() {
            return Ok(SpecifiedValue(None))
        }
        if !try!(input.expect_function()).eq_ignore_ascii_case("rect") {
            return Err(())
        }

        input.parse_nested_block(|input| {
            let top = try!(parse_argument(context, input));
            let right;
            let bottom;
            let left;

            if input.try(|input| input.expect_comma()).is_ok() {
                right = try!(parse_argument(context, input));
                try!(input.expect_comma());
                bottom = try!(parse_argument(context, input));
                try!(input.expect_comma());
                left = try!(parse_argument(context, input));
            } else {
                right = try!(parse_argument(context, input));
                bottom = try!(parse_argument(context, input));
                left = try!(parse_argument(context, input));
            }
            Ok(SpecifiedValue(Some(SpecifiedClipRect {
                top: top.unwrap_or(Length::Absolute(Au(0))),
                right: right,
                bottom: bottom,
                left: left.unwrap_or(Length::Absolute(Au(0))),
            })))
        })
    }
</%helpers:longhand>

// FIXME: This prop should be animatable
<%helpers:longhand name="filter" animatable="False" extra_prefixes="webkit"
                   spec="https://drafts.fxtf.org/filters/#propdef-filter">
    //pub use self::computed_value::T as SpecifiedValue;
    use cssparser;
    use std::fmt;
    use style_traits::{self, ToCss};
    use values::{CSSFloat, HasViewportPercentage};
    use values::specified::{Angle, CSSColor, Length, Shadow};

    impl HasViewportPercentage for SpecifiedValue {
        fn has_viewport_percentage(&self) -> bool {
            let &SpecifiedValue(ref vec) = self;
            vec.iter().any(|ref x| x.has_viewport_percentage())
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub struct SpecifiedValue(pub Vec<SpecifiedFilter>);

    impl HasViewportPercentage for SpecifiedFilter {
        fn has_viewport_percentage(&self) -> bool {
            match *self {
                SpecifiedFilter::Blur(length) => length.has_viewport_percentage(),
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
        % endif
    }

    pub mod computed_value {
        use app_units::Au;
        use values::CSSFloat;
        use values::computed::{CSSColor, Shadow};
        use values::specified::{Angle};

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
                computed_value::Filter::Blur(value) => {
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
                    try!(dest.write_str(", "));
                    try!(shadow.offset_y.to_css(dest));
                    try!(dest.write_str(", "));
                    try!(shadow.blur_radius.to_css(dest));
                    try!(dest.write_str(", "));
                    try!(shadow.color.to_css(dest));
                    try!(dest.write_str(")"));
                }
                % endif
            }
            Ok(())
        }
    }

    impl ToCss for SpecifiedFilter {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match *self {
                SpecifiedFilter::Blur(value) => {
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
                    try!(dest.write_str(", "));
                    try!(shadow.offset_y.to_css(dest));
                    try!(dest.write_str(", "));
                    try!(shadow.blur_radius.to_css(dest));
                    if let Some(ref color) = shadow.color {
                        try!(dest.write_str(", "));
                        try!(color.to_css(dest));
                    }
                    try!(dest.write_str(")"));
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
            if let Ok(function_name) = input.try(|input| input.expect_function()) {
                filters.push(try!(input.parse_nested_block(|input| {
                    match_ignore_ascii_case! { function_name,
                        "blur" => specified::Length::parse_non_negative(input).map(SpecifiedFilter::Blur),
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
            Ok(Token::Number(value)) => Ok(value.value),
            Ok(Token::Percentage(value)) => Ok(value.unit_value),
            _ => Err(())
        }
    }

    impl ToComputedValue for SpecifiedValue {
        type ComputedValue = computed_value::T;

        fn to_computed_value(&self, context: &Context) -> computed_value::T {
            computed_value::T{ filters: self.0.iter().map(|value| {
                match *value {
                    SpecifiedFilter::Blur(factor) =>
                        computed_value::Filter::Blur(factor.to_computed_value(context)),
                    SpecifiedFilter::Brightness(factor) => computed_value::Filter::Brightness(factor),
                    SpecifiedFilter::Contrast(factor) => computed_value::Filter::Contrast(factor),
                    SpecifiedFilter::Grayscale(factor) => computed_value::Filter::Grayscale(factor),
                    SpecifiedFilter::HueRotate(factor) => computed_value::Filter::HueRotate(factor),
                    SpecifiedFilter::Invert(factor) => computed_value::Filter::Invert(factor),
                    SpecifiedFilter::Opacity(factor) => computed_value::Filter::Opacity(factor),
                    SpecifiedFilter::Saturate(factor) => computed_value::Filter::Saturate(factor),
                    SpecifiedFilter::Sepia(factor) => computed_value::Filter::Sepia(factor),
                    % if product == "gecko":
                    SpecifiedFilter::DropShadow(ref shadow) => {
                        computed_value::Filter::DropShadow(shadow.to_computed_value(context))
                    },
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
                    computed_value::Filter::HueRotate(factor) => SpecifiedFilter::HueRotate(factor),
                    computed_value::Filter::Invert(factor) => SpecifiedFilter::Invert(factor),
                    computed_value::Filter::Opacity(factor) => SpecifiedFilter::Opacity(factor),
                    computed_value::Filter::Saturate(factor) => SpecifiedFilter::Saturate(factor),
                    computed_value::Filter::Sepia(factor) => SpecifiedFilter::Sepia(factor),
                    % if product == "gecko":
                    computed_value::Filter::DropShadow(shadow) => {
                        SpecifiedFilter::DropShadow(
                            ToComputedValue::from_computed_value(&shadow),
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
    pub depth: Option<specified::Length>
}

pub fn parse_origin(context: &ParserContext, input: &mut Parser) -> Result<OriginParseResult,()> {
    use values::specified::{LengthOrPercentage, Percentage};
    let (mut horizontal, mut vertical, mut depth) = (None, None, None);
    loop {
        if let Err(_) = input.try(|input| {
            let token = try!(input.expect_ident());
            match_ignore_ascii_case! {
                token,
                "left" => {
                    if horizontal.is_none() {
                        horizontal = Some(LengthOrPercentage::Percentage(Percentage(0.0)))
                    } else {
                        return Err(())
                    }
                },
                "center" => {
                    if horizontal.is_none() {
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
            match LengthOrPercentage::parse(context, input) {
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
                         animatable=False,
                         spec="https://drafts.fxtf.org/compositing/#propdef-mix-blend-mode")}
