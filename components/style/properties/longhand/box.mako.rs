/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />
<% from data import Keyword, Method, to_rust_ident %>

<% data.new_style_struct("Box",
                         inherited=False,
                         gecko_name="Display",
                         additional_methods=[Method("transition_count", "usize")]) %>

// TODO(SimonSapin): don't parse `inline-table`, since we don't support it
<%helpers:longhand name="display" need_clone="True" custom_cascade="${product == 'servo'}">
    <%
        values = """inline block inline-block
            table inline-table table-row-group table-header-group table-footer-group
            table-row table-column-group table-column table-cell table-caption
            list-item flex
            none
        """.split()
        if product == "gecko":
            values += "-moz-box -moz-inline-box".split()
        experimental_values = set("flex".split())
    %>
    pub use self::computed_value::T as SpecifiedValue;
    use values::computed::{Context, ComputedValueAsSpecified};

    pub mod computed_value {
        #[allow(non_camel_case_types)]
        #[derive(Clone, Eq, PartialEq, Copy, Hash, RustcEncodable, Debug)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf, Deserialize, Serialize))]
        pub enum T {
            % for value in values:
                ${to_rust_ident(value)},
            % endfor
        }

        impl ::cssparser::ToCss for T {
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
                    % if value in experimental_values:
                        if !::util::prefs::get_pref("layout.${value}.enabled")
                            .as_boolean().unwrap_or(false) {
                            return Err(())
                        }
                    % endif
                    Ok(computed_value::T::${to_rust_ident(value)})
                },
            % endfor
            _ => Err(())
        }
    }

    impl ComputedValueAsSpecified for SpecifiedValue {}

    % if product == "servo":
        fn cascade_property_custom<C: ComputedValues>(
                                   _declaration: &PropertyDeclaration,
                                   _inherited_style: &C,
                                   context: &mut computed::Context<C>,
                                   _seen: &mut PropertyBitField,
                                   _cacheable: &mut bool,
                                   _error_reporter: &mut StdBox<ParseErrorReporter + Send>) {
            longhands::_servo_display_for_hypothetical_box::derive_from_display(context);
            longhands::_servo_text_decorations_in_effect::derive_from_display(context);
            longhands::_servo_under_display_none::derive_from_display(context);
        }
    % endif

</%helpers:longhand>

${helpers.single_keyword("position", "static absolute relative fixed", need_clone=True, extra_gecko_values="sticky")}

<%helpers:single_keyword_computed name="float" values="none left right" need_clone="True" gecko_ffi_name="mFloats">
    impl ToComputedValue for SpecifiedValue {
        type ComputedValue = computed_value::T;

        #[inline]
        fn to_computed_value<Cx: TContext>(&self, context: &Cx) -> computed_value::T {
            let positioned = matches!(context.style().get_box().clone_position(),
                longhands::position::SpecifiedValue::absolute |
                longhands::position::SpecifiedValue::fixed);
            if positioned {
                SpecifiedValue::none
            } else {
                *self
            }
        }
    }

</%helpers:single_keyword_computed>

${helpers.single_keyword("clear", "none left right both", gecko_ffi_name="mBreakType")}

<%helpers:longhand name="-servo-display-for-hypothetical-box" derived_from="display" products="servo">
    pub use super::display::{SpecifiedValue, get_initial_value};
    pub use super::display::{parse};

    pub mod computed_value {
        pub type T = super::SpecifiedValue;
    }

    #[inline]
    pub fn derive_from_display<Cx: TContext>(context: &mut Cx) {
        let d = context.style().get_box().clone_display();
        context.mutate_style().mutate_box().set__servo_display_for_hypothetical_box(d);
    }

</%helpers:longhand>

