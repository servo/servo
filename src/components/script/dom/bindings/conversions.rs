/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use js::jsapi::JSVal;
use js::{JSVAL_FALSE, JSVAL_TRUE};
use js::glue::{RUST_UINT_TO_JSVAL, RUST_JSVAL_TO_INT, RUST_DOUBLE_TO_JSVAL, RUST_JSVAL_TO_DOUBLE};

pub trait JSValConvertible {
    fn to_jsval(&self) -> JSVal;
    fn from_jsval(val: JSVal) -> Option<Self>;
}


impl JSValConvertible for i64 {
    fn to_jsval(&self) -> JSVal {
        unsafe {
            RUST_DOUBLE_TO_JSVAL(*self as f64)
        }
    }

    fn from_jsval(val: JSVal) -> Option<i64> {
        unsafe {
            Some(RUST_JSVAL_TO_DOUBLE(val) as i64)
        }
    }
}

impl JSValConvertible for u32 {
    fn to_jsval(&self) -> JSVal {
        unsafe {
            RUST_UINT_TO_JSVAL(*self)
        }
    }

    fn from_jsval(val: JSVal) -> Option<u32> {
        unsafe {
            Some(RUST_JSVAL_TO_INT(val) as u32)
        }
    }
}

impl JSValConvertible for i32 {
    fn to_jsval(&self) -> JSVal {
        unsafe {
            RUST_UINT_TO_JSVAL(*self as u32)
        }
    }

    fn from_jsval(val: JSVal) -> Option<i32> {
        unsafe {
            Some(RUST_JSVAL_TO_INT(val) as i32)
        }
    }
}

impl JSValConvertible for u16 {
    fn to_jsval(&self) -> JSVal {
        unsafe {
            RUST_UINT_TO_JSVAL(*self as u32)
        }
    }

    fn from_jsval(val: JSVal) -> Option<u16> {
        unsafe {
            Some(RUST_JSVAL_TO_INT(val) as u16)
        }
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

impl JSValConvertible for f32 {
    fn to_jsval(&self) -> JSVal {
        unsafe {
            RUST_DOUBLE_TO_JSVAL(*self as f64)
        }
    }

    fn from_jsval(val: JSVal) -> Option<f32> {
        unsafe {
            Some(RUST_JSVAL_TO_DOUBLE(val) as f32)
        }
    }
}

impl JSValConvertible for f64 {
    fn to_jsval(&self) -> JSVal {
        unsafe {
            RUST_DOUBLE_TO_JSVAL(*self as f64)
        }
    }

    fn from_jsval(val: JSVal) -> Option<f64> {
        unsafe {
            Some(RUST_JSVAL_TO_DOUBLE(val) as f64)
        }
    }
}
