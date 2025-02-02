/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use html5ever::{local_name, LocalName, Prefix};
use js::rust::HandleObject;

use crate::dom::activation::Activatable;
use crate::dom::bindings::codegen::Bindings::NodeBinding::Node_Binding::NodeMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::document::Document;
use crate::dom::element::Element;
use crate::dom::event::Event;
use crate::dom::eventtarget::EventTarget;
use crate::dom::htmldetailselement::HTMLDetailsElement;
use crate::dom::htmlelement::HTMLElement;
use crate::dom::node::Node;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct HTMLSummaryElement {
    htmlelement: HTMLElement,
}

impl HTMLSummaryElement {
    fn new_inherited(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> HTMLSummaryElement {
        HTMLSummaryElement {
            htmlelement: HTMLElement::new_inherited(local_name, prefix, document),
        }
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<HTMLSummaryElement> {
        Node::reflect_node_with_proto(
            Box::new(HTMLSummaryElement::new_inherited(
                local_name, prefix, document,
            )),
            document,
            proto,
            can_gc,
        )
    }

    /// <https://html.spec.whatwg.org/#summary-for-its-parent-details>
    fn is_a_summary_for_its_parent_details(&self) -> bool {
        // Step 1. If this summary element has no parent, then return false.
        // Step 2. Let parent be this summary element's parent.
        let Some(parent) = self.upcast::<Node>().GetParentNode() else {
            return false;
        };

        // Step 3. If parent is not a details element, then return false.
        if !parent.is::<HTMLDetailsElement>() {
            return false;
        };

        // Step 4. If parent's first summary element child is not this summary
        // element, then return false.
        // Step 5. Return true.
        parent
            .children()
            .find(|node| node.is::<HTMLSummaryElement>())
            .is_some_and(|node| Dom::from_ref(self.upcast::<Node>()) == node.as_traced())
    }
}

impl Activatable for HTMLSummaryElement {
    fn as_element(&self) -> &Element {
        self.upcast()
    }

    fn is_instance_activatable(&self) -> bool {
        true
    }

    // https://html.spec.whatwg.org/multipage/#run-post-click-activation-steps
    fn activation_behavior(&self, _event: &Event, _target: &EventTarget, can_gc: CanGc) {
        // Step 1. If this summary element is not the summary for its parent details, then return.
        if !self.is_a_summary_for_its_parent_details() {
            return;
        }

        // Step 2. Let parent be this summary element's parent.
        let parent = self
            .upcast::<Node>()
            .GetParentNode()
            .and_then(DomRoot::downcast::<Element>)
            .unwrap();

        // Step 3. If the open attribute is present on parent, then remove it.
        // Otherwise, set parent's open attribute to the empty string.
        let is_present = parent.has_attribute(&local_name!("open"));
        parent.set_bool_attribute(&local_name!("open"), !is_present, can_gc);
    }
}
