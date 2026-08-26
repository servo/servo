/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use js::jsapi::HandleObject;
use script_bindings::str::DOMString;
use webgpu_traits::WebGPUAdapterResponse;

use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::gpu::GPU;
use crate::dom::gpuadapter::GPUAdapter;
use crate::dom::promise::Promise;
use crate::routed_promise::RoutedPromiseListener;

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
