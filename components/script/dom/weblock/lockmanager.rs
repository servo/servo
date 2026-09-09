use dom_struct::dom_struct;
use script_bindings::{
    callback::RootedCallback,
    codegen::GenericBindings::WebLockBinding::{
        LockGrantedCallback, LockManagerMethods, LockOptions,
    },
    interfaces::PromiseHelpers,
    reflector::Reflector,
    str::DOMString,
};

#[dom_struct]
pub(crate) struct LockManager {
    reflector_: Reflector,
}

impl LockManagerMethods<crate::DomTypeHolder> for LockManager {
    fn Request(
        &self,
        name: DOMString,
        callback: RootedCallback<LockGrantedCallback<crate::DomTypeHolder>>,
    ) -> <<crate::DomTypeHolder as script_bindings::DomTypes>::Promise as PromiseHelpers<
        crate::DomTypeHolder,
    >>::StackRoot {
        todo!()
    }

    fn Request_(
        &self,
        name: DOMString,
        options: &LockOptions<crate::DomTypeHolder>,
        callback: RootedCallback<LockGrantedCallback<crate::DomTypeHolder>>,
    ) -> <<crate::DomTypeHolder as script_bindings::DomTypes>::Promise as PromiseHelpers<
        crate::DomTypeHolder,
    >>::StackRoot {
        todo!()
    }

    fn Query(
        &self,
    ) -> <<crate::DomTypeHolder as script_bindings::DomTypes>::Promise as PromiseHelpers<
        crate::DomTypeHolder,
    >>::StackRoot {
        todo!()
    }
}
