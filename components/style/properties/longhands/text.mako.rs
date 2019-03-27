/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />
<% from data import Method %>

<% data.new_style_struct("Text", inherited=False, gecko_name="TextReset") %>

${helpers.predefined_type(
    "text-overflow",
    "TextOverflow",
    "computed::TextOverflow::get_initial_value()",
    animation_value_type="discrete",
    boxed=True,
    flags="APPLIES_TO_PLACEHOLDER",
    spec="https://drafts.csswg.org/css-ui/#propdef-text-overflow",
    servo_restyle_damage="rebuild_and_reflow",
)}

${helpers.single_keyword(
    "unicode-bidi",
    "normal embed isolate bidi-override isolate-override plaintext",
    animation_value_type="none",
    spec="https://drafts.csswg.org/css-writing-modes/#propdef-unicode-bidi",
    servo_restyle_damage="rebuild_and_reflow",
)}

${helpers.predefined_type(
    "text-decoration-line",
    "TextDecorationLine",
    "specified::TextDecorationLine::none()",
    initial_specified_value="specified::TextDecorationLine::none()",
    animation_value_type="discrete",
    flags="APPLIES_TO_FIRST_LETTER APPLIES_TO_FIRST_LINE APPLIES_TO_PLACEHOLDER",
    spec="https://drafts.csswg.org/css-text-decor/#propdef-text-decoration-line",
    servo_restyle_damage="rebuild_and_reflow",
)}

${helpers.single_keyword(
    "text-decoration-style",
    "solid double dotted dashed wavy -moz-none",
    products="gecko",
    animation_value_type="discrete",
    flags="APPLIES_TO_FIRST_LETTER APPLIES_TO_FIRST_LINE APPLIES_TO_PLACEHOLDER",
    spec="https://drafts.csswg.org/css-text-decor/#propdef-text-decoration-style",
)}

${helpers.predefined_type(
    "text-decoration-color",
    "Color",
    "computed_value::T::currentcolor()",
    initial_specified_value="specified::Color::currentcolor()",
    products="gecko",
    animation_value_type="AnimatedColor",
    ignored_when_colors_disabled=True,
    flags="APPLIES_TO_FIRST_LETTER APPLIES_TO_FIRST_LINE APPLIES_TO_PLACEHOLDER",
    spec="https://drafts.csswg.org/css-text-decor/#propdef-text-decoration-color",
)}

${helpers.predefined_type(
    "initial-letter",
    "InitialLetter",
    "computed::InitialLetter::normal()",
    initial_specified_value="specified::InitialLetter::normal()",
    animation_value_type="discrete",
    products="gecko",
    flags="APPLIES_TO_FIRST_LETTER",
    gecko_pref="layout.css.initial-letter.enabled",
    spec="https://drafts.csswg.org/css-inline/#sizing-drop-initials",
)}
