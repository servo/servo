/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%page args="helpers" />

${helpers.new_style_struct("Margin", is_inherited=False, gecko_name="nsStyleMargin")}

% for side in ["top", "right", "bottom", "left"]:
    ${helpers.predefined_type("margin-" + side, "LengthOrPercentageOrAuto",
                              "computed::LengthOrPercentageOrAuto::Length(Au(0))")}
% endfor
