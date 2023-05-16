/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />

<% data.new_style_struct("SVG", inherited=False, gecko_name="SVGReset") %>

${helpers.single_keyword(
    "vector-effect",
    "none non-scaling-stroke",
    engines="gecko",
    gecko_enum_prefix="StyleVectorEffect",
    animation_value_type="discrete",
    spec="https://www.w3.org/TR/SVGTiny12/painting.html#VectorEffectProperty",
)}

// Section 13 - Gradients and Patterns

${helpers.predefined_type(
    "stop-color",
    "Color",
    "RGBA::new(0, 0, 0, 255).into()",
    engines="gecko",
    animation_value_type="AnimatedRGBA",
    spec="https://www.w3.org/TR/SVGTiny12/painting.html#StopColorProperty",
)}

${helpers.predefined_type(
    "stop-opacity",
    "Opacity",
    "1.0",
    engines="gecko",
    animation_value_type="ComputedValue",
    spec="https://svgwg.org/svg2-draft/pservers.html#StopOpacityProperty",
)}

// Section 15 - Filter Effects

${helpers.predefined_type(
    "flood-color",
    "Color",
    "RGBA::new(0, 0, 0, 255).into()",
    engines="gecko",
    animation_value_type="AnimatedColor",
    spec="https://www.w3.org/TR/SVG/filters.html#FloodColorProperty",
)}

${helpers.predefined_type(
    "flood-opacity",
    "Opacity",
    "1.0",
    engines="gecko",
    animation_value_type="ComputedValue",
    spec="https://drafts.fxtf.org/filter-effects/#FloodOpacityProperty",
)}

${helpers.predefined_type(
    "lighting-color",
    "Color",
    "RGBA::new(255, 255, 255, 255).into()",
    engines="gecko",
    animation_value_type="AnimatedColor",
    spec="https://www.w3.org/TR/SVG/filters.html#LightingColorProperty",
)}

// CSS Masking Module Level 1
// https://drafts.fxtf.org/css-masking
${helpers.single_keyword(
    "mask-type",
    "luminance alpha",
    engines="gecko",
    gecko_enum_prefix="StyleMaskType",
    animation_value_type="discrete",
    spec="https://drafts.fxtf.org/css-masking/#propdef-mask-type",
)}

${helpers.predefined_type(
    "clip-path",
    "basic_shape::ClipPath",
    "generics::basic_shape::ClipPath::None",
    engines="gecko",
    animation_value_type="basic_shape::ClipPath",
    spec="https://drafts.fxtf.org/css-masking/#propdef-clip-path",
)}

${helpers.single_keyword(
    "mask-mode",
    "match-source alpha luminance",
    engines="gecko",
    gecko_enum_prefix="StyleMaskMode",
    vector=True,
    animation_value_type="discrete",
    spec="https://drafts.fxtf.org/css-masking/#propdef-mask-mode",
)}

${helpers.predefined_type(
    "mask-repeat",
    "BackgroundRepeat",
    "computed::BackgroundRepeat::repeat()",
    engines="gecko",
    initial_specified_value="specified::BackgroundRepeat::repeat()",
    extra_prefixes="webkit",
    animation_value_type="discrete",
    spec="https://drafts.fxtf.org/css-masking/#propdef-mask-repeat",
    vector=True,
)}

% for (axis, direction) in [("x", "Horizontal"), ("y", "Vertical")]:
    ${helpers.predefined_type(
        "mask-position-" + axis,
        "position::" + direction + "Position",
        "computed::LengthPercentage::zero_percent()",
        engines="gecko",
        extra_prefixes="webkit",
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
    engines="gecko",
    extra_gecko_values="fill-box stroke-box view-box no-clip",
    vector=True,
    extra_prefixes="webkit",
    gecko_enum_prefix="StyleGeometryBox",
    gecko_inexhaustive=True,
    animation_value_type="discrete",
    spec="https://drafts.fxtf.org/css-masking/#propdef-mask-clip",
)}

${helpers.single_keyword(
    "mask-origin",
    "border-box content-box padding-box",
    engines="gecko",
    extra_gecko_values="fill-box stroke-box view-box",
    vector=True,
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
    engines="gecko",
    initial_specified_value="specified::BackgroundSize::auto()",
    extra_prefixes="webkit",
    spec="https://drafts.fxtf.org/css-masking/#propdef-mask-size",
    animation_value_type="MaskSizeList",
    vector=True,
    vector_animation_type="repeatable_list",
)}

${helpers.single_keyword(
    "mask-composite",
    "add subtract intersect exclude",
    engines="gecko",
    gecko_enum_prefix="StyleMaskComposite",
    vector=True,
    extra_prefixes="webkit",
    animation_value_type="discrete",
    spec="https://drafts.fxtf.org/css-masking/#propdef-mask-composite",
)}

${helpers.predefined_type(
    "mask-image",
    "Image",
    engines="gecko",
    initial_value="computed::Image::None",
    initial_specified_value="specified::Image::None",
    parse_method="parse_with_cors_anonymous",
    spec="https://drafts.fxtf.org/css-masking/#propdef-mask-image",
    vector=True,
    extra_prefixes="webkit",
    animation_value_type="discrete",
)}

${helpers.predefined_type(
    "x",
    "LengthPercentage",
    "computed::LengthPercentage::zero()",
    engines="gecko",
    animation_value_type="ComputedValue",
    spec="https://svgwg.org/svg2-draft/geometry.html#X",
)}

${helpers.predefined_type(
    "y",
    "LengthPercentage",
    "computed::LengthPercentage::zero()",
    engines="gecko",
    animation_value_type="ComputedValue",
    spec="https://svgwg.org/svg2-draft/geometry.html#Y",
)}

${helpers.predefined_type(
    "cx",
    "LengthPercentage",
    "computed::LengthPercentage::zero()",
    engines="gecko",
    animation_value_type="ComputedValue",
    spec="https://svgwg.org/svg2-draft/geometry.html#CX",
)}

${helpers.predefined_type(
    "cy",
    "LengthPercentage",
    "computed::LengthPercentage::zero()",
    engines="gecko",
    animation_value_type="ComputedValue",
    spec="https://svgwg.org/svg2-draft/geometry.html#CY",
)}

${helpers.predefined_type(
    "rx",
    "NonNegativeLengthPercentageOrAuto",
    "computed::NonNegativeLengthPercentageOrAuto::auto()",
    engines="gecko",
    animation_value_type="LengthPercentageOrAuto",
    spec="https://svgwg.org/svg2-draft/geometry.html#RX",
)}

${helpers.predefined_type(
    "ry",
    "NonNegativeLengthPercentageOrAuto",
    "computed::NonNegativeLengthPercentageOrAuto::auto()",
    engines="gecko",
    animation_value_type="LengthPercentageOrAuto",
    spec="https://svgwg.org/svg2-draft/geometry.html#RY",
)}

${helpers.predefined_type(
    "r",
    "NonNegativeLengthPercentage",
    "computed::NonNegativeLengthPercentage::zero()",
    engines="gecko",
    animation_value_type="LengthPercentage",
    spec="https://svgwg.org/svg2-draft/geometry.html#R",
)}
