/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />

<% data.new_style_struct("InheritedText", inherited=True, gecko_name="Text") %>

<%helpers:longhand name="line-height" animatable="True"
                   spec="https://drafts.csswg.org/css2/visudet.html#propdef-line-height">
    use std::fmt;
    use style_traits::ToCss;
    use values::{CSSFloat, HasViewportPercentage};

    impl HasViewportPercentage for SpecifiedValue {
        fn has_viewport_percentage(&self) -> bool {
            match *self {
                SpecifiedValue::LengthOrPercentage(length) => length.has_viewport_percentage(),
                _ => false
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, Copy)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub enum SpecifiedValue {
        Normal,
        % if product == "gecko":
            MozBlockHeight,
        % endif
        Number(CSSFloat),
        LengthOrPercentage(specified::LengthOrPercentage),
    }

    impl ToCss for SpecifiedValue {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match *self {
                SpecifiedValue::Normal => dest.write_str("normal"),
                % if product == "gecko":
                    SpecifiedValue::MozBlockHeight => dest.write_str("-moz-block-height"),
                % endif
                SpecifiedValue::LengthOrPercentage(value) => value.to_css(dest),
                SpecifiedValue::Number(number) => write!(dest, "{}", number),
            }
        }
    }
    /// normal | <number> | <length> | <percentage>
    pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
        use cssparser::Token;
        use std::ascii::AsciiExt;
        input.try(specified::LengthOrPercentage::parse_non_negative)
        .map(SpecifiedValue::LengthOrPercentage)
        .or_else(|()| {
            match try!(input.next()) {
                Token::Number(ref value) if value.value >= 0. => {
                    Ok(SpecifiedValue::Number(value.value))
                }
                Token::Ident(ref value) if value.eq_ignore_ascii_case("normal") => {
                    Ok(SpecifiedValue::Normal)
                }
                % if product == "gecko":
                Token::Ident(ref value) if value.eq_ignore_ascii_case("-moz-block-height") => {
                    Ok(SpecifiedValue::MozBlockHeight)
                }
                % endif
                _ => Err(()),
            }
        })
    }
    pub mod computed_value {
        use app_units::Au;
        use std::fmt;
        use values::CSSFloat;
        #[derive(PartialEq, Copy, Clone, Debug)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub enum T {
            Normal,
            % if product == "gecko":
                MozBlockHeight,
            % endif
            Length(Au),
            Number(CSSFloat),
        }
    }
    impl ToCss for computed_value::T {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match *self {
                computed_value::T::Normal => dest.write_str("normal"),
                % if product == "gecko":
                    computed_value::T::MozBlockHeight => dest.write_str("-moz-block-height"),
                % endif
                computed_value::T::Length(length) => length.to_css(dest),
                computed_value::T::Number(number) => write!(dest, "{}", number),
            }
        }
    }
     #[inline]
    pub fn get_initial_value() -> computed_value::T { computed_value::T::Normal }

    impl ToComputedValue for SpecifiedValue {
        type ComputedValue = computed_value::T;

        #[inline]
        fn to_computed_value(&self, context: &Context) -> computed_value::T {
            match *self {
                SpecifiedValue::Normal => computed_value::T::Normal,
                % if product == "gecko":
                    SpecifiedValue::MozBlockHeight => computed_value::T::MozBlockHeight,
                % endif
                SpecifiedValue::Number(value) => computed_value::T::Number(value),
                SpecifiedValue::LengthOrPercentage(value) => {
                    match value {
                        specified::LengthOrPercentage::Length(value) =>
                            computed_value::T::Length(value.to_computed_value(context)),
                        specified::LengthOrPercentage::Percentage(specified::Percentage(value)) => {
                            let fr = specified::Length::FontRelative(specified::FontRelativeLength::Em(value));
                            computed_value::T::Length(fr.to_computed_value(context))
                        },
                        specified::LengthOrPercentage::Calc(calc) => {
                            let calc = calc.to_computed_value(context);
                            let fr = specified::FontRelativeLength::Em(calc.percentage());
                            let fr = specified::Length::FontRelative(fr);
                            computed_value::T::Length(calc.length() + fr.to_computed_value(context))
                        }
                    }
                }
            }
        }

        #[inline]
        fn from_computed_value(computed: &computed_value::T) -> Self {
            match *computed {
                computed_value::T::Normal => SpecifiedValue::Normal,
                % if product == "gecko":
                    computed_value::T::MozBlockHeight => SpecifiedValue::MozBlockHeight,
                % endif
                computed_value::T::Number(value) => SpecifiedValue::Number(value),
                computed_value::T::Length(au) => {
                    SpecifiedValue::LengthOrPercentage(specified::LengthOrPercentage::Length(
                        ToComputedValue::from_computed_value(&au)
                    ))
                }
            }
        }
    }
