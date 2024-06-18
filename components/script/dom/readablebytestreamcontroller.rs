/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::jsapi::{
    AutoRequireNoGC, HandleObject, HandleValue, Heap, IsReadableStream, JSContext, JSObject,
};
use js::jsval::{JSVal, ObjectValue, UndefinedValue};
use js::rust::{HandleObject as SafeHandleObject, HandleValue as SafeHandleValue, IntoHandle};

use crate::dom::bindings::codegen::Bindings::ReadableByteStreamControllerBinding::ReadableByteStreamControllerMethods;
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
pub struct ReadableByteStreamController {
    reflector_: Reflector,
}

impl ReadableByteStreamControllerMethods for ReadableByteStreamController {
    fn GetByobRequest(
        &self,
    ) -> Fallible<Option<DomRoot<super::readablestreambyobrequest::ReadableStreamBYOBRequest>>>
    {
        todo!()
    }

    fn GetDesiredSize(&self) -> Option<f64> {
        todo!()
    }

    fn Close(&self) -> Fallible<()> {
        todo!()
    }

    fn Enqueue(
        &self,
        chunk: js::gc::CustomAutoRooterGuard<js::typedarray::ArrayBufferView>,
    ) -> Fallible<()> {
        todo!()
    }

    fn Error(&self, cx: SafeJSContext, e: SafeHandleValue) -> Fallible<()> {
        todo!()
    }
}
