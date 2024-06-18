/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::jsapi::{
    AutoRequireNoGC, HandleObject, HandleValue, Heap, IsReadableStream, JSContext, JSObject,
};
use js::jsval::{JSVal, ObjectValue, UndefinedValue};
use js::rust::{HandleObject as SafeHandleObject, HandleValue as SafeHandleValue, IntoHandle};

use crate::dom::bindings::codegen::Bindings::ReadableStreamBYOBRequestBinding::ReadableStreamBYOBRequestMethods;
use crate::dom::bindings::conversions::{ConversionBehavior, ConversionResult};
use crate::dom::bindings::error::Error;
use crate::dom::bindings::import::module::Fallible;
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::settings_stack::{AutoEntryScript, AutoIncumbentScript};
use crate::dom::bindings::utils::get_dictionary_property;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::js::conversions::FromJSValConvertible;
use crate::realms::{enter_realm, InRealm};
use crate::script_runtime::JSContext as SafeJSContext;

#[dom_struct]
pub struct ReadableStreamBYOBRequest {
    reflector_: Reflector,
}

impl ReadableStreamBYOBRequestMethods for ReadableStreamBYOBRequest {
    fn GetView(&self, cx: SafeJSContext) -> Option<js::typedarray::ArrayBufferView> {
        todo!()
    }

    fn Respond(&self, bytesWritten: u64) -> Fallible<()> {
        todo!()
    }

    fn RespondWithNewView(
        &self,
        view: js::gc::CustomAutoRooterGuard<js::typedarray::ArrayBufferView>,
    ) -> Fallible<()> {
        todo!()
    }
}
