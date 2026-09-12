/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use script_bindings::codegen::GenericBindings::WebGPUBinding::GPUPipelineErrorReason;
use script_bindings::error::Error;
use webgpu_traits::{
    PopError, WebGPUComputePipeline, WebGPUComputePipelineResponse, WebGPUPoppedErrorScopeResponse,
    WebGPURenderPipeline, WebGPURenderPipelineResponse,
};

use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::promise::RootedPromise;
use crate::dom::types::{
    GPUComputePipeline, GPUDevice, GPUError, GPUPipelineError, GPURenderPipeline,
};
use crate::routed_promise::RoutedPromiseListener;

impl RoutedPromiseListener<WebGPUPoppedErrorScopeResponse> for GPUDevice {
    fn handle_response(
        &self,
        cx: &mut js::context::JSContext,
        response: WebGPUPoppedErrorScopeResponse,
        promise: &RootedPromise,
    ) {
        match response {
            Ok(None) | Err(PopError::Lost) => promise.resolve_native(cx, &None::<Option<GPUError>>),
            Err(PopError::Empty) => promise.reject_error(
                cx,
                Error::Operation(Some("Error scope stack is empty".into())),
            ),
            Ok(Some(error)) => {
                let error = GPUError::from_error(cx, &self.global(), error);
                promise.resolve_native(cx, &error);
            },
        }
    }
}

impl RoutedPromiseListener<WebGPUComputePipelineResponse> for GPUDevice {
    fn handle_response(
        &self,
        cx: &mut js::context::JSContext,
        response: WebGPUComputePipelineResponse,
        promise: &RootedPromise,
    ) {
        match response {
            Ok(pipeline) => {
                let gpu_compute_pipeline = GPUComputePipeline::new(
                    cx,
                    &self.global(),
                    WebGPUComputePipeline(pipeline.id),
                    pipeline.label.into(),
                    self,
                );
                promise.resolve_native(cx, &gpu_compute_pipeline)
            },
            Err(webgpu_traits::Error::Validation(msg)) => {
                let gpu_pipeline_error = GPUPipelineError::new(
                    cx,
                    &self.global(),
                    msg.into(),
                    GPUPipelineErrorReason::Validation,
                );
                promise.reject_native(cx, &gpu_pipeline_error)
            },
            Err(webgpu_traits::Error::OutOfMemory(msg) | webgpu_traits::Error::Internal(msg)) => {
                let gpu_pipeline_error = GPUPipelineError::new(
                    cx,
                    &self.global(),
                    msg.into(),
                    GPUPipelineErrorReason::Internal,
                );
                promise.reject_native(cx, &gpu_pipeline_error)
            },
        }
    }
}

impl RoutedPromiseListener<WebGPURenderPipelineResponse> for GPUDevice {
    fn handle_response(
        &self,
        cx: &mut js::context::JSContext,
        response: WebGPURenderPipelineResponse,
        promise: &RootedPromise,
    ) {
        match response {
            Ok(pipeline) => {
                let gpu_pipeline = GPURenderPipeline::new(
                    cx,
                    &self.global(),
                    WebGPURenderPipeline(pipeline.id),
                    pipeline.label.into(),
                    self,
                );
                promise.resolve_native(cx, &gpu_pipeline)
            },
            Err(webgpu_traits::Error::Validation(msg)) => {
                let pipeline_error = GPUPipelineError::new(
                    cx,
                    &self.global(),
                    msg.into(),
                    GPUPipelineErrorReason::Validation,
                );

                promise.reject_native(cx, &pipeline_error)
            },
            Err(webgpu_traits::Error::OutOfMemory(msg) | webgpu_traits::Error::Internal(msg)) => {
                let pipeline_error = GPUPipelineError::new(
                    cx,
                    &self.global(),
                    msg.into(),
                    GPUPipelineErrorReason::Internal,
                );
                promise.reject_native(cx, &pipeline_error)
            },
        }
    }
}