<%helpers:longhand name="vertical-align">
  use cssparser::ToCss;
  use std::fmt;

  <% vertical_align = data.longhands_by_name["vertical-align"] %>
  <% vertical_align.keyword = Keyword("vertical-align",
                                      "baseline sub super top text-top middle bottom text-bottom",
                                      extra_gecko_values="middle-with-baseline") %>
  <% vertical_align_keywords = vertical_align.keyword.values_for(product) %>

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
      use values::AuExtensionMethods;
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
      impl ::cssparser::ToCss for T {
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
      fn to_computed_value<Cx: TContext>(&self, context: &Cx) -> computed_value::T {
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
  }
</%helpers:longhand>


// CSS 2.1, Section 11 - Visual effects

// Non-standard, see https://developer.mozilla.org/en-US/docs/Web/CSS/overflow-clip-box#Specifications
${helpers.single_keyword("-servo-overflow-clip-box", "padding-box content-box", products="servo",
               internal=True)}

${helpers.single_keyword("overflow-clip-box", "padding-box content-box", products="gecko",
               internal=True)}

// FIXME(pcwalton, #2742): Implement scrolling for `scroll` and `auto`.
${helpers.single_keyword("overflow-x", "visible hidden scroll auto", need_clone=True,
                       gecko_constant_prefix="NS_STYLE_OVERFLOW")}

// FIXME(pcwalton, #2742): Implement scrolling for `scroll` and `auto`.
<%helpers:longhand name="overflow-y" need_clone="True">
  use super::overflow_x;

  use cssparser::ToCss;
  use std::fmt;

  pub use self::computed_value::T as SpecifiedValue;

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

  impl ToComputedValue for SpecifiedValue {
      type ComputedValue = computed_value::T;

      #[inline]
      fn to_computed_value<Cx: TContext>(&self, context: &Cx) -> computed_value::T {
          computed_value::T(self.0.to_computed_value(context))
      }
  }

  pub fn get_initial_value() -> computed_value::T {
      computed_value::T(overflow_x::get_initial_value())
  }

  pub fn parse(context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue,()> {
      overflow_x::parse(context, input).map(SpecifiedValue)
  }
</%helpers:longhand>

// TODO(pcwalton): Multiple transitions.
<%helpers:longhand name="transition-duration">
    use values::specified::Time;

    pub use self::computed_value::T as SpecifiedValue;
    pub use values::specified::Time as SingleSpecifiedValue;

    pub mod computed_value {
        use cssparser::ToCss;
        use std::fmt;
        use values::computed::{TContext, ToComputedValue};

        pub use values::computed::Time as SingleComputedValue;

        #[derive(Debug, Clone, PartialEq)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub struct T(pub Vec<SingleComputedValue>);

        impl ToComputedValue for T {
            type ComputedValue = T;

            #[inline]
            fn to_computed_value<Cx: TContext>(&self, _: &Cx) -> T {
                (*self).clone()
            }
        }

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
<%helpers:longhand name="transition-timing-function">
    use self::computed_value::{StartEnd, TransitionTimingFunction};

    use euclid::point::Point2D;

    pub use self::computed_value::SingleComputedValue as SingleSpecifiedValue;
    pub use self::computed_value::T as SpecifiedValue;

    static EASE: TransitionTimingFunction = TransitionTimingFunction::CubicBezier(Point2D {
        x: 0.25,
        y: 0.1,
    }, Point2D {
        x: 0.25,
        y: 1.0,
    });
    static LINEAR: TransitionTimingFunction = TransitionTimingFunction::CubicBezier(Point2D {
        x: 0.0,
        y: 0.0,
    }, Point2D {
        x: 1.0,
        y: 1.0,
    });
    static EASE_IN: TransitionTimingFunction = TransitionTimingFunction::CubicBezier(Point2D {
        x: 0.42,
        y: 0.0,
    }, Point2D {
        x: 1.0,
        y: 1.0,
    });
    static EASE_OUT: TransitionTimingFunction = TransitionTimingFunction::CubicBezier(Point2D {
        x: 0.0,
        y: 0.0,
    }, Point2D {
        x: 0.58,
        y: 1.0,
    });
    static EASE_IN_OUT: TransitionTimingFunction =
        TransitionTimingFunction::CubicBezier(Point2D {
            x: 0.42,
            y: 0.0,
        }, Point2D {
            x: 0.58,
            y: 1.0,
        });
    static STEP_START: TransitionTimingFunction =
        TransitionTimingFunction::Steps(1, StartEnd::Start);
    static STEP_END: TransitionTimingFunction =
        TransitionTimingFunction::Steps(1, StartEnd::End);

    pub mod computed_value {
        use cssparser::ToCss;
        use euclid::point::Point2D;
        use std::fmt;

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

    impl ToComputedValue for SpecifiedValue {
        type ComputedValue = computed_value::T;

        #[inline]
        fn to_computed_value<Cx: TContext>(&self, _: &Cx) -> computed_value::T {
            (*self).clone()
        }
    }

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        computed_value::T(vec![get_initial_single_value()])
    }

    #[inline]
    pub fn get_initial_single_value() -> TransitionTimingFunction {
        EASE
    }

    pub fn parse_one(input: &mut Parser) -> Result<SingleSpecifiedValue,()> {
        if let Ok(function_name) = input.try(|input| input.expect_function()) {
            return match_ignore_ascii_case! {
                function_name,
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
                    let (mut step_count, mut start_end) = (0, computed_value::StartEnd::Start);
                    try!(input.parse_nested_block(|input| {
                        step_count = try!(specified::parse_integer(input));
                        try!(input.expect_comma());
                        start_end = try!(match_ignore_ascii_case! {
                            try!(input.expect_ident()),
                            "start" => Ok(computed_value::StartEnd::Start),
                            "end" => Ok(computed_value::StartEnd::End),
                            _ => Err(())
                        });
                        Ok(())
                    }));
                    Ok(TransitionTimingFunction::Steps(step_count as u32, start_end))
                },
                _ => Err(())
            }
        }
        match_ignore_ascii_case! {
            try!(input.expect_ident()),
            "ease" => Ok(EASE),
            "linear" => Ok(LINEAR),
            "ease-in" => Ok(EASE_IN),
            "ease-out" => Ok(EASE_OUT),
            "ease-in-out" => Ok(EASE_IN_OUT),
            "step-start" => Ok(STEP_START),
            "step-end" => Ok(STEP_END),
            _ => Err(())
        }
    }

    pub fn parse(_: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue,()> {
        Ok(SpecifiedValue(try!(input.parse_comma_separated(parse_one))))
    }
</%helpers:longhand>

// TODO(pcwalton): Lots more properties.
<%helpers:longhand name="transition-property">
    use self::computed_value::TransitionProperty;

    pub use self::computed_value::SingleComputedValue as SingleSpecifiedValue;
    pub use self::computed_value::T as SpecifiedValue;

    pub mod computed_value {
        use cssparser::ToCss;
        use std::fmt;

        pub use self::TransitionProperty as SingleComputedValue;

        #[derive(Copy, Clone, Debug, PartialEq)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub enum TransitionProperty {
            All,
            BackgroundColor,
            BackgroundPosition,
            BorderBottomColor,
            BorderBottomWidth,
            BorderLeftColor,
            BorderLeftWidth,
            BorderRightColor,
            BorderRightWidth,
            BorderSpacing,
            BorderTopColor,
            BorderTopWidth,
            Bottom,
            Color,
            Clip,
            FontSize,
            FontWeight,
            Height,
            Left,
            LetterSpacing,
            LineHeight,
            MarginBottom,
            MarginLeft,
            MarginRight,
            MarginTop,
            MaxHeight,
            MaxWidth,
            MinHeight,
            MinWidth,
            Opacity,
            OutlineColor,
            OutlineWidth,
            PaddingBottom,
            PaddingLeft,
            PaddingRight,
            PaddingTop,
            Right,
            TextIndent,
            TextShadow,
            Top,
            Transform,
            VerticalAlign,
            Visibility,
            Width,
            WordSpacing,
            ZIndex,
        }

        pub static ALL_TRANSITION_PROPERTIES: [TransitionProperty; 45] = [
            TransitionProperty::BackgroundColor,
            TransitionProperty::BackgroundPosition,
            TransitionProperty::BorderBottomColor,
            TransitionProperty::BorderBottomWidth,
            TransitionProperty::BorderLeftColor,
            TransitionProperty::BorderLeftWidth,
            TransitionProperty::BorderRightColor,
            TransitionProperty::BorderRightWidth,
            TransitionProperty::BorderSpacing,
            TransitionProperty::BorderTopColor,
            TransitionProperty::BorderTopWidth,
            TransitionProperty::Bottom,
            TransitionProperty::Color,
            TransitionProperty::Clip,
            TransitionProperty::FontSize,
            TransitionProperty::FontWeight,
            TransitionProperty::Height,
            TransitionProperty::Left,
            TransitionProperty::LetterSpacing,
            TransitionProperty::LineHeight,
            TransitionProperty::MarginBottom,
            TransitionProperty::MarginLeft,
            TransitionProperty::MarginRight,
            TransitionProperty::MarginTop,
            TransitionProperty::MaxHeight,
            TransitionProperty::MaxWidth,
            TransitionProperty::MinHeight,
            TransitionProperty::MinWidth,
            TransitionProperty::Opacity,
            TransitionProperty::OutlineColor,
            TransitionProperty::OutlineWidth,
            TransitionProperty::PaddingBottom,
            TransitionProperty::PaddingLeft,
            TransitionProperty::PaddingRight,
            TransitionProperty::PaddingTop,
            TransitionProperty::Right,
            TransitionProperty::TextIndent,
            TransitionProperty::TextShadow,
            TransitionProperty::Top,
            TransitionProperty::Transform,
            TransitionProperty::VerticalAlign,
            TransitionProperty::Visibility,
            TransitionProperty::Width,
            TransitionProperty::WordSpacing,
            TransitionProperty::ZIndex,
        ];

        impl ToCss for TransitionProperty {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                match *self {
                    TransitionProperty::All => dest.write_str("all"),
                    TransitionProperty::BackgroundColor => dest.write_str("background-color"),
                    TransitionProperty::BackgroundPosition => dest.write_str("background-position"),
                    TransitionProperty::BorderBottomColor => dest.write_str("border-bottom-color"),
                    TransitionProperty::BorderBottomWidth => dest.write_str("border-bottom-width"),
                    TransitionProperty::BorderLeftColor => dest.write_str("border-left-color"),
                    TransitionProperty::BorderLeftWidth => dest.write_str("border-left-width"),
                    TransitionProperty::BorderRightColor => dest.write_str("border-right-color"),
                    TransitionProperty::BorderRightWidth => dest.write_str("border-right-width"),
                    TransitionProperty::BorderSpacing => dest.write_str("border-spacing"),
                    TransitionProperty::BorderTopColor => dest.write_str("border-top-color"),
                    TransitionProperty::BorderTopWidth => dest.write_str("border-top-width"),
                    TransitionProperty::Bottom => dest.write_str("bottom"),
                    TransitionProperty::Color => dest.write_str("color"),
                    TransitionProperty::Clip => dest.write_str("clip"),
                    TransitionProperty::FontSize => dest.write_str("font-size"),
                    TransitionProperty::FontWeight => dest.write_str("font-weight"),
                    TransitionProperty::Height => dest.write_str("height"),
                    TransitionProperty::Left => dest.write_str("left"),
                    TransitionProperty::LetterSpacing => dest.write_str("letter-spacing"),
                    TransitionProperty::LineHeight => dest.write_str("line-height"),
                    TransitionProperty::MarginBottom => dest.write_str("margin-bottom"),
                    TransitionProperty::MarginLeft => dest.write_str("margin-left"),
                    TransitionProperty::MarginRight => dest.write_str("margin-right"),
                    TransitionProperty::MarginTop => dest.write_str("margin-top"),
                    TransitionProperty::MaxHeight => dest.write_str("max-height"),
                    TransitionProperty::MaxWidth => dest.write_str("max-width"),
                    TransitionProperty::MinHeight => dest.write_str("min-height"),
                    TransitionProperty::MinWidth => dest.write_str("min-width"),
                    TransitionProperty::Opacity => dest.write_str("opacity"),
                    TransitionProperty::OutlineColor => dest.write_str("outline-color"),
                    TransitionProperty::OutlineWidth => dest.write_str("outline-width"),
                    TransitionProperty::PaddingBottom => dest.write_str("padding-bottom"),
                    TransitionProperty::PaddingLeft => dest.write_str("padding-left"),
                    TransitionProperty::PaddingRight => dest.write_str("padding-right"),
                    TransitionProperty::PaddingTop => dest.write_str("padding-top"),
                    TransitionProperty::Right => dest.write_str("right"),
                    TransitionProperty::TextIndent => dest.write_str("text-indent"),
                    TransitionProperty::TextShadow => dest.write_str("text-shadow"),
                    TransitionProperty::Top => dest.write_str("top"),
                    TransitionProperty::Transform => dest.write_str("transform"),
                    TransitionProperty::VerticalAlign => dest.write_str("vertical-align"),
                    TransitionProperty::Visibility => dest.write_str("visibility"),
                    TransitionProperty::Width => dest.write_str("width"),
                    TransitionProperty::WordSpacing => dest.write_str("word-spacing"),
                    TransitionProperty::ZIndex => dest.write_str("z-index"),
                }
            }
        }

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

    pub fn parse_one(input: &mut Parser) -> Result<SingleSpecifiedValue,()> {
        match_ignore_ascii_case! {
            try!(input.expect_ident()),
            "all" => Ok(TransitionProperty::All),
            "background-color" => Ok(TransitionProperty::BackgroundColor),
            "background-position" => Ok(TransitionProperty::BackgroundPosition),
            "border-bottom-color" => Ok(TransitionProperty::BorderBottomColor),
            "border-bottom-width" => Ok(TransitionProperty::BorderBottomWidth),
            "border-left-color" => Ok(TransitionProperty::BorderLeftColor),
            "border-left-width" => Ok(TransitionProperty::BorderLeftWidth),
            "border-right-color" => Ok(TransitionProperty::BorderRightColor),
            "border-right-width" => Ok(TransitionProperty::BorderRightWidth),
            "border-spacing" => Ok(TransitionProperty::BorderSpacing),
            "border-top-color" => Ok(TransitionProperty::BorderTopColor),
            "border-top-width" => Ok(TransitionProperty::BorderTopWidth),
            "bottom" => Ok(TransitionProperty::Bottom),
            "color" => Ok(TransitionProperty::Color),
            "clip" => Ok(TransitionProperty::Clip),
            "font-size" => Ok(TransitionProperty::FontSize),
            "font-weight" => Ok(TransitionProperty::FontWeight),
            "height" => Ok(TransitionProperty::Height),
            "left" => Ok(TransitionProperty::Left),
            "letter-spacing" => Ok(TransitionProperty::LetterSpacing),
            "line-height" => Ok(TransitionProperty::LineHeight),
            "margin-bottom" => Ok(TransitionProperty::MarginBottom),
            "margin-left" => Ok(TransitionProperty::MarginLeft),
            "margin-right" => Ok(TransitionProperty::MarginRight),
            "margin-top" => Ok(TransitionProperty::MarginTop),
            "max-height" => Ok(TransitionProperty::MaxHeight),
            "max-width" => Ok(TransitionProperty::MaxWidth),
            "min-height" => Ok(TransitionProperty::MinHeight),
            "min-width" => Ok(TransitionProperty::MinWidth),
            "opacity" => Ok(TransitionProperty::Opacity),
            "outline-color" => Ok(TransitionProperty::OutlineColor),
            "outline-width" => Ok(TransitionProperty::OutlineWidth),
            "padding-bottom" => Ok(TransitionProperty::PaddingBottom),
            "padding-left" => Ok(TransitionProperty::PaddingLeft),
            "padding-right" => Ok(TransitionProperty::PaddingRight),
            "padding-top" => Ok(TransitionProperty::PaddingTop),
            "right" => Ok(TransitionProperty::Right),
            "text-indent" => Ok(TransitionProperty::TextIndent),
            "text-shadow" => Ok(TransitionProperty::TextShadow),
            "top" => Ok(TransitionProperty::Top),
            "transform" => Ok(TransitionProperty::Transform),
            "vertical-align" => Ok(TransitionProperty::VerticalAlign),
            "visibility" => Ok(TransitionProperty::Visibility),
            "width" => Ok(TransitionProperty::Width),
            "word-spacing" => Ok(TransitionProperty::WordSpacing),
            "z-index" => Ok(TransitionProperty::ZIndex),
            _ => Err(())
        }
    }

    pub fn parse(_: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue,()> {
        Ok(SpecifiedValue(try!(input.parse_comma_separated(parse_one))))
    }

    impl ToComputedValue for SpecifiedValue {
        type ComputedValue = computed_value::T;

        #[inline]
        fn to_computed_value<Cx: TContext>(&self, _: &Cx) -> computed_value::T {
            (*self).clone()
        }
    }
</%helpers:longhand>

<%helpers:longhand name="transition-delay">
    pub use properties::longhands::transition_duration::{SingleSpecifiedValue, SpecifiedValue};
    pub use properties::longhands::transition_duration::{computed_value};
    pub use properties::longhands::transition_duration::{get_initial_single_value};
    pub use properties::longhands::transition_duration::{get_initial_value, parse, parse_one};
</%helpers:longhand>

// CSSOM View Module
// https://www.w3.org/TR/cssom-view-1/
${helpers.single_keyword("scroll-behavior",
                         "auto smooth",
                         products="gecko")}

// Non-standard: https://developer.mozilla.org/en-US/docs/Web/CSS/scroll-snap-type-x
${helpers.single_keyword("scroll-snap-type-x",
                         "none mandatory proximity",
                         products="gecko",
                         gecko_constant_prefix="NS_STYLE_SCROLL_SNAP_TYPE")}

// Non-standard: https://developer.mozilla.org/en-US/docs/Web/CSS/scroll-snap-type-y
${helpers.single_keyword("scroll-snap-type-y",
                         "none mandatory proximity",
                         products="gecko",
                         gecko_constant_prefix="NS_STYLE_SCROLL_SNAP_TYPE")}

// Compositing and Blending Level 1
// http://www.w3.org/TR/compositing-1/
${helpers.single_keyword("isolation",
                         "auto isolate",
                         products="gecko")}

${helpers.single_keyword("page-break-after",
                         "auto always avoid left right",
                         products="gecko")}
${helpers.single_keyword("page-break-before",
                         "auto always avoid left right",
                         products="gecko")}
${helpers.single_keyword("page-break-inside",
                         "auto avoid",
                         products="gecko",
                         gecko_ffi_name="mBreakInside",
                         gecko_constant_prefix="NS_STYLE_PAGE_BREAK")}

// CSS Basic User Interface Module Level 3
// http://dev.w3.org/csswg/css-ui/
${helpers.single_keyword("resize",
                         "none both horizontal vertical",
                         products="gecko")}

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
                         products="gecko")}

// Non-standard: https://developer.mozilla.org/en-US/docs/Web/CSS/-moz-binding
<%helpers:longhand name="-moz-binding" products="gecko">
    use cssparser::{CssStringWriter, ToCss};
    use gecko_bindings::ptr::{GeckoArcPrincipal, GeckoArcURI};
    use std::fmt::{self, Write};
    use url::Url;
    use values::computed::ComputedValueAsSpecified;

    #[derive(PartialEq, Clone, Debug)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub struct UrlExtraData {
        pub base: GeckoArcURI,
        pub referrer: GeckoArcURI,
        pub principal: GeckoArcPrincipal,
    }

    #[derive(PartialEq, Clone, Debug)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub enum SpecifiedValue {
        Url(Url, UrlExtraData),
        None,
    }

    impl ComputedValueAsSpecified for SpecifiedValue {}

    impl ToCss for SpecifiedValue {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            use values::LocalToCss;
            match *self {
                SpecifiedValue::Url(ref url, _) => {
                    url.to_css(dest)
                }
                SpecifiedValue::None => {
                    try!(dest.write_str("none"));
                    Ok(())
                }
            }
        }
    }

    pub mod computed_value {
        pub type T = super::SpecifiedValue;
    }

    #[inline] pub fn get_initial_value() -> SpecifiedValue {
        SpecifiedValue::None
    }

    pub fn parse(context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
        if input.try(|input| input.expect_ident_matching("none")).is_ok() {
            return Ok(SpecifiedValue::None);
        }

        let url = context.parse_url(&*try!(input.expect_url()));
        match context.extra_data {
            ParserContextExtraData {
                base: Some(ref base),
                referrer: Some(ref referrer),
                principal: Some(ref principal),
            } => {
                let extra_data = UrlExtraData {
                    base: base.clone(),
                    referrer: referrer.clone(),
                    principal: principal.clone(),
                };
                Ok(SpecifiedValue::Url(url, extra_data))
            },
            _ => {
                // FIXME(heycam) should ensure we always have a principal, etc., when parsing
                // style attributes and re-parsing due to CSS Variables.
                println!("stylo: skipping -moz-binding declaration without ParserContextExtraData");
                Err(())
            },
        }
    }
</%helpers:longhand>
