/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use dom_struct::dom_struct;
use js::jsapi::{Heap, JSObject};
use webgpu::wgc::instance::RequestDeviceError;
use webgpu::wgt::MemoryHints;
use webgpu::{wgt, WebGPU, WebGPUAdapter, WebGPURequest, WebGPUResponse};

use super::gpusupportedfeatures::GPUSupportedFeatures;
use super::gpusupportedlimits::set_limit;
use crate::dom::bindings::codegen::Bindings::WebGPUBinding::{
    GPUAdapterMethods, GPUDeviceDescriptor, GPUDeviceLostReason,
};
use crate::dom::bindings::error::Error;
use crate::dom::bindings::reflector::{reflect_dom_object, DomGlobal, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::dom::types::{GPUAdapterInfo, GPUSupportedLimits};
use crate::dom::webgpu::gpu::{response_async, AsyncWGPUListener};
use crate::dom::webgpu::gpudevice::GPUDevice;
use crate::dom::webgpu::gpusupportedfeatures::gpu_to_wgt_feature;
use crate::realms::InRealm;
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
        features: wgt::Features,
        limits: wgt::Limits,
        info: wgt::AdapterInfo,
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
        let sender = response_async(&promise, self);
        let mut required_features = wgt::Features::empty();
        for &ext in descriptor.requiredFeatures.iter() {
            if let Some(feature) = gpu_to_wgt_feature(ext) {
                required_features.insert(feature);
            } else {
                promise.reject_error(Error::Type(format!(
                    "{} is not supported feature",
                    ext.as_str()
                )));
                return promise;
            }
        }

        let mut required_limits = wgt::Limits::default();
        if let Some(limits) = &descriptor.requiredLimits {
            for (limit, value) in (*limits).iter() {
                if !set_limit(&mut required_limits, limit.as_ref(), *value) {
                    warn!("Unknown GPUDevice limit: {limit}");
                    promise.reject_error(Error::Operation);
                    return promise;
                }
            }
        }

        let desc = wgt::DeviceDescriptor {
            required_features,
            required_limits,
            label: Some(descriptor.parent.label.to_string()),
            memory_hints: MemoryHints::MemoryUsage,
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
            promise.reject_error(Error::Operation);
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
        promise.resolve_native(&*self.info);
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

impl AsyncWGPUListener for GPUAdapter {
    fn handle_response(&self, response: WebGPUResponse, promise: &Rc<Promise>, can_gc: CanGc) {
        match response {
            WebGPUResponse::Device((device_id, queue_id, Ok(descriptor))) => {
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
                promise.resolve_native(&device);
            },
            WebGPUResponse::Device((_, _, Err(RequestDeviceError::UnsupportedFeature(f)))) => {
                promise.reject_error(Error::Type(
                    RequestDeviceError::UnsupportedFeature(f).to_string(),
                ))
            },
            WebGPUResponse::Device((_, _, Err(RequestDeviceError::LimitsExceeded(_)))) => {
                promise.reject_error(Error::Operation)
            },
            WebGPUResponse::Device((device_id, queue_id, Err(e))) => {
                let device = GPUDevice::new(
                    &self.global(),
                    self.channel.clone(),
                    self,
                    Heap::default(),
                    wgt::Features::default(),
                    wgt::Limits::default(),
                    device_id,
                    queue_id,
                    String::new(),
                    can_gc,
                );
                device.lose(GPUDeviceLostReason::Unknown, e.to_string());
                promise.resolve_native(&device);
            },
            WebGPUResponse::None => unreachable!("Failed to get a response for RequestDevice"),
            _ => unreachable!("GPUAdapter received wrong WebGPUResponse"),
        }
    }
}
