/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::JSContext;
use js::rust::HandleObject;
use malloc_size_of_derive::MallocSizeOf;
use script_bindings::DomTypes;
use script_bindings::codegen::GenericBindings::WebGPUBinding::{
    GPUOutOfMemoryErrorMethods, GPUOutOfMemoryErrorWrap,
};
use script_bindings::conversions::DerivedFrom;
use script_bindings::inheritance::Castable;
use script_bindings::reflector::reflect_dom_object_with_proto_and_wrap;

use crate::JSTraceable;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::gpuerror::GPUError;
use crate::traits::Equivalence;

#[dom_struct]
pub struct GPUOutOfMemoryError<D: DomTypes> {
    gpu_error: GPUError<D>,
}

impl<D> GPUOutOfMemoryError<D>
where
    D: Equivalence,
    D::GPUError: Castable,
    D::GPUValidationError: DerivedFrom<GPUError<D>>,
    D::GPUOutOfMemoryError: DerivedFrom<GPUError<D>>,
    D::GPUInternalError: DerivedFrom<GPUError<D>>,
{
    fn new_inherited(message: DOMString) -> Self {
        Self {
            gpu_error: GPUError::new_inherited(message),
        }
    }

    pub(crate) fn new_with_proto(
        cx: &mut JSContext,
        global: &D::GlobalScope,
        proto: Option<HandleObject>,
        message: DOMString,
    ) -> DomRoot<Self> {
        reflect_dom_object_with_proto_and_wrap::<D, _, _>(
            Box::new(Self::new_inherited(message)),
            global,
            proto,
            cx,
            GPUOutOfMemoryErrorWrap::<D>,
        )
    }
}

impl<D> GPUOutOfMemoryErrorMethods<D> for GPUOutOfMemoryError<D>
where
    D: Equivalence,
    D::GPUError: Castable,
    D::GPUValidationError: DerivedFrom<GPUError<D>>,
    D::GPUOutOfMemoryError: DerivedFrom<GPUError<D>>,
    D::GPUInternalError: DerivedFrom<GPUError<D>>,
{
    /// <https://gpuweb.github.io/gpuweb/#dom-GPUOutOfMemoryError-GPUOutOfMemoryError>
    fn Constructor(
        cx: &mut JSContext,
        global: &D::GlobalScope,
        proto: Option<HandleObject>,
        message: DOMString,
    ) -> DomRoot<Self> {
        Self::new_with_proto(cx, global, proto, message)
    }
}
