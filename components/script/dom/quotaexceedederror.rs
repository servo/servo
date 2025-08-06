use crate::dom::{
    bindings::{
        error::Error,
        reflector::{reflect_dom_object, reflect_dom_object_with_proto},
    },
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
    fn new_inherited(
        message: DOMString,
        quota: Option<Finite<f64>>,
        requested: Option<Finite<f64>>,
    ) -> Self {
        Self {
            dom_exception: DOMException::new_inherited(
                message,
                DOMString::from_string("QuotaExceededError".to_string()),
            ),
            quota,
            requested,
        }
    }

    pub(crate) fn new(
        global: &GlobalScope,
        message: DOMString,
        quota: Option<Finite<f64>>,
        requested: Option<Finite<f64>>,
        can_gc: CanGc,
    ) -> DomRoot<Self> {
        reflect_dom_object(
            Box::new(Self::new_inherited(message, quota, requested)),
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
    ) -> Result<DomRoot<Self>, Error> {
        if let Some(quota) = options.quota {
            if *quota < 0.0 {
                return Err(Error::Range(
                    "quota must be at least zero if present".to_string(),
                ));
            }
        }
        if let Some(requested) = options.requested {
            if *requested < 0.0 {
                return Err(Error::Range(
                    "requested must be at least zero if present".to_string(),
                ));
            }
        }
        Ok(reflect_dom_object_with_proto(
            Box::new(QuotaExceededError::new_inherited(
                message,
                options.quota,
                options.requested,
            )),
            global,
            proto,
            can_gc,
        ))
    }

    fn GetQuota(&self) -> Option<Finite<f64>> {
        self.quota
    }

    fn GetRequested(&self) -> Option<Finite<f64>> {
        self.requested
    }
}
