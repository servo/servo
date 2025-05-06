/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use dom_struct::dom_struct;
use js::jsapi::{Heap, JSObject};
use webgpu_traits::{
    RequestDeviceError, WebGPU, WebGPUAdapter, WebGPUDeviceResponse, WebGPURequest,
};
use wgpu_types::{self, MemoryHints};

use super::gpusupportedfeatures::GPUSupportedFeatures;
use super::gpusupportedlimits::set_limit;
use crate::dom::bindings::codegen::Bindings::WebGPUBinding::{
    GPUAdapterMethods, GPUDeviceDescriptor, GPUDeviceLostReason,
};
use crate::dom::bindings::error::Error;
use crate::dom::bindings::reflector::{DomGlobal, Reflector, reflect_dom_object};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::dom::types::{GPUAdapterInfo, GPUSupportedLimits};
use crate::dom::webgpu::gpudevice::GPUDevice;
use crate::dom::webgpu::gpusupportedfeatures::gpu_to_wgt_feature;
use crate::realms::InRealm;
use crate::routed_promise::{RoutedPromiseListener, route_promise};
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct GPUAdapter {
    reflector_: Reflector,
    #[ignore_malloc_size_of = "channels are hard"]
    #[no_trace]
    channel: WebGPU,
    name: DOMString,
    #[ignore_malloc_size_of = "mozjs"]
    extensions: Heap<*mut JSObject>,
    features: Dom<GPUSupportedFeatures>,
    limits: Dom<GPUSupportedLimits>,
    info: Dom<GPUAdapterInfo>,
    #[no_trace]
    adapter: WebGPUAdapter,
}

