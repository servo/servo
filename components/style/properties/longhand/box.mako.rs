/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />
<% from data import Keyword, Method, to_rust_ident, to_camel_case%>

<% data.new_style_struct("Box",
                         inherited=False,
                         gecko_name="Display") %>

// TODO(SimonSapin): don't parse `inline-table`, since we don't support it
//
// We allow "display" to apply to placeholders because we need to make the
// placeholder pseudo-element an inline-block in the UA stylesheet in Gecko.
<%helpers:longhand name="display"
                   need_clone="True"
                   animation_value_type="discrete"
                   custom_cascade="${product == 'servo'}"
                   flags="APPLIES_TO_PLACEHOLDER"
                   spec="https://drafts.csswg.org/css-display/#propdef-display">
    <%
        values = """inline block inline-block
            table inline-table table-row-group table-header-group table-footer-group
            table-row table-column-group table-column table-cell table-caption
            list-item none
        """.split()
        webkit_prefixed_values = "flex inline-flex".split()
        values += webkit_prefixed_values
        if product == "gecko":
            values += """grid inline-grid ruby ruby-base ruby-base-container
                ruby-text ruby-text-container contents flow-root -webkit-box
                -webkit-inline-box -moz-box -moz-inline-box -moz-grid -moz-inline-grid
                -moz-grid-group -moz-grid-line -moz-stack -moz-inline-stack -moz-deck
                -moz-popup -moz-groupbox""".split()
    %>
    use values::computed::ComputedValueAsSpecified;
    use style_traits::ToCss;
    no_viewport_percentage!(SpecifiedValue);

    pub mod computed_value {
        pub use super::SpecifiedValue as T;

        impl T {
            /// Returns whether this "display" value is the display of a flex or
            /// grid container.
            ///
            /// This is used to implement various style fixups.
            pub fn is_item_container(&self) -> bool {
                matches!(*self,
                         T::flex
                         | T::inline_flex
                         % if product == "gecko":
                         | T::grid
                         | T::inline_grid
                         % endif
                )
            }

            /// Returns whether an element with this display type is a line
            /// participant, which means it may lay its children on the same
            /// line as itself.
            pub fn is_line_participant(&self) -> bool {
                matches!(*self,
                         T::inline
                         % if product == "gecko":
                         | T::contents
                         | T::ruby
                         | T::ruby_base_container
                         % endif
                )
            }

            /// Returns whether this "display" value is one of the types for
            /// ruby.
            #[cfg(feature = "gecko")]
            pub fn is_ruby_type(&self) -> bool {
                matches!(*self, T::ruby | T::ruby_base | T::ruby_text |
                         T::ruby_base_container | T::ruby_text_container)
            }

            /// Returns whether this "display" value is a ruby level container.
            #[cfg(feature = "gecko")]
            pub fn is_ruby_level_container(&self) -> bool {
                matches!(*self, T::ruby_base_container | T::ruby_text_container)
            }

            /// Convert this display into an equivalent block display.
            ///
            /// Also used for style adjustments.
            pub fn equivalent_block_display(&self, _is_root_element: bool) -> Self {
                match *self {
                    // Values that have a corresponding block-outside version.
                    T::inline_table => T::table,
                    T::inline_flex => T::flex,

                    % if product == "gecko":
                    T::inline_grid => T::grid,
                    T::_webkit_inline_box => T::_webkit_box,
                    % endif

                    // Special handling for contents and list-item on the root
                    // element for Gecko.
                    % if product == "gecko":
                    T::contents | T::list_item if _is_root_element => T::block,
                    % endif

                    // These are not changed by blockification.
                    T::none | T::block | T::flex | T::list_item | T::table => *self,
                    % if product == "gecko":
                    T::contents | T::flow_root | T::grid | T::_webkit_box => *self,
                    % endif

                    // Everything else becomes block.
                    _ => T::block,
                }

            }

            /// Convert this display into an inline-outside display.
            ///
            /// Ideally it should implement spec: https://drafts.csswg.org/css-display/#inlinify
            /// but the spec isn't stable enough, so we copy what Gecko does for now.
            #[cfg(feature = "gecko")]
            pub fn inlinify(&self) -> Self {
                match *self {
                    T::block | T::flow_root => T::inline_block,
                    T::table => T::inline_table,
                    T::flex => T::inline_flex,
                    T::grid => T::inline_grid,
                    T::_moz_box => T::_moz_inline_box,
                    T::_moz_stack => T::_moz_inline_stack,
                    T::_webkit_box => T::_webkit_inline_box,
                    other => other,
                }
            }
        }
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
    pub fn parse<'i, 't>(_context: &ParserContext, input: &mut Parser<'i, 't>)
                         -> Result<SpecifiedValue, ParseError<'i>> {
        try_match_ident_ignore_ascii_case! { input.expect_ident()?,
            % for value in values:
                "${value}" => {
                    Ok(computed_value::T::${to_rust_ident(value)})
                },
            % endfor
            % for value in webkit_prefixed_values:
                "-webkit-${value}" => {
                    Ok(computed_value::T::${to_rust_ident(value)})
                },
            % endfor
        }
    }

    impl ComputedValueAsSpecified for SpecifiedValue {}

    % if product == "servo":
        fn cascade_property_custom(_declaration: &PropertyDeclaration,
                                   context: &mut computed::Context) {
            longhands::_servo_display_for_hypothetical_box::derive_from_display(context);
            longhands::_servo_text_decorations_in_effect::derive_from_display(context);
            longhands::_servo_under_display_none::derive_from_display(context);
        }
    % endif

    ${helpers.gecko_keyword_conversion(Keyword('display', ' '.join(values),
                                               gecko_enum_prefix='StyleDisplay',
                                               gecko_strip_moz_prefix=False))}

</%helpers:longhand>

${helpers.single_keyword("-moz-top-layer", "none top",
                         gecko_constant_prefix="NS_STYLE_TOP_LAYER",
                         gecko_ffi_name="mTopLayer", need_clone=True,
                         products="gecko", animation_value_type="none", internal=True,
                         spec="Internal (not web-exposed)")}

${helpers.single_keyword("position", "static absolute relative fixed",
                         extra_gecko_values="sticky",
                         animation_value_type="discrete",
                         flags="CREATES_STACKING_CONTEXT ABSPOS_CB",
                         spec="https://drafts.csswg.org/css-position/#position-property")}

<%helpers:single_keyword_computed name="float"
                                  values="none left right"
                                  // https://drafts.csswg.org/css-logical-props/#float-clear
                                  extra_specified="inline-start inline-end"
                                  needs_conversion="True"
                                  animation_value_type="discrete"
                                  gecko_enum_prefix="StyleFloat"
                                  gecko_inexhaustive="True"
                                  gecko_ffi_name="mFloat"
                                  gecko_pref_ident="float_"
                                  flags="APPLIES_TO_FIRST_LETTER"
                                  spec="https://drafts.csswg.org/css-box/#propdef-float">
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
                                  gecko_inexhaustive="True"
                                  animation_value_type="discrete"
                                  gecko_enum_prefix="StyleClear"
                                  gecko_ffi_name="mBreakType"
                                  spec="https://www.w3.org/TR/CSS2/visuren.html#flow-control">
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
                   animation_value_type="none"
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
        context.builder.set__servo_display_for_hypothetical_box(d);
    }

</%helpers:longhand>

<%helpers:longhand name="vertical-align" animation_value_type="ComputedValue"
                   flags="APPLIES_TO_FIRST_LETTER APPLIES_TO_FIRST_LINE APPLIES_TO_PLACEHOLDER",
                   spec="https://www.w3.org/TR/CSS2/visudet.html#propdef-vertical-align">
    use std::fmt;
    use style_traits::ToCss;
    use values::specified::AllowQuirks;

    <% vertical_align = data.longhands_by_name["vertical-align"] %>
    <% vertical_align.keyword = Keyword("vertical-align",
                                        "baseline sub super top text-top middle bottom text-bottom",
                                        extra_gecko_values="-moz-middle-with-baseline") %>
    <% vertical_align_keywords = vertical_align.keyword.values_for(product) %>

    ${helpers.gecko_keyword_conversion(vertical_align.keyword)}

    /// The `vertical-align` value.
    #[allow(non_camel_case_types)]
    #[derive(Clone, Debug, HasViewportPercentage, PartialEq)]
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
    pub fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>)
                         -> Result<SpecifiedValue, ParseError<'i>> {
        if let Ok(lop) = input.try(|i| specified::LengthOrPercentage::parse_quirky(context, i, AllowQuirks::Yes)) {
            return Ok(SpecifiedValue::LengthOrPercentage(lop));
        }

        try_match_ident_ignore_ascii_case! { input.expect_ident()?,
            % for keyword in vertical_align_keywords:
                "${keyword}" => Ok(SpecifiedValue::${to_rust_ident(keyword)}),
            % endfor
        }
    }

    /// The computed value for `vertical-align`.
    pub mod computed_value {
        use std::fmt;
        use style_traits::ToCss;
        use values::computed;

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
    products="servo", animation_value_type="none", internal=True,
    spec="Internal, not web-exposed, \
          may be standardized in the future (https://developer.mozilla.org/en-US/docs/Web/CSS/overflow-clip-box)")}

