/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::types::*;
use dom::bindings::codegen::*;
use dom::bindings::node::unwrap;
use dom::bindings::utils::jsval_to_str;
use dom::bindings::utils::{domstring_to_jsval, WrapNewBindingObject};
use dom::bindings::utils::{str, CacheableWrapper, DOM_OBJECT_SLOT, DOMString};
use dom::bindings::utils::{BindingObject, WrapperCache};
use dom::element::Element;
use dom::element::{HTMLImageElementTypeId, HTMLHeadElementTypeId, HTMLScriptElementTypeId,
                   HTMLDivElementTypeId};
use dom::node::{AbstractNode, ScriptView, ElementNodeTypeId};
use layout_interface::{ContentBoxQuery, ContentBoxResponse};
use script_task::page_from_context;
use super::utils;

use std::cast;
use std::i32;
use std::libc;
use std::libc::c_uint;
use std::comm;
use std::ptr;
use std::ptr::null;
use js::glue::*;
use js::jsapi::*;
use js::jsapi::{JSContext, JSVal, JSObject, JSBool, JSFreeOp, JSPropertySpec};
use js::jsapi::{JSNativeWrapper, JSTracer, JSTRACE_OBJECT};
use js::jsapi::{JSPropertyOpWrapper, JSStrictPropertyOpWrapper, JSFunctionSpec};
use js::rust::{Compartment, jsobj};
use js::{JS_ARGV, JSPROP_ENUMERATE, JSPROP_SHARED};
use js::{JS_THIS_OBJECT, JSPROP_NATIVE_ACCESSORS};

extern fn finalize(_fop: *JSFreeOp, obj: *JSObject) {
    debug!("element finalize: %x!", obj as uint);
    unsafe {
        let node: AbstractNode<ScriptView> = unwrap(obj);
        //XXXjdm We need separate finalizers for each specialty element type like headings
        let _elem: @Element = cast::transmute(node.raw_object());
    }
}

