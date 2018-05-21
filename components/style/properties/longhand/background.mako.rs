/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />

<% data.new_style_struct("Background", inherited=False) %>

${helpers.predefined_type(
    "background-color",
    "Color",
    "computed_value::T::transparent()",
    initial_specified_value="SpecifiedValue::transparent()",
    spec="https://drafts.csswg.org/css-backgrounds/#background-color",
    animation_value_type="AnimatedColor",
    ignored_when_colors_disabled=True,
    allow_quirks=True,
    flags="APPLIES_TO_FIRST_LETTER APPLIES_TO_FIRST_LINE APPLIES_TO_PLACEHOLDER",
)}

${helpers.predefined_type("background-image", "ImageLayer",
    initial_value="Either::First(None_)",
    initial_specified_value="Either::First(None_)",
    spec="https://drafts.csswg.org/css-backgrounds/#the-background-image",
    vector="True",
    animation_value_type="discrete",
    ignored_when_colors_disabled="True",
    flags="APPLIES_TO_FIRST_LETTER APPLIES_TO_FIRST_LINE APPLIES_TO_PLACEHOLDER")}

% for (axis, direction, initial) in [("x", "Horizontal", "left"), ("y", "Vertical", "top")]:
    ${helpers.predefined_type(
        "background-position-" + axis,
        "position::" + direction + "Position",
        initial_value="computed::LengthOrPercentage::zero()",
        initial_specified_value="SpecifiedValue::initial_specified_value()",
        spec="https://drafts.csswg.org/css-backgrounds-4/#propdef-background-position-" + axis,
        animation_value_type="ComputedValue",
        vector=True,
        flags="APPLIES_TO_FIRST_LETTER APPLIES_TO_FIRST_LINE APPLIES_TO_PLACEHOLDER",
    )}
% endfor

${helpers.predefined_type(
    "background-repeat",
    "BackgroundRepeat",
    "computed::BackgroundRepeat::repeat()",
    initial_specified_value="specified::BackgroundRepeat::repeat()",
    animation_value_type="discrete",
    vector=True,
    spec="https://drafts.csswg.org/css-backgrounds/#the-background-repeat",
    flags="APPLIES_TO_FIRST_LETTER APPLIES_TO_FIRST_LINE APPLIES_TO_PLACEHOLDER",
)}

${helpers.single_keyword("background-attachment",
                         "scroll fixed" + (" local" if product == "gecko" else ""),
                         vector=True,
                         gecko_enum_prefix="StyleImageLayerAttachment",
                         spec="https://drafts.csswg.org/css-backgrounds/#the-background-attachment",
                         animation_value_type="discrete",
                         flags="APPLIES_TO_FIRST_LETTER APPLIES_TO_FIRST_LINE APPLIES_TO_PLACEHOLDER")}

${helpers.single_keyword("background-clip",
                         "border-box padding-box content-box",
                         extra_gecko_values="text",
                         vector=True, extra_prefixes="webkit",
                         gecko_enum_prefix="StyleGeometryBox",
                         spec="https://drafts.csswg.org/css-backgrounds/#the-background-clip",
                         animation_value_type="discrete",
                         flags="APPLIES_TO_FIRST_LETTER APPLIES_TO_FIRST_LINE APPLIES_TO_PLACEHOLDER")}

${helpers.single_keyword("background-origin",
                         "padding-box border-box content-box",
                         vector=True, extra_prefixes="webkit",
                         gecko_enum_prefix="StyleGeometryBox",
                         spec="https://drafts.csswg.org/css-backgrounds/#the-background-origin",
                         animation_value_type="discrete",
                         flags="APPLIES_TO_FIRST_LETTER APPLIES_TO_FIRST_LINE APPLIES_TO_PLACEHOLDER")}

${helpers.predefined_type("background-size", "BackgroundSize",
    initial_value="computed::BackgroundSize::auto()",
    initial_specified_value="specified::BackgroundSize::auto()",
    spec="https://drafts.csswg.org/css-backgrounds/#the-background-size",
    vector=True,
    animation_value_type="BackgroundSizeList",
    need_animatable=True,
    flags="APPLIES_TO_FIRST_LETTER APPLIES_TO_FIRST_LINE APPLIES_TO_PLACEHOLDER",
    extra_prefixes="webkit")}

// https://drafts.fxtf.org/compositing/#background-blend-mode
${helpers.single_keyword("background-blend-mode",
                         """normal multiply screen overlay darken lighten color-dodge
                            color-burn hard-light soft-light difference exclusion hue
                            saturation color luminosity""",
                         gecko_constant_prefix="NS_STYLE_BLEND",
                         gecko_pref="layout.css.background-blend-mode.enabled",
                         vector=True, products="gecko", animation_value_type="discrete",
                         spec="https://drafts.fxtf.org/compositing/#background-blend-mode",
                         flags="APPLIES_TO_FIRST_LETTER APPLIES_TO_FIRST_LINE APPLIES_TO_PLACEHOLDER")}
