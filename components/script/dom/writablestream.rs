/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use dom_struct::dom_struct;
use js::jsapi::JSObject;
use js::rust::{
    HandleObject as SafeHandleObject, HandleValue as SafeHandleValue,
    MutableHandleValue as SafeMutableHandleValue,
};

use crate::dom::bindings::codegen::Bindings::QueuingStrategyBinding::QueuingStrategy;
use crate::dom::bindings::codegen::Bindings::UnderlyingSinkBinding::UnderlyingSink;
use crate::dom::bindings::codegen::Bindings::WritableStreamBinding::WritableStreamMethods;
use crate::dom::bindings::error::Error;
use crate::dom::bindings::reflector::{
    reflect_dom_object, reflect_dom_object_with_proto, DomObject, Reflector,
};
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::dom::writablestreamdefaultwriter::WritableStreamDefaultWriter;
use crate::script_runtime::{CanGc, JSContext as SafeJSContext};

/// <https://streams.spec.whatwg.org/#ws-class>
#[dom_struct]
pub struct WritableStream {
    reflector_: Reflector,
}

impl WritableStreamMethods<crate::DomTypeHolder> for WritableStream {
    /// <https://streams.spec.whatwg.org/#ws-constructor>
    fn Constructor(
        cx: SafeJSContext,
        global: &GlobalScope,
        proto: Option<SafeHandleObject>,
        can_gc: CanGc,
        underlyingSink: Option<*mut JSObject>,
        strategy: &QueuingStrategy,
    ) -> DomRoot<WritableStream> {
        todo!()
    }

    /// <https://streams.spec.whatwg.org/#ws-locked>
    fn Locked(&self) -> bool {
        todo!()
    }

    /// <https://streams.spec.whatwg.org/#ws-abort>
    fn Abort(&self, cx: SafeJSContext, reason: SafeHandleValue, _can_gc: CanGc) -> Rc<Promise> {
        todo!()
    }

    /// <https://streams.spec.whatwg.org/#ws-close>
    fn Close(&self, _can_gc: CanGc) -> Rc<Promise> {
        todo!()
    }

    fn GetWriter(&self) -> DomRoot<WritableStreamDefaultWriter> {
        todo!()
    }
}
