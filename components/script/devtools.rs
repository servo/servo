/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use devtools_traits::TimelineMarkerType;
use devtools_traits::{AutoMargins, CONSOLE_API, CachedConsoleMessage, CachedConsoleMessageTypes};
use devtools_traits::{ComputedNodeLayout, ConsoleAPI, PageError, ScriptToDevtoolsControlMsg};
use devtools_traits::{EvaluateJSReply, Modification, NodeInfo, PAGE_ERROR, TimelineMarker};
use dom::bindings::codegen::Bindings::CSSStyleDeclarationBinding::CSSStyleDeclarationMethods;
use dom::bindings::codegen::Bindings::DOMRectBinding::DOMRectMethods;
use dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use dom::bindings::codegen::Bindings::ElementBinding::ElementMethods;
use dom::bindings::codegen::Bindings::LocationBinding::LocationMethods;
use dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use dom::bindings::conversions::{FromJSValConvertible, jsstring_to_str};
use dom::bindings::global::GlobalRef;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::Root;
use dom::bindings::str::DOMString;
use dom::browsingcontext::BrowsingContext;
use dom::element::Element;
use dom::node::Node;
use dom::window::Window;
use ipc_channel::ipc::IpcSender;
use js::jsapi::{JSAutoCompartment, ObjectClassName};
use js::jsval::UndefinedValue;
use msg::constellation_msg::PipelineId;
use script_thread::get_browsing_context;
use std::ffi::CStr;
use std::str;
use style::properties::longhands::{margin_top, margin_right, margin_bottom, margin_left};
use uuid::Uuid;

#[allow(unsafe_code)]
pub fn handle_evaluate_js(global: &GlobalRef, eval: String, reply: IpcSender<EvaluateJSReply>) {
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
            EvaluateJSReply::NumberValue(FromJSValConvertible::from_jsval(cx, rval.handle(), ())
                                             .unwrap())
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

pub fn handle_get_root_node(context: &BrowsingContext, pipeline: PipelineId, reply: IpcSender<NodeInfo>) {
    let context = get_browsing_context(context, pipeline);
    let document = context.active_document();

    let node = document.upcast::<Node>();
    reply.send(node.summarize()).unwrap();
}

pub fn handle_get_document_element(context: &BrowsingContext,
                                   pipeline: PipelineId,
                                   reply: IpcSender<NodeInfo>) {
    let context = get_browsing_context(context, pipeline);
    let document = context.active_document();
    let document_element = document.GetDocumentElement().unwrap();

    let node = document_element.upcast::<Node>();
    reply.send(node.summarize()).unwrap();
}

fn find_node_by_unique_id(context: &BrowsingContext,
                          pipeline: PipelineId,
                          node_id: String)
                          -> Root<Node> {
    let context = get_browsing_context(context, pipeline);
    let document = context.active_document();
    let node = document.upcast::<Node>();

    for candidate in node.traverse_preorder() {
        if candidate.unique_id() == node_id {
            return candidate;
        }
    }

    panic!("couldn't find node with unique id {}", node_id)
}

pub fn handle_get_children(context: &BrowsingContext,
                           pipeline: PipelineId,
                           node_id: String,
                           reply: IpcSender<Vec<NodeInfo>>) {
    let parent = find_node_by_unique_id(context, pipeline, node_id);
    let children = parent.children()
                         .map(|child| child.summarize())
                         .collect();
    reply.send(children).unwrap();
}

pub fn handle_get_layout(context: &BrowsingContext,
                         pipeline: PipelineId,
                         node_id: String,
                         reply: IpcSender<ComputedNodeLayout>) {
    let node = find_node_by_unique_id(context, pipeline, node_id);

    let elem = node.downcast::<Element>().expect("should be getting layout of element");
    let rect = elem.GetBoundingClientRect();
    let width = rect.Width() as f32;
    let height = rect.Height() as f32;

    let window = context.active_window();
    let elem = node.downcast::<Element>().expect("should be getting layout of element");
    let computed_style = window.r().GetComputedStyle(elem, None);

    reply.send(ComputedNodeLayout {
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
    }).unwrap();
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

pub fn handle_modify_attribute(context: &BrowsingContext,
                               pipeline: PipelineId,
                               node_id: String,
                               modifications: Vec<Modification>) {
    let node = find_node_by_unique_id(context, pipeline, node_id);
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

pub fn handle_set_timeline_markers(context: &BrowsingContext,
                                   marker_types: Vec<TimelineMarkerType>,
                                   reply: IpcSender<TimelineMarker>) {
    let window = context.active_window();
    window.set_devtools_timeline_markers(marker_types, reply);
}

pub fn handle_drop_timeline_markers(context: &BrowsingContext,
                                    marker_types: Vec<TimelineMarkerType>) {
    let window = context.active_window();
    window.drop_devtools_timeline_markers(marker_types);
}

pub fn handle_request_animation_frame(context: &BrowsingContext,
                                      id: PipelineId,
                                      actor_name: String) {
    let context = context.find(id).expect("There is no such context");
    let doc = context.active_document();
    let devtools_sender = context.active_window().devtools_chan().unwrap();
    doc.request_animation_frame(box move |time| {
        let msg = ScriptToDevtoolsControlMsg::FramerateTick(actor_name, time);
        devtools_sender.send(msg).unwrap();
    });
}

pub fn handle_reload(context: &BrowsingContext,
                     id: PipelineId) {
    let context = context.find(id).expect("There is no such context");
    let win = context.active_window();
    let location = win.Location();
    location.Reload();
}
