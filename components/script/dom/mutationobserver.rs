use core::ptr::null;
use dom;
use dom_struct::dom_struct;
use dom::bindings::codegen::Bindings::MutationObserverBinding::MutationCallback;
use dom::bindings::codegen::Bindings::MutationObserverBinding::MutationObserverBinding::MutationObserverMethods;
use dom::bindings::codegen::Bindings::MutationObserverBinding::MutationObserverInit;
use dom::bindings::error::Error;
use dom::bindings::js::Root;
use dom::bindings::reflector::Reflector;
use dom::bindings::trace::JSTraceable;
use dom::mutationrecord::MutationRecord;
use dom::node::Node;
use dom::window::Window;
use std::rc::Rc;

#[dom_struct]
pub struct MutationObserver {
  reflector_: Reflector,
  callback: MutationCallback,
}

impl MutationObserver {

   fn new(global: &dom::bindings::js::Root<dom::window::Window>, callback: MutationCallback) -> MutationObserver {
        MutationObserver {
            reflector_: Reflector::new(),
            callback: callback,
        }
    }

  pub fn Constructor(global: &dom::bindings::js::Root<dom::window::Window>, callback: Rc<MutationCallback>) -> Result<Root<MutationObserver>, Error> {
  	let observer = MutationObserver::new(global, &callback);
    ScriptThread.set_mutation_observer(observer)
  }

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
