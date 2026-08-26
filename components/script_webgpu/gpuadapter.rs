/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use dom_struct::dom_struct;
use js::jsapi::{HandleObject, Heap, JSObject};
use js::realm::CurrentRealm;
use jstraceable_derive::JSTraceable;
use log::warn;
use malloc_size_of_derive::MallocSizeOf;
use script_bindings::codegen::GenericBindings::WebGPUBinding::{
    GPUAdapterMethods, GPUAdapterWrap, GPUDeviceDescriptor,
};
use script_bindings::interfaces::{GlobalScopeHelpers, PromiseHelpers};
use script_bindings::like::Setlike;
use script_bindings::reflector::{DomGlobalGeneric, Reflector, reflect_dom_object_with_wrap};
use script_bindings::{DomTypes, cformat};
use webgpu_traits::{WebGPU, WebGPUAdapter, WebGPURequest};
use wgpu_types::{AdapterInfo, ExperimentalFeatures, MemoryHints};

use crate::dom::bindings::error::Error;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::gpuadapterinfo::GPUAdapterInfo;
use crate::gpusupportedfeatures::{GPUSupportedFeatures, gpu_to_wgt_feature};
use crate::gpusupportedlimits::{GPUSupportedLimits, set_limit};
use crate::traits::{WebGPUGlobalTrait, WebGPUPromiseTrait};

#[derive(JSTraceable, MallocSizeOf)]
struct DroppableGPUAdapter {
    #[no_trace]
    channel: WebGPU,
    #[no_trace]
    adapter: WebGPUAdapter,
}

