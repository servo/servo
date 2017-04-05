/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />
<% from data import Keyword, Method, PHYSICAL_SIDES, ALL_SIDES, maybe_moz_logical_alias %>

<% data.new_style_struct("Border", inherited=False,
                   additional_methods=[Method("border_" + side + "_has_nonzero_width",
                                              "bool") for side in ["top", "right", "bottom", "left"]]) %>
<%
    def maybe_logical_spec(side, kind):
        if side[1]: # if it is logical
            return "https://drafts.csswg.org/css-logical-props/#propdef-border-%s-%s" % (side[0], kind)
        else:
            return "https://drafts.csswg.org/css-backgrounds/#border-%s-%s" % (side[0], kind)
%>
% for side in ALL_SIDES:
    ${helpers.predefined_type("border-%s-color" % side[0], "CSSColor",
                              "::cssparser::Color::CurrentColor",
                              alias=maybe_moz_logical_alias(product, side, "-moz-border-%s-color"),
                              spec=maybe_logical_spec(side, "color"),
                              animatable=True, logical = side[1])}
% endfor

% for side in ALL_SIDES:
    ${helpers.predefined_type("border-%s-style" % side[0], "BorderStyle",
                              "specified::BorderStyle::none",
                              need_clone=True,
                              alias=maybe_moz_logical_alias(product, side, "-moz-border-%s-style"),
                              spec=maybe_logical_spec(side, "style"),
                              animatable=False, logical = side[1])}
% endfor

${helpers.gecko_keyword_conversion(Keyword('border-style',
                                   "none solid double dotted dashed hidden groove ridge inset outset"),
                                   type="::values::specified::BorderStyle")}
% for side in ALL_SIDES:
    <%helpers:longhand name="border-${side[0]}-width" boxed="True" animatable="True" logical="${side[1]}"
                       alias="${maybe_moz_logical_alias(product, side, '-moz-border-%s-width')}"
                       spec="${maybe_logical_spec(side, 'width')}">
        use app_units::Au;
        use std::fmt;
        use style_traits::ToCss;
        use values::HasViewportPercentage;
        use values::specified::BorderWidth;

        pub type SpecifiedValue = BorderWidth;

        #[inline]
        pub fn parse(context: &ParserContext, input: &mut Parser)
                     -> Result<SpecifiedValue, ()> {
            BorderWidth::parse(context, input)
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
                              "parse", extra_prefixes="webkit",
                              spec="https://drafts.csswg.org/css-backgrounds/#border-%s-radius" % corner,
                              boxed=True,
                              animatable=True)}
% endfor

${helpers.single_keyword("box-decoration-break", "slice clone",
                         gecko_enum_prefix="StyleBoxDecorationBreak",
                         gecko_inexhaustive=True,
                         spec="https://drafts.csswg.org/css-break/#propdef-box-decoration-break",
                         products="gecko", animatable=False)}

${helpers.single_keyword("-moz-float-edge", "content-box margin-box",
                         gecko_ffi_name="mFloatEdge",
                         gecko_enum_prefix="StyleFloatEdge",
                         gecko_inexhaustive=True,
                         products="gecko",
                         spec="Nonstandard (https://developer.mozilla.org/en-US/docs/Web/CSS/-moz-float-edge)",
                         animatable=False)}

