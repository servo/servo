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

${helpers.single_keyword("text-anchor", "start middle end", products="gecko")}

// Section 11 - Painting: Filling, Stroking and Marker Symbols
${helpers.single_keyword("color-interpolation", "auto sRGB linearRGB", products="gecko")}

${helpers.single_keyword("color-interpolation-filters",
                         "auto sRGB linearRGB",
                         products="gecko",
                         gecko_constant_prefix="NS_STYLE_COLOR_INTERPOLATION")}

${helpers.predefined_type("fill-opacity", "Opacity", "1.0", products="gecko")}

${helpers.single_keyword("fill-rule", "nonzero evenodd", products="gecko")}

${helpers.single_keyword("shape-rendering",
                         "auto optimizeSpeed crispEdges geometricPrecision",
                         products="gecko")}

${helpers.single_keyword("stroke-linecap", "butt round square", products="gecko")}

${helpers.single_keyword("stroke-linejoin", "miter round bevel", products="gecko")}

${helpers.predefined_type("stroke-miterlimit", "Number", "4.0", "parse_at_least_one",
                          products="gecko")}

${helpers.predefined_type("stroke-opacity", "Opacity", "1.0", products="gecko")}

// Section 14 - Clipping, Masking and Compositing
${helpers.single_keyword("clip-rule", "nonzero evenodd",
                         products="gecko",
                         gecko_constant_prefix="NS_STYLE_FILL_RULE")}
