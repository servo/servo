/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />

// SVG 1.1 (Second Edition)
// https://www.w3.org/TR/SVG/
<% data.new_style_struct("InheritedSVG", inherited=True, gecko_name="SVG") %>

// Section 10 - Text

${helpers.single_keyword(
    "dominant-baseline",
    """auto ideographic alphabetic hanging mathematical central middle
       text-after-edge text-before-edge""",
    engines="gecko",
    animation_value_type="discrete",
    spec="https://www.w3.org/TR/css-inline-3/#propdef-dominant-baseline",
    gecko_enum_prefix="StyleDominantBaseline",
)}

${helpers.single_keyword(
    "text-anchor",
    "start middle end",
    engines="gecko",
    animation_value_type="discrete",
    spec="https://www.w3.org/TR/SVG/text.html#TextAnchorProperty",
    gecko_enum_prefix="StyleTextAnchor",
)}

// Section 11 - Painting: Filling, Stroking and Marker Symbols
${helpers.single_keyword(
    "color-interpolation",
    "srgb auto linearrgb",
    engines="gecko",
    animation_value_type="discrete",
    spec="https://www.w3.org/TR/SVG11/painting.html#ColorInterpolationProperty",
    gecko_enum_prefix="StyleColorInterpolation",
)}

${helpers.single_keyword(
    "color-interpolation-filters",
    "linearrgb auto srgb",
    engines="gecko",
    animation_value_type="discrete",
    spec="https://www.w3.org/TR/SVG11/painting.html#ColorInterpolationFiltersProperty",
    gecko_enum_prefix="StyleColorInterpolation",
)}

${helpers.predefined_type(
    "fill",
    "SVGPaint",
    "crate::values::computed::SVGPaint::black()",
    engines="gecko",
    animation_value_type="IntermediateSVGPaint",
    boxed=True,
    spec="https://www.w3.org/TR/SVG2/painting.html#SpecifyingFillPaint",
)}

${helpers.predefined_type(
    "fill-opacity",
    "SVGOpacity",
    "Default::default()",
    engines="gecko",
    animation_value_type="ComputedValue",
    spec="https://svgwg.org/svg2-draft/painting.html#FillOpacity",
)}

${helpers.predefined_type(
    "fill-rule",
    "FillRule",
    "Default::default()",
    engines="gecko",
    animation_value_type="discrete",
    spec="https://www.w3.org/TR/SVG11/painting.html#FillRuleProperty",
)}

${helpers.single_keyword(
    "shape-rendering",
    "auto optimizespeed crispedges geometricprecision",
    engines="gecko",
    animation_value_type="discrete",
    spec="https://www.w3.org/TR/SVG11/painting.html#ShapeRenderingProperty",
    gecko_enum_prefix = "StyleShapeRendering",
)}

${helpers.predefined_type(
    "stroke",
    "SVGPaint",
    "Default::default()",
    engines="gecko",
    animation_value_type="IntermediateSVGPaint",
    boxed=True,
    spec="https://www.w3.org/TR/SVG2/painting.html#SpecifyingStrokePaint",
)}

${helpers.predefined_type(
    "stroke-width",
    "SVGWidth",
    "computed::SVGWidth::one()",
    engines="gecko",
    animation_value_type="crate::values::computed::SVGWidth",
    spec="https://www.w3.org/TR/SVG2/painting.html#StrokeWidth",
)}

${helpers.single_keyword(
    "stroke-linecap",
    "butt round square",
    engines="gecko",
    animation_value_type="discrete",
    spec="https://www.w3.org/TR/SVG11/painting.html#StrokeLinecapProperty",
    gecko_enum_prefix = "StyleStrokeLinecap",
)}

${helpers.single_keyword(
    "stroke-linejoin",
    "miter round bevel",
    engines="gecko",
    animation_value_type="discrete",
    spec="https://www.w3.org/TR/SVG11/painting.html#StrokeLinejoinProperty",
    gecko_enum_prefix = "StyleStrokeLinejoin",
)}

${helpers.predefined_type(
    "stroke-miterlimit",
    "NonNegativeNumber",
    "From::from(4.0)",
    engines="gecko",
    animation_value_type="crate::values::computed::NonNegativeNumber",
    spec="https://www.w3.org/TR/SVG2/painting.html#StrokeMiterlimitProperty",
)}

${helpers.predefined_type(
    "stroke-opacity",
    "SVGOpacity",
    "Default::default()",
    engines="gecko",
    animation_value_type="ComputedValue",
    spec="https://svgwg.org/svg2-draft/painting.html#StrokeOpacity",
)}

${helpers.predefined_type(
    "stroke-dasharray",
    "SVGStrokeDashArray",
    "Default::default()",
    engines="gecko",
    animation_value_type="crate::values::computed::SVGStrokeDashArray",
    spec="https://www.w3.org/TR/SVG2/painting.html#StrokeDashing",
)}

${helpers.predefined_type(
    "stroke-dashoffset",
    "SVGLength",
    "computed::SVGLength::zero()",
    engines="gecko",
    animation_value_type="ComputedValue",
    spec="https://www.w3.org/TR/SVG2/painting.html#StrokeDashing",
)}

// Section 14 - Clipping, Masking and Compositing
${helpers.predefined_type(
    "clip-rule",
    "FillRule",
    "Default::default()",
    engines="gecko",
    animation_value_type="discrete",
    spec="https://www.w3.org/TR/SVG11/masking.html#ClipRuleProperty",
)}

${helpers.predefined_type(
    "marker-start",
    "url::UrlOrNone",
    "computed::url::UrlOrNone::none()",
    engines="gecko",
    animation_value_type="discrete",
    spec="https://www.w3.org/TR/SVG2/painting.html#VertexMarkerProperties",
)}

${helpers.predefined_type(
    "marker-mid",
    "url::UrlOrNone",
    "computed::url::UrlOrNone::none()",
    engines="gecko",
    animation_value_type="discrete",
    spec="https://www.w3.org/TR/SVG2/painting.html#VertexMarkerProperties",
)}

${helpers.predefined_type(
    "marker-end",
    "url::UrlOrNone",
    "computed::url::UrlOrNone::none()",
    engines="gecko",
    animation_value_type="discrete",
    spec="https://www.w3.org/TR/SVG2/painting.html#VertexMarkerProperties",
)}

${helpers.predefined_type(
    "paint-order",
    "SVGPaintOrder",
    "computed::SVGPaintOrder::normal()",
    engines="gecko",
    animation_value_type="discrete",
    spec="https://www.w3.org/TR/SVG2/painting.html#PaintOrder",
)}

${helpers.predefined_type(
    "-moz-context-properties",
    "MozContextProperties",
    "computed::MozContextProperties::default()",
    engines="gecko",
    enabled_in="chrome",
    gecko_pref="svg.context-properties.content.enabled",
    has_effect_on_gecko_scrollbars=False,
    animation_value_type="none",
    spec="Nonstandard (https://developer.mozilla.org/en-US/docs/Web/CSS/-moz-context-properties)",
)}
