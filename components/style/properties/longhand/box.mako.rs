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
    use style_traits::ToCss;

    pub mod computed_value {
        pub use super::SpecifiedValue as T;
        use super::SpecifiedValue as Display;

        impl Display {
            /// Returns whether this "display" value is the display of a flex or
            /// grid container.
            ///
            /// This is used to implement various style fixups.
            pub fn is_item_container(&self) -> bool {
                matches!(*self,
                         Display::Flex
                         | Display::InlineFlex
                         % if product == "gecko":
                         | Display::Grid
                         | Display::InlineGrid
                         % endif
                )
            }

            /// Returns whether an element with this display type is a line
            /// participant, which means it may lay its children on the same
            /// line as itself.
            pub fn is_line_participant(&self) -> bool {
                matches!(*self,
                         Display::Inline
                         % if product == "gecko":
                         | Display::Contents
                         | Display::Ruby
                         | Display::RubyBaseContainer
                         % endif
                )
            }

            /// Whether `new_display` should be ignored, given a previous
            /// `old_display` value.
            ///
            /// This is used to ignore `display: -moz-box` declarations after an
            /// equivalent `display: -webkit-box` declaration, since the former
            /// has a vastly different meaning. See bug 1107378 and bug 1407701.
            ///
            /// FIXME(emilio): This is a pretty decent hack, we should try to
            /// remove it.
            pub fn should_ignore_parsed_value(
                _old_display: Self,
                _new_display: Self,
            ) -> bool {
                #[cfg(feature = "gecko")]
                {
                    match (_old_display, _new_display) {
                        (Display::WebkitBox, Display::MozBox) |
                        (Display::WebkitInlineBox, Display::MozInlineBox) => {
                            return true;
                        }
                        _ => {},
                    }
                }

                return false;
            }

            /// Returns whether this "display" value is one of the types for
            /// ruby.
            #[cfg(feature = "gecko")]
            pub fn is_ruby_type(&self) -> bool {
                matches!(*self,
                         Display::Ruby |
                         Display::RubyBase |
                         Display::RubyText |
                         Display::RubyBaseContainer |
                         Display::RubyTextContainer)
            }

            /// Returns whether this "display" value is a ruby level container.
            #[cfg(feature = "gecko")]
            pub fn is_ruby_level_container(&self) -> bool {
                matches!(*self,
                         Display::RubyBaseContainer |
                         Display::RubyTextContainer)
            }

            /// Convert this display into an equivalent block display.
            ///
            /// Also used for style adjustments.
            pub fn equivalent_block_display(&self, _is_root_element: bool) -> Self {
                match *self {
                    // Values that have a corresponding block-outside version.
                    Display::InlineTable => Display::Table,
                    Display::InlineFlex => Display::Flex,

                    % if product == "gecko":
                    Display::InlineGrid => Display::Grid,
                    Display::WebkitInlineBox => Display::WebkitBox,
                    % endif

                    // Special handling for contents and list-item on the root
                    // element for Gecko.
                    % if product == "gecko":
                    Display::Contents | Display::ListItem if _is_root_element => Display::Block,
                    % endif

                    // These are not changed by blockification.
                    Display::None |
                    Display::Block |
                    Display::Flex |
                    Display::ListItem |
                    Display::Table => *self,

                    % if product == "gecko":
                    Display::Contents |
                    Display::FlowRoot |
                    Display::Grid |
                    Display::WebkitBox => *self,
                    % endif

                    // Everything else becomes block.
                    _ => Display::Block,
                }

            }

            /// Convert this display into an inline-outside display.
            ///
            /// Ideally it should implement spec: https://drafts.csswg.org/css-display/#inlinify
            /// but the spec isn't stable enough, so we copy what Gecko does for now.
            #[cfg(feature = "gecko")]
            pub fn inlinify(&self) -> Self {
                match *self {
                    Display::Block |
                    Display::FlowRoot => Display::InlineBlock,
                    Display::Table => Display::InlineTable,
                    Display::Flex => Display::InlineFlex,
                    Display::Grid => Display::InlineGrid,
                    Display::MozBox => Display::MozInlineBox,
                    Display::MozStack => Display::MozInlineStack,
                    Display::WebkitBox => Display::WebkitInlineBox,
                    other => other,
                }
            }
        }
    }

    // FIXME(emilio): Why does this reinvent the wheel, again?
    #[allow(non_camel_case_types)]
    #[derive(Clone, Copy, Debug, Eq, Hash, MallocSizeOf, PartialEq, ToComputedValue)]
    #[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
    pub enum SpecifiedValue {
        % for value in values:
            ${to_camel_case(value)},
        % endfor
    }

    // TODO(emilio): derive.
    impl ToCss for SpecifiedValue {
        fn to_css<W>(&self, dest: &mut W) -> ::std::fmt::Result
            where W: ::std::fmt::Write,
        {
            match *self {
                % for value in values:
                    SpecifiedValue::${to_camel_case(value)} => dest.write_str("${value}"),
                % endfor
            }
        }
    }

    /// The initial display value.
    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        computed_value::T::${to_camel_case(values[0])}
    }

    /// Parse a display value.
    pub fn parse<'i, 't>(_context: &ParserContext, input: &mut Parser<'i, 't>)
                         -> Result<SpecifiedValue, ParseError<'i>> {
        try_match_ident_ignore_ascii_case! { input,
            % for value in values:
                "${value}" => {
                    Ok(computed_value::T::${to_camel_case(value)})
                },
            % endfor
            % for value in webkit_prefixed_values:
                "-webkit-${value}" => {
                    Ok(computed_value::T::${to_camel_case(value)})
                },
            % endfor
        }
    }

    % if product == "servo":
        fn cascade_property_custom(_declaration: &PropertyDeclaration,
                                   context: &mut computed::Context) {
            longhands::_servo_display_for_hypothetical_box::derive_from_display(context);
            longhands::_servo_text_decorations_in_effect::derive_from_display(context);
        }
    % endif

    ${helpers.gecko_keyword_conversion(Keyword('display', ' '.join(values),
                                               gecko_enum_prefix='StyleDisplay',
                                               gecko_strip_moz_prefix=False))}

