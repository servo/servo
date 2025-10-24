/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix};
use js::rust::HandleObject;

use crate::dom::bindings::codegen::Bindings::HTMLMapElementBinding::HTMLMapElementMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::document::Document;
use crate::dom::html::htmlareaelement::HTMLAreaElement;
use crate::dom::html::htmlcollection::HTMLCollection;
use crate::dom::html::htmlelement::HTMLElement;
use crate::dom::node::{Node, NodeTraits, ShadowIncluding};
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct HTMLMapElement {
    htmlelement: HTMLElement,
    /// <https://html.spec.whatwg.org/multipage/#dom-map-areas>
    areas: MutNullableDom<HTMLCollection>,
}

impl HTMLMapElement {
    fn new_inherited(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> HTMLMapElement {
        HTMLMapElement {
            htmlelement: HTMLElement::new_inherited(local_name, prefix, document),
            areas: Default::default(),
        }
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<HTMLMapElement> {
        Node::reflect_node_with_proto(
            Box::new(HTMLMapElement::new_inherited(local_name, prefix, document)),
            document,
            proto,
            can_gc,
        )
    }

    pub(crate) fn get_area_elements(&self) -> Vec<DomRoot<HTMLAreaElement>> {
        self.upcast::<Node>()
            .traverse_preorder(ShadowIncluding::No)
            .filter_map(DomRoot::downcast::<HTMLAreaElement>)
            .collect()
    }
}

impl HTMLMapElementMethods<crate::DomTypeHolder> for HTMLMapElement {
    // <https://html.spec.whatwg.org/multipage/#dom-map-name>
    make_getter!(Name, "name");

    // <https://html.spec.whatwg.org/multipage/#dom-map-name>
    make_atomic_setter!(SetName, "name");

    /// <https://html.spec.whatwg.org/multipage/#dom-map-areas>
    fn Areas(&self, can_gc: CanGc) -> DomRoot<HTMLCollection> {
        // The areas attribute must return an HTMLCollection rooted at the map element, whose filter
        // matches only area elements.
        self.areas.or_init(|| {
            HTMLCollection::new_with_filter_fn(
                &self.owner_window(),
                self.upcast(),
                |element, _| element.is::<HTMLAreaElement>(),
                can_gc,
            )
        })
    }
}
