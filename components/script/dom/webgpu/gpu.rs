/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use dom_struct::dom_struct;
use ipc_channel::ipc::{self, IpcSender};
use ipc_channel::router::ROUTER;
use js::jsapi::Heap;
use script_traits::ScriptMsg;
use webgpu::wgt::PowerPreference;
use webgpu::{wgc, WebGPUResponse};

use super::wgsllanguagefeatures::WGSLLanguageFeatures;
use crate::dom::bindings::codegen::Bindings::WebGPUBinding::{
    GPUMethods, GPUPowerPreference, GPURequestAdapterOptions, GPUTextureFormat,
};
use crate::dom::bindings::error::Error;
use crate::dom::bindings::refcounted::{Trusted, TrustedPromise};
use crate::dom::bindings::reflector::{reflect_dom_object, DomGlobal, DomObject, Reflector};
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::dom::webgpu::gpuadapter::GPUAdapter;
use crate::realms::InRealm;
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

    pub(crate) fn new(global: &GlobalScope) -> DomRoot<GPU> {
        reflect_dom_object(Box::new(GPU::new_inherited()), global, CanGc::note())
    }
}

pub(crate) trait AsyncWGPUListener {
    fn handle_response(&self, response: WebGPUResponse, promise: &Rc<Promise>, can_gc: CanGc);
}

struct WGPUResponse<T: AsyncWGPUListener + DomObject> {
    trusted: TrustedPromise,
    receiver: Trusted<T>,
}

impl<T: AsyncWGPUListener + DomObject> WGPUResponse<T> {
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    fn response(self, response: WebGPUResponse, can_gc: CanGc) {
        let promise = self.trusted.root();
        self.receiver
            .root()
            .handle_response(response, &promise, can_gc);
    }
}

pub(crate) fn response_async<T: AsyncWGPUListener + DomObject + 'static>(
    promise: &Rc<Promise>,
    receiver: &T,
) -> IpcSender<WebGPUResponse> {
    let (action_sender, action_receiver) = ipc::channel().unwrap();
    let task_source = receiver
        .global()
        .task_manager()
        .dom_manipulation_task_source()
        .to_sendable();
    let mut trusted: Option<TrustedPromise> = Some(TrustedPromise::new(promise.clone()));
    let trusted_receiver = Trusted::new(receiver);
    ROUTER.add_typed_route(
        action_receiver,
        Box::new(move |message| {
            let trusted = if let Some(trusted) = trusted.take() {
                trusted
            } else {
                error!("WebGPU callback called twice!");
                return;
            };

            let context = WGPUResponse {
                trusted,
                receiver: trusted_receiver.clone(),
            };
            task_source.queue(task!(process_webgpu_task: move|| {
                context.response(message.unwrap(), CanGc::note());
            }));
        }),
    );
    action_sender
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
        let sender = response_async(&promise, self);
        let power_preference = match options.powerPreference {
            Some(GPUPowerPreference::Low_power) => PowerPreference::LowPower,
            Some(GPUPowerPreference::High_performance) => PowerPreference::HighPerformance,
            None => PowerPreference::default(),
        };
        let ids = global.wgpu_id_hub().create_adapter_id();

        let script_to_constellation_chan = global.script_to_constellation_chan();
        if script_to_constellation_chan
            .send(ScriptMsg::RequestAdapter(
                sender,
                wgc::instance::RequestAdapterOptions {
                    power_preference,
                    compatible_surface: None,
                    force_fallback_adapter: options.forceFallbackAdapter,
                },
                ids,
            ))
            .is_err()
        {
            promise.reject_error(Error::Operation);
        }
        promise
    }

    /// <https://gpuweb.github.io/gpuweb/#dom-gpu-getpreferredcanvasformat>
    fn GetPreferredCanvasFormat(&self) -> GPUTextureFormat {
        // TODO: real implementation
        GPUTextureFormat::Rgba8unorm
    }

    /// <https://www.w3.org/TR/webgpu/#dom-gpu-wgsllanguagefeatures>
    fn WgslLanguageFeatures(&self, can_gc: CanGc) -> DomRoot<WGSLLanguageFeatures> {
        self.wgsl_language_features
            .or_init(|| WGSLLanguageFeatures::new(&self.global(), None, can_gc))
    }
}

impl AsyncWGPUListener for GPU {
    fn handle_response(&self, response: WebGPUResponse, promise: &Rc<Promise>, can_gc: CanGc) {
        match response {
            WebGPUResponse::Adapter(Ok(adapter)) => {
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
                promise.resolve_native(&adapter);
            },
            WebGPUResponse::Adapter(Err(e)) => {
                warn!("Could not get GPUAdapter ({:?})", e);
                promise.resolve_native(&None::<GPUAdapter>);
            },
            WebGPUResponse::None => {
                warn!("Couldn't get a response, because WebGPU is disabled");
                promise.resolve_native(&None::<GPUAdapter>);
            },
            _ => unreachable!("GPU received wrong WebGPUResponse"),
        }
    }
}
