/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />

<% data.new_style_struct("InheritedTable", inherited=True, gecko_name="TableBorder") %>

${helpers.single_keyword("border-collapse", "separate collapse",
                         gecko_constant_prefix="NS_STYLE_BORDER",
                         animation_value_type="discrete",
                         spec="https://drafts.csswg.org/css-tables/#propdef-border-collapse")}
${helpers.single_keyword("empty-cells", "show hide",
                         gecko_constant_prefix="NS_STYLE_TABLE_EMPTY_CELLS",
                         animation_value_type="discrete",
                         spec="https://drafts.csswg.org/css-tables/#propdef-empty-cells")}
${helpers.single_keyword("caption-side", "top bottom",
                         extra_gecko_values="right left top-outside bottom-outside",
                         needs_conversion="True",
                         animation_value_type="discrete",
                         spec="https://drafts.csswg.org/css-tables/#propdef-caption-side")}

${helpers.predefined_type("border-spacing",
                          "BorderSpacing",
                          "computed::BorderSpacing::zero()",
                          animation_value_type="BorderSpacing",
                          boxed=True,
                          spec="https://drafts.csswg.org/css-tables/#propdef-border-spacing")}