impl GPUAdapter {
    fn new_inherited(
        channel: WebGPU,
        name: DOMString,
        extensions: Heap<*mut JSObject>,
        features: &GPUSupportedFeatures,
        limits: &GPUSupportedLimits,
        info: &GPUAdapterInfo,
        adapter: WebGPUAdapter,
    ) -> Self {
        Self {
            reflector_: Reflector::new(),
            channel,
            name,
            extensions,
            features: Dom::from_ref(features),
            limits: Dom::from_ref(limits),
            info: Dom::from_ref(info),
            adapter,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        global: &GlobalScope,
        channel: WebGPU,
        name: DOMString,
        extensions: Heap<*mut JSObject>,
        features: wgpu_types::Features,
        limits: wgpu_types::Limits,
        info: wgpu_types::AdapterInfo,
        adapter: WebGPUAdapter,
        can_gc: CanGc,
    ) -> DomRoot<Self> {
        let features = GPUSupportedFeatures::Constructor(global, None, features, can_gc).unwrap();
        let limits = GPUSupportedLimits::new(global, limits, can_gc);
        let info = GPUAdapterInfo::new(global, info, can_gc);
        reflect_dom_object(
            Box::new(GPUAdapter::new_inherited(
                channel, name, extensions, &features, &limits, &info, adapter,
            )),
            global,
            can_gc,
        )
    }
}

impl Drop for GPUAdapter {
    fn drop(&mut self) {
        if let Err(e) = self
            .channel
            .0
            .send(WebGPURequest::DropAdapter(self.adapter.0))
        {
            warn!(
                "Failed to send WebGPURequest::DropAdapter({:?}) ({})",
                self.adapter.0, e
            );
        };
    }
}

impl GPUAdapterMethods<crate::DomTypeHolder> for GPUAdapter {
    /// <https://gpuweb.github.io/gpuweb/#dom-gpuadapter-requestdevice>
    fn RequestDevice(
        &self,
        descriptor: &GPUDeviceDescriptor,
        comp: InRealm,
        can_gc: CanGc,
    ) -> Rc<Promise> {
        // Step 2
        let promise = Promise::new_in_current_realm(comp, can_gc);
        let sender = route_promise(
            &promise,
            self,
            self.global().task_manager().dom_manipulation_task_source(),
        );
        let mut required_features = wgpu_types::Features::empty();
        for &ext in descriptor.requiredFeatures.iter() {
            if let Some(feature) = gpu_to_wgt_feature(ext) {
                required_features.insert(feature);
            } else {
                promise.reject_error(
                    Error::Type(format!("{} is not supported feature", ext.as_str())),
                    can_gc,
                );
                return promise;
            }
        }

        let mut required_limits = wgpu_types::Limits::default();
        if let Some(limits) = &descriptor.requiredLimits {
            for (limit, value) in (*limits).iter() {
                if !set_limit(&mut required_limits, limit.as_ref(), *value) {
                    warn!("Unknown GPUDevice limit: {limit}");
                    promise.reject_error(Error::Operation, can_gc);
                    return promise;
                }
            }
        }

        let desc = wgpu_types::DeviceDescriptor {
            required_features,
            required_limits,
            label: Some(descriptor.parent.label.to_string()),
            memory_hints: MemoryHints::MemoryUsage,
            trace: wgpu_types::Trace::Off,
        };
        let device_id = self.global().wgpu_id_hub().create_device_id();
        let queue_id = self.global().wgpu_id_hub().create_queue_id();
        let pipeline_id = self.global().pipeline_id();
        if self
            .channel
            .0
            .send(WebGPURequest::RequestDevice {
                sender,
                adapter_id: self.adapter,
                descriptor: desc,
                device_id,
                queue_id,
                pipeline_id,
            })
            .is_err()
        {
            promise.reject_error(Error::Operation, can_gc);
        }
        // Step 5
        promise
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpuadapter-isfallbackadapter>
    fn IsFallbackAdapter(&self) -> bool {
        //TODO
        false
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpuadapter-requestadapterinfo>
    fn RequestAdapterInfo(
        &self,
        unmask_hints: Vec<DOMString>,
        comp: InRealm,
        can_gc: CanGc,
    ) -> Rc<Promise> {
        // XXX: Adapter info should be generated here ...
        // Step 1
        let promise = Promise::new_in_current_realm(comp, can_gc);
        // Step 4
        if !unmask_hints.is_empty() {
            todo!("unmaskHints on RequestAdapterInfo");
        }
        promise.resolve_native(&*self.info, can_gc);
        // Step 5
        promise
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpuadapter-features>
    fn Features(&self) -> DomRoot<GPUSupportedFeatures> {
        DomRoot::from_ref(&self.features)
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpuadapter-limits>
    fn Limits(&self) -> DomRoot<GPUSupportedLimits> {
        DomRoot::from_ref(&self.limits)
    }
}

impl RoutedPromiseListener<WebGPUDeviceResponse> for GPUAdapter {
    /// <https://www.w3.org/TR/webgpu/#dom-gpuadapter-requestdevice>
    fn handle_response(
        &self,
        response: WebGPUDeviceResponse,
        promise: &Rc<Promise>,
        can_gc: CanGc,
    ) {
        match response {
            // 3.1 Let device be a new device with the capabilities described by descriptor.
            (device_id, queue_id, Ok(descriptor)) => {
                let device = GPUDevice::new(
                    &self.global(),
                    self.channel.clone(),
                    self,
                    Heap::default(),
                    descriptor.required_features,
                    descriptor.required_limits,
                    device_id,
                    queue_id,
                    descriptor.label.unwrap_or_default(),
                    can_gc,
                );
                self.global().add_gpu_device(&device);
                promise.resolve_native(&device, can_gc);
            },
            // 1. If features are not supported reject promise with a TypeError.
            (_, _, Err(RequestDeviceError::UnsupportedFeature(f))) => promise.reject_error(
                Error::Type(
                    wgpu_core::instance::RequestDeviceError::UnsupportedFeature(f).to_string(),
                ),
                can_gc,
            ),
            // 2. If limits are not supported reject promise with an OperationError.
            (_, _, Err(RequestDeviceError::LimitsExceeded(l))) => {
                warn!(
                    "{}",
                    wgpu_core::instance::RequestDeviceError::LimitsExceeded(l)
                );
                promise.reject_error(Error::Operation, can_gc)
            },
            // 3. user agent otherwise cannot fulfill the request
            (device_id, queue_id, Err(RequestDeviceError::Other(e))) => {
                // 1. Let device be a new device.
                let device = GPUDevice::new(
                    &self.global(),
                    self.channel.clone(),
                    self,
                    Heap::default(),
                    wgpu_types::Features::default(),
                    wgpu_types::Limits::default(),
                    device_id,
                    queue_id,
                    String::new(),
                    can_gc,
                );
                // 2. Lose the device(device, "unknown").
                device.lose(GPUDeviceLostReason::Unknown, e, can_gc);
                promise.resolve_native(&device, can_gc);
            },
        }
    }
}
