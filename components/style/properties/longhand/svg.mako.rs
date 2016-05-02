/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />

<% data.new_style_struct("SVG", inherited=False, gecko_name="SVGReset") %>

${helpers.single_keyword("dominant-baseline",
                 """auto use-script no-change reset-size ideographic alphabetic hanging
                    mathematical central middle text-after-edge text-before-edge""",
                 products="gecko")}

${helpers.single_keyword("vector-effect", "none non-scaling-stroke", products="gecko")}

// Section 13 - Gradients and Patterns

${helpers.predefined_type(
    "stop-color", "CSSColor",
    "::cssparser::Color::RGBA(::cssparser::RGBA { red: 0., green: 0., blue: 0., alpha: 1. })",
    products="gecko")}

${helpers.predefined_type("stop-opacity", "Opacity", "1.0", products="gecko")}

// Section 15 - Filter Effects

${helpers.predefined_type(
    "flood-color", "CSSColor",
    "::cssparser::Color::RGBA(::cssparser::RGBA { red: 0., green: 0., blue: 0., alpha: 1. })",
    products="gecko")}

${helpers.predefined_type("flood-opacity", "Opacity", "1.0", products="gecko")}

${helpers.predefined_type(
    "lighting-color", "CSSColor",
    "::cssparser::Color::RGBA(::cssparser::RGBA { red: 1., green: 1., blue: 1., alpha: 1. })",
    products="gecko")}

// CSS Masking Module Level 1
// https://www.w3.org/TR/css-masking-1/
${helpers.single_keyword("mask-type", "luminance alpha", products="gecko")}
