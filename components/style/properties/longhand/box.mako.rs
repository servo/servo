/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />
<% from data import Keyword, Method, to_rust_ident, to_camel_case%>

<% data.new_style_struct("Box",
                         inherited=False,
                         gecko_name="Display") %>

// TODO(SimonSapin): don't parse `inline-table`, since we don't support it
<%helpers:longhand name="display"
                   need_clone="True"
                   animatable="False"
                   custom_cascade="${product == 'servo'}"
                   spec="https://drafts.csswg.org/css-display/#propdef-display">
    <%
        values = """inline block inline-block
            table inline-table table-row-group table-header-group table-footer-group
            table-row table-column-group table-column table-cell table-caption
            list-item flex inline-flex
            none
        """.split()
        if product == "gecko":
            values += """grid inline-grid ruby ruby-base ruby-base-container
                ruby-text ruby-text-container contents flow-root -webkit-box
                -webkit-inline-box -moz-box -moz-inline-box -moz-grid -moz-inline-grid
                -moz-grid-group -moz-grid-line -moz-stack -moz-inline-stack -moz-deck
                -moz-popup -moz-groupbox""".split()
    %>
    use values::computed::ComputedValueAsSpecified;
    use style_traits::ToCss;
    use values::HasViewportPercentage;
    no_viewport_percentage!(SpecifiedValue);

    pub mod computed_value {
        pub use super::SpecifiedValue as T;
    }

    #[allow(non_camel_case_types)]
    #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf, Deserialize, Serialize))]
    pub enum SpecifiedValue {
        % for value in values:
            ${to_rust_ident(value)},
        % endfor
    }

    impl ToCss for SpecifiedValue {
        fn to_css<W>(&self, dest: &mut W) -> ::std::fmt::Result
            where W: ::std::fmt::Write,
        {
            match *self {
                % for value in values:
                    SpecifiedValue::${to_rust_ident(value)} => dest.write_str("${value}"),
                % endfor
            }
        }
    }

    /// The initial display value.
    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        computed_value::T::${to_rust_ident(values[0])}
    }

    /// Parse a display value.
    pub fn parse(_context: &ParserContext, input: &mut Parser)
                 -> Result<SpecifiedValue, ()> {
        match_ignore_ascii_case! { &try!(input.expect_ident()),
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
                                   _cacheable: &mut bool,
                                   _error_reporter: &ParseErrorReporter) {
            longhands::_servo_display_for_hypothetical_box::derive_from_display(context);
            longhands::_servo_text_decorations_in_effect::derive_from_display(context);
            longhands::_servo_under_display_none::derive_from_display(context);
        }
    % endif

    ${helpers.gecko_keyword_conversion(Keyword('display', ' '.join(values),
                                               gecko_enum_prefix='StyleDisplay'))}

</%helpers:longhand>

${helpers.single_keyword("-moz-top-layer", "none top",
                         gecko_constant_prefix="NS_STYLE_TOP_LAYER",
                         gecko_ffi_name="mTopLayer", need_clone=True,
                         products="gecko", animatable=False, internal=True,
                         spec="Internal (not web-exposed)")}

${helpers.single_keyword("position", "static absolute relative fixed",
                         need_clone="True",
                         extra_gecko_values="sticky",
                         animatable="False",
                         creates_stacking_context="True",
                         abspos_cb="True",
                         spec="https://drafts.csswg.org/css-position/#position-property")}

<%helpers:single_keyword_computed name="float"
                                  values="none left right"
                                  // https://drafts.csswg.org/css-logical-props/#float-clear
                                  extra_specified="inline-start inline-end"
                                  needs_conversion="True"
                                  animatable="False"
                                  need_clone="True"
                                  gecko_enum_prefix="StyleFloat"
                                  gecko_inexhaustive="True"
                                  gecko_ffi_name="mFloat"
                                  spec="https://drafts.csswg.org/css-box/#propdef-float">
    use values::HasViewportPercentage;
    no_viewport_percentage!(SpecifiedValue);
    impl ToComputedValue for SpecifiedValue {
        type ComputedValue = computed_value::T;

        #[inline]
        fn to_computed_value(&self, context: &Context) -> computed_value::T {
            let ltr = context.style().writing_mode.is_bidi_ltr();
            // https://drafts.csswg.org/css-logical-props/#float-clear
            match *self {
                SpecifiedValue::inline_start if ltr => computed_value::T::left,
                SpecifiedValue::inline_start => computed_value::T::right,
                SpecifiedValue::inline_end if ltr => computed_value::T::right,
                SpecifiedValue::inline_end => computed_value::T::left,
                % for value in "none left right".split():
                    SpecifiedValue::${value} => computed_value::T::${value},
                % endfor
            }
        }
        #[inline]
        fn from_computed_value(computed: &computed_value::T) -> SpecifiedValue {
            match *computed {
                % for value in "none left right".split():
                    computed_value::T::${value} => SpecifiedValue::${value},
                % endfor
            }
        }
    }
</%helpers:single_keyword_computed>

<%helpers:single_keyword_computed name="clear"
                                  values="none left right both"
                                  // https://drafts.csswg.org/css-logical-props/#float-clear
                                  extra_specified="inline-start inline-end"
                                  needs_conversion="True"
                                  animatable="False"
                                  gecko_enum_prefix="StyleClear"
                                  gecko_ffi_name="mBreakType"
                                  spec="https://www.w3.org/TR/CSS2/visuren.html#flow-control">
    use values::HasViewportPercentage;
    no_viewport_percentage!(SpecifiedValue);
    impl ToComputedValue for SpecifiedValue {
        type ComputedValue = computed_value::T;

        #[inline]
        fn to_computed_value(&self, context: &Context) -> computed_value::T {
            let ltr = context.style().writing_mode.is_bidi_ltr();
            // https://drafts.csswg.org/css-logical-props/#float-clear
            match *self {
                SpecifiedValue::inline_start if ltr => computed_value::T::left,
                SpecifiedValue::inline_start => computed_value::T::right,
                SpecifiedValue::inline_end if ltr => computed_value::T::right,
                SpecifiedValue::inline_end => computed_value::T::left,
                % for value in "none left right both".split():
                    SpecifiedValue::${value} => computed_value::T::${value},
                % endfor
            }
        }
        #[inline]
        fn from_computed_value(computed: &computed_value::T) -> SpecifiedValue {
            match *computed {
                % for value in "none left right both".split():
                    computed_value::T::${value} => SpecifiedValue::${value},
                % endfor
            }
        }
    }
</%helpers:single_keyword_computed>

<%helpers:longhand name="-servo-display-for-hypothetical-box"
                   animatable="False"
                   derived_from="display"
                   products="servo"
                   spec="Internal (not web-exposed)">
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

<%helpers:longhand name="vertical-align" animatable="True"
                   spec="https://www.w3.org/TR/CSS2/visudet.html#propdef-vertical-align">
    use std::fmt;
    use style_traits::ToCss;
    use values::HasViewportPercentage;

    <% vertical_align = data.longhands_by_name["vertical-align"] %>
    <% vertical_align.keyword = Keyword("vertical-align",
                                        "baseline sub super top text-top middle bottom text-bottom",
                                        extra_gecko_values="middle-with-baseline") %>
    <% vertical_align_keywords = vertical_align.keyword.values_for(product) %>

    ${helpers.gecko_keyword_conversion(vertical_align.keyword)}

    impl HasViewportPercentage for SpecifiedValue {
        fn has_viewport_percentage(&self) -> bool {
            match *self {
                SpecifiedValue::LengthOrPercentage(ref length) => length.has_viewport_percentage(),
                _ => false
            }
        }
    }

    /// The `vertical-align` value.
    #[allow(non_camel_case_types)]
    #[derive(Debug, Clone, PartialEq)]
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
                SpecifiedValue::LengthOrPercentage(ref value) => value.to_css(dest),
            }
        }
    }
    /// baseline | sub | super | top | text-top | middle | bottom | text-bottom
    /// | <percentage> | <length>
    pub fn parse(context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
        input.try(|i| specified::LengthOrPercentage::parse(context, i))
        .map(SpecifiedValue::LengthOrPercentage)
        .or_else(|_| {
            match_ignore_ascii_case! { &try!(input.expect_ident()),
                % for keyword in vertical_align_keywords:
                    "${keyword}" => Ok(SpecifiedValue::${to_rust_ident(keyword)}),
                % endfor
                _ => Err(())
            }
        })
    }

    /// The computed value for `vertical-align`.
    pub mod computed_value {
        use app_units::Au;
        use std::fmt;
        use style_traits::ToCss;
        use values::{CSSFloat, computed};

        /// The keywords are the same, and the `LengthOrPercentage` is computed
        /// here.
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
                    T::LengthOrPercentage(ref value) => value.to_css(dest),
                }
            }
        }
    }

    /// The initial computed value for `vertical-align`.
    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        computed_value::T::baseline
    }

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
                SpecifiedValue::LengthOrPercentage(ref value) =>
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

