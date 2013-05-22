/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::node::unwrap;
use dom::bindings::utils::jsval_to_str;
use dom::bindings::utils::{domstring_to_jsval, WrapNewBindingObject};
use dom::bindings::utils::{str, CacheableWrapper, DOM_OBJECT_SLOT, DOMString};
use dom::element::*;
use dom::node::{AbstractNode, Element, ElementNodeTypeId, ScriptView};
use layout_interface::{ContentBoxQuery, ContentBoxResponse};
use script_task::task_from_context;
use super::utils;

use core::libc::c_uint;
use core::ptr::null;
use js::glue::bindgen::*;
use js::jsapi::bindgen::*;
use js::jsapi::{JSContext, JSVal, JSObject, JSBool, JSFreeOp, JSPropertySpec};
use js::jsapi::{JSNativeWrapper, JSTracer, JSTRACE_OBJECT};
use js::jsapi::{JSPropertyOpWrapper, JSStrictPropertyOpWrapper, JSFunctionSpec};
use js::rust::{Compartment, jsobj};
use js::{JS_ARGV, JSPROP_ENUMERATE, JSPROP_SHARED, JSVAL_NULL};
use js::{JS_THIS_OBJECT, JS_SET_RVAL, JSPROP_NATIVE_ACCESSORS};

extern fn finalize(_fop: *JSFreeOp, obj: *JSObject) {
    debug!("element finalize: %x!", obj as uint);
    unsafe {
        let node: AbstractNode<ScriptView> = unwrap(obj);
        //XXXjdm We need separate finalizers for each specialty element type like headings
        let _elem: ~Element = cast::transmute(node.raw_object());
    }
}

pub extern fn trace(tracer: *mut JSTracer, obj: *JSObject) {
    let node = unsafe { unwrap(obj) };

    fn trace_node(tracer: *mut JSTracer, node: Option<AbstractNode<ScriptView>>, name: &str) {
        if node.is_none() {
            return;
        }
        error!("tracing %s", name);
        let mut node = node.get();
        let cache = node.get_wrappercache();
        let wrapper = cache.get_wrapper();
        assert!(wrapper.is_not_null());
        unsafe {
            (*tracer).debugPrinter = ptr::null();
            (*tracer).debugPrintIndex = -1;
            do str::as_c_str(name) |name| {
                (*tracer).debugPrintArg = name as *libc::c_void;
                JS_CallTracer(cast::transmute(tracer), wrapper, JSTRACE_OBJECT as u32);
            }
        }
    }
    error!("tracing %?:", obj as uint);
    trace_node(tracer, node.parent_node(), "parent");
    trace_node(tracer, node.first_child(), "first child");
    trace_node(tracer, node.last_child(), "last child");
    trace_node(tracer, node.next_sibling(), "next sibling");
    trace_node(tracer, node.prev_sibling(), "prev sibling");
}

