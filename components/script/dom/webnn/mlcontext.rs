/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::rc::Rc;

use dom_struct::dom_struct;
use js::context::JSContext;
use script_bindings::cell::DomRefCell;
use script_bindings::cformat;
use script_bindings::domstring::DOMString;
use script_bindings::record::Record;
use script_bindings::reflector::{Reflector, reflect_dom_object_with_cx};
use script_bindings::root::DomRoot;
use servo_base::generic_channel::GenericCallback;

use crate::dom::bindings::codegen::Bindings::WebNNBinding::{
    MLContextLostInfo, MLContextMethods, MLContextOptions, MLInputOperandLayout, MLOpSupportLimits,
    MLOperandDataType, MLOperandDescriptor, MLPowerPreference, MLRankRange,
    MLSingleInputSupportLimits, MLTensorDescriptor, MLTensorLimits,
};
use crate::dom::bindings::codegen::UnionTypes::MaybeSharedArrayBufferViewOrMaybeSharedArrayBuffer;
use crate::dom::bindings::error::Error;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::str::USVString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::dom::webnn::mlgraph::MLGraph;
use crate::dom::webnn::mltensor::MLTensor;

/// <https://tc39.es/ecma262/#table-the-typedarray-constructors>
fn element_size(data_type: MLOperandDataType) -> usize {
    match data_type {
        MLOperandDataType::Float32 | MLOperandDataType::Int32 | MLOperandDataType::Uint32 => 4,
        MLOperandDataType::Float16 => 2,
        MLOperandDataType::Int64 | MLOperandDataType::Uint64 => 8,
        MLOperandDataType::Int8 | MLOperandDataType::Uint8 => 1,
    }
}

/// <https://www.w3.org/TR/webnn/#mloperanddescriptor-byte-length>
pub(crate) fn operand_descriptor_byte_length(descriptor: &MLOperandDescriptor) -> usize {
    // Step 1. Let elementLength be 1.
    // Step 2. For each dimension of desc.shape:
    // Step 2.1. Set elementLength to elementLength * dimension.
    let element_length: usize = descriptor
        .shape
        .iter()
        .map(|&dimension| dimension as usize)
        .product();
    // Step 3. Let elementSize be the element size of one of the
    // ArrayBufferView types that matches desc.dataType according to this table.
    let element_size = element_size(descriptor.dataType);
    // Step 4. Return elementLength * elementSize.
    element_length * element_size
}

/// <https://www.w3.org/TR/webnn/#valid-dimension>
fn valid_dimension(dimension: u32) -> bool {
    // > A valid dimension is an integer greater than zero and in the range of long.
    dimension > 0
}

/// <https://www.w3.org/TR/webnn/#mloperanddescriptor-element-count>
fn element_count(descriptor: &MLOperandDescriptor) -> usize {
    // Step 1. Let elementCount be 1.
    // Step 2. For each dimension of desc.shape:
    // Step 2.1. Set elementCount to elementCount * dimension.
    let element_count: usize = descriptor
        .shape
        .iter()
        .map(|&dimension| dimension as usize)
        .product();
    // Step 3. Return elementCount.
    element_count
}

/// <https://www.w3.org/TR/webnn/#mloperanddescriptor-check-dimensions>
pub(crate) fn check_dimensions(descriptor: &MLOperandDescriptor) -> bool {
    // Step 1. If any item of descriptor.shape is not a valid dimension, then
    // return false.
    // Step 2. If descriptor.shape's size is too large to be supported by the
    // implementation, then return false.
    // Step 3. If descriptor's element count is not a valid dimension, then
    // return false.
    // Step 4. If descriptor's byte length is not supported by the
    // implementation, then return false.
    if descriptor
        .shape
        .iter()
        .any(|&dimension| !valid_dimension(dimension)) ||
        descriptor.shape.len() > 8 ||
        !valid_dimension(element_count(descriptor) as u32) ||
        operand_descriptor_byte_length(descriptor) > usize::MAX
    {
        return false;
    }
    // Step 5. Return true.
    true
}

