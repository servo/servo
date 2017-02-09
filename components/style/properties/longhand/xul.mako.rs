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
                         gecko_inexhaustive=True,
                         animatable=False,
                         alias="-webkit-box-align",
                         spec="Nonstandard (https://developer.mozilla.org/en-US/docs/Web/CSS/box-align)")}

${helpers.predefined_type("-moz-box-flex", "Number", "0.0", "parse_non_negative",
                          products="gecko", gecko_ffi_name="mBoxFlex",
                          needs_context=False,
                          animatable=False,
                          alias="-webkit-box-flex",
                          spec="Nonstandard (https://developer.mozilla.org/en-US/docs/Web/CSS/box-flex)")}


${helpers.single_keyword("-moz-stack-sizing", "stretch-to-fit ignore",
                         products="gecko", gecko_ffi_name="mStretchStack",
                         gecko_constant_prefix="NS_STYLE_STACK_SIZING",
                         animatable=False,
                         spec="Nonstandard (https://developer.mozilla.org/en-US/docs/Web/CSS/-moz-stack-sizing)")}
