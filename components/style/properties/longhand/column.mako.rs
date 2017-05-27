/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />

<% data.new_style_struct("Column", inherited=False) %>

${helpers.predefined_type("column-width",
                          "length::LengthOrAuto",
                          "Either::Second(Auto)",
                          initial_specified_value="Either::Second(Auto)",
                          parse_method="parse_non_negative_length",
                          extra_prefixes="moz",
                          animation_value_type="ComputedValue",
                          experimental=True,
                          spec="https://drafts.csswg.org/css-multicol/#propdef-column-width")}


${helpers.predefined_type("column-count",
                          "IntegerOrAuto",
                          "Either::Second(Auto)",
                          parse_method="parse_positive",
                          initial_specified_value="Either::Second(Auto)",
                          experimental="True",
                          animation_value_type="ComputedValue",
                          extra_prefixes="moz",
                          spec="https://drafts.csswg.org/css-multicol/#propdef-column-count")}

${helpers.predefined_type("column-gap",
                          "length::LengthOrNormal",
                          "Either::Second(Normal)",
                          parse_method='parse_non_negative_length',
                          extra_prefixes="moz",
                          experimental=True,
                          animation_value_type="ComputedValue",
                          spec="https://drafts.csswg.org/css-multicol/#propdef-column-gap")}

${helpers.single_keyword("column-fill", "balance auto", extra_prefixes="moz",
                         products="gecko", animation_value_type="none",
                         spec="https://drafts.csswg.org/css-multicol/#propdef-column-fill")}

${helpers.predefined_type("column-rule-width", "BorderWidth", "Au::from_px(3)",
                          initial_specified_value="specified::BorderWidth::Medium",
                          products="gecko", computed_type="::app_units::Au",
                          spec="https://drafts.csswg.org/css-multicol/#propdef-column-rule-width",
                          animation_value_type="ComputedValue", extra_prefixes="moz")}

// https://drafts.csswg.org/css-multicol-1/#crc
${helpers.predefined_type("column-rule-color", "CSSColor",
                          "::cssparser::Color::CurrentColor",
                          initial_specified_value="specified::CSSColor::currentcolor()",
                          products="gecko", animation_value_type="IntermediateColor", extra_prefixes="moz",
                          complex_color=True, need_clone=True,
                          spec="https://drafts.csswg.org/css-multicol/#propdef-column-rule-color")}

${helpers.single_keyword("column-span", "none all",
                         products="gecko", animation_value_type="none",
                         spec="https://drafts.csswg.org/css-multicol/#propdef-column-span")}

${helpers.single_keyword("column-rule-style",
                         "none hidden dotted dashed solid double groove ridge inset outset",
                         products="gecko", extra_prefixes="moz",
                         gecko_constant_prefix="NS_STYLE_BORDER_STYLE",
                         animation_value_type="none",
                         spec="https://drafts.csswg.org/css-multicol/#propdef-column-rule-style")}