/// <https://www.w3.org/TR/webnn/#validate-buffer-with-descriptor>
#[allow(unsafe_code)]
pub(crate) fn validate_buffer_with_descriptor(
    descriptor: &MLOperandDescriptor,
    buffer: &MaybeSharedArrayBufferViewOrMaybeSharedArrayBuffer,
) -> bool {
    // Step 1. If bufferSource's byte length is not equal to descriptor's byte
    // length, then return false.
    let buffer_source_byte_length = match buffer {
        MaybeSharedArrayBufferViewOrMaybeSharedArrayBuffer::ArrayBuffer(buffer) => unsafe {
            buffer.as_slice().len()
        },
        MaybeSharedArrayBufferViewOrMaybeSharedArrayBuffer::ArrayBufferView(view) => unsafe {
            view.as_slice().len()
        },
    };
    if buffer_source_byte_length != operand_descriptor_byte_length(descriptor) {
        return false;
    }
    // TODO: Step 2. Switch on the type of bufferSource.
    true
}

/// <https://www.w3.org/TR/webnn/#api-mlcontext>
#[dom_struct]
pub(crate) struct MLContext {
    reflector_: Reflector,
    /// <https://www.w3.org/TR/webnn/#dom-mlcontext-accelerated-slot>
    accelerated: bool,
    /// <https://www.w3.org/TR/webnn/#dom-mlcontext-powerpreference-slot>
    power_preference: MLPowerPreference,
    /// <https://www.w3.org/TR/webnn/#mlcontext-is-lost>
    is_lost: Cell<bool>,
    /// <https://www.w3.org/TR/webnn/#dom-mlcontext-lost-slot>
    #[ignore_malloc_size_of = "Rc"]
    lost_promise: DomRefCell<Option<Rc<Promise>>>,
    #[no_trace]
    #[ignore_malloc_size_of = "GenericSender"]
    channel: webnn::WebNN,
}

impl MLContext {
    pub(crate) fn new_inherited(options: &MLContextOptions) -> MLContext {
        MLContext {
            reflector_: Reflector::new(),
            accelerated: options.accelerated,
            power_preference: options.powerPreference.clone(),
            is_lost: Cell::new(false),
            lost_promise: DomRefCell::new(None),
            channel: webnn::WebNN::new(webnn::MockBackend::new()),
        }
    }

    pub(crate) fn new(
        global: &GlobalScope,
        options: &MLContextOptions,
        cx: &mut JSContext,
    ) -> DomRoot<MLContext> {
        reflect_dom_object_with_cx(Box::new(MLContext::new_inherited(options)), global, cx)
    }

    pub(crate) fn channel(&self) -> &webnn::WebNN {
        &self.channel
    }

    /// <https://www.w3.org/TR/webnn/#mlcontext-is-lost>
    pub(crate) fn is_lost(&self) -> bool {
        self.is_lost.get()
    }
}

/// Returns all supported `MLOperandDataType` variants.
/// <https://www.w3.org/TR/webnn/#enumdef-mloperanddatatype>
fn all_data_types() -> Vec<MLOperandDataType> {
    vec![
        MLOperandDataType::Float32,
        MLOperandDataType::Float16,
        MLOperandDataType::Int32,
        MLOperandDataType::Uint32,
        MLOperandDataType::Int64,
        MLOperandDataType::Uint64,
        MLOperandDataType::Int8,
        MLOperandDataType::Uint8,
    ]
}

/// Returns mock `MLTensorLimits` claiming support for all data types and
/// ranks 0-8, used as a default for `opSupportLimits()`.
/// <https://www.w3.org/TR/webnn/#dictdef-mltensorlimits>
fn default_tensor_limits() -> MLTensorLimits {
    MLTensorLimits {
        dataTypes: Some(all_data_types()),
        rankRange: Some(MLRankRange {
            min: Some(0),
            max: Some(8),
        }),
    }
}