${helpers.single_keyword("overflow-clip-box", "padding-box content-box",
    products="gecko", animation_value_type="discrete", internal=True,
    flags="APPLIES_TO_PLACEHOLDER",
    spec="Internal, not web-exposed, \
          may be standardized in the future (https://developer.mozilla.org/en-US/docs/Web/CSS/overflow-clip-box)")}

<%
    overflow_custom_consts = { "-moz-hidden-unscrollable": "CLIP" }
%>

// FIXME(pcwalton, #2742): Implement scrolling for `scroll` and `auto`.
${helpers.single_keyword("overflow-x", "visible hidden scroll auto",
                         need_clone=True, animation_value_type="discrete",
                         extra_gecko_values="-moz-hidden-unscrollable",
                         custom_consts=overflow_custom_consts,
                         gecko_constant_prefix="NS_STYLE_OVERFLOW",
                         flags="APPLIES_TO_PLACEHOLDER",
                         spec="https://drafts.csswg.org/css-overflow/#propdef-overflow-x")}

// FIXME(pcwalton, #2742): Implement scrolling for `scroll` and `auto`.
<%helpers:longhand name="overflow-y" need_clone="True" animation_value_type="discrete"
                   flags="APPLIES_TO_PLACEHOLDER",
                   spec="https://drafts.csswg.org/css-overflow/#propdef-overflow-y">
    pub use super::overflow_x::{SpecifiedValue, parse, get_initial_value, computed_value};
</%helpers:longhand>

${helpers.predefined_type("transition-duration",
                          "Time",
                          "computed::Time::zero()",
                          initial_specified_value="specified::Time::zero()",
                          parse_method="parse_non_negative",
                          vector=True,
                          need_index=True,
                          animation_value_type="none",
                          extra_prefixes="moz webkit",
                          spec="https://drafts.csswg.org/css-transitions/#propdef-transition-duration")}

${helpers.predefined_type("transition-timing-function",
                          "TimingFunction",
                          "computed::TimingFunction::ease()",
                          initial_specified_value="specified::TimingFunction::ease()",
                          vector=True,
                          need_index=True,
                          animation_value_type="none",
                          extra_prefixes="moz webkit",
                          spec="https://drafts.csswg.org/css-transitions/#propdef-transition-timing-function")}

${helpers.predefined_type(
    "transition-property",
    "TransitionProperty",
    "computed::TransitionProperty::All",
    initial_specified_value="specified::TransitionProperty::All",
    vector=True,
    allow_empty="NotInitial",
    need_index=True,
    needs_context=False,
    animation_value_type="none",
    extra_prefixes="moz webkit",
    spec="https://drafts.csswg.org/css-transitions/#propdef-transition-property",
)}

${helpers.predefined_type("transition-delay",
                          "Time",
                          "computed::Time::zero()",
                          initial_specified_value="specified::Time::zero()",
                          vector=True,
                          need_index=True,
                          animation_value_type="none",
                          extra_prefixes="moz webkit",
                          spec="https://drafts.csswg.org/css-transitions/#propdef-transition-delay")}

<%helpers:vector_longhand name="animation-name"
                          need_index="True"
                          animation_value_type="none",
                          extra_prefixes="moz webkit"
                          allowed_in_keyframe_block="False"
                          spec="https://drafts.csswg.org/css-animations/#propdef-animation-name">
    use Atom;
    use std::fmt;
    use style_traits::ToCss;
    use values::computed::ComputedValueAsSpecified;
    use values::KeyframesName;

    pub mod computed_value {
        pub use super::SpecifiedValue as T;
    }

    #[derive(Clone, Debug, Hash, Eq, PartialEq)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub struct SpecifiedValue(pub Option<KeyframesName>);

    impl SpecifiedValue {
        /// As an Atom
        pub fn as_atom(&self) -> Option< &Atom> {
            self.0.as_ref().map(|n| n.as_atom())
        }
    }

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        get_initial_specified_value()
    }

    #[inline]
    pub fn get_initial_specified_value() -> SpecifiedValue {
        SpecifiedValue(None)
    }

    impl fmt::Display for SpecifiedValue {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            self.to_css(f)
        }
    }

    impl ToCss for SpecifiedValue {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            if let Some(ref name) = self.0 {
                name.to_css(dest)
            } else {
                dest.write_str("none")
            }
        }
    }

    impl Parse for SpecifiedValue {
        fn parse<'i, 't>(context: &ParserContext, input: &mut ::cssparser::Parser<'i, 't>)
                         -> Result<Self, ParseError<'i>> {
            if let Ok(name) = input.try(|input| KeyframesName::parse(context, input)) {
                Ok(SpecifiedValue(Some(name)))
            } else {
                input.expect_ident_matching("none").map(|_| SpecifiedValue(None)).map_err(|e| e.into())
            }
        }
    }
    no_viewport_percentage!(SpecifiedValue);

    pub fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>)
                         -> Result<SpecifiedValue,ParseError<'i>> {
        SpecifiedValue::parse(context, input)
    }

    impl ComputedValueAsSpecified for SpecifiedValue {}
</%helpers:vector_longhand>

${helpers.predefined_type("animation-duration",
                          "Time",
                          "computed::Time::zero()",
                          initial_specified_value="specified::Time::zero()",
                          parse_method="parse_non_negative",
                          vector=True,
                          need_index=True,
                          animation_value_type="none",
                          extra_prefixes="moz webkit",
                          spec="https://drafts.csswg.org/css-transitions/#propdef-transition-duration")}

// animation-timing-function is the exception to the rule for allowed_in_keyframe_block:
// https://drafts.csswg.org/css-animations/#keyframes
${helpers.predefined_type("animation-timing-function",
                          "TimingFunction",
                          "computed::TimingFunction::ease()",
                          initial_specified_value="specified::TimingFunction::ease()",
                          vector=True,
                          need_index=True,
                          animation_value_type="none",
                          extra_prefixes="moz webkit",
                          allowed_in_keyframe_block=True,
                          spec="https://drafts.csswg.org/css-transitions/#propdef-animation-timing-function")}

<%helpers:vector_longhand name="animation-iteration-count"
                          need_index="True"
                          animation_value_type="none",
                          extra_prefixes="moz webkit"
                          spec="https://drafts.csswg.org/css-animations/#propdef-animation-iteration-count",
                          allowed_in_keyframe_block="False">
    use values::computed::ComputedValueAsSpecified;

    pub mod computed_value {
        pub use super::SpecifiedValue as T;
    }

    // https://drafts.csswg.org/css-animations/#animation-iteration-count
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    #[derive(Debug, Clone, PartialEq, ToCss)]
    pub enum SpecifiedValue {
        Number(f32),
        Infinite,
    }

    impl Parse for SpecifiedValue {
        fn parse<'i, 't>(_context: &ParserContext, input: &mut ::cssparser::Parser<'i, 't>)
                         -> Result<Self, ParseError<'i>> {
            if input.try(|input| input.expect_ident_matching("infinite")).is_ok() {
                return Ok(SpecifiedValue::Infinite)
            }

            let number = input.expect_number()?;
            if number < 0.0 {
                return Err(StyleParseError::UnspecifiedError.into());
            }

            Ok(SpecifiedValue::Number(number))
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
    pub fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>)
                         -> Result<SpecifiedValue, ParseError<'i>> {
        SpecifiedValue::parse(context, input)
    }

    impl ComputedValueAsSpecified for SpecifiedValue {}
</%helpers:vector_longhand>

<% animation_direction_custom_consts = { "alternate-reverse": "Alternate_reverse" } %>
${helpers.single_keyword("animation-direction",
                         "normal reverse alternate alternate-reverse",
                         need_index=True,
                         animation_value_type="none",
                         vector=True,
                         gecko_enum_prefix="PlaybackDirection",
                         custom_consts=animation_direction_custom_consts,
                         extra_prefixes="moz webkit",
                         spec="https://drafts.csswg.org/css-animations/#propdef-animation-direction",
                         allowed_in_keyframe_block=False)}

${helpers.single_keyword("animation-play-state",
                         "running paused",
                         need_clone=True,
                         need_index=True,
                         animation_value_type="none",
                         vector=True,
                         extra_prefixes="moz webkit",
                         spec="https://drafts.csswg.org/css-animations/#propdef-animation-play-state",
                         allowed_in_keyframe_block=False)}

${helpers.single_keyword("animation-fill-mode",
                         "none forwards backwards both",
                         need_index=True,
                         animation_value_type="none",
                         vector=True,
                         gecko_enum_prefix="FillMode",
                         extra_prefixes="moz webkit",
                         spec="https://drafts.csswg.org/css-animations/#propdef-animation-fill-mode",
                         allowed_in_keyframe_block=False)}

