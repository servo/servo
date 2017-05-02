/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />

${helpers.four_sides_shorthand("padding", "padding-%s", "specified::LengthOrPercentage::parse",
                               spec="https://drafts.csswg.org/css-box-3/#propdef-padding",
                               allow_quirks=True)}