/// Returns `MLSingleInputSupportLimits` for single-input operators (e.g.
/// `add`), applying the same default tensor limits to both the input and
/// output operands.
/// <https://www.w3.org/TR/webnn/#dictdef-mlsingleinputsupportlimits>
fn single_limits() -> MLSingleInputSupportLimits {
    let limits = default_tensor_limits();
    MLSingleInputSupportLimits {
        input: Some(limits.clone()),
        output: Some(limits),
    }
}

impl MLContextMethods<crate::DomTypeHolder> for MLContext {
    /// <https://www.w3.org/TR/webnn/#dom-mlcontext-dispatch>
    fn Dispatch(
        &self,
        _cx: &mut JSContext,
        graph: &MLGraph,
        inputs: Record<USVString, DomRoot<MLTensor>>,
        outputs: Record<USVString, DomRoot<MLTensor>>,
    ) -> Result<(), Error> {
        // The dispatch(graph, inputs, outputs) method steps are:
        // Step 1. If graph.[[context]] is not this, then throw a TypeError.
        if graph.context() != self {
            return Err(Error::Type(
                c"Graph is not associated with this MLContext.".to_owned(),
            ));
        }
        // Step 2. If graph.[[isDestroyed]] is true, then throw an
        // "InvalidStateError" DOMException.
        if graph.is_destroyed() {
            return Err(Error::InvalidState(Some("Graph is destroyed.".to_owned())));
        }
        // Step 3. Let allTensors be a list of MLTensors consisting of
        // inputs's values extended by outputs's values.
        let all_tensors: Vec<&DomRoot<MLTensor>> = inputs
            .iter()
            .map(|(_, t)| t)
            .chain(outputs.iter().map(|(_, t)| t))
            .collect();
        // Step 4. If allTensors contains any duplicate items, then
        // throw a TypeError.
        for i in 0..all_tensors.len() {
            for j in (i + 1)..all_tensors.len() {
                if std::ptr::eq(
                    &**all_tensors[i] as *const MLTensor,
                    &**all_tensors[j] as *const MLTensor,
                ) {
                    return Err(Error::Type(
                        c"Duplicate tensors in inputs and outputs.".to_owned(),
                    ));
                }
            }
        }
        // Step 5. For each tensor of allTensors:
        // Step 5.1. If tensor.[[context]] is not this, then throw a
        // TypeError.
        // Step 5.2. If tensor.[[isDestroyed]] is true, then throw a
        // TypeError.
        for tensor in &all_tensors {
            if tensor.context() != self {
                return Err(Error::Type(
                    c"Tensor is not associated with this MLContext.".to_owned(),
                ));
            }
            if tensor.is_destroyed() {
                return Err(Error::Type(c"Tensor is destroyed.".to_owned()));
            }
        }
        // TODO: Step 6. If validating tensors with descriptors given inputs
        // and graph.[[inputDescriptors]] returns false, then throw a
        // TypeError.
        // (Backend-handled: inputDescriptors are not stored on MLGraph.)
        // Step 7. If validating tensors with descriptors given outputs
        // and graph.[[outputDescriptors]] returns false, then throw a
        // TypeError.
        let output_descriptors = graph.output_descriptors();
        if outputs.len() != output_descriptors.len() {
            return Err(Error::Type(
                c"Number of output tensors does not match graph outputs.".to_owned(),
            ));
        }
        for (name, tensor) in outputs.iter() {
            if tensor.is_constant() {
                return Err(Error::Type(
                    c"Constant tensor cannot be used as output.".to_owned(),
                ));
            }
            let Some(&(data_type, ref shape)) = output_descriptors.get(name.0.as_str()) else {
                return Err(Error::Type(cformat!(
                    "Output '{}' not found in graph outputs.",
                    name.0
                )));
            };
            if tensor.data_type() != data_type || tensor.shape() != shape.as_slice() {
                return Err(Error::Type(
                    c"Output tensor descriptor does not match graph output descriptor.".to_owned(),
                ));
            }
        }
        // Step 8. Enqueue the following steps to
        // graph.[[context]].[[timeline]]:
        // Step 8.1. Run these steps, but abort when this is lost:
        // Step 8.1.1. Issue a compute request to graph.[[implementation]]
        // given inputs and outputs.
        // (Implementation: compute runs asynchronously on the backend thread.
        // The callback writes results to output tensors via a task queued on
        // the ml_task_source.)
        let mut input_data: Vec<(String, Vec<u8>)> = Vec::new();
        for (name, tensor) in inputs.iter() {
            if let Some(data) = tensor.read_data() {
                input_data.push((name.0.clone(), data));
            }
        }
        let input_slices: Vec<(String, &[u8])> = input_data
            .iter()
            .map(|(n, d)| (n.clone(), d.as_slice()))
            .collect();
        let output_labels: Vec<String> = outputs.iter().map(|(k, _)| k.0.clone()).collect();
        let output_labels_for_callback = output_labels.clone();
        let output_tensors: Vec<(String, Trusted<MLTensor>)> = outputs
            .iter()
            .map(|(name, tensor)| (name.0.clone(), Trusted::new(&**tensor)))
            .collect();
        let task_source = self.global().task_manager().ml_task_source().to_sendable();
        let callback = GenericCallback::new(move |response: Result<webnn::RunResponse, _>| {
            let response = response.unwrap();
            let output_tensors = output_tensors.clone();
            let output_labels = output_labels_for_callback.clone();
            task_source.queue(task!(webnn_dispatch_result: move |_cx| {
                match response.result {
                    Ok(result) => {
                        for (name, trusted_tensor) in output_tensors.iter() {
                            let tensor = trusted_tensor.root();
                            let idx = output_labels
                                .iter()
                                .position(|n| n == name.as_str())
                                .unwrap_or(0);
                            if let Some(data) = result.outputs.get(idx) {
                                tensor.write_data(data);
                            }
                        }
                    },
                    Err(e) => log::error!("WebNN dispatch error: {e}"),
                }
            }));
        })
        .expect("WebNN callback creation");
        self.channel
            .run(graph.graph_id(), &input_slices, &output_labels, callback);
        Ok(())
    }

