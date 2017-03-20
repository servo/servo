use core::ptr::null;
use dom_struct::dom_struct;
use dom::bindings::codegen::Bindings::MutationObserverBinding::MutationObserverBinding::MutationObserverMethods;
use dom::bindings::codegen::Bindings::MutationObserverBinding::MutationObserverInit;
use dom::bindings::js::Root;
use dom::bindings::reflector::Reflector;
use dom::mutationrecord::MutationRecord;
use dom::node::Node;

#[dom_struct]
pub struct MutationObserver {
  reflector_: Reflector,
}

impl MutationObserver {
}

impl MutationObserverMethods for MutationObserver {

  //void observe(Node target, optional MutationObserverInit options);
  fn Observe(&self, target: &Node, options: &MutationObserverInit) {
	// TODO implement
  }


  //void disconnect();
  fn Disconnect(&self) {
	// TODO implement
  }

  //sequence<MutationRecord> takeRecords();
  fn TakeRecords(&self) -> Vec<Root<MutationRecord>> {
      return vec![];
	//TODO implement
  }

}
