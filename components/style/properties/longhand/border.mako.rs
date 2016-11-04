/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />
<% from data import Method %>

<% data.new_style_struct("Border", inherited=False,
                   additional_methods=[Method("border_" + side + "_has_nonzero_width",
                                              "bool") for side in ["top", "right", "bottom", "left"]]) %>

% for side in ["top", "right", "bottom", "left"]:
    ${helpers.predefined_type("border-%s-color" % side, "CSSColor",
                              "::cssparser::Color::CurrentColor",
                              animatable=True)}
% endfor

% for side in ["top", "right", "bottom", "left"]:
    ${helpers.predefined_type("border-%s-style" % side, "BorderStyle",
                              "specified::BorderStyle::none",
                              need_clone=True, animatable=False)}
% endfor

% for side in ["top", "right", "bottom", "left"]:
    <%helpers:longhand name="border-${side}-width" animatable="True">
        use app_units::Au;
        use cssparser::ToCss;
        use std::fmt;
        use values::HasViewportPercentage;
        use values::specified::BorderWidth;

        pub type SpecifiedValue = BorderWidth;

        #[inline]
        pub fn parse(_context: &ParserContext, input: &mut Parser)
                     -> Result<SpecifiedValue, ()> {
            BorderWidth::parse(input)
        }

        pub mod computed_value {
            use app_units::Au;
            pub type T = Au;
        }
        #[inline] pub fn get_initial_value() -> computed_value::T {
            Au::from_px(3)  // medium
        }
    </%helpers:longhand>
% endfor

// FIXME(#4126): when gfx supports painting it, make this Size2D<LengthOrPercentage>
% for corner in ["top-left", "top-right", "bottom-right", "bottom-left"]:
    ${helpers.predefined_type("border-" + corner + "-radius", "BorderRadiusSize",
                              "computed::BorderRadiusSize::zero()",
                              "parse",
                              animatable=True)}
% endfor

${helpers.single_keyword("box-decoration-break", "slice clone",
                         gecko_enum_prefix="StyleBoxDecorationBreak",
                         products="gecko", animatable=False)}

${helpers.single_keyword("-moz-float-edge", "content-box margin-box",
                         gecko_ffi_name="mFloatEdge",
                         gecko_enum_prefix="StyleFloatEdge",
                         products="gecko",
                         animatable=False)}

// https://drafts.csswg.org/css-backgrounds-3/#border-image-source
<%helpers:longhand name="border-image-source" products="gecko" animatable="False">
    use cssparser::ToCss;
    use std::fmt;
    use values::LocalToCss;
    use values::NoViewportPercentage;
    use values::specified::Image;

    impl NoViewportPercentage for SpecifiedValue {}

    pub mod computed_value {
        use values::computed;
        #[derive(Debug, Clone, PartialEq)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub struct T(pub Option<computed::Image>);
    }

    #[derive(Debug, Clone, PartialEq)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub struct SpecifiedValue(pub Option<Image>);

    impl ToCss for computed_value::T {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match self.0 {
                Some(ref image) => image.to_css(dest),
                None => dest.write_str("none"),
            }
        }
    }
    impl ToCss for SpecifiedValue {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match self.0 {
                Some(ref image) => image.to_css(dest),
                None => dest.write_str("none"),
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
            match self.0 {
                Some(ref image) => computed_value::T(Some(image.to_computed_value(context))),
                None => computed_value::T(None),
            }
        }
        #[inline]
        fn from_computed_value(computed: &computed_value::T) -> Self {
            match computed.0 {
                Some(ref image) =>
                    SpecifiedValue(Some(ToComputedValue::from_computed_value(image))),
                None => SpecifiedValue(None),
            }
        }
    }

    pub fn parse(context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
        if input.try(|input| input.expect_ident_matching("none")).is_ok() {
            return Ok(SpecifiedValue(None));
        }

        Ok(SpecifiedValue(Some(try!(Image::parse(context, input)))))
    }
</%helpers:longhand>