    /// <https://www.w3.org/TR/webnn/#dom-mlcontext-createtensor>
    fn CreateTensor(
        &self,
        cx: &mut js::context::JSContext,
        descriptor: &MLTensorDescriptor,
    ) -> Rc<Promise> {
        // The createTensor(descriptor) method steps are:
        // Step 1. Let global be this's relevant global object.
        // Step 2. Let realm be this's relevant realm.
        let global = &self.global();
        // Step 3. If this is lost, then return a new promise in realm
        // rejected with an "InvalidStateError" DOMException.
        if self.is_lost() {
            let promise = Promise::new(cx, &self.global());
            promise.reject_error(cx, Error::InvalidState(Some("Context is lost".into())));
            return promise;
        }
        // Step 4. Let tensor be the result of creating an MLTensor given
        // this, and descriptor.
        let tensor = MLTensor::new(
            global,
            self,
            descriptor.parent.dataType,
            descriptor.parent.shape.clone(),
            descriptor.readable,
            descriptor.writable,
            false,
            cx,
        );
        // Step 5. Let promise be a new promise in realm.
        // Step 6. Enqueue the following steps to this.[[timeline]]:
        // (Implementation: data allocation happens synchronously; no async
        // timeline.)
        // Step 6.1. Run these steps, but abort when this is lost:
        // Step 6.1.1. Create tensor.[[data]] given descriptor and initialize
        // all bytes to zeros.
        // Step 6.1.2. If that fails, then queue an ML task with global to
        // reject promise with an "UnknownError" DOMException, and abort
        // these steps.
        // Step 6.1.3. Otherwise, queue an ML task with global to resolve
        // promise with tensor.
        // TODO: Step 6.2. If aborted, then queue an ML task with global to
        // reject promise with an "InvalidStateError" DOMException.
        // Step 7. Return promise.
        let promise = Promise::new(cx, &self.global());
        promise.resolve_native(cx, &tensor);
        promise
    }

