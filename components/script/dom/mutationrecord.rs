/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use core::default::Default;
use dom::bindings::codegen::Bindings::MutationRecordBinding::MutationRecordBinding::MutationRecordMethods;
use dom::bindings::error::Fallible;
use dom::bindings::js::{JS, Root, MutNullableJS};
use dom::bindings::reflector::Reflector;
use dom::bindings::str::DOMString;
use dom::node::Node;
use dom::nodelist::NodeList;
use dom::window::Window;
use dom_struct::dom_struct;

#[dom_struct]
pub struct MutationRecord {
    reflector_: Reflector,

    //property for record type
    record_type: DOMString,

    //property for target node
    target: JS<Node>,

    //property for attribute name
    attribute_name: Option<DOMString>,

    //property for attribute namespace
    attribute_namespace: Option<DOMString>,

    //property for old value
    old_value: Option<DOMString>,
}

impl MutationRecord {
     pub fn new(record_type: DOMString, target: &Node) -> MutationRecord {
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
     pub fn SetAttributeName(&mut self, attr_name: DOMString) {
         self.attribute_name = Some(attr_name);
     }
     // setter for attr_namespace
     pub fn SetAttributeNamespace(&mut self, attr_namespace: DOMString) {
         self.attribute_namespace = Some(attr_namespace);
     }
     // setter for attr_oldvalue
     pub fn SetoldValue(&mut self, attr_oldvalue: DOMString) {
         self.old_value = Some(attr_oldvalue);
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
}
