/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />
<% from data import PAGE_RULE %>

<% data.new_style_struct("Page", inherited=False) %>

${helpers.predefined_type(
    "size",
    "PageSize",
    "computed::PageSize::auto()",
    engines="gecko",
    gecko_pref="layout.css.page-size.enabled",
    initial_specified_value="specified::PageSize::auto()",
    spec="https://drafts.csswg.org/css-page-3/#page-size-prop",
    boxed=True,
    animation_value_type="none",
    rule_types_allowed=PAGE_RULE,
)}
