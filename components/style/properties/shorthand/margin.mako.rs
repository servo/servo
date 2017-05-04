/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />

${helpers.four_sides_shorthand("margin", "margin-%s", "specified::LengthOrPercentageOrAuto::parse",
                               spec="https://drafts.csswg.org/css-box/#propdef-margin",
                               allowed_in_page_rule=True,
                               allow_quirks=True)}
