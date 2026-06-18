/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::collections::HashMap;
use std::rc::Rc;

use dom_struct::dom_struct;
use js::context::JSContext;
use js::gc::HandleObject;
use script_bindings::cformat;
use script_bindings::codegen::GenericBindings::WindowBinding::WindowMethods;
use script_bindings::record::Record;
use script_bindings::reflector::{Reflector, reflect_dom_object_with_proto_and_cx};
use script_bindings::root::DomRoot;

use crate::dom::bindings::codegen::Bindings::PermissionStatusBinding::PermissionName;
use crate::dom::bindings::codegen::Bindings::WebNNBinding::{
    MLGraphBuilderMethods, MLOperandDataType, MLOperandDescriptor, MLOperatorOptions,
};
use crate::dom::bindings::codegen::UnionTypes::MaybeSharedArrayBufferViewOrMaybeSharedArrayBuffer;
use crate::dom::bindings::error::Error;
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::str::USVString;
use crate::dom::bindings::weakref::WeakRef;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::dom::webnn::mlcontext::{MLContext, check_dimensions, validate_buffer_with_descriptor};
use crate::dom::webnn::mlgraph::MLGraph;
use crate::dom::webnn::mloperand::MLOperand;
use crate::routed_promise::callback_promise;

/// <https://www.w3.org/TR/webnn/#mlgraphbuilder-validate-operand>
fn validate_operand(builder: &MLGraphBuilder, operand: &MLOperand) -> bool {
    // > To validate operand given MLGraphBuilder builder and MLOperand operand,
    // return true if operand.[[builder]] is builder, and false otherwise.
    operand.builder() == builder
}

/// <https://www.w3.org/TR/webnn/#bidirectionally-broadcasting>
fn bidirectionally_broadcast(shape_a: &[u32], shape_b: &[u32]) -> Result<Vec<u32>, ()> {
    // Step 1. Let sizeA be shapeA's size.
    let size_a = shape_a.len();
    // Step 2. Let sizeB be shapeB's size.
    let size_b = shape_b.len();
    // Step 3. Let outputSize be the maximum of sizeA and sizeB.
    let output_size = std::cmp::max(size_a, size_b);
    // Step 4. Let paddedA be a clone of shapeA.
    let mut padded_a = shape_a.to_vec();
    // Step 5. While paddedA's size is less than outputSize, prepend 1 to paddedA.
    while padded_a.len() < output_size {
        padded_a.insert(0, 1);
    }
    // Step 6. Let paddedB be a clone of shapeB.
    let mut padded_b = shape_b.to_vec();
    // Step 7. While paddedB's size is less than outputSize, prepend 1 to paddedB.
    while padded_b.len() < output_size {
        padded_b.insert(0, 1);
    }
    // Step 8. Let outputShape be a new list.
    let mut output_shape = Vec::new();
    // Step 9. For each index in the range 0 to outputSize, exclusive:
    for index in 0..output_size {
        // Step 9.1. Let dimA be paddedA[index].
        let dim_a = padded_a[index];
        // Step 9.2. Let dimB be paddedB[index].
        let dim_b = padded_b[index];
        // Step 9.3. If dimA is not equal to dimB, and dimA is not equal to 1, and dimB is not equal to 1, then return failure.
        if dim_a != dim_b && dim_a != 1 && dim_b != 1 {
            return Err(());
        }
        // Step 9.4. Append the maximum of dimA and dimB to outputShape.
        output_shape.push(std::cmp::max(dim_a, dim_b));
    }
    // Step 10. Return outputShape.
    Ok(output_shape)
}

/// <https://www.w3.org/TR/webnn/#mlgraphbuilder>
#[dom_struct]
pub(crate) struct MLGraphBuilder {
    reflector_: Reflector,
    /// <https://www.w3.org/TR/webnn/#dom-mlgraphbuilder-context-slot
    context: WeakRef<MLContext>,
    #[no_trace]
    #[ignore_malloc_size_of = "GenericSender"]
    channel: webnn::WebNN,
    /// <https://www.w3.org/TR/webnn/#dom-mlgraphbuilder-context-slot>
    has_built: Cell<bool>,
    builder_id: Cell<webnn::BuilderId>,
}

impl MLGraphBuilder {
    pub(crate) fn new_inherited(context: &MLContext) -> MLGraphBuilder {
        let channel = context.channel().clone();
        let id = channel.create_builder();
        MLGraphBuilder {
            reflector_: Reflector::new(),
            context: WeakRef::new(context),
            channel,
            has_built: Cell::new(false),
            builder_id: Cell::new(id),
        }
    }

