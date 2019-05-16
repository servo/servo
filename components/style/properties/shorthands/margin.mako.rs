/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />

${helpers.four_sides_shorthand(
    "margin",
    "margin-%s",
    "specified::LengthPercentageOrAuto::parse",
    spec="https://drafts.csswg.org/css-box/#propdef-margin",
    allowed_in_page_rule=True,
    allow_quirks="Yes",
)}

${helpers.two_properties_shorthand(
    "margin-block",
    "margin-block-start",
    "margin-block-end",
    "specified::LengthPercentageOrAuto::parse",
    spec="https://drafts.csswg.org/css-logical/#propdef-margin-block"
)}

${helpers.two_properties_shorthand(
    "margin-inline",
    "margin-inline-start",
    "margin-inline-end",
    "specified::LengthPercentageOrAuto::parse",
    spec="https://drafts.csswg.org/css-logical/#propdef-margin-inline"
)}

${helpers.four_sides_shorthand(
    "scroll-margin",
    "scroll-margin-%s",
    "specified::Length::parse",
    spec="https://drafts.csswg.org/css-scroll-snap-1/#propdef-scroll-margin",
    products="gecko",
    gecko_pref="layout.css.scroll-snap-v1.enabled",
)}

${helpers.two_properties_shorthand(
    "scroll-margin-block",
    "scroll-margin-block-start",
    "scroll-margin-block-end",
    "specified::Length::parse",
    spec="https://drafts.csswg.org/css-scroll-snap-1/#propdef-scroll-margin-block",
    products="gecko",
    gecko_pref="layout.css.scroll-snap-v1.enabled",
)}

${helpers.two_properties_shorthand(
    "scroll-margin-inline",
    "scroll-margin-inline-start",
    "scroll-margin-inline-end",
    "specified::Length::parse",
    spec="https://drafts.csswg.org/css-scroll-snap-1/#propdef-scroll-margin-inline",
    products="gecko",
    gecko_pref="layout.css.scroll-snap-v1.enabled",
)}
