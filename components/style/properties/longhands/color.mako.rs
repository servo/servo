/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />

<% data.new_style_struct("Color", inherited=True) %>

<% from data import to_rust_ident %>

${helpers.predefined_type(
    "color",
    "ColorPropertyValue",
    "::cssparser::RGBA::new(0, 0, 0, 255)",
    animation_value_type="AnimatedRGBA",
    flags="APPLIES_TO_FIRST_LETTER APPLIES_TO_FIRST_LINE APPLIES_TO_PLACEHOLDER",
    ignored_when_colors_disabled="True",
    spec="https://drafts.csswg.org/css-color/#color",
)}
