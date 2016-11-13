/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />

// Box-shadow, etc.
<% data.new_style_struct("Effects", inherited=False) %>

${helpers.predefined_type("opacity",
                          "Opacity",
                          "1.0",
                          animatable=True)}

<%helpers:vector_longhand name="box-shadow" allow_empty="True" animatable="True">
    use cssparser;
    use std::fmt;
    use style_traits::ToCss;
    use values::HasViewportPercentage;

    #[derive(Debug, Clone, PartialEq)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub struct SpecifiedValue {
        pub offset_x: specified::Length,
        pub offset_y: specified::Length,
        pub blur_radius: specified::Length,
        pub spread_radius: specified::Length,
        pub color: Option<specified::CSSColor>,
        pub inset: bool,
    }

    impl HasViewportPercentage for SpecifiedValue {
        fn has_viewport_percentage(&self) -> bool {
            self.offset_x.has_viewport_percentage() ||
            self.offset_y.has_viewport_percentage() ||
            self.blur_radius.has_viewport_percentage() ||
            self.spread_radius.has_viewport_percentage()
        }
    }

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

        #[derive(Clone, PartialEq, Copy, Debug)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub struct T {
            pub offset_x: Au,
            pub offset_y: Au,
            pub blur_radius: Au,
            pub spread_radius: Au,
            pub color: computed::CSSColor,
            pub inset: bool,
        }
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

    impl ToComputedValue for SpecifiedValue {
        type ComputedValue = computed_value::T;

        #[inline]
        fn to_computed_value(&self, context: &Context) -> computed_value::T {
            computed_value::T {
                offset_x: self.offset_x.to_computed_value(context),
                offset_y: self.offset_y.to_computed_value(context),
                blur_radius: self.blur_radius.to_computed_value(context),
                spread_radius: self.spread_radius.to_computed_value(context),
                color: self.color
                            .as_ref()
                            .map(|color| color.parsed)
                            .unwrap_or(cssparser::Color::CurrentColor),
                inset: self.inset,
            }
        }

        #[inline]
        fn from_computed_value(computed: &computed_value::T) -> Self {
            SpecifiedValue {
                offset_x: ToComputedValue::from_computed_value(&computed.offset_x),
                offset_y: ToComputedValue::from_computed_value(&computed.offset_y),
                blur_radius: ToComputedValue::from_computed_value(&computed.blur_radius),
                spread_radius: ToComputedValue::from_computed_value(&computed.spread_radius),
                color: Some(ToComputedValue::from_computed_value(&computed.color)),
                inset: computed.inset,
            }
        }
    }

    pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
        use app_units::Au;
        let mut lengths = [specified::Length::Absolute(Au(0)); 4];
        let mut lengths_parsed = false;
        let mut color = None;
        let mut inset = false;

        loop {
            if !inset {
                if input.try(|input| input.expect_ident_matching("inset")).is_ok() {
                    inset = true;
                    continue
                }
            }
            if !lengths_parsed {
                if let Ok(value) = input.try(specified::Length::parse) {
                    lengths[0] = value;
                    let mut length_parsed_count = 1;
                    while length_parsed_count < 4 {
                        if let Ok(value) = input.try(specified::Length::parse) {
                            lengths[length_parsed_count] = value
                        } else {
                            break
                        }
                        length_parsed_count += 1;
                    }

                    // The first two lengths must be specified.
                    if length_parsed_count < 2 {
                        return Err(())
                    }

                    lengths_parsed = true;
                    continue
                }
            }
            if color.is_none() {
                if let Ok(value) = input.try(specified::CSSColor::parse) {
                    color = Some(value);
                    continue
                }
            }
            break
        }

        // Lengths must be specified.
        if !lengths_parsed {
            return Err(())
        }

        Ok(SpecifiedValue {
            offset_x: lengths[0],
            offset_y: lengths[1],
            blur_radius: lengths[2],
            spread_radius: lengths[3],
            color: color,
            inset: inset,
        })
    }
</%helpers:vector_longhand>

