/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use constellation_traits::ScriptToConstellationMessage;
use dom_struct::dom_struct;
use js::jsapi::Heap;
use webgpu_traits::WebGPUAdapterResponse;
use wgpu_types::PowerPreference;

use super::wgsllanguagefeatures::WGSLLanguageFeatures;
use crate::dom::bindings::codegen::Bindings::WebGPUBinding::{
    GPUMethods, GPUPowerPreference, GPURequestAdapterOptions, GPUTextureFormat,
};
use crate::dom::bindings::error::Error;
use crate::dom::bindings::reflector::{DomGlobal, Reflector, reflect_dom_object};
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::dom::webgpu::gpuadapter::GPUAdapter;
use crate::realms::InRealm;
use crate::routed_promise::{RoutedPromiseListener, route_promise};
use crate::script_runtime::CanGc;

#[dom_struct]
#[allow(clippy::upper_case_acronyms)]
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

    pub(crate) fn new(global: &GlobalScope, can_gc: CanGc) -> DomRoot<GPU> {
        reflect_dom_object(Box::new(GPU::new_inherited()), global, can_gc)
    }
}

impl GPUMethods<crate::DomTypeHolder> for GPU {
    // https://gpuweb.github.io/gpuweb/#dom-gpu-requestadapter
    fn RequestAdapter(
        &self,
        options: &GPURequestAdapterOptions,
        comp: InRealm,
        can_gc: CanGc,
    ) -> Rc<Promise> {
        let global = &self.global();
        let promise = Promise::new_in_current_realm(comp, can_gc);
        let task_source = global.task_manager().dom_manipulation_task_source();
        let sender = route_promise(&promise, self, task_source);

        let power_preference = match options.powerPreference {
            Some(GPUPowerPreference::Low_power) => PowerPreference::LowPower,
            Some(GPUPowerPreference::High_performance) => PowerPreference::HighPerformance,
            None => PowerPreference::default(),
        };
        let ids = global.wgpu_id_hub().create_adapter_id();

        let script_to_constellation_chan = global.script_to_constellation_chan();
        if script_to_constellation_chan
            .send(ScriptToConstellationMessage::RequestAdapter(
                sender,
                wgpu_core::instance::RequestAdapterOptions {
                    power_preference,
                    compatible_surface: None,
                    force_fallback_adapter: options.forceFallbackAdapter,
                },
                ids,
            ))
            .is_err()
        {
            promise.reject_error(Error::Operation, can_gc);
        }
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
    fn WgslLanguageFeatures(&self, can_gc: CanGc) -> DomRoot<WGSLLanguageFeatures> {
        self.wgsl_language_features
            .or_init(|| WGSLLanguageFeatures::new(&self.global(), None, can_gc))
    }
}

impl RoutedPromiseListener<WebGPUAdapterResponse> for GPU {
    fn handle_response(
        &self,
        response: WebGPUAdapterResponse,
        promise: &Rc<Promise>,
        can_gc: CanGc,
    ) {
        match response {
            Some(Ok(adapter)) => {
                let adapter = GPUAdapter::new(
                    &self.global(),
                    adapter.channel,
                    DOMString::from(format!(
                        "{} ({:?})",
                        adapter.adapter_info.name, adapter.adapter_id.0
                    )),
                    Heap::default(),
                    adapter.features,
                    adapter.limits,
                    adapter.adapter_info,
                    adapter.adapter_id,
                    can_gc,
                );
                promise.resolve_native(&adapter, can_gc);
            },
            Some(Err(e)) => {
                warn!("Could not get GPUAdapter ({:?})", e);
                promise.resolve_native(&None::<GPUAdapter>, can_gc);
            },
            None => {
                warn!("Couldn't get a response, because WebGPU is disabled");
                promise.resolve_native(&None::<GPUAdapter>, can_gc);
            },
        }
    }
}