${helpers.predefined_type("animation-delay",
                          "Time",
                          "computed::Time::zero()",
                          initial_specified_value="specified::Time::zero()",
                          vector=True,
                          need_index=True,
                          animation_value_type="none",
                          extra_prefixes="moz webkit",
                          spec="https://drafts.csswg.org/css-animations/#propdef-animation-delay",
                          allowed_in_keyframe_block=False)}

% for axis in ["x", "y"]:
    ${helpers.predefined_type(
        "scroll-snap-points-" + axis,
        "ScrollSnapPoint",
        "computed::ScrollSnapPoint::none()",
        animation_value_type="discrete",
        products="gecko",
        disable_when_testing=True,
        spec="Nonstandard (https://www.w3.org/TR/2015/WD-css-snappoints-1-20150326/#scroll-snap-points)",
    )}
% endfor

${helpers.predefined_type("scroll-snap-destination",
                          "Position",
                          "computed::Position::zero()",
                          products="gecko",
                          boxed="True",
                          spec="Nonstandard (https://developer.mozilla.org/en-US/docs/Web/CSS/scroll-snap-destination)",
                          animation_value_type="ComputedValue")}

${helpers.predefined_type(
    "scroll-snap-coordinate",
    "Position",
    "computed::Position::zero()",
    vector=True,
    products="gecko",
    spec="Nonstandard (https://developer.mozilla.org/en-US/docs/Web/CSS/scroll-snap-destination)",
    animation_value_type="ComputedValue",
    allow_empty="NotInitial"
)}