<%helpers:longhand name="border-image-source" animatable="False" boxed="True"
                   spec="https://drafts.csswg.org/css-backgrounds/#border-image-source">
    use std::fmt;
    use style_traits::ToCss;
    use values::HasViewportPercentage;
    use values::specified::Image;

    no_viewport_percentage!(SpecifiedValue);

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

    #[inline]
    pub fn get_initial_specified_value() -> SpecifiedValue {
        SpecifiedValue(None)
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

<%helpers:longhand name="border-image-outset" animatable="False"
                   spec="https://drafts.csswg.org/css-backgrounds/#border-image-outset">
    use std::fmt;
    use style_traits::ToCss;
    use values::HasViewportPercentage;
    use values::specified::{LengthOrNumber, Number};

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
        computed_value::T(Either::Second(0.0), Either::Second(0.0),
                          Either::Second(0.0), Either::Second(0.0))
    }

    #[inline]
    pub fn get_initial_specified_value() -> SpecifiedValue {
        SpecifiedValue(vec![Either::Second(Number::new(0.0))])
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

    pub fn parse(context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
        let mut values = vec![];
        for _ in 0..4 {
            let value = input.try(|input| LengthOrNumber::parse_non_negative(context, input));
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

<%helpers:longhand name="border-image-repeat" animatable="False"
                   spec="https://drafts.csswg.org/css-backgrounds/#border-image-repeat">
    use std::fmt;
    use style_traits::ToCss;
    use values::HasViewportPercentage;

    no_viewport_percentage!(SpecifiedValue);

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
            self.1.to_css(dest)
        }
    }
    impl ToCss for SpecifiedValue {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            try!(self.0.to_css(dest));
            if let Some(second) = self.1 {
                try!(dest.write_str(" "));
                try!(second.to_css(dest));
            }
            Ok(())
        }
    }

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        computed_value::T(RepeatKeyword::Stretch, RepeatKeyword::Stretch)
    }

    #[inline]
    pub fn get_initial_specified_value() -> SpecifiedValue {
        SpecifiedValue(RepeatKeyword::Stretch, None)
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

<%helpers:longhand name="border-image-width" animatable="False"
                   spec="https://drafts.csswg.org/css-backgrounds/#border-image-width">
    use std::fmt;
    use style_traits::ToCss;
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
                computed_value::SingleComputedValue::LengthOrPercentage(ref len) => len.to_css(dest),
                computed_value::SingleComputedValue::Number(number) => number.to_css(dest),
                computed_value::SingleComputedValue::Auto => dest.write_str("auto"),
            }
        }
    }
    impl ToCss for SingleSpecifiedValue {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match *self {
                SingleSpecifiedValue::LengthOrPercentage(ref len) => len.to_css(dest),
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
                SingleSpecifiedValue::LengthOrPercentage(ref len) => {
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

    #[inline]
    pub fn get_initial_specified_value() -> SpecifiedValue {
        SpecifiedValue(vec![SingleSpecifiedValue::Number(Number::new(1.0))])
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
        fn parse(_context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
            if input.try(|input| input.expect_ident_matching("auto")).is_ok() {
                return Ok(SingleSpecifiedValue::Auto);
            }

            if let Ok(len) = input.try(|input| LengthOrPercentage::parse_non_negative(input)) {
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

<%helpers:longhand name="border-image-slice" boxed="True" animatable="False"
                   spec="https://drafts.csswg.org/css-backgrounds/#border-image-slice">
    use std::fmt;
    use style_traits::ToCss;
    use values::HasViewportPercentage;
    use values::computed::NumberOrPercentage as ComputedNumberOrPercentage;
    use values::specified::{NumberOrPercentage, Percentage};

    no_viewport_percentage!(SpecifiedValue);

    pub mod computed_value {
        use values::computed::NumberOrPercentage;
        #[derive(Debug, Clone, PartialEq)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub struct T {
            pub corners: Vec<NumberOrPercentage>,
            pub fill: bool,
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub struct SpecifiedValue {
        pub corners: Vec<NumberOrPercentage>,
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
                try!(dest.write_str(" fill"));
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
                try!(dest.write_str(" fill"));
            }
            Ok(())
        }
    }

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        computed_value::T {
            corners: vec![ComputedNumberOrPercentage::Percentage(Percentage(1.0)),
                          ComputedNumberOrPercentage::Percentage(Percentage(1.0)),
                          ComputedNumberOrPercentage::Percentage(Percentage(1.0)),
                          ComputedNumberOrPercentage::Percentage(Percentage(1.0))],
            fill: false,
        }
    }

    #[inline]
    pub fn get_initial_specified_value() -> SpecifiedValue {
        SpecifiedValue {
            corners: vec![NumberOrPercentage::Percentage(Percentage(1.0))],
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

    pub fn parse(context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
        let mut fill = input.try(|input| input.expect_ident_matching("fill")).is_ok();

        let mut values = vec![];
        for _ in 0..4 {
            let value = input.try(|input| NumberOrPercentage::parse(context, input));
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
