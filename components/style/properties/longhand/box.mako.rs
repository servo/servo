/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />
<% from data import Keyword, Method, to_rust_ident %>

<% data.new_style_struct("Box",
                         inherited=False,
                         gecko_name="Display") %>

// TODO(SimonSapin): don't parse `inline-table`, since we don't support it
<%helpers:longhand name="display"
                   need_clone="True"
                   animatable="False"
                   custom_cascade="${product == 'servo'}">
    <%
        values = """inline block inline-block
            table inline-table table-row-group table-header-group table-footer-group
            table-row table-column-group table-column table-cell table-caption
            list-item flex
            none
        """.split()
        if product == "gecko":
            values += "-moz-box -moz-inline-box".split()
    %>
    pub use self::computed_value::T as SpecifiedValue;
    use values::computed::ComputedValueAsSpecified;
    use style_traits::ToCss;
    use values::NoViewportPercentage;
    impl NoViewportPercentage for SpecifiedValue {}

    pub mod computed_value {
        use style_traits::ToCss;
        #[allow(non_camel_case_types)]
        #[derive(Clone, Eq, PartialEq, Copy, Hash, RustcEncodable, Debug)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf, Deserialize, Serialize))]
        pub enum T {
            % for value in values:
                ${to_rust_ident(value)},
            % endfor
        }

        impl ToCss for T {
            fn to_css<W>(&self, dest: &mut W) -> ::std::fmt::Result
            where W: ::std::fmt::Write {
                match *self {
                    % for value in values:
                        T::${to_rust_ident(value)} => dest.write_str("${value}"),
                    % endfor
                }
            }
        }
    }
    #[inline] pub fn get_initial_value() -> computed_value::T {
        computed_value::T::${to_rust_ident(values[0])}
    }
    pub fn parse(_context: &ParserContext, input: &mut Parser)
                 -> Result<SpecifiedValue, ()> {
        match_ignore_ascii_case! { try!(input.expect_ident()),
            % for value in values:
                "${value}" => {
                    Ok(computed_value::T::${to_rust_ident(value)})
                },
            % endfor
            _ => Err(())
        }
    }

    impl ComputedValueAsSpecified for SpecifiedValue {}

    % if product == "servo":
        fn cascade_property_custom(_declaration: &PropertyDeclaration,
                                   _inherited_style: &ComputedValues,
                                   context: &mut computed::Context,
                                   _seen: &mut PropertyBitField,
                                   _cacheable: &mut bool,
                                   _error_reporter: &mut StdBox<ParseErrorReporter + Send>) {
            longhands::_servo_display_for_hypothetical_box::derive_from_display(context);
            longhands::_servo_text_decorations_in_effect::derive_from_display(context);
            longhands::_servo_under_display_none::derive_from_display(context);
        }
    % endif

</%helpers:longhand>

${helpers.single_keyword("position", "static absolute relative fixed",
                         need_clone=True, extra_gecko_values="sticky", animatable=False)}

<%helpers:single_keyword_computed name="float"
                                  values="none left right"
                                  animatable="False"
                                  need_clone="True"
                                  gecko_enum_prefix="StyleFloat"
                                  gecko_ffi_name="mFloat">
    use values::NoViewportPercentage;
    impl NoViewportPercentage for SpecifiedValue {}
    impl ToComputedValue for SpecifiedValue {
        type ComputedValue = computed_value::T;

        #[inline]
        fn to_computed_value(&self, context: &Context) -> computed_value::T {
            let positioned = matches!(context.style().get_box().clone_position(),
                longhands::position::SpecifiedValue::absolute |
                longhands::position::SpecifiedValue::fixed);
            if positioned {
                SpecifiedValue::none
            } else {
                *self
            }
        }
        #[inline]
        fn from_computed_value(computed: &computed_value::T) -> SpecifiedValue {
          *computed
        }
    }

</%helpers:single_keyword_computed>

${helpers.single_keyword("clear", "none left right both",
                         animatable=False, gecko_ffi_name="mBreakType",
                         gecko_enum_prefix="StyleClear")}

