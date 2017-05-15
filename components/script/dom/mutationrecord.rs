/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use core::default::Default;
use core::ops::Deref;
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::MutationRecordBinding::MutationRecordBinding;
use dom::bindings::codegen::Bindings::MutationRecordBinding::MutationRecordBinding::MutationRecordMethods;
use dom::bindings::js::{JS, Root};
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bindings::str::DOMString;
use dom::node::{Node, window_from_node};
use dom::nodelist::NodeList;
use dom_struct::dom_struct;

#[dom_struct]
pub struct MutationRecord {
    reflector_: Reflector,

    //property for record type
    record_type: DOMString,

    //property for target node
    target: JS<Node>,

    //property for attribute name
    attribute_name: DOMRefCell<Option<DOMString>>,

    //property for attribute namespace
    attribute_namespace: DOMRefCell<Option<DOMString>>,

    //property for old value
    old_value: DOMRefCell<Option<DOMString>>,
}

impl MutationRecord {
    pub fn new(record_type: DOMString, target: &Node) -> Root<MutationRecord> {
        let boxed_record = box MutationRecord::new_inherited(record_type, target);
        return reflect_dom_object(boxed_record, window_from_node(target).deref(),
            MutationRecordBinding::Wrap);
    }

    fn new_inherited(record_type: DOMString, target: &Node) -> MutationRecord {
        MutationRecord {
            reflector_: Reflector::new(),
            record_type: record_type,
            target: JS::from_ref(target),
            attribute_name: Default::default(),
            attribute_namespace: Default::default(),
            old_value: Default::default(),
        }
    }

    // setter for attr_name
    pub fn SetAttributeName(&self, attr_name: DOMString) {
        *self.attribute_name.borrow_mut() = Some(attr_name);
    }
    // setter for attr_namespace
    pub fn SetAttributeNamespace(&self, attr_namespace: DOMString) {
        *self.attribute_namespace.borrow_mut() = Some(attr_namespace);
    }
    // setter for oldvalue
    pub fn SetoldValue(&self, attr_oldvalue: DOMString) {
        *self.old_value.borrow_mut() = Some(attr_oldvalue);
    }
}

impl MutationRecordMethods for MutationRecord {
    // https://dom.spec.whatwg.org/#dom-mutationrecord-type
    fn Type(&self) -> DOMString {
        self.record_type.clone()
    }

    // https://dom.spec.whatwg.org/#dom-mutationrecord-target
    fn Target(&self) -> Root<Node> {
        return Root::from_ref(&*self.target);
    }

    // https://dom.spec.whatwg.org/#dom-mutationrecord-attributename
    fn GetAttributeName(&self) -> Option<DOMString> {
        self.attribute_name.borrow().clone()
    }

    // https://dom.spec.whatwg.org/#dom-mutationrecord-attributenamespace
    fn GetAttributeNamespace(&self) -> Option<DOMString> {
        self.attribute_namespace.borrow().clone()
    }

    // https://dom.spec.whatwg.org/#dom-mutationrecord-oldvalue
    fn GetOldValue(&self) -> Option<DOMString> {
        self.old_value.borrow().clone()
    }

    // https://dom.spec.whatwg.org/#dom-mutationrecord-addednodes
    fn AddedNodes(&self) -> Root<NodeList> {
        let window = window_from_node(self.target.deref());
        NodeList::empty(&window)
    }

    // https://dom.spec.whatwg.org/#dom-mutationrecord-removednodes
    fn RemovedNodes(&self) -> Root<NodeList> {
        let window = window_from_node(self.target.deref());
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