</%helpers:longhand>

// CSS Text Module Level 3

// TODO(pcwalton): `full-width`
${helpers.single_keyword("text-transform",
                         "none capitalize uppercase lowercase",
                         extra_gecko_values="full-width",
                         animatable=False,
                         spec="https://drafts.csswg.org/css-text/#propdef-text-transform")}

${helpers.single_keyword("hyphens", "none manual auto",
                         products="gecko", animatable=False,
                         spec="https://drafts.csswg.org/css-text/#propdef-hyphens")}

${helpers.predefined_type("text-indent",
                          "LengthOrPercentage",
                          "computed::LengthOrPercentage::Length(Au(0))",
                          animatable=True,
                          spec="https://drafts.csswg.org/css-text/#propdef-text-indent")}

// Also known as "word-wrap" (which is more popular because of IE), but this is the preferred
// name per CSS-TEXT 6.2.
${helpers.single_keyword("overflow-wrap",
                         "normal break-word",
                         gecko_constant_prefix="NS_STYLE_OVERFLOWWRAP",
                         animatable=False,
                         spec="https://drafts.csswg.org/css-text/#propdef-overflow-wrap")}

// TODO(pcwalton): Support `word-break: keep-all` once we have better CJK support.
${helpers.single_keyword("word-break",
                         "normal break-all keep-all",
                         gecko_constant_prefix="NS_STYLE_WORDBREAK",
                         animatable=False,
                         spec="https://drafts.csswg.org/css-text/#propdef-word-break")}

// TODO(pcwalton): Support `text-justify: distribute`.
${helpers.single_keyword("text-justify",
                         "auto none inter-word",
                         products="servo",
                         animatable=False,
                         spec="https://drafts.csswg.org/css-text/#propdef-text-justify")}

${helpers.single_keyword("text-align-last",
                         "auto start end left right center justify",
                         products="gecko",
                         gecko_constant_prefix="NS_STYLE_TEXT_ALIGN",
                         animatable=False,
                         spec="https://drafts.csswg.org/css-text/#propdef-text-align-last")}

// TODO make this a shorthand and implement text-align-last/text-align-all
<%helpers:longhand name="text-align" animatable="False" spec="https://drafts.csswg.org/css-text/#propdef-text-align">
    pub use self::computed_value::T as SpecifiedValue;
    use values::computed::ComputedValueAsSpecified;
    use values::NoViewportPercentage;
    impl ComputedValueAsSpecified for SpecifiedValue {}
    impl NoViewportPercentage for SpecifiedValue {}
    pub mod computed_value {
        use style_traits::ToCss;
        macro_rules! define_text_align {
            ( $( $name: ident ( $string: expr ) => $discriminant: expr, )+ ) => {
                define_css_keyword_enum! { T:
                    $(
                        $string => $name,
                    )+
                }
                impl T {
                    pub fn to_u32(self) -> u32 {
                        match self {
                            $(
                                T::$name => $discriminant,
                            )+
                        }
                    }
                    pub fn from_u32(discriminant: u32) -> Option<T> {
                        match discriminant {
                            $(
                                $discriminant => Some(T::$name),
                            )+
                            _ => None
                        }
                    }
                }
            }
        }
        define_text_align! {
            start("start") => 0,
            end("end") => 1,
            left("left") => 2,
            right("right") => 3,
            center("center") => 4,
            justify("justify") => 5,
            % if product == "servo":
            servo_center("-servo-center") => 6,
            servo_left("-servo-left") => 7,
            servo_right("-servo-right") => 8,
            % else:
            _moz_center("-moz-center") => 6,
            _moz_left("-moz-left") => 7,
            _moz_right("-moz-right") => 8,
            match_parent("match-parent") => 9,
            % endif
        }
    }
    #[inline] pub fn get_initial_value() -> computed_value::T {
        computed_value::T::start
    }
    pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
        computed_value::T::parse(input)
    }
