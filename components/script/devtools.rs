/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use devtools_traits::{AutoMargins, CONSOLE_API, CachedConsoleMessage, CachedConsoleMessageTypes};
use devtools_traits::{ComputedNodeLayout, ConsoleAPI, PageError};
use devtools_traits::{EvaluateJSReply, Modification, NodeInfo, PAGE_ERROR, TimelineMarker};
use devtools_traits::TimelineMarkerType;
use dom::bindings::codegen::Bindings::CSSStyleDeclarationBinding::CSSStyleDeclarationMethods;
use dom::bindings::codegen::Bindings::DOMRectBinding::DOMRectMethods;
use dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use dom::bindings::codegen::Bindings::ElementBinding::ElementMethods;
use dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use dom::bindings::conversions::{ConversionResult, FromJSValConvertible, jsstring_to_str};
use dom::bindings::inheritance::Castable;
use dom::bindings::js::Root;
use dom::bindings::reflector::DomObject;
use dom::bindings::str::DOMString;
use dom::document::AnimationFrameCallback;
use dom::element::Element;
use dom::globalscope::GlobalScope;
use dom::node::{Node, window_from_node};
use dom::window::Window;
use ipc_channel::ipc::IpcSender;
use js::jsapi::{JSAutoCompartment, ObjectClassName};
use js::jsval::UndefinedValue;
use msg::constellation_msg::PipelineId;
use script_thread::Documents;
use std::ffi::CStr;
use std::str;
use style::properties::longhands::{margin_bottom, margin_left, margin_right, margin_top};
use uuid::Uuid;


