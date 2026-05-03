/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use html5ever::local_name;
use js::context::JSContext;
use script_bindings::inheritance::Castable;

use crate::dom::bindings::root::DomRoot;
use crate::dom::element::Element;
use crate::dom::execcommand::basecommand::CommandName;
use crate::dom::html::htmlanchorelement::HTMLAnchorElement;
use crate::dom::html::htmlelement::HTMLElement;
use crate::dom::selection::Selection;

/// <https://w3c.github.io/editing/docs/execCommand/#the-unlink-command>
pub(crate) fn execute_unlink_command(cx: &mut JSContext, selection: &Selection) -> bool {
    // Step 1. Let hyperlinks be a list of every a element that has an href attribute
    // and is contained in the active range or is an ancestor of one of its boundary points.
    let active_range = selection
        .active_range()
        .expect("Must always have an active range");
    let mut hyperlinks = vec![];
    active_range.for_each_effectively_contained_child(|node| {
        if let Some(anchor) = node.downcast::<HTMLAnchorElement>() {
            if anchor
                .upcast::<Element>()
                .has_attribute(&local_name!("href"))
            {
                hyperlinks.push(DomRoot::from_ref(anchor));
            }
        }
    });
    // Step 2. Clear the value of each member of hyperlinks.
    for anchor in hyperlinks.iter() {
        anchor
            .upcast::<HTMLElement>()
            .clear_the_value(cx, &CommandName::Unlink);
    }
    // Step 3. Return true.
    true
}