<%helpers:longhand name="-servo-display-for-hypothetical-box"
                   animatable="False"
                   derived_from="display"
                   products="servo">
    pub use super::display::{SpecifiedValue, get_initial_value};
    pub use super::display::{parse};

    pub mod computed_value {
        pub type T = super::SpecifiedValue;
    }

    #[inline]
    pub fn derive_from_display(context: &mut Context) {
        let d = context.style().get_box().clone_display();
        context.mutate_style().mutate_box().set__servo_display_for_hypothetical_box(d);
    }

</%helpers:longhand>

<%helpers:longhand name="vertical-align"
                   animatable="True">
  use std::fmt;
  use style_traits::ToCss;
  use values::HasViewportPercentage;

  <% vertical_align = data.longhands_by_name["vertical-align"] %>
  <% vertical_align.keyword = Keyword("vertical-align",
                                      "baseline sub super top text-top middle bottom text-bottom",
                                      extra_gecko_values="middle-with-baseline") %>
  <% vertical_align_keywords = vertical_align.keyword.values_for(product) %>

  impl HasViewportPercentage for SpecifiedValue {
      fn has_viewport_percentage(&self) -> bool {
          match *self {
              SpecifiedValue::LengthOrPercentage(length) => length.has_viewport_percentage(),
              _ => false
          }
      }
  }

  #[allow(non_camel_case_types)]
  #[derive(Debug, Clone, PartialEq, Copy)]
  #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
  pub enum SpecifiedValue {
      % for keyword in vertical_align_keywords:
          ${to_rust_ident(keyword)},
      % endfor
      LengthOrPercentage(specified::LengthOrPercentage),
  }

  impl ToCss for SpecifiedValue {
      fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
          match *self {
              % for keyword in vertical_align_keywords:
                  SpecifiedValue::${to_rust_ident(keyword)} => dest.write_str("${keyword}"),
              % endfor
              SpecifiedValue::LengthOrPercentage(value) => value.to_css(dest),
          }
      }
  }
  /// baseline | sub | super | top | text-top | middle | bottom | text-bottom
  /// | <percentage> | <length>
  pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
      input.try(specified::LengthOrPercentage::parse)
      .map(SpecifiedValue::LengthOrPercentage)
      .or_else(|()| {
          match_ignore_ascii_case! { try!(input.expect_ident()),
              % for keyword in vertical_align_keywords:
                  "${keyword}" => Ok(SpecifiedValue::${to_rust_ident(keyword)}),
              % endfor
              _ => Err(())
          }
      })
  }
  pub mod computed_value {
      use app_units::Au;
      use std::fmt;
      use style_traits::ToCss;
      use values::{CSSFloat, computed};
      #[allow(non_camel_case_types)]
      #[derive(PartialEq, Copy, Clone, Debug)]
      #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
      pub enum T {
          % for keyword in vertical_align_keywords:
              ${to_rust_ident(keyword)},
          % endfor
          LengthOrPercentage(computed::LengthOrPercentage),
      }
      impl ToCss for T {
          fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
              match *self {
                  % for keyword in vertical_align_keywords:
                      T::${to_rust_ident(keyword)} => dest.write_str("${keyword}"),
                  % endfor
                  T::LengthOrPercentage(value) => value.to_css(dest),
              }
          }
      }
  }
  #[inline]
  pub fn get_initial_value() -> computed_value::T { computed_value::T::baseline }

  impl ToComputedValue for SpecifiedValue {
      type ComputedValue = computed_value::T;

      #[inline]
      fn to_computed_value(&self, context: &Context) -> computed_value::T {
          match *self {
              % for keyword in vertical_align_keywords:
                  SpecifiedValue::${to_rust_ident(keyword)} => {
                      computed_value::T::${to_rust_ident(keyword)}
                  }
              % endfor
              SpecifiedValue::LengthOrPercentage(value) =>
                  computed_value::T::LengthOrPercentage(value.to_computed_value(context)),
          }
      }
      #[inline]
      fn from_computed_value(computed: &computed_value::T) -> Self {
          match *computed {
              % for keyword in vertical_align_keywords:
                  computed_value::T::${to_rust_ident(keyword)} => {
                      SpecifiedValue::${to_rust_ident(keyword)}
                  }
              % endfor
              computed_value::T::LengthOrPercentage(value) =>
                  SpecifiedValue::LengthOrPercentage(
                    ToComputedValue::from_computed_value(&value)
                  ),
          }
      }
  }
