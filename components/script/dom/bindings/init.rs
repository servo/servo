/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::RegisterBindings;
use dom::bindings::proxyhandler::dom_proxy_shadows;
use dom::bindings::utils::instance_class_matches_proto;
use js;
use js::jsapi::JSRuntime;
use js::jsfriendapi::{SetDOMCallbacks, DOMCallbacks};
use js::glue::{GetProxyHandlerFamily, SetDOMProxyInformation};

static dom_callbacks: DOMCallbacks = DOMCallbacks {
    instanceClassMatchesProto: Some(instance_class_matches_proto),
};

pub fn global_init() {
    unsafe {
        let family = GetProxyHandlerFamily();
        SetDOMProxyInformation(family, js::PROXY_EXTRA_SLOT + js::PROXY_PRIVATE_SLOT,
                               Some(dom_proxy_shadows));
    }
    RegisterBindings::RegisterProxyHandlers();
}

pub fn init(rt: *mut JSRuntime) {
    unsafe {
        SetDOMCallbacks(rt, &dom_callbacks);
    }
}
