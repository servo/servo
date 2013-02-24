use content::content_task::{Content, task_from_context};
use dom::bindings::node::unwrap;
use dom::bindings::utils::{rust_box, squirrel_away_unique, get_compartment};
use dom::bindings::utils::{domstring_to_jsval, WrapNewBindingObject};
use dom::bindings::utils::{str};
use dom::element::*;
use dom::node::{AbstractNode, Node, Element, ElementNodeTypeId};
use layout::layout_task;
use super::utils;

use core::libc::c_uint;
use core::ptr::null;
use js::crust::{JS_PropertyStub, JS_StrictPropertyStub, JS_EnumerateStub, JS_ConvertStub};
use js::glue::bindgen::*;
use js::jsapi::bindgen::*;
use js::jsapi::{JSContext, JSVal, JSObject, JSBool, jsid, JSClass, JSFreeOp, JSPropertySpec};
use js::jsapi::{JSPropertyOpWrapper, JSStrictPropertyOpWrapper};
use js::rust::{Compartment, jsobj};
use js::{JS_ARGV, JSCLASS_HAS_RESERVED_SLOTS, JSPROP_ENUMERATE, JSPROP_SHARED, JSVAL_NULL};
use js::{JS_THIS_OBJECT, JS_SET_RVAL, JSPROP_NATIVE_ACCESSORS};

extern fn finalize(_fop: *JSFreeOp, obj: *JSObject) {
    debug!("element finalize!");
    unsafe {
        let val = JS_GetReservedSlot(obj, 0);
        let _node: ~AbstractNode = cast::reinterpret_cast(&RUST_JSVAL_TO_PRIVATE(val));
    }
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
                                     selfHostedName: null()}];
    vec::as_imm_buf(*methods, |fns, _len| {
        JS_DefineFunctions(compartment.cx.ptr, obj.ptr, fns);
    });

    compartment.register_class(utils::instance_jsclass(~"GenericElementInstance",
                                                       finalize));

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

/*trait Element: utils::CacheableWrapper {
    fn getClientRects() -> Option<@ClientRectListImpl>;
}*/

/*extern fn getClientRects(cx: *JSContext, argc: c_uint, vp: *JSVal) -> JSBool {
  unsafe {
    let self: @Element =
        cast::reinterpret_cast(&utils::unwrap::<ElementData>(JS_THIS_OBJECT(cx, vp)));
    let rval = self.getClientRects();
    if rval.is_none() {
      JS_SET_RVAL(cx, vp, JSVAL_NULL);
    } else {
      assert WrapNewBindingObject(cx, (self as utils::CacheableWrapper).get_wrapper(), rval.get(), cast::transmute(vp));
    }
    cast::forget(self);
    return 1;
  }
}*/

#[allow(non_implicitly_copyable_typarams)]
extern fn HTMLImageElement_getWidth(cx: *JSContext, _argc: c_uint, vp: *mut JSVal) -> JSBool {
    unsafe {
        let obj = JS_THIS_OBJECT(cx, cast::transmute(vp));
        if obj.is_null() {
            return 0;
        }

        let node = &(*unwrap(obj)).payload;
        let width = match node.type_id() {
            ElementNodeTypeId(HTMLImageElementTypeId) => {
                let content = task_from_context(cx);
                let node = Node::as_abstract_node(~*node);
                match (*content).query_layout(layout_task::ContentBox(node)) {
                    Ok(rect) => rect.width,
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
        let obj = JS_THIS_OBJECT(cx, cast::reinterpret_cast(&vp));
        if obj.is_null() {
            return 0;
        }

        let node = &(*unwrap(obj)).payload;
        match node.type_id() {
            ElementNodeTypeId(HTMLImageElementTypeId) => {
                do node.as_mut_element |elem| {
                    let arg = ptr::offset(JS_ARGV(cx, cast::reinterpret_cast(&vp)), 0);
                    elem.set_attr(~"width", (RUST_JSVAL_TO_INT(*arg) as int).to_str())
                }
            }
            ElementNodeTypeId(_) => fail!(~"why is this not an image element?"),
            _ => fail!(~"why is this not an element?")
        };
        return 1;
    }
}

#[allow(non_implicitly_copyable_typarams)]
extern fn getTagName(cx: *JSContext, _argc: c_uint, vp: *mut JSVal) -> JSBool {
    unsafe {
        let obj = JS_THIS_OBJECT(cx, cast::reinterpret_cast(&vp));
        if obj.is_null() {
            return 0;
        }

        let node = &(*unwrap(obj)).payload;
        do node.with_imm_element |elem| {
            let s = str(copy elem.tag_name);
            *vp = domstring_to_jsval(cx, &s);            
        }
    }
    return 1;
}

#[allow(non_implicitly_copyable_typarams)]
pub fn create(cx: *JSContext, node: AbstractNode) -> jsobj {
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
 
    unsafe {
        let raw_ptr: *libc::c_void =
            cast::reinterpret_cast(&squirrel_away_unique(~node));
        JS_SetReservedSlot(obj.ptr, 0, RUST_PRIVATE_TO_JSVAL(raw_ptr));
    }
    
    return obj;
}
