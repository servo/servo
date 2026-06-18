/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use dom_struct::dom_struct;
use js::context::JSContext;
use script_bindings::codegen::GenericBindings::WindowBinding::WindowMethods;
use script_bindings::reflector::{Reflector, reflect_dom_object_with_cx};

use crate::dom::bindings::codegen::Bindings::PermissionStatusBinding::PermissionName;
use crate::dom::bindings::codegen::Bindings::WebNNBinding::{MLContextOptions, MLMethods};
use crate::dom::bindings::error::Error;
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::dom::webnn::mlcontext::MLContext;

/// <https://www.w3.org/TR/webnn/#ml>
#[dom_struct]
pub(crate) struct ML {
    reflector_: Reflector,
}

impl ML {
    pub(crate) fn new_inherited() -> ML {
        ML {
            reflector_: Reflector::new(),
        }
    }

    pub(crate) fn new(global: &GlobalScope, cx: &mut JSContext) -> DomRoot<ML> {
        reflect_dom_object_with_cx(Box::new(ML::new_inherited()), global, cx)
    }
}

impl MLMethods<crate::DomTypeHolder> for ML {
    /// <https://www.w3.org/TR/webnn/#dom-ml-createcontext>
    fn CreateContext(&self, cx: &mut JSContext, options: &MLContextOptions) -> Rc<Promise> {
        // Step 1. Let global be this's relevant global object.
        // Step 2. Let realm be this's relevant realm.
        let global = self.global();
        let window = global.as_window();
        let document = window.Document();
        // Step 3. If global's associated Document is not allowed to
        //         use the webnn feature, then return a new promise in realm
        //         rejected with a "SecurityError" DOMException.
        if !document.allowed_to_use_feature(PermissionName::WebNN) {
            let promise = Promise::new(cx, &global);
            promise.reject_error(
                cx,
                Error::Security(Some("WebNN permission not allowed".into())),
            );
            return promise;
        }
        // Step 4. Let promise be a new promise in realm.
        let promise = Promise::new(cx, &global);

        // Step 5. Run the following steps in parallel.
        // Step 5.1. Let context be the result of creating a context given
        //         realm and options. If that returns failure, then queue an
        //         ML task with global to reject promise with a
        //         "NotSupportedError" DOMException and abort these steps.
        let context = MLContext::new(&global, options, cx);
        // TODO: Step 5.2. Queue an ML task with global to resolve promise with
        //         context.
        promise.resolve_native(cx, &context);
        // Step 6. Return promise.
        promise
    }
}
