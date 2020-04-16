/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />
<% from data import ALL_AXES, Keyword, Method, to_rust_ident, to_camel_case%>

<% data.new_style_struct("Box",
                         inherited=False,
                         gecko_name="Display") %>

${helpers.predefined_type(
    "display",
    "Display",
    "computed::Display::inline()",
    engines="gecko servo-2013 servo-2020",
    initial_specified_value="specified::Display::inline()",
    animation_value_type="discrete",
    spec="https://drafts.csswg.org/css-display/#propdef-display",
    servo_restyle_damage="rebuild_and_reflow",
)}

${helpers.single_keyword(
    "-moz-top-layer",
    "none top",
    engines="gecko",
    gecko_enum_prefix="StyleTopLayer",
    gecko_ffi_name="mTopLayer",
    animation_value_type="none",
    enabled_in="ua",
    spec="Internal (not web-exposed)",
)}

// An internal-only property for elements in a top layer
// https://fullscreen.spec.whatwg.org/#top-layer
${helpers.single_keyword(
    "-servo-top-layer",
    "none top",
    engines="servo-2013 servo-2020",
    animation_value_type="none",
    enabled_in="ua",
    spec="Internal (not web-exposed)",
)}

<%helpers:single_keyword
    name="position"
    values="static absolute relative fixed ${'sticky' if engine in ['gecko', 'servo-2013'] else ''}"
    engines="gecko servo-2013 servo-2020"
    animation_value_type="discrete"
    gecko_enum_prefix="StylePositionProperty"
    flags="CREATES_STACKING_CONTEXT ABSPOS_CB"
    spec="https://drafts.csswg.org/css-position/#position-property"
    servo_restyle_damage="rebuild_and_reflow"
>
impl computed_value::T {
    pub fn is_absolutely_positioned(self) -> bool {
        matches!(self, Self::Absolute | Self::Fixed)
    }
    pub fn is_relative(self) -> bool {
        self == Self::Relative
    }
}
</%helpers:single_keyword>

${helpers.predefined_type(
    "float",
    "Float",
    "computed::Float::None",
    engines="gecko servo-2013 servo-2020",
    servo_2020_pref="layout.2020.unimplemented",
    initial_specified_value="specified::Float::None",
    spec="https://drafts.csswg.org/css-box/#propdef-float",
    animation_value_type="discrete",
    needs_context=False,
    servo_restyle_damage="rebuild_and_reflow",
    gecko_ffi_name="mFloat",
)}

${helpers.predefined_type(
    "clear",
    "Clear",
    "computed::Clear::None",
    engines="gecko servo-2013",
    animation_value_type="discrete",
    needs_context=False,
    gecko_ffi_name="mBreakType",
    spec="https://drafts.csswg.org/css-box/#propdef-clear",
    servo_restyle_damage="rebuild_and_reflow",
)}

${helpers.predefined_type(
    "vertical-align",
    "VerticalAlign",
    "computed::VerticalAlign::baseline()",
    engines="gecko servo-2013",
    animation_value_type="ComputedValue",
    spec="https://www.w3.org/TR/CSS2/visudet.html#propdef-vertical-align",
    servo_restyle_damage = "reflow",
)}

// CSS 2.1, Section 11 - Visual effects

${helpers.single_keyword(
    "-servo-overflow-clip-box",
    "padding-box content-box",
    engines="servo-2013",
    animation_value_type="none",
    enabled_in="ua",
    spec="Internal, not web-exposed, \
          may be standardized in the future (https://developer.mozilla.org/en-US/docs/Web/CSS/overflow-clip-box)",
)}

% for direction in ["inline", "block"]:
    ${helpers.predefined_type(
        "overflow-clip-box-" + direction,
        "OverflowClipBox",
        "computed::OverflowClipBox::PaddingBox",
        engines="gecko",
        enabled_in="ua",
        needs_context=False,
        gecko_pref="layout.css.overflow-clip-box.enabled",
        animation_value_type="discrete",
        spec="Internal, may be standardized in the future: \
              https://developer.mozilla.org/en-US/docs/Web/CSS/overflow-clip-box",
    )}
% endfor