pub fn init(compartment: @mut Compartment) {
    let obj = utils::define_empty_prototype(~"Element", Some(~"Node"), compartment);
    let attrs = @~[
        JSPropertySpec {
         name: compartment.add_name(~"tagName"),
         tinyid: 0,
         flags: (JSPROP_ENUMERATE | JSPROP_SHARED | JSPROP_NATIVE_ACCESSORS) as u8,
         getter: JSPropertyOpWrapper {op: getTagName, info: null()},
         setter: JSStrictPropertyOpWrapper {op: null(), info: null()}},
        JSPropertySpec {
         name: null(),
         tinyid: 0,
         flags: (JSPROP_SHARED | JSPROP_ENUMERATE | JSPROP_NATIVE_ACCESSORS) as u8,
         getter: JSPropertyOpWrapper {op: null(), info: null()},
         setter: JSStrictPropertyOpWrapper {op: null(), info: null()}}];
    vec::push(&mut compartment.global_props, attrs);
    vec::as_imm_buf(*attrs, |specs, _len| {
        JS_DefineProperties(compartment.cx.ptr, obj.ptr, specs);
    });

    let methods = @~[JSFunctionSpec {name: compartment.add_name(~"getClientRects"),
                                     call: JSNativeWrapper {op: getClientRects, info: null()},
                                     nargs: 0,
                                     flags: 0,
                                     selfHostedName: null()},
                     JSFunctionSpec {name: compartment.add_name(~"getBoundingClientRect"),
                                     call: JSNativeWrapper {op: getBoundingClientRect, info: null()},
                                     nargs: 0,
                                     flags: 0,
                                     selfHostedName: null()},
                     JSFunctionSpec {name: compartment.add_name(~"setAttribute"),
                                     call: JSNativeWrapper {op: setAttribute, info: null()},
                                     nargs: 0,
                                     flags: 0,
                                     selfHostedName: null()},
                     JSFunctionSpec {name: null(),
                                     call: JSNativeWrapper {op: null(), info: null()},
                                     nargs: 0,
                                     flags: 0,
                                     selfHostedName: null()}];
    vec::as_imm_buf(*methods, |fns, _len| {
        JS_DefineFunctions(compartment.cx.ptr, obj.ptr, fns);
    });

    compartment.register_class(utils::instance_jsclass(~"GenericElementInstance",
                                                       finalize, trace));

    let _ = utils::define_empty_prototype(~"HTMLElement", Some(~"Element"), compartment);
    let _ = utils::define_empty_prototype(~"HTMLDivElement", Some(~"HTMLElement"), compartment);
    let _ = utils::define_empty_prototype(~"HTMLScriptElement", Some(~"HTMLElement"), compartment);
    let _ = utils::define_empty_prototype(~"HTMLHeadElement", Some(~"HTMLElement"), compartment);

    let obj = utils::define_empty_prototype(~"HTMLImageElement", Some(~"HTMLElement"), compartment);
    let attrs = @~[
        JSPropertySpec {name: compartment.add_name(~"width"),
         tinyid: 0,
         flags: (JSPROP_SHARED | JSPROP_ENUMERATE | JSPROP_NATIVE_ACCESSORS) as u8,
         getter: JSPropertyOpWrapper {op: HTMLImageElement_getWidth, info: null()},
         setter: JSStrictPropertyOpWrapper {op: HTMLImageElement_setWidth, info: null()}},
        JSPropertySpec {name: null(),
         tinyid: 0,
         flags: (JSPROP_SHARED | JSPROP_ENUMERATE | JSPROP_NATIVE_ACCESSORS) as u8,
         getter: JSPropertyOpWrapper {op: null(), info: null()},
         setter: JSStrictPropertyOpWrapper {op: null(), info: null()}}];
    vec::push(&mut compartment.global_props, attrs);
    vec::as_imm_buf(*attrs, |specs, _len| {
        JS_DefineProperties(compartment.cx.ptr, obj.ptr, specs);
    });
}

extern fn getClientRects(cx: *JSContext, _argc: c_uint, vp: *JSVal) -> JSBool {
  unsafe {
      let obj = JS_THIS_OBJECT(cx, vp);
      let mut node = unwrap(obj);
      let rval = do node.with_imm_element |elem| {
          elem.getClientRects()
      };
      if rval.is_none() {
          JS_SET_RVAL(cx, vp, JSVAL_NULL);
      } else {
          let cache = node.get_wrappercache();
          let rval = rval.get() as @mut CacheableWrapper;
          assert!(WrapNewBindingObject(cx, cache.get_wrapper(),
                                       rval,
                                       cast::transmute(vp)));
      }
      return 1;
  }
}

extern fn getBoundingClientRect(cx: *JSContext, _argc: c_uint, vp: *JSVal) -> JSBool {
  unsafe {
      let obj = JS_THIS_OBJECT(cx, vp);
      let mut node = unwrap(obj);
      let rval = do node.with_imm_element |elem| {
          elem.getBoundingClientRect()
      };
      if rval.is_none() {
          JS_SET_RVAL(cx, vp, JSVAL_NULL);
      } else {
          let cache = node.get_wrappercache();
          let rval = rval.get() as @mut CacheableWrapper;
          assert!(WrapNewBindingObject(cx, cache.get_wrapper(),
                                       rval,
                                       cast::transmute(vp)));
      }
      return 1;
  }
}

extern fn setAttribute(cx: *JSContext, argc: c_uint, vp: *JSVal) -> JSBool {
    unsafe {
        let obj = JS_THIS_OBJECT(cx, vp);
        let node = unwrap(obj);

        if (argc < 2) {
            return 0; //XXXjdm throw exception
        }

        let argv = JS_ARGV(cx, cast::transmute(vp));

        let arg0: DOMString;
        let strval = jsval_to_str(cx, (*argv.offset(0)));
        if strval.is_err() {
            return 0;
        }
        arg0 = str(strval.get());

        let arg1: DOMString;
        let strval = jsval_to_str(cx, (*argv.offset(1)));
        if strval.is_err() {
            return 0;
        }
        arg1 = str(strval.get());

        do node.as_mut_element |elem| {
            elem.set_attr(&arg0, &arg1);
        };

        return 1;
    }
}