    /// <https://www.w3.org/TR/webnn/#dom-mlcontext-createconstanttensor>
    fn CreateConstantTensor(
        &self,
        cx: &mut js::context::JSContext,
        descriptor: &MLOperandDescriptor,
        input_data: MaybeSharedArrayBufferViewOrMaybeSharedArrayBuffer,
    ) -> Rc<Promise> {
        // The createConstantTensor(descriptor, inputData) method steps are:
        // Step 1. Let global be this's relevant global object.
        // Step 2. Let realm be this's relevant realm.
        let global = &self.global();
        // Step 3. If this is lost, then return a new promise in realm
        // rejected with an "InvalidStateError" DOMException.
        if self.is_lost() {
            let promise = Promise::new(cx, global);
            promise.reject_error(cx, Error::InvalidState(Some("Context is lost".into())));
            return promise;
        }
        // Step 4. If checking dimensions given descriptor returns false, then
        // return a new promise in realm rejected with a TypeError.
        if !check_dimensions(descriptor) {
            let promise = Promise::new(cx, global);
            promise.reject_error(cx, Error::Type(c"A dimension size cannot be 0.".to_owned()));
            return promise;
        }
        // Step 5. If validating buffer with descriptor given inputData and
        // descriptor returns false, then return a new promise in realm
        // rejected with a TypeError.
        if !validate_buffer_with_descriptor(descriptor, &input_data) {
            let promise = Promise::new(cx, global);
            promise.reject_error(
                cx,
                Error::Type(c"Buffer size does not match descriptor.".to_owned()),
            );
            return promise;
        }
        // Step 6. Let bytes be the result of getting a copy of the bytes
        // held by the buffer source given inputData.
        #[allow(unsafe_code)]
        let src: &[u8] = match input_data {
            MaybeSharedArrayBufferViewOrMaybeSharedArrayBuffer::ArrayBuffer(ref data) => unsafe {
                data.as_slice()
            },
            MaybeSharedArrayBufferViewOrMaybeSharedArrayBuffer::ArrayBufferView(ref data) => unsafe {
                data.as_slice()
            },
        };
        // Step 7. Assert: bytes's length is equal to descriptor's byte length.
        // Step 8. Let tensor be the result of creating a constant MLTensor
        // given this, and descriptor.
        let tensor = MLTensor::new(
            global,
            self,
            descriptor.dataType,
            descriptor.shape.clone(),
            false,
            false,
            true,
            cx,
        );
        // Step 9. Let promise be a new promise in realm.
        // Step 10. Enqueue the following steps to this.[[timeline]]:
        // (Implementation: data write happens synchronously; no async
        // timeline.)
        // Step 10.1. Run these steps, but abort when this is lost:
        // Step 10.1.1. Create tensor.[[data]] given descriptor.
        // Step 10.1.2. If that fails, then queue an ML task with global to
        // reject promise with an "UnknownError" DOMException, and abort
        // these steps.
        // Step 10.1.3. Copy bytes to tensor.[[data]].
        // Step 10.1.4. If that fails, then queue an ML task with global to
        // reject promise with an "UnknownError" DOMException, and abort
        // these steps.
        // Step 10.1.5. Otherwise, queue an ML task with global to resolve
        // promise with tensor.
        // TODO: Step 10.2. If aborted, then queue an ML task with global to
        // reject promise with an "InvalidStateError" DOMException.
        tensor.write_data(src);
        // Step 11. Return promise.
        let promise = Promise::new(cx, &self.global());
        promise.resolve_native(cx, &tensor);
        promise
    }

