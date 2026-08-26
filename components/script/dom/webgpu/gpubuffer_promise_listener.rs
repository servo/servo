/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use webgpu_traits::Mapping;
use wgpu_core::resource::BufferAccessError;

use crate::dom::promise::Promise;
use crate::dom::types::GPUBuffer;
use crate::routed_promise::RoutedPromiseListener;

impl RoutedPromiseListener<Result<Mapping, BufferAccessError>> for GPUBuffer {
    fn handle_response(
        &self,
        cx: &mut js::context::JSContext,
        response: Result<Mapping, BufferAccessError>,
        promise: &Rc<Promise>,
    ) {
        match response {
            Ok(mapping) => self.map_success(cx, promise, mapping),
            Err(_) => self.map_failure(cx, promise),
        }
    }
}
