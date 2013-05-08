/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use js::jsapi::JSVal;
use js::{JSVAL_FALSE, JSVAL_TRUE};
use js::glue::bindgen::{RUST_UINT_TO_JSVAL, RUST_JSVAL_TO_INT};

pub trait JSValConvertible {
    fn to_jsval(&self) -> JSVal;
    fn from_jsval(val: JSVal) -> Option<Self>;
}

impl JSValConvertible for u32 {
    fn to_jsval(&self) -> JSVal {
        RUST_UINT_TO_JSVAL(*self)
    }

    fn from_jsval(val: JSVal) -> Option<u32> {
        Some(RUST_JSVAL_TO_INT(val) as u32)
    }
}

impl JSValConvertible for bool {
    fn to_jsval(&self) -> JSVal {
        if *self {
            JSVAL_TRUE
        } else {
            JSVAL_FALSE
        }
    }

    fn from_jsval(val: JSVal) -> Option<bool> {
        if val == JSVAL_TRUE {
            Some(true)
        } else if val == JSVAL_FALSE {
            Some(false)
        } else {
            None
        }
    }
}