impl Drop for DroppableGPUAdapter {
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

#[dom_struct]
pub struct GPUAdapter<D: DomTypes> {
    reflector_: Reflector,
    name: DOMString,
    #[ignore_malloc_size_of = "mozjs"]
    extensions: Heap<*mut JSObject>,
    features: Dom<GPUSupportedFeatures<D>>,
    limits: Dom<GPUSupportedLimits<D>>,
    info: Dom<GPUAdapterInfo<D>>,
    droppable: DroppableGPUAdapter,
}

impl<D> GPUAdapter<D>
where
    D: DomTypes<
            GPUAdapter = GPUAdapter<D>,
            GPUAdapterInfo = GPUAdapterInfo<D>,
            GPUSupportedFeatures = GPUSupportedFeatures<D>,
            GPUSupportedLimits = GPUSupportedLimits<D>,
        >,
{
    fn new_inherited(
        channel: WebGPU,
        name: DOMString,
        features: &GPUSupportedFeatures<D>,
        limits: &GPUSupportedLimits<D>,
        info: &GPUAdapterInfo<D>,
        adapter: WebGPUAdapter,
    ) -> Self {
        Self {
            reflector_: Reflector::new(),
            name,
            extensions: Heap::default(),
            features: Dom::from_ref(features),
            limits: Dom::from_ref(limits),
            info: Dom::from_ref(info),
            droppable: DroppableGPUAdapter { channel, adapter },
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new(
        cx: &mut js::context::JSContext,
        global: &D::GlobalScope,
        channel: WebGPU,
        name: DOMString,
        extensions: HandleObject,
        features: wgpu_types::Features,
        limits: wgpu_types::Limits,
        info: wgpu_types::AdapterInfo,
        adapter: WebGPUAdapter,
    ) -> DomRoot<Self> {
        let features = GPUSupportedFeatures::Constructor(cx, global, None, features).unwrap();
        let limits = GPUSupportedLimits::new(cx, global, limits);
        let info = GPUAdapter::create_adapter_info(cx, global, info, &features);
        let dom_root = reflect_dom_object_with_wrap::<D, _, _>(
            Box::new(GPUAdapter::new_inherited(
                channel, name, &features, &limits, &info, adapter,
            )),
            global,
            cx,
            GPUAdapterWrap::<D>,
        );
        dom_root.extensions.set(*extensions);
        dom_root
    }

    /// <https://gpuweb.github.io/gpuweb/#abstract-opdef-new-adapter-info>
    fn create_adapter_info(
        cx: &mut js::context::JSContext,
        global: &D::GlobalScope,
        info: AdapterInfo,
        features: &GPUSupportedFeatures<D>,
    ) -> DomRoot<GPUAdapterInfo<D>> {
        // Step 2. If the vendor is known, set adapterInfo.vendor to the name of adapter’s vendor as
        // a normalized identifier string. To preserve privacy, the user agent may instead set
        // adapterInfo.vendor to the empty string or a reasonable approximation of the vendor as a
        // normalized identifier string.
        let vendor = if info.vendor != 0 {
            info.vendor.to_string().into()
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
            info.device.to_string().into()
        } else {
            DOMString::new()
        };

        // Step 5. If a description is known, set adapterInfo.description to a description of the
        // adapter as reported by the driver. To preserve privacy, the user agent may instead set
        // adapterInfo.description to the empty string or a reasonable approximation of a
        // description.
        let description = info.name.clone().into();

        // Step 6. If "subgroups" is supported, set subgroupMinSize to the smallest supported
        // subgroup size. Otherwise, set this value to 4.
        // Step 7. If "subgroups" is supported, set subgroupMaxSize to the largest supported
        // subgroup size. Otherwise, set this value to 128.
        let (subgroup_min_size, subgroup_max_size) = if features.has(cx, "subgroups".into()) {
            (info.subgroup_min_size, info.subgroup_max_size)
        } else {
            (4, 128)
        };

        // Step 8. Set adapterInfo.isFallbackAdapter to adapter.[[fallback]].
        let is_fallback_adapter = info.device_type == wgpu_types::DeviceType::Cpu;

        // Step 1. Let adapterInfo be a new GPUAdapterInfo.
        GPUAdapterInfo::new(
            cx,
            global,
            vendor,
            architecture,
            device,
            description,
            subgroup_min_size,
            subgroup_max_size,
            is_fallback_adapter,
        )
    }

    pub fn channel(&self) -> WebGPU {
        self.droppable.channel.clone()
    }

    fn global(&self) -> DomRoot<D::GlobalScope> {
        <Self as DomGlobalGeneric<D>>::global_from_reflector(self)
    }
}

impl<D> GPUAdapterMethods<D> for GPUAdapter<D>
where
    D: DomTypes<
            GPUAdapter = GPUAdapter<D>,
            GPUAdapterInfo = GPUAdapterInfo<D>,
            GPUSupportedFeatures = GPUSupportedFeatures<D>,
            GPUSupportedLimits = GPUSupportedLimits<D>,
        >,
    D::Promise: WebGPUPromiseTrait<D> + PromiseHelpers<D>,
    D::GlobalScope: WebGPUGlobalTrait + GlobalScopeHelpers<D>,
{
    /// <https://gpuweb.github.io/gpuweb/#dom-gpuadapter-requestdevice>
    fn RequestDevice(
        &self,
        cx: &mut CurrentRealm<'_>,
        descriptor: &GPUDeviceDescriptor,
    ) -> Rc<D::Promise> {
        // Step 2
        let promise = D::Promise::new_in_realm(cx);

        let callback = WebGPUPromiseTrait::<D>::callback_promise_adapter(&promise, self);
        let mut required_features = wgpu_types::Features::empty();
        for &ext in descriptor.requiredFeatures.iter() {
            if let Some(feature) = gpu_to_wgt_feature(ext) {
                required_features.insert(feature);
            } else {
                promise.reject_error(
                    cx,
                    Error::Type(cformat!("{} is not supported feature", ext.as_str())),
                );
                return promise;
            }
        }

        let mut required_limits = wgpu_types::Limits::default();
        if let Some(limits) = &descriptor.requiredLimits {
            for (limit, value) in (*limits).iter() {
                if !set_limit(&mut required_limits, &limit.str(), *value) {
                    warn!("Unknown GPUDevice limit: {limit}");
                    promise.reject_error(
                        cx,
                        Error::Operation(Some(format!("Unknown GPUDevice limit: {limit}"))),
                    );
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
            experimental_features: ExperimentalFeatures::disabled(),
        };
        let device_id = self.global().global_wgpu_id_hub().create_device_id();
        let queue_id = self.global().global_wgpu_id_hub().create_queue_id();
        let pipeline_id = self.global().pipeline_id();
        if self
            .droppable
            .channel
            .0
            .send(WebGPURequest::RequestDevice {
                sender: callback,
                adapter_id: self.droppable.adapter,
                descriptor: desc,
                device_id,
                queue_id,
                pipeline_id,
            })
            .is_err()
        {
            promise.reject_error(
                cx,
                Error::Operation(Some("Could not Request GPU Device".to_string())),
            );
        }
        // Step 5
        promise
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpuadapter-features>
    fn Features(&self) -> DomRoot<GPUSupportedFeatures<D>> {
        DomRoot::from_ref(&self.features)
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpuadapter-limits>
    fn Limits(&self) -> DomRoot<GPUSupportedLimits<D>> {
        DomRoot::from_ref(&self.limits)
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpuadapter-info>
    fn Info(&self) -> DomRoot<GPUAdapterInfo<D>> {
        DomRoot::from_ref(&self.info)
    }
}
