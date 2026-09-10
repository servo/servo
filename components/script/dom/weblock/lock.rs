use dom_struct::dom_struct;
use script_bindings::codegen::GenericBindings::WebLockBinding::{LockMethods, LockMode};
use script_bindings::reflector::Reflector;
use script_bindings::str::DOMString;

#[dom_struct]
pub(crate) struct Lock {
    reflector_: Reflector,
}

impl LockMethods<crate::DomTypeHolder> for Lock {
    fn Name(&self) -> DOMString {
        todo!()
    }

    fn Mode(&self) -> LockMode {
        todo!()
    }
}
