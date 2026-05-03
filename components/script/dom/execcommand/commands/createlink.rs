/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use html5ever::local_name;
use js::context::JSContext;
use script_bindings::inheritance::Castable;

use crate::dom::ShadowIncluding;
use crate::dom::bindings::codegen::Bindings::HTMLAnchorElementBinding::HTMLAnchorElementMethods;
use crate::dom::bindings::str::DOMString;
use crate::dom::document::Document;
use crate::dom::element::Element;
use crate::dom::execcommand::basecommand::CommandName;
use crate::dom::html::htmlanchorelement::HTMLAnchorElement;
use crate::dom::selection::Selection;

/// <https://w3c.github.io/editing/docs/execCommand/#the-createlink-command>
pub(crate) fn execute_createlink_command(
    cx: &mut JSContext,
    document: &Document,
    selection: &Selection,
    value: DOMString,
) -> bool {
    // Step 1. If value is the empty string, return false.
    if value.is_empty() {
        return false;
    }
    // Step 2. For each editable a element that has an href attribute and
    // is an ancestor of some node effectively contained in the active range,
    // set that a element's href attribute to value.
    let active_range = selection
        .active_range()
        .expect("Must always have an active range");
    active_range.for_each_effectively_contained_child(|node| {
        for ancestor in node.inclusive_ancestors(ShadowIncluding::No) {
            if !ancestor.is_editable() {
                return;
            }
            let Some(anchor) = ancestor.downcast::<HTMLAnchorElement>() else {
                return;
            };
            if anchor
                .upcast::<Element>()
                .has_attribute(&local_name!("href"))
            {
                anchor.SetHref(cx, value.to_string().into());
            }
        }
    });
    // Step 3. Set the selection's value to value.
    selection.set_the_selection_value(cx, Some(value), CommandName::CreateLink, document);
    // Step 4. Return true.
    true
}