    /// <https://www.w3.org/TR/webnn/#dom-mlcontext-readtensor>
    fn ReadTensor(&self, cx: &mut js::context::JSContext, tensor: &MLTensor) -> Rc<Promise> {
        // The readTensor(tensor) method steps are:
        // Step 1. Let global be this's relevant global object.
        // Step 2. Let realm be this's relevant realm.
        // Step 3. If tensor.[[context]] is not this, then return a new
        // promise in realm rejected with a TypeError.
        if tensor.context() != self {
            let promise = Promise::new(cx, &self.global());
            promise.reject_error(
                cx,
                Error::Type(c"Tensor is not associated with this MLContext.".to_owned()),
            );
            return promise;
        }
        // Step 4. If tensor.[[isDestroyed]] is true, then return a new
        // promise in realm rejected with a TypeError.
        if tensor.is_destroyed() {
            let promise = Promise::new(cx, &self.global());
            promise.reject_error(cx, Error::Type(c"Tensor is destroyed.".to_owned()));
            return promise;
        }
        // Step 5. If tensor.[[descriptor]].readable is false, then return a
        // new promise in realm rejected with a TypeError.
        if !tensor.readable() {
            let promise = Promise::new(cx, &self.global());
            promise.reject_error(cx, Error::Type(c"Tensor is not readable.".to_owned()));
            return promise;
        }
        // Step 6. Let promise be a new promise in realm.
        let promise = Promise::new(cx, &self.global());
        // TODO: Step 7. Append promise to tensor.[[pendingPromises]].
        // Step 8. Enqueue the following steps to
        // tensor.[[context]].[[timeline]]:
        // (Implementation: data read happens synchronously; no async
        // timeline.)
        // Step 8.1. Run these steps, but abort when this is lost:
        // Step 8.1.1. Let bytes be a byte sequence containing a copy of
        // tensor.[[data]].
        if let Some(data) = tensor.read_data() {
            // Step 8.1.2. If that fails, then queue an ML task with global
            // and the following steps:
            // Step 8.1.2.1. Remove promise from tensor.[[pendingPromises]].
            // Step 8.1.2.2. Reject promise with an "UnknownError"
            // DOMException, and abort these steps.
            let len = data.len();
            if len == 0 {
                promise.reject_error(cx, Error::NotSupported(None));
                return promise;
            }
            // Step 8.1.3. Otherwise, queue an ML task with global and the
            // following steps:
            // Step 8.1.3.1. Remove promise from tensor.[[pendingPromises]].
            // Step 8.1.3.2. Let buffer be the result of creating an
            // ArrayBuffer from bytes in realm.
            // Step 8.1.3.3. Resolve promise with buffer.
            #[allow(unsafe_code)]
            unsafe {
                let ptr = libc::malloc(len);
                if ptr.is_null() {
                    promise.reject_error(cx, Error::NotSupported(None));
                    return promise;
                }
                std::ptr::copy_nonoverlapping(data.as_ptr(), ptr as *mut u8, len);
                let obj = js::jsapi::NewArrayBufferWithContents(cx.raw_cx(), len, ptr);
                if obj.is_null() {
                    libc::free(ptr);
                    promise.reject_error(cx, Error::NotSupported(None));
                    return promise;
                }
                let obj_val = js::jsval::ObjectValue(obj);
                promise.resolve_native(cx, &obj_val);
            }
        } else {
            // TODO: Step 8.2. If aborted, then queue an ML task with global
            // to reject promise with an "InvalidStateError" DOMException.
            promise.reject_error(cx, Error::NotSupported(None));
        }
        // Step 9. Return promise.
        promise
    }