// FIXME: This prop should be animatable
<%helpers:longhand name="clip" products="servo" animatable="False">
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

    pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
        use app_units::Au;
        use std::ascii::AsciiExt;
        use values::specified::Length;

        fn parse_argument(input: &mut Parser) -> Result<Option<Length>, ()> {
            if input.try(|input| input.expect_ident_matching("auto")).is_ok() {
                Ok(None)
            } else {
                Length::parse(input).map(Some)
            }
        }

        if input.try(|input| input.expect_ident_matching("auto")).is_ok() {
            return Ok(SpecifiedValue(None))
        }
        if !try!(input.expect_function()).eq_ignore_ascii_case("rect") {
            return Err(())
        }

        input.parse_nested_block(|input| {
            let top = try!(parse_argument(input));
            let right;
            let bottom;
            let left;

            if input.try(|input| input.expect_comma()).is_ok() {
                right = try!(parse_argument(input));
                try!(input.expect_comma());
                bottom = try!(parse_argument(input));
                try!(input.expect_comma());
                left = try!(parse_argument(input));
            } else {
                right = try!(parse_argument(input));
                bottom = try!(parse_argument(input));
                left = try!(parse_argument(input));
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
<%helpers:longhand name="filter" animatable="False">
    //pub use self::computed_value::T as SpecifiedValue;
    use std::fmt;
    use style_traits::ToCss;
    use values::{CSSFloat, HasViewportPercentage};
    use values::specified::{Angle, Length};

    impl HasViewportPercentage for SpecifiedValue {
        fn has_viewport_percentage(&self) -> bool {
            let &SpecifiedValue(ref vec) = self;
            vec.iter().any(|ref x| x.has_viewport_percentage())
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub struct SpecifiedValue(Vec<SpecifiedFilter>);

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
    }

    pub mod computed_value {
        use app_units::Au;
        use values::CSSFloat;
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
            }
            Ok(())
        }
    }

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        computed_value::T::new(Vec::new())
    }

    pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
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
                        "hue-rotate" => Angle::parse(input).map(SpecifiedFilter::HueRotate),
                        "invert" => parse_factor(input).map(SpecifiedFilter::Invert),
                        "opacity" => parse_factor(input).map(SpecifiedFilter::Opacity),
                        "saturate" => parse_factor(input).map(SpecifiedFilter::Saturate),
                        "sepia" => parse_factor(input).map(SpecifiedFilter::Sepia),
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

pub fn parse_origin(_: &ParserContext, input: &mut Parser) -> Result<OriginParseResult,()> {
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
            match LengthOrPercentage::parse(input) {
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

${helpers.single_keyword("backface-visibility",
                         "visible hidden",
                         animatable=False)}

${helpers.single_keyword("transform-box",
                         "border-box fill-box view-box",
                         products="gecko",
                         animatable=False)}

${helpers.single_keyword("transform-style",
                         "auto flat preserve-3d",
                         animatable=False)}

<%helpers:longhand name="transform-origin" products="servo" animatable="True">
    use app_units::Au;
    use std::fmt;
    use style_traits::ToCss;
    use values::HasViewportPercentage;
    use values::specified::{Length, LengthOrPercentage, Percentage};

    pub mod computed_value {
        use properties::animated_properties::Interpolate;
        use values::computed::{Length, LengthOrPercentage};

        #[derive(Clone, Copy, Debug, PartialEq)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub struct T {
            pub horizontal: LengthOrPercentage,
            pub vertical: LengthOrPercentage,
            pub depth: Length,
        }

        impl Interpolate for T {
            fn interpolate(&self, other: &Self, time: f64) -> Result<Self, ()> {
                Ok(T {
                    horizontal: try!(self.horizontal.interpolate(&other.horizontal, time)),
                    vertical: try!(self.vertical.interpolate(&other.vertical, time)),
                    depth: try!(self.depth.interpolate(&other.depth, time)),
                })
            }
        }
    }

    impl HasViewportPercentage for SpecifiedValue {
        fn has_viewport_percentage(&self) -> bool {
            self.horizontal.has_viewport_percentage() ||
            self.vertical.has_viewport_percentage() ||
            self.depth.has_viewport_percentage()
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub struct SpecifiedValue {
        horizontal: LengthOrPercentage,
        vertical: LengthOrPercentage,
        depth: Length,
    }

    impl ToCss for computed_value::T {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            try!(self.horizontal.to_css(dest));
            try!(dest.write_str(" "));
            try!(self.vertical.to_css(dest));
            try!(dest.write_str(" "));
            self.depth.to_css(dest)
        }
    }

    impl ToCss for SpecifiedValue {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            try!(self.horizontal.to_css(dest));
            try!(dest.write_str(" "));
            try!(self.vertical.to_css(dest));
            try!(dest.write_str(" "));
            self.depth.to_css(dest)
        }
    }

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        computed_value::T {
            horizontal: computed::LengthOrPercentage::Percentage(0.5),
            vertical: computed::LengthOrPercentage::Percentage(0.5),
            depth: Au(0),
        }
    }

    pub fn parse(context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue,()> {
        let result = try!(super::parse_origin(context, input));
        Ok(SpecifiedValue {
            horizontal: result.horizontal.unwrap_or(LengthOrPercentage::Percentage(Percentage(0.5))),
            vertical: result.vertical.unwrap_or(LengthOrPercentage::Percentage(Percentage(0.5))),
            depth: result.depth.unwrap_or(Length::Absolute(Au(0))),
        })
    }

    impl ToComputedValue for SpecifiedValue {
        type ComputedValue = computed_value::T;

        #[inline]
        fn to_computed_value(&self, context: &Context) -> computed_value::T {
            computed_value::T {
                horizontal: self.horizontal.to_computed_value(context),
                vertical: self.vertical.to_computed_value(context),
                depth: self.depth.to_computed_value(context),
            }
        }

        #[inline]
        fn from_computed_value(computed: &computed_value::T) -> Self {
            SpecifiedValue {
                horizontal: ToComputedValue::from_computed_value(&computed.horizontal),
                vertical: ToComputedValue::from_computed_value(&computed.vertical),
                depth: ToComputedValue::from_computed_value(&computed.depth),
            }
        }
    }
</%helpers:longhand>

${helpers.predefined_type("perspective",
                          "LengthOrNone",
                          "Either::Second(None_)",
                          products="servo",
                          animatable=True)}

// FIXME: This prop should be animatable
<%helpers:longhand name="perspective-origin" products="servo" animatable="False">
    use std::fmt;
    use style_traits::ToCss;
    use values::HasViewportPercentage;
    use values::specified::{LengthOrPercentage, Percentage};

    pub mod computed_value {
        use values::computed::LengthOrPercentage;

        #[derive(Clone, Copy, Debug, PartialEq)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub struct T {
            pub horizontal: LengthOrPercentage,
            pub vertical: LengthOrPercentage,
        }
    }

    impl ToCss for computed_value::T {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            try!(self.horizontal.to_css(dest));
            try!(dest.write_str(" "));
            self.vertical.to_css(dest)
        }
    }

    impl HasViewportPercentage for SpecifiedValue {
        fn has_viewport_percentage(&self) -> bool {
            self.horizontal.has_viewport_percentage() || self.vertical.has_viewport_percentage()
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub struct SpecifiedValue {
        horizontal: LengthOrPercentage,
        vertical: LengthOrPercentage,
    }

    impl ToCss for SpecifiedValue {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            try!(self.horizontal.to_css(dest));
            try!(dest.write_str(" "));
            self.vertical.to_css(dest)
        }
    }

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        computed_value::T {
            horizontal: computed::LengthOrPercentage::Percentage(0.5),
            vertical: computed::LengthOrPercentage::Percentage(0.5),
        }
    }

    pub fn parse(context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue,()> {
        let result = try!(super::parse_origin(context, input));
        match result.depth {
            Some(_) => Err(()),
            None => Ok(SpecifiedValue {
                horizontal: result.horizontal.unwrap_or(LengthOrPercentage::Percentage(Percentage(0.5))),
                vertical: result.vertical.unwrap_or(LengthOrPercentage::Percentage(Percentage(0.5))),
            })
        }
    }

    impl ToComputedValue for SpecifiedValue {
        type ComputedValue = computed_value::T;

        #[inline]
        fn to_computed_value(&self, context: &Context) -> computed_value::T {
            computed_value::T {
                horizontal: self.horizontal.to_computed_value(context),
                vertical: self.vertical.to_computed_value(context),
            }
        }

        #[inline]
        fn from_computed_value(computed: &computed_value::T) -> Self {
            SpecifiedValue {
                horizontal: ToComputedValue::from_computed_value(&computed.horizontal),
                vertical: ToComputedValue::from_computed_value(&computed.vertical),
            }
        }
    }
</%helpers:longhand>

${helpers.single_keyword("mix-blend-mode",
                         """normal multiply screen overlay darken lighten color-dodge
                            color-burn hard-light soft-light difference exclusion hue
                            saturation color luminosity""", gecko_constant_prefix="NS_STYLE_BLEND",
                         animatable=False)}