<%helpers:longhand name="transform" extra_prefixes="webkit"
                   animation_value_type="ComputedValue"
                   flags="CREATES_STACKING_CONTEXT FIXPOS_CB"
                   spec="https://drafts.csswg.org/css-transforms/#propdef-transform">
    use app_units::Au;
    use values::computed::{LengthOrPercentageOrNumber as ComputedLoPoNumber, LengthOrNumber as ComputedLoN};
    use values::computed::{LengthOrPercentage as ComputedLoP, Length as ComputedLength};
    use values::generics::transform::Matrix;
    use values::specified::{Angle, Integer, Length, LengthOrPercentage};
    use values::specified::{LengthOrNumber, LengthOrPercentageOrNumber as LoPoNumber, Number};
    use style_traits::ToCss;
    use style_traits::values::Css;

    use std::fmt;

    pub mod computed_value {
        use app_units::Au;
        use values::CSSFloat;
        use values::computed;
        use values::computed::{Length, LengthOrPercentage};

        #[derive(Clone, Copy, Debug, PartialEq)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub struct ComputedMatrix {
            pub m11: CSSFloat, pub m12: CSSFloat, pub m13: CSSFloat, pub m14: CSSFloat,
            pub m21: CSSFloat, pub m22: CSSFloat, pub m23: CSSFloat, pub m24: CSSFloat,
            pub m31: CSSFloat, pub m32: CSSFloat, pub m33: CSSFloat, pub m34: CSSFloat,
            pub m41: CSSFloat, pub m42: CSSFloat, pub m43: CSSFloat, pub m44: CSSFloat,
        }

        #[derive(Clone, Copy, Debug, PartialEq)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub struct ComputedMatrixWithPercents {
            pub m11: CSSFloat, pub m12: CSSFloat, pub m13: CSSFloat, pub m14: CSSFloat,
            pub m21: CSSFloat, pub m22: CSSFloat, pub m23: CSSFloat, pub m24: CSSFloat,
            pub m31: CSSFloat, pub m32: CSSFloat, pub m33: CSSFloat, pub m34: CSSFloat,
            pub m41: LengthOrPercentage, pub m42: LengthOrPercentage,
            pub m43: Length, pub m44: CSSFloat,
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

        impl ComputedMatrixWithPercents {
            pub fn identity() -> ComputedMatrixWithPercents {
                ComputedMatrixWithPercents {
                    m11: 1.0, m12: 0.0, m13: 0.0, m14: 0.0,
                    m21: 0.0, m22: 1.0, m23: 0.0, m24: 0.0,
                    m31: 0.0, m32: 0.0, m33: 1.0, m34: 0.0,
                    m41: LengthOrPercentage::zero(), m42: LengthOrPercentage::zero(),
                    m43: Au(0), m44: 1.0
                }
            }
        }

        #[derive(Clone, Debug, PartialEq)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub enum ComputedOperation {
            Matrix(ComputedMatrix),
            // For `-moz-transform` matrix and matrix3d.
            MatrixWithPercents(ComputedMatrixWithPercents),
            Skew(computed::Angle, computed::Angle),
            Translate(computed::LengthOrPercentage,
                      computed::LengthOrPercentage,
                      computed::Length),
            Scale(CSSFloat, CSSFloat, CSSFloat),
            Rotate(CSSFloat, CSSFloat, CSSFloat, computed::Angle),
            Perspective(computed::Length),
            // For mismatched transform lists.
            // A vector of |ComputedOperation| could contain an |InterpolateMatrix| and other
            // |ComputedOperation|s, and multiple nested |InterpolateMatrix|s is acceptable.
            // e.g.
            // [ InterpolateMatrix { from_list: [ InterpolateMatrix { ... },
            //                                    Scale(...) ],
            //                       to_list: [ AccumulateMatrix { from_list: ...,
            //                                                     to_list: [ InterpolateMatrix,
            //                                                                 ... ],
            //                                                     count: ... } ],
            //                       progress: ... } ]
            InterpolateMatrix { from_list: T,
                                to_list: T,
                                progress: computed::Percentage },
            // For accumulate operation of mismatched transform lists.
            AccumulateMatrix { from_list: T,
                               to_list: T,
                               count: computed::Integer },
        }

        #[derive(Clone, Debug, PartialEq)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub struct T(pub Option<Vec<ComputedOperation>>);
    }

    /// Describes a single parsed
    /// [Transform Function](https://drafts.csswg.org/css-transforms/#typedef-transform-function).
    ///
    /// Multiple transform functions compose a transformation.
    ///
    /// Some transformations can be expressed by other more general functions.
    #[derive(Clone, Debug, HasViewportPercentage, PartialEq)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub enum SpecifiedOperation {
        /// Represents a 2D 2x3 matrix.
        Matrix(Matrix<Number>),
        /// Represents a 3D 4x4 matrix with percentage and length values.
        /// For `moz-transform`.
        PrefixedMatrix(Matrix<Number, LoPoNumber>),
        /// Represents a 3D 4x4 matrix.
        Matrix3D {
            m11: Number, m12: Number, m13: Number, m14: Number,
            m21: Number, m22: Number, m23: Number, m24: Number,
            m31: Number, m32: Number, m33: Number, m34: Number,
            m41: Number, m42: Number, m43: Number, m44: Number,
        },
        /// Represents a 3D 4x4 matrix with percentage and length values.
        /// For `moz-transform`.
        PrefixedMatrix3D {
            m11: Number,     m12: Number,     m13: Number,         m14: Number,
            m21: Number,     m22: Number,     m23: Number,         m24: Number,
            m31: Number,     m32: Number,     m33: Number,         m34: Number,
            m41: LoPoNumber, m42: LoPoNumber, m43: LengthOrNumber, m44: Number,
        },
        /// A 2D skew.
        ///
        /// If the second angle is not provided it is assumed zero.
        Skew(Angle, Option<Angle>),
        SkewX(Angle),
        SkewY(Angle),
        Translate(LengthOrPercentage, Option<LengthOrPercentage>),
        TranslateX(LengthOrPercentage),
        TranslateY(LengthOrPercentage),
        TranslateZ(Length),
        Translate3D(LengthOrPercentage, LengthOrPercentage, Length),
        /// A 2D scaling factor.
        ///
        /// `scale(2)` is parsed as `Scale(Number::new(2.0), None)` and is equivalent to
        /// writing `scale(2, 2)` (`Scale(Number::new(2.0), Some(Number::new(2.0)))`).
        ///
        /// Negative values are allowed and flip the element.
        Scale(Number, Option<Number>),
        ScaleX(Number),
        ScaleY(Number),
        ScaleZ(Number),
        Scale3D(Number, Number, Number),
        /// Describes a 2D Rotation.
        ///
        /// In a 3D scene `rotate(angle)` is equivalent to `rotateZ(angle)`.
        Rotate(Angle),
        /// Rotation in 3D space around the x-axis.
        RotateX(Angle),
        /// Rotation in 3D space around the y-axis.
        RotateY(Angle),
        /// Rotation in 3D space around the z-axis.
        RotateZ(Angle),
        /// Rotation in 3D space.
        ///
        /// Generalization of rotateX, rotateY and rotateZ.
        Rotate3D(Number, Number, Number, Angle),
        /// Specifies a perspective projection matrix.
        ///
        /// Part of CSS Transform Module Level 2 and defined at
        /// [§ 13.1. 3D Transform Function](https://drafts.csswg.org/css-transforms-2/#funcdef-perspective).
        ///
        /// The value must be greater than or equal to zero.
        Perspective(specified::Length),
        /// A intermediate type for interpolation of mismatched transform lists.
        InterpolateMatrix { from_list: SpecifiedValue,
                            to_list: SpecifiedValue,
                            progress: computed::Percentage },
        /// A intermediate type for accumulation of mismatched transform lists.
        AccumulateMatrix { from_list: SpecifiedValue,
                           to_list: SpecifiedValue,
                           count: Integer },
    }

    impl ToCss for computed_value::T {
        fn to_css<W>(&self, _: &mut W) -> fmt::Result where W: fmt::Write {
            // TODO(pcwalton)
            Ok(())
        }
    }

    impl ToCss for SpecifiedOperation {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match *self {
                SpecifiedOperation::Matrix(ref m) => m.to_css(dest),
                SpecifiedOperation::PrefixedMatrix(ref m) => m.to_css(dest),
                SpecifiedOperation::Matrix3D {
                    m11, m12, m13, m14,
                    m21, m22, m23, m24,
                    m31, m32, m33, m34,
                    m41, m42, m43, m44 } => write!(
                        dest, "matrix3d({}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {})",
                        Css(m11), Css(m12), Css(m13), Css(m14),
                        Css(m21), Css(m22), Css(m23), Css(m24),
                        Css(m31), Css(m32), Css(m33), Css(m34),
                        Css(m41), Css(m42), Css(m43), Css(m44)),
                SpecifiedOperation::PrefixedMatrix3D {
                    m11, m12, m13, m14,
                    m21, m22, m23, m24,
                    m31, m32, m33, m34,
                    ref m41, ref m42, ref m43, m44 } => write!(
                        dest, "matrix3d({}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {})",
                        Css(m11), Css(m12), Css(m13), Css(m14),
                        Css(m21), Css(m22), Css(m23), Css(m24),
                        Css(m31), Css(m32), Css(m33), Css(m34),
                        Css(m41), Css(m42), Css(m43), Css(m44)),
                SpecifiedOperation::Skew(ax, None) => write!(dest, "skew({})", Css(ax)),
                SpecifiedOperation::Skew(ax, Some(ay)) => write!(dest, "skew({}, {})", Css(ax), Css(ay)),
                SpecifiedOperation::SkewX(angle) => write!(dest, "skewX({})", Css(angle)),
                SpecifiedOperation::SkewY(angle) => write!(dest, "skewY({})", Css(angle)),
                SpecifiedOperation::Translate(ref tx, None) => write!(dest, "translate({})", Css(tx)),
                SpecifiedOperation::Translate(ref tx, Some(ref ty)) => {
                    write!(dest, "translate({}, {})", Css(tx), Css(ty))
                },
                SpecifiedOperation::TranslateX(ref tx) => write!(dest, "translateX({})", Css(tx)),
                SpecifiedOperation::TranslateY(ref ty) => write!(dest, "translateY({})", Css(ty)),
                SpecifiedOperation::TranslateZ(ref tz) => write!(dest, "translateZ({})", Css(tz)),
                SpecifiedOperation::Translate3D(ref tx, ref ty, ref tz) => write!(
                    dest, "translate3d({}, {}, {})", Css(tx), Css(ty), Css(tz)),
                SpecifiedOperation::Scale(factor, None) => write!(dest, "scale({})", Css(factor)),
                SpecifiedOperation::Scale(sx, Some(sy)) => write!(dest, "scale({}, {})", Css(sx), Css(sy)),
                SpecifiedOperation::ScaleX(sx) => write!(dest, "scaleX({})", Css(sx)),
                SpecifiedOperation::ScaleY(sy) => write!(dest, "scaleY({})", Css(sy)),
                SpecifiedOperation::ScaleZ(sz) => write!(dest, "scaleZ({})", Css(sz)),
                SpecifiedOperation::Scale3D(sx, sy, sz) => {
                    write!(dest, "scale3d({}, {}, {})", Css(sx), Css(sy), Css(sz))
                },
                SpecifiedOperation::Rotate(theta) => write!(dest, "rotate({})", Css(theta)),
                SpecifiedOperation::RotateX(theta) => write!(dest, "rotateX({})", Css(theta)),
                SpecifiedOperation::RotateY(theta) => write!(dest, "rotateY({})", Css(theta)),
                SpecifiedOperation::RotateZ(theta) => write!(dest, "rotateZ({})", Css(theta)),
                SpecifiedOperation::Rotate3D(x, y, z, theta) => write!(
                    dest, "rotate3d({}, {}, {}, {})",
                    Css(x), Css(y), Css(z), Css(theta)),
                SpecifiedOperation::Perspective(ref length) => write!(dest, "perspective({})", Css(length)),
                SpecifiedOperation::InterpolateMatrix { ref from_list, ref to_list, progress } => {
                    write!(dest, "interpolatematrix({}, {}, {})",
                           Css(from_list), Css(to_list), Css(progress))
                },
                SpecifiedOperation::AccumulateMatrix { ref from_list, ref to_list, count } => {
                    write!(dest, "accumulatematrix({}, {}, {})",
                           Css(from_list), Css(to_list), Css(count))
                }
            }
        }
    }

    #[derive(Clone, Debug, HasViewportPercentage, PartialEq)]
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
                    dest.write_str(" ")?;
                }
                first = false;
                operation.to_css(dest)?
            }
            Ok(())
        }
    }

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        computed_value::T(None)
    }

    // Allow unitless zero angle for rotate() and skew() to align with gecko
    fn parse_internal<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
        prefixed: bool,
    ) -> Result<SpecifiedValue,ParseError<'i>> {
        use style_traits::{Separator, Space};

        if input.try(|input| input.expect_ident_matching("none")).is_ok() {
            return Ok(SpecifiedValue(Vec::new()))
        }

        Ok(SpecifiedValue(Space::parse(input, |input| {
            let function = input.expect_function()?.clone();
            input.parse_nested_block(|input| {
                let result = match_ignore_ascii_case! { &function,
                    "matrix" => {
                        let a = specified::parse_number(context, input)?;
                        input.expect_comma()?;
                        let b = specified::parse_number(context, input)?;
                        input.expect_comma()?;
                        let c = specified::parse_number(context, input)?;
                        input.expect_comma()?;
                        let d = specified::parse_number(context, input)?;
                        input.expect_comma()?;
                        if !prefixed {
                            // Standard matrix parsing.
                            let e = specified::parse_number(context, input)?;
                            input.expect_comma()?;
                            let f = specified::parse_number(context, input)?;
                            Ok(SpecifiedOperation::Matrix(Matrix { a, b, c, d, e, f }))
                        } else {
                            // Non-standard prefixed matrix parsing for -moz-transform.
                            let e = LoPoNumber::parse(context, input)?;
                            input.expect_comma()?;
                            let f = LoPoNumber::parse(context, input)?;
                            Ok(SpecifiedOperation::PrefixedMatrix(Matrix { a, b, c, d, e, f }))
                        }
                    },
                    "matrix3d" => {
                        let m11 = specified::parse_number(context, input)?;
                        input.expect_comma()?;
                        let m12 = specified::parse_number(context, input)?;
                        input.expect_comma()?;
                        let m13 = specified::parse_number(context, input)?;
                        input.expect_comma()?;
                        let m14 = specified::parse_number(context, input)?;
                        input.expect_comma()?;
                        let m21 = specified::parse_number(context, input)?;
                        input.expect_comma()?;
                        let m22 = specified::parse_number(context, input)?;
                        input.expect_comma()?;
                        let m23 = specified::parse_number(context, input)?;
                        input.expect_comma()?;
                        let m24 = specified::parse_number(context, input)?;
                        input.expect_comma()?;
                        let m31 = specified::parse_number(context, input)?;
                        input.expect_comma()?;
                        let m32 = specified::parse_number(context, input)?;
                        input.expect_comma()?;
                        let m33 = specified::parse_number(context, input)?;
                        input.expect_comma()?;
                        let m34 = specified::parse_number(context, input)?;
                        input.expect_comma()?;
                        if !prefixed {
                            // Standard matrix3d parsing.
                            let m41 = specified::parse_number(context, input)?;
                            input.expect_comma()?;
                            let m42 = specified::parse_number(context, input)?;
                            input.expect_comma()?;
                            let m43 = specified::parse_number(context, input)?;
                            input.expect_comma()?;
                            let m44 = specified::parse_number(context, input)?;
                            Ok(SpecifiedOperation::Matrix3D {
                                m11, m12, m13, m14,
                                m21, m22, m23, m24,
                                m31, m32, m33, m34,
                                m41, m42, m43, m44,
                            })
                        } else {
                            // Non-standard prefixed matrix parsing for -moz-transform.
                            let m41 = LoPoNumber::parse(context, input)?;
                            input.expect_comma()?;
                            let m42 = LoPoNumber::parse(context, input)?;
                            input.expect_comma()?;
                            let m43 = LengthOrNumber::parse(context, input)?;
                            input.expect_comma()?;
                            let m44 = specified::parse_number(context, input)?;
                            Ok(SpecifiedOperation::PrefixedMatrix3D {
                                m11, m12, m13, m14,
                                m21, m22, m23, m24,
                                m31, m32, m33, m34,
                                m41, m42, m43, m44,
                            })
                        }
                    },
                    "translate" => {
                        let sx = specified::LengthOrPercentage::parse(context, input)?;
                        if input.try(|input| input.expect_comma()).is_ok() {
                            let sy = specified::LengthOrPercentage::parse(context, input)?;
                            Ok(SpecifiedOperation::Translate(sx, Some(sy)))
                        } else {
                            Ok(SpecifiedOperation::Translate(sx, None))
                        }
                    },
                    "translatex" => {
                        let tx = specified::LengthOrPercentage::parse(context, input)?;
                        Ok(SpecifiedOperation::TranslateX(tx))
                    },
                    "translatey" => {
                        let ty = specified::LengthOrPercentage::parse(context, input)?;
                        Ok(SpecifiedOperation::TranslateY(ty))
                    },
                    "translatez" => {
                        let tz = specified::Length::parse(context, input)?;
                        Ok(SpecifiedOperation::TranslateZ(tz))
                    },
                    "translate3d" => {
                        let tx = specified::LengthOrPercentage::parse(context, input)?;
                        input.expect_comma()?;
                        let ty = specified::LengthOrPercentage::parse(context, input)?;
                        input.expect_comma()?;
                        let tz = specified::Length::parse(context, input)?;
                        Ok(SpecifiedOperation::Translate3D(tx, ty, tz))
                    },
                    "scale" => {
                        let sx = specified::parse_number(context, input)?;
                        if input.try(|input| input.expect_comma()).is_ok() {
                            let sy = specified::parse_number(context, input)?;
                            Ok(SpecifiedOperation::Scale(sx, Some(sy)))
                        } else {
                            Ok(SpecifiedOperation::Scale(sx, None))
                        }
                    },
                    "scalex" => {
                        let sx = specified::parse_number(context, input)?;
                        Ok(SpecifiedOperation::ScaleX(sx))
                    },
                    "scaley" => {
                        let sy = specified::parse_number(context, input)?;
                        Ok(SpecifiedOperation::ScaleY(sy))
                    },
                    "scalez" => {
                        let sz = specified::parse_number(context, input)?;
                        Ok(SpecifiedOperation::ScaleZ(sz))
                    },
                    "scale3d" => {
                        let sx = specified::parse_number(context, input)?;
                        input.expect_comma()?;
                        let sy = specified::parse_number(context, input)?;
                        input.expect_comma()?;
                        let sz = specified::parse_number(context, input)?;
                        Ok(SpecifiedOperation::Scale3D(sx, sy, sz))
                    },
                    "rotate" => {
                        let theta = specified::Angle::parse_with_unitless(context, input)?;
                        Ok(SpecifiedOperation::Rotate(theta))
                    },
                    "rotatex" => {
                        let theta = specified::Angle::parse_with_unitless(context, input)?;
                        Ok(SpecifiedOperation::RotateX(theta))
                    },
                    "rotatey" => {
                        let theta = specified::Angle::parse_with_unitless(context, input)?;
                        Ok(SpecifiedOperation::RotateY(theta))
                    },
                    "rotatez" => {
                        let theta = specified::Angle::parse_with_unitless(context, input)?;
                        Ok(SpecifiedOperation::RotateZ(theta))
                    },
                    "rotate3d" => {
                        let ax = specified::parse_number(context, input)?;
                        input.expect_comma()?;
                        let ay = specified::parse_number(context, input)?;
                        input.expect_comma()?;
                        let az = specified::parse_number(context, input)?;
                        input.expect_comma()?;
                        let theta = specified::Angle::parse_with_unitless(context, input)?;
                        // TODO(gw): Check that the axis can be normalized.
                        Ok(SpecifiedOperation::Rotate3D(ax, ay, az, theta))
                    },
                    "skew" => {
                        let ax = specified::Angle::parse_with_unitless(context, input)?;
                        if input.try(|input| input.expect_comma()).is_ok() {
                            let ay = specified::Angle::parse_with_unitless(context, input)?;
                            Ok(SpecifiedOperation::Skew(ax, Some(ay)))
                        } else {
                            Ok(SpecifiedOperation::Skew(ax, None))
                        }
                    },
                    "skewx" => {
                        let theta = specified::Angle::parse_with_unitless(context, input)?;
                        Ok(SpecifiedOperation::SkewX(theta))
                    },
                    "skewy" => {
                        let theta = specified::Angle::parse_with_unitless(context, input)?;
                        Ok(SpecifiedOperation::SkewY(theta))
                    },
                    "perspective" => {
                        let d = specified::Length::parse_non_negative(context, input)?;
                        Ok(SpecifiedOperation::Perspective(d))
                    },
                    _ => Err(()),
                };
                result
                    .map_err(|()| StyleParseError::UnexpectedFunction(function.clone()).into())
            })
        })?))
    }

    /// Parses `transform` property.
    #[inline]
    pub fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>)
                         -> Result<SpecifiedValue,ParseError<'i>> {
        parse_internal(context, input, false)
    }

    /// Parses `-moz-transform` property. This prefixed property also accepts LengthOrPercentage
    /// in the nondiagonal homogeneous components of matrix and matrix3d.
    #[inline]
    pub fn parse_prefixed<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>)
                                  -> Result<SpecifiedValue,ParseError<'i>> {
        parse_internal(context, input, true)
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
                    SpecifiedOperation::Matrix(Matrix { a, b, c, d, e, f }) => {
                        let mut comp = computed_value::ComputedMatrix::identity();
                        comp.m11 = a.to_computed_value(context);
                        comp.m12 = b.to_computed_value(context);
                        comp.m21 = c.to_computed_value(context);
                        comp.m22 = d.to_computed_value(context);
                        comp.m41 = e.to_computed_value(context);
                        comp.m42 = f.to_computed_value(context);
                        result.push(computed_value::ComputedOperation::Matrix(comp));
                    }
                    SpecifiedOperation::PrefixedMatrix(Matrix { a, b, c, d, ref e, ref f }) => {
                        let mut comp = computed_value::ComputedMatrixWithPercents::identity();
                        comp.m11 = a.to_computed_value(context);
                        comp.m12 = b.to_computed_value(context);
                        comp.m21 = c.to_computed_value(context);
                        comp.m22 = d.to_computed_value(context);
                        comp.m41 = lopon_to_lop(&e.to_computed_value(context));
                        comp.m42 = lopon_to_lop(&f.to_computed_value(context));
                        result.push(computed_value::ComputedOperation::MatrixWithPercents(comp));
                    }
                    SpecifiedOperation::Matrix3D {
                        m11, m12, m13, m14,
                        m21, m22, m23, m24,
                        m31, m32, m33, m34,
                        ref m41, ref m42, ref m43, m44 } => {
                            let comp = computed_value::ComputedMatrix {
                                m11: m11.to_computed_value(context),
                                m12: m12.to_computed_value(context),
                                m13: m13.to_computed_value(context),
                                m14: m14.to_computed_value(context),
                                m21: m21.to_computed_value(context),
                                m22: m22.to_computed_value(context),
                                m23: m23.to_computed_value(context),
                                m24: m24.to_computed_value(context),
                                m31: m31.to_computed_value(context),
                                m32: m32.to_computed_value(context),
                                m33: m33.to_computed_value(context),
                                m34: m34.to_computed_value(context),
                                m41: m41.to_computed_value(context),
                                m42: m42.to_computed_value(context),
                                m43: m43.to_computed_value(context),
                                m44: m44.to_computed_value(context),
                            };
                        result.push(computed_value::ComputedOperation::Matrix(comp));
                    }
                    SpecifiedOperation::PrefixedMatrix3D {
                        m11, m12, m13, m14,
                        m21, m22, m23, m24,
                        m31, m32, m33, m34,
                        ref m41, ref m42, ref m43, m44 } => {
                            let comp = computed_value::ComputedMatrixWithPercents {
                                m11: m11.to_computed_value(context),
                                m12: m12.to_computed_value(context),
                                m13: m13.to_computed_value(context),
                                m14: m14.to_computed_value(context),
                                m21: m21.to_computed_value(context),
                                m22: m22.to_computed_value(context),
                                m23: m23.to_computed_value(context),
                                m24: m24.to_computed_value(context),
                                m31: m31.to_computed_value(context),
                                m32: m32.to_computed_value(context),
                                m33: m33.to_computed_value(context),
                                m34: m34.to_computed_value(context),
                                m41: lopon_to_lop(&m41.to_computed_value(context)),
                                m42: lopon_to_lop(&m42.to_computed_value(context)),
                                m43: lon_to_length(&m43.to_computed_value(context)),
                                m44: m44.to_computed_value(context),
                            };
                        result.push(computed_value::ComputedOperation::MatrixWithPercents(comp));
                    }
                    SpecifiedOperation::Translate(ref tx, None) => {
                        let tx = tx.to_computed_value(context);
                        result.push(computed_value::ComputedOperation::Translate(
                            tx,
                            computed::length::LengthOrPercentage::zero(),
                            computed::length::Length::new(0)));
                    }
                    SpecifiedOperation::Translate(ref tx, Some(ref ty)) => {
                        let tx = tx.to_computed_value(context);
                        let ty = ty.to_computed_value(context);
                        result.push(computed_value::ComputedOperation::Translate(
                            tx,
                            ty,
                            computed::length::Length::new(0)));
                    }
                    SpecifiedOperation::TranslateX(ref tx) => {
                        let tx = tx.to_computed_value(context);
                        result.push(computed_value::ComputedOperation::Translate(
                            tx,
                            computed::length::LengthOrPercentage::zero(),
                            computed::length::Length::new(0)));
                    }
                    SpecifiedOperation::TranslateY(ref ty) => {
                        let ty = ty.to_computed_value(context);
                        result.push(computed_value::ComputedOperation::Translate(
                            computed::length::LengthOrPercentage::zero(),
                            ty,
                            computed::length::Length::new(0)));
                    }
                    SpecifiedOperation::TranslateZ(ref tz) => {
                        let tz = tz.to_computed_value(context);
                        result.push(computed_value::ComputedOperation::Translate(
                            computed::length::LengthOrPercentage::zero(),
                            computed::length::LengthOrPercentage::zero(),
                            tz));
                    }
                    SpecifiedOperation::Translate3D(ref tx, ref ty, ref tz) => {
                        let tx = tx.to_computed_value(context);
                        let ty = ty.to_computed_value(context);
                        let tz = tz.to_computed_value(context);
                        result.push(computed_value::ComputedOperation::Translate(tx, ty, tz));
                    }
                    SpecifiedOperation::Scale(factor, None) => {
                        let factor = factor.to_computed_value(context);
                        result.push(computed_value::ComputedOperation::Scale(factor, factor, 1.0));
                    }
                    SpecifiedOperation::Scale(sx, Some(sy)) => {
                        let sx = sx.to_computed_value(context);
                        let sy = sy.to_computed_value(context);
                        result.push(computed_value::ComputedOperation::Scale(sx, sy, 1.0));
                    }
                    SpecifiedOperation::ScaleX(sx) => {
                        let sx = sx.to_computed_value(context);
                        result.push(computed_value::ComputedOperation::Scale(sx, 1.0, 1.0));
                    }
                    SpecifiedOperation::ScaleY(sy) => {
                        let sy = sy.to_computed_value(context);
                        result.push(computed_value::ComputedOperation::Scale(1.0, sy, 1.0));
                    }
                    SpecifiedOperation::ScaleZ(sz) => {
                        let sz = sz.to_computed_value(context);
                        result.push(computed_value::ComputedOperation::Scale(1.0, 1.0, sz));
                    }
                    SpecifiedOperation::Scale3D(sx, sy, sz) => {
                        let sx = sx.to_computed_value(context);
                        let sy = sy.to_computed_value(context);
                        let sz = sz.to_computed_value(context);
                        result.push(computed_value::ComputedOperation::Scale(sx, sy, sz));
                    }
                    SpecifiedOperation::Rotate(theta) => {
                        let theta = theta.to_computed_value(context);
                        result.push(computed_value::ComputedOperation::Rotate(0.0, 0.0, 1.0, theta));
                    }
                    SpecifiedOperation::RotateX(theta) => {
                        let theta = theta.to_computed_value(context);
                        result.push(computed_value::ComputedOperation::Rotate(1.0, 0.0, 0.0, theta));
                    }
                    SpecifiedOperation::RotateY(theta) => {
                        let theta = theta.to_computed_value(context);
                        result.push(computed_value::ComputedOperation::Rotate(0.0, 1.0, 0.0, theta));
                    }
                    SpecifiedOperation::RotateZ(theta) => {
                        let theta = theta.to_computed_value(context);
                        result.push(computed_value::ComputedOperation::Rotate(0.0, 0.0, 1.0, theta));
                    }
                    SpecifiedOperation::Rotate3D(ax, ay, az, theta) => {
                        let ax = ax.to_computed_value(context);
                        let ay = ay.to_computed_value(context);
                        let az = az.to_computed_value(context);
                        let theta = theta.to_computed_value(context);
                        let len = (ax * ax + ay * ay + az * az).sqrt();
                        result.push(computed_value::ComputedOperation::Rotate(ax / len, ay / len, az / len, theta));
                    }
                    SpecifiedOperation::Skew(theta_x, None) => {
                        let theta_x = theta_x.to_computed_value(context);
                        result.push(computed_value::ComputedOperation::Skew(theta_x, computed::Angle::zero()));
                    }
                    SpecifiedOperation::Skew(theta_x, Some(theta_y)) => {
                        let theta_x = theta_x.to_computed_value(context);
                        let theta_y = theta_y.to_computed_value(context);
                        result.push(computed_value::ComputedOperation::Skew(theta_x, theta_y));
                    }
                    SpecifiedOperation::SkewX(theta_x) => {
                        let theta_x = theta_x.to_computed_value(context);
                        result.push(computed_value::ComputedOperation::Skew(theta_x, computed::Angle::zero()));
                    }
                    SpecifiedOperation::SkewY(theta_y) => {
                        let theta_y = theta_y.to_computed_value(context);
                        result.push(computed_value::ComputedOperation::Skew(computed::Angle::zero(), theta_y));
                    }
                    SpecifiedOperation::Perspective(ref d) => {
                        result.push(computed_value::ComputedOperation::Perspective(d.to_computed_value(context)));
                    }
                    SpecifiedOperation::InterpolateMatrix { ref from_list, ref to_list, progress } => {
                        result.push(computed_value::ComputedOperation::InterpolateMatrix {
                            from_list: from_list.to_computed_value(context),
                            to_list: to_list.to_computed_value(context),
                            progress: progress
                        });
                    }
                    SpecifiedOperation::AccumulateMatrix { ref from_list, ref to_list, count } => {
                        result.push(computed_value::ComputedOperation::AccumulateMatrix {
                            from_list: from_list.to_computed_value(context),
                            to_list: to_list.to_computed_value(context),
                            count: count.value()
                        });
                    }
                };
            }

            computed_value::T(Some(result))
        }

        #[inline]
        fn from_computed_value(computed: &computed_value::T) -> Self {
            SpecifiedValue(computed.0.as_ref().map(|computed| {
                let mut result = vec![];
                for operation in computed {
                    match *operation {
                        computed_value::ComputedOperation::Matrix(ref computed) => {
                            result.push(SpecifiedOperation::Matrix3D {
                                m11: Number::from_computed_value(&computed.m11),
                                m12: Number::from_computed_value(&computed.m12),
                                m13: Number::from_computed_value(&computed.m13),
                                m14: Number::from_computed_value(&computed.m14),
                                m21: Number::from_computed_value(&computed.m21),
                                m22: Number::from_computed_value(&computed.m22),
                                m23: Number::from_computed_value(&computed.m23),
                                m24: Number::from_computed_value(&computed.m24),
                                m31: Number::from_computed_value(&computed.m31),
                                m32: Number::from_computed_value(&computed.m32),
                                m33: Number::from_computed_value(&computed.m33),
                                m34: Number::from_computed_value(&computed.m34),
                                m41: Number::from_computed_value(&computed.m41),
                                m42: Number::from_computed_value(&computed.m42),
                                m43: Number::from_computed_value(&computed.m43),
                                m44: Number::from_computed_value(&computed.m44),
                            });
                        }
                        computed_value::ComputedOperation::MatrixWithPercents(ref computed) => {
                            result.push(SpecifiedOperation::PrefixedMatrix3D {
                                m11: Number::from_computed_value(&computed.m11),
                                m12: Number::from_computed_value(&computed.m12),
                                m13: Number::from_computed_value(&computed.m13),
                                m14: Number::from_computed_value(&computed.m14),
                                m21: Number::from_computed_value(&computed.m21),
                                m22: Number::from_computed_value(&computed.m22),
                                m23: Number::from_computed_value(&computed.m23),
                                m24: Number::from_computed_value(&computed.m24),
                                m31: Number::from_computed_value(&computed.m31),
                                m32: Number::from_computed_value(&computed.m32),
                                m33: Number::from_computed_value(&computed.m33),
                                m34: Number::from_computed_value(&computed.m34),
                                m41: Either::Second(LengthOrPercentage::from_computed_value(&computed.m41)),
                                m42: Either::Second(LengthOrPercentage::from_computed_value(&computed.m42)),
                                m43: LengthOrNumber::from_computed_value(&Either::First(computed.m43)),
                                m44: Number::from_computed_value(&computed.m44),
                            });
                        }
                        computed_value::ComputedOperation::Translate(ref tx, ref ty, ref tz) => {
                            // XXXManishearth we lose information here; perhaps we should try to
                            // recover the original function? Not sure if this can be observed.
                            result.push(SpecifiedOperation::Translate3D(
                                              ToComputedValue::from_computed_value(tx),
                                              ToComputedValue::from_computed_value(ty),
                                              ToComputedValue::from_computed_value(tz)));
                        }
                        computed_value::ComputedOperation::Scale(ref sx, ref sy, ref sz) => {
                            result.push(SpecifiedOperation::Scale3D(
                                    Number::from_computed_value(sx),
                                    Number::from_computed_value(sy),
                                    Number::from_computed_value(sz)));
                        }
                        computed_value::ComputedOperation::Rotate(ref ax, ref ay, ref az, ref theta) => {
                            result.push(SpecifiedOperation::Rotate3D(
                                    Number::from_computed_value(ax),
                                    Number::from_computed_value(ay),
                                    Number::from_computed_value(az),
                                    specified::Angle::from_computed_value(theta)));
                        }
                        computed_value::ComputedOperation::Skew(ref theta_x, ref theta_y) => {
                            result.push(SpecifiedOperation::Skew(
                                    specified::Angle::from_computed_value(theta_x),
                                    Some(specified::Angle::from_computed_value(theta_y))))
                        }
                        computed_value::ComputedOperation::Perspective(ref d) => {
                            result.push(SpecifiedOperation::Perspective(
                                ToComputedValue::from_computed_value(d)
                            ));
                        }
                        computed_value::ComputedOperation::InterpolateMatrix { ref from_list,
                                                                               ref to_list,
                                                                               progress } => {
                            result.push(SpecifiedOperation::InterpolateMatrix {
                                from_list: SpecifiedValue::from_computed_value(from_list),
                                to_list: SpecifiedValue::from_computed_value(to_list),
                                progress: progress
                            });
                        }
                        computed_value::ComputedOperation::AccumulateMatrix { ref from_list,
                                                                              ref to_list,
                                                                              count } => {
                            result.push(SpecifiedOperation::AccumulateMatrix {
                                from_list: SpecifiedValue::from_computed_value(from_list),
                                to_list: SpecifiedValue::from_computed_value(to_list),
                                count: Integer::new(count)
                            });
                        }
                    };
                }
                result
            }).unwrap_or(Vec::new()))
        }
    }

    // Converts computed LengthOrPercentageOrNumber into computed
    // LengthOrPercentage. Number maps into Length
    fn lopon_to_lop(value: &ComputedLoPoNumber) -> ComputedLoP {
        match *value {
            Either::First(number) => ComputedLoP::Length(Au::from_f32_px(number)),
            Either::Second(length_or_percentage) => length_or_percentage,
        }
    }

    // Converts computed LengthOrNumber into computed Length.
    // Number maps into Length.
    fn lon_to_length(value: &ComputedLoN) -> ComputedLength {
        match *value {
            Either::First(length) => length,
            Either::Second(number) => Au::from_f32_px(number),
        }
    }
