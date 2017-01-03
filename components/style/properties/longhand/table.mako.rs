/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />

<% data.new_style_struct("Table", inherited=False) %>

${helpers.single_keyword("table-layout", "auto fixed",
                         gecko_ffi_name="mLayoutStrategy", animatable=False,
                         spec="https://drafts.csswg.org/css-tables/#propdef-table-layout")}
