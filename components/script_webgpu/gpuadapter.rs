/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::jsapi::{HandleObject, Heap, JSObject};
use js::realm::CurrentRealm;
use jstraceable_derive::JSTraceable;
use log::warn;
use malloc_size_of_derive::MallocSizeOf;
use script_bindings::callback::CallbackContainer;
use script_bindings::codegen::GenericBindings::EventHandlerBinding::EventHandlerNonNull;
use script_bindings::codegen::GenericBindings::WebGPUBinding::{
    GPUAdapterMethods, GPUAdapterWrap, GPUDeviceDescriptor, GPUDeviceLostReason,
};
use script_bindings::interfaces::{GlobalScopeHelpers, PromiseHelpers};
use script_bindings::like::Setlike;
use script_bindings::reflector::{DomGlobalGeneric, Reflector, reflect_dom_object_with_wrap};
use script_bindings::routed_promise::RoutedPromiseListener;
use script_bindings::{DomTypes, cformat};
use webgpu_traits::{
    AdapterInfo, DeviceDescriptor, DeviceType, ExperimentalFeatures, Features, Limits, MemoryHints,
    RequestDeviceError, Trace, WebGPU, WebGPUAdapter, WebGPUDeviceResponse, WebGPURequest,
};

use crate::dom::bindings::error::Error;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::gpuadapterinfo::GPUAdapterInfo;
use crate::gpudevice::GPUDevice;
use crate::gpusupportedfeatures::{GPUSupportedFeatures, gpu_to_wgt_feature};
use crate::gpusupportedlimits::{GPUSupportedLimits, set_limit};
use crate::traits::{
    Equivalence, WebGPUGlobalTrait, WebGPUPromise, WebGPUPromiseCallbackTrait,
    WebGPURootedPromiseTrait,
};

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
    D: Equivalence,
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
        features: Features,
        limits: Limits,
        info: AdapterInfo,
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
        let (subgroup_min_size, subgroup_max_size) =
            if features.has(cx, DOMString::from_static("subgroups")) {
                (info.subgroup_min_size, info.subgroup_max_size)
            } else {
                (4, 128)
            };

        // Step 8. Set adapterInfo.isFallbackAdapter to adapter.[[fallback]].
        let is_fallback_adapter = info.device_type == DeviceType::Cpu;

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

    /// duplicates GPUAdapter::Info but it has reduced bounds
    pub fn info(&self) -> DomRoot<GPUAdapterInfo<D>> {
        DomRoot::from_ref(&self.info)
    }
}

impl<D> GPUAdapterMethods<D> for GPUAdapter<D>
where
    D: Equivalence,
    <D::Promise as PromiseHelpers<D>>::StackRoot: WebGPUPromise<D>,
{
    /// <https://gpuweb.github.io/gpuweb/#dom-gpuadapter-requestdevice>
    fn RequestDevice(
        &self,
        cx: &mut CurrentRealm<'_>,
        descriptor: &GPUDeviceDescriptor,
    ) -> <D::Promise as PromiseHelpers<D>>::StackRoot {
        // Step 2
        let promise = D::Promise::new_in_realm_rooted(cx);

        let callback = promise.callback_promise_dom_manipulation_task_source(self);
        let mut required_features = Features::empty();
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

        let mut required_limits = Limits::default();
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

        let desc = DeviceDescriptor {
            required_features,
            required_limits,
            label: Some(descriptor.parent.label.to_string()),
            memory_hints: MemoryHints::MemoryUsage,
            trace: Trace::Off,
            experimental_features: ExperimentalFeatures::disabled(),
        };
        let device_id = self
            .global_from_reflector()
            .global_wgpu_id_hub()
            .create_device_id();
        let queue_id = self
            .global_from_reflector()
            .global_wgpu_id_hub()
            .create_queue_id();
        let pipeline_id = self.global_from_reflector().pipeline_id();
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

impl<D: Equivalence> RoutedPromiseListener<D, WebGPUDeviceResponse> for GPUAdapter<D>
where
    Self: DomGlobalGeneric<D>,
    <D::Promise as PromiseHelpers<D>>::StackRoot: WebGPURootedPromiseTrait<D>,
    EventHandlerNonNull<D>: CallbackContainer<D>,
{
    /// <https://www.w3.org/TR/webgpu/#dom-gpuadapter-requestdevice>
    fn handle_response(
        &self,
        cx: &mut js::context::JSContext,
        response: WebGPUDeviceResponse,
        promise: &<D::Promise as PromiseHelpers<D>>::StackRoot,
    ) {
        match response {
            // 3.1 Let device be a new device with the capabilities described by descriptor.
            (device_id, queue_id, Ok(descriptor)) => {
                let device = GPUDevice::<D>::new(
                    cx,
                    &self.global_from_reflector(),
                    self.channel(),
                    self,
                    HandleObject::null(),
                    descriptor.required_features,
                    descriptor.required_limits,
                    device_id,
                    queue_id,
                    descriptor.label.unwrap_or_default(),
                );
                self.global_from_reflector().add_webgpu_device(&device);
                promise.resolve_native(cx, &device);
            },
            // 1. If features are not supported reject promise with a TypeError.
            (_, _, Err(RequestDeviceError::UnsupportedFeature(f))) => promise.reject_error(
                cx,
                Error::Type(cformat!("Unsupported features were requested: {}", f)),
            ),
            // 2. If limits are not supported reject promise with an OperationError.
            (_, _, Err(RequestDeviceError::LimitsExceeded(l))) => {
                warn!("{}", l);
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
                let device = GPUDevice::<D>::new(
                    cx,
                    &self.global_from_reflector(),
                    self.channel(),
                    self,
                    HandleObject::null(),
                    Features::default(),
                    Limits::default(),
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
