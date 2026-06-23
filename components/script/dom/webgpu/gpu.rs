/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use dom_struct::dom_struct;
use js::context::JSContext;
use js::jsapi::HandleObject;
use js::realm::CurrentRealm;
use script_bindings::reflector::{Reflector, reflect_dom_object_with_cx};
use servo_constellation_traits::ScriptToConstellationMessage;
use webgpu_traits::WebGPUAdapterResponse;
use wgpu_types::PowerPreference;

use super::wgsllanguagefeatures::WGSLLanguageFeatures;
use crate::dom::bindings::codegen::Bindings::WebGPUBinding::{
    GPUMethods, GPUPowerPreference, GPURequestAdapterOptions, GPUTextureFormat,
};
use crate::dom::bindings::error::Error;
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::dom::webgpu::gpuadapter::GPUAdapter;
use crate::routed_promise::{RoutedPromiseListener, callback_promise};

#[dom_struct]
#[expect(clippy::upper_case_acronyms)]
pub(crate) struct GPU {
    reflector_: Reflector,
    /// Same object for <https://www.w3.org/TR/webgpu/#dom-gpu-wgsllanguagefeatures>
    wgsl_language_features: MutNullableDom<WGSLLanguageFeatures>,
}

impl GPU {
    pub(crate) fn new_inherited() -> GPU {
        GPU {
            reflector_: Reflector::new(),
            wgsl_language_features: MutNullableDom::default(),
        }
    }

    pub(crate) fn new(cx: &mut JSContext, global: &GlobalScope) -> DomRoot<GPU> {
        reflect_dom_object_with_cx(Box::new(GPU::new_inherited()), global, cx)
    }
}

impl GPUMethods<crate::DomTypeHolder> for GPU {
    /// <https://gpuweb.github.io/gpuweb/#dom-gpu-requestadapter>
    fn RequestAdapter(
        &self,
        cx: &mut CurrentRealm,
        options: &GPURequestAdapterOptions,
    ) -> Rc<Promise> {
        let global = &self.global();
        // 1. Let promise be a new promise.
        let promise = Promise::new_in_realm(cx);
        let task_manager = global.task_manager();
        let task_source = task_manager.dom_manipulation_task_source();
        let callback = callback_promise(&promise, self, task_source);

        let power_preference = match options.powerPreference {
            Some(GPUPowerPreference::Low_power) => PowerPreference::LowPower,
            Some(GPUPowerPreference::High_performance) => PowerPreference::HighPerformance,
            None => PowerPreference::default(),
        };
        let ids = global.wgpu_id_hub().create_adapter_id();

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
                promise.resolve_native(cx, &None::<GPUAdapter>);
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
                },
                ids,
            ))
            .is_err()
        {
            promise.reject_error(cx, Error::Operation(None));
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
    ) -> DomRoot<WGSLLanguageFeatures> {
        self.wgsl_language_features
            .or_init(|| WGSLLanguageFeatures::new(cx, &self.global(), None))
    }
}

impl RoutedPromiseListener<WebGPUAdapterResponse> for GPU {
    fn handle_response(
        &self,
        cx: &mut js::context::JSContext,
        response: WebGPUAdapterResponse,
        promise: &Rc<Promise>,
    ) {
        match response {
            Some(Ok(adapter)) => {
                let adapter = GPUAdapter::new(
                    cx,
                    &self.global(),
                    adapter.channel,
                    DOMString::from(format!(
                        "{} ({:?})",
                        adapter.adapter_info.name, adapter.adapter_id.0
                    )),
                    HandleObject::null(),
                    adapter.features,
                    adapter.limits,
                    adapter.adapter_info,
                    adapter.adapter_id,
                );
                promise.resolve_native(cx, &adapter);
            },
            Some(Err(e)) => {
                warn!("Could not get GPUAdapter ({:?})", e);
                promise.resolve_native(cx, &None::<GPUAdapter>);
            },
            None => {
                warn!("Couldn't get a response, because WebGPU is disabled");
                promise.resolve_native(cx, &None::<GPUAdapter>);
            },
        }
    }
}
