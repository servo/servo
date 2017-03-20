use core::ptr::null;
use dom_struct::dom_struct;
use dom::bindings::codegen::Bindings::MutationRecordBinding::MutationRecordBinding::MutationRecordMethods;
use dom::bindings::js::{JS, Root};
use dom::bindings::str::DOMString;
use dom::bindings::reflector::Reflector;
use dom::node::Node;
use dom::nodelist::NodeList;
use dom::window::Window;

#[dom_struct]
pub struct MutationRecord {
  reflector_: Reflector,

//  readonly attribute DOMString record_type;
  record_type: DOMString,

//  [SameObject]
//  readonly attribute Node target;
  target: Root<Node>,

//  [SameObject]
//  readonly attribute NodeList addedNodes;
  addedNodes: Root<NodeList>,

//  [SameObject]
//  readonly attribute NodeList removedNodes;
  removedNodes: Root<NodeList>,

//  readonly attribute Node? previousSibling;
  previousSibling: Root<Node>,

//  readonly attribute Node? nextSibling;
  nextSibling: Root<Node>,

//  readonly attribute DOMString? attributeName;
  attributeName: DOMString,

//  readonly attribute DOMString? attributeNamespace;
  attributeNamespace: DOMString,

//  readonly attribute DOMString? oldValue;
  oldValue: DOMString,

}

impl MutationRecord {
    fn new(window: &Window, record_type: DOMString, target: Root<Node>) -> Root<MutationRecord> {
        MutationRecord {
            reflector_: Reflector::new(),
            record_type: record_type,
            target: target,
            addedNodes: NodeList::empty(window),
            removedNodes: NodeList::empty(window),
            previousSibling: None,
            nextSibling: None,
            attributeName: None,
            attributeNamespace: None,
            oldValue: None,
        }
    }
}

impl MutationRecordMethods for MutationRecord {

  fn Type(&self) -> DOMString {
      return self.record_type;
      //return "characterData";
      //return "childList";
  }

  fn Target(&self) -> Root<Node> {
      return self.target;
  }

  fn AddedNodes(&self) -> Root<NodeList> {
      return self.addedNodes;
  }

  fn RemovedNodes(&self) -> Root<NodeList> {
      return self.removedNodes;
  }

  fn GetPreviousSibling(&self) -> Option<Root<Node>> {
      if self.previousSibling.is_null() {
          return None;
      } else {
          return Some(self.previousSibling);
      }
  }

  fn GetNextSibling(&self) -> Option<Root<Node>> {
      if self.nextSibling.is_null() {
          return None;
      } else {
          return Some(self.nextSibling);
      }
  }

  fn GetAttributeName(&self) -> Option<DOMString> {
      if self.attributeName.is_null() {
          return None;
      } else {
          return Some(self.attributeName);
      }
  }

  fn GetAttributeNamespace(&self) -> Option<DOMString> {
      if self.attributeNamespace.is_null() {
          return None;
      } else {
          return Some(self.attributeNamespace);
      }
  }

  fn GetOldValue(&self) -> Option<DOMString> {
      if self.oldValue.is_null() {
          return None;
      } else {
          return Some(self.oldValue);
      }
  }

}