% for (axis, logical) in ALL_AXES:
    <% full_name = "overflow-{}".format(axis) %>
    ${helpers.predefined_type(
        full_name,
        "Overflow",
        "computed::Overflow::Visible",
        engines="gecko servo-2013 servo-2020",
        logical_group="overflow",
        logical=logical,
        animation_value_type="discrete",
        spec="https://drafts.csswg.org/css-overflow-3/#propdef-{}".format(full_name),
        needs_context=False,
        servo_restyle_damage = "reflow",
        gecko_pref="layout.css.overflow-logical.enabled" if logical else None,
    )}
% endfor

${helpers.predefined_type(
    "overflow-anchor",
    "OverflowAnchor",
    "computed::OverflowAnchor::Auto",
    engines="gecko",
    initial_specified_value="specified::OverflowAnchor::Auto",
    needs_context=False,
    gecko_pref="layout.css.scroll-anchoring.enabled",
    spec="https://drafts.csswg.org/css-scroll-anchoring/#exclusion-api",
    animation_value_type="discrete",
)}

<% transition_extra_prefixes = "moz:layout.css.prefixes.transitions webkit" %>

${helpers.predefined_type(
    "transition-duration",
    "Time",
    "computed::Time::zero()",
    engines="gecko servo-2013 servo-2020",
    servo_2020_pref="layout.2020.unimplemented",
    initial_specified_value="specified::Time::zero()",
    parse_method="parse_non_negative",
    vector=True,
    need_index=True,
    animation_value_type="none",
    extra_prefixes=transition_extra_prefixes,
    spec="https://drafts.csswg.org/css-transitions/#propdef-transition-duration",
)}

${helpers.predefined_type(
    "transition-timing-function",
    "TimingFunction",
    "computed::TimingFunction::ease()",
    engines="gecko servo-2013 servo-2020",
    servo_2020_pref="layout.2020.unimplemented",
    initial_specified_value="specified::TimingFunction::ease()",
    vector=True,
    need_index=True,
    animation_value_type="none",
    extra_prefixes=transition_extra_prefixes,
    spec="https://drafts.csswg.org/css-transitions/#propdef-transition-timing-function",
)}

${helpers.predefined_type(
    "transition-property",
    "TransitionProperty",
    "computed::TransitionProperty::all()",
    engines="gecko servo-2013 servo-2020",
    servo_2020_pref="layout.2020.unimplemented",
    initial_specified_value="specified::TransitionProperty::all()",
    vector=True,
    allow_empty="NotInitial",
    need_index=True,
    animation_value_type="none",
    extra_prefixes=transition_extra_prefixes,
    spec="https://drafts.csswg.org/css-transitions/#propdef-transition-property",
)}

${helpers.predefined_type(
    "transition-delay",
    "Time",
    "computed::Time::zero()",
    engines="gecko servo-2013 servo-2020",
    servo_2020_pref="layout.2020.unimplemented",
    initial_specified_value="specified::Time::zero()",
    vector=True,
    need_index=True,
    animation_value_type="none",
    extra_prefixes=transition_extra_prefixes,
    spec="https://drafts.csswg.org/css-transitions/#propdef-transition-delay",
)}

<% animation_extra_prefixes = "moz:layout.css.prefixes.animations webkit" %>

${helpers.predefined_type(
    "animation-name",
    "AnimationName",
    "computed::AnimationName::none()",
    engines="gecko servo-2013 servo-2020",
    servo_2020_pref="layout.2020.unimplemented",
    initial_specified_value="specified::AnimationName::none()",
    vector=True,
    need_index=True,
    animation_value_type="none",
    extra_prefixes=animation_extra_prefixes,
    allowed_in_keyframe_block=False,
    spec="https://drafts.csswg.org/css-animations/#propdef-animation-name",
)}

${helpers.predefined_type(
    "animation-duration",
    "Time",
    "computed::Time::zero()",
    engines="gecko servo-2013 servo-2020",
    servo_2020_pref="layout.2020.unimplemented",
    initial_specified_value="specified::Time::zero()",
    parse_method="parse_non_negative",
    vector=True,
    need_index=True,
    animation_value_type="none",
    extra_prefixes=animation_extra_prefixes,
    spec="https://drafts.csswg.org/css-transitions/#propdef-transition-duration",
)}