</%helpers:longhand>


// CSS 2.1, Section 11 - Visual effects

// Non-standard, see https://developer.mozilla.org/en-US/docs/Web/CSS/overflow-clip-box#Specifications
${helpers.single_keyword("-servo-overflow-clip-box", "padding-box content-box",
                         products="servo", animatable=False, internal=True)}

${helpers.single_keyword("overflow-clip-box", "padding-box content-box",
                         products="gecko", animatable=False, internal=True)}

// FIXME(pcwalton, #2742): Implement scrolling for `scroll` and `auto`.
${helpers.single_keyword("overflow-x", "visible hidden scroll auto",
                         need_clone=True, animatable=False,
                         gecko_constant_prefix="NS_STYLE_OVERFLOW")}

// FIXME(pcwalton, #2742): Implement scrolling for `scroll` and `auto`.
<%helpers:longhand name="overflow-y"
                   need_clone="True"
                   animatable="False">
  use super::overflow_x;

  use std::fmt;
  use style_traits::ToCss;
  use values::computed::ComputedValueAsSpecified;
  use values::NoViewportPercentage;

  pub use self::computed_value::T as SpecifiedValue;

  impl NoViewportPercentage for SpecifiedValue {}

  impl ToCss for SpecifiedValue {
      fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
          self.0.to_css(dest)
      }
  }

  pub mod computed_value {
      #[derive(Debug, Clone, Copy, PartialEq)]
      #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
      pub struct T(pub super::super::overflow_x::computed_value::T);
  }

  impl ComputedValueAsSpecified for SpecifiedValue {}

  pub fn get_initial_value() -> computed_value::T {
      computed_value::T(overflow_x::get_initial_value())
  }

  pub fn parse(context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue,()> {
      overflow_x::parse(context, input).map(SpecifiedValue)
  }
</%helpers:longhand>

// TODO(pcwalton): Multiple transitions.
<%helpers:longhand name="transition-duration"
                   need_index="True"
                   animatable="False">
    use values::computed::ComputedValueAsSpecified;
    use values::specified::Time;

    pub use self::computed_value::T as SpecifiedValue;
    pub use values::specified::Time as SingleSpecifiedValue;
    use values::NoViewportPercentage;
    impl NoViewportPercentage for SpecifiedValue {}

    pub mod computed_value {
        use std::fmt;
        use style_traits::ToCss;
        use values::computed::{Context, ToComputedValue};

        pub use values::computed::Time as SingleComputedValue;

        #[derive(Debug, Clone, PartialEq)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub struct T(pub Vec<SingleComputedValue>);

        impl ToCss for T {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                if self.0.is_empty() {
                    return dest.write_str("none")
                }
                for (i, value) in self.0.iter().enumerate() {
                    if i != 0 {
                        try!(dest.write_str(", "))
                    }
                    try!(value.to_css(dest))
                }
                Ok(())
            }
        }
    }

    impl ComputedValueAsSpecified for SpecifiedValue {}

    #[inline]
    pub fn parse_one(input: &mut Parser) -> Result<SingleSpecifiedValue,()> {
        Time::parse(input)
    }

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        computed_value::T(vec![get_initial_single_value()])
    }

    #[inline]
    pub fn get_initial_single_value() -> Time {
        Time(0.0)
    }

    pub fn parse(_: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue,()> {
        Ok(SpecifiedValue(try!(input.parse_comma_separated(parse_one))))
    }
</%helpers:longhand>

// TODO(pcwalton): Lots more timing functions.
// TODO(pcwalton): Multiple transitions.
<%helpers:longhand name="transition-timing-function"
                   need_index="True"
                   animatable="False">
    use self::computed_value::{StartEnd, TransitionTimingFunction};

    use euclid::point::{Point2D, TypedPoint2D};
    use std::marker::PhantomData;
    use values::computed::ComputedValueAsSpecified;

    pub use self::computed_value::SingleComputedValue as SingleSpecifiedValue;
    pub use self::computed_value::T as SpecifiedValue;

    // FIXME: This could use static variables and const functions when they are available.
    #[inline(always)]
    fn ease() -> TransitionTimingFunction {
        TransitionTimingFunction::CubicBezier(TypedPoint2D::new(0.25, 0.1),
                                              TypedPoint2D::new(0.25, 1.0))
    }

    #[inline(always)]
    fn linear() -> TransitionTimingFunction {
        TransitionTimingFunction::CubicBezier(TypedPoint2D::new(0.0, 0.0),
                                              TypedPoint2D::new(1.0, 1.0))
    }

    #[inline(always)]
    fn ease_in() -> TransitionTimingFunction {
        TransitionTimingFunction::CubicBezier(TypedPoint2D::new(0.42, 0.0),
                                              TypedPoint2D::new(1.0, 1.0))
    }

    #[inline(always)]
    fn ease_out() -> TransitionTimingFunction {
        TransitionTimingFunction::CubicBezier(TypedPoint2D::new(0.0, 0.0),
                                              TypedPoint2D::new(0.58, 1.0))
    }

    #[inline(always)]
    fn ease_in_out() -> TransitionTimingFunction {
        TransitionTimingFunction::CubicBezier(TypedPoint2D::new(0.42, 0.0),
                                              TypedPoint2D::new(0.58, 1.0))
    }

    static STEP_START: TransitionTimingFunction =
        TransitionTimingFunction::Steps(1, StartEnd::Start);
    static STEP_END: TransitionTimingFunction =
        TransitionTimingFunction::Steps(1, StartEnd::End);

    pub mod computed_value {
        use euclid::point::Point2D;
        use std::fmt;
        use style_traits::ToCss;
        use values::computed::ComputedValueAsSpecified;

        pub use self::TransitionTimingFunction as SingleComputedValue;

        #[derive(Copy, Clone, Debug, PartialEq)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub enum TransitionTimingFunction {
            CubicBezier(Point2D<f32>, Point2D<f32>),
            Steps(u32, StartEnd),
        }

        impl ToCss for TransitionTimingFunction {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                match *self {
                    TransitionTimingFunction::CubicBezier(p1, p2) => {
                        try!(dest.write_str("cubic-bezier("));
                        try!(p1.x.to_css(dest));
                        try!(dest.write_str(", "));
                        try!(p1.y.to_css(dest));
                        try!(dest.write_str(", "));
                        try!(p2.x.to_css(dest));
                        try!(dest.write_str(", "));
                        try!(p2.y.to_css(dest));
                        dest.write_str(")")
                    }
                    TransitionTimingFunction::Steps(steps, start_end) => {
                        try!(dest.write_str("steps("));
                        try!(steps.to_css(dest));
                        try!(dest.write_str(", "));
                        try!(start_end.to_css(dest));
                        dest.write_str(")")
                    }
                }
            }
        }

        #[derive(Copy, Clone, Debug, PartialEq)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub enum StartEnd {
            Start,
            End,
        }

        impl ToCss for StartEnd {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                match *self {
                    StartEnd::Start => dest.write_str("start"),
                    StartEnd::End => dest.write_str("end"),
                }
            }
        }

        #[derive(Clone, Debug, PartialEq)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub struct T(pub Vec<TransitionTimingFunction>);

        impl ToCss for T {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                if self.0.is_empty() {
                    return dest.write_str("none")
                }
                for (i, value) in self.0.iter().enumerate() {
                    if i != 0 {
                        try!(dest.write_str(", "))
                    }
                    try!(value.to_css(dest))
                }
                Ok(())
            }
        }
    }

    use values::NoViewportPercentage;
    impl NoViewportPercentage for SpecifiedValue {}

    impl ComputedValueAsSpecified for SpecifiedValue {}

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        computed_value::T(vec![get_initial_single_value()])
    }

    #[inline]
    pub fn get_initial_single_value() -> TransitionTimingFunction {
        ease()
    }

    pub fn parse_one(input: &mut Parser) -> Result<SingleSpecifiedValue,()> {
        if let Ok(function_name) = input.try(|input| input.expect_function()) {
            return match_ignore_ascii_case! { function_name,
                "cubic-bezier" => {
                    let (mut p1x, mut p1y, mut p2x, mut p2y) = (0.0, 0.0, 0.0, 0.0);
                    try!(input.parse_nested_block(|input| {
                        p1x = try!(specified::parse_number(input));
                        try!(input.expect_comma());
                        p1y = try!(specified::parse_number(input));
                        try!(input.expect_comma());
                        p2x = try!(specified::parse_number(input));
                        try!(input.expect_comma());
                        p2y = try!(specified::parse_number(input));
                        Ok(())
                    }));
                    let (p1, p2) = (Point2D::new(p1x, p1y), Point2D::new(p2x, p2y));
                    Ok(TransitionTimingFunction::CubicBezier(p1, p2))
                },
                "steps" => {
                    let (mut step_count, mut start_end) = (0, computed_value::StartEnd::End);
                    try!(input.parse_nested_block(|input| {
                        step_count = try!(specified::parse_integer(input));
                        if input.try(|input| input.expect_comma()).is_ok() {
                            start_end = try!(match_ignore_ascii_case! {
                                try!(input.expect_ident()),
                                "start" => Ok(computed_value::StartEnd::Start),
                                "end" => Ok(computed_value::StartEnd::End),
                                _ => Err(())
                            });
                        }
                        Ok(())
                    }));
                    Ok(TransitionTimingFunction::Steps(step_count as u32, start_end))
                },
                _ => Err(())
            }
        }
        match_ignore_ascii_case! {
            try!(input.expect_ident()),
            "ease" => Ok(ease()),
            "linear" => Ok(linear()),
            "ease-in" => Ok(ease_in()),
            "ease-out" => Ok(ease_out()),
            "ease-in-out" => Ok(ease_in_out()),
            "step-start" => Ok(STEP_START),
            "step-end" => Ok(STEP_END),
            _ => Err(())
        }
    }

    pub fn parse(_: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue,()> {
        Ok(SpecifiedValue(try!(input.parse_comma_separated(parse_one))))
    }
</%helpers:longhand>

<%helpers:longhand name="transition-property"
                   need_index="True"
                   animatable="False">

    use values::computed::ComputedValueAsSpecified;

    pub use self::computed_value::SingleComputedValue as SingleSpecifiedValue;
    pub use self::computed_value::T as SpecifiedValue;

    pub mod computed_value {
        use std::fmt;
        use style_traits::ToCss;
        // NB: Can't generate the type here because it needs all the longhands
        // generated beforehand.
        pub use properties::animated_properties::TransitionProperty;
        pub use properties::animated_properties::TransitionProperty as SingleComputedValue;

        #[derive(Clone, Debug, PartialEq)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub struct T(pub Vec<SingleComputedValue>);

        impl ToCss for T {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                if self.0.is_empty() {
                    return dest.write_str("none")
                }
                for (i, value) in self.0.iter().enumerate() {
                    if i != 0 {
                        try!(dest.write_str(", "))
                    }
                    try!(value.to_css(dest))
                }
                Ok(())
            }
        }
    }

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        computed_value::T(Vec::new())
    }


    #[inline]
    pub fn parse_one(input: &mut Parser) -> Result<SingleSpecifiedValue, ()> {
        SingleSpecifiedValue::parse(input)
    }

    pub fn parse(_: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue,()> {
        Ok(SpecifiedValue(try!(input.parse_comma_separated(SingleSpecifiedValue::parse))))
    }

    use values::NoViewportPercentage;
    impl NoViewportPercentage for SpecifiedValue {}

    impl ComputedValueAsSpecified for SpecifiedValue { }
</%helpers:longhand>

<%helpers:longhand name="transition-delay"
                   need_index="True"
                   animatable="False">
    pub use properties::longhands::transition_duration::{SingleSpecifiedValue, SpecifiedValue};
    pub use properties::longhands::transition_duration::{computed_value};
    pub use properties::longhands::transition_duration::{get_initial_single_value};
    pub use properties::longhands::transition_duration::{get_initial_value, parse, parse_one};
</%helpers:longhand>

<%helpers:longhand name="animation-name"
                   need_index="True"
                   animatable="False",
                   allowed_in_keyframe_block="False">
    use values::computed::ComputedValueAsSpecified;
    use values::NoViewportPercentage;

    pub mod computed_value {
        use std::fmt;
        use Atom;
        use style_traits::ToCss;

        pub use Atom as SingleComputedValue;

        #[derive(Debug, Clone, PartialEq)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub struct T(pub Vec<Atom>);

        impl ToCss for T {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                if self.0.is_empty() {
                    return dest.write_str("none")
                }

                for (i, name) in self.0.iter().enumerate() {
                    if i != 0 {
                        try!(dest.write_str(", "));
                    }
                    // NB: to_string() needed due to geckolib backend.
                    try!(dest.write_str(&*name.to_string()));
                }
                Ok(())
            }
        }
    }

    pub use self::computed_value::T as SpecifiedValue;
    impl NoViewportPercentage for SpecifiedValue {}
    pub use Atom as SingleSpecifiedValue;

    #[inline]
    pub fn parse_one(input: &mut Parser) -> Result<SingleSpecifiedValue, ()> {
        use cssparser::Token;

        Ok(match input.next() {
            Ok(Token::Ident(ref value)) if value != "none" => Atom::from(&**value),
            Ok(Token::QuotedString(value)) => Atom::from(&*value),
            _ => return Err(()),
        })
    }

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        computed_value::T(vec![])
    }

    pub fn parse(_: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue,()> {
        use std::borrow::Cow;
        Ok(SpecifiedValue(try!(input.parse_comma_separated(parse_one))))
    }

    impl ComputedValueAsSpecified for SpecifiedValue {}
</%helpers:longhand>

<%helpers:longhand name="animation-duration"
                   need_index="True"
                   animatable="False",
                   allowed_in_keyframe_block="False">
    pub use super::transition_duration::computed_value;
    pub use super::transition_duration::{get_initial_value, get_initial_single_value};
    pub use super::transition_duration::{parse, parse_one};
    pub use super::transition_duration::SpecifiedValue;
    pub use super::transition_duration::SingleSpecifiedValue;
</%helpers:longhand>

<%helpers:longhand name="animation-timing-function"
                   need_index="True"
                   animatable="False",
                   allowed_in_keyframe_block="False">
    pub use super::transition_timing_function::computed_value;
    pub use super::transition_timing_function::{get_initial_value, get_initial_single_value};
    pub use super::transition_timing_function::{parse, parse_one};
    pub use super::transition_timing_function::SpecifiedValue;
    pub use super::transition_timing_function::SingleSpecifiedValue;
</%helpers:longhand>

<%helpers:longhand name="animation-iteration-count"
                   need_index="True"
                   animatable="False",
                   allowed_in_keyframe_block="False">
    use values::computed::ComputedValueAsSpecified;
    use values::NoViewportPercentage;

    pub mod computed_value {
        use std::fmt;
        use style_traits::ToCss;

        pub use self::AnimationIterationCount as SingleComputedValue;

        #[derive(Debug, Clone, PartialEq)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub enum AnimationIterationCount {
            Number(u32),
            Infinite,
        }

        impl ToCss for AnimationIterationCount {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                match *self {
                    AnimationIterationCount::Number(n) => write!(dest, "{}", n),
                    AnimationIterationCount::Infinite => dest.write_str("infinite"),
                }
            }
        }

        #[derive(Debug, Clone, PartialEq)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub struct T(pub Vec<AnimationIterationCount>);

        impl ToCss for T {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                if self.0.is_empty() {
                    return dest.write_str("none")
                }
                for (i, value) in self.0.iter().enumerate() {
                    if i != 0 {
                        try!(dest.write_str(", "))
                    }
                    try!(value.to_css(dest))
                }
                Ok(())
            }
        }
    }

    pub use self::computed_value::AnimationIterationCount;
    pub use self::computed_value::AnimationIterationCount as SingleSpecifiedValue;
    pub use self::computed_value::T as SpecifiedValue;
    impl NoViewportPercentage for SpecifiedValue {}

    #[inline]
    pub fn get_initial_single_value() -> AnimationIterationCount {
        AnimationIterationCount::Number(1)
    }

    pub fn parse_one(input: &mut Parser) -> Result<AnimationIterationCount, ()> {
        if input.try(|input| input.expect_ident_matching("infinite")).is_ok() {
            Ok(AnimationIterationCount::Infinite)
        } else {
            let number = try!(input.expect_integer());
            if number < 0 {
                return Err(());
            }
            Ok(AnimationIterationCount::Number(number as u32))
        }
    }


    #[inline]
    pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
        Ok(SpecifiedValue(try!(input.parse_comma_separated(parse_one))))
    }

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        computed_value::T(vec![get_initial_single_value()])
    }

    impl ComputedValueAsSpecified for SpecifiedValue {}
