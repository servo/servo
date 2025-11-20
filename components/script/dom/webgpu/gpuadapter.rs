/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use dom_struct::dom_struct;
use js::jsapi::{HandleObject, Heap, JSObject};
use webgpu_traits::{
    RequestDeviceError, WebGPU, WebGPUAdapter, WebGPUDeviceResponse, WebGPURequest,
};
use wgpu_types::{self, AdapterInfo, MemoryHints};

use super::gpusupportedfeatures::GPUSupportedFeatures;
use super::gpusupportedlimits::set_limit;
use crate::dom::bindings::codegen::Bindings::WebGPUBinding::{
    GPUAdapterMethods, GPUDeviceDescriptor, GPUDeviceLostReason,
};
use crate::dom::bindings::error::Error;
use crate::dom::bindings::like::Setlike;
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
        features: &GPUSupportedFeatures,
        limits: &GPUSupportedLimits,
        info: &GPUAdapterInfo,
        adapter: WebGPUAdapter,
    ) -> Self {
        Self {
            reflector_: Reflector::new(),
            channel,
            name,
            extensions: Heap::default(),
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
        extensions: HandleObject,
        features: wgpu_types::Features,
        limits: wgpu_types::Limits,
        info: wgpu_types::AdapterInfo,
        adapter: WebGPUAdapter,
        can_gc: CanGc,
    ) -> DomRoot<Self> {
        let features = GPUSupportedFeatures::Constructor(global, None, features, can_gc).unwrap();
        let limits = GPUSupportedLimits::new(global, limits, can_gc);
        let info = GPUAdapter::create_adapter_info(global, info, &features, &limits, can_gc);
        let dom_root = reflect_dom_object(
            Box::new(GPUAdapter::new_inherited(
                channel, name, &features, &limits, &info, adapter,
            )),
            global,
            can_gc,
        );
        dom_root.extensions.set(*extensions);
        dom_root
    }

    /// <https://gpuweb.github.io/gpuweb/#abstract-opdef-new-adapter-info>
    fn create_adapter_info(
        global: &GlobalScope,
        info: AdapterInfo,
        features: &GPUSupportedFeatures,
        limits: &GPUSupportedLimits,
        can_gc: CanGc,
    ) -> DomRoot<GPUAdapterInfo> {
        // Step 2. If the vendor is known, set adapterInfo.vendor to the name of adapter’s vendor as
        // a normalized identifier string. To preserve privacy, the user agent may instead set
        // adapterInfo.vendor to the empty string or a reasonable approximation of the vendor as a
        // normalized identifier string.
        let vendor = if info.vendor != 0 {
            DOMString::from_string(info.vendor.to_string())
        } else {
            DOMString::new()
        };

        // Step 3. If the architecture is known, set adapterInfo.architecture to a normalized
        // identifier string representing the family or class of adapters to which adapter belongs.
        // To preserve privacy, the user agent may instead set adapterInfo.architecture to the empty
        // string or a reasonable approximation of the architecture as a normalized identifier
        // string.
        // TODO: AdapterInfo::architecture missing
        // https://github.com/gfx-rs/wgpu/issues/2170
        let architecture = DOMString::new();

        // Step 4. If the device is known, set adapterInfo.device to a normalized identifier string
        // representing a vendor-specific identifier for adapter. To preserve privacy, the user
        // agent may instead set adapterInfo.device to to the empty string or a reasonable
        // approximation of a vendor-specific identifier as a normalized identifier string.
        let device = if info.device != 0 {
            DOMString::from_string(info.device.to_string())
        } else {
            DOMString::new()
        };

        // Step 5. If a description is known, set adapterInfo.description to a description of the
        // adapter as reported by the driver. To preserve privacy, the user agent may instead set
        // adapterInfo.description to the empty string or a reasonable approximation of a
        // description.
        let description = DOMString::from_string(info.name.clone());

        // Step 6. If "subgroups" is supported, set subgroupMinSize to the smallest supported
        // subgroup size. Otherwise, set this value to 4.
        // Step 7. If "subgroups" is supported, set subgroupMaxSize to the largest supported
        // subgroup size. Otherwise, set this value to 128.
        let (subgroup_min_size, subgroup_max_size) = if features.has("subgroups".into()) {
            (
                limits.wgpu_limits().min_subgroup_size,
                limits.wgpu_limits().max_subgroup_size,
            )
        } else {
            (4, 128)
        };

        // Step 8. Set adapterInfo.isFallbackAdapter to adapter.[[fallback]].
        let is_fallback_adapter = info.device_type == wgpu_types::DeviceType::Cpu;

        // Step 1. Let adapterInfo be a new GPUAdapterInfo.
        GPUAdapterInfo::new(
            global,
            vendor,
            architecture,
            device,
            description,
            subgroup_min_size,
            subgroup_max_size,
            is_fallback_adapter,
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
                if !set_limit(&mut required_limits, &limit.str(), *value) {
                    warn!("Unknown GPUDevice limit: {limit}");
                    promise.reject_error(Error::Operation(None), can_gc);
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
            promise.reject_error(Error::Operation(None), can_gc);
        }
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

    /// <https://gpuweb.github.io/gpuweb/#dom-gpuadapter-info>
    fn Info(&self) -> DomRoot<GPUAdapterInfo> {
        DomRoot::from_ref(&self.info)
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
                    HandleObject::null(),
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
                promise.reject_error(Error::Operation(None), can_gc)
            },
            // 3. user agent otherwise cannot fulfill the request
            (device_id, queue_id, Err(RequestDeviceError::Other(e))) => {
                // 1. Let device be a new device.
                let device = GPUDevice::new(
                    &self.global(),
                    self.channel.clone(),
                    self,
                    HandleObject::null(),
                    wgpu_types::Features::default(),
                    wgpu_types::Limits::default(),
                    device_id,
                    queue_id,
                    String::new(),
                    can_gc,
                );
                // 2. Lose the device(device, "unknown").
                device.lose(GPUDeviceLostReason::Unknown, e);
                promise.resolve_native(&device, can_gc);
            },
        }
    }
}
