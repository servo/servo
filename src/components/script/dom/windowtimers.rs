/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use js::jsapi::JSContext;
use js::jsval::JSVal;

pub trait WindowTimers {
    fn SetTimeout(&mut self, _cx: *JSContext, callback: JSVal, timeout: i32) -> i32;
    fn ClearTimeout(&mut self, handle: i32);
    fn SetInterval(&mut self, _cx: *JSContext, callback: JSVal, timeout: i32) -> i32;
    fn ClearInterval(&mut self, handle: i32);
}