${helpers.single_keyword("-servo-overflow-clip-box", "padding-box content-box",
    products="servo", animatable=False, internal=True,
    spec="Internal, not web-exposed, \
          may be standardized in the future (https://developer.mozilla.org/en-US/docs/Web/CSS/overflow-clip-box)")}

${helpers.single_keyword("overflow-clip-box", "padding-box content-box",
    products="gecko", animatable=False, internal=True,
    spec="Internal, not web-exposed, \
          may be standardized in the future (https://developer.mozilla.org/en-US/docs/Web/CSS/overflow-clip-box)")}

// FIXME(pcwalton, #2742): Implement scrolling for `scroll` and `auto`.
${helpers.single_keyword("overflow-x", "visible hidden scroll auto",
                         need_clone=True, animatable=False,
                         gecko_constant_prefix="NS_STYLE_OVERFLOW",
                         spec="https://drafts.csswg.org/css-overflow/#propdef-overflow-x")}

// FIXME(pcwalton, #2742): Implement scrolling for `scroll` and `auto`.
<%helpers:longhand name="overflow-y" need_clone="True" animatable="False"
                   spec="https://drafts.csswg.org/css-overflow/#propdef-overflow-y">
    use super::overflow_x;

    use std::fmt;
    use style_traits::ToCss;
    use values::computed::ComputedValueAsSpecified;
    use values::HasViewportPercentage;

    no_viewport_percentage!(SpecifiedValue);

    impl ToCss for SpecifiedValue {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            self.0.to_css(dest)
        }
    }

    /// The specified and computed value for overflow-y is a wrapper on top of
    /// `overflow-x`, so we re-use the logic, but prevent errors from mistakenly
    /// assign one to other.
    ///
    /// TODO(Manishearth, emilio): We may want to just use the same value.
    pub mod computed_value {
        pub use super::SpecifiedValue as T;
    }

    #[derive(Debug, Clone, Copy, PartialEq)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub struct SpecifiedValue(pub super::overflow_x::SpecifiedValue);

    impl ComputedValueAsSpecified for SpecifiedValue {}

    #[inline]
    #[allow(missing_docs)]
    pub fn get_initial_value() -> computed_value::T {
        computed_value::T(overflow_x::get_initial_value())
    }

    #[inline]
    #[allow(missing_docs)]
    pub fn parse(context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue,()> {
        overflow_x::parse(context, input).map(SpecifiedValue)
    }
</%helpers:longhand>

<%helpers:vector_longhand name="transition-duration"
                          need_index="True"
                          animatable="False"
                          extra_prefixes="moz webkit"
                          spec="https://drafts.csswg.org/css-transitions/#propdef-transition-duration">
    use values::specified::Time;

    pub use values::specified::Time as SpecifiedValue;
    use values::HasViewportPercentage;
    no_viewport_percentage!(SpecifiedValue);

    pub mod computed_value {
        pub use values::computed::Time as T;
    }

    #[inline]
    pub fn get_initial_value() -> Time {
        Time(0.0)
    }

    #[inline]
    pub fn get_initial_specified_value() -> SpecifiedValue {
        SpecifiedValue(0.0)
    }

    pub fn parse(context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue,()> {
        Time::parse(context, input)
    }
</%helpers:vector_longhand>

// TODO(pcwalton): Lots more timing functions.
<%helpers:vector_longhand name="transition-timing-function"
                          need_index="True"
                          animatable="False"
                          extra_prefixes="moz webkit"
                          spec="https://drafts.csswg.org/css-transitions/#propdef-transition-timing-function">
    use self::computed_value::StartEnd;

    use euclid::point::{Point2D, TypedPoint2D};
    use std::fmt;
    use std::marker::PhantomData;
    use style_traits::ToCss;

    // FIXME: This could use static variables and const functions when they are available.
    #[inline(always)]
    fn ease() -> computed_value::T {
        computed_value::T::CubicBezier(TypedPoint2D::new(0.25, 0.1),
                                              TypedPoint2D::new(0.25, 1.0))
    }

    #[inline(always)]
    fn linear() -> computed_value::T {
        computed_value::T::CubicBezier(TypedPoint2D::new(0.0, 0.0),
                                              TypedPoint2D::new(1.0, 1.0))
    }

    #[inline(always)]
    fn ease_in() -> computed_value::T {
        computed_value::T::CubicBezier(TypedPoint2D::new(0.42, 0.0),
                                              TypedPoint2D::new(1.0, 1.0))
    }

    #[inline(always)]
    fn ease_out() -> computed_value::T {
        computed_value::T::CubicBezier(TypedPoint2D::new(0.0, 0.0),
                                              TypedPoint2D::new(0.58, 1.0))
    }

    #[inline(always)]
    fn ease_in_out() -> computed_value::T {
        computed_value::T::CubicBezier(TypedPoint2D::new(0.42, 0.0),
                                              TypedPoint2D::new(0.58, 1.0))
    }

    static STEP_START: computed_value::T =
        computed_value::T::Steps(1, StartEnd::Start);
    static STEP_END: computed_value::T =
        computed_value::T::Steps(1, StartEnd::End);

    pub mod computed_value {
        use euclid::point::Point2D;
        use parser::{Parse, ParserContext};
        use std::fmt;
        use style_traits::ToCss;

        pub use super::parse;

        #[derive(Copy, Clone, Debug, PartialEq)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub enum T {
            CubicBezier(Point2D<f32>, Point2D<f32>),
            Steps(u32, StartEnd),
        }

        impl ToCss for T {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                match *self {
                    T::CubicBezier(p1, p2) => {
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
                    T::Steps(steps, start_end) => {
                        super::serialize_steps(dest, steps, start_end)
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
    }

    define_css_keyword_enum!(FunctionKeyword:
                             "ease" => Ease,
                             "linear" => Linear,
                             "ease-in" => EaseIn,
                             "ease-out" => EaseOut,
                             "ease-in-out" => EaseInOut,
                             "step-start" => StepStart,
                             "step-end" => StepEnd);

    #[derive(Copy, Clone, Debug, PartialEq)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub enum SpecifiedValue {
        CubicBezier(Point2D<f32>, Point2D<f32>),
        Steps(u32, StartEnd),
        Keyword(FunctionKeyword),
    }

    impl Parse for SpecifiedValue {
        fn parse(_context: &ParserContext, input: &mut ::cssparser::Parser) -> Result<Self, ()> {
            if let Ok(function_name) = input.try(|input| input.expect_function()) {
                return match_ignore_ascii_case! { &function_name,
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
                        if p1x < 0.0 || p1x > 1.0 || p2x < 0.0 || p2x > 1.0 {
                            return Err(())
                        }

                        let (p1, p2) = (Point2D::new(p1x, p1y), Point2D::new(p2x, p2y));
                        Ok(SpecifiedValue::CubicBezier(p1, p2))
                    },
                    "steps" => {
                        let (mut step_count, mut start_end) = (0, StartEnd::End);
                        try!(input.parse_nested_block(|input| {
                            step_count = try!(specified::parse_integer(input));
                            if step_count < 1 {
                                return Err(())
                            }

                            if input.try(|input| input.expect_comma()).is_ok() {
                                start_end = try!(match_ignore_ascii_case! {
                                    &try!(input.expect_ident()),
                                    "start" => Ok(StartEnd::Start),
                                    "end" => Ok(StartEnd::End),
                                    _ => Err(())
                                });
                            }
                            Ok(())
                        }));
                        Ok(SpecifiedValue::Steps(step_count as u32, start_end))
                    },
                    _ => Err(())
                }
            }
            Ok(SpecifiedValue::Keyword(try!(FunctionKeyword::parse(input))))
        }
    }

    fn serialize_steps<W>(dest: &mut W, steps: u32,
                          start_end: StartEnd) -> fmt::Result where W: fmt::Write {
        try!(dest.write_str("steps("));
        try!(steps.to_css(dest));
        if let StartEnd::Start = start_end {
            try!(dest.write_str(", start"));
        }
        dest.write_str(")")
    }

    // https://drafts.csswg.org/css-transitions/#serializing-a-timing-function
    impl ToCss for SpecifiedValue {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match *self {
                SpecifiedValue::CubicBezier(p1, p2) => {
                    try!(dest.write_str("cubic-bezier("));
                    try!(p1.x.to_css(dest));
                    try!(dest.write_str(", "));
                    try!(p1.y.to_css(dest));
                    try!(dest.write_str(", "));
                    try!(p2.x.to_css(dest));
                    try!(dest.write_str(", "));
                    try!(p2.y.to_css(dest));
                    dest.write_str(")")
                },
                SpecifiedValue::Steps(steps, start_end) => {
                    serialize_steps(dest, steps, start_end)
                },
                SpecifiedValue::Keyword(keyword) => {
                    match keyword {
                        FunctionKeyword::StepStart => {
                            serialize_steps(dest, 1, StartEnd::Start)
                        },
                        FunctionKeyword::StepEnd => {
                            serialize_steps(dest, 1, StartEnd::End)
                        },
                        _ => {
                            keyword.to_css(dest)
                        },
                    }
                },
            }
        }
    }

    impl ToComputedValue for SpecifiedValue {
        type ComputedValue = computed_value::T;

        #[inline]
        fn to_computed_value(&self, _context: &Context) -> computed_value::T {
            match *self {
                SpecifiedValue::CubicBezier(p1, p2) => {
                    computed_value::T::CubicBezier(p1, p2)
                },
                SpecifiedValue::Steps(count, start_end) => {
                    computed_value::T::Steps(count, start_end)
                },
                SpecifiedValue::Keyword(keyword) => {
                    match keyword {
                        FunctionKeyword::Ease => ease(),
                        FunctionKeyword::Linear => linear(),
                        FunctionKeyword::EaseIn => ease_in(),
                        FunctionKeyword::EaseOut => ease_out(),
                        FunctionKeyword::EaseInOut => ease_in_out(),
                        FunctionKeyword::StepStart => STEP_START,
                        FunctionKeyword::StepEnd => STEP_END,
                    }
                },
            }
        }
        #[inline]
        fn from_computed_value(computed: &computed_value::T) -> Self {
            match *computed {
                computed_value::T::CubicBezier(p1, p2) => {
                    SpecifiedValue::CubicBezier(p1, p2)
                },
                computed_value::T::Steps(count, start_end) => {
                    SpecifiedValue::Steps(count, start_end)
                },
            }
        }
    }

    use values::HasViewportPercentage;
    no_viewport_percentage!(SpecifiedValue);

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        ease()
    }

    #[inline]
    pub fn get_initial_specified_value() -> SpecifiedValue {
        SpecifiedValue::Keyword(FunctionKeyword::Ease)
    }

    pub fn parse(context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue,()> {
        SpecifiedValue::parse(context, input)
    }
</%helpers:vector_longhand>

<%helpers:vector_longhand name="transition-property"
                          allow_empty="True"
                          need_index="True"
                          animatable="False"
                          extra_prefixes="moz webkit"
                          spec="https://drafts.csswg.org/css-transitions/#propdef-transition-property">

    use values::computed::ComputedValueAsSpecified;

    pub use properties::animated_properties::TransitionProperty;
    pub use properties::animated_properties::TransitionProperty as SpecifiedValue;

    pub mod computed_value {
        use std::fmt;
        use style_traits::ToCss;
        // NB: Can't generate the type here because it needs all the longhands
        // generated beforehand.
        pub use super::SpecifiedValue as T;
    }

    pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue,()> {
        SpecifiedValue::parse(input)
    }

    use values::HasViewportPercentage;
    no_viewport_percentage!(SpecifiedValue);

    impl ComputedValueAsSpecified for SpecifiedValue { }
</%helpers:vector_longhand>

<%helpers:vector_longhand name="transition-delay"
                          need_index="True"
                          animatable="False"
                          extra_prefixes="moz webkit"
                          spec="https://drafts.csswg.org/css-transitions/#propdef-transition-delay">
    pub use properties::longhands::transition_duration::single_value::SpecifiedValue;
    pub use properties::longhands::transition_duration::single_value::computed_value;
    pub use properties::longhands::transition_duration::single_value::{get_initial_value, get_initial_specified_value};
    pub use properties::longhands::transition_duration::single_value::parse;
</%helpers:vector_longhand>

<%helpers:vector_longhand name="animation-name"
                          need_index="True"
                          animatable="False",
                          extra_prefixes="moz webkit"
                          allowed_in_keyframe_block="False"
                          spec="https://drafts.csswg.org/css-animations/#propdef-animation-name">
    use Atom;
    use std::fmt;
    use std::ops::Deref;
    use style_traits::ToCss;
    use values::computed::ComputedValueAsSpecified;
    use values::HasViewportPercentage;

    pub mod computed_value {
        pub use super::SpecifiedValue as T;
    }

    #[derive(Clone, Debug, Hash, Eq, PartialEq)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub struct SpecifiedValue(pub Atom);

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        get_initial_specified_value()
    }

    #[inline]
    pub fn get_initial_specified_value() -> SpecifiedValue {
        SpecifiedValue(atom!(""))
    }

    impl fmt::Display for SpecifiedValue {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            self.0.fmt(f)
        }
    }

    impl ToCss for SpecifiedValue {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            if self.0 == atom!("") {
                dest.write_str("none")
            } else {
                dest.write_str(&*self.0.to_string())
            }
        }
    }

    impl Parse for SpecifiedValue {
        fn parse(_context: &ParserContext, input: &mut ::cssparser::Parser) -> Result<Self, ()> {
            use cssparser::Token;
            Ok(match input.next() {
                Ok(Token::Ident(ref value)) => SpecifiedValue(if value == "none" {
                    // FIXME We may want to support `@keyframes ""` at some point.
                    atom!("")
                } else {
                    Atom::from(&**value)
                }),
                Ok(Token::QuotedString(value)) => SpecifiedValue(Atom::from(&*value)),
                _ => return Err(()),
            })
        }
    }
    no_viewport_percentage!(SpecifiedValue);

    pub fn parse(context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue,()> {
        SpecifiedValue::parse(context, input)
    }

    impl ComputedValueAsSpecified for SpecifiedValue {}
</%helpers:vector_longhand>

<%helpers:vector_longhand name="animation-duration"
                          need_index="True"
                          animatable="False",
                          extra_prefixes="moz webkit"
                          spec="https://drafts.csswg.org/css-animations/#propdef-animation-duration",
                          allowed_in_keyframe_block="False">
    pub use properties::longhands::transition_duration::single_value::computed_value;
    pub use properties::longhands::transition_duration::single_value::get_initial_specified_value;
    pub use properties::longhands::transition_duration::single_value::{get_initial_value, parse};
    pub use properties::longhands::transition_duration::single_value::SpecifiedValue;
</%helpers:vector_longhand>

<%helpers:vector_longhand name="animation-timing-function"
                          need_index="True"
                          animatable="False",
                          extra_prefixes="moz webkit"
                          spec="https://drafts.csswg.org/css-animations/#propdef-animation-timing-function",
                          allowed_in_keyframe_block="True">
    pub use properties::longhands::transition_timing_function::single_value::computed_value;
    pub use properties::longhands::transition_timing_function::single_value::get_initial_value;
    pub use properties::longhands::transition_timing_function::single_value::get_initial_specified_value;
    pub use properties::longhands::transition_timing_function::single_value::parse;
    pub use properties::longhands::transition_timing_function::single_value::SpecifiedValue;
</%helpers:vector_longhand>

<%helpers:vector_longhand name="animation-iteration-count"
                          need_index="True"
                          animatable="False",
                          extra_prefixes="moz webkit"
                          spec="https://drafts.csswg.org/css-animations/#propdef-animation-iteration-count",
                          allowed_in_keyframe_block="False">
    use std::fmt;
    use style_traits::ToCss;
    use values::computed::ComputedValueAsSpecified;
    use values::HasViewportPercentage;

    pub mod computed_value {
        pub use super::SpecifiedValue as T;
    }

    // https://drafts.csswg.org/css-animations/#animation-iteration-count
    #[derive(Debug, Clone, PartialEq)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub enum SpecifiedValue {
        Number(f32),
        Infinite,
    }

    impl Parse for SpecifiedValue {
        fn parse(_context: &ParserContext, input: &mut ::cssparser::Parser) -> Result<Self, ()> {
            if input.try(|input| input.expect_ident_matching("infinite")).is_ok() {
                return Ok(SpecifiedValue::Infinite)
            }

            let number = try!(input.expect_number());
            if number < 0.0 {
                return Err(());
            }

            Ok(SpecifiedValue::Number(number))
        }
    }

    impl ToCss for SpecifiedValue {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match *self {
                SpecifiedValue::Number(n) => write!(dest, "{}", n),
                SpecifiedValue::Infinite => dest.write_str("infinite"),
            }
        }
    }

    no_viewport_percentage!(SpecifiedValue);

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        get_initial_specified_value()
    }

    #[inline]
    pub fn get_initial_specified_value() -> SpecifiedValue {
        SpecifiedValue::Number(1.0)
    }

    #[inline]
    pub fn parse(context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
        SpecifiedValue::parse(context, input)
    }

    impl ComputedValueAsSpecified for SpecifiedValue {}
