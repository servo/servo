/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />
<% from data import ALL_SIDES %>
<% data.new_style_struct("Padding", inherited=False) %>

% for side in ALL_SIDES:
    ${helpers.predefined_type("padding-%s" % side[0], "LengthOrPercentage",
                               "computed::LengthOrPercentage::Length(Au(0))",
                               "parse_non_negative",
                               animatable=True,
                               logical = side[1])}
% endfor