</%helpers:longhand>

${helpers.single_keyword("-moz-top-layer", "none top",
                         gecko_constant_prefix="NS_STYLE_TOP_LAYER",
                         gecko_ffi_name="mTopLayer",
                         products="gecko", animation_value_type="none",
                         enabled_in="ua",
                         spec="Internal (not web-exposed)")}

${helpers.single_keyword("position", "static absolute relative fixed sticky",
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
                                  flags="APPLIES_TO_FIRST_LETTER"
                                  spec="https://drafts.csswg.org/css-box/#propdef-float">
    impl ToComputedValue for SpecifiedValue {
        type ComputedValue = computed_value::T;

        #[inline]
        fn to_computed_value(&self, context: &Context) -> computed_value::T {
            let ltr = context.style().writing_mode.is_bidi_ltr();
            // https://drafts.csswg.org/css-logical-props/#float-clear
            match *self {
                SpecifiedValue::InlineStart => {
                    context.rule_cache_conditions.borrow_mut()
                        .set_writing_mode_dependency(context.builder.writing_mode);
                    if ltr {
                        computed_value::T::Left
                    } else {
                        computed_value::T::Right
                    }
                }
                SpecifiedValue::InlineEnd => {
                    context.rule_cache_conditions.borrow_mut()
                        .set_writing_mode_dependency(context.builder.writing_mode);
                    if ltr {
                        computed_value::T::Right
                    } else {
                        computed_value::T::Left
                    }
                }
                % for value in "None Left Right".split():
                    SpecifiedValue::${value} => computed_value::T::${value},
                % endfor
            }
        }
        #[inline]
        fn from_computed_value(computed: &computed_value::T) -> SpecifiedValue {
            match *computed {
                % for value in "None Left Right".split():
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
    impl ToComputedValue for SpecifiedValue {
        type ComputedValue = computed_value::T;

        #[inline]
        fn to_computed_value(&self, context: &Context) -> computed_value::T {
            let ltr = context.style().writing_mode.is_bidi_ltr();
            // https://drafts.csswg.org/css-logical-props/#float-clear
            match *self {
                SpecifiedValue::InlineStart => {
                    context.rule_cache_conditions.borrow_mut()
                        .set_writing_mode_dependency(context.builder.writing_mode);
                    if ltr {
                        computed_value::T::Left
                    } else {
                        computed_value::T::Right
                    }
                }
                SpecifiedValue::InlineEnd => {
                    context.rule_cache_conditions.borrow_mut()
                        .set_writing_mode_dependency(context.builder.writing_mode);
                    if ltr {
                        computed_value::T::Right
                    } else {
                        computed_value::T::Left
                    }
                }
                % for value in "None Left Right Both".split():
                    SpecifiedValue::${value} => computed_value::T::${value},
                % endfor
            }
        }
        #[inline]
        fn from_computed_value(computed: &computed_value::T) -> SpecifiedValue {
            match *computed {
                % for value in "None Left Right Both".split():
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


${helpers.predefined_type(
    "vertical-align",
    "VerticalAlign",
    "computed::VerticalAlign::baseline()",
    animation_value_type="ComputedValue",
    flags="APPLIES_TO_FIRST_LETTER APPLIES_TO_FIRST_LINE APPLIES_TO_PLACEHOLDER",
    spec="https://www.w3.org/TR/CSS2/visudet.html#propdef-vertical-align",
)}

// CSS 2.1, Section 11 - Visual effects

${helpers.single_keyword("-servo-overflow-clip-box", "padding-box content-box",
    products="servo", animation_value_type="none", enabled_in="ua",
    spec="Internal, not web-exposed, \
          may be standardized in the future (https://developer.mozilla.org/en-US/docs/Web/CSS/overflow-clip-box)")}

% for direction in ["inline", "block"]:
    ${helpers.predefined_type(
        "overflow-clip-box-" + direction,
        "OverflowClipBox",
        "computed::OverflowClipBox::PaddingBox",
        products="gecko",
        enabled_in="ua",
        needs_context=False,
        flags="APPLIES_TO_PLACEHOLDER",
        gecko_pref="layout.css.overscroll-behavior.enabled",
        animation_value_type="discrete",
        spec="Internal, may be standardized in the future: \
              https://developer.mozilla.org/en-US/docs/Web/CSS/overflow-clip-box",
    )}
% endfor

<%
    overflow_custom_consts = { "-moz-hidden-unscrollable": "CLIP" }
%>

// FIXME(pcwalton, #2742): Implement scrolling for `scroll` and `auto`.
${helpers.single_keyword("overflow-x", "visible hidden scroll auto",
                         animation_value_type="discrete",
                         extra_gecko_values="-moz-hidden-unscrollable",
                         custom_consts=overflow_custom_consts,
                         gecko_constant_prefix="NS_STYLE_OVERFLOW",
                         flags="APPLIES_TO_PLACEHOLDER",
                         spec="https://drafts.csswg.org/css-overflow/#propdef-overflow-x")}

// FIXME(pcwalton, #2742): Implement scrolling for `scroll` and `auto`.
<%helpers:longhand name="overflow-y" animation_value_type="discrete"
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


${helpers.predefined_type(
    "animation-name",
    "AnimationName",
    "computed::AnimationName::none()",
    initial_specified_value="specified::AnimationName::none()",
    vector=True,
    need_index=True,
    animation_value_type="none",
    extra_prefixes="moz webkit",
    allowed_in_keyframe_block=False,
    spec="https://drafts.csswg.org/css-animations/#propdef-animation-name",
)}

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

${helpers.predefined_type(
    "animation-iteration-count",
    "AnimationIterationCount",
    "computed::AnimationIterationCount::one()",
    initial_specified_value="specified::AnimationIterationCount::one()",
    vector=True,
    need_index=True,
    animation_value_type="none",
    extra_prefixes="moz webkit",
    allowed_in_keyframe_block=False,
    spec="https://drafts.csswg.org/css-animations/#propdef-animation-iteration-count",
)}

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
        gecko_pref="layout.css.scroll-snap.enabled",
        products="gecko",
        spec="Nonstandard (https://www.w3.org/TR/2015/WD-css-snappoints-1-20150326/#scroll-snap-points)",
    )}
% endfor

${helpers.predefined_type("scroll-snap-destination",
                          "Position",
                          "computed::Position::zero()",
                          products="gecko",
                          gecko_pref="layout.css.scroll-snap.enabled",
                          boxed="True",
                          spec="Nonstandard (https://developer.mozilla.org/en-US/docs/Web/CSS/scroll-snap-destination)",
                          animation_value_type="discrete")}

${helpers.predefined_type(
    "scroll-snap-coordinate",
    "Position",
    "computed::Position::zero()",
    vector=True,
    products="gecko",
    gecko_pref="layout.css.scroll-snap.enabled",
    spec="Nonstandard (https://developer.mozilla.org/en-US/docs/Web/CSS/scroll-snap-destination)",
    animation_value_type="discrete",
    allow_empty="NotInitial"
)}

${helpers.predefined_type("transform", "Transform",
                          "generics::transform::Transform::none()",
                          extra_prefixes="webkit",
                          animation_value_type="ComputedValue",
                          gecko_ffi_name="mSpecifiedTransform",
                          flags="CREATES_STACKING_CONTEXT FIXPOS_CB",
                          spec="https://drafts.csswg.org/css-transforms/#propdef-transform")}

// CSSOM View Module
// https://www.w3.org/TR/cssom-view-1/
${helpers.single_keyword("scroll-behavior",
                         "auto smooth",
                         gecko_pref="layout.css.scroll-behavior.property-enabled",
                         products="gecko",
                         spec="https://drafts.csswg.org/cssom-view/#propdef-scroll-behavior",
                         animation_value_type="discrete")}

% for axis in ["x", "y"]:
    ${helpers.predefined_type(
        "scroll-snap-type-" + axis,
        "ScrollSnapType",
        "computed::ScrollSnapType::None",
        products="gecko",
        needs_context=False,
        gecko_pref="layout.css.scroll-snap.enabled",
        spec="Nonstandard (https://developer.mozilla.org/en-US/docs/Web/CSS/scroll-snap-type-x)",
        animation_value_type="discrete"
    )}
% endfor

% for axis in ["x", "y"]:
    ${helpers.predefined_type(
        "overscroll-behavior-" + axis,
        "OverscrollBehavior",
        "computed::OverscrollBehavior::Auto",
        products="gecko",
        needs_context=False,
        gecko_pref="layout.css.overscroll-behavior.enabled",
        spec="https://wicg.github.io/overscroll-behavior/#overscroll-behavior-properties",
        animation_value_type="discrete"
    )}
% endfor

// Compositing and Blending Level 1
// http://www.w3.org/TR/compositing-1/
${helpers.single_keyword("isolation",
                         "auto isolate",
                         products="gecko",
                         gecko_pref="layout.css.isolation.enabled",
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
                         gecko_pref="svg.transform-box.enabled",
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
                          gecko_ffi_name="mTransformOrigin",
                          boxed=True,
                          spec="https://drafts.csswg.org/css-transforms/#transform-origin-property")}

// FIXME: `size` and `content` values are not implemented and `strict` is implemented
// like `content`(layout style paint) in gecko. We should implement `size` and `content`,
// also update the glue once they are implemented in gecko.
<%helpers:longhand name="contain" animation_value_type="discrete" products="gecko"
                   flags="FIXPOS_CB"
                   gecko_pref="layout.css.contain.enabled",
                   spec="https://drafts.csswg.org/css-contain/#contain-property">
    use std::fmt;
    use style_traits::ToCss;

    pub mod computed_value {
        pub type T = super::SpecifiedValue;
    }

    bitflags! {
        #[derive(MallocSizeOf, ToComputedValue)]
        pub struct SpecifiedValue: u8 {
            const LAYOUT = 0x01;
            const STYLE = 0x02;
            const PAINT = 0x04;
            const STRICT = 0x8;
            const STRICT_BITS = SpecifiedValue::LAYOUT.bits | SpecifiedValue::STYLE.bits | SpecifiedValue::PAINT.bits;
        }
    }

    impl ToCss for SpecifiedValue {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            if self.is_empty() {
                return dest.write_str("none")
            }
            if self.contains(SpecifiedValue::STRICT) {
                return dest.write_str("strict")
            }

            let mut has_any = false;
            macro_rules! maybe_write_value {
                ($ident:path => $str:expr) => {
                    if self.contains($ident) {
                        if has_any {
                            dest.write_str(" ")?;
                        }
                        has_any = true;
                        dest.write_str($str)?;
                    }
                }
            }
            maybe_write_value!(SpecifiedValue::LAYOUT => "layout");
            maybe_write_value!(SpecifiedValue::STYLE => "style");
            maybe_write_value!(SpecifiedValue::PAINT => "paint");

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
            result.insert(SpecifiedValue::STRICT | SpecifiedValue::STRICT_BITS);
            return Ok(result)
        }

        while let Ok(name) = input.try(|i| i.expect_ident_cloned()) {
            let flag = match_ignore_ascii_case! { &name,
                "layout" => Some(SpecifiedValue::LAYOUT),
                "style" => Some(SpecifiedValue::STYLE),
                "paint" => Some(SpecifiedValue::PAINT),
                _ => None
            };
            let flag = match flag {
                Some(flag) if !result.contains(flag) => flag,
                _ => return Err(input.new_custom_error(SelectorParseErrorKind::UnexpectedIdent(name.clone())))
            };
            result.insert(flag);
        }

        if !result.is_empty() {
            Ok(result)
        } else {
            Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
        }
    }
</%helpers:longhand>

// Non-standard
${helpers.single_keyword("-moz-appearance",
                         """none button button-arrow-down button-arrow-next button-arrow-previous button-arrow-up
                            button-bevel button-focus caret checkbox checkbox-container checkbox-label checkmenuitem
                            dialog dualbutton groupbox inner-spin-button listbox listitem menuarrow menubar menucheckbox
                            menuimage menuitem menuitemtext menulist menulist-button menulist-text menulist-textfield
                            menupopup menuradio menuseparator meterbar meterchunk number-input progressbar
                            progressbar-vertical progresschunk progresschunk-vertical radio radio-container radio-label
                            radiomenuitem range range-thumb resizer resizerpanel scale-horizontal scalethumbend
                            scalethumb-horizontal scalethumbstart scalethumbtick scalethumb-vertical scale-vertical
                            scrollbar scrollbar-horizontal scrollbar-small scrollbar-vertical scrollbarbutton-down
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
                            -moz-mac-vibrancy-light -moz-mac-vibrant-titlebar-light -moz-mac-vibrant-titlebar-dark
                            -moz-win-borderless-glass -moz-win-browsertabbar-toolbox
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
                         animation_value_type="discrete")}

${helpers.predefined_type("-moz-binding", "UrlOrNone", "Either::Second(None_)",
                          products="gecko",
                          boxed="True" if product == "gecko" else "False",
                          animation_value_type="none",
                          gecko_ffi_name="mBinding",
                          spec="Nonstandard (https://developer.mozilla.org/en-US/docs/Web/CSS/-moz-binding)")}

${helpers.single_keyword("-moz-orient",
                          "inline block horizontal vertical",
                          products="gecko",
                          gecko_ffi_name="mOrient",
                          gecko_enum_prefix="StyleOrient",
                          spec="Nonstandard (https://developer.mozilla.org/en-US/docs/Web/CSS/-moz-orient)",
                          animation_value_type="discrete")}

${helpers.predefined_type(
    "will-change",
    "WillChange",
    "computed::WillChange::auto()",
    products="gecko",
    animation_value_type="discrete",
    spec="https://drafts.csswg.org/css-will-change/#will-change"
)}

${helpers.predefined_type(
    "shape-image-threshold", "Opacity", "0.0",
    products="gecko",
    gecko_pref="layout.css.shape-outside.enabled",
    animation_value_type="ComputedValue",
    spec="https://drafts.csswg.org/css-shapes/#shape-image-threshold-property",
)}

${helpers.predefined_type(
    "shape-outside",
    "basic_shape::FloatAreaShape",
    "generics::basic_shape::ShapeSource::None",
    products="gecko",
    boxed=True,
    gecko_pref="layout.css.shape-outside.enabled",
    animation_value_type="ComputedValue",
    flags="APPLIES_TO_FIRST_LETTER",
    spec="https://drafts.csswg.org/css-shapes/#shape-outside-property",
)}

${helpers.predefined_type(
    "touch-action",
    "TouchAction",
    "computed::TouchAction::auto()",
    products="gecko",
    gecko_pref="layout.css.touch_action.enabled",
    animation_value_type="discrete",
    spec="https://compat.spec.whatwg.org/#touch-action",
)}
