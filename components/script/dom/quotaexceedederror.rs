use crate::dom::types::{DOMException, GlobalScope};

use dom_struct::dom_struct;
use js::gc::HandleObject;
use script_bindings::{
    codegen::GenericBindings::QuotaExceededErrorBinding::{
        QuotaExceededErrorMethods, QuotaExceededErrorOptions,
    },
    root::DomRoot,
    script_runtime::CanGc,
    str::DOMString,
};

#[dom_struct]
pub(crate) struct QuotaExceededError {
    dom_exception: DOMException,
}

impl QuotaExceededErrorMethods<crate::DomTypeHolder> for QuotaExceededError {
    fn Constructor(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        message: DOMString,
        options: &QuotaExceededErrorOptions,
    ) -> DomRoot<QuotaExceededError> {
        todo!()
    }

    fn GetQuota(&self) -> Option<script_bindings::num::Finite<f64>> {
        todo!()
    }

    fn GetRequested(&self) -> Option<script_bindings::num::Finite<f64>> {
        todo!()
    }
}
