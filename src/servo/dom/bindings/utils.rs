import js::rust::{compartment, bare_compartment};
import js::{JS_ARGV, JSCLASS_HAS_RESERVED_SLOTS, JSPROP_ENUMERATE, JSPROP_SHARED, JSVAL_NULL,
            JS_THIS_OBJECT, JS_SET_RVAL};
import js::jsapi::{JSContext, jsval, JSObject, JSBool, jsid, JSClass, JSFreeOp};
import js::jsapi::bindgen::{JS_ValueToString, JS_GetStringCharsZAndLength, JS_ReportError,
                            JS_GetReservedSlot, JS_SetReservedSlot, JS_NewStringCopyN,
                            JS_DefineFunctions, JS_DefineProperty, JS_GetContextPrivate};
import js::glue::bindgen::*;
import result::{result, ok, err};

enum DOMString {
    str(~str),
    null_string
}

type rust_box<T> = {rc: uint, td: *sys::type_desc, next: *(), prev: *(), payload: T};

unsafe fn squirrel_away<T>(+x: @T) -> *rust_box<T> {
    let y: *rust_box<T> = unsafe::reinterpret_cast(x);
    unsafe::forget(x);
    y
}

type rust_unique<T> = {payload: T};

unsafe fn squirrel_away_unique<T>(+x: ~T) -> *rust_box<T> {
    let y: *rust_box<T> = unsafe::reinterpret_cast(x);
    unsafe::forget(x);
    y
}

//XXX very incomplete
fn jsval_to_str(cx: *JSContext, v: jsval) -> result<~str, ()> {
    let jsstr;
    if RUST_JSVAL_IS_STRING(v) == 1 {
        jsstr = RUST_JSVAL_TO_STRING(v)
    } else {
        jsstr = JS_ValueToString(cx, v);
        if jsstr.is_null() {
            return err(());
        }
    }

    let len = 0;
    let chars = JS_GetStringCharsZAndLength(cx, jsstr, ptr::addr_of(len));
    return if chars.is_null() {
        err(())
    } else {
        unsafe {
            let buf = vec::unsafe::from_buf(chars as *u8, len as uint);
            ok(str::from_bytes(buf))
        }
    }
}

unsafe fn domstring_to_jsval(cx: *JSContext, str: DOMString) -> jsval {
    alt str {
      null_string => {
        JSVAL_NULL
      }
      str(s) => {
        str::as_buf(s, |buf, len| {
            let cbuf = unsafe::reinterpret_cast(buf);
            RUST_STRING_TO_JSVAL(JS_NewStringCopyN(cx, cbuf, len as libc::size_t))
        })
      }
    }
}

fn get_compartment(cx: *JSContext) -> *bare_compartment {
    unsafe {
        let priv: *libc::c_void = JS_GetContextPrivate(cx);
        let compartment: *bare_compartment = unsafe::reinterpret_cast(priv);
        assert cx == (*compartment).cx.ptr;
        compartment
    }
}