</%helpers:longhand>

${helpers.keyword_list("animation-direction",
                       "normal reverse alternate alternate-reverse",
                       need_index=True,
                       animatable=False,
                       allowed_in_keyframe_block=False)}

// animation-play-state is the exception to the rule for allowed_in_keyframe_block:
// https://drafts.csswg.org/css-animations/#keyframes
${helpers.keyword_list("animation-play-state",
                       "running paused",
                       need_clone=True,
                       need_index=True,
                       animatable=False,
                       allowed_in_keyframe_block=True)}

${helpers.keyword_list("animation-fill-mode",
                       "none forwards backwards both",
                       need_index=True,
                       animatable=False,
                       allowed_in_keyframe_block=False)}

<%helpers:longhand name="animation-delay"
                   need_index="True"
                   animatable="False",
                   allowed_in_keyframe_block="False">
    pub use super::transition_duration::computed_value;
    pub use super::transition_duration::{get_initial_value, get_initial_single_value};
    pub use super::transition_duration::{parse, parse_one};
    pub use super::transition_duration::SpecifiedValue;
    pub use super::transition_duration::SingleSpecifiedValue;
</%helpers:longhand>

<%helpers:longhand products="gecko" name="scroll-snap-points-y" animatable="False">
    use std::fmt;
    use style_traits::ToCss;
    use values::HasViewportPercentage;
    use values::specified::LengthOrPercentage;

    impl HasViewportPercentage for SpecifiedValue {
        fn has_viewport_percentage(&self) -> bool {
            match *self {
                SpecifiedValue::Repeat(length) => length.has_viewport_percentage(),
                _ => false
            }
        }
    }

    pub mod computed_value {
        use values::computed::LengthOrPercentage;

        #[derive(Debug, Clone, PartialEq)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub struct T(pub Option<LengthOrPercentage>);
    }

    #[derive(Debug, Clone, Copy, PartialEq)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub enum SpecifiedValue {
        None,
        Repeat(LengthOrPercentage),
    }

    impl ToCss for computed_value::T {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match self.0 {
                None => dest.write_str("none"),
                Some(l) => {
                    try!(dest.write_str("repeat("));
                    try!(l.to_css(dest));
                    dest.write_str(")")
                },
            }
        }
    }
    impl ToCss for SpecifiedValue {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match *self {
                SpecifiedValue::None => dest.write_str("none"),
                SpecifiedValue::Repeat(ref l) => {
                    try!(dest.write_str("repeat("));
                    try!(l.to_css(dest));
                    dest.write_str(")")
                },
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
                SpecifiedValue::None => computed_value::T(None),
                SpecifiedValue::Repeat(l) =>
                    computed_value::T(Some(l.to_computed_value(context))),
            }
        }
        #[inline]
        fn from_computed_value(computed: &computed_value::T) -> Self {
            match *computed {
                computed_value::T(None) => SpecifiedValue::None,
                computed_value::T(Some(l)) =>
                    SpecifiedValue::Repeat(ToComputedValue::from_computed_value(&l))
            }
        }
    }

    pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
        if input.try(|input| input.expect_ident_matching("none")).is_ok() {
            Ok(SpecifiedValue::None)
        } else if input.try(|input| input.expect_function_matching("repeat")).is_ok() {
            input.parse_nested_block(|input| {
                LengthOrPercentage::parse_non_negative(input).map(SpecifiedValue::Repeat)
            })
        } else {
            Err(())
        }
    }