#[allow(non_implicitly_copyable_typarams)]
extern fn HTMLImageElement_getWidth(cx: *JSContext, _argc: c_uint, vp: *mut JSVal) -> JSBool {
    unsafe {
        let obj = JS_THIS_OBJECT(cx, cast::transmute(vp));
        if obj.is_null() {
            return 0;
        }

        let node = unwrap(obj);
        let width = match node.type_id() {
            ElementNodeTypeId(HTMLImageElementTypeId) => {
                let script_context = task_from_context(cx);
                match (*script_context).query_layout(ContentBoxQuery(node)) {
                    Ok(rect) => {
                        match rect {
                            ContentBoxResponse(rect) => rect.size.width.to_px(),
                            _ => fail!(~"unexpected layout reply")
                        }
                    }
                    Err(()) => 0
                }
                // TODO: if nothing is being rendered(?), return zero dimensions
            }
            ElementNodeTypeId(_) => fail!(~"why is this not an image element?"),
            _ => fail!(~"why is this not an element?")
        };

        *vp = RUST_INT_TO_JSVAL(
                (width & (i32::max_value as int)) as libc::c_int);
        return 1;
    }
}

#[allow(non_implicitly_copyable_typarams)]
extern fn HTMLImageElement_setWidth(cx: *JSContext, _argc: c_uint, vp: *mut JSVal) -> JSBool {
    unsafe {
        let obj = JS_THIS_OBJECT(cx, cast::transmute(vp));
        if obj.is_null() {
            return 0;
        }

        let node = unwrap(obj);
        match node.type_id() {
            ElementNodeTypeId(HTMLImageElementTypeId) => {
                do node.as_mut_element |elem| {
                    let arg = ptr::offset(JS_ARGV(cx, cast::transmute(vp)), 0);
                    elem.set_attr(&str(~"width"),
                                  &str((RUST_JSVAL_TO_INT(*arg) as int).to_str()))
                }
            }
            ElementNodeTypeId(_) => fail!(~"why is this not an image element?"),
            _ => fail!(~"why is this not an element?")
        };

        return 1;
    }
}

extern fn getTagName(cx: *JSContext, _argc: c_uint, vp: *mut JSVal) -> JSBool {
    unsafe {
        let obj = JS_THIS_OBJECT(cx, cast::transmute(vp));
        if obj.is_null() {
            return 0;
        }

        let node = unwrap(obj);
        do node.with_imm_element |elem| {
            let s = str(copy elem.tag_name);
            *vp = domstring_to_jsval(cx, &s);            
        }
    }
    return 1;
}

pub fn create(cx: *JSContext, node: &mut AbstractNode<ScriptView>) -> jsobj {
    let proto = match node.type_id() {
        ElementNodeTypeId(HTMLDivElementTypeId) => ~"HTMLDivElement",
        ElementNodeTypeId(HTMLHeadElementTypeId) => ~"HTMLHeadElement",
        ElementNodeTypeId(HTMLImageElementTypeId) => ~"HTMLImageElement",
        ElementNodeTypeId(HTMLScriptElementTypeId) => ~"HTMLScriptElement",
        ElementNodeTypeId(_) => ~"HTMLElement",
        _ => fail!(~"element::create only handles elements")
    };

    //XXXjdm the parent should probably be the node parent instead of the global
    //TODO error checking
    let compartment = utils::get_compartment(cx);
    let obj = result::unwrap(compartment.new_object_with_proto(~"GenericElementInstance",
                                                               proto,
                                                               compartment.global_obj.ptr));

    let cache = node.get_wrappercache();
    assert!(cache.get_wrapper().is_null());
    cache.set_wrapper(obj.ptr);

    let raw_ptr = node.raw_object() as *libc::c_void;
    JS_SetReservedSlot(obj.ptr, DOM_OBJECT_SLOT as u32, RUST_PRIVATE_TO_JSVAL(raw_ptr));

    return obj;
}