</%helpers:longhand>

// FIXME: This prop should be animatable.
<%helpers:longhand name="letter-spacing" animatable="False"
                   spec="https://drafts.csswg.org/css-text/#propdef-letter-spacing">
    use std::fmt;
    use style_traits::ToCss;
    use values::HasViewportPercentage;

    impl HasViewportPercentage for SpecifiedValue {
        fn has_viewport_percentage(&self) -> bool {
            match *self {
                SpecifiedValue::Specified(length) => length.has_viewport_percentage(),
                _ => false
            }
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub enum SpecifiedValue {
        Normal,
        Specified(specified::Length),
    }

    impl ToCss for SpecifiedValue {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match *self {
                SpecifiedValue::Normal => dest.write_str("normal"),
                SpecifiedValue::Specified(l) => l.to_css(dest),
            }
        }
    }

    pub mod computed_value {
        use app_units::Au;
        #[derive(Debug, Clone, PartialEq)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub struct T(pub Option<Au>);
    }

    impl ToCss for computed_value::T {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match self.0 {
                None => dest.write_str("normal"),
                Some(l) => l.to_css(dest),
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
            match *self {
                SpecifiedValue::Normal => computed_value::T(None),
                SpecifiedValue::Specified(l) =>
                    computed_value::T(Some(l.to_computed_value(context)))
            }
        }

        #[inline]
        fn from_computed_value(computed: &computed_value::T) -> Self {
            computed.0.map(|ref au| {
                SpecifiedValue::Specified(ToComputedValue::from_computed_value(au))
            }).unwrap_or(SpecifiedValue::Normal)
        }
    }

    pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
        if input.try(|input| input.expect_ident_matching("normal")).is_ok() {
            Ok(SpecifiedValue::Normal)
        } else {
            specified::Length::parse_non_negative(input).map(SpecifiedValue::Specified)
        }
    }
</%helpers:longhand>

<%helpers:longhand name="word-spacing" animatable="False"
                   spec="https://drafts.csswg.org/css-text/#propdef-word-spacing">
    use std::fmt;
    use style_traits::ToCss;
    use values::HasViewportPercentage;

    impl HasViewportPercentage for SpecifiedValue {
        fn has_viewport_percentage(&self) -> bool {
            match *self {
                SpecifiedValue::Specified(length) => length.has_viewport_percentage(),
                _ => false
            }
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub enum SpecifiedValue {
        Normal,
        Specified(specified::LengthOrPercentage),
    }

    impl ToCss for SpecifiedValue {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match *self {
                SpecifiedValue::Normal => dest.write_str("normal"),
                SpecifiedValue::Specified(l) => l.to_css(dest),
            }
        }
    }

    pub mod computed_value {
        use values::computed::LengthOrPercentage;
        #[derive(Debug, Clone, PartialEq)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub struct T(pub Option<LengthOrPercentage>);
    }

    impl ToCss for computed_value::T {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match self.0 {
                None => dest.write_str("normal"),
                Some(l) => l.to_css(dest),
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
            match *self {
                SpecifiedValue::Normal => computed_value::T(None),
                SpecifiedValue::Specified(l) =>
                    computed_value::T(Some(l.to_computed_value(context))),
            }
        }

        #[inline]
        fn from_computed_value(computed: &computed_value::T) -> Self {
            computed.0.map(|ref lop| {
                SpecifiedValue::Specified(ToComputedValue::from_computed_value(lop))
            }).unwrap_or(SpecifiedValue::Normal)
        }
    }

    pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
        if input.try(|input| input.expect_ident_matching("normal")).is_ok() {
            Ok(SpecifiedValue::Normal)
        } else {
            specified::LengthOrPercentage::parse_non_negative(input)
                                          .map(SpecifiedValue::Specified)
        }
    }
</%helpers:longhand>