</%helpers:longhand>

// CSSOM View Module
// https://www.w3.org/TR/cssom-view-1/
${helpers.single_keyword("scroll-behavior",
                         "auto smooth",
                         products="gecko",
                         spec="https://drafts.csswg.org/cssom-view/#propdef-scroll-behavior",
                         animation_value_type="discrete")}

${helpers.single_keyword("scroll-snap-type-x",
                         "none mandatory proximity",
                         products="gecko",
                         gecko_constant_prefix="NS_STYLE_SCROLL_SNAP_TYPE",
                         spec="Nonstandard (https://developer.mozilla.org/en-US/docs/Web/CSS/scroll-snap-type-x)",
                         animation_value_type="discrete")}

<%helpers:longhand products="gecko" name="scroll-snap-type-y" animation_value_type="discrete"
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
                         flags="CREATES_STACKING_CONTEXT",
                         animation_value_type="discrete")}

// TODO add support for logical values recto and verso
${helpers.single_keyword("page-break-after",
                         "auto always avoid left right",
                         products="gecko",
                         spec="https://drafts.csswg.org/css2/page.html#propdef-page-break-after",
                         animation_value_type="discrete")}
${helpers.single_keyword("page-break-before",
                         "auto always avoid left right",
                         products="gecko",
                         spec="https://drafts.csswg.org/css2/page.html#propdef-page-break-before",
                         animation_value_type="discrete")}
