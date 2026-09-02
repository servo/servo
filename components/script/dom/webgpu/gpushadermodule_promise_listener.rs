/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use webgpu_traits::ShaderCompilationInfo;

use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::promise::Promise;
use crate::dom::types::{GPUCompilationInfo, GPUShaderModule};
use crate::routed_promise::RoutedPromiseListener;

impl RoutedPromiseListener<Option<ShaderCompilationInfo>> for GPUShaderModule {
    fn handle_response(
        &self,
        cx: &mut js::context::JSContext,
        response: Option<ShaderCompilationInfo>,
        promise: &Rc<Promise>,
    ) {
        let info = GPUCompilationInfo::from(cx, &self.global(), response);
        promise.resolve_native(cx, &info);
    }
}