</%helpers:vector_longhand>

<% animation_direction_custom_consts = { "alternate-reverse": "Alternate_reverse" } %>
${helpers.single_keyword("animation-direction",
                         "normal reverse alternate alternate-reverse",
                         need_index=True,
                         animatable=False,
                         vector=True,
                         gecko_enum_prefix="PlaybackDirection",
                         custom_consts=animation_direction_custom_consts,
                         extra_prefixes="moz webkit",
                         spec="https://drafts.csswg.org/css-animations/#propdef-animation-direction",
                         allowed_in_keyframe_block=False)}

// animation-play-state is the exception to the rule for allowed_in_keyframe_block:
// https://drafts.csswg.org/css-animations/#keyframes
${helpers.single_keyword("animation-play-state",
                         "running paused",
                         need_clone=True,
                         need_index=True,
                         animatable=False,
                         vector=True,
                         extra_prefixes="moz webkit",
                         spec="https://drafts.csswg.org/css-animations/#propdef-animation-play-state",
                         allowed_in_keyframe_block=True)}

${helpers.single_keyword("animation-fill-mode",
                         "none forwards backwards both",
                         need_index=True,
                         animatable=False,
                         vector=True,
                         gecko_enum_prefix="FillMode",
                         extra_prefixes="moz webkit",
                         spec="https://drafts.csswg.org/css-animations/#propdef-animation-fill-mode",
                         allowed_in_keyframe_block=False)}

