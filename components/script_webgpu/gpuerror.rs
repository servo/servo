/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::marker::PhantomData;

use dom_struct::dom_struct;
use js::context::JSContext;
use js::rust::HandleObject;
use malloc_size_of_derive::MallocSizeOf;
use script_bindings::DomTypes;
use script_bindings::codegen::GenericBindings::WebGPUBinding::{
    GPUErrorFilter, GPUErrorMethods, GPUErrorWrap,
};
use script_bindings::reflector::{Reflector, reflect_dom_object_with_proto_and_wrap};
use webgpu_traits::{Error, ErrorFilter};

use crate::JSTraceable;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::gpuconvert::WebGPUConvert;
use crate::gpuinternalerror::GPUInternalError;
use crate::gpuoutofmemoryerror::GPUOutOfMemoryError;
use crate::gpuvalidationerror::GPUValidationError;
use crate::traits::Equivalence;

#[dom_struct]
pub struct GPUError<D: DomTypes> {
    reflector_: Reflector,
    message: DOMString,
    #[no_trace = "PhantomData does not exist"]
    phantom: PhantomData<D>,
}

impl<D> GPUError<D>
where
    D: Equivalence,
{
    pub(crate) fn new_inherited(message: DOMString) -> Self {
        Self {
            reflector_: Reflector::new(),
            message,
            phantom: PhantomData,
        }
    }

    #[expect(dead_code)]
    pub(crate) fn new(
        cx: &mut JSContext,
        global: &D::GlobalScope,
        message: DOMString,
    ) -> DomRoot<Self> {
        Self::new_with_proto(cx, global, None, message)
    }

    pub(crate) fn new_with_proto(
        cx: &mut JSContext,
        global: &D::GlobalScope,
        proto: Option<HandleObject>,
        message: DOMString,
    ) -> DomRoot<Self> {
        reflect_dom_object_with_proto_and_wrap::<D, _, _>(
            Box::new(GPUError::new_inherited(message)),
            global,
            proto,
            cx,
            GPUErrorWrap::<D>,
        )
    }

    pub fn from_error(cx: &mut JSContext, global: &D::GlobalScope, error: Error) -> DomRoot<Self> {
        match error {
            Error::Validation(msg) => DomRoot::upcast(GPUValidationError::new_with_proto(
                cx,
                global,
                None,
                msg.into(),
            )),
            Error::OutOfMemory(msg) => DomRoot::upcast(GPUOutOfMemoryError::new_with_proto(
                cx,
                global,
                None,
                msg.into(),
            )),
            Error::Internal(msg) => DomRoot::upcast(GPUInternalError::new_with_proto(
                cx,
                global,
                None,
                msg.into(),
            )),
        }
    }
}

impl<D: Equivalence> GPUErrorMethods<D> for GPUError<D> {
    /// <https://gpuweb.github.io/gpuweb/#dom-gpuerror-message>
    fn Message(&self) -> DOMString {
        self.message.clone()
    }
}

impl WebGPUConvert<GPUErrorFilter> for ErrorFilter {
    fn convert(self) -> GPUErrorFilter {
        match self {
            ErrorFilter::Validation => GPUErrorFilter::Validation,
            ErrorFilter::OutOfMemory => GPUErrorFilter::Out_of_memory,
            ErrorFilter::Internal => GPUErrorFilter::Internal,
        }
    }
}

pub trait AsWebGpu {
    fn as_webgpu(&self) -> ErrorFilter;
}

impl AsWebGpu for GPUErrorFilter {
    fn as_webgpu(&self) -> ErrorFilter {
        match self {
            GPUErrorFilter::Validation => ErrorFilter::Validation,
            GPUErrorFilter::Out_of_memory => ErrorFilter::OutOfMemory,
            GPUErrorFilter::Internal => ErrorFilter::Internal,
        }
    }
}