<%helpers:longhand name="-servo-text-decorations-in-effect"
                   derived_from="display text-decoration"
                   need_clone="True" products="servo"
                   animatable="False"
                   spec="Nonstandard (Internal property used by Servo)">
    use cssparser::RGBA;
    use std::fmt;
    use style_traits::ToCss;
    use values::NoViewportPercentage;
    use values::computed::ComputedValueAsSpecified;

    impl ComputedValueAsSpecified for SpecifiedValue {}
    impl NoViewportPercentage for SpecifiedValue {}

    #[derive(Clone, PartialEq, Copy, Debug)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub struct SpecifiedValue {
        pub underline: Option<RGBA>,
        pub overline: Option<RGBA>,
        pub line_through: Option<RGBA>,
    }

    pub mod computed_value {
        pub type T = super::SpecifiedValue;
    }

    impl ToCss for SpecifiedValue {
        fn to_css<W>(&self, _: &mut W) -> fmt::Result where W: fmt::Write {
            // Web compat doesn't matter here.
            Ok(())
        }
    }

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        SpecifiedValue {
            underline: None,
            overline: None,
            line_through: None,
        }
    }

    fn maybe(flag: bool, context: &Context) -> Option<RGBA> {
        if flag {
            Some(context.style().get_color().clone_color())
        } else {
            None
        }
    }

    fn derive(context: &Context) -> computed_value::T {
        // Start with no declarations if this is an atomic inline-level box; otherwise, start with the
        // declarations in effect and add in the text decorations that this block specifies.
        let mut result = match context.style().get_box().clone_display() {
            super::display::computed_value::T::inline_block |
            super::display::computed_value::T::inline_table => SpecifiedValue {
                underline: None,
                overline: None,
                line_through: None,
            },
            _ => context.inherited_style().get_inheritedtext().clone__servo_text_decorations_in_effect()
        };

        result.underline = maybe(context.style().get_text().has_underline()
                                 || result.underline.is_some(), context);
        result.overline = maybe(context.style().get_text().has_overline()
                                || result.overline.is_some(), context);
        result.line_through = maybe(context.style().get_text().has_line_through()
                                    || result.line_through.is_some(), context);

        result
    }

    #[inline]
    pub fn derive_from_text_decoration(context: &mut Context) {
        let derived = derive(context);
        context.mutate_style().mutate_inheritedtext().set__servo_text_decorations_in_effect(derived);
    }

    #[inline]
    pub fn derive_from_display(context: &mut Context) {
        let derived = derive(context);
        context.mutate_style().mutate_inheritedtext().set__servo_text_decorations_in_effect(derived);
    }
</%helpers:longhand>

<%helpers:single_keyword_computed name="white-space"
                                  values="normal pre nowrap pre-wrap pre-line"
                                  gecko_constant_prefix="NS_STYLE_WHITESPACE"
                                  animatable="False"
                                  spec="https://drafts.csswg.org/css-text/#propdef-white-space">
    use values::computed::ComputedValueAsSpecified;
    use values::NoViewportPercentage;
    impl ComputedValueAsSpecified for SpecifiedValue {}
    impl NoViewportPercentage for SpecifiedValue {}

    impl SpecifiedValue {
        pub fn allow_wrap(&self) -> bool {
            match *self {
                SpecifiedValue::nowrap |
                SpecifiedValue::pre => false,
                SpecifiedValue::normal |
                SpecifiedValue::pre_wrap |
                SpecifiedValue::pre_line => true,
            }
        }

        pub fn preserve_newlines(&self) -> bool {
            match *self {
                SpecifiedValue::normal |
                SpecifiedValue::nowrap => false,
                SpecifiedValue::pre |
                SpecifiedValue::pre_wrap |
                SpecifiedValue::pre_line => true,
            }
        }

        pub fn preserve_spaces(&self) -> bool {
            match *self {
                SpecifiedValue::normal |
                SpecifiedValue::nowrap |
                SpecifiedValue::pre_line => false,
                SpecifiedValue::pre |
                SpecifiedValue::pre_wrap => true,
            }
        }
    }
</%helpers:single_keyword_computed>