<%helpers:vector_longhand name="animation-delay"
                          need_index="True"
                          animatable="False",
                          extra_prefixes="moz webkit",
                          spec="https://drafts.csswg.org/css-animations/#propdef-animation-delay",
                          allowed_in_keyframe_block="False">
    pub use properties::longhands::transition_duration::single_value::computed_value;
    pub use properties::longhands::transition_duration::single_value::get_initial_specified_value;
    pub use properties::longhands::transition_duration::single_value::{get_initial_value, parse};
    pub use properties::longhands::transition_duration::single_value::SpecifiedValue;
</%helpers:vector_longhand>

<%helpers:longhand products="gecko" name="scroll-snap-points-y" animatable="False"
                   spec="Nonstandard (https://www.w3.org/TR/2015/WD-css-snappoints-1-20150326/#scroll-snap-points)">
    use std::fmt;
    use style_traits::ToCss;
    use values::HasViewportPercentage;
    use values::specified::LengthOrPercentage;

    impl HasViewportPercentage for SpecifiedValue {
        fn has_viewport_percentage(&self) -> bool {
            match *self {
                SpecifiedValue::Repeat(ref length) => length.has_viewport_percentage(),
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

    #[derive(Debug, Clone, PartialEq)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub enum SpecifiedValue {
        None,
        Repeat(LengthOrPercentage),
    }

    impl ToCss for computed_value::T {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match self.0 {
                None => dest.write_str("none"),
                Some(ref l) => {
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
                SpecifiedValue::Repeat(ref l) =>
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

<%helpers:longhand products="gecko" name="scroll-snap-points-x" animatable="False"
                   spec="Nonstandard (https://www.w3.org/TR/2015/WD-css-snappoints-1-20150326/#scroll-snap-points)">
    pub use super::scroll_snap_points_y::SpecifiedValue;
    pub use super::scroll_snap_points_y::computed_value;
    pub use super::scroll_snap_points_y::get_initial_value;
    pub use super::scroll_snap_points_y::parse;
</%helpers:longhand>


${helpers.predefined_type("scroll-snap-destination",
                          "Position",
                          "computed::Position::zero()",
                          products="gecko",
                          boxed="True",
                          spec="Nonstandard (https://developer.mozilla.org/en-US/docs/Web/CSS/scroll-snap-destination)",
                          animatable=True)}

${helpers.predefined_type("scroll-snap-coordinate",
                          "Position",
                          "computed::Position::zero()",
                          vector=True,
                          products="gecko",
                          spec="Nonstandard (https://developer.mozilla.org/en-US/docs/Web/CSS/scroll-snap-destination)",
                          animatable=True,
                          allow_empty=True,
                          delegate_animate=True)}



<%helpers:longhand name="transform" products="gecko servo" extra_prefixes="webkit"
                   animatable="True"
                   creates_stacking_context="True"
                   fixpos_cb="True"
                   spec="https://drafts.csswg.org/css-transforms/#propdef-transform">
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
                SpecifiedOperation::Translate(_, ref l1, ref l2, ref l3) => {
                    l1.has_viewport_percentage() ||
                    l2.has_viewport_percentage() ||
                    l3.has_viewport_percentage()
                },
                SpecifiedOperation::Perspective(ref length) => length.has_viewport_percentage(),
                _ => false
            }
        }
    }

    impl ToCss for SpecifiedOperation {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match *self {
                // todo(gw): implement serialization for transform
                // types other than translate.
                SpecifiedOperation::Matrix(..) => {
                    Ok(())
                }
                SpecifiedOperation::Skew(..) => {
                    Ok(())
                }
                SpecifiedOperation::Translate(kind, ref tx, ref ty, ref tz) => {
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
                SpecifiedOperation::Scale(..) => {
                    Ok(())
                }
                SpecifiedOperation::Rotate(..) => {
                    Ok(())
                }
                SpecifiedOperation::Perspective(_) => {
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

            if self.0.is_empty() {
                return dest.write_str("none")
            }

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
                &name,
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
                                                                  specified::Length::zero()));
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
                            specified::Length::zero()));
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
                            specified::Length::zero()));
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
                    SpecifiedOperation::Perspective(ref d) => {
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
                         spec="https://drafts.csswg.org/cssom-view/#propdef-scroll-behavior",
                         animatable=False)}