// https://drafts.csswg.org/css-backgrounds-3/#border-image-outset
<%helpers:longhand name="border-image-outset" products="gecko" animatable="False">
    use cssparser::ToCss;
    use std::fmt;
    use values::HasViewportPercentage;
    use values::LocalToCss;
    use values::specified::LengthOrNumber;

    impl HasViewportPercentage for SpecifiedValue {
        fn has_viewport_percentage(&self) -> bool {
            let mut viewport_percentage = false;
            for value in self.0.iter() {
                let vp = value.has_viewport_percentage();
                viewport_percentage = vp || viewport_percentage;
            }
            viewport_percentage
        }
    }

    pub mod computed_value {
        use values::computed::LengthOrNumber;
        #[derive(Debug, Clone, PartialEq)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub struct T(pub LengthOrNumber, pub LengthOrNumber,
                     pub LengthOrNumber, pub LengthOrNumber);
    }

    #[derive(Debug, Clone, PartialEq)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub struct SpecifiedValue(pub Vec<LengthOrNumber>);

    impl ToCss for computed_value::T {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            try!(self.0.to_css(dest));
            try!(dest.write_str(" "));
            try!(self.1.to_css(dest));
            try!(dest.write_str(" "));
            try!(self.2.to_css(dest));
            try!(dest.write_str(" "));
            self.3.to_css(dest)
        }
    }
    impl ToCss for SpecifiedValue {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            try!(self.0[0].to_css(dest));
            for value in self.0.iter().skip(1) {
                try!(dest.write_str(" "));
                try!(value.to_css(dest));
            }
            Ok(())
        }
    }

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        computed_value::T(computed::LengthOrNumber::Number(0.0),
                          computed::LengthOrNumber::Number(0.0),
                          computed::LengthOrNumber::Number(0.0),
                          computed::LengthOrNumber::Number(0.0))
    }

    impl ToComputedValue for SpecifiedValue {
        type ComputedValue = computed_value::T;

        #[inline]
        fn to_computed_value(&self, context: &Context) -> computed_value::T {
            let length = self.0.len();
            match length {
                4 => computed_value::T(self.0[0].to_computed_value(context),
                                       self.0[1].to_computed_value(context),
                                       self.0[2].to_computed_value(context),
                                       self.0[3].to_computed_value(context)),
                3 => computed_value::T(self.0[0].to_computed_value(context),
                                       self.0[1].to_computed_value(context),
                                       self.0[2].to_computed_value(context),
                                       self.0[1].to_computed_value(context)),
                2 => computed_value::T(self.0[0].to_computed_value(context),
                                       self.0[1].to_computed_value(context),
                                       self.0[0].to_computed_value(context),
                                       self.0[1].to_computed_value(context)),
                1 => computed_value::T(self.0[0].to_computed_value(context),
                                       self.0[0].to_computed_value(context),
                                       self.0[0].to_computed_value(context),
                                       self.0[0].to_computed_value(context)),
                _ => unreachable!(),
            }
        }
        #[inline]
        fn from_computed_value(computed: &computed_value::T) -> Self {
            SpecifiedValue(vec![ToComputedValue::from_computed_value(&computed.0),
                                ToComputedValue::from_computed_value(&computed.1),
                                ToComputedValue::from_computed_value(&computed.2),
                                ToComputedValue::from_computed_value(&computed.3)])
        }
    }

    pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
        let mut values = vec![];
        for _ in 0..4 {
            let value = input.try(|input| LengthOrNumber::parse(input));
            match value {
                Ok(val) => values.push(val),
                Err(_) => break,
            }
        }

        if values.len() > 0 {
            Ok(SpecifiedValue(values))
        } else {
            Err(())
        }
    }
</%helpers:longhand>

// https://drafts.csswg.org/css-backgrounds-3/#border-image-repeat
<%helpers:longhand name="border-image-repeat" products="gecko" animatable="False">
    use cssparser::ToCss;
    use std::fmt;
    use values::LocalToCss;
    use values::NoViewportPercentage;

    impl NoViewportPercentage for SpecifiedValue {}

    pub mod computed_value {
        pub use super::RepeatKeyword;
        use values::computed;

        #[derive(Debug, Clone, PartialEq)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub struct T(pub RepeatKeyword, pub RepeatKeyword);
    }

    #[derive(Debug, Clone, PartialEq)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub struct SpecifiedValue(pub RepeatKeyword,
                              pub Option<RepeatKeyword>);

    define_css_keyword_enum!(RepeatKeyword:
                             "stretch" => Stretch,
                             "repeat" => Repeat,
                             "round" => Round,
                             "space" => Space);


    impl ToCss for computed_value::T {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            try!(self.0.to_css(dest));
            try!(dest.write_str(" "));
            self.0.to_css(dest)
        }
    }
    impl ToCss for SpecifiedValue {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            try!(self.0.to_css(dest));
            if self.1.is_some() {
                try!(dest.write_str(" "));
                try!(self.0.to_css(dest));
            }
            Ok(())
        }
    }

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        computed_value::T(RepeatKeyword::Stretch, RepeatKeyword::Stretch)
    }

    impl ToComputedValue for SpecifiedValue {
        type ComputedValue = computed_value::T;

        #[inline]
        fn to_computed_value(&self, _context: &Context) -> computed_value::T {
            computed_value::T(self.0, self.1.unwrap_or(self.0))
        }
        #[inline]
        fn from_computed_value(computed: &computed_value::T) -> Self {
            SpecifiedValue(computed.0, Some(computed.1))
        }
    }

    pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
        let first = try!(RepeatKeyword::parse(input));
        let second = input.try(RepeatKeyword::parse).ok();

        Ok(SpecifiedValue(first, second))
    }