// animation-timing-function is the exception to the rule for allowed_in_keyframe_block:
// https://drafts.csswg.org/css-animations/#keyframes
${helpers.predefined_type(
    "animation-timing-function",
    "TimingFunction",
    "computed::TimingFunction::ease()",
    engines="gecko servo-2013 servo-2020",
    servo_2020_pref="layout.2020.unimplemented",
    initial_specified_value="specified::TimingFunction::ease()",
    vector=True,
    need_index=True,
    animation_value_type="none",
    extra_prefixes=animation_extra_prefixes,
    allowed_in_keyframe_block=True,
    spec="https://drafts.csswg.org/css-transitions/#propdef-animation-timing-function",
)}

${helpers.predefined_type(
    "animation-iteration-count",
    "AnimationIterationCount",
    "computed::AnimationIterationCount::one()",
    engines="gecko servo-2013 servo-2020",
    servo_2020_pref="layout.2020.unimplemented",
    initial_specified_value="specified::AnimationIterationCount::one()",
    vector=True,
    need_index=True,
    animation_value_type="none",
    extra_prefixes=animation_extra_prefixes,
    allowed_in_keyframe_block=False,
    spec="https://drafts.csswg.org/css-animations/#propdef-animation-iteration-count",
)}

<% animation_direction_custom_consts = { "alternate-reverse": "Alternate_reverse" } %>
${helpers.single_keyword(
    "animation-direction",
    "normal reverse alternate alternate-reverse",
    engines="gecko servo-2013 servo-2020",
    servo_2020_pref="layout.2020.unimplemented",
    need_index=True,
    animation_value_type="none",
    vector=True,
    gecko_enum_prefix="PlaybackDirection",
    custom_consts=animation_direction_custom_consts,
    extra_prefixes=animation_extra_prefixes,
    gecko_inexhaustive=True,
    spec="https://drafts.csswg.org/css-animations/#propdef-animation-direction",
    allowed_in_keyframe_block=False,
)}

${helpers.single_keyword(
    "animation-play-state",
    "running paused",
    engines="gecko servo-2013 servo-2020",
    servo_2020_pref="layout.2020.unimplemented",
    need_index=True,
    animation_value_type="none",
    vector=True,
    extra_prefixes=animation_extra_prefixes,
    gecko_enum_prefix="StyleAnimationPlayState",
    spec="https://drafts.csswg.org/css-animations/#propdef-animation-play-state",
    allowed_in_keyframe_block=False,
)}

${helpers.single_keyword(
    "animation-fill-mode",
    "none forwards backwards both",
    engines="gecko servo-2013",
    need_index=True,
    animation_value_type="none",
    vector=True,
    gecko_enum_prefix="FillMode",
    extra_prefixes=animation_extra_prefixes,
    gecko_inexhaustive=True,
    spec="https://drafts.csswg.org/css-animations/#propdef-animation-fill-mode",
    allowed_in_keyframe_block=False,
)}

${helpers.predefined_type(
    "animation-delay",
    "Time",
    "computed::Time::zero()",
    engines="gecko servo-2013 servo-2020",
    servo_2020_pref="layout.2020.unimplemented",
    initial_specified_value="specified::Time::zero()",
    vector=True,
    need_index=True,
    animation_value_type="none",
    extra_prefixes=animation_extra_prefixes,
    spec="https://drafts.csswg.org/css-animations/#propdef-animation-delay",
    allowed_in_keyframe_block=False,
)}

<% transform_extra_prefixes = "moz:layout.css.prefixes.transforms webkit" %>

${helpers.predefined_type(
    "transform",
    "Transform",
    "generics::transform::Transform::none()",
    engines="gecko servo-2013 servo-2020",
    extra_prefixes=transform_extra_prefixes,
    animation_value_type="ComputedValue",
    flags="CREATES_STACKING_CONTEXT FIXPOS_CB CAN_ANIMATE_ON_COMPOSITOR",
    spec="https://drafts.csswg.org/css-transforms/#propdef-transform",
    servo_restyle_damage="reflow_out_of_flow",
)}

