/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />

// SVG 1.1 (Second Edition)
// https://www.w3.org/TR/SVG/
<% data.new_style_struct("InheritedSVG",
                         inherited=True,
                         gecko_name="SVG") %>

// Section 10 - Text

${helpers.single_keyword("text-anchor",
                         "start middle end",
                         products="gecko",
                         animation_value_type="discrete",
                         spec="https://www.w3.org/TR/SVG/text.html#TextAnchorProperty")}

// Section 11 - Painting: Filling, Stroking and Marker Symbols
${helpers.single_keyword("color-interpolation",
                         "srgb auto linearrgb",
                         products="gecko",
                         animation_value_type="discrete",
                         spec="https://www.w3.org/TR/SVG11/painting.html#ColorInterpolationProperty")}

${helpers.single_keyword("color-interpolation-filters", "linearrgb auto srgb",
                         products="gecko",
                         gecko_constant_prefix="NS_STYLE_COLOR_INTERPOLATION",
                         animation_value_type="discrete",
                         spec="https://www.w3.org/TR/SVG11/painting.html#ColorInterpolationFiltersProperty")}

${helpers.predefined_type(
    "fill", "SVGPaint",
    "::values::computed::SVGPaint::black()",
    products="gecko",
    animation_value_type="IntermediateSVGPaint",
    boxed=True,
    spec="https://www.w3.org/TR/SVG2/painting.html#SpecifyingFillPaint")}

${helpers.predefined_type("fill-opacity", "SVGOpacity", "Default::default()",
                          products="gecko", animation_value_type="ComputedValue",
                          spec="https://www.w3.org/TR/SVG11/painting.html#FillOpacityProperty")}

${helpers.single_keyword("fill-rule", "nonzero evenodd",
                         gecko_enum_prefix="StyleFillRule",
                         products="gecko", animation_value_type="discrete",
                         spec="https://www.w3.org/TR/SVG11/painting.html#FillRuleProperty")}

${helpers.single_keyword("shape-rendering",
                         "auto optimizespeed crispedges geometricprecision",
                         products="gecko",
                         animation_value_type="discrete",
                         spec="https://www.w3.org/TR/SVG11/painting.html#ShapeRenderingProperty")}

${helpers.predefined_type(
    "stroke", "SVGPaint",
    "Default::default()",
    products="gecko",
    animation_value_type="IntermediateSVGPaint",
    boxed=True,
    spec="https://www.w3.org/TR/SVG2/painting.html#SpecifyingStrokePaint")}

${helpers.predefined_type(
    "stroke-width", "SVGWidth",
    "::values::computed::NonNegativeLength::new(1.).into()",
    products="gecko",
    animation_value_type="::values::computed::SVGWidth",
    spec="https://www.w3.org/TR/SVG2/painting.html#StrokeWidth",
)}

${helpers.single_keyword("stroke-linecap", "butt round square",
                         products="gecko", animation_value_type="discrete",
                         spec="https://www.w3.org/TR/SVG11/painting.html#StrokeLinecapProperty")}

${helpers.single_keyword("stroke-linejoin", "miter round bevel",
                         products="gecko", animation_value_type="discrete",
                         spec="https://www.w3.org/TR/SVG11/painting.html#StrokeLinejoinProperty")}

${helpers.predefined_type("stroke-miterlimit", "GreaterThanOrEqualToOneNumber",
                          "From::from(4.0)",
                          products="gecko",
                          animation_value_type="::values::computed::GreaterThanOrEqualToOneNumber",
                          spec="https://www.w3.org/TR/SVG11/painting.html#StrokeMiterlimitProperty")}

${helpers.predefined_type("stroke-opacity", "SVGOpacity", "Default::default()",
                          products="gecko", animation_value_type="ComputedValue",
                          spec="https://www.w3.org/TR/SVG11/painting.html#StrokeOpacityProperty")}

${helpers.predefined_type(
    "stroke-dasharray",
    "SVGStrokeDashArray",
    "Default::default()",
    products="gecko",
    animation_value_type="::values::computed::SVGStrokeDashArray",
    spec="https://www.w3.org/TR/SVG2/painting.html#StrokeDashing",
)}

${helpers.predefined_type(
    "stroke-dashoffset", "SVGLength",
    "Au(0).into()",
    products="gecko",
    animation_value_type="ComputedValue",
    spec="https://www.w3.org/TR/SVG2/painting.html#StrokeDashing",
)}

// Section 14 - Clipping, Masking and Compositing
${helpers.single_keyword("clip-rule", "nonzero evenodd",
                         products="gecko",
                         gecko_enum_prefix="StyleFillRule",
                         animation_value_type="discrete",
                         spec="https://www.w3.org/TR/SVG11/masking.html#ClipRuleProperty")}

${helpers.predefined_type("marker-start", "url::UrlOrNone", "computed::url::UrlOrNone::none()",
                          products="gecko",
                          animation_value_type="discrete",
                          spec="https://www.w3.org/TR/SVG2/painting.html#VertexMarkerProperties")}

${helpers.predefined_type("marker-mid", "url::UrlOrNone", "computed::url::UrlOrNone::none()",
                          products="gecko",
                          animation_value_type="discrete",
                          spec="https://www.w3.org/TR/SVG2/painting.html#VertexMarkerProperties")}

${helpers.predefined_type("marker-end", "url::UrlOrNone", "computed::url::UrlOrNone::none()",
                          products="gecko",
                          animation_value_type="discrete",
                          spec="https://www.w3.org/TR/SVG2/painting.html#VertexMarkerProperties")}

${helpers.predefined_type("paint-order", "SVGPaintOrder", "computed::SVGPaintOrder::normal()",
                          products="gecko",
                          animation_value_type="discrete",
                          spec="https://www.w3.org/TR/SVG2/painting.html#PaintOrder")}

${helpers.predefined_type("-moz-context-properties",
                          "MozContextProperties",
                          initial_value=None,
                          vector=True,
                          need_index=True,
                          animation_value_type="none",
                          products="gecko",
                          spec="Nonstandard (https://developer.mozilla.org/en-US/docs/Web/CSS/-moz-context-properties)",
                          allow_empty=True)}
