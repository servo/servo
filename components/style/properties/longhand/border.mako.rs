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

        impl ToCss for SpecifiedValue {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                self.0.to_css(dest)
            }
        }

        #[inline]
        pub fn parse(_context: &ParserContext, input: &mut Parser)
                               -> Result<SpecifiedValue, ()> {
            specified::parse_border_width(input).map(SpecifiedValue)
        }
        #[derive(Debug, Clone, PartialEq)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub struct SpecifiedValue(pub specified::Length);

        impl HasViewportPercentage for SpecifiedValue {
            fn has_viewport_percentage(&self) -> bool {
                let &SpecifiedValue(length) = self;
                length.has_viewport_percentage()
            }
        }

        pub mod computed_value {
            use app_units::Au;
            pub type T = Au;
        }
        #[inline] pub fn get_initial_value() -> computed_value::T {
            Au::from_px(3)  // medium
        }

        impl ToComputedValue for SpecifiedValue {
            type ComputedValue = computed_value::T;

            #[inline]
            fn to_computed_value(&self, context: &Context) -> computed_value::T {
                self.0.to_computed_value(context)
            }

            #[inline]
            fn from_computed_value(computed: &computed_value::T) -> Self {
                SpecifiedValue(ToComputedValue::from_computed_value(computed))
            }
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

<%helpers:longhand name="border-image-source" products="none" animatable="False">
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
        pub enum T {
            Image(computed::Image),
            None,
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub enum SpecifiedValue {
        Image(Image),
        None,
    }

    impl ToCss for computed_value::T {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match *self {
                computed_value::T::Image(ref image) => image.to_css(dest),
                computed_value::T::None => dest.write_str("none"),
            }
        }
    }
    impl ToCss for SpecifiedValue {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match *self {
                SpecifiedValue::Image(ref image) => image.to_css(dest),
                SpecifiedValue::None => dest.write_str("none"),
            }
        }
    }

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        computed_value::T::None
    }

    impl ToComputedValue for SpecifiedValue {
        type ComputedValue = computed_value::T;

        #[inline]
        fn to_computed_value(&self, context: &Context) -> computed_value::T {
            match *self {
                SpecifiedValue::Image(ref image) =>
                    computed_value::T::Image(image.to_computed_value(context)),
                SpecifiedValue::None => computed_value::T::None,
            }
        }
        #[inline]
        fn from_computed_value(computed: &computed_value::T) -> Self {
            match *computed {
                computed_value::T::Image(ref image) =>
                    SpecifiedValue::Image(ToComputedValue::from_computed_value(image)),
                computed_value::T::None => SpecifiedValue::None,
            }
        }
    }

    pub fn parse(context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
        if input.try(|input| input.expect_ident_matching("none")).is_ok() {
            return Ok(SpecifiedValue::None);
        }

        Ok(SpecifiedValue::Image(try!(Image::parse(context, input))))
    }
</%helpers:longhand>

