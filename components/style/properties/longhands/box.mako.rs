/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />
<% from data import Keyword, Method, to_rust_ident, to_camel_case%>

<% data.new_style_struct("Box",
                         inherited=False,
                         gecko_name="Display") %>

// We allow "display" to apply to placeholders because we need to make the
// placeholder pseudo-element an inline-block in the UA stylesheet in Gecko.
${helpers.predefined_type(
    "display",
    "Display",
    "computed::Display::inline()",
    initial_specified_value="specified::Display::inline()",
    animation_value_type="discrete",
    flags="APPLIES_TO_PLACEHOLDER",
    spec="https://drafts.csswg.org/css-display/#propdef-display",
    servo_restyle_damage="rebuild_and_reflow",
    needs_context=product == "gecko"
)}

// FIXME(emilio): Listing all the display values here is very unfortunate, we should teach C++ to use the
// Rust enum directly, or generate the conversions to `StyleDisplay`.
${helpers.gecko_keyword_conversion(
    Keyword('display', """
        inline block inline-block
        table inline-table table-row-group table-header-group table-footer-group
        table-row table-column-group table-column table-cell table-caption
        list-item none flex inline-flex grid inline-grid ruby ruby-base ruby-base-container
        ruby-text ruby-text-container contents flow-root -webkit-box
        -webkit-inline-box -moz-box -moz-inline-box -moz-grid -moz-inline-grid
        -moz-grid-group -moz-grid-line -moz-stack -moz-inline-stack -moz-deck
        -moz-popup -moz-groupbox
    """,
    gecko_enum_prefix='StyleDisplay',
    gecko_strip_moz_prefix=False),
    type="::values::specified::Display"
)}

${helpers.single_keyword("-moz-top-layer", "none top",
                         gecko_constant_prefix="NS_STYLE_TOP_LAYER",
                         gecko_ffi_name="mTopLayer",
                         products="gecko", animation_value_type="none",
                         enabled_in="ua",
                         spec="Internal (not web-exposed)")}

${helpers.single_keyword("position", "static absolute relative fixed sticky",
                         animation_value_type="discrete",
                         flags="CREATES_STACKING_CONTEXT ABSPOS_CB",
                         spec="https://drafts.csswg.org/css-position/#position-property",
                         servo_restyle_damage="rebuild_and_reflow")}

${helpers.predefined_type(
    "float",
    "Float",
    "computed::Float::None",
    initial_specified_value="specified::Float::None",
    spec="https://drafts.csswg.org/css-box/#propdef-float",
    animation_value_type="discrete",
    needs_context=False,
    flags="APPLIES_TO_FIRST_LETTER",
    servo_restyle_damage="rebuild_and_reflow",
    gecko_ffi_name="mFloat"
)}

<%helpers:single_keyword
    name="clear"
    values="none left right both"
    // https://drafts.csswg.org/css-logical-props/#float-clear
    extra_specified="inline-start inline-end"
    needs_conversion="True"
    gecko_inexhaustive="True"
    animation_value_type="discrete"
    gecko_enum_prefix="StyleClear"
    gecko_ffi_name="mBreakType"
    spec="https://drafts.csswg.org/css-box/#propdef-clear"
    servo_restyle_damage="rebuild_and_reflow"
>
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
</%helpers:single_keyword>

${helpers.predefined_type(
    "vertical-align",
    "VerticalAlign",
    "computed::VerticalAlign::baseline()",
    animation_value_type="ComputedValue",
    flags="APPLIES_TO_FIRST_LETTER APPLIES_TO_FIRST_LINE APPLIES_TO_PLACEHOLDER",
    spec="https://www.w3.org/TR/CSS2/visudet.html#propdef-vertical-align",
    servo_restyle_damage = "reflow"
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
        gecko_pref="layout.css.overflow-clip-box.enabled",
        animation_value_type="discrete",
        spec="Internal, may be standardized in the future: \
              https://developer.mozilla.org/en-US/docs/Web/CSS/overflow-clip-box",
    )}
% endfor

<%
    overflow_custom_consts = { "-moz-hidden-unscrollable": "CLIP" }
%>

// FIXME(pcwalton, #2742): Implement scrolling for `scroll` and `auto`.
//
// We allow it to apply to placeholders for UA sheets, which set it !important.
${helpers.single_keyword("overflow-x", "visible hidden scroll auto",
                         animation_value_type="discrete",
                         extra_gecko_values="-moz-hidden-unscrollable",
                         custom_consts=overflow_custom_consts,
                         gecko_constant_prefix="NS_STYLE_OVERFLOW",
                         flags="APPLIES_TO_PLACEHOLDER",
                         spec="https://drafts.csswg.org/css-overflow/#propdef-overflow-x",
                         servo_restyle_damage = "reflow")}

