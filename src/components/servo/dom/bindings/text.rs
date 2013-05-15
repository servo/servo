/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::element;
use dom::bindings::node::unwrap;
use dom::bindings::utils;
use dom::bindings::utils::{DOM_OBJECT_SLOT, CacheableWrapper};
use dom::node::{AbstractNode, Text, Comment, Doctype, TextNodeTypeId, CommentNodeTypeId};
use dom::node::{DoctypeNodeTypeId};

use js::jsapi::{JSFreeOp, JSObject, JSContext};
use js::jsapi::bindgen::{JS_SetReservedSlot};
use js::glue::bindgen::{RUST_PRIVATE_TO_JSVAL};
use js::rust::{Compartment, jsobj};

extern fn finalize_text(_fop: *JSFreeOp, obj: *JSObject) {
    debug!("text finalize: %?!", obj as uint);
    unsafe {
        let node: AbstractNode = unwrap(obj);
        let _elem: ~Text = cast::transmute(node.raw_object());
    }
}

extern fn finalize_comment(_fop: *JSFreeOp, obj: *JSObject) {
    debug!("comment finalize: %?!", obj as uint);
    unsafe {
        let node: AbstractNode = unwrap(obj);
        let _elem: ~Comment = cast::transmute(node.raw_object());
    }
}

extern fn finalize_doctype(_fop: *JSFreeOp, obj: *JSObject) {
    debug!("doctype finalize: %?!", obj as uint);
    unsafe {
        let node: AbstractNode = unwrap(obj);
        let _elem: ~Doctype = cast::transmute(node.raw_object());
    }
}

pub fn init(compartment: @mut Compartment) {
    let _ = utils::define_empty_prototype(~"CharacterData", Some(~"Node"), compartment);
    
    let _ = utils::define_empty_prototype(~"TextPrototype",
                                          Some(~"CharacterData"),
                                          compartment);
    let _ = utils::define_empty_prototype(~"CommentPrototype",
                                          Some(~"CharacterData"),
                                          compartment);
    let _ = utils::define_empty_prototype(~"DocumentTypePrototype",
                                          Some(~"Node"),
                                          compartment);

    compartment.register_class(utils::instance_jsclass(~"Text",
                                                       finalize_text,
                                                       element::trace));
    compartment.register_class(utils::instance_jsclass(~"Comment",
                                                       finalize_comment,
                                                       element::trace));
    compartment.register_class(utils::instance_jsclass(~"DocumentType",
                                                       finalize_doctype,
                                                       element::trace));

    
}

pub fn create(cx: *JSContext, node: &mut AbstractNode) -> jsobj {
    let (proto, instance) = match node.type_id() {
      TextNodeTypeId => (~"TextPrototype", ~"Text"),
      CommentNodeTypeId => (~"CommentPrototype", ~"Comment"),
      DoctypeNodeTypeId => (~"DocumentTypePrototype", ~"DocumentType"),
      _ => fail!(~"text::create only handles textual nodes")
    };

    //XXXjdm the parent should probably be the node parent instead of the global
    //TODO error checking
    let compartment = utils::get_compartment(cx);
    let obj = result::unwrap(compartment.new_object_with_proto(instance,
                                                               proto,
                                                               compartment.global_obj.ptr));

    let cache = node.get_wrappercache();
    assert!(cache.get_wrapper().is_null());
    cache.set_wrapper(obj.ptr);

    let raw_ptr = node.raw_object() as *libc::c_void;
    JS_SetReservedSlot(obj.ptr, DOM_OBJECT_SLOT as u32, RUST_PRIVATE_TO_JSVAL(raw_ptr));

    return obj;
}
