import jsapi::*;
import jsapi::bindgen::*;
import ptr::{null, addr_of};
import result::{result, ok, err, extensions};
import libc::c_char;

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
        @cx_rsrc({ptr: JS_NewContext(self.ptr, default_stacksize),
                  rt: self})
    }
}

// ___________________________________________________________________________
// contexts

type cx = @cx_rsrc;
resource cx_rsrc(self: {ptr: *JSContext, rt: rt}) {
    JS_DestroyContext(self.ptr);
}

impl methods for cx {
    fn rooted_obj(obj: *JSObject) -> jsobj {
        let jsobj = @jsobj_rsrc({cx: self, cxptr: self.ptr, ptr: obj});
        JS_AddObjectRoot(self.ptr, ptr::addr_of(jsobj.ptr));
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

    fn evaluate_script(glob: jsobj, bytes: [u8], filename: str,
                       line_num: uint) -> result<(),()> {
        vec::as_buf(bytes) { |bytes_ptr|
            str::as_c_str(filename) { |filename_cstr|
                let bytes_ptr = bytes_ptr as *c_char;
                let v: jsval = 0_u64;
                #debug["Evaluating script from %s", filename];
                if JS_EvaluateScript(self.ptr, glob.ptr,
                                     bytes_ptr, bytes.len() as uintN,
                                     filename_cstr, line_num as uintN,
                                     ptr::addr_of(v)) == ERR {
                    #debug["...err!"];
                    err(())
                } else {
                    // we could return the script result but then we'd have
                    // to root it and so forth and, really, who cares?
                    #debug["...ok!"];
                    ok(())
                }
            }
        }
    }
}

// ___________________________________________________________________________
// objects

type jsobj = @jsobj_rsrc;

resource jsobj_rsrc(self: {cx: cx, cxptr: *JSContext, ptr: *JSObject}) {
    JS_RemoveObjectRoot(self.cxptr, ptr::addr_of(self.ptr));
}

#[cfg(test)]
mod test {

    #[test]
    fn dummy() {
        let rt = rt();
        let cx = rt.cx();
        let gc = jsglobal::global_class();
        cx.new_global(gc).chain {
            |glob|
            str::bytes("x = 1;") {
                |bytes|
                cx.evaluate_script(glob, bytes, "test", 1u);
            }
        };
    }

}