/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />

<% data.new_style_struct("Column", inherited=False) %>

${helpers.predefined_type(
    "column-width",
    "length::NonNegativeLengthOrAuto",
    "Either::Second(Auto)",
    initial_specified_value="Either::Second(Auto)",
    extra_prefixes="moz",
    animation_value_type="NonNegativeLengthOrAuto",
    servo_pref="layout.columns.enabled",
    spec="https://drafts.csswg.org/css-multicol/#propdef-column-width",
    servo_restyle_damage="rebuild_and_reflow",
)}

${helpers.predefined_type(
    "column-count",
    "ColumnCount",
    "computed::ColumnCount::auto()",
    initial_specified_value="specified::ColumnCount::auto()",
    servo_pref="layout.columns.enabled",
    animation_value_type="AnimatedColumnCount",
    extra_prefixes="moz",
    spec="https://drafts.csswg.org/css-multicol/#propdef-column-count",
    servo_restyle_damage="rebuild_and_reflow",
)}

${helpers.single_keyword(
    "column-fill",
    "balance auto",
    extra_prefixes="moz",
    products="gecko",
    animation_value_type="discrete",
    gecko_enum_prefix="StyleColumnFill",
    spec="https://drafts.csswg.org/css-multicol/#propdef-column-fill",
)}

${helpers.predefined_type(
    "column-rule-width",
    "BorderSideWidth",
    "crate::values::computed::NonNegativeLength::new(3.)",
    initial_specified_value="specified::BorderSideWidth::Medium",
    computed_type="crate::values::computed::NonNegativeLength",
    products="gecko",
    spec="https://drafts.csswg.org/css-multicol/#propdef-column-rule-width",
    animation_value_type="NonNegativeLength",
    extra_prefixes="moz",
)}

// https://drafts.csswg.org/css-multicol-1/#crc
${helpers.predefined_type(
    "column-rule-color",
    "Color",
    "computed_value::T::currentcolor()",
    initial_specified_value="specified::Color::currentcolor()",
    products="gecko",
    animation_value_type="AnimatedColor",
    extra_prefixes="moz",
    ignored_when_colors_disabled=True,
    spec="https://drafts.csswg.org/css-multicol/#propdef-column-rule-color",
)}

// FIXME: Remove enabled_in="ua" once column-span is enabled on nightly (bug 1423383).
${helpers.single_keyword(
    "column-span",
    "none all",
    products="gecko",
    animation_value_type="discrete",
    gecko_enum_prefix="StyleColumnSpan",
    gecko_pref="layout.css.column-span.enabled",
    enabled_in="ua",
    spec="https://drafts.csswg.org/css-multicol/#propdef-column-span",
    extra_prefixes="moz:layout.css.column-span.enabled",
)}

${helpers.predefined_type(
    "column-rule-style",
    "BorderStyle",
    "computed::BorderStyle::None",
    needs_context=False,
    initial_specified_value="specified::BorderStyle::None",
    products="gecko",
    extra_prefixes="moz",
    animation_value_type="discrete",
    spec="https://drafts.csswg.org/css-multicol/#propdef-column-rule-style",
)}