${helpers.single_keyword("scroll-snap-type-x",
                         "none mandatory proximity",
                         products="gecko",
                         gecko_constant_prefix="NS_STYLE_SCROLL_SNAP_TYPE",
                         spec="Nonstandard (https://developer.mozilla.org/en-US/docs/Web/CSS/scroll-snap-type-x)",
                         animatable=False)}

<%helpers:longhand products="gecko" name="scroll-snap-type-y" animatable="False"
                   spec="Nonstandard (https://developer.mozilla.org/en-US/docs/Web/CSS/scroll-snap-type-x)">
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
                         spec="https://drafts.fxtf.org/compositing/#isolation",
                         creates_stacking_context=True,
                         animatable=False)}

// TODO add support for logical values recto and verso
${helpers.single_keyword("page-break-after",
                         "auto always avoid left right",
                         products="gecko",
                         spec="https://drafts.csswg.org/css2/page.html#propdef-page-break-after",
                         animatable=False)}
${helpers.single_keyword("page-break-before",
                         "auto always avoid left right",
                         products="gecko",
                         spec="https://drafts.csswg.org/css2/page.html#propdef-page-break-before",
                         animatable=False)}
${helpers.single_keyword("page-break-inside",
                         "auto avoid",
                         products="gecko",
                         gecko_ffi_name="mBreakInside",
                         gecko_constant_prefix="NS_STYLE_PAGE_BREAK",
                         spec="https://drafts.csswg.org/css2/page.html#propdef-page-break-inside",
                         animatable=False)}

