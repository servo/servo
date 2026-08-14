/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::JSContext;
use script_bindings::reflector::{Reflector, reflect_dom_object_with_cx};
use script_bindings::root::DomRoot;

use crate::dom::bindings::codegen::Bindings::WebNNBinding::MLContextOptions;
use crate::dom::globalscope::GlobalScope;

/// <https://www.w3.org/TR/webnn/#api-mlcontext>
#[dom_struct]
pub(crate) struct MLContext {
    reflector_: Reflector,
}

impl MLContext {
    pub(crate) fn new_inherited(_options: &MLContextOptions) -> MLContext {
        MLContext {
            reflector_: Reflector::new(),
        }
    }

    pub(crate) fn new(
        global: &GlobalScope,
        cx: &mut JSContext,
        options: &MLContextOptions,
    ) -> DomRoot<MLContext> {
        reflect_dom_object_with_cx(Box::new(MLContext::new_inherited(options)), global, cx)
    }
}
