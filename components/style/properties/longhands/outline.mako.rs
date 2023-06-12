/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />
<% from data import Method %>

<% data.new_style_struct("Outline",
                         inherited=False,
                         additional_methods=[Method("outline_has_nonzero_width", "bool")]) %>

// TODO(pcwalton): `invert`
${helpers.predefined_type(
    "outline-color",
    "Color",
    "computed_value::T::currentcolor()",
    engines="gecko servo",
    initial_specified_value="specified::Color::currentcolor()",
    animation_value_type="AnimatedColor",
    ignored_when_colors_disabled=True,
    spec="https://drafts.csswg.org/css-ui/#propdef-outline-color",
)}

${helpers.predefined_type(
    "outline-style",
    "OutlineStyle",
    "computed::OutlineStyle::none()",
    engines="gecko servo",
    initial_specified_value="specified::OutlineStyle::none()",
    animation_value_type="discrete",
    spec="https://drafts.csswg.org/css-ui/#propdef-outline-style",
)}

${helpers.predefined_type(
    "outline-width",
    "BorderSideWidth",
    "app_units::Au::from_px(3)",
    engines="gecko servo",
    initial_specified_value="specified::BorderSideWidth::medium()",
    animation_value_type="NonNegativeLength",
    spec="https://drafts.csswg.org/css-ui/#propdef-outline-width",
)}

${helpers.predefined_type(
    "outline-offset",
    "Length",
    "crate::values::computed::Length::new(0.)",
    engines="gecko servo",
    animation_value_type="ComputedValue",
    spec="https://drafts.csswg.org/css-ui/#propdef-outline-offset",
)}