${helpers.single_keyword("page-break-inside",
                         "auto avoid",
                         products="gecko",
                         gecko_ffi_name="mBreakInside",
                         gecko_constant_prefix="NS_STYLE_PAGE_BREAK",
                         spec="https://drafts.csswg.org/css2/page.html#propdef-page-break-inside",
                         animation_value_type="discrete")}

// CSS Basic User Interface Module Level 3
// http://dev.w3.org/csswg/css-ui
// FIXME support logical values `block` and `inline` (https://drafts.csswg.org/css-logical-props/#resize)
//
// This is APPLIES_TO_PLACEHOLDER so we can override, in the UA sheet, the
// 'resize' property we'd inherit from textarea otherwise.  Basically, just
// makes the UA rules easier to write.
${helpers.single_keyword("resize",
                         "none both horizontal vertical",
                         products="gecko",
                         spec="https://drafts.csswg.org/css-ui/#propdef-resize",
                         flags="APPLIES_TO_PLACEHOLDER",
                         animation_value_type="discrete")}


${helpers.predefined_type("perspective",
                          "LengthOrNone",
                          "Either::Second(None_)",
                          "parse_non_negative_length",
                          gecko_ffi_name="mChildPerspective",
                          spec="https://drafts.csswg.org/css-transforms/#perspective",
                          extra_prefixes="moz webkit",
                          flags="CREATES_STACKING_CONTEXT FIXPOS_CB",
                          animation_value_type="ComputedValue")}

