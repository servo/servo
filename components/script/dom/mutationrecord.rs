/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::MutationRecordBinding::MutationRecordBinding::MutationRecordMethods;
use dom::bindings::js::{JS, Root};
use dom::bindings::reflector::Reflector;
use dom::bindings::str::DOMString;
use dom::node::Node;
use dom_struct::dom_struct;

#[dom_struct]
pub struct MutationRecord {
    reflector_: Reflector,

    //property for record type
    record_type: DOMString,

    //property for target node
    target: JS<Node>,
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

}
