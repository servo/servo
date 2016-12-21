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
            list-item flex none
        """.split()
        if product == "gecko":
            values += """inline-flex grid inline-grid ruby ruby-base ruby-base-container
                ruby-text ruby-text-container contents -webkit-box -webkit-inline-box
                -moz-box -moz-inline-box -moz-grid -moz-inline-grid -moz-grid-group
                -moz-grid-line -moz-stack -moz-inline-stack -moz-deck -moz-popup
                -moz-groupbox""".split()
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
                                  gecko_inexhaustive="True"
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
  pub fn parse(context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
      input.try(|i| specified::LengthOrPercentage::parse(context, i))
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
    pub fn get_initial_single_value() -> Time {
        Time(0.0)
    }

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        computed_value::T(vec![get_initial_single_value()])
    }

    pub fn parse(context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue,()> {
        Ok(SpecifiedValue(try!(input.parse_comma_separated(|i| Time::parse(context, i)))))
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
        use parser::{Parse, ParserContext};
        use std::fmt;
        use style_traits::ToCss;
        use values::specified;
        use values::computed::ComputedValueAsSpecified;

        pub use self::TransitionTimingFunction as SingleComputedValue;

        #[derive(Copy, Clone, Debug, PartialEq)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub enum TransitionTimingFunction {
            CubicBezier(Point2D<f32>, Point2D<f32>),
            Steps(u32, StartEnd),
        }

        impl Parse for TransitionTimingFunction {
            fn parse(_context: &ParserContext, input: &mut ::cssparser::Parser) -> Result<Self, ()> {
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
                            let (mut step_count, mut start_end) = (0, StartEnd::End);
                            try!(input.parse_nested_block(|input| {
                                step_count = try!(specified::parse_integer(input));
                                if input.try(|input| input.expect_comma()).is_ok() {
                                    start_end = try!(match_ignore_ascii_case! {
                                        try!(input.expect_ident()),
                                        "start" => Ok(StartEnd::Start),
                                        "end" => Ok(StartEnd::End),
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
                    "ease" => Ok(super::ease()),
                    "linear" => Ok(super::linear()),
                    "ease-in" => Ok(super::ease_in()),
                    "ease-out" => Ok(super::ease_out()),
                    "ease-in-out" => Ok(super::ease_in_out()),
                    "step-start" => Ok(super::STEP_START),
                    "step-end" => Ok(super::STEP_END),
                    _ => Err(())
                }
            }
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
    pub fn get_initial_single_value() -> TransitionTimingFunction {
        ease()
    }

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        computed_value::T(vec![get_initial_single_value()])
    }

    pub fn parse(context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue,()> {
        Ok(SpecifiedValue(try!(input.parse_comma_separated(|i| {
            TransitionTimingFunction::parse(context, i)
        }))))
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

    pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue,()> {
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
    pub use properties::longhands::transition_duration::computed_value;
    pub use properties::longhands::transition_duration::{get_initial_value, get_initial_single_value, parse};
</%helpers:longhand>

<%helpers:longhand name="animation-name"
                   need_index="True"
                   animatable="False",
                   allowed_in_keyframe_block="False">
    use values::computed::ComputedValueAsSpecified;
    use values::NoViewportPercentage;

    pub mod computed_value {
        use Atom;
        use parser::{Parse, ParserContext};
        use std::fmt;
        use std::ops::Deref;
        use style_traits::ToCss;

        #[derive(Clone, Debug, Hash, Eq, PartialEq)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub struct AnimationName(pub Atom);

        impl fmt::Display for AnimationName {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                self.0.fmt(f)
            }
        }

        pub use self::AnimationName as SingleComputedValue;

        impl Parse for AnimationName {
            fn parse(_context: &ParserContext, input: &mut ::cssparser::Parser) -> Result<Self, ()> {
                use cssparser::Token;
                Ok(match input.next() {
                    Ok(Token::Ident(ref value)) if value != "none" => AnimationName(Atom::from(&**value)),
                    Ok(Token::QuotedString(value)) => AnimationName(Atom::from(&*value)),
                    _ => return Err(()),
                })
            }
        }

        #[derive(Debug, Clone, PartialEq)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub struct T(pub Vec<AnimationName>);

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
    pub use self::computed_value::SingleComputedValue as SingleSpecifiedValue;

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        computed_value::T(vec![])
    }

    pub fn parse(context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue,()> {
        use std::borrow::Cow;
        Ok(SpecifiedValue(try!(input.parse_comma_separated(|i| SingleSpecifiedValue::parse(context, i)))))
    }

    impl ComputedValueAsSpecified for SpecifiedValue {}
</%helpers:longhand>

<%helpers:longhand name="animation-duration"
                   need_index="True"
                   animatable="False",
                   allowed_in_keyframe_block="False">
    pub use super::transition_duration::computed_value;
    pub use super::transition_duration::{get_initial_value, get_initial_single_value, parse};
    pub use super::transition_duration::SpecifiedValue;
    pub use super::transition_duration::SingleSpecifiedValue;
</%helpers:longhand>

<%helpers:longhand name="animation-timing-function"
                   need_index="True"
                   animatable="False",
                   allowed_in_keyframe_block="False">
    pub use super::transition_timing_function::computed_value;
    pub use super::transition_timing_function::{get_initial_value, get_initial_single_value, parse};
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
        use parser::{Parse, ParserContext};
        use std::fmt;
        use style_traits::ToCss;

        pub use self::AnimationIterationCount as SingleComputedValue;

        #[derive(Debug, Clone, PartialEq)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub enum AnimationIterationCount {
            Number(u32),
            Infinite,
        }

        impl Parse for AnimationIterationCount {
            fn parse(_context: &ParserContext, input: &mut ::cssparser::Parser) -> Result<Self, ()> {
                if input.try(|input| input.expect_ident_matching("infinite")).is_ok() {
                    return Ok(AnimationIterationCount::Infinite)
                }

                let number = try!(input.expect_integer());
                if number < 0 {
                    return Err(());
                }

                Ok(AnimationIterationCount::Number(number as u32))
            }
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

    #[inline]
    pub fn parse(context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
        Ok(SpecifiedValue(try!(input.parse_comma_separated(|i| {
            AnimationIterationCount::parse(context, i)
        }))))
    }

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        computed_value::T(vec![get_initial_single_value()])
    }

    impl ComputedValueAsSpecified for SpecifiedValue {}
</%helpers:longhand>

${helpers.single_keyword("animation-direction",
                         "normal reverse alternate alternate-reverse",
                         need_index=True,
                         animatable=False,
                         vector=True,
                         allowed_in_keyframe_block=False)}

// animation-play-state is the exception to the rule for allowed_in_keyframe_block:
// https://drafts.csswg.org/css-animations/#keyframes
${helpers.single_keyword("animation-play-state",
                         "running paused",
                         need_clone=True,
                         need_index=True,
                         animatable=False,
                         vector=True,
                         allowed_in_keyframe_block=True)}

${helpers.single_keyword("animation-fill-mode",
                         "none forwards backwards both",
                         need_index=True,
                         animatable=False,
                         vector=True,
                         allowed_in_keyframe_block=False)}

<%helpers:longhand name="animation-delay"
                   need_index="True"
                   animatable="False",
                   allowed_in_keyframe_block="False">
    pub use super::transition_duration::computed_value;
    pub use super::transition_duration::{get_initial_value, get_initial_single_value, parse};
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



<%helpers:longhand name="transform" products="gecko servo" animatable="${product == 'servo'}">
    use app_units::Au;
    use style_traits::ToCss;
    use values::CSSFloat;
    use values::HasViewportPercentage;

    use std::fmt;

    pub mod computed_value {
        use values::CSSFloat;
        use values::computed;

        #[derive(Clone, Copy, Debug, PartialEq)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub struct ComputedMatrix {
            pub m11: CSSFloat, pub m12: CSSFloat, pub m13: CSSFloat, pub m14: CSSFloat,
            pub m21: CSSFloat, pub m22: CSSFloat, pub m23: CSSFloat, pub m24: CSSFloat,
            pub m31: CSSFloat, pub m32: CSSFloat, pub m33: CSSFloat, pub m34: CSSFloat,
            pub m41: CSSFloat, pub m42: CSSFloat, pub m43: CSSFloat, pub m44: CSSFloat,
        }

        impl ComputedMatrix {
            pub fn identity() -> ComputedMatrix {
                ComputedMatrix {
                    m11: 1.0, m12: 0.0, m13: 0.0, m14: 0.0,
                    m21: 0.0, m22: 1.0, m23: 0.0, m24: 0.0,
                    m31: 0.0, m32: 0.0, m33: 1.0, m34: 0.0,
                    m41: 0.0, m42: 0.0, m43: 0.0, m44: 1.0
                }
            }
        }

        #[derive(Clone, Debug, PartialEq)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub enum ComputedOperation {
            Matrix(ComputedMatrix),
            Skew(computed::Angle, computed::Angle),
            Translate(computed::LengthOrPercentage,
                      computed::LengthOrPercentage,
                      computed::Length),
            Scale(CSSFloat, CSSFloat, CSSFloat),
            Rotate(CSSFloat, CSSFloat, CSSFloat, computed::Angle),
            Perspective(computed::Length),
        }

        #[derive(Clone, Debug, PartialEq)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub struct T(pub Option<Vec<ComputedOperation>>);
    }

    pub use self::computed_value::ComputedMatrix as SpecifiedMatrix;

    fn parse_two_lengths_or_percentages(context: &ParserContext, input: &mut Parser)
                                        -> Result<(specified::LengthOrPercentage,
                                                   specified::LengthOrPercentage),()> {
        let first = try!(specified::LengthOrPercentage::parse(context, input));
        let second = input.try(|input| {
            try!(input.expect_comma());
            specified::LengthOrPercentage::parse(context, input)
        }).unwrap_or(specified::LengthOrPercentage::zero());
        Ok((first, second))
    }

    fn parse_two_floats(input: &mut Parser) -> Result<(CSSFloat,CSSFloat),()> {
        let first = try!(specified::parse_number(input));
        let second = input.try(|input| {
            try!(input.expect_comma());
            specified::parse_number(input)
        }).unwrap_or(first);
        Ok((first, second))
    }

    fn parse_two_angles(context: &ParserContext, input: &mut Parser)
                       -> Result<(specified::Angle, specified::Angle),()> {
        let first = try!(specified::Angle::parse(context, input));
        let second = input.try(|input| {
            try!(input.expect_comma());
            specified::Angle::parse(context, input)
        }).unwrap_or(specified::Angle(0.0));
        Ok((first, second))
    }

    #[derive(Copy, Clone, Debug, PartialEq)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    enum TranslateKind {
        Translate,
        TranslateX,
        TranslateY,
        TranslateZ,
        Translate3D,
    }

    #[derive(Clone, Debug, PartialEq)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    enum SpecifiedOperation {
        Matrix(SpecifiedMatrix),
        Skew(specified::Angle, specified::Angle),
        Translate(TranslateKind,
                  specified::LengthOrPercentage,
                  specified::LengthOrPercentage,
                  specified::Length),
        Scale(CSSFloat, CSSFloat, CSSFloat),
        Rotate(CSSFloat, CSSFloat, CSSFloat, specified::Angle),
        Perspective(specified::Length),
    }

    impl ToCss for computed_value::T {
        fn to_css<W>(&self, _: &mut W) -> fmt::Result where W: fmt::Write {
            // TODO(pcwalton)
            Ok(())
        }
    }

    impl HasViewportPercentage for SpecifiedOperation {
        fn has_viewport_percentage(&self) -> bool {
            match *self {
                SpecifiedOperation::Translate(_, l1, l2, l3) => {
                    l1.has_viewport_percentage() ||
                    l2.has_viewport_percentage() ||
                    l3.has_viewport_percentage()
                },
                SpecifiedOperation::Perspective(length) => length.has_viewport_percentage(),
                _ => false
            }
        }
    }

    impl ToCss for SpecifiedOperation {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match *self {
                // todo(gw): implement serialization for transform
                // types other than translate.
                SpecifiedOperation::Matrix(_m) => {
                    Ok(())
                }
                SpecifiedOperation::Skew(_sx, _sy) => {
                    Ok(())
                }
                SpecifiedOperation::Translate(kind, tx, ty, tz) => {
                    match kind {
                        TranslateKind::Translate => {
                            try!(dest.write_str("translate("));
                            try!(tx.to_css(dest));
                            try!(dest.write_str(", "));
                            try!(ty.to_css(dest));
                            dest.write_str(")")
                        }
                        TranslateKind::TranslateX => {
                            try!(dest.write_str("translateX("));
                            try!(tx.to_css(dest));
                            dest.write_str(")")
                        }
                        TranslateKind::TranslateY => {
                            try!(dest.write_str("translateY("));
                            try!(ty.to_css(dest));
                            dest.write_str(")")
                        }
                        TranslateKind::TranslateZ => {
                            try!(dest.write_str("translateZ("));
                            try!(tz.to_css(dest));
                            dest.write_str(")")
                        }
                        TranslateKind::Translate3D => {
                            try!(dest.write_str("translate3d("));
                            try!(tx.to_css(dest));
                            try!(dest.write_str(", "));
                            try!(ty.to_css(dest));
                            try!(dest.write_str(", "));
                            try!(tz.to_css(dest));
                            dest.write_str(")")
                        }
                    }
                }
                SpecifiedOperation::Scale(_sx, _sy, _sz) => {
                    Ok(())
                }
                SpecifiedOperation::Rotate(_ax, _ay, _az, _angle) => {
                    Ok(())
                }
                SpecifiedOperation::Perspective(_p) => {
                    Ok(())
                }
            }
        }
    }

    impl HasViewportPercentage for SpecifiedValue {
        fn has_viewport_percentage(&self) -> bool {
            let &SpecifiedValue(ref specified_ops) = self;
            specified_ops.iter().any(|ref x| x.has_viewport_percentage())
        }
    }

    #[derive(Clone, Debug, PartialEq)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub struct SpecifiedValue(Vec<SpecifiedOperation>);

    impl ToCss for SpecifiedValue {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            let mut first = true;
            for operation in &self.0 {
                if !first {
                    try!(dest.write_str(" "));
                }
                first = false;
                try!(operation.to_css(dest))
            }
            Ok(())
        }
    }

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        computed_value::T(None)
    }

    pub fn parse(context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue,()> {
        if input.try(|input| input.expect_ident_matching("none")).is_ok() {
            return Ok(SpecifiedValue(Vec::new()))
        }

        let mut result = Vec::new();
        loop {
            let name = match input.expect_function() {
                Ok(name) => name,
                Err(_) => break,
            };
            match_ignore_ascii_case! {
                name,
                "matrix" => {
                    try!(input.parse_nested_block(|input| {
                        let values = try!(input.parse_comma_separated(|input| {
                            specified::parse_number(input)
                        }));
                        if values.len() != 6 {
                            return Err(())
                        }
                        result.push(SpecifiedOperation::Matrix(
                                SpecifiedMatrix {
                                    m11: values[0], m12: values[1], m13: 0.0, m14: 0.0,
                                    m21: values[2], m22: values[3], m23: 0.0, m24: 0.0,
                                    m31:       0.0, m32:       0.0, m33: 1.0, m34: 0.0,
                                    m41: values[4], m42: values[5], m43: 0.0, m44: 1.0
                                }));
                        Ok(())
                    }))
                },
                "matrix3d" => {
                    try!(input.parse_nested_block(|input| {
                        let values = try!(input.parse_comma_separated(|input| {
                            specified::parse_number(input)
                        }));
                        if values.len() != 16 {
                            return Err(())
                        }
                        result.push(SpecifiedOperation::Matrix(
                                SpecifiedMatrix {
                                    m11: values[ 0], m12: values[ 1], m13: values[ 2], m14: values[ 3],
                                    m21: values[ 4], m22: values[ 5], m23: values[ 6], m24: values[ 7],
                                    m31: values[ 8], m32: values[ 9], m33: values[10], m34: values[11],
                                    m41: values[12], m42: values[13], m43: values[14], m44: values[15]
                                }));
                        Ok(())
                    }))
                },
                "translate" => {
                    try!(input.parse_nested_block(|input| {
                        let (tx, ty) = try!(parse_two_lengths_or_percentages(context, input));
                        result.push(SpecifiedOperation::Translate(TranslateKind::Translate,
                                                                  tx,
                                                                  ty,
                                                                  specified::Length::Absolute(Au(0))));
                        Ok(())
                    }))
                },
                "translatex" => {
                    try!(input.parse_nested_block(|input| {
                        let tx = try!(specified::LengthOrPercentage::parse(context, input));
                        result.push(SpecifiedOperation::Translate(
                            TranslateKind::TranslateX,
                            tx,
                            specified::LengthOrPercentage::zero(),
                            specified::Length::Absolute(Au(0))));
                        Ok(())
                    }))
                },
                "translatey" => {
                    try!(input.parse_nested_block(|input| {
                        let ty = try!(specified::LengthOrPercentage::parse(context, input));
                        result.push(SpecifiedOperation::Translate(
                            TranslateKind::TranslateY,
                            specified::LengthOrPercentage::zero(),
                            ty,
                            specified::Length::Absolute(Au(0))));
                        Ok(())
                    }))
                },
                "translatez" => {
                    try!(input.parse_nested_block(|input| {
                        let tz = try!(specified::Length::parse(context, input));
                        result.push(SpecifiedOperation::Translate(
                            TranslateKind::TranslateZ,
                            specified::LengthOrPercentage::zero(),
                            specified::LengthOrPercentage::zero(),
                            tz));
                        Ok(())
                    }))
                },
                "translate3d" => {
                    try!(input.parse_nested_block(|input| {
                        let tx = try!(specified::LengthOrPercentage::parse(context, input));
                        try!(input.expect_comma());
                        let ty = try!(specified::LengthOrPercentage::parse(context, input));
                        try!(input.expect_comma());
                        let tz = try!(specified::Length::parse(context, input));
                        result.push(SpecifiedOperation::Translate(
                            TranslateKind::Translate3D,
                            tx,
                            ty,
                            tz));
                        Ok(())
                    }))

                },
                "scale" => {
                    try!(input.parse_nested_block(|input| {
                        let (sx, sy) = try!(parse_two_floats(input));
                        result.push(SpecifiedOperation::Scale(sx, sy, 1.0));
                        Ok(())
                    }))
                },
                "scalex" => {
                    try!(input.parse_nested_block(|input| {
                        let sx = try!(specified::parse_number(input));
                        result.push(SpecifiedOperation::Scale(sx, 1.0, 1.0));
                        Ok(())
                    }))
                },
                "scaley" => {
                    try!(input.parse_nested_block(|input| {
                        let sy = try!(specified::parse_number(input));
                        result.push(SpecifiedOperation::Scale(1.0, sy, 1.0));
                        Ok(())
                    }))
                },
                "scalez" => {
                    try!(input.parse_nested_block(|input| {
                        let sz = try!(specified::parse_number(input));
                        result.push(SpecifiedOperation::Scale(1.0, 1.0, sz));
                        Ok(())
                    }))
                },
                "scale3d" => {
                    try!(input.parse_nested_block(|input| {
                        let sx = try!(specified::parse_number(input));
                        try!(input.expect_comma());
                        let sy = try!(specified::parse_number(input));
                        try!(input.expect_comma());
                        let sz = try!(specified::parse_number(input));
                        result.push(SpecifiedOperation::Scale(sx, sy, sz));
                        Ok(())
                    }))
                },
                "rotate" => {
                    try!(input.parse_nested_block(|input| {
                        let theta = try!(specified::Angle::parse(context,input));
                        result.push(SpecifiedOperation::Rotate(0.0, 0.0, 1.0, theta));
                        Ok(())
                    }))
                },
                "rotatex" => {
                    try!(input.parse_nested_block(|input| {
                        let theta = try!(specified::Angle::parse(context,input));
                        result.push(SpecifiedOperation::Rotate(1.0, 0.0, 0.0, theta));
                        Ok(())
                    }))
                },
                "rotatey" => {
                    try!(input.parse_nested_block(|input| {
                        let theta = try!(specified::Angle::parse(context,input));
                        result.push(SpecifiedOperation::Rotate(0.0, 1.0, 0.0, theta));
                        Ok(())
                    }))
                },
                "rotatez" => {
                    try!(input.parse_nested_block(|input| {
                        let theta = try!(specified::Angle::parse(context,input));
                        result.push(SpecifiedOperation::Rotate(0.0, 0.0, 1.0, theta));
                        Ok(())
                    }))
                },
                "rotate3d" => {
                    try!(input.parse_nested_block(|input| {
                        let ax = try!(specified::parse_number(input));
                        try!(input.expect_comma());
                        let ay = try!(specified::parse_number(input));
                        try!(input.expect_comma());
                        let az = try!(specified::parse_number(input));
                        try!(input.expect_comma());
                        let theta = try!(specified::Angle::parse(context,input));
                        // TODO(gw): Check the axis can be normalized!!
                        result.push(SpecifiedOperation::Rotate(ax, ay, az, theta));
                        Ok(())
                    }))
                },
                "skew" => {
                    try!(input.parse_nested_block(|input| {
                        let (theta_x, theta_y) = try!(parse_two_angles(context, input));
                        result.push(SpecifiedOperation::Skew(theta_x, theta_y));
                        Ok(())
                    }))
                },
                "skewx" => {
                    try!(input.parse_nested_block(|input| {
                        let theta_x = try!(specified::Angle::parse(context,input));
                        result.push(SpecifiedOperation::Skew(theta_x, specified::Angle(0.0)));
                        Ok(())
                    }))
                },
                "skewy" => {
                    try!(input.parse_nested_block(|input| {
                        let theta_y = try!(specified::Angle::parse(context,input));
                        result.push(SpecifiedOperation::Skew(specified::Angle(0.0), theta_y));
                        Ok(())
                    }))
                },
                "perspective" => {
                    try!(input.parse_nested_block(|input| {
                        let d = try!(specified::Length::parse(context, input));
                        result.push(SpecifiedOperation::Perspective(d));
                        Ok(())
                    }))
                },
                _ => return Err(())
            }
        }

        if !result.is_empty() {
            Ok(SpecifiedValue(result))
        } else {
            Err(())
        }
    }

    impl ToComputedValue for SpecifiedValue {
        type ComputedValue = computed_value::T;

        #[inline]
        fn to_computed_value(&self, context: &Context) -> computed_value::T {
            if self.0.is_empty() {
                return computed_value::T(None)
            }

            let mut result = vec!();
            for operation in &self.0 {
                match *operation {
                    SpecifiedOperation::Matrix(ref matrix) => {
                        result.push(computed_value::ComputedOperation::Matrix(*matrix));
                    }
                    SpecifiedOperation::Translate(_, ref tx, ref ty, ref tz) => {
                        result.push(computed_value::ComputedOperation::Translate(tx.to_computed_value(context),
                                                                                 ty.to_computed_value(context),
                                                                                 tz.to_computed_value(context)));
                    }
                    SpecifiedOperation::Scale(sx, sy, sz) => {
                        result.push(computed_value::ComputedOperation::Scale(sx, sy, sz));
                    }
                    SpecifiedOperation::Rotate(ax, ay, az, theta) => {
                        let len = (ax * ax + ay * ay + az * az).sqrt();
                        result.push(computed_value::ComputedOperation::Rotate(ax / len, ay / len, az / len, theta));
                    }
                    SpecifiedOperation::Skew(theta_x, theta_y) => {
                        result.push(computed_value::ComputedOperation::Skew(theta_x, theta_y));
                    }
                    SpecifiedOperation::Perspective(d) => {
                        result.push(computed_value::ComputedOperation::Perspective(d.to_computed_value(context)));
                    }
                };
            }

            computed_value::T(Some(result))
        }

        #[inline]
        fn from_computed_value(computed: &computed_value::T) -> Self {
            SpecifiedValue(computed.0.as_ref().map(|computed| {
                let mut result = vec!();
                for operation in computed {
                    match *operation {
                        computed_value::ComputedOperation::Matrix(ref matrix) => {
                            result.push(SpecifiedOperation::Matrix(*matrix));
                        }
                        computed_value::ComputedOperation::Translate(ref tx, ref ty, ref tz) => {
                            // XXXManishearth we lose information here; perhaps we should try to
                            // recover the original function? Not sure if this can be observed.
                            result.push(SpecifiedOperation::Translate(TranslateKind::Translate,
                                              ToComputedValue::from_computed_value(tx),
                                              ToComputedValue::from_computed_value(ty),
                                              ToComputedValue::from_computed_value(tz)));
                        }
                        computed_value::ComputedOperation::Scale(sx, sy, sz) => {
                            result.push(SpecifiedOperation::Scale(sx, sy, sz));
                        }
                        computed_value::ComputedOperation::Rotate(ax, ay, az, theta) => {
                            result.push(SpecifiedOperation::Rotate(ax, ay, az, theta));
                        }
                        computed_value::ComputedOperation::Skew(theta_x, theta_y) => {
                            result.push(SpecifiedOperation::Skew(theta_x, theta_y));
                        }
                        computed_value::ComputedOperation::Perspective(ref d) => {
                            result.push(SpecifiedOperation::Perspective(
                                ToComputedValue::from_computed_value(d)
                            ));
                        }
                    };
                }
                result
            }).unwrap_or(Vec::new()))
        }
    }
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
<%helpers:longhand products="gecko" name="scroll-snap-type-y" animatable="False">
    pub use super::scroll_snap_type_x::SpecifiedValue;
    pub use super::scroll_snap_type_x::computed_value;
    pub use super::scroll_snap_type_x::get_initial_value;
    pub use super::scroll_snap_type_x::parse;
</%helpers:longhand>

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
${helpers.predefined_type("-moz-binding", "UrlOrNone", "Either::Second(None_)",
                          products="gecko",
                          animatable="False",
                          disable_when_testing="True")}

// Non-standard: https://developer.mozilla.org/en-US/docs/Web/CSS/-moz-orient
${helpers.single_keyword("-moz-orient",
                          "inline block horizontal vertical",
                          products="gecko",
                          gecko_ffi_name="mOrient",
                          gecko_enum_prefix="StyleOrient",
                          animatable=False)}