</%helpers:longhand>

<%helpers:longhand products="gecko" name="scroll-snap-points-x" animatable="False">
    pub use super::scroll_snap_points_y::SpecifiedValue;
    pub use super::scroll_snap_points_y::computed_value;
    pub use super::scroll_snap_points_y::get_initial_value;
    pub use super::scroll_snap_points_y::parse;
</%helpers:longhand>


// CSSOM View Module
// https://www.w3.org/TR/cssom-view-1/
${helpers.single_keyword("scroll-behavior",
                         "auto smooth",
                         products="gecko",
                         animatable=False)}

// Non-standard: https://developer.mozilla.org/en-US/docs/Web/CSS/scroll-snap-type-x
${helpers.single_keyword("scroll-snap-type-x",
                         "none mandatory proximity",
                         products="gecko",
                         gecko_constant_prefix="NS_STYLE_SCROLL_SNAP_TYPE",
                         animatable=False)}

// Non-standard: https://developer.mozilla.org/en-US/docs/Web/CSS/scroll-snap-type-y
${helpers.single_keyword("scroll-snap-type-y",
                         "none mandatory proximity",
                         products="gecko",
                         gecko_constant_prefix="NS_STYLE_SCROLL_SNAP_TYPE",
                         animatable=False)}

// Compositing and Blending Level 1
// http://www.w3.org/TR/compositing-1/
${helpers.single_keyword("isolation",
                         "auto isolate",
                         products="gecko",
                         animatable=False)}