${helpers.predefined_type(
    "rotate",
    "Rotate",
    "generics::transform::Rotate::None",
    engines="gecko servo-2013",
    animation_value_type="ComputedValue",
    boxed=True,
    flags="CREATES_STACKING_CONTEXT FIXPOS_CB CAN_ANIMATE_ON_COMPOSITOR",
    gecko_pref="layout.css.individual-transform.enabled",
    spec="https://drafts.csswg.org/css-transforms-2/#individual-transforms",
    servo_restyle_damage = "reflow_out_of_flow",
)}

${helpers.predefined_type(
    "scale",
    "Scale",
    "generics::transform::Scale::None",
    engines="gecko servo-2013",
    animation_value_type="ComputedValue",
    boxed=True,
    flags="CREATES_STACKING_CONTEXT FIXPOS_CB CAN_ANIMATE_ON_COMPOSITOR",
    gecko_pref="layout.css.individual-transform.enabled",
    spec="https://drafts.csswg.org/css-transforms-2/#individual-transforms",
    servo_restyle_damage = "reflow_out_of_flow",
)}

${helpers.predefined_type(
    "translate",
    "Translate",
    "generics::transform::Translate::None",
    engines="gecko servo-2013",
    animation_value_type="ComputedValue",
    boxed=True,
    flags="CREATES_STACKING_CONTEXT FIXPOS_CB CAN_ANIMATE_ON_COMPOSITOR",
    gecko_pref="layout.css.individual-transform.enabled",
    spec="https://drafts.csswg.org/css-transforms-2/#individual-transforms",
    servo_restyle_damage="reflow_out_of_flow",
)}

// Motion Path Module Level 1
${helpers.predefined_type(
    "offset-path",
    "OffsetPath",
    "computed::OffsetPath::none()",
    engines="gecko",
    animation_value_type="ComputedValue",
    gecko_pref="layout.css.motion-path.enabled",
    flags="CREATES_STACKING_CONTEXT FIXPOS_CB CAN_ANIMATE_ON_COMPOSITOR",
    spec="https://drafts.fxtf.org/motion-1/#offset-path-property",
    servo_restyle_damage="reflow_out_of_flow"
)}

// Motion Path Module Level 1
${helpers.predefined_type(
    "offset-distance",
    "LengthPercentage",
    "computed::LengthPercentage::zero()",
    engines="gecko",
    animation_value_type="ComputedValue",
    gecko_pref="layout.css.motion-path.enabled",
    flags="CAN_ANIMATE_ON_COMPOSITOR",
    spec="https://drafts.fxtf.org/motion-1/#offset-distance-property",
    servo_restyle_damage="reflow_out_of_flow"
)}

// Motion Path Module Level 1
${helpers.predefined_type(
    "offset-rotate",
    "OffsetRotate",
    "computed::OffsetRotate::auto()",
    engines="gecko",
    animation_value_type="ComputedValue",
    gecko_pref="layout.css.motion-path.enabled",
    flags="CAN_ANIMATE_ON_COMPOSITOR",
    spec="https://drafts.fxtf.org/motion-1/#offset-rotate-property",
    servo_restyle_damage="reflow_out_of_flow"
)}

// Motion Path Module Level 1
${helpers.predefined_type(
    "offset-anchor",
    "PositionOrAuto",
    "computed::PositionOrAuto::auto()",
    engines="gecko",
    animation_value_type="ComputedValue",
    gecko_pref="layout.css.motion-path.enabled",
    flags="CAN_ANIMATE_ON_COMPOSITOR",
    spec="https://drafts.fxtf.org/motion-1/#offset-anchor-property",
    servo_restyle_damage="reflow_out_of_flow",
    boxed=True
)}

// CSSOM View Module
// https://www.w3.org/TR/cssom-view-1/
${helpers.single_keyword(
    "scroll-behavior",
    "auto smooth",
    engines="gecko",
    spec="https://drafts.csswg.org/cssom-view/#propdef-scroll-behavior",
    animation_value_type="discrete",
    gecko_enum_prefix="StyleScrollBehavior",
)}

${helpers.predefined_type(
    "scroll-snap-align",
    "ScrollSnapAlign",
    "computed::ScrollSnapAlign::none()",
    engines="gecko",
    spec="https://drafts.csswg.org/css-scroll-snap-1/#scroll-snap-align",
    animation_value_type="discrete",
)}

