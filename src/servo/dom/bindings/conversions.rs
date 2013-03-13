use js::jsapi::JSVal;
use js::glue::bindgen::{RUST_INT_TO_JSVAL, RUST_JSVAL_TO_INT};

pub trait JSValConvertible<T> {
    fn to_jsval(&self) -> JSVal;
    static fn from_jsval(val: JSVal) -> Option<T>;
}

impl JSValConvertible<u32> for u32 {
    fn to_jsval(&self) -> JSVal {
        RUST_INT_TO_JSVAL(*self as i32)
    }

    static fn from_jsval(val: JSVal) -> Option<u32> {
        Some(RUST_JSVAL_TO_INT(val) as u32)
    }
}
