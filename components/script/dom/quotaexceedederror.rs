use crate::dom::{
    bindings::reflector::reflect_dom_object,
    domexception::DOMErrorName,
    types::{DOMException, GlobalScope},
};

use dom_struct::dom_struct;
use js::gc::HandleObject;
use script_bindings::{
    codegen::GenericBindings::QuotaExceededErrorBinding::{
        QuotaExceededErrorMethods, QuotaExceededErrorOptions,
    },
    num::Finite,
    root::DomRoot,
    script_runtime::CanGc,
    str::DOMString,
};

#[dom_struct]
pub(crate) struct QuotaExceededError {
    dom_exception: DOMException,
    quota: Option<Finite<f64>>,
    requested: Option<Finite<f64>>,
}

impl QuotaExceededError {
    fn new_inherited(quota: Option<Finite<f64>>, requested: Option<Finite<f64>>) -> Self {
        let (exception_msg, exception_name) =
            DOMException::get_error_data_by_code(DOMErrorName::QuotaExceededError);
        Self {
            dom_exception: DOMException::new_inherited(exception_msg, exception_name),
            quota,
            requested,
        }
    }

    pub(crate) fn new(
        global: &GlobalScope,
        quota: Option<Finite<f64>>,
        requested: Option<Finite<f64>>,
        can_gc: CanGc,
    ) -> DomRoot<Self> {
        reflect_dom_object(
            Box::new(Self::new_inherited(quota, requested)),
            global,
            can_gc,
        )
    }
}

impl QuotaExceededErrorMethods<crate::DomTypeHolder> for QuotaExceededError {
    fn Constructor(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        message: DOMString,
        options: &QuotaExceededErrorOptions,
    ) -> DomRoot<Self> {
        todo!()
    }

    fn GetQuota(&self) -> Option<Finite<f64>> {
        self.quota
    }

    fn GetRequested(&self) -> Option<Finite<f64>> {
        self.requested
    }
}
