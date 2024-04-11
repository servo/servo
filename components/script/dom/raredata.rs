/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use euclid::default::Rect;
use servo_atoms::Atom;

use crate::dom::bindings::root::Dom;
use crate::dom::customelementregistry::{
    CustomElementDefinition, CustomElementReaction, CustomElementState,
};
use crate::dom::elementinternals::ElementInternals;
use crate::dom::mutationobserver::RegisteredObserver;
use crate::dom::node::UniqueId;
use crate::dom::shadowroot::ShadowRoot;
use crate::dom::window::LayoutValue;

//XXX(ferjm) Ideally merge NodeRareData and ElementRareData so they share
//           storage.

#[derive(Default, JSTraceable, MallocSizeOf)]
#[crown::unrooted_must_root_lint::must_root]
pub struct NodeRareData {
    /// The shadow root the node belongs to.
    /// This is None if the node is not in a shadow tree or
    /// if it is a ShadowRoot.
    pub containing_shadow_root: Option<Dom<ShadowRoot>>,
    /// Registered observers for this node.
    pub mutation_observers: Vec<RegisteredObserver>,
    /// Lazily-generated Unique Id for this node.
    pub unique_id: Option<UniqueId>,
}

#[derive(Default, JSTraceable, MallocSizeOf)]
#[crown::unrooted_must_root_lint::must_root]
pub struct ElementRareData {
    /// <https://dom.spec.whatwg.org/#dom-element-shadowroot>
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
    /// The "name" content attribute; not used as frequently as id, but used
    /// in named getter loops so it's worth looking up quickly when present
    #[no_trace]
    pub name_attribute: Option<Atom>,
    /// The client rect reported by layout.
    #[no_trace]
    pub client_rect: Option<LayoutValue<Rect<i32>>>,
    /// <https://html.spec.whatwg.org/multipage#elementinternals>
    pub element_internals: Option<Dom<ElementInternals>>,
}
