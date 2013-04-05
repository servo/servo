/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use js::jsapi::JSVal;
use js::glue::bindgen::{RUST_INT_TO_JSVAL, RUST_JSVAL_TO_INT};

pub trait JSValConvertible<T> {
    fn to_jsval(&self) -> JSVal;
    fn from_jsval(val: JSVal) -> Option<T>;
}

impl JSValConvertible<u32> for u32 {
    fn to_jsval(&self) -> JSVal {
        RUST_INT_TO_JSVAL(*self as i32)
    }

    fn from_jsval(val: JSVal) -> Option<u32> {
        Some(RUST_JSVAL_TO_INT(val) as u32)
    }
}
