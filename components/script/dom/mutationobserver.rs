use core::ptr::null;
use dom;
use dom::bindings::codegen::Bindings::MutationObserverBinding::MutationCallback;
use dom::bindings::codegen::Bindings::MutationObserverBinding::MutationObserverInit;
use dom::bindings::error::Error;
use dom::bindings::js::Root;
use dom::bindings::reflector::Reflector;
use dom::bindings::trace::JSTraceable;
use dom::mutationrecord::MutationRecord;
use dom::node::Node;
use dom::window::Window;
use dom_struct::dom_struct;
use script_thread::ScriptThread;
use std::ops::Deref;
use std::rc::Rc;

#[dom_struct]
pub struct MutationObserver {
    reflector_: Reflector,
    callback: MutationCallback,
}

impl MutationObserver {
    fn new(global: &Window, callback: MutationCallback) -> MutationObserver {
        MutationObserver {
            reflector_: Reflector::new(),
            callback: callback,
        }
    }

    pub fn Constructor(global: &Window, callback: Rc<MutationCallback>) -> Result<Root<MutationObserver>, Error> {
        let observer = MutationObserver::new(global, Rc::deref(&callback));
        ScriptThread::add_mutation_observer(&observer)
    }

}
