/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use js::jsapi::{JSVal, JSContext};
use js::{JSVAL_FALSE, JSVAL_TRUE};
use js::glue::{RUST_UINT_TO_JSVAL, RUST_JSVAL_TO_INT, RUST_DOUBLE_TO_JSVAL};
use js::glue::{RUST_JSVAL_TO_DOUBLE, RUST_JSVAL_IS_INT, RUST_JSVAL_IS_DOUBLE};
use js::glue::{RUST_JSVAL_IS_BOOLEAN, RUST_JSVAL_TO_BOOLEAN};

pub trait JSValConvertible {
    fn to_jsval(&self) -> JSVal;
    fn from_jsval(cx: *JSContext, val: JSVal) -> Option<Self>;
}


impl JSValConvertible for i64 {
    fn to_jsval(&self) -> JSVal {
        unsafe {
            RUST_DOUBLE_TO_JSVAL(*self as f64)
        }
    }

    fn from_jsval(_cx: *JSContext, val: JSVal) -> Option<i64> {
        unsafe {
            if RUST_JSVAL_IS_INT(val) != 0 {
                Some(RUST_JSVAL_TO_DOUBLE(val) as i64)
            } else {
                None
            }
        }
    }
}

impl JSValConvertible for u32 {
    fn to_jsval(&self) -> JSVal {
        unsafe {
            RUST_UINT_TO_JSVAL(*self)
        }
    }

    fn from_jsval(_cx: *JSContext, val: JSVal) -> Option<u32> {
        unsafe {
            if RUST_JSVAL_IS_INT(val) != 0 {
                Some(RUST_JSVAL_TO_INT(val) as u32)
            } else {
                None
            }
        }
    }
}

impl JSValConvertible for i32 {
    fn to_jsval(&self) -> JSVal {
        unsafe {
            RUST_UINT_TO_JSVAL(*self as u32)
        }
    }

    fn from_jsval(_cx: *JSContext, val: JSVal) -> Option<i32> {
        unsafe {
            if RUST_JSVAL_IS_INT(val) != 0 {
                Some(RUST_JSVAL_TO_INT(val) as i32)
            } else {
                None
            }
        }
    }
}

impl JSValConvertible for u16 {
    fn to_jsval(&self) -> JSVal {
        unsafe {
            RUST_UINT_TO_JSVAL(*self as u32)
        }
    }

    fn from_jsval(_cx: *JSContext, val: JSVal) -> Option<u16> {
        unsafe {
            if RUST_JSVAL_IS_INT(val) != 0 {
                Some(RUST_JSVAL_TO_INT(val) as u16)
            } else {
                None
            }
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

    fn from_jsval(_cx: *JSContext, val: JSVal) -> Option<bool> {
        unsafe {
            if RUST_JSVAL_IS_BOOLEAN(val) != 0 {
                Some(RUST_JSVAL_TO_BOOLEAN(val) != 0)
            } else {
                None
            }
        }
    }
}

impl JSValConvertible for f32 {
    fn to_jsval(&self) -> JSVal {
        unsafe {
            RUST_DOUBLE_TO_JSVAL(*self as f64)
        }
    }

    fn from_jsval(_cx: *JSContext, val: JSVal) -> Option<f32> {
        unsafe {
            if RUST_JSVAL_IS_DOUBLE(val) != 0 {
                Some(RUST_JSVAL_TO_DOUBLE(val) as f32)
            } else {
                None
            }
        }
    }
}

impl JSValConvertible for f64 {
    fn to_jsval(&self) -> JSVal {
        unsafe {
            RUST_DOUBLE_TO_JSVAL(*self as f64)
        }
    }

    fn from_jsval(_cx: *JSContext, val: JSVal) -> Option<f64> {
        unsafe {
            if RUST_JSVAL_IS_DOUBLE(val) != 0 {
                Some(RUST_JSVAL_TO_DOUBLE(val) as f64)
            } else {
                None
            }
        }
    }
}
