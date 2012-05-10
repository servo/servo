// Definition for the global object that we use:

import jsapi::*;
import jsapi::bindgen::*;
import ptr::null;

crust fn PropertyStub(++arg0: *JSContext,
                      ++arg1: *JSObject,
                      ++arg2: jsid,
                      ++arg3: *jsval) -> JSBool {
    JS_PropertyStub(arg0, arg1, arg2, arg3)
}

crust fn StrictPropertyStub(++arg0: *JSContext,
                            ++arg1: *JSObject,
                            ++arg2: jsid,
                            ++arg3: JSBool,
                            ++arg4: *jsval) -> JSBool {
    JS_StrictPropertyStub(arg0, arg1, arg2, arg3, arg4)
}

crust fn EnumerateStub(++arg0: *JSContext, ++arg1: *JSObject) -> JSBool {
    JS_EnumerateStub(arg0, arg1)
}

crust fn ResolveStub(++arg0: *JSContext,
                     ++arg1: *JSObject,
                     ++arg2: jsid) -> JSBool {
    JS_ResolveStub(arg0, arg1, arg2)
}

crust fn ConvertStub(++arg0: *JSContext,
                     ++arg1: *JSObject,
                     ++arg2: JSType,
                     ++arg3: *jsval) -> JSBool {
    JS_ConvertStub(arg0, arg1, arg2, arg3)
}

fn global_class() -> js::named_class {
    let name = "global";
    let c_str = str::as_c_str(name) { |bytes| bytes };
    @{name: name, // in theory, this should *move* the str in here..
      jsclass: {name: c_str, // ...and so this ptr ought to be valid.
                flags: 0x48000_u32,
                addProperty: PropertyStub,
                delProperty: PropertyStub,
                getProperty: PropertyStub,
                setProperty: StrictPropertyStub,
                enumerate: EnumerateStub,
                resolve: ResolveStub,
                convert: ConvertStub,
                finalize: null(),
                reserved0: null(),
                checkAccess: null(),
                call: null(),
                construct: null(),
                xdrObject: null(),
                hasInstance: null(),
                trace: null(),
                reserved1: null(),
                reserved: (null(), null(), null(), null(), null(),   // 05
                           null(), null(), null(), null(), null(),   // 10
                           null(), null(), null(), null(), null(),   // 15
                           null(), null(), null(), null(), null(),   // 20
                           null(), null(), null(), null(), null(),   // 25
                           null(), null(), null(), null(), null(),   // 30
                           null(), null(), null(), null(), null(),   // 35
                           null(), null(), null(), null(), null())}} // 40
}