${helpers.single_keyword("page-break-after",
                         "auto always avoid left right",
                         products="gecko",
                         animatable=False)}
${helpers.single_keyword("page-break-before",
                         "auto always avoid left right",
                         products="gecko",
                         animatable=False)}
${helpers.single_keyword("page-break-inside",
                         "auto avoid",
                         products="gecko",
                         gecko_ffi_name="mBreakInside",
                         gecko_constant_prefix="NS_STYLE_PAGE_BREAK",
                         animatable=False)}

// CSS Basic User Interface Module Level 3
// http://dev.w3.org/csswg/css-ui/
${helpers.single_keyword("resize",
                         "none both horizontal vertical",
                         products="gecko",
                         animatable=False)}

// Non-standard
${helpers.single_keyword("-moz-appearance",
                         """none button button-arrow-down button-arrow-next button-arrow-previous button-arrow-up
                            button-bevel button-focus caret checkbox checkbox-container checkbox-label checkmenuitem
                            dualbutton groupbox listbox listitem menuarrow menubar menucheckbox menuimage menuitem
                            menuitemtext menulist menulist-button menulist-text menulist-textfield menupopup menuradio
                            menuseparator meterbar meterchunk progressbar progressbar-vertical progresschunk
                            progresschunk-vertical radio radio-container radio-label radiomenuitem range range-thumb
                            resizer resizerpanel scale-horizontal scalethumbend scalethumb-horizontal scalethumbstart
                            scalethumbtick scalethumb-vertical scale-vertical scrollbarbutton-down scrollbarbutton-left
                            scrollbarbutton-right scrollbarbutton-up scrollbarthumb-horizontal scrollbarthumb-vertical
                            scrollbartrack-horizontal scrollbartrack-vertical searchfield separator spinner
                            spinner-downbutton spinner-textfield spinner-upbutton splitter statusbar statusbarpanel tab
                            tabpanel tabpanels tab-scroll-arrow-back tab-scroll-arrow-forward textfield
                            textfield-multiline toolbar toolbarbutton toolbarbutton-dropdown toolbargripper toolbox
                            tooltip treeheader treeheadercell treeheadersortarrow treeitem treeline treetwisty
                            treetwistyopen treeview -moz-win-borderless-glass -moz-win-browsertabbar-toolbox
                            -moz-win-communications-toolbox -moz-win-exclude-glass -moz-win-glass -moz-win-media-toolbox
                            -moz-window-button-box -moz-window-button-box-maximized -moz-window-button-close
                            -moz-window-button-maximize -moz-window-button-minimize -moz-window-button-restore
                            -moz-window-frame-bottom -moz-window-frame-left -moz-window-frame-right -moz-window-titlebar
                            -moz-window-titlebar-maximized
                         """,
                         gecko_ffi_name="mAppearance",
                         gecko_constant_prefix="NS_THEME",
                         products="gecko",
                         animatable=False)}

// Non-standard: https://developer.mozilla.org/en-US/docs/Web/CSS/-moz-binding
${helpers.predefined_type("-moz-binding", "UrlOrNone", "computed_value::T::None",
                          needs_context=True,
                          products="gecko",
                          animatable="False",
                          disable_when_testing="True")}
