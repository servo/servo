/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />
<% from data import ALL_SIDES, DEFAULT_RULES_AND_PAGE, maybe_moz_logical_alias %>
<% data.new_style_struct("Margin", inherited=False) %>

% for side in ALL_SIDES:
    <%
        spec = "https://drafts.csswg.org/css-box/#propdef-margin-%s" % side[0]
        if side[1]:
            spec = "https://drafts.csswg.org/css-logical-props/#propdef-margin-%s" % side[1]
    %>
    ${helpers.predefined_type(
        "margin-%s" % side[0],
        "LengthPercentageOrAuto",
        "computed::LengthPercentageOrAuto::zero()",
        engines="gecko servo-2013 servo-2020",
        aliases=maybe_moz_logical_alias(engine, side, "-moz-margin-%s"),
        allow_quirks="No" if side[1] else "Yes",
        animation_value_type="ComputedValue",
        logical=side[1],
        logical_group="margin",
        spec=spec,
        rule_types_allowed=DEFAULT_RULES_AND_PAGE,
        servo_restyle_damage="reflow"
    )}
% endfor

% for side in ALL_SIDES:
    ${helpers.predefined_type(
        "scroll-margin-%s" % side[0],
        "Length",
        "computed::Length::zero()",
        engines="gecko",
        logical=side[1],
        logical_group="scroll-margin",
        spec="https://drafts.csswg.org/css-scroll-snap-1/#propdef-scroll-margin-%s" % side[0],
        animation_value_type="ComputedValue",
    )}
% endfor
