/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />

// SVG 1.1 (Second Edition)
// https://www.w3.org/TR/SVG/
<% data.new_style_struct("InheritedSVG",
                         inherited=True,
                         gecko_name="SVG") %>

// TODO(emilio): Should some of these types be animatable?

// Section 10 - Text

${helpers.single_keyword("text-anchor",
                         "start middle end",
                         products="gecko",
                         animatable=False,
                         spec="https://www.w3.org/TR/SVG/text.html#TextAnchorProperty")}

// Section 11 - Painting: Filling, Stroking and Marker Symbols
${helpers.single_keyword("color-interpolation",
                         "auto sRGB linearRGB",
                         products="gecko",
                         animatable=False,
                         spec="https://www.w3.org/TR/SVG11/painting.html#ColorInterpolationProperty")}

${helpers.single_keyword("color-interpolation-filters", "auto sRGB linearRGB",
                         products="gecko",
                         gecko_constant_prefix="NS_STYLE_COLOR_INTERPOLATION",
                         animatable=False,
                         spec="https://www.w3.org/TR/SVG11/painting.html#ColorInterpolationFiltersProperty")}

${helpers.predefined_type("fill-opacity", "Opacity", "1.0",
                          products="gecko", animatable=False,
                          spec="https://www.w3.org/TR/SVG11/painting.html#FillOpacityProperty")}

${helpers.single_keyword("fill-rule", "nonzero evenodd",
                         gecko_enum_prefix="StyleFillRule",
                         gecko_inexhaustive=True,
                         products="gecko", animatable=False,
                         spec="https://www.w3.org/TR/SVG11/painting.html#FillRuleProperty")}

${helpers.single_keyword("shape-rendering",
                         "auto optimizeSpeed crispEdges geometricPrecision",
                         products="gecko",
                         animatable=False,
                         spec="https://www.w3.org/TR/SVG11/painting.html#ShapeRenderingProperty")}

${helpers.single_keyword("stroke-linecap", "butt round square",
                         products="gecko", animatable=False,
                         spec="https://www.w3.org/TR/SVG11/painting.html#StrokeLinecapProperty")}

${helpers.single_keyword("stroke-linejoin", "miter round bevel",
                         products="gecko", animatable=False,
                         spec="https://www.w3.org/TR/SVG11/painting.html#StrokeLinejoinProperty")}

${helpers.predefined_type("stroke-miterlimit", "Number", "4.0",
                          "parse_at_least_one", products="gecko",
                          needs_context=False,
                          animatable=False,
                          spec="https://www.w3.org/TR/SVG11/painting.html#StrokeMiterlimitProperty")}

${helpers.predefined_type("stroke-opacity", "Opacity", "1.0",
                          products="gecko", animatable=False,
                          spec="https://www.w3.org/TR/SVG11/painting.html#StrokeOpacityProperty")}

// Section 14 - Clipping, Masking and Compositing
${helpers.single_keyword("clip-rule", "nonzero evenodd",
                         products="gecko",
                         gecko_enum_prefix="StyleFillRule",
                         gecko_inexhaustive=True,
                         animatable=False,
                         spec="https://www.w3.org/TR/SVG11/masking.html#ClipRuleProperty")}
