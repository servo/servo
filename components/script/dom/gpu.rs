/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::compartments::InCompartment;
use crate::dom::bindings::codegen::Bindings::GPUBinding::GPURequestAdapterOptions;
use crate::dom::bindings::codegen::Bindings::GPUBinding::{self, GPUMethods, GPUPowerPreference};
use crate::dom::bindings::error::Error;
use crate::dom::bindings::refcounted::{Trusted, TrustedPromise};
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::gpuadapter::GPUAdapter;
use crate::dom::promise::Promise;
use crate::task_source::TaskSource;
use dom_struct::dom_struct;
use ipc_channel::ipc::{self, IpcSender};
use ipc_channel::router::ROUTER;
use js::jsapi::Heap;
use std::rc::Rc;
use webgpu::wgpu;
use webgpu::{WebGPU, WebGPURequest, WebGPUResponse, WebGPUResponseResult};

#[dom_struct]
pub struct GPU {
    reflector_: Reflector,
}

impl GPU {
    pub fn new_inherited() -> GPU {
        GPU {
            reflector_: Reflector::new(),
        }
    }

    pub fn new(global: &GlobalScope) -> DomRoot<GPU> {
        reflect_dom_object(Box::new(GPU::new_inherited()), global, GPUBinding::Wrap)
    }

    fn wgpu_channel(&self) -> Option<WebGPU> {
        self.global().as_window().webgpu_channel()
    }
}

pub trait AsyncWGPUListener {
    fn handle_response(&self, response: WebGPUResponse, promise: &Rc<Promise>);
}

struct WGPUResponse<T: AsyncWGPUListener + DomObject> {
    trusted: TrustedPromise,
    receiver: Trusted<T>,
}

impl<T: AsyncWGPUListener + DomObject> WGPUResponse<T> {
    #[allow(unrooted_must_root)]
    fn response(self, response: WebGPUResponseResult) {
        let promise = self.trusted.root();
        match response {
            Ok(response) => self.receiver.root().handle_response(response, &promise),
            Err(error) => promise.reject_error(Error::Type(format!(
                "Received error from WebGPU thread: {}",
                error
            ))),
        }
    }
}

pub fn response_async<T: AsyncWGPUListener + DomObject + 'static>(
    promise: &Rc<Promise>,
    receiver: &T,
) -> IpcSender<WebGPUResponseResult> {
    let (action_sender, action_receiver) = ipc::channel().unwrap();
    let (task_source, canceller) = receiver
        .global()
        .as_window()
        .task_manager()
        .dom_manipulation_task_source_with_canceller();
    let mut trusted = Some(TrustedPromise::new(promise.clone()));
    let trusted_receiver = Trusted::new(receiver);
    ROUTER.add_route(
        action_receiver.to_opaque(),
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
            let result = task_source.queue_with_canceller(
                task!(process_webgpu_task: move|| {
                    context.response(message.to().unwrap());
                }),
                &canceller,
            );
            if let Err(err) = result {
                error!("Failed to queue GPU listener-task: {:?}", err);
            }
        }),
    );
    action_sender
}

impl GPUMethods for GPU {
    // https://gpuweb.github.io/gpuweb/#dom-gpu-requestadapter
    fn RequestAdapter(
        &self,
        options: &GPURequestAdapterOptions,
        comp: InCompartment,
    ) -> Rc<Promise> {
        let promise = Promise::new_in_current_compartment(&self.global(), comp);
        let sender = response_async(&promise, self);
        let power_preference = match options.powerPreference {
            Some(GPUPowerPreference::Low_power) => wgpu::PowerPreference::LowPower,
            Some(GPUPowerPreference::High_performance) => wgpu::PowerPreference::HighPerformance,
            None => wgpu::PowerPreference::Default,
        };

        match self.wgpu_channel() {
            Some(channel) => {
                channel
                    .0
                    .send(WebGPURequest::RequestAdapter(
                        sender,
                        wgpu::RequestAdapterOptions { power_preference },
                    ))
                    .unwrap();
            },
            None => promise.reject_error(Error::Type("No WebGPU thread...".to_owned())),
        };
        promise
    }
}

impl AsyncWGPUListener for GPU {
    fn handle_response(&self, response: WebGPUResponse, promise: &Rc<Promise>) {
        match response {
            WebGPUResponse::RequestAdapter(name, adapter) => {
                let adapter = GPUAdapter::new(
                    &self.global(),
                    DOMString::from(name),
                    Heap::default(),
                    adapter,
                );
                promise.resolve_native(&adapter);
            },
            response => promise.reject_error(Error::Type(format!(
                "Wrong response received for GPU from WebGPU thread {:?}",
                response,
            ))),
        }
    }
}