</%helpers:longhand>

// https://drafts.csswg.org/css-backgrounds-3/#border-image-width
<%helpers:longhand name="border-image-width" products="gecko" animatable="False">
    use cssparser::ToCss;
    use std::fmt;
    use values::LocalToCss;
    use values::HasViewportPercentage;
    use values::specified::{LengthOrPercentage, Number};

    impl HasViewportPercentage for SpecifiedValue {
        fn has_viewport_percentage(&self) -> bool {
            let mut viewport_percentage = false;
            for value in self.0.clone() {
                let vp = match value {
                    SingleSpecifiedValue::LengthOrPercentage(len) => len.has_viewport_percentage(),
                    _ => false,
                };
                viewport_percentage = vp || viewport_percentage;
            }
            viewport_percentage
        }
    }

    pub mod computed_value {
        use values::computed::{LengthOrPercentage, Number};
        #[derive(Debug, Clone, PartialEq)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub struct T(pub SingleComputedValue, pub SingleComputedValue,
                     pub SingleComputedValue, pub SingleComputedValue);

        #[derive(Debug, Clone, PartialEq)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub enum SingleComputedValue {
            LengthOrPercentage(LengthOrPercentage),
            Number(Number),
            Auto,
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub struct SpecifiedValue(pub Vec<SingleSpecifiedValue>);

    impl ToCss for computed_value::T {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            try!(self.0.to_css(dest));
            try!(dest.write_str(" "));
            try!(self.1.to_css(dest));
            try!(dest.write_str(" "));
            try!(self.2.to_css(dest));
            try!(dest.write_str(" "));
            self.3.to_css(dest)
        }
    }
    impl ToCss for SpecifiedValue {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            try!(self.0[0].to_css(dest));
            for value in self.0.iter().skip(1) {
                try!(dest.write_str(" "));
                try!(value.to_css(dest));
            }
            Ok(())
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub enum SingleSpecifiedValue {
        LengthOrPercentage(LengthOrPercentage),
        Number(Number),
        Auto,
    }

    impl ToCss for computed_value::SingleComputedValue {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match *self {
                computed_value::SingleComputedValue::LengthOrPercentage(len) => len.to_css(dest),
                computed_value::SingleComputedValue::Number(number) => number.to_css(dest),
                computed_value::SingleComputedValue::Auto => dest.write_str("auto"),
            }
        }
    }
    impl ToCss for SingleSpecifiedValue {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match *self {
                SingleSpecifiedValue::LengthOrPercentage(len) => len.to_css(dest),
                SingleSpecifiedValue::Number(number) => number.to_css(dest),
                SingleSpecifiedValue::Auto => dest.write_str("auto"),
            }
        }
    }

    impl ToComputedValue for SingleSpecifiedValue {
        type ComputedValue = computed_value::SingleComputedValue;

        #[inline]
        fn to_computed_value(&self, context: &Context) -> computed_value::SingleComputedValue {
            match *self {
                SingleSpecifiedValue::LengthOrPercentage(len) => {
                    computed_value::SingleComputedValue::LengthOrPercentage(
                        len.to_computed_value(context))
                },
                SingleSpecifiedValue::Number(number) =>
                    computed_value::SingleComputedValue::Number(number.to_computed_value(context)),
                SingleSpecifiedValue::Auto => computed_value::SingleComputedValue::Auto,
            }
        }
        #[inline]
        fn from_computed_value(computed: &computed_value::SingleComputedValue) -> Self {
            match *computed {
                computed_value::SingleComputedValue::LengthOrPercentage(len) => {
                    SingleSpecifiedValue::LengthOrPercentage(
                        ToComputedValue::from_computed_value(&len))
                },
                computed_value::SingleComputedValue::Number(number) =>
                    SingleSpecifiedValue::Number(ToComputedValue::from_computed_value(&number)),
                computed_value::SingleComputedValue::Auto => SingleSpecifiedValue::Auto,
            }
        }
    }

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        computed_value::T(computed_value::SingleComputedValue::Number(1.0),
                          computed_value::SingleComputedValue::Number(1.0),
                          computed_value::SingleComputedValue::Number(1.0),
                          computed_value::SingleComputedValue::Number(1.0))
    }

    impl ToComputedValue for SpecifiedValue {
        type ComputedValue = computed_value::T;

        #[inline]
        fn to_computed_value(&self, context: &Context) -> computed_value::T {
            let length = self.0.len();
            match length {
                4 => computed_value::T(self.0[0].to_computed_value(context),
                                       self.0[1].to_computed_value(context),
                                       self.0[2].to_computed_value(context),
                                       self.0[3].to_computed_value(context)),
                3 => computed_value::T(self.0[0].to_computed_value(context),
                                       self.0[1].to_computed_value(context),
                                       self.0[2].to_computed_value(context),
                                       self.0[1].to_computed_value(context)),
                2 => computed_value::T(self.0[0].to_computed_value(context),
                                       self.0[1].to_computed_value(context),
                                       self.0[0].to_computed_value(context),
                                       self.0[1].to_computed_value(context)),
                1 => computed_value::T(self.0[0].to_computed_value(context),
                                       self.0[0].to_computed_value(context),
                                       self.0[0].to_computed_value(context),
                                       self.0[0].to_computed_value(context)),
                _ => unreachable!(),
            }
        }
        #[inline]
        fn from_computed_value(computed: &computed_value::T) -> Self {
            SpecifiedValue(vec![ToComputedValue::from_computed_value(&computed.0),
                                ToComputedValue::from_computed_value(&computed.1),
                                ToComputedValue::from_computed_value(&computed.2),
                                ToComputedValue::from_computed_value(&computed.3)])
        }
    }

    impl Parse for SingleSpecifiedValue {
        fn parse(input: &mut Parser) -> Result<Self, ()> {
            if input.try(|input| input.expect_ident_matching("auto")).is_ok() {
                return Ok(SingleSpecifiedValue::Auto);
            }

            if let Ok(len) = input.try(|input| LengthOrPercentage::parse(input)) {
                return Ok(SingleSpecifiedValue::LengthOrPercentage(len));
            }

            let num = try!(Number::parse_non_negative(input));
            Ok(SingleSpecifiedValue::Number(num))
        }
    }

    pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
        let mut values = vec![];
        for _ in 0..4 {
            let value = input.try(|input| SingleSpecifiedValue::parse(input));
            match value {
                Ok(val) => values.push(val),
                Err(_) => break,
            }
        }

        if values.len() > 0 {
            Ok(SpecifiedValue(values))
        } else {
            Err(())
        }
    }