#[allow(unsafe_code)]
pub fn handle_evaluate_js(global: &GlobalScope, eval: String, reply: IpcSender<EvaluateJSReply>) {
    // global.get_cx() returns a valid `JSContext` pointer, so this is safe.
    let result = unsafe {
        let cx = global.get_cx();
        let globalhandle = global.reflector().get_jsobject();
        let _ac = JSAutoCompartment::new(cx, globalhandle.get());
        rooted!(in(cx) let mut rval = UndefinedValue());
        global.evaluate_js_on_global_with_result(&eval, rval.handle_mut());

        if rval.is_undefined() {
            EvaluateJSReply::VoidValue
        } else if rval.is_boolean() {
            EvaluateJSReply::BooleanValue(rval.to_boolean())
        } else if rval.is_double() || rval.is_int32() {
            EvaluateJSReply::NumberValue(
                match FromJSValConvertible::from_jsval(cx, rval.handle(), ()) {
                    Ok(ConversionResult::Success(v)) => v,
                    _ => unreachable!(),
                })
        } else if rval.is_string() {
            EvaluateJSReply::StringValue(String::from(jsstring_to_str(cx, rval.to_string())))
        } else if rval.is_null() {
            EvaluateJSReply::NullValue
        } else {
            assert!(rval.is_object());

            rooted!(in(cx) let obj = rval.to_object());
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

pub fn handle_get_root_node(documents: &Documents, pipeline: PipelineId, reply: IpcSender<Option<NodeInfo>>) {
    let info = documents.find_document(pipeline)
        .map(|document| document.upcast::<Node>().summarize());
    reply.send(info).unwrap();
}

pub fn handle_get_document_element(documents: &Documents,
                                   pipeline: PipelineId,
                                   reply: IpcSender<Option<NodeInfo>>) {
    let info = documents.find_document(pipeline)
        .and_then(|document| document.GetDocumentElement())
        .map(|element| element.upcast::<Node>().summarize());
    reply.send(info).unwrap();
}

fn find_node_by_unique_id(documents: &Documents,
                          pipeline: PipelineId,
                          node_id: &str)
                          -> Option<Root<Node>> {
    documents.find_document(pipeline).and_then(|document|
        document.upcast::<Node>().traverse_preorder().find(|candidate| candidate.unique_id() == node_id)
    )
}

pub fn handle_get_children(documents: &Documents,
                           pipeline: PipelineId,
                           node_id: String,
                           reply: IpcSender<Option<Vec<NodeInfo>>>) {
    match find_node_by_unique_id(documents, pipeline, &*node_id) {
        None => return reply.send(None).unwrap(),
        Some(parent) => {
            let children = parent.children()
                                 .map(|child| child.summarize())
                                 .collect();

            reply.send(Some(children)).unwrap();
        }
    };
}

pub fn handle_get_layout(documents: &Documents,
                         pipeline: PipelineId,
                         node_id: String,
                         reply: IpcSender<Option<ComputedNodeLayout>>) {
    let node = match find_node_by_unique_id(documents, pipeline, &*node_id) {
        None => return reply.send(None).unwrap(),
        Some(found_node) => found_node
    };

    let elem = node.downcast::<Element>().expect("should be getting layout of element");
    let rect = elem.GetBoundingClientRect();
    let width = rect.Width() as f32;
    let height = rect.Height() as f32;

    let window = window_from_node(&*node);
    let elem = node.downcast::<Element>().expect("should be getting layout of element");
    let computed_style = window.GetComputedStyle(elem, None);

    reply.send(Some(ComputedNodeLayout {
        display: String::from(computed_style.Display()),
        position: String::from(computed_style.Position()),
        zIndex: String::from(computed_style.ZIndex()),
        boxSizing: String::from(computed_style.BoxSizing()),
        autoMargins: determine_auto_margins(&window, &*node),
        marginTop: String::from(computed_style.MarginTop()),
        marginRight: String::from(computed_style.MarginRight()),
        marginBottom: String::from(computed_style.MarginBottom()),
        marginLeft: String::from(computed_style.MarginLeft()),
        borderTopWidth: String::from(computed_style.BorderTopWidth()),
        borderRightWidth: String::from(computed_style.BorderRightWidth()),
        borderBottomWidth: String::from(computed_style.BorderBottomWidth()),
        borderLeftWidth: String::from(computed_style.BorderLeftWidth()),
        paddingTop: String::from(computed_style.PaddingTop()),
        paddingRight: String::from(computed_style.PaddingRight()),
        paddingBottom: String::from(computed_style.PaddingBottom()),
        paddingLeft: String::from(computed_style.PaddingLeft()),
        width: width,
        height: height,
    })).unwrap();
}

fn determine_auto_margins(window: &Window, node: &Node) -> AutoMargins {
    let margin = window.margin_style_query(node.to_trusted_node_address());
    AutoMargins {
        top: margin.top == margin_top::computed_value::T::Auto,
        right: margin.right == margin_right::computed_value::T::Auto,
        bottom: margin.bottom == margin_bottom::computed_value::T::Auto,
        left: margin.left == margin_left::computed_value::T::Auto,
    }
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
            type_: "PageError".to_owned(),
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
            type_: "ConsoleAPI".to_owned(),
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

pub fn handle_modify_attribute(documents: &Documents,
                               pipeline: PipelineId,
                               node_id: String,
                               modifications: Vec<Modification>) {
    let node = match find_node_by_unique_id(documents, pipeline, &*node_id) {
        None => return warn!("node id {} for pipeline id {} is not found", &node_id, &pipeline),
        Some(found_node) => found_node
    };

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

pub fn handle_wants_live_notifications(global: &GlobalScope, send_notifications: bool) {
    global.set_devtools_wants_updates(send_notifications);
}

pub fn handle_set_timeline_markers(documents: &Documents,
                                   pipeline: PipelineId,
                                   marker_types: Vec<TimelineMarkerType>,
                                   reply: IpcSender<Option<TimelineMarker>>) {
    match documents.find_window(pipeline) {
        None => reply.send(None).unwrap(),
        Some(window) => window.set_devtools_timeline_markers(marker_types, reply),
    }
}

pub fn handle_drop_timeline_markers(documents: &Documents,
                                    pipeline: PipelineId,
                                    marker_types: Vec<TimelineMarkerType>) {
    if let Some(window) = documents.find_window(pipeline) {
        window.drop_devtools_timeline_markers(marker_types);
    }
}

pub fn handle_request_animation_frame(documents: &Documents,
                                      id: PipelineId,
                                      actor_name: String) {
    if let Some(doc) = documents.find_document(id) {
        doc.request_animation_frame(AnimationFrameCallback::DevtoolsFramerateTick { actor_name });
    }
}

pub fn handle_reload(documents: &Documents,
                     id: PipelineId) {
    if let Some(win) = documents.find_window(id) {
        win.Location().reload_without_origin_check();
    }
}
