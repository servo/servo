/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />

<% data.new_style_struct("Table", inherited=False) %>

${helpers.single_keyword(
    "table-layout",
    "auto fixed",
    engines="gecko servo-2013",
    gecko_ffi_name="mLayoutStrategy",
    animation_value_type="discrete",
    spec="https://drafts.csswg.org/css-tables/#propdef-table-layout",
    servo_restyle_damage="reflow",
)}

${helpers.predefined_type(
    "-x-span",
    "Integer",
    "1",
    engines="gecko",
    spec="Internal-only (for `<col span>` pres attr)",
    animation_value_type="none",
    enabled_in="",
)}