// FIXME(pcwalton, #2742): Implement scrolling for `scroll` and `auto`.
//
// We allow it to apply to placeholders for UA sheets, which set it !important.
<%helpers:longhand name="overflow-y" animation_value_type="discrete"
                   flags="APPLIES_TO_PLACEHOLDER",
                   spec="https://drafts.csswg.org/css-overflow/#propdef-overflow-y"
                   servo_restyle_damage = "reflow">
    pub use super::overflow_x::{SpecifiedValue, parse, get_initial_value, computed_value};
</%helpers:longhand>

<% transition_extra_prefixes = "moz:layout.css.prefixes.transitions webkit" %>

${helpers.predefined_type("transition-duration",
                          "Time",
                          "computed::Time::zero()",
                          initial_specified_value="specified::Time::zero()",
                          parse_method="parse_non_negative",
                          vector=True,
                          need_index=True,
                          animation_value_type="none",
                          extra_prefixes=transition_extra_prefixes,
                          spec="https://drafts.csswg.org/css-transitions/#propdef-transition-duration")}

${helpers.predefined_type("transition-timing-function",
                          "TimingFunction",
                          "computed::TimingFunction::ease()",
                          initial_specified_value="specified::TimingFunction::ease()",
                          vector=True,
                          need_index=True,
                          animation_value_type="none",
                          extra_prefixes=transition_extra_prefixes,
                          spec="https://drafts.csswg.org/css-transitions/#propdef-transition-timing-function")}

${helpers.predefined_type(
    "transition-property",
    "TransitionProperty",
    "computed::TransitionProperty::all()",
    initial_specified_value="specified::TransitionProperty::all()",
    vector=True,
    allow_empty="NotInitial",
    need_index=True,
    animation_value_type="none",
    extra_prefixes=transition_extra_prefixes,
    spec="https://drafts.csswg.org/css-transitions/#propdef-transition-property",
)}

${helpers.predefined_type("transition-delay",
                          "Time",
                          "computed::Time::zero()",
                          initial_specified_value="specified::Time::zero()",
                          vector=True,
                          need_index=True,
                          animation_value_type="none",
                          extra_prefixes=transition_extra_prefixes,
                          spec="https://drafts.csswg.org/css-transitions/#propdef-transition-delay")}


<% animation_extra_prefixes = "moz:layout.css.prefixes.animations webkit" %>

${helpers.predefined_type(
    "animation-name",
    "AnimationName",
    "computed::AnimationName::none()",
    initial_specified_value="specified::AnimationName::none()",
    vector=True,
    need_index=True,
    animation_value_type="none",
    extra_prefixes=animation_extra_prefixes,
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
                          extra_prefixes=animation_extra_prefixes,
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
                          extra_prefixes=animation_extra_prefixes,
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
    extra_prefixes=animation_extra_prefixes,
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
                         extra_prefixes=animation_extra_prefixes,
                         spec="https://drafts.csswg.org/css-animations/#propdef-animation-direction",
                         allowed_in_keyframe_block=False)}

${helpers.single_keyword("animation-play-state",
                         "running paused",
                         need_index=True,
                         animation_value_type="none",
                         vector=True,
                         extra_prefixes=animation_extra_prefixes,
                         spec="https://drafts.csswg.org/css-animations/#propdef-animation-play-state",
                         allowed_in_keyframe_block=False)}

${helpers.single_keyword("animation-fill-mode",
                         "none forwards backwards both",
                         need_index=True,
                         animation_value_type="none",
                         vector=True,
                         gecko_enum_prefix="FillMode",
                         extra_prefixes=animation_extra_prefixes,
                         spec="https://drafts.csswg.org/css-animations/#propdef-animation-fill-mode",
                         allowed_in_keyframe_block=False)}

