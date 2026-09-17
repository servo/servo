/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::promise::RootedPromise;
use crate::dom::types::GPUQueue;
use crate::routed_promise::RoutedPromiseListener;

impl RoutedPromiseListener<()> for GPUQueue {
    fn handle_response(
        &self,
        cx: &mut js::context::JSContext,
        _response: (),
        promise: &RootedPromise,
    ) {
        promise.resolve_native(cx, &());
    }
}
