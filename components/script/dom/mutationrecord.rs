/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::MutationRecordBinding::MutationRecordBinding;
use dom::bindings::codegen::Bindings::MutationRecordBinding::MutationRecordBinding::MutationRecordMethods;
use dom::bindings::js::{JS, Root};
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bindings::str::DOMString;
use dom::node::{Node, window_from_node};
use dom::nodelist::NodeList;
use dom_struct::dom_struct;
use html5ever::{LocalName, Namespace};

#[dom_struct]
pub struct MutationRecord {
    reflector_: Reflector,
    record_type: DOMString,
    target: JS<Node>,
    attribute_name: Option<DOMString>,
    attribute_namespace: Option<DOMString>,
    old_value: Option<DOMString>,
}

impl MutationRecord {
    #[allow(unrooted_must_root)]
    pub fn attribute_mutated(target: &Node,
                             attribute_name: &LocalName,
                             attribute_namespace: Option<&Namespace>,
                             old_value: Option<DOMString>) -> Root<MutationRecord> {
        let record = box MutationRecord::new_inherited("attributes",
                                                       target,
                                                       Some(DOMString::from(&**attribute_name)),
                                                       attribute_namespace.map(|n| DOMString::from(&**n)),
                                                       old_value);
        reflect_dom_object(record, &*window_from_node(target), MutationRecordBinding::Wrap)
    }

    fn new_inherited(record_type: &str,
                     target: &Node,
                     attribute_name: Option<DOMString>,
                     attribute_namespace: Option<DOMString>,
                     old_value: Option<DOMString>) -> MutationRecord {
        MutationRecord {
            reflector_: Reflector::new(),
            record_type: DOMString::from(record_type),
            target: JS::from_ref(target),
            attribute_name: attribute_name,
            attribute_namespace: attribute_namespace,
            old_value: old_value,
        }
    }
}

impl MutationRecordMethods for MutationRecord {
    // https://dom.spec.whatwg.org/#dom-mutationrecord-type
    fn Type(&self) -> DOMString {
        self.record_type.clone()
    }

    // https://dom.spec.whatwg.org/#dom-mutationrecord-target
    fn Target(&self) -> Root<Node> {
        Root::from_ref(&*self.target)
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
    fn AddedNodes(&self) -> Root<NodeList> {
        let window = window_from_node(&*self.target);
        NodeList::empty(&window)
    }

    // https://dom.spec.whatwg.org/#dom-mutationrecord-removednodes
    fn RemovedNodes(&self) -> Root<NodeList> {
        let window = window_from_node(&*self.target);
        NodeList::empty(&window)
    }

    // https://dom.spec.whatwg.org/#dom-mutationrecord-previoussibling
    fn GetPreviousSibling(&self) -> Option<Root<Node>> {
        None
    }

    // https://dom.spec.whatwg.org/#dom-mutationrecord-previoussibling
    fn GetNextSibling(&self) -> Option<Root<Node>> {
        None
    }

}
