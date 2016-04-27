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

// CSS Masking Module Level 1
// https://www.w3.org/TR/css-masking-1/
${helpers.single_keyword("mask-type", "luminance alpha", products="gecko")}
