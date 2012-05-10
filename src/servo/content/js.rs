import jsapi::*;
import jsapi::bindgen::*;
import ptr::{null, addr_of};
import result::{result, ok, err, extensions};

export rt;
export methods;
export cx;
export named_class;
export jsobj;

const default_heapsize: u32 = 8_u32 * 1024_u32 * 1024_u32;
const default_stacksize: uint = 8192u;
const ERR: JSBool = 0_i32;

fn result(n: JSBool) -> result<(),()> {
    if n != ERR {ok(())} else {err(())}
}

type named_class = @{
    name: str,
    jsclass: JSClass
};

// ___________________________________________________________________________
// runtimes

type rt = @rt_rsrc;

resource rt_rsrc(self: {ptr: *JSRuntime}) {
    JS_Finish(self.ptr)
}

fn rt() -> rt {
    @rt_rsrc({ptr: JS_Init(default_heapsize)})
}

impl methods for rt {
    fn cx() -> cx {
        @cx_rsrc({ptr: JS_NewContext(self.ptr, default_stacksize)})
    }
}

// ___________________________________________________________________________
// contexts

type cx = @cx_rsrc;
resource cx_rsrc(self: {ptr: *JSContext}) {
    JS_DestroyContext(self.ptr);
}

impl methods for cx {
    fn rooted_obj(obj: *JSObject) -> jsobj {
        let jsobj = @jsobj_rsrc({cx: self.ptr, obj: obj});
        JS_AddObjectRoot(self.ptr, ptr::addr_of(jsobj.obj));
        jsobj
    }

    fn new_global(globcls: named_class) -> result<jsobj,()> {
        let globobj =
            JS_NewCompartmentAndGlobalObject(
                self.ptr,
                addr_of(globcls.jsclass),
                null());
        result(JS_InitStandardClasses(self.ptr, globobj)).chain { |_ok|
            ok(self.rooted_obj(globobj))
        }
    }
}

// ___________________________________________________________________________
// objects

type jsobj = @jsobj_rsrc;

resource jsobj_rsrc(self: {cx: *JSContext, obj: *JSObject}) {
    JS_RemoveObjectRoot(self.cx, ptr::addr_of(self.obj));
}

