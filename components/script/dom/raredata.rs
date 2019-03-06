/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::root::MutNullableDom;
use crate::dom::customelementregistry::{
    CustomElementDefinition, CustomElementReaction, CustomElementState,
};
use crate::dom::mutationobserver::RegisteredObserver;
use crate::dom::shadowroot::ShadowRoot;
use std::cell::Cell;
use std::rc::Rc;

#[derive(Default, JSTraceable, MallocSizeOf)]
#[must_root]
pub struct NodeRareData {
    /// The shadow root the node belongs to.
    /// This is None if the node is not in a shadow tree or
    /// if it is a ShadowRoot.
    pub owner_shadow_root: MutNullableDom<ShadowRoot>,
    /// Registered observers for this node.
    pub mutation_observers: DomRefCell<Vec<RegisteredObserver>>,
}

#[derive(Default, JSTraceable, MallocSizeOf)]
#[must_root]
pub struct ElementRareData {
    /// https://dom.spec.whatwg.org/#dom-element-shadowroot
    /// XXX This is currently not exposed to web content. Only for
    ///     internal use.
    pub shadow_root: MutNullableDom<ShadowRoot>,
    /// <https://html.spec.whatwg.org/multipage/#custom-element-reaction-queue>
    pub custom_element_reaction_queue: DomRefCell<Vec<CustomElementReaction>>,
    /// <https://dom.spec.whatwg.org/#concept-element-custom-element-definition>
    #[ignore_malloc_size_of = "Rc"]
    pub custom_element_definition: DomRefCell<Option<Rc<CustomElementDefinition>>>,
    /// <https://dom.spec.whatwg.org/#concept-element-custom-element-state>
    pub custom_element_state: Cell<CustomElementState>,
}