    pub(crate) fn new(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        context: &MLContext,
        cx: &mut JSContext,
    ) -> DomRoot<MLGraphBuilder> {
        reflect_dom_object_with_proto_and_cx(
            Box::new(Self::new_inherited(context)),
            global,
            proto,
            cx,
        )
    }

    #[allow(dead_code)]
    pub(crate) fn context(&self) -> &WeakRef<MLContext> {
        &self.context
    }

    /// <https://www.w3.org/TR/webnn/#mlgraphbuilder-can-build>
    pub(crate) fn can_build(&self) -> bool {
        // > An MLGraphBuilder can build if its [[hasBuilt]] is false and its [[context]] is not lost.
        if self.has_built.get() {
            return false;
        }
        if let Some(context) = self.context.root() {
            if context.is_lost() {
                return false;
            }
        }
        true
    }

    /// <https://www.w3.org/TR/webnn/#mlgraphbuilder-element-wise-binary-op>
    fn create_element_wise_binary(
        &self,
        cx: &mut JSContext,
        operation: &str,
        a: &MLOperand,
        b: &MLOperand,
        options: &MLOperatorOptions,
    ) -> Result<DomRoot<MLOperand>, Error> {
        // To create an element-wise binary operation given op, a, b, and options:
        // TODO: Step 1. Assert: op is one of "add", "sub", "mul", "div",
        // "max", "min", "pow".
        // Step 2. If this can not build, then throw an "InvalidStateError"
        // DOMException.
        if !self.can_build() {
            return Err(Error::InvalidState(Some("Cannot build.".to_owned())));
        }
        // Step 3. If validating operand with this and any of a and b returns
        // false, then throw a TypeError.
        if !validate_operand(self, a) || !validate_operand(self, b) {
            return Err(Error::Type(cformat!(
                "Input is from another builder. [{}]",
                options.label.0
            )));
        }
        // Step 4. If a's dataType is not equal to b's dataType, then throw a
        // TypeError.
        if a.data_type() != b.data_type() {
            return Err(Error::Type(cformat!(
                "Inputs must have the same data type. [{}]",
                options.label.0
            )));
        }
        // Step 5. Let outputShape be the result of bidirectionally broadcasting
        // a's shape and b's shape.
        // Step 5.1. If that returns failure, then throw a TypeError.
        let shape = bidirectionally_broadcast(a.shape(), b.shape()).map_err(|_| {
            Error::Type(cformat!(
                "Input shapes are not broadcastable. [{}]",
                options.label.0
            ))
        })?;
        // Step 6. Let descriptor be the result of creating an
        // MLOperandDescriptor given a's dataType and outputShape.
        // Step 7. Make graph connections:
        // Step 7.1. Let output be the result of creating an MLOperand given
        // this and descriptor.
        // Step 7.2. Let operator be an operator for the op operation, given a,
        // b, and options.
        // Step 7.3. Set output.[[operator]] to operator.
        // Step 7.4. Set operator's inputs to a and b.
        // Step 7.5. Set operator's output to output.
        let input_ids = [a.operand_id(), b.operand_id()];
        let operand_id = self.channel.add_operator(
            self.builder_id.get(),
            operation,
            &input_ids,
            a.data_type() as u32,
            &shape,
            &webnn::OperatorOptions::new(),
            options.label.0.as_str(),
        );
        // Step 8. Return output.
        Ok(MLOperand::new(
            &self.global(),
            operand_id,
            a.data_type(),
            shape,
            self,
            cx,
        ))
    }
}

impl MLGraphBuilderMethods<crate::DomTypeHolder> for MLGraphBuilder {
    /// <https://www.w3.org/TR/webnn/#dom-mlgraphbuilder-mlgraphbuilder>
    fn Constructor(
        cx: &mut JSContext,
        global: &GlobalScope,
        proto: Option<HandleObject>,
        context: &MLContext,
    ) -> Result<DomRoot<MLGraphBuilder>, Error> {
        // Step 1. If this's relevant global object's associated Document is
        //         not allowed to use the webnn feature, then throw a
        //         "SecurityError" DOMException.
        let window = global.as_window();
        let document = window.Document();
        if !document.allowed_to_use_feature(PermissionName::WebNN) {
            return Err(Error::Security(Some("WebNN not allowed to use".into())));
        }
        // Step 2. If context is lost, then throw an "InvalidStateError"
        //         DOMException.
        if context.is_lost() {
            return Err(Error::InvalidState(Some(
                "Cannot construct MLGraphBuilder: context is lost.".to_owned(),
            )));
        }
        // Step 3. Set this.[[context]] to context.
        // Step 4. Set this.[[hasBuilt]] to false.
        Ok(MLGraphBuilder::new(global, proto, context, cx))
    }

