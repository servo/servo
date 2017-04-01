use core::ptr;
use dom_struct::dom_struct;
use dom::bindings::codegen::Bindings::MutationRecordBinding::MutationRecordBinding::MutationRecordMethods;
use dom::bindings::js::{JS, Root};
use dom::bindings::str::DOMString;
use dom::bindings::reflector::Reflector;
use dom::node::Node;
use dom::nodelist::NodeList;
use dom::window::Window;
use std::default::Default;

#[dom_struct]
pub struct MutationRecord {
    reflector_: Reflector,

    //property for record type
    record_type: DOMString,

    //property for target node
    target: JS<Node>,
}

impl MutationRecord {
    fn new(window: &Window, record_type: DOMString, target: JS<Node>) -> MutationRecord {
        MutationRecord {
            reflector_: Reflector::new(),
            record_type: record_type,
            target: target,
        }
    }
}

impl MutationRecordMethods for MutationRecord {
    fn Type(&self) -> DOMString {
        return self.record_type;
    }

    fn Target(&self) -> Root<Node> {
        return Root::from_ref(&*self.target);
    }

}