${helpers.predefined_type(
    "scroll-snap-type",
    "ScrollSnapType",
    "computed::ScrollSnapType::none()",
    engines="gecko",
    spec="https://drafts.csswg.org/css-scroll-snap-1/#scroll-snap-type",
    animation_value_type="discrete",
)}

% for (axis, logical) in ALL_AXES:
    ${helpers.predefined_type(
        "overscroll-behavior-" + axis,
        "OverscrollBehavior",
        "computed::OverscrollBehavior::Auto",
        engines="gecko",
        needs_context=False,
        logical_group="overscroll-behavior",
        logical=logical,
        gecko_pref="layout.css.overscroll-behavior.enabled",
        spec="https://wicg.github.io/overscroll-behavior/#overscroll-behavior-properties",
        animation_value_type="discrete",
    )}
% endfor

// Compositing and Blending Level 1
// http://www.w3.org/TR/compositing-1/
${helpers.single_keyword(
    "isolation",
    "auto isolate",
    engines="gecko",
    spec="https://drafts.fxtf.org/compositing/#isolation",
    flags="CREATES_STACKING_CONTEXT",
    gecko_enum_prefix="StyleIsolation",
    animation_value_type="discrete",
)}

${helpers.predefined_type(
    "break-after",
    "BreakBetween",
    "computed::BreakBetween::Auto",
    engines="gecko",
    needs_context=False,
    spec="https://drafts.csswg.org/css-break/#propdef-break-after",
    animation_value_type="discrete",
)}

${helpers.predefined_type(
    "break-before",
    "BreakBetween",
    "computed::BreakBetween::Auto",
    engines="gecko",
    needs_context=False,
    spec="https://drafts.csswg.org/css-break/#propdef-break-before",
    animation_value_type="discrete",
)}

${helpers.predefined_type(
    "break-inside",
    "BreakWithin",
    "computed::BreakWithin::Auto",
    engines="gecko",
    needs_context=False,
    alias="page-break-inside",
    spec="https://drafts.csswg.org/css-break/#propdef-break-inside",
    animation_value_type="discrete",
)}

// CSS Basic User Interface Module Level 3
// http://dev.w3.org/csswg/css-ui
${helpers.predefined_type(
    "resize",
    "Resize",
    "computed::Resize::None",
    engines="gecko",
    animation_value_type="discrete",
    needs_context=False,
    gecko_ffi_name="mResize",
    spec="https://drafts.csswg.org/css-ui/#propdef-resize",
)}

${helpers.predefined_type(
    "perspective",
    "Perspective",
    "computed::Perspective::none()",
    engines="gecko servo-2013 servo-2020",
    gecko_ffi_name="mChildPerspective",
    spec="https://drafts.csswg.org/css-transforms/#perspective",
    extra_prefixes=transform_extra_prefixes,
    flags="CREATES_STACKING_CONTEXT FIXPOS_CB",
    animation_value_type="AnimatedPerspective",
    servo_restyle_damage = "reflow_out_of_flow",
)}

${helpers.predefined_type(
    "perspective-origin",
    "Position",
    "computed::position::Position::center()",
    engines="gecko servo-2013 servo-2020",
    boxed=True,
    extra_prefixes=transform_extra_prefixes,
    spec="https://drafts.csswg.org/css-transforms-2/#perspective-origin-property",
    animation_value_type="ComputedValue",
    servo_restyle_damage="reflow_out_of_flow"
)}

${helpers.single_keyword(
    "backface-visibility",
    "visible hidden",
    engines="gecko servo-2013 servo-2020",
    gecko_enum_prefix="StyleBackfaceVisibility",
    spec="https://drafts.csswg.org/css-transforms/#backface-visibility-property",
    extra_prefixes=transform_extra_prefixes,
    animation_value_type="discrete",
)}

${helpers.single_keyword(
    "transform-box",
    "border-box fill-box view-box",
    engines="gecko",
    gecko_enum_prefix="StyleGeometryBox",
    gecko_pref="svg.transform-box.enabled",
    spec="https://drafts.csswg.org/css-transforms/#transform-box",
    gecko_inexhaustive="True",
    animation_value_type="discrete",
)}