pub extern fn trace(tracer: *mut JSTracer, obj: *JSObject) {
    let node = unsafe { unwrap(obj) };

    fn trace_node(tracer: *mut JSTracer, node: Option<AbstractNode<ScriptView>>, name: &str) {
        if node.is_none() {
            return;
        }
        error!("tracing %s", name);
        let mut node = node.unwrap();
        let cache = node.get_wrappercache();
        let wrapper = cache.get_wrapper();
        assert!(wrapper.is_not_null());
        unsafe {
            (*tracer).debugPrinter = ptr::null();
            (*tracer).debugPrintIndex = -1;
            do name.to_c_str().with_ref |name| {
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
    compartment.global_props.push(attrs);
    do attrs.as_imm_buf |specs, _len| {
        unsafe {
            JS_DefineProperties(compartment.cx.ptr, obj.ptr, specs);
        }
    }

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
    do methods.as_imm_buf |fns, _len| {
        unsafe {
            JS_DefineFunctions(compartment.cx.ptr, obj.ptr, fns);
        }
    }

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
    compartment.global_props.push(attrs);
    do attrs.as_imm_buf |specs, _len| {
        unsafe {
            JS_DefineProperties(compartment.cx.ptr, obj.ptr, specs);
        }
    }
}

extern fn getClientRects(cx: *JSContext, _argc: c_uint, vp: *JSVal) -> JSBool {
  unsafe {
      let obj = JS_THIS_OBJECT(cx, vp);
      let mut node = unwrap(obj);
      let rval = do node.with_imm_element |elem| {
          elem.GetClientRects(node)
      };
      let cache = node.get_wrappercache();
      let rval = rval as @mut CacheableWrapper;
      assert!(WrapNewBindingObject(cx, cache.get_wrapper(),
                                   rval,
                                   cast::transmute(vp)));
      return 1;
  }
}

extern fn getBoundingClientRect(cx: *JSContext, _argc: c_uint, vp: *JSVal) -> JSBool {
  unsafe {
      let obj = JS_THIS_OBJECT(cx, vp);
      let mut node = unwrap(obj);
      let rval = do node.with_imm_element |elem| {
          elem.GetBoundingClientRect(node)
      };
      let cache = node.get_wrappercache();
      let rval = rval as @mut CacheableWrapper;
      assert!(WrapNewBindingObject(cx, cache.get_wrapper(),
                                   rval,
                                   cast::transmute(vp)));
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
        arg0 = str(strval.unwrap());

        let arg1: DOMString;
        let strval = jsval_to_str(cx, (*argv.offset(1)));
        if strval.is_err() {
            return 0;
        }
        arg1 = str(strval.unwrap());

        do node.as_mut_element |elem| {
            elem.set_attr(&arg0, &arg1);
        };

        return 1;
    }
}

extern fn HTMLImageElement_getWidth(cx: *JSContext, _argc: c_uint, vp: *mut JSVal) -> JSBool {
    unsafe {
        let obj = JS_THIS_OBJECT(cx, cast::transmute(vp));
        if obj.is_null() {
            return 0;
        }

        let node = unwrap(obj);
        let width = match node.type_id() {
            ElementNodeTypeId(HTMLImageElementTypeId) => {
                let page = page_from_context(cx);
                let (port, chan) = comm::stream();
                // TODO(tkuehn): currently this just queries top-level page's layout. Need to handle subframes.
                match (*page).query_layout(ContentBoxQuery(node, chan), port) {
                    Ok(ContentBoxResponse(rect)) => rect.size.width.to_nearest_px(),
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
            let s = str(elem.tag_name.clone());
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
    let obj = compartment.new_object_with_proto(~"GenericElementInstance",
                                                proto,
                                                compartment.global_obj.ptr).unwrap();

    let cache = node.get_wrappercache();
    assert!(cache.get_wrapper().is_null());
    cache.set_wrapper(obj.ptr);

    unsafe {
        let raw_ptr = node.raw_object() as *libc::c_void;
        JS_SetReservedSlot(obj.ptr, DOM_OBJECT_SLOT as u32, RUST_PRIVATE_TO_JSVAL(raw_ptr));
    }

    return obj;
}

pub macro_rules! generate_cacheable_wrapper(
    ($name: path, $wrap: path) => (
        impl CacheableWrapper for $name {
            fn get_wrappercache(&mut self) -> &mut WrapperCache {
                self.parent.get_wrappercache()
            }

            fn wrap_object_shared(@mut self, cx: *JSContext, scope: *JSObject) -> *JSObject {
                let mut unused = false;
                $wrap(cx, scope, self, &mut unused)
            }
        }
    )
)

pub macro_rules! generate_binding_object(
    ($name: path) => (
        impl BindingObject for $name {
            fn GetParentObject(&self, cx: *JSContext) -> Option<@mut CacheableWrapper> {
                self.parent.GetParentObject(cx)
            }
        }
    )
)

generate_cacheable_wrapper!(Comment, CommentBinding::Wrap)
generate_binding_object!(Comment)
generate_cacheable_wrapper!(DocumentType<ScriptView>, DocumentTypeBinding::Wrap)
generate_binding_object!(DocumentType<ScriptView>)
generate_cacheable_wrapper!(Text, TextBinding::Wrap)
generate_binding_object!(Text)
generate_cacheable_wrapper!(HTMLHeadElement, HTMLHeadElementBinding::Wrap)
generate_binding_object!(HTMLHeadElement)
generate_cacheable_wrapper!(HTMLAnchorElement, HTMLAnchorElementBinding::Wrap)
generate_binding_object!(HTMLAnchorElement)
generate_cacheable_wrapper!(HTMLAppletElement, HTMLAppletElementBinding::Wrap)
generate_binding_object!(HTMLAppletElement)
generate_cacheable_wrapper!(HTMLAreaElement, HTMLAreaElementBinding::Wrap)
generate_binding_object!(HTMLAreaElement)
generate_cacheable_wrapper!(HTMLBaseElement, HTMLBaseElementBinding::Wrap)
generate_binding_object!(HTMLBaseElement)
generate_cacheable_wrapper!(HTMLBodyElement, HTMLBodyElementBinding::Wrap)
generate_binding_object!(HTMLBodyElement)
generate_cacheable_wrapper!(HTMLButtonElement, HTMLButtonElementBinding::Wrap)
generate_binding_object!(HTMLButtonElement)
generate_cacheable_wrapper!(HTMLCanvasElement, HTMLCanvasElementBinding::Wrap)
generate_binding_object!(HTMLCanvasElement)
generate_cacheable_wrapper!(HTMLDataListElement, HTMLDataListElementBinding::Wrap)
generate_binding_object!(HTMLDataListElement)
generate_cacheable_wrapper!(HTMLDListElement, HTMLDListElementBinding::Wrap)
generate_binding_object!(HTMLDListElement)
generate_cacheable_wrapper!(HTMLFrameElement, HTMLFrameElementBinding::Wrap)
generate_binding_object!(HTMLFrameElement)
generate_cacheable_wrapper!(HTMLFrameSetElement, HTMLFrameSetElementBinding::Wrap)
generate_binding_object!(HTMLFrameSetElement)
generate_cacheable_wrapper!(HTMLBRElement, HTMLBRElementBinding::Wrap)
generate_binding_object!(HTMLBRElement)
generate_cacheable_wrapper!(HTMLHRElement, HTMLHRElementBinding::Wrap)
generate_binding_object!(HTMLHRElement)
generate_cacheable_wrapper!(HTMLHtmlElement, HTMLHtmlElementBinding::Wrap)
generate_binding_object!(HTMLHtmlElement)
generate_cacheable_wrapper!(HTMLDataElement, HTMLDataElementBinding::Wrap)
generate_binding_object!(HTMLDataElement)
generate_cacheable_wrapper!(HTMLDirectoryElement, HTMLDirectoryElementBinding::Wrap)
generate_binding_object!(HTMLDirectoryElement)
generate_cacheable_wrapper!(HTMLDivElement, HTMLDivElementBinding::Wrap)
generate_binding_object!(HTMLDivElement)
generate_cacheable_wrapper!(HTMLEmbedElement, HTMLEmbedElementBinding::Wrap)
generate_binding_object!(HTMLEmbedElement)
generate_cacheable_wrapper!(HTMLFieldSetElement, HTMLFieldSetElementBinding::Wrap)
generate_binding_object!(HTMLFieldSetElement)
generate_cacheable_wrapper!(HTMLFontElement, HTMLFontElementBinding::Wrap)
generate_binding_object!(HTMLFontElement)
generate_cacheable_wrapper!(HTMLHeadingElement, HTMLHeadingElementBinding::Wrap)
generate_binding_object!(HTMLHeadingElement)
generate_cacheable_wrapper!(HTMLIFrameElement, HTMLIFrameElementBinding::Wrap)
generate_binding_object!(HTMLIFrameElement)
generate_cacheable_wrapper!(HTMLImageElement, HTMLImageElementBinding::Wrap)
generate_binding_object!(HTMLImageElement)
generate_cacheable_wrapper!(HTMLInputElement, HTMLInputElementBinding::Wrap)
generate_binding_object!(HTMLInputElement)
generate_cacheable_wrapper!(HTMLLIElement, HTMLLIElementBinding::Wrap)
generate_binding_object!(HTMLLIElement)
generate_cacheable_wrapper!(HTMLLinkElement, HTMLLinkElementBinding::Wrap)
generate_binding_object!(HTMLLinkElement)
generate_cacheable_wrapper!(HTMLMapElement, HTMLMapElementBinding::Wrap)
generate_binding_object!(HTMLMapElement)
generate_cacheable_wrapper!(HTMLMetaElement, HTMLMetaElementBinding::Wrap)
generate_binding_object!(HTMLMetaElement)
generate_cacheable_wrapper!(HTMLMeterElement, HTMLMeterElementBinding::Wrap)
generate_binding_object!(HTMLMeterElement)
generate_cacheable_wrapper!(HTMLModElement, HTMLModElementBinding::Wrap)
generate_binding_object!(HTMLModElement)
generate_cacheable_wrapper!(HTMLObjectElement, HTMLObjectElementBinding::Wrap)
generate_binding_object!(HTMLObjectElement)
generate_cacheable_wrapper!(HTMLOListElement, HTMLOListElementBinding::Wrap)
generate_binding_object!(HTMLOListElement)
generate_cacheable_wrapper!(HTMLOptGroupElement, HTMLOptGroupElementBinding::Wrap)
generate_binding_object!(HTMLOptGroupElement)
generate_cacheable_wrapper!(HTMLOptionElement, HTMLOptionElementBinding::Wrap)
generate_binding_object!(HTMLOptionElement)
generate_cacheable_wrapper!(HTMLOutputElement, HTMLOutputElementBinding::Wrap)
generate_binding_object!(HTMLOutputElement)
generate_cacheable_wrapper!(HTMLParagraphElement, HTMLParagraphElementBinding::Wrap)
generate_binding_object!(HTMLParagraphElement)
generate_cacheable_wrapper!(HTMLParamElement, HTMLParamElementBinding::Wrap)
generate_binding_object!(HTMLParamElement)
generate_cacheable_wrapper!(HTMLProgressElement, HTMLProgressElementBinding::Wrap)
generate_binding_object!(HTMLProgressElement)
generate_cacheable_wrapper!(HTMLQuoteElement, HTMLQuoteElementBinding::Wrap)
generate_binding_object!(HTMLQuoteElement)
generate_cacheable_wrapper!(HTMLScriptElement, HTMLScriptElementBinding::Wrap)
generate_binding_object!(HTMLScriptElement)
generate_cacheable_wrapper!(HTMLSelectElement, HTMLSelectElementBinding::Wrap)
generate_binding_object!(HTMLSelectElement)
generate_cacheable_wrapper!(HTMLSourceElement, HTMLSourceElementBinding::Wrap)
generate_binding_object!(HTMLSourceElement)
generate_cacheable_wrapper!(HTMLSpanElement, HTMLSpanElementBinding::Wrap)
generate_binding_object!(HTMLSpanElement)
generate_cacheable_wrapper!(HTMLStyleElement, HTMLStyleElementBinding::Wrap)
generate_binding_object!(HTMLStyleElement)
generate_cacheable_wrapper!(HTMLTableElement, HTMLTableElementBinding::Wrap)
generate_binding_object!(HTMLTableElement)
generate_cacheable_wrapper!(HTMLTableCaptionElement, HTMLTableCaptionElementBinding::Wrap)
generate_binding_object!(HTMLTableCaptionElement)
generate_cacheable_wrapper!(HTMLTableCellElement, HTMLTableCellElementBinding::Wrap)
generate_binding_object!(HTMLTableCellElement)
generate_cacheable_wrapper!(HTMLTableColElement, HTMLTableColElementBinding::Wrap)
generate_binding_object!(HTMLTableColElement)
generate_cacheable_wrapper!(HTMLTableRowElement, HTMLTableRowElementBinding::Wrap)
generate_binding_object!(HTMLTableRowElement)
generate_cacheable_wrapper!(HTMLTableSectionElement, HTMLTableSectionElementBinding::Wrap)
generate_binding_object!(HTMLTableSectionElement)
generate_cacheable_wrapper!(HTMLTextAreaElement, HTMLTextAreaElementBinding::Wrap)
generate_binding_object!(HTMLTextAreaElement)
generate_cacheable_wrapper!(HTMLTitleElement, HTMLTitleElementBinding::Wrap)
generate_binding_object!(HTMLTitleElement)
generate_cacheable_wrapper!(HTMLTimeElement, HTMLTimeElementBinding::Wrap)
generate_binding_object!(HTMLTimeElement)
generate_cacheable_wrapper!(HTMLUListElement, HTMLUListElementBinding::Wrap)
generate_binding_object!(HTMLUListElement)
generate_cacheable_wrapper!(HTMLUnknownElement, HTMLUnknownElementBinding::Wrap)
generate_binding_object!(HTMLUnknownElement)