<%helpers:longhand name="text-shadow" animatable="True"
                   spec="https://drafts.csswg.org/css-text-decor/#propdef-text-shadow">
    use cssparser;
    use std::fmt;
    use style_traits::ToCss;
    use values::HasViewportPercentage;

    impl HasViewportPercentage for SpecifiedValue {
        fn has_viewport_percentage(&self) -> bool {
            let &SpecifiedValue(ref vec) = self;
            vec.iter().any(|ref x| x .has_viewport_percentage())
        }
    }

    #[derive(Clone, PartialEq, Debug)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub struct SpecifiedValue(Vec<SpecifiedTextShadow>);

    impl HasViewportPercentage for SpecifiedTextShadow {
        fn has_viewport_percentage(&self) -> bool {
            self.offset_x.has_viewport_percentage() ||
            self.offset_y.has_viewport_percentage() ||
            self.blur_radius.has_viewport_percentage()
        }
    }

    #[derive(Clone, PartialEq, Debug)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub struct SpecifiedTextShadow {
        pub offset_x: specified::Length,
        pub offset_y: specified::Length,
        pub blur_radius: specified::Length,
        pub color: Option<specified::CSSColor>,
    }

    pub mod computed_value {
        use app_units::Au;
        use cssparser::Color;

        #[derive(Clone, PartialEq, Debug)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub struct T(pub Vec<TextShadow>);

        #[derive(Clone, PartialEq, Debug)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub struct TextShadow {
            pub offset_x: Au,
            pub offset_y: Au,
            pub blur_radius: Au,
            pub color: Color,
        }
    }

    impl ToCss for computed_value::T {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            let mut iter = self.0.iter();
            if let Some(shadow) = iter.next() {
                try!(shadow.to_css(dest));
            } else {
                try!(dest.write_str("none"));
                return Ok(())
            }
            for shadow in iter {
                try!(dest.write_str(", "));
                try!(shadow.to_css(dest));
            }
            Ok(())
        }
    }

    impl ToCss for computed_value::TextShadow {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            try!(self.offset_x.to_css(dest));
            try!(dest.write_str(" "));
            try!(self.offset_y.to_css(dest));
            try!(dest.write_str(" "));
            try!(self.blur_radius.to_css(dest));
            try!(dest.write_str(" "));
            try!(self.color.to_css(dest));
            Ok(())
        }
    }

    impl ToCss for SpecifiedValue {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            let mut iter = self.0.iter();
            if let Some(shadow) = iter.next() {
                try!(shadow.to_css(dest));
            } else {
                try!(dest.write_str("none"));
                return Ok(())
            }
            for shadow in iter {
                try!(dest.write_str(", "));
                try!(shadow.to_css(dest));
            }
            Ok(())
        }
    }

    impl ToCss for SpecifiedTextShadow {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            try!(self.offset_x.to_css(dest));
            try!(dest.write_str(" "));
            try!(self.offset_y.to_css(dest));
            try!(dest.write_str(" "));
            try!(self.blur_radius.to_css(dest));

            if let Some(ref color) = self.color {
                try!(dest.write_str(" "));
                try!(color.to_css(dest));
            }
            Ok(())
        }
    }

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        computed_value::T(Vec::new())
    }

    pub fn parse(context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue,()> {
        if input.try(|input| input.expect_ident_matching("none")).is_ok() {
            Ok(SpecifiedValue(Vec::new()))
        } else {
            input.parse_comma_separated(|i| parse_one_text_shadow(context, i)).map(SpecifiedValue)
        }
    }

    fn parse_one_text_shadow(context: &ParserContext, input: &mut Parser) -> Result<SpecifiedTextShadow,()> {
        use app_units::Au;
        let mut lengths = [specified::Length::Absolute(Au(0)); 3];
        let mut lengths_parsed = false;
        let mut color = None;

        loop {
            if !lengths_parsed {
                if let Ok(value) = input.try(|i| specified::Length::parse(context, i)) {
                    lengths[0] = value;
                    let mut length_parsed_count = 1;
                    while length_parsed_count < 3 {
                        if let Ok(value) = input.try(|i| specified::Length::parse(context, i)) {
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
                if let Ok(value) = input.try(|i| specified::CSSColor::parse(context, i)) {
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

        Ok(SpecifiedTextShadow {
            offset_x: lengths[0],
            offset_y: lengths[1],
            blur_radius: lengths[2],
            color: color,
        })
    }

    impl ToComputedValue for SpecifiedValue {
        type ComputedValue = computed_value::T;

        fn to_computed_value(&self, context: &Context) -> computed_value::T {
            computed_value::T(self.0.iter().map(|value| {
                computed_value::TextShadow {
                    offset_x: value.offset_x.to_computed_value(context),
                    offset_y: value.offset_y.to_computed_value(context),
                    blur_radius: value.blur_radius.to_computed_value(context),
                    color: value.color
                                .as_ref()
                                .map(|color| color.parsed)
                                .unwrap_or(cssparser::Color::CurrentColor),
                }
            }).collect())
        }

        fn from_computed_value(computed: &computed_value::T) -> Self {
            SpecifiedValue(computed.0.iter().map(|value| {
                SpecifiedTextShadow {
                    offset_x: ToComputedValue::from_computed_value(&value.offset_x),
                    offset_y: ToComputedValue::from_computed_value(&value.offset_y),
                    blur_radius: ToComputedValue::from_computed_value(&value.blur_radius),
                    color: Some(ToComputedValue::from_computed_value(&value.color)),
                }
            }).collect())
        }
    }
</%helpers:longhand>

<%helpers:longhand name="text-emphasis-style" products="gecko" need_clone="True" animatable="False"
                   spec="https://drafts.csswg.org/css-text-decor/#propdef-text-emphasis-style">
    use computed_values::writing_mode::T as writing_mode;
    use std::fmt;
    use style_traits::ToCss;
    use unicode_segmentation::UnicodeSegmentation;
    use values::NoViewportPercentage;

    impl NoViewportPercentage for SpecifiedValue {}

    pub mod computed_value {
        #[derive(Debug, Clone, PartialEq)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub enum T {
            Keyword(KeywordValue),
            None,
            String(String),
        }

        #[derive(Debug, Clone, PartialEq)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub struct KeywordValue {
            pub fill: bool,
            pub shape: super::ShapeKeyword,
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub enum SpecifiedValue {
        Keyword(KeywordValue),
        None,
        String(String),
    }

    impl ToCss for computed_value::T {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match *self {
                computed_value::T::Keyword(ref keyword) => keyword.to_css(dest),
                computed_value::T::None => dest.write_str("none"),
                computed_value::T::String(ref string) => write!(dest, "\"{}\"", string),
            }
        }
    }
    impl ToCss for SpecifiedValue {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match *self {
                SpecifiedValue::Keyword(ref keyword) => keyword.to_css(dest),
                SpecifiedValue::None => dest.write_str("none"),
                SpecifiedValue::String(ref string) => write!(dest, "\"{}\"", string),
            }
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub enum KeywordValue {
        Fill(bool),
        Shape(ShapeKeyword),
        FillAndShape(bool, ShapeKeyword),
    }

    impl ToCss for KeywordValue {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            if let Some(fill) = self.fill() {
                if fill {
                    try!(dest.write_str("filled"));
                } else {
                    try!(dest.write_str("open"));
                }
            }
            if let Some(shape) = self.shape() {
                if self.fill().is_some() {
                    try!(dest.write_str(" "));
                }
                try!(shape.to_css(dest));
            }
            Ok(())
        }
    }
    impl ToCss for computed_value::KeywordValue {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            if self.fill {
                try!(dest.write_str("filled"));
            } else {
                try!(dest.write_str("open"));
            }
            try!(dest.write_str(" "));
            self.shape.to_css(dest)
        }
    }

    impl KeywordValue {
        fn fill(&self) -> Option<bool> {
            match *self {
                KeywordValue::Fill(fill) |
                KeywordValue::FillAndShape(fill,_) => Some(fill),
                _ => None,
            }
        }
        fn shape(&self) -> Option<ShapeKeyword> {
            match *self {
                KeywordValue::Shape(shape) |
                KeywordValue::FillAndShape(_, shape) => Some(shape),
                _ => None,
            }
        }
    }

    define_css_keyword_enum!(ShapeKeyword:
                             "dot" => Dot,
                             "circle" => Circle,
                             "double-circle" => DoubleCircle,
                             "triangle" => Triangle,
                             "sesame" => Sesame);

    impl ShapeKeyword {
        pub fn char(&self, fill: bool) -> &str {
            match *self {
                ShapeKeyword::Dot => if fill { "\u{2022}" } else { "\u{25e6}" },
                ShapeKeyword::Circle =>  if fill { "\u{25cf}" } else { "\u{25cb}" },
                ShapeKeyword::DoubleCircle =>  if fill { "\u{25c9}" } else { "\u{25ce}" },
                ShapeKeyword::Triangle =>  if fill { "\u{25b2}" } else { "\u{25b3}" },
                ShapeKeyword::Sesame =>  if fill { "\u{fe45}" } else { "\u{fe46}" },
            }
        }
    }

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        computed_value::T::None
    }

    #[inline]
    pub fn get_initial_specified_value() -> SpecifiedValue {
        SpecifiedValue::None
    }

    impl ToComputedValue for SpecifiedValue {
        type ComputedValue = computed_value::T;

        #[inline]
        fn to_computed_value(&self, context: &Context) -> computed_value::T {
            match *self {
                SpecifiedValue::Keyword(ref keyword) => {
                    let default_shape = if context.style().get_inheritedbox()
                                                  .clone_writing_mode() == writing_mode::horizontal_tb {
                        ShapeKeyword::Circle
                    } else {
                        ShapeKeyword::Sesame
                    };
                    computed_value::T::Keyword(computed_value::KeywordValue {
                        fill: keyword.fill().unwrap_or(true),
                        shape: keyword.shape().unwrap_or(default_shape),
                    })
                },
                SpecifiedValue::None => computed_value::T::None,
                SpecifiedValue::String(ref s) => {
                    // Passing `true` to iterate over extended grapheme clusters, following
                    // recommendation at http://www.unicode.org/reports/tr29/#Grapheme_Cluster_Boundaries
                    let string = s.graphemes(true).next().unwrap_or("").to_string();
                    computed_value::T::String(string)
                }
            }
        }
        #[inline]
        fn from_computed_value(computed: &computed_value::T) -> Self {
            match *computed {
                computed_value::T::Keyword(ref keyword) =>
                    SpecifiedValue::Keyword(KeywordValue::FillAndShape(keyword.fill,keyword.shape)),
                computed_value::T::None => SpecifiedValue::None,
                computed_value::T::String(ref string) => SpecifiedValue::String(string.clone())
            }
        }
    }

    pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
        if input.try(|input| input.expect_ident_matching("none")).is_ok() {
            return Ok(SpecifiedValue::None);
        }

        if let Ok(s) = input.try(|input| input.expect_string()) {
            // Handle <string>
            return Ok(SpecifiedValue::String(s.into_owned()));
        }

        // Handle a pair of keywords
        let mut shape = input.try(ShapeKeyword::parse);
        let fill = if input.try(|input| input.expect_ident_matching("filled")).is_ok() {
            Some(true)
        } else if input.try(|input| input.expect_ident_matching("open")).is_ok() {
            Some(false)
        } else { None };
        if shape.is_err() {
            shape = input.try(ShapeKeyword::parse);
        }

        // At least one of shape or fill must be handled
        let keyword_value = match (fill, shape) {
            (Some(fill), Ok(shape)) => KeywordValue::FillAndShape(fill,shape),
            (Some(fill), Err(_)) => KeywordValue::Fill(fill),
            (None, Ok(shape)) => KeywordValue::Shape(shape),
            _ => return Err(()),
        };
        Ok(SpecifiedValue::Keyword(keyword_value))
    }
</%helpers:longhand>

<%helpers:longhand name="text-emphasis-position" animatable="False" products="none"
                   spec="https://drafts.csswg.org/css-text-decor/#propdef-text-emphasis-position">
    use std::fmt;
    use values::computed::ComputedValueAsSpecified;
    use values::NoViewportPercentage;
    use style_traits::ToCss;

    define_css_keyword_enum!(HorizontalWritingModeValue:
                             "over" => Over,
                             "under" => Under);
    define_css_keyword_enum!(VerticalWritingModeValue:
                             "right" => Right,
                             "left" => Left);

    #[derive(Debug, Clone, PartialEq)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub struct SpecifiedValue(pub HorizontalWritingModeValue, pub VerticalWritingModeValue);

    pub mod computed_value {
        pub type T = super::SpecifiedValue;
    }

    impl ComputedValueAsSpecified for SpecifiedValue {}
    impl NoViewportPercentage for SpecifiedValue {}

    pub fn get_initial_value() -> computed_value::T {
        SpecifiedValue(HorizontalWritingModeValue::Over, VerticalWritingModeValue::Right)
    }

    pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
       if let Ok(horizontal) = input.try(|input| HorizontalWritingModeValue::parse(input)) {
            let vertical = try!(VerticalWritingModeValue::parse(input));
            Ok(SpecifiedValue(horizontal, vertical))
        } else {
            let vertical = try!(VerticalWritingModeValue::parse(input));
            let horizontal = try!(HorizontalWritingModeValue::parse(input));
            Ok(SpecifiedValue(horizontal, vertical))
        }
    }

    impl ToCss for SpecifiedValue {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            let SpecifiedValue(horizontal, vertical) = *self;
            try!(horizontal.to_css(dest));
            try!(dest.write_char(' '));
            vertical.to_css(dest)
        }
    }
</%helpers:longhand>

${helpers.predefined_type("text-emphasis-color", "CSSColor",
                          "::cssparser::Color::CurrentColor",
                          products="gecko",animatable=True,
                          complex_color=True, need_clone=True,
                          spec="https://drafts.csswg.org/css-text-decor/#propdef-text-emphasis-color")}

// CSS Compatibility
// https://compat.spec.whatwg.org
${helpers.predefined_type(
    "-webkit-text-fill-color", "CSSColor",
    "CSSParserColor::CurrentColor",
    products="gecko", animatable=True,
    complex_color=True, need_clone=True,
    spec="https://compat.spec.whatwg.org/#the-webkit-text-fill-color")}

${helpers.predefined_type(
    "-webkit-text-stroke-color", "CSSColor",
    "CSSParserColor::CurrentColor",
    products="gecko", animatable=True,
    complex_color=True, need_clone=True,
    spec="https://compat.spec.whatwg.org/#the-webkit-text-stroke-color")}

<%helpers:longhand products="gecko" name="-webkit-text-stroke-width" animatable="False"
                   spec="https://compat.spec.whatwg.org/#the-webkit-text-stroke-width">
    use app_units::Au;
    use std::fmt;
    use style_traits::ToCss;
    use values::HasViewportPercentage;
    use values::specified::BorderWidth;

    pub type SpecifiedValue = BorderWidth;

    #[inline]
    pub fn parse(context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
        BorderWidth::parse(context, input)
    }

    pub mod computed_value {
        use app_units::Au;
        pub type T = Au;
    }
    #[inline] pub fn get_initial_value() -> computed_value::T {
        Au::from_px(0)
    }
</%helpers:longhand>

// CSS Ruby Layout Module Level 1
// https://drafts.csswg.org/css-ruby/
${helpers.single_keyword("ruby-align", "start center space-between space-around",
                         products="gecko", animatable=False,
                         spec="https://drafts.csswg.org/css-ruby/#ruby-align-property")}

${helpers.single_keyword("ruby-position", "over under",
                         products="gecko", animatable=False,
                         spec="https://drafts.csswg.org/css-ruby/#ruby-position-property")}

// CSS Writing Modes Module Level 3
// https://drafts.csswg.org/css-writing-modes-3/

// The spec has "digits <integer>?" value in addition. But that value is
// at-risk, and Gecko's layout code doesn't support that either. So we
// can just take the easy way for now.
${helpers.single_keyword("text-combine-upright", "none all",
                         products="gecko", animatable=False,
                         spec="https://drafts.csswg.org/css-writing-modes-3/#text-combine-upright")}

// SVG 1.1: Section 11 - Painting: Filling, Stroking and Marker Symbols
${helpers.single_keyword("text-rendering",
                         "auto optimizespeed optimizelegibility geometricprecision",
                         animatable=False,
                         spec="https://www.w3.org/TR/SVG11/painting.html#TextRenderingProperty")}
