/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use js::jsapi::HandleObject;
use script_bindings::cformat;
use script_bindings::codegen::GenericBindings::WebGPUBinding::GPUDeviceLostReason;
use webgpu_traits::{RequestDeviceError, WebGPUDeviceResponse};

use crate::dom::bindings::error::Error;
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::gpuadapter::GPUAdapter;
use crate::dom::promise::Promise;
use crate::dom::types::GPUDevice;
use crate::routed_promise::RoutedPromiseListener;

impl RoutedPromiseListener<WebGPUDeviceResponse> for GPUAdapter {
    /// <https://www.w3.org/TR/webgpu/#dom-gpuadapter-requestdevice>
    fn handle_response(
        &self,
        cx: &mut js::context::JSContext,
        response: WebGPUDeviceResponse,
        promise: &Rc<Promise>,
    ) {
        match response {
            // 3.1 Let device be a new device with the capabilities described by descriptor.
            (device_id, queue_id, Ok(descriptor)) => {
                let device = GPUDevice::new(
                    cx,
                    &self.global(),
                    self.channel(),
                    self,
                    HandleObject::null(),
                    descriptor.required_features,
                    descriptor.required_limits,
                    device_id,
                    queue_id,
                    descriptor.label.unwrap_or_default(),
                );
                self.global().add_gpu_device(&device);
                promise.resolve_native(cx, &device);
            },
            // 1. If features are not supported reject promise with a TypeError.
            (_, _, Err(RequestDeviceError::UnsupportedFeature(f))) => promise.reject_error(
                cx,
                Error::Type(cformat!(
                    "{}",
                    wgpu_core::instance::RequestDeviceError::UnsupportedFeature(f)
                )),
            ),
            // 2. If limits are not supported reject promise with an OperationError.
            (_, _, Err(RequestDeviceError::LimitsExceeded(l))) => {
                warn!(
                    "{}",
                    wgpu_core::instance::RequestDeviceError::LimitsExceeded(l)
                );
                promise.reject_error(
                    cx,
                    Error::Operation(Some("WebGPU Device Limit exceeded".to_string())),
                )
            },
            // 3. user agent otherwise cannot fulfill the request
            (device_id, queue_id, Err(RequestDeviceError::Other(e))) => {
                // TODO(sagudev): firefox always says operation error,
                // meanwhile we create "invalid" device that is not invalid in wgpu
                // causing crashes when one tries to use it
                // 1. Let device be a new device.
                let device = GPUDevice::new(
                    cx,
                    &self.global(),
                    self.channel(),
                    self,
                    HandleObject::null(),
                    wgpu_types::Features::default(),
                    wgpu_types::Limits::default(),
                    device_id,
                    queue_id,
                    String::new(),
                );
                // 2. Lose the device(device, "unknown").
                device.lose(GPUDeviceLostReason::Unknown, e);
                promise.resolve_native(cx, &device);
            },
        }
    }
}
