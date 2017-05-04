/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />
<% from data import ALL_SIDES, maybe_moz_logical_alias %>
<% data.new_style_struct("Margin", inherited=False) %>

% for side in ALL_SIDES:
    <%
        spec = "https://drafts.csswg.org/css-box/#propdef-margin-%s" % side[0]
        if side[1]:
            spec = "https://drafts.csswg.org/css-logical-props/#propdef-margin-%s" % side[1]
    %>
    ${helpers.predefined_type("margin-%s" % side[0], "LengthOrPercentageOrAuto",
                              "computed::LengthOrPercentageOrAuto::Length(Au(0))",
                              alias=maybe_moz_logical_alias(product, side, "-moz-margin-%s"),
                              allow_quirks=not side[1],
                              animation_value_type="ComputedValue", logical = side[1], spec = spec,
                              allowed_in_page_rule=True)}
% endfor