// CSS Basic User Interface Module Level 3
// http://dev.w3.org/csswg/css-ui
// FIXME support logical values `block` and `inline` (https://drafts.csswg.org/css-logical-props/#resize)
${helpers.single_keyword("resize",
                         "none both horizontal vertical",
                         products="gecko",
                         spec="https://drafts.csswg.org/css-ui/#propdef-resize",
                         animatable=False)}


${helpers.predefined_type("perspective",
                          "LengthOrNone",
                          "Either::Second(None_)",
                          gecko_ffi_name="mChildPerspective",
                          spec="https://drafts.csswg.org/css-transforms/#perspective",
                          extra_prefixes="moz webkit",
                          boxed=True,
                          creates_stacking_context=True,
                          fixpos_cb=True,
                          animatable=True)}

// FIXME: This prop should be animatable
<%helpers:longhand name="perspective-origin" boxed="True" animatable="False" extra_prefixes="moz webkit"
                   spec="https://drafts.csswg.org/css-transforms/#perspective-origin-property">
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

    #[derive(Clone, Debug, PartialEq)]
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

${helpers.single_keyword("backface-visibility",
                         "visible hidden",
                         spec="https://drafts.csswg.org/css-transforms/#backface-visibility-property",
                         extra_prefixes="moz webkit",
                         animatable=False)}

