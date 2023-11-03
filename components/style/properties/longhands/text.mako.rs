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
    engines="gecko servo",
    servo_pref="layout.legacy_layout",
    animation_value_type="discrete",
    boxed=True,
    spec="https://drafts.csswg.org/css-ui/#propdef-text-overflow",
    servo_restyle_damage="rebuild_and_reflow",
)}

${helpers.single_keyword(
    "unicode-bidi",
    "normal embed isolate bidi-override isolate-override plaintext",
    engines="gecko servo",
    servo_pref="layout.legacy_layout",
    gecko_enum_prefix="StyleUnicodeBidi",
    animation_value_type="none",
    spec="https://drafts.csswg.org/css-writing-modes/#propdef-unicode-bidi",
    servo_restyle_damage="rebuild_and_reflow",
)}

${helpers.predefined_type(
    "text-decoration-line",
    "TextDecorationLine",
    "specified::TextDecorationLine::none()",
    engines="gecko servo",
    initial_specified_value="specified::TextDecorationLine::none()",
    animation_value_type="discrete",
    spec="https://drafts.csswg.org/css-text-decor/#propdef-text-decoration-line",
    servo_restyle_damage="rebuild_and_reflow",
)}

${helpers.single_keyword(
    "text-decoration-style",
    "solid double dotted dashed wavy -moz-none",
    engines="gecko servo",
    gecko_enum_prefix="StyleTextDecorationStyle",
    animation_value_type="discrete",
    spec="https://drafts.csswg.org/css-text-decor/#propdef-text-decoration-style",
)}

${helpers.predefined_type(
    "text-decoration-color",
    "Color",
    "computed_value::T::currentcolor()",
    engines="gecko servo",
    initial_specified_value="specified::Color::currentcolor()",
    animation_value_type="AnimatedColor",
    ignored_when_colors_disabled=True,
    spec="https://drafts.csswg.org/css-text-decor/#propdef-text-decoration-color",
)}

${helpers.predefined_type(
    "initial-letter",
    "InitialLetter",
    "computed::InitialLetter::normal()",
    engines="gecko",
    initial_specified_value="specified::InitialLetter::normal()",
    animation_value_type="discrete",
    gecko_pref="layout.css.initial-letter.enabled",
    spec="https://drafts.csswg.org/css-inline/#sizing-drop-initials",
)}

${helpers.predefined_type(
   "text-decoration-thickness",
   "TextDecorationLength",
   "generics::text::GenericTextDecorationLength::Auto",
   engines="gecko",
   initial_specified_value="generics::text::GenericTextDecorationLength::Auto",
   animation_value_type="ComputedValue",
   spec="https://drafts.csswg.org/css-text-decor-4/#text-decoration-width-property"
)}
