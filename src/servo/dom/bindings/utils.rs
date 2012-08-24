import js::rust::{compartment, bare_compartment, methods};
import js::{JS_ARGV, JSCLASS_HAS_RESERVED_SLOTS, JSPROP_ENUMERATE, JSPROP_SHARED, JSVAL_NULL,
            JS_THIS_OBJECT, JS_SET_RVAL};
import js::jsapi::{JSContext, jsval, JSObject, JSBool, jsid, JSClass, JSFreeOp};
import js::jsapi::bindgen::{JS_ValueToString, JS_GetStringCharsZAndLength, JS_ReportError,
                            JS_GetReservedSlot, JS_SetReservedSlot, JS_NewStringCopyN,
                            JS_DefineFunctions, JS_DefineProperty, JS_GetContextPrivate,
                            JS_GetClass, JS_GetPrototype};
import js::crust::{JS_ConvertStub, JS_PropertyStub, JS_StrictPropertyStub,
                   JS_EnumerateStub, JS_ConvertStub, JS_ResolveStub};
import js::glue::bindgen::*;
import ptr::null;
import result::{result, ok, err};

enum DOMString {
    str(~str),
    null_string
}

type rust_box<T> = {rc: uint, td: *sys::TypeDesc, next: *(), prev: *(), payload: T};

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
    match str {
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

extern fn has_instance(_cx: *JSContext, obj: *JSObject, v: *jsval, bp: *mut JSBool) -> JSBool {
    //XXXjdm this is totally broken for non-object values
    let mut o = RUST_JSVAL_TO_OBJECT(unsafe {*v});
    let clasp = JS_GetClass(obj);
    unsafe { *bp = 0; }
    while o.is_not_null() {
        if JS_GetClass(o) == clasp {
            unsafe { *bp = 1; }
            break;
        }
        o = JS_GetPrototype(o);
    }
    return 1;
}

fn prototype_jsclass(name: ~str) -> fn(bare_compartment) -> JSClass {
    return fn@(compartment: bare_compartment) -> JSClass {
        {name: compartment.add_name(name),
         flags: 0,
         addProperty: JS_PropertyStub,
         delProperty: JS_PropertyStub,
         getProperty: JS_PropertyStub,
         setProperty: JS_StrictPropertyStub,
         enumerate: JS_EnumerateStub,
         resolve: JS_PropertyStub,
         convert: JS_ConvertStub,
         finalize: null(),
         checkAccess: null(),
         call: null(),
         hasInstance: has_instance,
         construct: null(),
         trace: null(),
         reserved: (null(), null(), null(), null(), null(),  // 05
                    null(), null(), null(), null(), null(),  // 10
                    null(), null(), null(), null(), null(),  // 15
                    null(), null(), null(), null(), null(),  // 20
                    null(), null(), null(), null(), null(),  // 25
                    null(), null(), null(), null(), null(),  // 30
                    null(), null(), null(), null(), null(),  // 35
                    null(), null(), null(), null(), null())} // 40
    };
}

fn instance_jsclass(name: ~str, finalize: *u8)
    -> fn(bare_compartment) -> JSClass {
    return fn@(compartment: bare_compartment) -> JSClass {
        {name: compartment.add_name(name),
         flags: JSCLASS_HAS_RESERVED_SLOTS(1),
         addProperty: JS_PropertyStub,
         delProperty: JS_PropertyStub,
         getProperty: JS_PropertyStub,
         setProperty: JS_StrictPropertyStub,
         enumerate: JS_EnumerateStub,
         resolve: JS_ResolveStub,
         convert: JS_ConvertStub,
         finalize: finalize,
         checkAccess: null(),
         call: null(),
         hasInstance: has_instance,
         construct: null(),
         trace: null(),
         reserved: (null(), null(), null(), null(), null(),  // 05
                    null(), null(), null(), null(), null(),  // 10
                    null(), null(), null(), null(), null(),  // 15
                    null(), null(), null(), null(), null(),  // 20
                    null(), null(), null(), null(), null(),  // 25
                    null(), null(), null(), null(), null(),  // 30
                    null(), null(), null(), null(), null(),  // 35
                    null(), null(), null(), null(), null())} // 40
    };
}

fn define_empty_prototype(name: ~str, proto: option<~str>, compartment: bare_compartment)
    -> js::rust::jsobj {
    compartment.register_class(utils::prototype_jsclass(name));

    //TODO error checking
    let obj = result::unwrap(
        match proto {
            some(s) => compartment.new_object_with_proto(name, s,
                                                         compartment.global_obj.ptr),
            none => compartment.new_object(name, null(), compartment.global_obj.ptr)
        });

    compartment.define_property(name, RUST_OBJECT_TO_JSVAL(obj.ptr),
                                JS_PropertyStub, JS_StrictPropertyStub,
                                JSPROP_ENUMERATE);
    compartment.stash_global_proto(name, obj);
    return obj;
}