${helpers.single_keyword("transform-box",
                         "border-box fill-box view-box",
                         gecko_enum_prefix="StyleGeometryBox",
                         products="gecko",
                         spec="https://drafts.csswg.org/css-transforms/#transform-box",
                         animatable=False)}

// `auto` keyword is not supported in gecko yet.
${helpers.single_keyword("transform-style",
                         "auto flat preserve-3d" if product == "servo" else
                         "flat preserve-3d",
                         spec="https://drafts.csswg.org/css-transforms/#transform-style-property",
                         extra_prefixes="moz webkit",
                         creates_stacking_context=True,
                         fixpos_cb=True,
                         animatable=False)}

<%helpers:longhand name="transform-origin" animatable="True" extra_prefixes="moz webkit" boxed="True"
                   spec="https://drafts.csswg.org/css-transforms/#transform-origin-property">
    use app_units::Au;
    use std::fmt;
    use style_traits::ToCss;
    use values::HasViewportPercentage;
    use values::specified::{NoCalcLength, LengthOrPercentage, Percentage};

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

    #[derive(Clone, Debug, PartialEq)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub struct SpecifiedValue {
        horizontal: LengthOrPercentage,
        vertical: LengthOrPercentage,
        depth: NoCalcLength,
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
            depth: result.depth.unwrap_or(NoCalcLength::zero()),
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

