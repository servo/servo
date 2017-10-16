/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::MutationRecordBinding::MutationRecordBinding;
use dom::bindings::codegen::Bindings::MutationRecordBinding::MutationRecordBinding::MutationRecordMethods;
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bindings::root::{Dom, DomRoot, MutNullableDom};
use dom::bindings::str::DOMString;
use dom::node::{Node, window_from_node};
use dom::nodelist::NodeList;
use dom_struct::dom_struct;
use html5ever::{LocalName, Namespace};

#[dom_struct]
pub struct MutationRecord {
    reflector_: Reflector,
    record_type: DOMString,
    target: Dom<Node>,
    attribute_name: Option<DOMString>,
    attribute_namespace: Option<DOMString>,
    old_value: Option<DOMString>,
    added_nodes: MutNullableDom<NodeList>,
    removed_nodes: MutNullableDom<NodeList>,
    next_sibling: Option<Dom<Node>>,
    prev_sibling: Option<Dom<Node>>,
}

impl MutationRecord {
    #[allow(unrooted_must_root)]
    pub fn attribute_mutated(target: &Node,
                             attribute_name: &LocalName,
                             attribute_namespace: Option<&Namespace>,
                             old_value: Option<DOMString>) -> DomRoot<MutationRecord> {
        let record = Box::new(MutationRecord::new_inherited(
            "attributes",
            target,
            Some(DOMString::from(&**attribute_name)),
            attribute_namespace.map(|n| DOMString::from(&**n)),
            old_value,
            None, None, None, None
        ));
        reflect_dom_object(record, &*window_from_node(target), MutationRecordBinding::Wrap)
    }

    pub fn child_list_mutated(target: &Node,
                              added_nodes: Option<&[&Node]>,
                              removed_nodes: Option<&[&Node]>,
                              next_sibling: Option<&Node>,
                              prev_sibling: Option<&Node>) -> DomRoot<MutationRecord> {
        let window = window_from_node(target);
        let added_nodes = added_nodes.map(|list| NodeList::new_simple_list_slice(&window, list));
        let removed_nodes = removed_nodes.map(|list| NodeList::new_simple_list_slice(&window, list));

        reflect_dom_object(
            Box::new(MutationRecord::new_inherited(
                "childList",
                target,
                None, None, None,
                added_nodes.as_ref().map(|list| &**list),
                removed_nodes.as_ref().map(|list| &**list),
                next_sibling,
                prev_sibling
            )),
            &*window,
            MutationRecordBinding::Wrap
        )
    }

    fn new_inherited(record_type: &str,
                     target: &Node,
                     attribute_name: Option<DOMString>,
                     attribute_namespace: Option<DOMString>,
                     old_value: Option<DOMString>,
                     added_nodes: Option<&NodeList>,
                     removed_nodes: Option<&NodeList>,
                     next_sibling: Option<&Node>,
                     prev_sibling: Option<&Node>) -> MutationRecord {
        MutationRecord {
            reflector_: Reflector::new(),
            record_type: DOMString::from(record_type),
            target: Dom::from_ref(target),
            attribute_name: attribute_name,
            attribute_namespace: attribute_namespace,
            old_value: old_value,
            added_nodes: MutNullableDom::new(added_nodes),
            removed_nodes: MutNullableDom::new(removed_nodes),
            next_sibling: next_sibling.map(Dom::from_ref),
            prev_sibling: prev_sibling.map(Dom::from_ref),
        }
    }
}

impl MutationRecordMethods for MutationRecord {
    // https://dom.spec.whatwg.org/#dom-mutationrecord-type
    fn Type(&self) -> DOMString {
        self.record_type.clone()
    }

    // https://dom.spec.whatwg.org/#dom-mutationrecord-target
    fn Target(&self) -> DomRoot<Node> {
        DomRoot::from_ref(&*self.target)
    }

    // https://dom.spec.whatwg.org/#dom-mutationrecord-attributename
    fn GetAttributeName(&self) -> Option<DOMString> {
        self.attribute_name.clone()
    }

    // https://dom.spec.whatwg.org/#dom-mutationrecord-attributenamespace
    fn GetAttributeNamespace(&self) -> Option<DOMString> {
        self.attribute_namespace.clone()
    }

    // https://dom.spec.whatwg.org/#dom-mutationrecord-oldvalue
    fn GetOldValue(&self) -> Option<DOMString> {
        self.old_value.clone()
    }

    // https://dom.spec.whatwg.org/#dom-mutationrecord-addednodes
    fn AddedNodes(&self) -> DomRoot<NodeList> {
        self.added_nodes.or_init(|| {
            let window = window_from_node(&*self.target);
            NodeList::empty(&window)
        })
    }

    // https://dom.spec.whatwg.org/#dom-mutationrecord-removednodes
    fn RemovedNodes(&self) -> DomRoot<NodeList> {
        self.removed_nodes.or_init(|| {
            let window = window_from_node(&*self.target);
            NodeList::empty(&window)
        })
    }

    // https://dom.spec.whatwg.org/#dom-mutationrecord-previoussibling
    fn GetPreviousSibling(&self) -> Option<DomRoot<Node>> {
        self.prev_sibling.as_ref().map(|node| DomRoot::from_ref(&**node))
    }

    // https://dom.spec.whatwg.org/#dom-mutationrecord-previoussibling
    fn GetNextSibling(&self) -> Option<DomRoot<Node>> {
        self.next_sibling.as_ref().map(|node| DomRoot::from_ref(&**node))
    }

}
