/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::JSContext;
use js::rust::HandleObject;
use malloc_size_of_derive::MallocSizeOf;
use script_bindings::DomTypes;
use script_bindings::codegen::GenericBindings::WebGPUBinding::{
    GPUPipelineErrorInit, GPUPipelineErrorMethods, GPUPipelineErrorReason, GPUPipelineErrorWrap,
};
use script_bindings::conversions::DerivedFrom;
use script_bindings::inheritance::Castable;
use script_bindings::reflector::reflect_dom_object_with_proto_and_wrap;

use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::gpuerror::GPUError;
use crate::traits::{Equivalence, WebGPUDomExceptionTrait};
use crate::{DomObject, JSTraceable};

/// <https://gpuweb.github.io/gpuweb/#gpupipelineerror>
#[dom_struct(special)]
pub struct GPUPipelineError<D: DomTypes> {
    exception: D::DOMException,
    reason: GPUPipelineErrorReason,
}

impl<D> GPUPipelineError<D>
where
    D: Equivalence,
    D::GPUError: Castable,
    D::GPUValidationError: DerivedFrom<GPUError<D>>,
    D::GPUOutOfMemoryError: DerivedFrom<GPUError<D>>,
    D::GPUInternalError: DerivedFrom<GPUError<D>>,
    D::DOMException: WebGPUDomExceptionTrait,
{
    fn new_inherited(message: DOMString, reason: GPUPipelineErrorReason) -> Self {
        Self {
            exception: D::DOMException::new_inherited(
                message,
                DOMString::from_static("GPUPipelineError"),
            ),
            reason,
        }
    }

    pub(crate) fn new_with_proto(
        cx: &mut JSContext,
        global: &D::GlobalScope,
        proto: Option<HandleObject>,
        message: DOMString,
        reason: GPUPipelineErrorReason,
    ) -> DomRoot<Self> {
        reflect_dom_object_with_proto_and_wrap::<D, _, _>(
            Box::new(Self::new_inherited(message, reason)),
            global,
            proto,
            cx,
            GPUPipelineErrorWrap::<D>,
        )
    }

    pub fn new(
        cx: &mut JSContext,
        global: &D::GlobalScope,
        message: DOMString,
        reason: GPUPipelineErrorReason,
    ) -> DomRoot<Self> {
        Self::new_with_proto(cx, global, None, message, reason)
    }
}

impl<D> GPUPipelineErrorMethods<D> for GPUPipelineError<D>
where
    D: Equivalence,
    D::GPUError: Castable,
    D::GPUValidationError: DerivedFrom<GPUError<D>>,
    D::GPUOutOfMemoryError: DerivedFrom<GPUError<D>>,
    D::GPUInternalError: DerivedFrom<GPUError<D>>,
    D::DOMException: WebGPUDomExceptionTrait,
{
    /// <https://gpuweb.github.io/gpuweb/#dom-gpupipelineerror-constructor>
    fn Constructor(
        cx: &mut JSContext,
        global: &D::GlobalScope,
        proto: Option<HandleObject>,
        message: DOMString,
        options: &GPUPipelineErrorInit,
    ) -> DomRoot<Self> {
        Self::new_with_proto(cx, global, proto, message, options.reason)
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpupipelineerror-reason>
    fn Reason(&self) -> GPUPipelineErrorReason {
        self.reason
    }
}
