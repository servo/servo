/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;
use std::sync::Arc;

use script_bindings::DomTypes;
use servo_base::generic_channel::GenericCallback;
use webgpu_traits::WebGPUDeviceResponse;

use crate::gpuadapter::GPUAdapter;
use crate::identityhub::IdentityHub;

/// The main trait for creating and using promises in script_webgpu.
pub trait WebGPUPromiseTrait<D: DomTypes> {
    fn callback_promise(
        self: &Rc<Self>,
        d: &GPUAdapter<D>,
    ) -> GenericCallback<WebGPUDeviceResponse>;
}

pub trait WebGPUGlobalTrait {
    fn global_wgpu_id_hub(&self) -> Arc<IdentityHub>;
}
