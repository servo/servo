use dom_struct::dom_struct;
use js::context::JSContext;
use script_bindings::codegen::GenericBindings::WebLockBinding::{LockMethods, LockMode};
use script_bindings::reflector::{Reflector, reflect_dom_object_with_cx};
use script_bindings::root::DomRoot;
use script_bindings::str::DOMString;
use storage_traits::weblocks::{LockId, LockMsg};

use crate::conversions::Convert;
use crate::dom::GlobalScope;

#[dom_struct]
pub(crate) struct Lock {
    reflector_: Reflector,
    #[no_trace]
    id: LockId,
    name: DOMString,
    mode: LockMode,
}

impl Lock {
    pub(crate) fn new_inherited(msg: LockMsg) -> Self {
        let id = msg.id;
        let name = msg.name.into();
        let mode = msg.mode.convert();
        Lock {
            reflector_: Reflector::new(),
            id,
            name,
            mode,
        }
    }

    pub(crate) fn new(cx: &mut JSContext, global: &GlobalScope, msg: LockMsg) -> DomRoot<Self> {
        reflect_dom_object_with_cx(Box::new(Self::new_inherited(msg)), global, cx)
    }
}

impl LockMethods<crate::DomTypeHolder> for Lock {
    fn Name(&self) -> DOMString {
        self.name.clone()
    }

    fn Mode(&self) -> LockMode {
        self.mode
    }
}