</%helpers:longhand>

// https://drafts.csswg.org/css-backgrounds-3/#border-image-slice
<%helpers:longhand name="border-image-slice" products="gecko" animatable="False">
    use cssparser::ToCss;
    use std::fmt;
    use values::LocalToCss;
    use values::NoViewportPercentage;
    use values::specified::{Number, Percentage};

    impl NoViewportPercentage for SpecifiedValue {}

    pub mod computed_value {
        use values::computed::Number;
        use values::specified::Percentage;
        #[derive(Debug, Clone, PartialEq)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub struct T {
            pub corners: Vec<PercentageOrNumber>,
            pub fill: bool,
        }

        #[derive(Debug, Clone, PartialEq)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub enum PercentageOrNumber {
            Percentage(Percentage),
            Number(Number),
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub struct SpecifiedValue {
        pub corners: Vec<PercentageOrNumber>,
        pub fill: bool,
    }

    impl ToCss for computed_value::T {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            try!(self.corners[0].to_css(dest));
            try!(dest.write_str(" "));
            try!(self.corners[1].to_css(dest));
            try!(dest.write_str(" "));
            try!(self.corners[2].to_css(dest));
            try!(dest.write_str(" "));
            try!(self.corners[3].to_css(dest));

            if self.fill {
                try!(dest.write_str("fill"));
            }
            Ok(())
        }
    }
    impl ToCss for SpecifiedValue {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            try!(self.corners[0].to_css(dest));
            for value in self.corners.iter().skip(1) {
                try!(dest.write_str(" "));
                try!(value.to_css(dest));
            }

            if self.fill {
                try!(dest.write_str("fill"));
            }
            Ok(())
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub enum PercentageOrNumber {
        Percentage(Percentage),
        Number(Number),
    }

    impl ToCss for computed_value::PercentageOrNumber {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match *self {
                computed_value::PercentageOrNumber::Percentage(percentage) => percentage.to_css(dest),
                computed_value::PercentageOrNumber::Number(number) => number.to_css(dest),
            }
        }
    }
    impl ToCss for PercentageOrNumber {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match *self {
                PercentageOrNumber::Percentage(percentage) => percentage.to_css(dest),
                PercentageOrNumber::Number(number) => number.to_css(dest),
            }
        }
    }

    impl ToComputedValue for PercentageOrNumber {
        type ComputedValue = computed_value::PercentageOrNumber;

        #[inline]
        fn to_computed_value(&self, context: &Context) -> computed_value::PercentageOrNumber {
            match *self {
                PercentageOrNumber::Percentage(percentage) =>
                    computed_value::PercentageOrNumber::Percentage(percentage),
                PercentageOrNumber::Number(number) =>
                    computed_value::PercentageOrNumber::Number(number.to_computed_value(context)),
            }
        }
        #[inline]
        fn from_computed_value(computed: &computed_value::PercentageOrNumber) -> Self {
            match *computed {
                computed_value::PercentageOrNumber::Percentage(percentage) =>
                    PercentageOrNumber::Percentage(percentage),
                computed_value::PercentageOrNumber::Number(number) =>
                    PercentageOrNumber::Number(ToComputedValue::from_computed_value(&number)),
            }
        }
    }

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        computed_value::T {
            corners: vec![computed_value::PercentageOrNumber::Percentage(Percentage(1.0)),
                          computed_value::PercentageOrNumber::Percentage(Percentage(1.0)),
                          computed_value::PercentageOrNumber::Percentage(Percentage(1.0)),
                          computed_value::PercentageOrNumber::Percentage(Percentage(1.0))],
            fill: false,
        }
    }

    impl ToComputedValue for SpecifiedValue {
        type ComputedValue = computed_value::T;

        #[inline]
        fn to_computed_value(&self, context: &Context) -> computed_value::T {
            let length = self.corners.len();
            let corners = match length {
                4 => vec![self.corners[0].to_computed_value(context),
                          self.corners[1].to_computed_value(context),
                          self.corners[2].to_computed_value(context),
                          self.corners[3].to_computed_value(context)],
                3 => vec![self.corners[0].to_computed_value(context),
                          self.corners[1].to_computed_value(context),
                          self.corners[2].to_computed_value(context),
                          self.corners[1].to_computed_value(context)],
                2 => vec![self.corners[0].to_computed_value(context),
                          self.corners[1].to_computed_value(context),
                          self.corners[0].to_computed_value(context),
                          self.corners[1].to_computed_value(context)],
                1 => vec![self.corners[0].to_computed_value(context),
                          self.corners[0].to_computed_value(context),
                          self.corners[0].to_computed_value(context),
                          self.corners[0].to_computed_value(context)],
                _ => unreachable!(),
            };
            computed_value::T {
                corners: corners,
                fill: self.fill,
            }
        }
        #[inline]
        fn from_computed_value(computed: &computed_value::T) -> Self {
            SpecifiedValue {
                corners: vec![ToComputedValue::from_computed_value(&computed.corners[0]),
                              ToComputedValue::from_computed_value(&computed.corners[1]),
                              ToComputedValue::from_computed_value(&computed.corners[2]),
                              ToComputedValue::from_computed_value(&computed.corners[3])],
                fill: computed.fill,
            }
        }
    }

    impl Parse for PercentageOrNumber {
        fn parse(input: &mut Parser) -> Result<Self, ()> {
            if let Ok(per) = input.try(|input| Percentage::parse(input)) {
                return Ok(PercentageOrNumber::Percentage(per));
            }

            let num = try!(Number::parse_non_negative(input));
            Ok(PercentageOrNumber::Number(num))
        }
    }

    pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
        let mut fill = input.try(|input| input.expect_ident_matching("fill")).is_ok();

        let mut values = vec![];
        for _ in 0..4 {
            let value = input.try(|input| PercentageOrNumber::parse(input));
            match value {
                Ok(val) => values.push(val),
                Err(_) => break,
            }
        }

        if fill == false {
            fill = input.try(|input| input.expect_ident_matching("fill")).is_ok();
        }

        if values.len() > 0 {
            Ok(SpecifiedValue {
                corners: values,
                fill: fill
            })
        } else {
            Err(())
        }
    }
</%helpers:longhand>
