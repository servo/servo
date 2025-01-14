/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;

use crate::dom::bindings::codegen::Bindings::ReadableStreamBYOBRequestBinding::ReadableStreamBYOBRequestMethods;
use crate::dom::bindings::import::module::{Error, Fallible};
use crate::dom::bindings::reflector::Reflector;
use crate::script_runtime::JSContext as SafeJSContext;

/// <https://streams.spec.whatwg.org/#readablestreambyobrequest>
#[dom_struct]
pub(crate) struct ReadableStreamBYOBRequest {
    reflector_: Reflector,
}

impl ReadableStreamBYOBRequestMethods<crate::DomTypeHolder> for ReadableStreamBYOBRequest {
    /// <https://streams.spec.whatwg.org/#rs-byob-request-view>
    fn GetView(&self, _cx: SafeJSContext) -> Option<js::typedarray::ArrayBufferView> {
        // TODO
        None
    }

    /// <https://streams.spec.whatwg.org/#rs-byob-request-respond>
    fn Respond(&self, _bytes_written: u64) -> Fallible<()> {
        // TODO
        Err(Error::NotFound)
    }

    /// <https://streams.spec.whatwg.org/#rs-byob-request-respond-with-new-view>
    fn RespondWithNewView(
        &self,
        _view: js::gc::CustomAutoRooterGuard<js::typedarray::ArrayBufferView>,
    ) -> Fallible<()> {
        // TODO
        Err(Error::NotFound)
    }
}
