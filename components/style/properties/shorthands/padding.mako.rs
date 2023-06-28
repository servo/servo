/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />

${helpers.four_sides_shorthand(
    "padding",
    "padding-%s",
    "specified::NonNegativeLengthPercentage::parse",
    engines="gecko servo",
    spec="https://drafts.csswg.org/css-box-3/#propdef-padding",
    allow_quirks="Yes",
)}

${helpers.two_properties_shorthand(
    "padding-block",
    "padding-block-start",
    "padding-block-end",
    "specified::NonNegativeLengthPercentage::parse",
    engines="gecko servo",
    spec="https://drafts.csswg.org/css-logical/#propdef-padding-block"
)}

${helpers.two_properties_shorthand(
    "padding-inline",
    "padding-inline-start",
    "padding-inline-end",
    "specified::NonNegativeLengthPercentage::parse",
    engines="gecko servo",
    spec="https://drafts.csswg.org/css-logical/#propdef-padding-inline"
)}

${helpers.four_sides_shorthand(
    "scroll-padding",
    "scroll-padding-%s",
    "specified::NonNegativeLengthPercentageOrAuto::parse",
    engines="gecko",
    spec="https://drafts.csswg.org/css-scroll-snap-1/#propdef-scroll-padding"
)}

${helpers.two_properties_shorthand(
    "scroll-padding-block",
    "scroll-padding-block-start",
    "scroll-padding-block-end",
    "specified::NonNegativeLengthPercentageOrAuto::parse",
    engines="gecko",
    spec="https://drafts.csswg.org/css-scroll-snap-1/#propdef-scroll-padding-block"
)}

${helpers.two_properties_shorthand(
    "scroll-padding-inline",
    "scroll-padding-inline-start",
    "scroll-padding-inline-end",
    "specified::NonNegativeLengthPercentageOrAuto::parse",
    engines="gecko",
    spec="https://drafts.csswg.org/css-scroll-snap-1/#propdef-scroll-padding-inline"
)}
