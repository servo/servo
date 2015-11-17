/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use devtools_traits::{CONSOLE_API, CachedConsoleMessage, CachedConsoleMessageTypes, PAGE_ERROR};
use devtools_traits::{ComputedNodeLayout, ConsoleAPI, PageError, ScriptToDevtoolsControlMsg};
use devtools_traits::{EvaluateJSReply, Modification, NodeInfo, TimelineMarker, TimelineMarkerType};
use dom::bindings::codegen::Bindings::DOMRectBinding::DOMRectMethods;
use dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use dom::bindings::codegen::Bindings::ElementBinding::ElementMethods;
use dom::bindings::conversions::{FromJSValConvertible, jsstring_to_str};
use dom::bindings::global::GlobalRef;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::Root;
use dom::element::Element;
use dom::node::Node;
use ipc_channel::ipc::IpcSender;
use js::jsapi::{ObjectClassName, RootedObject, RootedValue};
use js::jsval::UndefinedValue;
use msg::constellation_msg::PipelineId;
use page::{IterablePage, Page};
use script_task::get_page;
use std::ffi::CStr;
use std::rc::Rc;
use std::str;
use util::str::DOMString;
use uuid::Uuid;

#[allow(unsafe_code)]
pub fn handle_evaluate_js(global: &GlobalRef, eval: String, reply: IpcSender<EvaluateJSReply>) {
    // global.get_cx() returns a valid `JSContext` pointer, so this is safe.
    let result = unsafe {
        let cx = global.get_cx();
        let mut rval = RootedValue::new(cx, UndefinedValue());
        global.evaluate_js_on_global_with_result(&eval, rval.handle_mut());

        if rval.ptr.is_undefined() {
            EvaluateJSReply::VoidValue
        } else if rval.ptr.is_boolean() {
            EvaluateJSReply::BooleanValue(rval.ptr.to_boolean())
        } else if rval.ptr.is_double() || rval.ptr.is_int32() {
            EvaluateJSReply::NumberValue(FromJSValConvertible::from_jsval(cx, rval.handle(), ())
                                             .unwrap())
        } else if rval.ptr.is_string() {
            EvaluateJSReply::StringValue(String::from(jsstring_to_str(cx, rval.ptr.to_string())))
        } else if rval.ptr.is_null() {
            EvaluateJSReply::NullValue
        } else {
            assert!(rval.ptr.is_object());

            let obj = RootedObject::new(cx, rval.ptr.to_object());
            let class_name = CStr::from_ptr(ObjectClassName(cx, obj.handle()));
            let class_name = str::from_utf8(class_name.to_bytes()).unwrap();

            EvaluateJSReply::ActorValue {
                class: class_name.to_owned(),
                uuid: Uuid::new_v4().to_string(),
            }
        }
    };
    reply.send(result).unwrap();
}

pub fn handle_get_root_node(page: &Rc<Page>, pipeline: PipelineId, reply: IpcSender<NodeInfo>) {
    let page = get_page(&*page, pipeline);
    let document = page.document();

    let node = document.upcast::<Node>();
    reply.send(node.summarize()).unwrap();
}

pub fn handle_get_document_element(page: &Rc<Page>,
                                   pipeline: PipelineId,
                                   reply: IpcSender<NodeInfo>) {
    let page = get_page(&*page, pipeline);
    let document = page.document();
    let document_element = document.GetDocumentElement().unwrap();

    let node = document_element.upcast::<Node>();
    reply.send(node.summarize()).unwrap();
}

fn find_node_by_unique_id(page: &Rc<Page>, pipeline: PipelineId, node_id: String) -> Root<Node> {
    let page = get_page(&*page, pipeline);
    let document = page.document();
    let node = document.upcast::<Node>();

    for candidate in node.traverse_preorder() {
        if candidate.get_unique_id() == node_id {
            return candidate;
        }
    }

    panic!("couldn't find node with unique id {}", node_id)
}

