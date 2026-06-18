/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use dom_struct::dom_struct;
use js::context::JSContext;
use js::conversions::ToJSValConvertible;
use js::rust::MutableHandleValue;
use script_bindings::cell::DomRefCell;
use script_bindings::reflector::{Reflector, reflect_dom_object_with_cx};
use script_bindings::root::DomRoot;

use crate::dom::bindings::codegen::Bindings::WebNNBinding::{MLOperandDataType, MLTensorMethods};
use crate::dom::bindings::weakref::WeakRef;
use crate::dom::globalscope::GlobalScope;
use crate::dom::webnn::mlcontext::MLContext;

/// <https://www.w3.org/TR/webnn/#mltensor>
#[dom_struct]
pub(crate) struct MLTensor {
    reflector_: Reflector,
    /// <https://www.w3.org/TR/webnn/#dom-mlgraphbuilder-context-slot>
    context: WeakRef<MLContext>,
    /// <https://www.w3.org/TR/webnn/#dom-mloperanddescriptor-datatype>
    data_type: MLOperandDataType,
    /// <https://www.w3.org/TR/webnn/#dom-mloperanddescriptor-shape>
    shape: Vec<u32>,
    /// <https://www.w3.org/TR/webnn/#dom-mltensordescriptor-readable>
    readable: bool,
    /// <https://www.w3.org/TR/webnn/#dom-mltensordescriptor-writable>
    writable: bool,
    /// <https://www.w3.org/TR/webnn/#dom-mltensor-isconstant-slot>
    is_constant: bool,
    /// <https://www.w3.org/TR/webnn/#dom-mltensor-isdestroyed-slot>
    is_destroyed: Cell<bool>,
    /// <https://www.w3.org/TR/webnn/#dom-mltensor-data-slot>
    data: DomRefCell<Option<Vec<u8>>>,
}

impl MLTensor {
    pub(crate) fn new_inherited(
        context: &MLContext,
        data_type: MLOperandDataType,
        shape: Vec<u32>,
        readable: bool,
        writable: bool,
        is_constant: bool,
    ) -> MLTensor {
        MLTensor {
            reflector_: Reflector::new(),
            context: WeakRef::new(context),
            data_type,
            shape,
            readable,
            writable,
            is_constant,
            is_destroyed: Cell::new(false),
            data: DomRefCell::new(None),
        }
    }

    pub(crate) fn new(
        global: &GlobalScope,
        context: &MLContext,
        data_type: MLOperandDataType,
        shape: Vec<u32>,
        readable: bool,
        writable: bool,
        is_constant: bool,
        cx: &mut JSContext,
    ) -> DomRoot<MLTensor> {
        reflect_dom_object_with_cx(
            Box::new(MLTensor::new_inherited(
                context,
                data_type,
                shape,
                readable,
                writable,
                is_constant,
            )),
            global,
            cx,
        )
    }

    // Write data from a byte slice into the tensor's buffer.
    pub(crate) fn write_data(&self, src: &[u8]) {
        if let Some(ref mut buf) = *self.data.borrow_mut() {
            let len = buf.len().min(src.len());
            buf[..len].copy_from_slice(&src[..len]);
        }
    }

    pub(crate) fn read_data(&self) -> Option<Vec<u8>> {
        self.data.borrow().clone()
    }

    pub(crate) fn data_type(&self) -> MLOperandDataType {
        self.data_type
    }

    pub(crate) fn shape(&self) -> &[u32] {
        &self.shape
    }

    pub(crate) fn context(&self) -> &WeakRef<MLContext> {
        &self.context
    }

    /// <https://www.w3.org/TR/webnn/#dom-mltensor-isdestroyed-slot>
    pub(crate) fn is_destroyed(&self) -> bool {
        self.is_destroyed.get()
    }

    /// <https://www.w3.org/TR/webnn/#dom-mltensor-isconstant-slot>
    pub(crate) fn is_constant(&self) -> bool {
        self.is_constant
    }

    /// <https://www.w3.org/TR/webnn/#dom-mltensordescriptor-readable>
    pub(crate) fn readable(&self) -> bool {
        self.readable
    }

    /// <https://www.w3.org/TR/webnn/#dom-mltensordescriptor-writable>
    pub(crate) fn writable(&self) -> bool {
        self.writable
    }
}

impl MLTensorMethods<crate::DomTypeHolder> for MLTensor {
    /// <https://www.w3.org/TR/webnn/#dom-mltensor-datatype>
    fn DataType(&self) -> MLOperandDataType {
        // > The dataType getter steps are to return this’s dataType.
        self.data_type
    }

    /// <https://www.w3.org/TR/webnn/#dom-mltensor-destroy>
    fn Destroy(&self) {
        // Step 1 .Set this.[[isDestroyed]] to true.
        self.is_destroyed.set(true);
        // TODO Step 2. For each promise in this.[[pendingPromises]]:
        // TODO Step 2.1. Remove promise from this.[[pendingPromises]].
        // TODO Step 2.2. Reject promise with an "InvalidStateError" DOMException.
        // Step 3. Enqueue the following steps to this.[[context]].[[timeline]]:
        // Step 3.1. Release this.[[data]].
        self.data.borrow_mut().take();
    }

    /// <https://www.w3.org/TR/webnn/#dom-mltensor-shape>
    #[allow(unsafe_code)]
    fn Shape(&self, cx: &mut JSContext, retval: MutableHandleValue) {
        // > The shape getter steps are to return this’s shape.
        let js_shape: Vec<f64> = self
            .shape
            .iter()
            .map(|&dimension| dimension as f64)
            .collect();
        unsafe { js_shape.to_jsval(cx.raw_cx(), retval) }
    }

    /// <https://www.w3.org/TR/webnn/#dom-mltensor-readable>
    fn Readable(&self) -> bool {
        // > The readable getter steps are to return this.[[descriptor]].readable.
        self.readable
    }

    /// <https://www.w3.org/TR/webnn/#dom-mltensor-writable>
    fn Writable(&self) -> bool {
        // > The writable getter steps are to return this.[[descriptor]].writable.
        self.writable
    }

    /// <https://www.w3.org/TR/webnn/#dom-mltensor-constant>
    fn Constant(&self) -> bool {
        // > The constant getter steps are to return this’s [[isConstant]].
        self.is_constant
    }
}
