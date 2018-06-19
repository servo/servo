/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />
<% from data import Method %>

// Non-standard properties that Gecko uses for XUL elements.
<% data.new_style_struct("XUL", inherited=False) %>

${helpers.single_keyword("-moz-box-align", "stretch start center baseline end",
                         products="gecko", gecko_ffi_name="mBoxAlign",
                         gecko_enum_prefix="StyleBoxAlign",
                         animation_value_type="discrete",
                         alias="-webkit-box-align",
                         spec="Nonstandard (https://developer.mozilla.org/en-US/docs/Web/CSS/box-align)")}

${helpers.single_keyword("-moz-box-direction", "normal reverse",
                         products="gecko", gecko_ffi_name="mBoxDirection",
                         gecko_enum_prefix="StyleBoxDirection",
                         animation_value_type="discrete",
                         alias="-webkit-box-direction",
                         spec="Nonstandard (https://developer.mozilla.org/en-US/docs/Web/CSS/box-direction)")}

${helpers.predefined_type("-moz-box-flex", "NonNegativeNumber", "From::from(0.)",
                          products="gecko", gecko_ffi_name="mBoxFlex",
                          animation_value_type="NonNegativeNumber",
                          alias="-webkit-box-flex",
                          spec="Nonstandard (https://developer.mozilla.org/en-US/docs/Web/CSS/box-flex)")}

${helpers.single_keyword("-moz-box-orient", "horizontal vertical",
                         products="gecko", gecko_ffi_name="mBoxOrient",
                         extra_gecko_aliases="inline-axis=horizontal block-axis=vertical",
                         gecko_enum_prefix="StyleBoxOrient",
                         animation_value_type="discrete",
                         alias="-webkit-box-orient",
                         spec="Nonstandard (https://developer.mozilla.org/en-US/docs/Web/CSS/box-orient)")}

${helpers.single_keyword("-moz-box-pack", "start center end justify",
                         products="gecko", gecko_ffi_name="mBoxPack",
                         gecko_enum_prefix="StyleBoxPack",
                         animation_value_type="discrete",
                         alias="-webkit-box-pack",
                         spec="Nonstandard (https://developer.mozilla.org/en-US/docs/Web/CSS/box-pack)")}

${helpers.single_keyword("-moz-stack-sizing", "stretch-to-fit ignore ignore-horizontal ignore-vertical",
                         products="gecko", gecko_ffi_name="mStackSizing",
                         gecko_enum_prefix="StyleStackSizing",
                         animation_value_type="discrete",
                         spec="Nonstandard (https://developer.mozilla.org/en-US/docs/Web/CSS/-moz-stack-sizing)")}

${helpers.predefined_type("-moz-box-ordinal-group", "Integer", "0",
                          parse_method="parse_non_negative",
                          products="gecko",
                          alias="-webkit-box-ordinal-group",
                          gecko_ffi_name="mBoxOrdinal",
                          animation_value_type="discrete",
                          spec="Nonstandard (https://developer.mozilla.org/en-US/docs/Web/CSS/-moz-box-ordinal-group)")}