    /// <https://www.w3.org/TR/webnn/#dom-mlgraphbuilder-input>
    fn Input(
        &self,
        cx: &mut JSContext,
        name: USVString,
        descriptor: &MLOperandDescriptor,
    ) -> Result<DomRoot<MLOperand>, Error> {
        // Step 1. If this cannot build, throw an "InvalidStateError" DOMException.
        if !self.can_build() {
            return Err(Error::InvalidState(Some("Cannot build.".to_owned())));
        };
        // Step 2. If name is empty, throw a TypeError.
        if name.0.is_empty() {
            return Err(Error::Type(c"The name is empty.".to_owned()));
        }
        // TODO: Step 3. If any MLOperand in this.[[graph]].[[inputs]] has
        //                [[name]] equal to name, then return that MLOperand.
        // Step 4. If checking dimensions given descriptor returns false, throw a TypeError.
        if !check_dimensions(descriptor) {
            return Err(Error::Type(c"A dimension size cannot be 0.".to_owned()));
        }

        // Step 5. Make graph connections:
        // Step 5.1. Let operand = creating an MLOperand given this and descriptor.
        // Step 5.2. Set operand.[[name]] to name. (Skipped as we are not using)
        // Step 5.3. Add operand to this's graph's inputs.
        let operand_id = self.channel.add_input(
            self.builder_id.get(),
            name.0.as_str(),
            descriptor.dataType as u32,
            &descriptor.shape,
        );
        let operand = MLOperand::new(
            &self.global(),
            operand_id,
            descriptor.dataType,
            descriptor.shape.clone(),
            self,
            cx,
        );
        // Step 6. Return operand.
        Ok(operand)
    }

    /// <https://www.w3.org/TR/webnn/#dom-mlgraphbuilder-constant>
    #[allow(unsafe_code)]
    fn Constant(
        &self,
        cx: &mut JSContext,
        descriptor: &MLOperandDescriptor,
        buffer: MaybeSharedArrayBufferViewOrMaybeSharedArrayBuffer,
    ) -> Result<DomRoot<MLOperand>, Error> {
        // Step 1. If this cannot build, throw an "InvalidStateError" DOMException.
        if !self.can_build() {
            return Err(Error::InvalidState(Some("Cannot build.".to_owned())));
        };
        // Step 2. If checking dimensions given descriptor returns false, throw a TypeError.
        if !check_dimensions(descriptor) {
            return Err(Error::Type(c"A dimension size cannot be 0.".to_owned()));
        }
        // Step 3. If validating buffer with descriptor returns false, throw a TypeError.
        if !validate_buffer_with_descriptor(descriptor, &buffer) {
            return Err(Error::Type(
                c"Buffer size does not match the expected size for the operand descriptor."
                    .to_owned(),
            ));
        }
        // Step 4. Make graph connections:
        // Step 4.1. Let operand = creating an MLOperand given this and descriptor.
        // Step 4.2. Let bytes = getting a copy of the bytes held by the buffer source.
        let bytes = match buffer {
            MaybeSharedArrayBufferViewOrMaybeSharedArrayBuffer::ArrayBuffer(buffer) => unsafe {
                buffer.as_slice().to_vec()
            },
            MaybeSharedArrayBufferViewOrMaybeSharedArrayBuffer::ArrayBufferView(view) => unsafe {
                view.as_slice().to_vec()
            },
        };
        // Step 4.3. Add operand to this's graph's constants with bytes as value.
        let operand_id = self.channel.add_constant(
            self.builder_id.get(),
            descriptor.dataType as u32,
            &descriptor.shape,
            &bytes,
        );
        let operand = MLOperand::new(
            &self.global(),
            operand_id,
            descriptor.dataType,
            descriptor.shape.clone(),
            self,
            cx,
        );
        // Step 5. Return operand.
        Ok(operand)
    }

