/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::marker::PhantomData;
use std::rc::Rc;

use dom_struct::dom_struct;
use js::context::JSContext;
use js::realm::CurrentRealm;
use jstraceable_derive::JSTraceable;
use malloc_size_of_derive::MallocSizeOf;
use script_bindings::DomTypes;
use script_bindings::codegen::GenericBindings::WebGPUBinding::{
    GPUMethods, GPUPowerPreference, GPURequestAdapterOptions, GPUTextureFormat, GPUWrap,
};
use script_bindings::dom::MutNullableDom;
use script_bindings::interfaces::{GlobalScopeHelpers, PromiseHelpers};
use script_bindings::reflector::{DomGlobalGeneric, Reflector, reflect_dom_object_with_wrap};
use script_bindings::root::DomRoot;
use servo_constellation_traits::ScriptToConstellationMessage;
use wgpu_types::PowerPreference;

use super::wgsllanguagefeatures::WGSLLanguageFeatures;
use crate::dom::bindings::error::Error;
use crate::gpuadapter::GPUAdapter;
use crate::traits::{WebGPUGlobalTrait, WebGPUPromiseTrait};

#[dom_struct]
pub struct GPU<D: DomTypes> {
    reflector_: Reflector,
    /// Same object for <https://www.w3.org/TR/webgpu/#dom-gpu-wgsllanguagefeatures>
    wgsl_language_features: MutNullableDom<WGSLLanguageFeatures<D>>,
    #[no_trace = "PhantomData does not exist"]
    phantom: PhantomData<D>,
}

impl<D> GPU<D>
where
    D: DomTypes<GPU = GPU<D>>,
{
    pub(crate) fn new_inherited() -> GPU<D> {
        GPU {
            reflector_: Reflector::new(),
            wgsl_language_features: MutNullableDom::default(),
            phantom: PhantomData,
        }
    }

    pub fn new(cx: &mut JSContext, global: &D::GlobalScope) -> DomRoot<GPU<D>> {
        reflect_dom_object_with_wrap::<D, _, _>(
            Box::new(GPU::new_inherited()),
            global,
            cx,
            GPUWrap::<D>,
        )
    }

    fn global(&self) -> DomRoot<D::GlobalScope> {
        <D::GPU as DomGlobalGeneric<D>>::global_from_reflector(self)
    }
}

impl<D: DomTypes> GPUMethods<D> for GPU<D>
where
    D: DomTypes<GPU = GPU<D>, WGSLLanguageFeatures = WGSLLanguageFeatures<D>>,
    D::Promise: PromiseHelpers<D> + WebGPUPromiseTrait<D>,
    D::GPU: DomGlobalGeneric<D>,
    D::GlobalScope: WebGPUGlobalTrait,
{
    /// <https://gpuweb.github.io/gpuweb/#dom-gpu-requestadapter>
    fn RequestAdapter(
        &self,
        cx: &mut CurrentRealm,
        options: &GPURequestAdapterOptions,
    ) -> Rc<D::Promise> {
        let global = &self.global();
        // 1. Let promise be a new promise.
        let promise = D::Promise::new_in_realm(cx);
        let callback = D::Promise::callback_promise_gpu(&promise, self);

        let power_preference = match options.powerPreference {
            Some(GPUPowerPreference::Low_power) => PowerPreference::LowPower,
            Some(GPUPowerPreference::High_performance) => PowerPreference::HighPerformance,
            None => PowerPreference::default(),
        };
        let ids = global.global_wgpu_id_hub().create_adapter_id();

        // 3. Issue the initialization steps on the Device timeline of this

        /*
        We do some steps here to avoid IPC round-trips
        1. options.featureLevel must be a feature level string.
        If any are unmet
            Let adapter be null, issue the resolution steps on contentTimeline, and return.
        If adapter is null:
            Resolve promise with null.
        */
        match &*options.featureLevel.str() {
            "core" => {},
            "compatibility" => {
                // Set options.featureLevel to "compatibility" if the user agent chooses to support it, or "core" if not.
                // and wgpu does not support "compatibility" yet so we return core for now
            },
            _ => {
                promise.resolve_native(cx, &None::<GPUAdapter<D>>);
                return promise;
            },
        }
        let script_to_constellation_chan = global.script_to_constellation_chan();
        if script_to_constellation_chan
            .send(ScriptToConstellationMessage::RequestAdapter(
                callback,
                wgpu_core::instance::RequestAdapterOptions {
                    power_preference,
                    compatible_surface: None,
                    force_fallback_adapter: options.forceFallbackAdapter,
                    apply_limit_buckets: false,
                },
                ids,
            ))
            .is_err()
        {
            promise.reject_error(
                cx,
                Error::Operation(Some("Could not request adapter".into())),
            );
        }
        // 4. Return promise
        promise
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpu-getpreferredcanvasformat>
    fn GetPreferredCanvasFormat(&self) -> GPUTextureFormat {
        // From https://github.com/mozilla-firefox/firefox/blob/24d49101ce17b78c3ba1217d00297fe2891be6b3/dom/webgpu/Instance.h#L68
        if cfg!(target_os = "android") {
            GPUTextureFormat::Rgba8unorm
        } else {
            GPUTextureFormat::Bgra8unorm
        }
    }

    /// <https://www.w3.org/TR/webgpu/#dom-gpu-wgsllanguagefeatures>
    fn WgslLanguageFeatures(
        &self,
        cx: &mut js::context::JSContext,
    ) -> DomRoot<WGSLLanguageFeatures<D>> {
        self.wgsl_language_features
            .or_init(|| WGSLLanguageFeatures::new(cx, &*self.global(), None))
    }
}