${helpers.predefined_type(
    "transform-style",
    "TransformStyle",
    "computed::TransformStyle::Flat",
    engines="gecko servo-2013 servo-2020",
    spec="https://drafts.csswg.org/css-transforms-2/#transform-style-property",
    needs_context=False,
    extra_prefixes=transform_extra_prefixes,
    flags="CREATES_STACKING_CONTEXT FIXPOS_CB",
    animation_value_type="discrete",
    servo_restyle_damage = "reflow_out_of_flow",
)}

${helpers.predefined_type(
    "transform-origin",
    "TransformOrigin",
    "computed::TransformOrigin::initial_value()",
    engines="gecko servo-2013 servo-2020",
    animation_value_type="ComputedValue",
    extra_prefixes=transform_extra_prefixes,
    gecko_ffi_name="mTransformOrigin",
    boxed=True,
    spec="https://drafts.csswg.org/css-transforms/#transform-origin-property",
    servo_restyle_damage="reflow_out_of_flow",
)}

${helpers.predefined_type(
    "contain",
    "Contain",
    "specified::Contain::empty()",
    engines="gecko",
    animation_value_type="none",
    flags="CREATES_STACKING_CONTEXT FIXPOS_CB",
    gecko_pref="layout.css.contain.enabled",
    spec="https://drafts.csswg.org/css-contain/#contain-property",
    enabled_in="chrome",
)}

// Non-standard
${helpers.predefined_type(
    "-moz-appearance",
    "Appearance",
    "computed::Appearance::None",
    engines="gecko",
    alias="-webkit-appearance",
    spec="Nonstandard (https://developer.mozilla.org/en-US/docs/Web/CSS/-moz-appearance)",
    animation_value_type="discrete",
    gecko_ffi_name="mAppearance",
)}

${helpers.single_keyword(
    "-moz-orient",
    "inline block horizontal vertical",
    engines="gecko",
    gecko_ffi_name="mOrient",
    gecko_enum_prefix="StyleOrient",
    spec="Nonstandard (https://developer.mozilla.org/en-US/docs/Web/CSS/-moz-orient)",
    animation_value_type="discrete",
)}

${helpers.predefined_type(
    "will-change",
    "WillChange",
    "computed::WillChange::auto()",
    engines="gecko",
    animation_value_type="none",
    spec="https://drafts.csswg.org/css-will-change/#will-change",
)}

// The spec issue for the parse_method: https://github.com/w3c/csswg-drafts/issues/4102.
${helpers.predefined_type(
    "shape-image-threshold",
    "Opacity",
    "0.0",
    engines="gecko",
    animation_value_type="ComputedValue",
    spec="https://drafts.csswg.org/css-shapes/#shape-image-threshold-property",
)}

${helpers.predefined_type(
    "shape-margin",
    "NonNegativeLengthPercentage",
    "computed::NonNegativeLengthPercentage::zero()",
    engines="gecko",
    animation_value_type="NonNegativeLengthPercentage",
    spec="https://drafts.csswg.org/css-shapes/#shape-margin-property",
)}

${helpers.predefined_type(
    "shape-outside",
    "basic_shape::ShapeOutside",
    "generics::basic_shape::ShapeOutside::None",
    engines="gecko",
    animation_value_type="basic_shape::ShapeOutside",
    spec="https://drafts.csswg.org/css-shapes/#shape-outside-property",
)}

${helpers.predefined_type(
    "touch-action",
    "TouchAction",
    "computed::TouchAction::auto()",
    engines="gecko",
    gecko_pref="layout.css.touch_action.enabled",
    animation_value_type="discrete",
    spec="https://compat.spec.whatwg.org/#touch-action",
)}

// Note that we only implement -webkit-line-clamp as a single, longhand
// property for now, but the spec defines line-clamp as a shorthand for separate
// max-lines, block-ellipsis, and continue properties.
${helpers.predefined_type(
    "-webkit-line-clamp",
    "PositiveIntegerOrNone",
    "Either::Second(None_)",
    engines="gecko",
    gecko_pref="layout.css.webkit-line-clamp.enabled",
    animation_value_type="Integer",
    spec="https://drafts.csswg.org/css-overflow-3/#line-clamp",
)}
