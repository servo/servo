/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::root::Dom;
use crate::dom::customelementregistry::{
    CustomElementDefinition, CustomElementReaction, CustomElementState,
};
use crate::dom::mutationobserver::RegisteredObserver;
use crate::dom::shadowroot::ShadowRoot;
use std::rc::Rc;

//XXX(ferjm) Ideally merge NodeRareData and ElementRareData so they share
//           storage.

#[derive(Default, JSTraceable, MallocSizeOf)]
#[must_root]
pub struct NodeRareData {
    /// The shadow root the node belongs to.
    /// This is None if the node is not in a shadow tree or
    /// if it is a ShadowRoot.
    pub containing_shadow_root: Option<Dom<ShadowRoot>>,
    /// Registered observers for this node.
    pub mutation_observers: Vec<RegisteredObserver>,
}

#[derive(Default, JSTraceable, MallocSizeOf)]
#[must_root]
pub struct ElementRareData {
    /// https://dom.spec.whatwg.org/#dom-element-shadowroot
    /// The ShadowRoot this element is host of.
    /// XXX This is currently not exposed to web content. Only for
    ///     internal use.
    pub shadow_root: Option<Dom<ShadowRoot>>,
    /// <https://html.spec.whatwg.org/multipage/#custom-element-reaction-queue>
    pub custom_element_reaction_queue: Vec<CustomElementReaction>,
    /// <https://dom.spec.whatwg.org/#concept-element-custom-element-definition>
    #[ignore_malloc_size_of = "Rc"]
    pub custom_element_definition: Option<Rc<CustomElementDefinition>>,
    /// <https://dom.spec.whatwg.org/#concept-element-custom-element-state>
    pub custom_element_state: CustomElementState,
}
