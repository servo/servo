/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />
<% from data import ALL_SIDES, maybe_moz_logical_alias %>
<% data.new_style_struct("Padding", inherited=False) %>

// APPLIES_TO_PLACEHOLDER so we can set it in UA  stylesheets.  But we use a
// !important value there, so pages can't set it.
% for side in ALL_SIDES:
    <%
        spec = "https://drafts.csswg.org/css-box/#propdef-padding-%s" % side[0]
        if side[1]:
            spec = "https://drafts.csswg.org/css-logical-props/#propdef-padding-%s" % side[1]
    %>
    ${helpers.predefined_type(
        "padding-%s" % side[0],
        "NonNegativeLengthPercentage",
        "computed::NonNegativeLengthPercentage::zero()",
        alias=maybe_moz_logical_alias(product, side, "-moz-padding-%s"),
        animation_value_type="NonNegativeLengthPercentage",
        logical=side[1],
        logical_group="padding",
        spec=spec,
        flags="APPLIES_TO_FIRST_LETTER APPLIES_TO_PLACEHOLDER GETCS_NEEDS_LAYOUT_FLUSH",
        allow_quirks=not side[1],
        servo_restyle_damage="reflow rebuild_and_reflow_inline"
    )}
% endfor

% for side in ALL_SIDES:
    ${helpers.predefined_type(
        "scroll-padding-%s" % side[0],
        "NonNegativeLengthPercentageOrAuto",
        "computed::NonNegativeLengthPercentageOrAuto::auto()",
        products="gecko",
        gecko_pref="layout.css.scroll-snap-v1.enabled",
        logical=side[1],
        logical_group="scroll-padding",
        spec="https://drafts.csswg.org/css-scroll-snap-1/#propdef-scroll-padding-%s" % side[0],
        animation_value_type="NonNegativeLengthPercentageOrAuto",
    )}
% endfor