    /// <https://www.w3.org/TR/webnn/#dom-mlcontext-readtensor>
    #[allow(unsafe_code)]
    fn ReadTensor_(
        &self,
        cx: &mut js::context::JSContext,
        tensor: &MLTensor,
        mut output_data: MaybeSharedArrayBufferViewOrMaybeSharedArrayBuffer,
    ) -> Rc<Promise> {
        // The readTensor(tensor, outputData) method steps are:
        // Step 1. Let global be this's relevant global object.
        // Step 2. Let realm be this's relevant realm.
        // Step 3. If tensor.[[context]] is not this, then return a new
        // promise in realm rejected with a TypeError.
        if tensor.context() != self {
            let promise = Promise::new(cx, &self.global());
            promise.reject_error(
                cx,
                Error::Type(c"Tensor is not associated with this MLContext.".to_owned()),
            );
            return promise;
        }
        // Step 4. If tensor.[[isDestroyed]] is true, then return a new
        // promise in realm rejected with a TypeError.
        if tensor.is_destroyed() {
            let promise = Promise::new(cx, &self.global());
            promise.reject_error(cx, Error::Type(c"Tensor is destroyed.".to_owned()));
            return promise;
        }
        // Step 5. If tensor.[[descriptor]].readable is false, then return a
        // new promise in realm rejected with a TypeError.
        if !tensor.readable() {
            let promise = Promise::new(cx, &self.global());
            promise.reject_error(cx, Error::Type(c"Tensor is not readable.".to_owned()));
            return promise;
        }
        // Step 6. If validating buffer with descriptor given outputData and
        // tensor.[[descriptor]] returns false, then return a new promise in
        // realm rejected with a TypeError.
        let tensor_descriptor = MLOperandDescriptor {
            dataType: tensor.data_type(),
            shape: tensor.shape().to_vec(),
        };
        if !validate_buffer_with_descriptor(&tensor_descriptor, &output_data) {
            let promise = Promise::new(cx, &self.global());
            promise.reject_error(
                cx,
                Error::Type(c"Buffer size does not match tensor descriptor.".to_owned()),
            );
            return promise;
        }
        // Step 7. Let promise be a new promise in realm.
        let promise = Promise::new(cx, &self.global());
        // TODO: Step 8. Append promise to tensor.[[pendingPromises]].
        // Step 9. Enqueue the following steps to
        // tensor.[[context]].[[timeline]]:
        // (Implementation: data copy happens synchronously; no async
        // timeline.)
        // Step 9.1. Run these steps, but abort when this is lost:
        // Step 9.1.1. Let bytes be a byte sequence containing a copy of
        // tensor.[[data]].
        let data = match tensor.read_data() {
            Some(d) => d,
            None => {
                promise.reject_error(cx, Error::NotSupported(None));
                return promise;
            },
        };
        // Step 9.1.2. If that fails, then queue an ML task with global to
        // run these steps:
        // Step 9.1.2.1. Remove promise from tensor.[[pendingPromises]].
        // Step 9.1.2.2. Reject promise with an "UnknownError" DOMException,
        // and abort these steps.
        // Step 9.1.3. Otherwise, queue an ML task with global to run these
        // steps:
        // Step 9.1.3.1. Remove promise from tensor.[[pendingPromises]].
        // TODO: Step 9.1.3.2. If outputData is detached, then reject promise
        // with a TypeError, and abort these steps.
        // Step 9.1.3.3. Write bytes to outputData.
        // Step 9.1.3.4. Resolve promise with undefined.
        let len = data.len();
        match output_data {
            MaybeSharedArrayBufferViewOrMaybeSharedArrayBuffer::ArrayBuffer(ref mut buf) => {
                let slice: &mut [u8] = unsafe { buf.as_mut_slice() };
                let end = len.min(slice.len());
                slice[..end].copy_from_slice(&data[..end]);
            },
            MaybeSharedArrayBufferViewOrMaybeSharedArrayBuffer::ArrayBufferView(ref mut view) => {
                let slice: &mut [u8] = unsafe { view.as_mut_slice() };
                let end = len.min(slice.len());
                slice[..end].copy_from_slice(&data[..end]);
            },
        }
        // TODO: Step 9.2. If aborted, then queue an ML task with global to
        // reject promise with an "InvalidStateError" DOMException.
        // Step 10. Return promise.
        promise.resolve_native(cx, &());
        promise
    }

