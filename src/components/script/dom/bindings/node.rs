/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::{CacheableWrapper, WrapperCache, Traceable};
use dom::element::*;
use dom::types::*;
use dom::node::{AbstractNode, ElementNodeTypeId, TextNodeTypeId, CommentNodeTypeId};
use dom::node::{DoctypeNodeTypeId, ScriptView};

use std::cast;
use std::libc;
use std::ptr;
use js::jsapi::{JSContext, JSObject, JSTracer, JSTRACE_OBJECT, JS_CallTracer};
use servo_util::tree::TreeNodeRef;

macro_rules! generate_element(
    ($name: path) => ({
        let node: @mut $name = unsafe { cast::transmute(node.raw_object()) };
        node.wrap_object_shared(cx, ptr::null())
    })
)

pub fn create(cx: *JSContext, node: &mut AbstractNode<ScriptView>) -> *JSObject {
    match node.type_id() {
        ElementNodeTypeId(HTMLElementTypeId) => generate_element!(HTMLElement),
        ElementNodeTypeId(HTMLAnchorElementTypeId) => generate_element!(HTMLAnchorElement),
        ElementNodeTypeId(HTMLAppletElementTypeId) => generate_element!(HTMLAppletElement),
        ElementNodeTypeId(HTMLAreaElementTypeId) => generate_element!(HTMLAreaElement),
        ElementNodeTypeId(HTMLAudioElementTypeId) => generate_element!(HTMLAudioElement),
        ElementNodeTypeId(HTMLBaseElementTypeId) => generate_element!(HTMLBaseElement),
        ElementNodeTypeId(HTMLBodyElementTypeId) => generate_element!(HTMLBodyElement),
        ElementNodeTypeId(HTMLBRElementTypeId) => generate_element!(HTMLBRElement),
        ElementNodeTypeId(HTMLButtonElementTypeId) => generate_element!(HTMLButtonElement),
        ElementNodeTypeId(HTMLCanvasElementTypeId) => generate_element!(HTMLCanvasElement),
        ElementNodeTypeId(HTMLDataElementTypeId) => generate_element!(HTMLDataElement),
        ElementNodeTypeId(HTMLDataListElementTypeId) => generate_element!(HTMLDataListElement),
        ElementNodeTypeId(HTMLDirectoryElementTypeId) => generate_element!(HTMLDirectoryElement),
        ElementNodeTypeId(HTMLDListElementTypeId) => generate_element!(HTMLDListElement),
        ElementNodeTypeId(HTMLDivElementTypeId) => generate_element!(HTMLDivElement),
        ElementNodeTypeId(HTMLEmbedElementTypeId) => generate_element!(HTMLEmbedElement),
        ElementNodeTypeId(HTMLFieldSetElementTypeId) => generate_element!(HTMLFieldSetElement),
        ElementNodeTypeId(HTMLFontElementTypeId) => generate_element!(HTMLFontElement),
        ElementNodeTypeId(HTMLFormElementTypeId) => generate_element!(HTMLFormElement),
        ElementNodeTypeId(HTMLFrameElementTypeId) => generate_element!(HTMLFrameElement),
        ElementNodeTypeId(HTMLFrameSetElementTypeId) => generate_element!(HTMLFrameSetElement),
        ElementNodeTypeId(HTMLHeadElementTypeId) => generate_element!(HTMLHeadElement),
        ElementNodeTypeId(HTMLHeadingElementTypeId) => generate_element!(HTMLHeadingElement),
        ElementNodeTypeId(HTMLHRElementTypeId) => generate_element!(HTMLHRElement),
        ElementNodeTypeId(HTMLHtmlElementTypeId) => generate_element!(HTMLHtmlElement),
        ElementNodeTypeId(HTMLIframeElementTypeId) => generate_element!(HTMLIFrameElement),
        ElementNodeTypeId(HTMLImageElementTypeId) => generate_element!(HTMLImageElement),
        ElementNodeTypeId(HTMLInputElementTypeId) => generate_element!(HTMLInputElement),
        ElementNodeTypeId(HTMLLabelElementTypeId) => generate_element!(HTMLLabelElement),
        ElementNodeTypeId(HTMLLegendElementTypeId) => generate_element!(HTMLLegendElement),
        ElementNodeTypeId(HTMLLIElementTypeId) => generate_element!(HTMLLIElement),
        ElementNodeTypeId(HTMLLinkElementTypeId) => generate_element!(HTMLLinkElement),
        ElementNodeTypeId(HTMLMapElementTypeId) => generate_element!(HTMLMapElement),
        ElementNodeTypeId(HTMLMediaElementTypeId) => generate_element!(HTMLMediaElement),
        ElementNodeTypeId(HTMLMetaElementTypeId) => generate_element!(HTMLMetaElement),
        ElementNodeTypeId(HTMLMeterElementTypeId) => generate_element!(HTMLMeterElement),
        ElementNodeTypeId(HTMLModElementTypeId) => generate_element!(HTMLModElement),
        ElementNodeTypeId(HTMLObjectElementTypeId) => generate_element!(HTMLObjectElement),
        ElementNodeTypeId(HTMLOListElementTypeId) => generate_element!(HTMLOListElement),
        ElementNodeTypeId(HTMLOptGroupElementTypeId) => generate_element!(HTMLOptGroupElement),
        ElementNodeTypeId(HTMLOptionElementTypeId) => generate_element!(HTMLOptionElement),
        ElementNodeTypeId(HTMLOutputElementTypeId) => generate_element!(HTMLOutputElement),
        ElementNodeTypeId(HTMLParagraphElementTypeId) => generate_element!(HTMLParagraphElement),
        ElementNodeTypeId(HTMLParamElementTypeId) => generate_element!(HTMLParamElement),
        ElementNodeTypeId(HTMLPreElementTypeId) => generate_element!(HTMLPreElement),
        ElementNodeTypeId(HTMLProgressElementTypeId) => generate_element!(HTMLProgressElement),
        ElementNodeTypeId(HTMLQuoteElementTypeId) => generate_element!(HTMLQuoteElement),
        ElementNodeTypeId(HTMLScriptElementTypeId) => generate_element!(HTMLScriptElement),
        ElementNodeTypeId(HTMLSelectElementTypeId) => generate_element!(HTMLSelectElement),
        ElementNodeTypeId(HTMLSourceElementTypeId) => generate_element!(HTMLSourceElement),
        ElementNodeTypeId(HTMLSpanElementTypeId) => generate_element!(HTMLSpanElement),
        ElementNodeTypeId(HTMLStyleElementTypeId) => generate_element!(HTMLStyleElement),
        ElementNodeTypeId(HTMLTableElementTypeId) => generate_element!(HTMLTableElement),
        ElementNodeTypeId(HTMLTableCellElementTypeId) => generate_element!(HTMLTableCellElement),
        ElementNodeTypeId(HTMLTableCaptionElementTypeId) => generate_element!(HTMLTableCaptionElement),
        ElementNodeTypeId(HTMLTableColElementTypeId) => generate_element!(HTMLTableColElement),
        ElementNodeTypeId(HTMLTableRowElementTypeId) => generate_element!(HTMLTableRowElement),
        ElementNodeTypeId(HTMLTableSectionElementTypeId) => generate_element!(HTMLTableSectionElement),
        ElementNodeTypeId(HTMLTemplateElementTypeId) => generate_element!(HTMLTemplateElement),
        ElementNodeTypeId(HTMLTextAreaElementTypeId) => generate_element!(HTMLTextAreaElement),
        ElementNodeTypeId(HTMLTimeElementTypeId) => generate_element!(HTMLTimeElement),
        ElementNodeTypeId(HTMLTitleElementTypeId) => generate_element!(HTMLTitleElement),
        ElementNodeTypeId(HTMLTrackElementTypeId) => generate_element!(HTMLTrackElement),
        ElementNodeTypeId(HTMLUListElementTypeId) => generate_element!(HTMLUListElement),
        ElementNodeTypeId(HTMLVideoElementTypeId) => generate_element!(HTMLVideoElement),
        ElementNodeTypeId(HTMLUnknownElementTypeId) => generate_element!(HTMLUnknownElement),
        CommentNodeTypeId => generate_element!(Comment),
        DoctypeNodeTypeId => generate_element!(DocumentType<ScriptView>),
        TextNodeTypeId => generate_element!(Text)
     }
}

impl CacheableWrapper for AbstractNode<ScriptView> {
    fn get_wrappercache(&mut self) -> &mut WrapperCache {
        do self.with_mut_base |base| {
            unsafe {
                cast::transmute(&base.wrapper)
            }
        }
    }

    fn wrap_object_shared(@mut self, _cx: *JSContext, _scope: *JSObject) -> *JSObject {
        fail!(~"need to implement wrapping");
    }
}

impl Traceable for Node<ScriptView> {
    fn trace(&self, tracer: *mut JSTracer) {
        #[fixed_stack_segment]
        fn trace_node(tracer: *mut JSTracer, node: Option<AbstractNode<ScriptView>>, name: &str) {
            if node.is_none() {
                return;
            }
            debug!("tracing %s", name);
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
        debug!("tracing %p?:", self.wrapper.get_wrapper());
        trace_node(tracer, self.parent_node, "parent");
        trace_node(tracer, self.first_child, "first child");
        trace_node(tracer, self.last_child, "last child");
        trace_node(tracer, self.next_sibling, "next sibling");
        trace_node(tracer, self.prev_sibling, "prev sibling");
    }
}
