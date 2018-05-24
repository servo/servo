/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />

<% data.new_style_struct("SVG", inherited=False, gecko_name="SVGReset") %>

${helpers.single_keyword("dominant-baseline",
                 """auto use-script no-change reset-size ideographic alphabetic hanging
                    mathematical central middle text-after-edge text-before-edge""",
                 products="gecko",
                 animation_value_type="discrete",
                 spec="https://www.w3.org/TR/SVG11/text.html#DominantBaselineProperty")}

${helpers.single_keyword("vector-effect", "none non-scaling-stroke",
                         products="gecko", animation_value_type="discrete",
                         spec="https://www.w3.org/TR/SVGTiny12/painting.html#VectorEffectProperty")}

// Section 13 - Gradients and Patterns

${helpers.predefined_type(
    "stop-color",
    "RGBAColor",
    "RGBA::new(0, 0, 0, 255)",
    products="gecko",
    animation_value_type="AnimatedRGBA",
    spec="https://www.w3.org/TR/SVGTiny12/painting.html#StopColorProperty",
)}

${helpers.predefined_type("stop-opacity", "Opacity", "1.0",
                          products="gecko",
                          animation_value_type="ComputedValue",
                          spec="https://www.w3.org/TR/SVGTiny12/painting.html#propdef-stop-opacity")}

// Section 15 - Filter Effects

${helpers.predefined_type(
    "flood-color",
    "RGBAColor",
    "RGBA::new(0, 0, 0, 255)",
    products="gecko",
    animation_value_type="AnimatedRGBA",
    spec="https://www.w3.org/TR/SVG/filters.html#FloodColorProperty",
)}

${helpers.predefined_type("flood-opacity", "Opacity",
                          "1.0", products="gecko", animation_value_type="ComputedValue",
                          spec="https://www.w3.org/TR/SVG/filters.html#FloodOpacityProperty")}

${helpers.predefined_type(
    "lighting-color",
    "RGBAColor",
    "RGBA::new(255, 255, 255, 255)",
    products="gecko",
    animation_value_type="AnimatedRGBA",
    spec="https://www.w3.org/TR/SVG/filters.html#LightingColorProperty",
)}

// CSS Masking Module Level 1
// https://drafts.fxtf.org/css-masking
${helpers.single_keyword("mask-type", "luminance alpha",
                         products="gecko", animation_value_type="discrete",
                         spec="https://drafts.fxtf.org/css-masking/#propdef-mask-type")}

${helpers.predefined_type(
    "clip-path",
    "basic_shape::ClippingShape",
    "generics::basic_shape::ShapeSource::None",
    products="gecko",
    boxed=True,
    animation_value_type="ComputedValue",
    flags="CREATES_STACKING_CONTEXT",
    spec="https://drafts.fxtf.org/css-masking/#propdef-clip-path",
)}

${helpers.single_keyword("mask-mode",
                         "match-source alpha luminance",
                         vector=True,
                         products="gecko",
                         animation_value_type="discrete",
                         spec="https://drafts.fxtf.org/css-masking/#propdef-mask-mode")}

${helpers.predefined_type(
    "mask-repeat",
    "BackgroundRepeat",
    "computed::BackgroundRepeat::repeat()",
    initial_specified_value="specified::BackgroundRepeat::repeat()",
    products="gecko",
    extra_prefixes="webkit",
    animation_value_type="discrete",
    spec="https://drafts.fxtf.org/css-masking/#propdef-mask-repeat",
    vector=True,
)}

% for (axis, direction) in [("x", "Horizontal"), ("y", "Vertical")]:
    ${helpers.predefined_type(
        "mask-position-" + axis,
        "position::" + direction + "Position",
        products="gecko",
        extra_prefixes="webkit",
        initial_value="computed::LengthOrPercentage::zero()",
        initial_specified_value="specified::PositionComponent::Center",
        spec="https://drafts.fxtf.org/css-masking/#propdef-mask-position",
        animation_value_type="ComputedValue",
        vector_animation_type="repeatable_list",
        vector=True,
    )}
% endfor

${helpers.single_keyword(
    "mask-clip",
    "border-box content-box padding-box",
    extra_gecko_values="fill-box stroke-box view-box no-clip",
    vector=True,
    products="gecko",
    extra_prefixes="webkit",
    gecko_enum_prefix="StyleGeometryBox",
    gecko_inexhaustive=True,
    animation_value_type="discrete",
    spec="https://drafts.fxtf.org/css-masking/#propdef-mask-clip",
)}

${helpers.single_keyword(
    "mask-origin",
    "border-box content-box padding-box",
    extra_gecko_values="fill-box stroke-box view-box",
    vector=True,
    products="gecko",
    extra_prefixes="webkit",
    gecko_enum_prefix="StyleGeometryBox",
    gecko_inexhaustive=True,
    animation_value_type="discrete",
    spec="https://drafts.fxtf.org/css-masking/#propdef-mask-origin",
)}

${helpers.predefined_type(
    "mask-size",
    "background::BackgroundSize",
    "computed::BackgroundSize::auto()",
    initial_specified_value="specified::BackgroundSize::auto()",
    products="gecko",
    extra_prefixes="webkit",
    spec="https://drafts.fxtf.org/css-masking/#propdef-mask-size",
    animation_value_type="MaskSizeList",
    vector=True,
    vector_animation_type="repeatable_list",
)}

${helpers.single_keyword("mask-composite",
                         "add subtract intersect exclude",
                         vector=True,
                         products="gecko",
                         extra_prefixes="webkit",
                         animation_value_type="discrete",
                         spec="https://drafts.fxtf.org/css-masking/#propdef-mask-composite")}

${helpers.predefined_type("mask-image", "ImageLayer",
    initial_value="Either::First(None_)",
    initial_specified_value="Either::First(None_)",
    spec="https://drafts.fxtf.org/css-masking/#propdef-mask-image",
    vector=True,
    products="gecko",
    extra_prefixes="webkit",
    animation_value_type="discrete",
    flags="CREATES_STACKING_CONTEXT")}
