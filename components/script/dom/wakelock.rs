use std::rc::Rc;

use dom_struct::dom_struct;

use crate::dom::bindings::codegen::Bindings::WakeLockBinding::{WakeLockMethods, WakeLockType};
use crate::dom::bindings::reflector::{Reflector, reflect_dom_object};
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::script_runtime::CanGc;

/// <https://w3c.github.io/screen-wake-lock/#the-wakelock-interface>
#[dom_struct]
pub(crate) struct WakeLock {
    reflector_: Reflector,
}

impl WakeLock {
    pub(crate) fn new_inherited() -> Self {
        Self {
            reflector_: Reflector::new(),
        }
    }

    pub(crate) fn new(global: &GlobalScope, can_gc: CanGc) -> DomRoot<Self> {
        reflect_dom_object(Box::new(Self::new_inherited()), global, can_gc)
    }
}

impl WakeLockMethods<crate::DomTypeHolder> for WakeLock {
    /// <https://w3c.github.io/screen-wake-lock/#the-request-method>
    fn Request(&self, _type_: WakeLockType) -> Rc<Promise> {
        todo!()
    }
}
