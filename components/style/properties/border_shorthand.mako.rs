/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%page args="helpers" />

// CSS Fragmentation Module Level 3
// https://www.w3.org/TR/css-break-3/
${helpers.switch_to_style_struct("Border")}

${helpers.single_keyword("box-decoration-break", "slice clone", products="gecko")}
