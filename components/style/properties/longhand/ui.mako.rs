/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />
<% from data import Method %>

// CSS Basic User Interface Module Level 1
// https://drafts.csswg.org/css-ui-3/
<% data.new_style_struct("UI", inherited=False, gecko_name="UIReset") %>

${helpers.single_keyword("ime-mode", "normal auto active disabled inactive", products="gecko",
                         gecko_ffi_name="mIMEMode")}