    /// <https://www.w3.org/TR/webnn/#dom-mlcontext-writetensor>
    #[allow(unsafe_code)]
    fn WriteTensor(
        &self,
        _cx: &mut js::context::JSContext,
        tensor: &MLTensor,
        input_data: MaybeSharedArrayBufferViewOrMaybeSharedArrayBuffer,
    ) -> Result<(), Error> {
        // The writeTensor(tensor, inputData) method steps are:
        // Step 1. If tensor.[[context]] is not this, then throw a TypeError.
        if tensor.context() != self {
            return Err(Error::Type(
                c"Tensor is not associated with this MLContext.".to_owned(),
            ));
        }
        // Step 2. If tensor.[[isDestroyed]] is true, then throw a TypeError.
        if tensor.is_destroyed() {
            return Err(Error::Type(c"Tensor is destroyed.".to_owned()));
        }
        // Step 3. If tensor.[[descriptor]].writable is false, then throw a
        // TypeError.
        if !tensor.writable() {
            return Err(Error::Type(c"Tensor is not writable.".to_owned()));
        }
        // Step 4. If validating buffer with descriptor given inputData and
        // tensor.[[descriptor]] returns false, then throw a TypeError.
        let tensor_descriptor = MLOperandDescriptor {
            dataType: tensor.data_type(),
            shape: tensor.shape().to_vec(),
        };
        if !validate_buffer_with_descriptor(&tensor_descriptor, &input_data) {
            return Err(Error::Type(
                c"Buffer size does not match tensor descriptor.".to_owned(),
            ));
        }
        // Step 5. Let bytes be the result of getting a copy of the bytes
        // held by the buffer source given inputData.
        let src: &[u8] = match input_data {
            MaybeSharedArrayBufferViewOrMaybeSharedArrayBuffer::ArrayBuffer(ref data) => unsafe {
                data.as_slice()
            },
            MaybeSharedArrayBufferViewOrMaybeSharedArrayBuffer::ArrayBufferView(ref data) => unsafe {
                data.as_slice()
            },
        };
        // Step 6. Assert: bytes's length is equal to tensor.[[descriptor]]'s
        // byte length.
        // Step 7. Enqueue the following steps to
        // tensor.[[context]].[[timeline]]:
        // (Implementation: data copy happens synchronously; no async
        // timeline.)
        // Step 7.1. Run these steps, but abort when this is lost:
        // Step 7.1.1. Copy bytes to tensor.[[data]].
        tensor.write_data(src);
        Ok(())
    }

    /// Returns the operator support limits for this context. The mock backend
    /// claims support for all data types, ranks 0-8, NCHW input layout, and a
    /// 1GB max tensor byte length. Currently only the `add` operator's limits
    /// are populated.
    /// <https://www.w3.org/TR/webnn/#dom-mlcontext-opsupportlimits>
    /// <https://www.w3.org/TR/webnn/#dictdef-mlopsupportlimits>
    fn OpSupportLimits(&self) -> MLOpSupportLimits {
        let tensor_limits = default_tensor_limits();
        let input_limits = MLTensorLimits {
            dataTypes: Some(all_data_types()),
            rankRange: Some(MLRankRange {
                min: Some(0),
                max: Some(8),
            }),
        };
        let single_limits = single_limits();
        MLOpSupportLimits {
            preferredInputLayout: Some(MLInputOperandLayout::Nchw),
            maxTensorByteLength: Some(1_000_000_000),
            input: Some(input_limits),
            constant: Some(tensor_limits.clone()),
            output: Some(tensor_limits.clone()),
            add: Some(single_limits.clone()),
        }
    }

    /// <https://www.w3.org/TR/webnn/#dom-mlcontext-destroy>
    fn Destroy(&self, cx: &mut JSContext) {
        // The destroy() method steps are:
        // Step 1. If this is lost, then abort these steps.
        if self.is_lost() {
            return;
        }
        // Step 2. Run the steps to lose this with an implementation-defined
        // message.
        self.is_lost.set(true);
        let promise = self.Lost(cx);
        let info = MLContextLostInfo {
            message: Some(DOMString::from("Context destroyed.")),
        };
        promise.resolve_native(cx, &info);
    }

    /// <https://www.w3.org/TR/webnn/#dom-mlcontext-accelerated>
    fn Accelerated(&self) -> bool {
        // The accelerated getter steps are to return this.[[accelerated]].
        self.accelerated
    }

    /// <https://www.w3.org/TR/webnn/#dom-mlcontext-lost>
    fn Lost(&self, cx: &mut JSContext) -> Rc<Promise> {
        // The lost getter steps are to return this's [[lost]] Promise.
        let mut slot = self.lost_promise.borrow_mut();
        if let Some(ref promise) = *slot {
            return Rc::clone(promise);
        }
        let promise = Promise::new(cx, &self.global());
        *slot = Some(Rc::clone(&promise));
        promise
    }
}
