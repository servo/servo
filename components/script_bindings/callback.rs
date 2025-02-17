/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use js::jsapi::JSObject;
use js::rust::HandleObject;

use crate::reflector::DomObject;

pub trait ThisReflector {
    fn jsobject(&self) -> *mut JSObject;
}

impl<T: DomObject> ThisReflector for T {
    fn jsobject(&self) -> *mut JSObject {
        self.reflector().get_jsobject().get()
    }
}

impl ThisReflector for HandleObject<'_> {
    fn jsobject(&self) -> *mut JSObject {
        self.get()
    }
}
