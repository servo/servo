/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::gc::HandleObject;
use script_bindings::codegen::GenericBindings::QuotaExceededErrorBinding::{
    QuotaExceededErrorMethods, QuotaExceededErrorOptions,
};
use script_bindings::num::Finite;
use script_bindings::root::DomRoot;
use script_bindings::script_runtime::CanGc;
use script_bindings::str::DOMString;

use crate::dom::bindings::error::Error;
use crate::dom::bindings::reflector::{reflect_dom_object, reflect_dom_object_with_proto};
use crate::dom::types::{DOMException, GlobalScope};

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
    // https://webidl.spec.whatwg.org/#dom-quotaexceedederror-quotaexceedederror
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

    // https://webidl.spec.whatwg.org/#dom-quotaexceedederror-quota
    fn GetQuota(&self) -> Option<Finite<f64>> {
        self.quota
    }

    // https://webidl.spec.whatwg.org/#dom-quotaexceedederror-requested
    fn GetRequested(&self) -> Option<Finite<f64>> {
        self.requested
    }
}
