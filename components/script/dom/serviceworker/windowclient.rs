/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use dom_struct::dom_struct;
use js::context::JSContext;
use script_bindings::codegen::GenericBindings::WindowClientBinding::WindowClientMethods;
use script_bindings::str::USVString;

use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::client::Client;
use crate::dom::promise::Promise;

#[dom_struct]
pub(crate) struct WindowClient {
    client: Client,
}

impl WindowClientMethods<crate::DomTypeHolder> for WindowClient {
    /// <https://w3c.github.io/ServiceWorker/#dom-windowclient-focus>
    fn Focus(&self, cx: &mut JSContext) -> Rc<Promise> {
        // TODO: Implement
        Promise::new(cx, &self.global())
    }

    /// <https://w3c.github.io/ServiceWorker/#dom-windowclient-navigate>
    fn Navigate(&self, cx: &mut JSContext, _url: USVString) -> Rc<Promise> {
        // TODO: Implement
        Promise::new(cx, &self.global())
    }
}