<%helpers:longhand name="contain" animatable="False" products="none"
                   spec="https://drafts.csswg.org/css-contain/#contain-property">
    use std::fmt;
    use style_traits::ToCss;
    use values::HasViewportPercentage;
    use values::computed::ComputedValueAsSpecified;

    impl ComputedValueAsSpecified for SpecifiedValue {}
    no_viewport_percentage!(SpecifiedValue);

    pub mod computed_value {
        pub type T = super::SpecifiedValue;
    }

    bitflags! {
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub flags SpecifiedValue: u8 {
            const SIZE = 0x01,
            const LAYOUT = 0x02,
            const STYLE = 0x04,
            const PAINT = 0x08,
            const STRICT = SIZE.bits | LAYOUT.bits | STYLE.bits | PAINT.bits,
            const CONTENT = LAYOUT.bits | STYLE.bits | PAINT.bits,
        }
    }

    impl ToCss for SpecifiedValue {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            if self.is_empty() {
                return dest.write_str("none")
            }
            if self.contains(STRICT) {
                return dest.write_str("strict")
            }
            if self.contains(CONTENT) {
                return dest.write_str("content")
            }

            let mut has_any = false;
            macro_rules! maybe_write_value {
                ($ident:ident => $str:expr) => {
                    if self.contains($ident) {
                        if has_any {
                            try!(dest.write_str(" "));
                        }
                        has_any = true;
                        try!(dest.write_str($str));
                    }
                }
            }
            maybe_write_value!(SIZE => "size");
            maybe_write_value!(LAYOUT => "layout");
            maybe_write_value!(STYLE => "style");
            maybe_write_value!(PAINT => "paint");

            debug_assert!(has_any);
            Ok(())
        }
    }

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        computed_value::T::empty()
    }

    /// none | strict | content | [ size || layout || style || paint ]
    pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
        let mut result = SpecifiedValue::empty();

        if input.try(|input| input.expect_ident_matching("none")).is_ok() {
            return Ok(result)
        }
        if input.try(|input| input.expect_ident_matching("strict")).is_ok() {
            result.insert(STRICT);
            return Ok(result)
        }
        if input.try(|input| input.expect_ident_matching("content")).is_ok() {
            result.insert(CONTENT);
            return Ok(result)
        }

        while let Ok(name) = input.try(|input| input.expect_ident()) {
            let flag = match_ignore_ascii_case! { &name,
                "size" => SIZE,
                "layout" => LAYOUT,
                "style" => STYLE,
                "paint" => PAINT,
                _ => return Err(())
            };
            if result.contains(flag) {
                return Err(())
            }
            result.insert(flag);
        }

        if !result.is_empty() {
            Ok(result)
        } else {
            Err(())
        }
    }
</%helpers:longhand>

${helpers.single_keyword("appearance",
                         "auto none",
                         gecko_ffi_name="mAppearance",
                         gecko_constant_prefix="NS_THEME",
                         products="gecko",
                         spec="https://drafts.csswg.org/css-ui-4/#appearance-switching",
                         alias="-webkit-appearance",
                         animatable=False)}

// Non-standard
${helpers.single_keyword("-moz-appearance",
                         """none button button-arrow-down button-arrow-next button-arrow-previous button-arrow-up
                            button-bevel button-focus caret checkbox checkbox-container checkbox-label checkmenuitem
                            dualbutton groupbox listbox listitem menuarrow menubar menucheckbox menuimage menuitem
                            menuitemtext menulist menulist-button menulist-text menulist-textfield menupopup menuradio
                            menuseparator meterbar meterchunk number-input progressbar progressbar-vertical
                            progresschunk
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
                         gecko_ffi_name="mMozAppearance",
                         gecko_constant_prefix="NS_THEME",
                         products="gecko",
                         spec="Nonstandard (https://developer.mozilla.org/en-US/docs/Web/CSS/-moz-appearance)",
                         animatable=False)}

${helpers.predefined_type("-moz-binding", "UrlOrNone", "Either::Second(None_)",
                          products="gecko",
                          animatable="False",
                          gecko_ffi_name="mBinding",
                          spec="Nonstandard (https://developer.mozilla.org/en-US/docs/Web/CSS/-moz-binding)",
                          disable_when_testing="True",
                          boxed=True)}

${helpers.single_keyword("-moz-orient",
                          "inline block horizontal vertical",
                          products="gecko",
                          gecko_ffi_name="mOrient",
                          gecko_enum_prefix="StyleOrient",
                          spec="Nonstandard (https://developer.mozilla.org/en-US/docs/Web/CSS/-moz-orient)",
                          animatable=False)}

<%helpers:longhand name="will-change" products="gecko" animatable="False"
                   spec="https://drafts.csswg.org/css-will-change/#will-change">
    use cssparser::serialize_identifier;
    use std::fmt;
    use style_traits::ToCss;
    use values::HasViewportPercentage;
    use values::computed::ComputedValueAsSpecified;

    impl ComputedValueAsSpecified for SpecifiedValue {}
    no_viewport_percentage!(SpecifiedValue);

    pub mod computed_value {
        pub use super::SpecifiedValue as T;
    }

    #[derive(Debug, Clone, PartialEq)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub enum SpecifiedValue {
        Auto,
        AnimateableFeatures(Vec<Atom>),
    }

    impl ToCss for SpecifiedValue {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match *self {
                SpecifiedValue::Auto => dest.write_str("auto"),
                SpecifiedValue::AnimateableFeatures(ref features) => {
                    let (first, rest) = features.split_first().unwrap();
                    // handle head element
                    serialize_identifier(&*first.to_string(), dest)?;
                    // handle tail, precede each with a delimiter
                    for feature in rest {
                        dest.write_str(", ")?;
                        serialize_identifier(&*feature.to_string(), dest)?;
                    }
                    Ok(())
                }
            }
        }
    }

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        computed_value::T::Auto
    }

    /// auto | <animateable-feature>#
    pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
        if input.try(|input| input.expect_ident_matching("auto")).is_ok() {
            Ok(computed_value::T::Auto)
        } else {
            input.parse_comma_separated(|i| {
                let ident = i.expect_ident()?;
                match_ignore_ascii_case! { &ident,
                    "will-change" | "none" | "all" | "auto" |
                    "initial" | "inherit" | "unset" | "default" => return Err(()),
                    _ => {},
                }
                Ok((Atom::from(ident)))
            }).map(SpecifiedValue::AnimateableFeatures)
        }
    }
</%helpers:longhand>