<%helpers:longhand name="border-image-outset" products="none" animatable="False">
    use cssparser::ToCss;
    use std::fmt;
    use values::LocalToCss;
    use values::NoViewportPercentage;
    use values::specified::{Length, Number};

    impl NoViewportPercentage for SpecifiedValue {}

    pub mod computed_value {
        use values::computed::{Length, Number};
        #[derive(Debug, Clone, PartialEq)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub struct T(pub LengthOrNumber, pub LengthOrNumber,
                     pub LengthOrNumber, pub LengthOrNumber);

        #[derive(Debug, Clone, PartialEq)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub enum LengthOrNumber {
            Length(Length),
            Number(Number),
        }
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
            let length = self.0.len();

            try!(self.0[0].to_css(dest));
            if length > 1 {
                try!(dest.write_str(" "));
                try!(self.0[1].to_css(dest));
                if length > 2 {
                    try!(dest.write_str(" "));
                    try!(self.0[2].to_css(dest));
                    if length > 3 {
                        try!(dest.write_str(" "));
                        try!(self.0[3].to_css(dest));
                    }
                }
            }
            Ok(())
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub enum LengthOrNumber {
        Length(Length),
        Number(Number),
    }

    impl ToCss for computed_value::LengthOrNumber {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match *self {
                computed_value::LengthOrNumber::Length(len) => len.to_css(dest),
                computed_value::LengthOrNumber::Number(number) => number.to_css(dest),
            }
        }
    }
    impl ToCss for LengthOrNumber {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match *self {
                LengthOrNumber::Length(len) => len.to_css(dest),
                LengthOrNumber::Number(number) => number.to_css(dest),
            }
        }
    }

    impl ToComputedValue for LengthOrNumber {
        type ComputedValue = computed_value::LengthOrNumber;

        #[inline]
        fn to_computed_value(&self, context: &Context) -> computed_value::LengthOrNumber {
            match *self {
                LengthOrNumber::Length(len) =>
                    computed_value::LengthOrNumber::Length(len.to_computed_value(context)),
                LengthOrNumber::Number(number) =>
                    computed_value::LengthOrNumber::Number(number.to_computed_value(context)),
            }
        }
        #[inline]
        fn from_computed_value(computed: &computed_value::LengthOrNumber) -> Self {
            match *computed {
                computed_value::LengthOrNumber::Length(len) =>
                    LengthOrNumber::Length(ToComputedValue::from_computed_value(&len)),
                computed_value::LengthOrNumber::Number(number) =>
                    LengthOrNumber::Number(ToComputedValue::from_computed_value(&number)),
            }
        }
    }

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        computed_value::T(computed_value::LengthOrNumber::Number(0.0),
                          computed_value::LengthOrNumber::Number(0.0),
                          computed_value::LengthOrNumber::Number(0.0),
                          computed_value::LengthOrNumber::Number(0.0))
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

    impl LengthOrNumber {
        pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<LengthOrNumber, ()> {
            let length = input.try(|input| Length::parse(input));
            if let Ok(len) = length {
                return Ok(LengthOrNumber::Length(len));
            }

            let num = try!(Number::parse_non_negative(input));
            Ok(LengthOrNumber::Number(num))
        }
    }

    pub fn parse(context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
        let mut values = vec![];
        for _ in 0..4 {
            let value = input.try(|input| LengthOrNumber::parse(context, input));
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

<%helpers:longhand name="border-image-repeat" products="none" animatable="False">
    use cssparser::ToCss;
    use std::fmt;
    use values::LocalToCss;
    use values::NoViewportPercentage;

    impl NoViewportPercentage for SpecifiedValue {}

    pub mod computed_value {
        use super::SingleSpecifiedValue;
        use values::computed;

        #[derive(Debug, Clone, PartialEq)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub struct T(pub SingleSpecifiedValue, pub SingleSpecifiedValue);
    }

    #[derive(Debug, Clone, PartialEq)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub struct SpecifiedValue(pub SingleSpecifiedValue,
                              pub Option<SingleSpecifiedValue>);

    define_css_keyword_enum!(SingleSpecifiedValue:
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
        computed_value::T(SingleSpecifiedValue::Stretch, SingleSpecifiedValue::Stretch)
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
        let first = try!(SingleSpecifiedValue::parse(input));
        let second = input.try(SingleSpecifiedValue::parse).ok();

        Ok(SpecifiedValue(first, second))
    }
</%helpers:longhand>

<%helpers:longhand name="border-image-width" products="none" animatable="False">
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
            let length = self.0.len();

            try!(self.0[0].to_css(dest));
            if length > 1 {
                try!(dest.write_str(" "));
                try!(self.0[1].to_css(dest));
                if length > 2 {
                    try!(dest.write_str(" "));
                    try!(self.0[2].to_css(dest));
                    if length > 3 {
                        try!(dest.write_str(" "));
                        try!(self.0[3].to_css(dest));
                    }
                }
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
                SingleSpecifiedValue::LengthOrPercentage(len) =>
                    computed_value::SingleComputedValue::LengthOrPercentage(len.to_computed_value(context)),
                SingleSpecifiedValue::Number(number) =>
                    computed_value::SingleComputedValue::Number(number.to_computed_value(context)),
                SingleSpecifiedValue::Auto => computed_value::SingleComputedValue::Auto,
            }
        }
        #[inline]
        fn from_computed_value(computed: &computed_value::SingleComputedValue) -> Self {
            match *computed {
                computed_value::SingleComputedValue::LengthOrPercentage(len) =>
                    SingleSpecifiedValue::LengthOrPercentage(ToComputedValue::from_computed_value(&len)),
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

    impl SingleSpecifiedValue {
        pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SingleSpecifiedValue, ()> {
            if input.try(|input| input.expect_ident_matching("auto")).is_ok() {
                return Ok(SingleSpecifiedValue::Auto);
            }

            let length = input.try(|input| LengthOrPercentage::parse(input));
            if let Ok(len) = length {
                return Ok(SingleSpecifiedValue::LengthOrPercentage(len));
            }

            let num = try!(Number::parse_non_negative(input));
            Ok(SingleSpecifiedValue::Number(num))
        }
    }

    pub fn parse(context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
        let mut values = vec![];
        for _ in 0..4 {
            let value = input.try(|input| SingleSpecifiedValue::parse(context, input));
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

<%helpers:longhand name="border-image-slice" products="none" animatable="False">
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
            pub corners: Vec<SingleComputedValue>,
            pub fill: bool,
        }

        #[derive(Debug, Clone, PartialEq)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub enum SingleComputedValue {
            Percentage(Percentage),
            Number(Number),
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub struct SpecifiedValue {
        pub corners: Vec<SingleSpecifiedValue>,
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
            let length = self.corners.len();

            try!(self.corners[0].to_css(dest));
            if length > 1 {
                try!(dest.write_str(" "));
                try!(self.corners[1].to_css(dest));
                if length > 2 {
                    try!(dest.write_str(" "));
                    try!(self.corners[2].to_css(dest));
                    if length > 3 {
                        try!(dest.write_str(" "));
                        try!(self.corners[3].to_css(dest));
                    }
                }
            }

            if self.fill {
                try!(dest.write_str("fill"));
            }
            Ok(())
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub enum SingleSpecifiedValue {
        Percentage(Percentage),
        Number(Number),
    }

    impl ToCss for computed_value::SingleComputedValue {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match *self {
                computed_value::SingleComputedValue::Percentage(percentage) => percentage.to_css(dest),
                computed_value::SingleComputedValue::Number(number) => number.to_css(dest),
            }
        }
    }
    impl ToCss for SingleSpecifiedValue {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match *self {
                SingleSpecifiedValue::Percentage(percentage) => percentage.to_css(dest),
                SingleSpecifiedValue::Number(number) => number.to_css(dest),
            }
        }
    }

    impl ToComputedValue for SingleSpecifiedValue {
        type ComputedValue = computed_value::SingleComputedValue;

        #[inline]
        fn to_computed_value(&self, context: &Context) -> computed_value::SingleComputedValue {
            match *self {
                SingleSpecifiedValue::Percentage(percentage) =>
                    computed_value::SingleComputedValue::Percentage(percentage),
                SingleSpecifiedValue::Number(number) =>
                    computed_value::SingleComputedValue::Number(number.to_computed_value(context)),
            }
        }
        #[inline]
        fn from_computed_value(computed: &computed_value::SingleComputedValue) -> Self {
            match *computed {
                computed_value::SingleComputedValue::Percentage(percentage) =>
                    SingleSpecifiedValue::Percentage(percentage),
                computed_value::SingleComputedValue::Number(number) =>
                    SingleSpecifiedValue::Number(ToComputedValue::from_computed_value(&number)),
            }
        }
    }

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        computed_value::T {
            corners: vec![computed_value::SingleComputedValue::Percentage(Percentage(1.0)),
                          computed_value::SingleComputedValue::Percentage(Percentage(1.0)),
                          computed_value::SingleComputedValue::Percentage(Percentage(1.0)),
                          computed_value::SingleComputedValue::Percentage(Percentage(1.0))],
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

    impl SingleSpecifiedValue {
        pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SingleSpecifiedValue, ()> {
            if let Ok(per) = input.try(|input| Percentage::parse(input)) {
                return Ok(SingleSpecifiedValue::Percentage(per));
            }

            let num = try!(Number::parse_non_negative(input));
            Ok(SingleSpecifiedValue::Number(num))
        }
    }

    pub fn parse(context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
        let mut fill = input.try(|input| input.expect_ident_matching("fill")).is_ok();

        let mut values = vec![];
        for _ in 0..4 {
            let value = input.try(|input| SingleSpecifiedValue::parse(context, input));
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