${helpers.predefined_type("perspective-origin",
                          "position::Position",
                          "computed::position::Position::center()",
                          boxed="True",
                          extra_prefixes="moz webkit",
                          spec="https://drafts.csswg.org/css-transforms-2/#perspective-origin-property",
                          animation_value_type="ComputedValue")}

${helpers.single_keyword("backface-visibility",
                         "visible hidden",
                         spec="https://drafts.csswg.org/css-transforms/#backface-visibility-property",
                         extra_prefixes="moz webkit",
                         animation_value_type="discrete")}

${helpers.single_keyword("transform-box",
                         "border-box fill-box view-box",
                         gecko_enum_prefix="StyleGeometryBox",
                         products="gecko",
                         spec="https://drafts.csswg.org/css-transforms/#transform-box",
                         gecko_inexhaustive="True",
                         animation_value_type="discrete")}

// `auto` keyword is not supported in gecko yet.
${helpers.single_keyword("transform-style",
                         "auto flat preserve-3d" if product == "servo" else
                         "flat preserve-3d",
                         spec="https://drafts.csswg.org/css-transforms/#transform-style-property",
                         extra_prefixes="moz webkit",
                         flags="CREATES_STACKING_CONTEXT FIXPOS_CB",
                         animation_value_type="discrete")}

${helpers.predefined_type("transform-origin",
                          "TransformOrigin",
                          "computed::TransformOrigin::initial_value()",
                          animation_value_type="ComputedValue",
                          extra_prefixes="moz webkit",
                          boxed=True,
                          spec="https://drafts.csswg.org/css-transforms/#transform-origin-property")}