${helpers.predefined_type("animation-delay",
                          "Time",
                          "computed::Time::zero()",
                          initial_specified_value="specified::Time::zero()",
                          vector=True,
                          need_index=True,
                          animation_value_type="none",
                          extra_prefixes=animation_extra_prefixes,
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
                          boxed=True,
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

<% transform_extra_prefixes = "moz:layout.css.prefixes.transforms webkit" %>

${helpers.predefined_type(
    "transform",
    "Transform",
    "generics::transform::Transform::none()",
    extra_prefixes=transform_extra_prefixes,
    animation_value_type="ComputedValue",
    gecko_ffi_name="mSpecifiedTransform",
    flags="CREATES_STACKING_CONTEXT FIXPOS_CB \
           GETCS_NEEDS_LAYOUT_FLUSH CAN_ANIMATE_ON_COMPOSITOR",
    spec="https://drafts.csswg.org/css-transforms/#propdef-transform",
    servo_restyle_damage="reflow_out_of_flow"
)}

${helpers.predefined_type("rotate", "Rotate",
                          "generics::transform::Rotate::None",
                          animation_value_type="ComputedValue",
                          boxed=True,
                          flags="CREATES_STACKING_CONTEXT FIXPOS_CB",
                          gecko_pref="layout.css.individual-transform.enabled",
                          spec="https://drafts.csswg.org/css-transforms-2/#individual-transforms",
                          servo_restyle_damage = "reflow_out_of_flow")}

${helpers.predefined_type("scale", "Scale",
                          "generics::transform::Scale::None",
                          animation_value_type="ComputedValue",
                          boxed=True,
                          flags="CREATES_STACKING_CONTEXT FIXPOS_CB",
                          gecko_pref="layout.css.individual-transform.enabled",
                          spec="https://drafts.csswg.org/css-transforms-2/#individual-transforms",
                          servo_restyle_damage = "reflow_out_of_flow")}

${helpers.predefined_type(
    "translate",
    "Translate",
    "generics::transform::Translate::None",
    animation_value_type="ComputedValue",
    boxed=True,
    flags="CREATES_STACKING_CONTEXT FIXPOS_CB GETCS_NEEDS_LAYOUT_FLUSH",
    gecko_pref="layout.css.individual-transform.enabled",
    spec="https://drafts.csswg.org/css-transforms-2/#individual-transforms",
    servo_restyle_damage="reflow_out_of_flow"
)}

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

${helpers.predefined_type(
    "perspective",
    "Perspective",
    "computed::Perspective::none()",
    gecko_ffi_name="mChildPerspective",
    spec="https://drafts.csswg.org/css-transforms/#perspective",
    extra_prefixes=transform_extra_prefixes,
    flags="CREATES_STACKING_CONTEXT FIXPOS_CB",
    animation_value_type="AnimatedPerspective",
    servo_restyle_damage = "reflow_out_of_flow",
)}

${helpers.predefined_type(
    "perspective-origin",
    "position::Position",
    "computed::position::Position::center()",
    boxed=True,
    extra_prefixes=transform_extra_prefixes,
    spec="https://drafts.csswg.org/css-transforms-2/#perspective-origin-property",
    flags="GETCS_NEEDS_LAYOUT_FLUSH",
    animation_value_type="ComputedValue",
    servo_restyle_damage="reflow_out_of_flow"
)}

${helpers.single_keyword("backface-visibility",
                         "visible hidden",
                         spec="https://drafts.csswg.org/css-transforms/#backface-visibility-property",
                         extra_prefixes=transform_extra_prefixes,
                         animation_value_type="discrete")}

${helpers.single_keyword("transform-box",
                         "border-box fill-box view-box",
                         gecko_enum_prefix="StyleGeometryBox",
                         products="gecko",
                         gecko_pref="svg.transform-box.enabled",
                         spec="https://drafts.csswg.org/css-transforms/#transform-box",
                         gecko_inexhaustive="True",
                         animation_value_type="discrete")}

${helpers.predefined_type(
    "transform-style",
    "TransformStyle",
    "computed::TransformStyle::" + ("Auto" if product == "servo" else "Flat"),
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
    animation_value_type="ComputedValue",
    extra_prefixes=transform_extra_prefixes,
    gecko_ffi_name="mTransformOrigin",
    boxed=True,
    flags="GETCS_NEEDS_LAYOUT_FLUSH",
    spec="https://drafts.csswg.org/css-transforms/#transform-origin-property",
    servo_restyle_damage="reflow_out_of_flow"
)}

${helpers.predefined_type("contain",
                          "Contain",
                          "specified::Contain::empty()",
                          animation_value_type="discrete",
                          products="gecko",
                          flags="CREATES_STACKING_CONTEXT FIXPOS_CB",
                          gecko_pref="layout.css.contain.enabled",
                          spec="https://drafts.csswg.org/css-contain/#contain-property")}

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
                            scrollbarthumb-vertical scrollbartrack-horizontal scrollbartrack-vertical scrollcorner
                            searchfield separator
                            spinner spinner-downbutton spinner-textfield spinner-upbutton splitter statusbar
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
                         alias="-webkit-appearance:layout.css.webkit-appearance.enabled",
                         spec="Nonstandard (https://developer.mozilla.org/en-US/docs/Web/CSS/-moz-appearance)",
                         animation_value_type="discrete")}

${helpers.predefined_type("-moz-binding", "url::UrlOrNone", "computed::url::UrlOrNone::none()",
                          products="gecko",
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
    flags="APPLIES_TO_FIRST_LETTER",
    spec="https://drafts.csswg.org/css-shapes/#shape-image-threshold-property",
)}

${helpers.predefined_type(
    "shape-margin",
    "NonNegativeLengthOrPercentage",
    "computed::NonNegativeLengthOrPercentage::zero()",
    products="gecko",
    gecko_pref="layout.css.shape-outside.enabled",
    animation_value_type="NonNegativeLengthOrPercentage",
    flags="APPLIES_TO_FIRST_LETTER",
    spec="https://drafts.csswg.org/css-shapes/#shape-margin-property",
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
