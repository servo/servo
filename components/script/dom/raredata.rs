/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use euclid::default::Rect;
use servo_atoms::Atom;

use crate::dom::bindings::root::{Dom, MutNullableDom};
use crate::dom::customelementregistry::{
    CustomElementDefinition, CustomElementReaction, CustomElementState,
};
use crate::dom::elementinternals::ElementInternals;
use crate::dom::htmlslotelement::SlottableData;
use crate::dom::intersectionobserver::IntersectionObserverRegistration;
use crate::dom::mutationobserver::RegisteredObserver;
use crate::dom::node::UniqueId;
use crate::dom::nodelist::NodeList;
use crate::dom::range::WeakRangeVec;
use crate::dom::shadowroot::ShadowRoot;
use crate::dom::window::LayoutValue;

//XXX(ferjm) Ideally merge NodeRareData and ElementRareData so they share
//           storage.

#[derive(Default, JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) struct NodeRareData {
    /// The shadow root the node belongs to.
    /// This is None if the node is not in a shadow tree or
    /// if it is a ShadowRoot.
    pub(crate) containing_shadow_root: Option<Dom<ShadowRoot>>,
    /// Registered observers for this node.
    pub(crate) mutation_observers: Vec<RegisteredObserver>,
    /// Lazily-generated Unique Id for this node.
    pub(crate) unique_id: Option<UniqueId>,

    pub(crate) slottable_data: SlottableData,

    /// A vector of weak references to Range instances of which the start
    /// or end containers are this node. No range should ever be found
    /// twice in this vector, even if both the start and end containers
    /// are this node.
    pub(crate) ranges: WeakRangeVec,

    /// The live list of children return by .childNodes.
    pub(crate) child_list: MutNullableDom<NodeList>,
}

#[derive(Default, JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) struct ElementRareData {
    /// <https://dom.spec.whatwg.org/#dom-element-shadowroot>
    /// The ShadowRoot this element is host of.
    pub(crate) shadow_root: Option<Dom<ShadowRoot>>,
    /// <https://html.spec.whatwg.org/multipage/#custom-element-reaction-queue>
    pub(crate) custom_element_reaction_queue: Vec<CustomElementReaction>,
    /// <https://dom.spec.whatwg.org/#concept-element-custom-element-definition>
    #[ignore_malloc_size_of = "Rc"]
    pub(crate) custom_element_definition: Option<Rc<CustomElementDefinition>>,
    /// <https://dom.spec.whatwg.org/#concept-element-custom-element-state>
    pub(crate) custom_element_state: CustomElementState,
    /// The "name" content attribute; not used as frequently as id, but used
    /// in named getter loops so it's worth looking up quickly when present
    #[no_trace]
    pub(crate) name_attribute: Option<Atom>,
    /// The client rect reported by layout.
    #[no_trace]
    pub(crate) client_rect: Option<LayoutValue<Rect<i32>>>,
    /// <https://html.spec.whatwg.org/multipage#elementinternals>
    pub(crate) element_internals: Option<Dom<ElementInternals>>,

    /// <https://w3c.github.io/IntersectionObserver/#element-private-slots>
    /// > Element objects have an internal [[RegisteredIntersectionObservers]] slot,
    /// > which is initialized to an empty list. This list holds IntersectionObserverRegistration records, which have:
    pub(crate) registered_intersection_observers: Vec<IntersectionObserverRegistration>,
}