// FIXME: `size` and `content` values are not implemented and `strict` is implemented
// like `content`(layout style paint) in gecko. We should implement `size` and `content`,
// also update the glue once they are implemented in gecko.
<%helpers:longhand name="contain" animation_value_type="discrete" products="gecko" need_clone="True"
                   flags="FIXPOS_CB"
                   spec="https://drafts.csswg.org/css-contain/#contain-property">
    use std::fmt;
    use style_traits::ToCss;
    use values::computed::ComputedValueAsSpecified;

    impl ComputedValueAsSpecified for SpecifiedValue {}
    no_viewport_percentage!(SpecifiedValue);

    pub mod computed_value {
        pub type T = super::SpecifiedValue;
    }

    bitflags! {
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub flags SpecifiedValue: u8 {
            const LAYOUT = 0x01,
            const STYLE = 0x02,
            const PAINT = 0x04,
            const STRICT = 0x8,
            const STRICT_BITS = LAYOUT.bits | STYLE.bits | PAINT.bits,
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

            let mut has_any = false;
            macro_rules! maybe_write_value {
                ($ident:ident => $str:expr) => {
                    if self.contains($ident) {
                        if has_any {
                            dest.write_str(" ")?;
                        }
                        has_any = true;
                        dest.write_str($str)?;
                    }
                }
            }
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
    pub fn parse<'i, 't>(_context: &ParserContext, input: &mut Parser<'i, 't>)
                         -> Result<SpecifiedValue, ParseError<'i>> {
        let mut result = SpecifiedValue::empty();

        if input.try(|input| input.expect_ident_matching("none")).is_ok() {
            return Ok(result)
        }
        if input.try(|input| input.expect_ident_matching("strict")).is_ok() {
            result.insert(STRICT | STRICT_BITS);
            return Ok(result)
        }

        while let Ok(name) = input.try(|i| i.expect_ident_cloned()) {
            let flag = match_ignore_ascii_case! { &name,
                "layout" => Some(LAYOUT),
                "style" => Some(STYLE),
                "paint" => Some(PAINT),
                _ => None
            };
            let flag = match flag {
                Some(flag) if !result.contains(flag) => flag,
                _ => return Err(SelectorParseError::UnexpectedIdent(name.clone()).into())
            };
            result.insert(flag);
        }

        if !result.is_empty() {
            Ok(result)
        } else {
            Err(StyleParseError::UnspecifiedError.into())
        }
    }
</%helpers:longhand>

// Non-standard
${helpers.single_keyword("-moz-appearance",
                         """none button button-arrow-down button-arrow-next button-arrow-previous button-arrow-up
                            button-bevel button-focus caret checkbox checkbox-container checkbox-label checkmenuitem
                            dialog dualbutton groupbox listbox listitem menuarrow menubar menucheckbox menuimage
                            menuitem menuitemtext menulist menulist-button menulist-text menulist-textfield menupopup
                            menuradio menuseparator meterbar meterchunk number-input progressbar progressbar-vertical
                            progresschunk progresschunk-vertical radio radio-container radio-label radiomenuitem range
                            range-thumb resizer resizerpanel scale-horizontal scalethumbend scalethumb-horizontal
                            scalethumbstart scalethumbtick scalethumb-vertical scale-vertical scrollbar
                            scrollbar-horizontal scrollbar-small scrollbar-vertical scrollbarbutton-down
                            scrollbarbutton-left scrollbarbutton-right scrollbarbutton-up scrollbarthumb-horizontal
                            scrollbarthumb-vertical scrollbartrack-horizontal scrollbartrack-vertical searchfield
                            separator spinner spinner-downbutton spinner-textfield spinner-upbutton splitter statusbar
                            statusbarpanel tab tabpanel tabpanels tab-scroll-arrow-back tab-scroll-arrow-forward
                            textfield textfield-multiline toolbar toolbarbutton toolbarbutton-dropdown toolbargripper
                            toolbox tooltip treeheader treeheadercell treeheadersortarrow treeitem treeline treetwisty
                            treetwistyopen treeview window
                            -moz-gtk-info-bar -moz-mac-active-source-list-selection -moz-mac-disclosure-button-closed
                            -moz-mac-disclosure-button-open -moz-mac-fullscreen-button -moz-mac-help-button
                            -moz-mac-source-list -moz-mac-source-list-selection -moz-mac-vibrancy-dark
                            -moz-mac-vibrancy-light -moz-win-borderless-glass -moz-win-browsertabbar-toolbox
                            -moz-win-communications-toolbox -moz-win-exclude-glass -moz-win-glass -moz-win-media-toolbox
                            -moz-window-button-box -moz-window-button-box-maximized -moz-window-button-close
                            -moz-window-button-maximize -moz-window-button-minimize -moz-window-button-restore
                            -moz-window-frame-bottom -moz-window-frame-left -moz-window-frame-right -moz-window-titlebar
                            -moz-window-titlebar-maximized
                         """,
                         gecko_ffi_name="mAppearance",
                         gecko_constant_prefix="ThemeWidgetType_NS_THEME",
                         products="gecko",
                         spec="Nonstandard (https://developer.mozilla.org/en-US/docs/Web/CSS/-moz-appearance)",
                         animation_value_type="none")}

${helpers.predefined_type("-moz-binding", "UrlOrNone", "Either::Second(None_)",
                          products="gecko",
                          boxed="True" if product == "gecko" else "False",
                          animation_value_type="none",
                          gecko_ffi_name="mBinding",
                          spec="Nonstandard (https://developer.mozilla.org/en-US/docs/Web/CSS/-moz-binding)",
                          disable_when_testing="True")}

${helpers.single_keyword("-moz-orient",
                          "inline block horizontal vertical",
                          products="gecko",
                          gecko_ffi_name="mOrient",
                          gecko_enum_prefix="StyleOrient",
                          spec="Nonstandard (https://developer.mozilla.org/en-US/docs/Web/CSS/-moz-orient)",
                          gecko_inexhaustive="True",
                          animation_value_type="discrete")}

<%helpers:longhand name="will-change" products="gecko" animation_value_type="discrete"
                   spec="https://drafts.csswg.org/css-will-change/#will-change">
    use std::fmt;
    use style_traits::ToCss;
    use values::CustomIdent;
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
        AnimateableFeatures(Vec<CustomIdent>),
    }

    impl ToCss for SpecifiedValue {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match *self {
                SpecifiedValue::Auto => dest.write_str("auto"),
                SpecifiedValue::AnimateableFeatures(ref features) => {
                    let (first, rest) = features.split_first().unwrap();
                    first.to_css(dest)?;
                    for feature in rest {
                        dest.write_str(", ")?;
                        feature.to_css(dest)?;
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
    pub fn parse<'i, 't>(_context: &ParserContext, input: &mut Parser<'i, 't>)
                         -> Result<SpecifiedValue, ParseError<'i>> {
        if input.try(|input| input.expect_ident_matching("auto")).is_ok() {
            Ok(computed_value::T::Auto)
        } else {
            input.parse_comma_separated(|i| {
                CustomIdent::from_ident(i.expect_ident()?, &[
                    "will-change",
                    "none",
                    "all",
                    "auto",
                ])
            }).map(SpecifiedValue::AnimateableFeatures)
        }
    }
</%helpers:longhand>

${helpers.predefined_type("shape-outside", "basic_shape::FloatAreaShape",
                          "generics::basic_shape::ShapeSource::None",
                          products="gecko", boxed="True",
                          animation_value_type="none",
                          flags="APPLIES_TO_FIRST_LETTER",
                          spec="https://drafts.csswg.org/css-shapes/#shape-outside-property")}

<%helpers:longhand name="touch-action"
                   products="gecko"
                   animation_value_type="discrete"
                   disable_when_testing="True"
                   spec="https://compat.spec.whatwg.org/#touch-action">
    use gecko_bindings::structs;
    use std::fmt;
    use style_traits::ToCss;
    use values::computed::ComputedValueAsSpecified;

    impl ComputedValueAsSpecified for SpecifiedValue {}
    no_viewport_percentage!(SpecifiedValue);

    pub mod computed_value {
        pub use super::SpecifiedValue as T;
    }

    bitflags! {
        /// These constants match Gecko's `NS_STYLE_TOUCH_ACTION_*` constants.
        pub flags SpecifiedValue: u8 {
            const TOUCH_ACTION_NONE = structs::NS_STYLE_TOUCH_ACTION_NONE as u8,
            const TOUCH_ACTION_AUTO = structs::NS_STYLE_TOUCH_ACTION_AUTO as u8,
            const TOUCH_ACTION_PAN_X = structs::NS_STYLE_TOUCH_ACTION_PAN_X as u8,
            const TOUCH_ACTION_PAN_Y = structs::NS_STYLE_TOUCH_ACTION_PAN_Y as u8,
            const TOUCH_ACTION_MANIPULATION = structs::NS_STYLE_TOUCH_ACTION_MANIPULATION as u8,
        }
    }

    impl ToCss for SpecifiedValue {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match *self {
                TOUCH_ACTION_NONE => dest.write_str("none"),
                TOUCH_ACTION_AUTO => dest.write_str("auto"),
                TOUCH_ACTION_MANIPULATION => dest.write_str("manipulation"),
                _ if self.contains(TOUCH_ACTION_PAN_X | TOUCH_ACTION_PAN_Y) => {
                    dest.write_str("pan-x pan-y")
                },
                _ if self.contains(TOUCH_ACTION_PAN_X) => {
                    dest.write_str("pan-x")
                },
                _ if self.contains(TOUCH_ACTION_PAN_Y) => {
                    dest.write_str("pan-y")
                },
                _ => panic!("invalid touch-action value"),
            }
        }
    }

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        TOUCH_ACTION_AUTO
    }

    pub fn parse<'i, 't>(_context: &ParserContext, input: &mut Parser<'i, 't>)
                         -> Result<SpecifiedValue, ParseError<'i>> {
        // FIXME: remove clone() when lifetimes are non-lexical
        try_match_ident_ignore_ascii_case! { input.expect_ident()?.clone(),
            "auto" => Ok(TOUCH_ACTION_AUTO),
            "none" => Ok(TOUCH_ACTION_NONE),
            "manipulation" => Ok(TOUCH_ACTION_MANIPULATION),
            "pan-x" => {
                if input.try(|i| i.expect_ident_matching("pan-y")).is_ok() {
                    Ok(TOUCH_ACTION_PAN_X | TOUCH_ACTION_PAN_Y)
                } else {
                    Ok(TOUCH_ACTION_PAN_X)
                }
            },
            "pan-y" => {
                if input.try(|i| i.expect_ident_matching("pan-x")).is_ok() {
                    Ok(TOUCH_ACTION_PAN_X | TOUCH_ACTION_PAN_Y)
                } else {
                    Ok(TOUCH_ACTION_PAN_Y)
                }
            },
        }
    }

    #[cfg(feature = "gecko")]
    impl_bitflags_conversions!(SpecifiedValue);
</%helpers:longhand>