pub fn handle_get_children(page: &Rc<Page>,
                           pipeline: PipelineId,
                           node_id: String,
                           reply: IpcSender<Vec<NodeInfo>>) {
    let parent = find_node_by_unique_id(&*page, pipeline, node_id);
    let children = parent.children()
                         .map(|child| child.summarize())
                         .collect();
    reply.send(children).unwrap();
}

pub fn handle_get_layout(page: &Rc<Page>,
                         pipeline: PipelineId,
                         node_id: String,
                         reply: IpcSender<ComputedNodeLayout>) {
    let node = find_node_by_unique_id(&*page, pipeline, node_id);
    let elem = node.downcast::<Element>().expect("should be getting layout of element");
    let rect = elem.GetBoundingClientRect();
    let width = rect.Width() as f32;
    let height = rect.Height() as f32;
    reply.send(ComputedNodeLayout {
             width: width,
             height: height,
         })
         .unwrap();
}

pub fn handle_get_cached_messages(_pipeline_id: PipelineId,
                                  message_types: CachedConsoleMessageTypes,
                                  reply: IpcSender<Vec<CachedConsoleMessage>>) {
    // TODO: check the messageTypes against a global Cache for console messages and page exceptions
    let mut messages = Vec::new();
    if message_types.contains(PAGE_ERROR) {
        // TODO: make script error reporter pass all reported errors
        //      to devtools and cache them for returning here.
        let msg = PageError {
            _type: "PageError".to_owned(),
            errorMessage: "page error test".to_owned(),
            sourceName: String::new(),
            lineText: String::new(),
            lineNumber: 0,
            columnNumber: 0,
            category: String::new(),
            timeStamp: 0,
            error: false,
            warning: false,
            exception: false,
            strict: false,
            private: false,
        };
        messages.push(CachedConsoleMessage::PageError(msg));
    }
    if message_types.contains(CONSOLE_API) {
        // TODO: do for real
        let msg = ConsoleAPI {
            _type: "ConsoleAPI".to_owned(),
            level: "error".to_owned(),
            filename: "http://localhost/~mihai/mozilla/test.html".to_owned(),
            lineNumber: 0,
            functionName: String::new(),
            timeStamp: 0,
            private: false,
            arguments: vec!["console error test".to_owned()],
        };
        messages.push(CachedConsoleMessage::ConsoleAPI(msg));
    }
    reply.send(messages).unwrap();
}

pub fn handle_modify_attribute(page: &Rc<Page>,
                               pipeline: PipelineId,
                               node_id: String,
                               modifications: Vec<Modification>) {
    let node = find_node_by_unique_id(&*page, pipeline, node_id);
    let elem = node.downcast::<Element>().expect("should be getting layout of element");

    for modification in modifications {
        match modification.newValue {
            Some(string) => {
                let _ = elem.SetAttribute(DOMString::from(modification.attributeName),
                                          DOMString::from(string));
            },
            None => elem.RemoveAttribute(DOMString::from(modification.attributeName)),
        }
    }
}

pub fn handle_wants_live_notifications(global: &GlobalRef, send_notifications: bool) {
    global.set_devtools_wants_updates(send_notifications);
}

pub fn handle_set_timeline_markers(page: &Rc<Page>,
                                   marker_types: Vec<TimelineMarkerType>,
                                   reply: IpcSender<TimelineMarker>) {
    let window = page.window();
    window.set_devtools_timeline_markers(marker_types, reply);
}

pub fn handle_drop_timeline_markers(page: &Rc<Page>, marker_types: Vec<TimelineMarkerType>) {
    let window = page.window();
    window.drop_devtools_timeline_markers(marker_types);
}

pub fn handle_request_animation_frame(page: &Rc<Page>, id: PipelineId, actor_name: String) {
    let page = page.find(id).expect("There is no such page");
    let doc = page.document();
    let devtools_sender = page.window().devtools_chan().unwrap();
    doc.request_animation_frame(box move |time| {
        let msg = ScriptToDevtoolsControlMsg::FramerateTick(actor_name, time);
        devtools_sender.send(msg).unwrap();
    });
}
