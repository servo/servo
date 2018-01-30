/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

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
    initial_specified_value="specified::Color::currentcolor()",
    animation_value_type="AnimatedColor",
    ignored_when_colors_disabled=True,
    spec="https://drafts.csswg.org/css-ui/#propdef-outline-color",
)}

${helpers.predefined_type(
    "outline-style",
    "OutlineStyle",
    "computed::OutlineStyle::none()",
    initial_specified_value="specified::OutlineStyle::none()",
    animation_value_type="discrete",
    spec="https://drafts.csswg.org/css-ui/#propdef-outline-style",
)}

${helpers.predefined_type("outline-width",
                          "BorderSideWidth",
                          "::values::computed::NonNegativeLength::new(3.)",
                          initial_specified_value="specified::BorderSideWidth::Medium",
                          computed_type="::values::computed::NonNegativeLength",
                          animation_value_type="NonNegativeLength",
                          spec="https://drafts.csswg.org/css-ui/#propdef-outline-width")}

// The -moz-outline-radius-* properties are non-standard and not on a standards track.
% for corner in ["topleft", "topright", "bottomright", "bottomleft"]:
    ${helpers.predefined_type("-moz-outline-radius-" + corner, "BorderCornerRadius",
        "computed::BorderCornerRadius::zero()",
        products="gecko",
        boxed=True,
        animation_value_type="BorderCornerRadius",
        spec="Nonstandard (https://developer.mozilla.org/en-US/docs/Web/CSS/-moz-outline-radius)")}
% endfor

${helpers.predefined_type("outline-offset", "Length", "::values::computed::Length::new(0.)",
                          products="servo gecko", animation_value_type="ComputedValue",
                          spec="https://drafts.csswg.org/css-ui/#propdef-outline-offset")}
