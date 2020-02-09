/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />

<% data.new_style_struct("InheritedTable", inherited=True, gecko_name="TableBorder") %>

${helpers.single_keyword(
    "border-collapse",
    "separate collapse",
    engines="gecko servo-2013",
    gecko_enum_prefix="StyleBorderCollapse",
    animation_value_type="discrete",
    spec="https://drafts.csswg.org/css-tables/#propdef-border-collapse",
    servo_restyle_damage = "reflow",
)}

${helpers.single_keyword(
    "empty-cells",
    "show hide",
    engines="gecko servo-2013",
    gecko_constant_prefix="NS_STYLE_TABLE_EMPTY_CELLS",
    animation_value_type="discrete",
    spec="https://drafts.csswg.org/css-tables/#propdef-empty-cells",
    servo_restyle_damage="rebuild_and_reflow",
)}

${helpers.single_keyword(
    "caption-side",
    "top bottom",
    engines="gecko servo-2013",
    extra_gecko_values="right left top-outside bottom-outside",
    needs_conversion="True",
    animation_value_type="discrete",
    spec="https://drafts.csswg.org/css-tables/#propdef-caption-side",
    servo_restyle_damage="rebuild_and_reflow",
)}

${helpers.predefined_type(
    "border-spacing",
    "BorderSpacing",
    "computed::BorderSpacing::zero()",
    engines="gecko servo-2013 servo-2020",
    servo_2020_pref="layout.2020.unimplemented",
    animation_value_type="BorderSpacing",
    boxed=True,
    spec="https://drafts.csswg.org/css-tables/#propdef-border-spacing",
    servo_restyle_damage = "reflow",
)}