    /// <https://www.w3.org/TR/webnn/#dom-mlgraphbuilder-build>
    fn Build(
        &self,
        cx: &mut JSContext,
        outputs: Record<USVString, DomRoot<MLOperand>>,
    ) -> Result<Rc<Promise>, Error> {
        // The build(outputs) method steps are:
        // Step 1. Let realm be this's relevant realm.
        let global = &self.global();
        // Step 2. If this can not build, then return a new promise in realm
        // rejected with an "InvalidStateError" DOMException.
        if !self.can_build() {
            let promise = Promise::new(cx, global);
            promise.reject_error(cx, Error::InvalidState(Some("Cannot build.".into())));
            return Ok(promise);
        }
        // Step 3. If outputs is empty, then return a new promise in realm
        // rejected with a TypeError.
        if outputs.iter().next().is_none() {
            let promise = Promise::new(cx, global);
            promise.reject_error(cx, Error::Type(c"outputs is empty.".to_owned()));
            return Ok(promise);
        }
        // Step 4. For each name -> operand of outputs:
        // Step 4.1. If name is empty, then return a new promise in realm
        // rejected with a TypeError.
        // Step 4.2. If validating operand given this and operand returns
        // false, then return a new promise in realm rejected with a TypeError.
        // TODO: Step 4.3. If operand is in this's graph's inputs or constants,
        // then return a new promise in realm rejected with a TypeError.
        // TODO: Step 4.4. If operand.[[constantTensor]] exists and
        // operand.[[constantTensor]].[[isDestroyed]] is true, then return a new
        // promise in realm rejected with a TypeError.
        for (name, operand) in outputs.iter() {
            if name.0.is_empty() {
                let promise = Promise::new(cx, global);
                promise.reject_error(cx, Error::Type(c"output name is empty.".to_owned()));
                return Ok(promise);
            }
            if !validate_operand(self, &operand) {
                let promise = Promise::new(cx, global);
                promise.reject_error(
                    cx,
                    Error::Type(cformat!(
                        "Output operand is from another builder. [{}]",
                        name.0
                    )),
                );
                return Ok(promise);
            }
        }
        // Steps 5-9 (graph traversal via BFS) and step 14 (inputDescriptors)
        //
        // Note: Steps are handled by the backend (rustnn/webnn-graph) during channel.build().
        // Step 10. Let global be this's relevant global object.
        // Step 11. Let graph be a new MLGraph in realm.
        // Step 12. Set graph.[[context]] to this.[[context]].
        // Step 13. Set graph.[[isDestroyed]] to false.
        let graph = MLGraph::new(global, &*self.context.root().unwrap(), cx);
        // Step 15. For each name -> operand of outputs:
        // Step 15.1. Set graph.[[outputDescriptors]][name] to
        // operand.[[descriptor]].
        let mut output_descriptors: HashMap<String, (MLOperandDataType, Vec<u32>)> = HashMap::new();
        for (name, operand) in outputs.iter() {
            output_descriptors.insert(
                name.0.clone(),
                (operand.data_type(), operand.shape().to_vec()),
            );
        }
        graph.set_output_descriptors(output_descriptors);
        let output_pairs: Vec<(String, webnn::OperandId)> = outputs
            .iter()
            .map(|(label, op)| (label.0.clone(), op.operand_id()))
            .collect();
        // Step 16. Set this.[[hasBuilt]] to true.
        self.has_built.set(true);
        // Step 17. Let promise be a new promise in realm.
        let promise = Promise::new(cx, &self.global());
        // Step 18. Enqueue the following steps to
        // graph.[[context]].[[timeline]]:
        // Step 18.1. Run these steps, but abort when graph.[[context]] is lost:
        // Step 18.1.1. Let graphImpl be the result of converting this's graph
        // with operands, operators, inputs, and outputs's values, as well as
        // graph.[[context]].[[powerPreference]] and
        // graph.[[context]].[[accelerated]] into an implementation-defined
        // format which can be interpreted by the underlying platform.
        // Step 18.1.2. If the previous step failed, then queue an ML task with
        // global to reject promise with an "OperationError" DOMException, and
        // abort these steps.
        // Step 18.1.3. Set graph.[[implementation]] to graphImpl.
        // Step 18.1.4. Queue an ML task with global to resolve promise with
        // graph.
        // (Implementation: build runs asynchronously on the backend thread.
        // The callback resolves/rejects the promise via RoutedPromiseListener
        // on MLGraph, queued on the ml_task_source.)
        let callback = callback_promise(
            &promise,
            &*graph,
            self.global().task_manager().ml_task_source(),
        );
        self.channel
            .build(self.builder_id.get(), &output_pairs, callback);
        // TODO: Step 18.2. If aborted, then queue an ML task with global to
        // reject promise with an "InvalidStateError" DOMException.
        // Step 19. Return promise.
        Ok(promise)
    }

    /// <https://www.w3.org/TR/webnn/#dom-mlgraphbuilder-add>
    fn Add(
        &self,
        cx: &mut JSContext,
        a: &MLOperand,
        b: &MLOperand,
        options: &MLOperatorOptions,
    ) -> Result<DomRoot<MLOperand>, Error> {
        // The add(a, b, options) method steps are:
        // Step 1. Let output be the result of creating an element-wise binary
        // operation given "add", a, b, and options.
        // Step 1.1. If that throws an error, then re-throw the error.
        // Step 2. Return output.
        self.create_element_wise_binary(cx, "add", a, b, options)
    }
}
