/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;
use std::str;

use base::id::PipelineId;
use devtools_traits::{
    AutoMargins, ComputedNodeLayout, EvaluateJSReply, Modification, NodeInfo, TimelineMarker,
    TimelineMarkerType,
};
use ipc_channel::ipc::IpcSender;
use js::jsval::UndefinedValue;
use js::rust::ToString;
use uuid::Uuid;

use crate::dom::bindings::codegen::Bindings::CSSStyleDeclarationBinding::CSSStyleDeclarationMethods;
use crate::dom::bindings::codegen::Bindings::DOMRectBinding::DOMRectMethods;
use crate::dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use crate::dom::bindings::codegen::Bindings::ElementBinding::ElementMethods;
use crate::dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use crate::dom::bindings::conversions::{jsstring_to_str, ConversionResult, FromJSValConvertible};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::document::AnimationFrameCallback;
use crate::dom::element::Element;
use crate::dom::globalscope::GlobalScope;
use crate::dom::htmlscriptelement::SourceCode;
use crate::dom::node::{window_from_node, Node, ShadowIncluding};
use crate::realms::enter_realm;
use crate::script_module::ScriptFetchOptions;
use crate::script_thread::Documents;

#[allow(unsafe_code)]
pub fn handle_evaluate_js(global: &GlobalScope, eval: String, reply: IpcSender<EvaluateJSReply>) {
    // global.get_cx() returns a valid `JSContext` pointer, so this is safe.
    let result = unsafe {
        let cx = GlobalScope::get_cx();
        let _ac = enter_realm(global);
        rooted!(in(*cx) let mut rval = UndefinedValue());
        let source_code = SourceCode::Text(Rc::new(DOMString::from_string(eval)));
        global.evaluate_script_on_global_with_result(
            &source_code,
            "<eval>",
            rval.handle_mut(),
            1,
            ScriptFetchOptions::default_classic_script(global),
            global.api_base_url(),
        );

        if rval.is_undefined() {
            EvaluateJSReply::VoidValue
        } else if rval.is_boolean() {
            EvaluateJSReply::BooleanValue(rval.to_boolean())
        } else if rval.is_double() || rval.is_int32() {
            EvaluateJSReply::NumberValue(
                match FromJSValConvertible::from_jsval(*cx, rval.handle(), ()) {
                    Ok(ConversionResult::Success(v)) => v,
                    _ => unreachable!(),
                },
            )
        } else if rval.is_string() {
            EvaluateJSReply::StringValue(String::from(jsstring_to_str(*cx, rval.to_string())))
        } else if rval.is_null() {
            EvaluateJSReply::NullValue
        } else {
            assert!(rval.is_object());

            let jsstr = ToString(*cx, rval.handle());
            let class_name = jsstring_to_str(*cx, jsstr);

            EvaluateJSReply::ActorValue {
                class: class_name.to_string(),
                uuid: Uuid::new_v4().to_string(),
            }
        }
    };
    reply.send(result).unwrap();
}

pub fn handle_get_root_node(
    documents: &Documents,
    pipeline: PipelineId,
    reply: IpcSender<Option<NodeInfo>>,
) {
    let info = documents
        .find_document(pipeline)
        .map(|document| document.upcast::<Node>().summarize());
    reply.send(info).unwrap();
}

pub fn handle_get_document_element(
    documents: &Documents,
    pipeline: PipelineId,
    reply: IpcSender<Option<NodeInfo>>,
) {
    let info = documents
        .find_document(pipeline)
        .and_then(|document| document.GetDocumentElement())
        .map(|element| element.upcast::<Node>().summarize());
    reply.send(info).unwrap();
}

fn find_node_by_unique_id(
    documents: &Documents,
    pipeline: PipelineId,
    node_id: &str,
) -> Option<DomRoot<Node>> {
    documents.find_document(pipeline).and_then(|document| {
        document
            .upcast::<Node>()
            .traverse_preorder(ShadowIncluding::Yes)
            .find(|candidate| candidate.unique_id() == node_id)
    })
}

pub fn handle_get_children(
    documents: &Documents,
    pipeline: PipelineId,
    node_id: String,
    reply: IpcSender<Option<Vec<NodeInfo>>>,
) {
    match find_node_by_unique_id(documents, pipeline, &node_id) {
        None => reply.send(None).unwrap(),
        Some(parent) => {
            let children = parent.children().map(|child| child.summarize()).collect();

            reply.send(Some(children)).unwrap();
        },
    };
}

pub fn handle_get_layout(
    documents: &Documents,
    pipeline: PipelineId,
    node_id: String,
    reply: IpcSender<Option<ComputedNodeLayout>>,
) {
    let node = match find_node_by_unique_id(documents, pipeline, &node_id) {
        None => return reply.send(None).unwrap(),
        Some(found_node) => found_node,
    };

    let elem = node
        .downcast::<Element>()
        .expect("should be getting layout of element");
    let rect = elem.GetBoundingClientRect();
    let width = rect.Width() as f32;
    let height = rect.Height() as f32;

    let window = window_from_node(&*node);
    let elem = node
        .downcast::<Element>()
        .expect("should be getting layout of element");
    let computed_style = window.GetComputedStyle(elem, None);

    reply
        .send(Some(ComputedNodeLayout {
            display: String::from(computed_style.Display()),
            position: String::from(computed_style.Position()),
            z_index: String::from(computed_style.ZIndex()),
            box_sizing: String::from(computed_style.BoxSizing()),
            auto_margins: determine_auto_margins(&node),
            margin_top: String::from(computed_style.MarginTop()),
            margin_right: String::from(computed_style.MarginRight()),
            margin_bottom: String::from(computed_style.MarginBottom()),
            margin_left: String::from(computed_style.MarginLeft()),
            border_top_width: String::from(computed_style.BorderTopWidth()),
            border_right_width: String::from(computed_style.BorderRightWidth()),
            border_bottom_width: String::from(computed_style.BorderBottomWidth()),
            border_left_width: String::from(computed_style.BorderLeftWidth()),
            padding_top: String::from(computed_style.PaddingTop()),
            padding_right: String::from(computed_style.PaddingRight()),
            padding_bottom: String::from(computed_style.PaddingBottom()),
            padding_left: String::from(computed_style.PaddingLeft()),
            width,
            height,
        }))
        .unwrap();
}

fn determine_auto_margins(node: &Node) -> AutoMargins {
    let style = node.style().unwrap();
    let margin = style.get_margin();
    AutoMargins {
        top: margin.margin_top.is_auto(),
        right: margin.margin_right.is_auto(),
        bottom: margin.margin_bottom.is_auto(),
        left: margin.margin_left.is_auto(),
    }
}

pub fn handle_modify_attribute(
    documents: &Documents,
    pipeline: PipelineId,
    node_id: String,
    modifications: Vec<Modification>,
) {
    let node = match find_node_by_unique_id(documents, pipeline, &node_id) {
        None => {
            return warn!(
                "node id {} for pipeline id {} is not found",
                &node_id, &pipeline
            );
        },
        Some(found_node) => found_node,
    };

    let elem = node
        .downcast::<Element>()
        .expect("should be getting layout of element");

    for modification in modifications {
        match modification.new_value {
            Some(string) => {
                let _ = elem.SetAttribute(
                    DOMString::from(modification.attribute_name),
                    DOMString::from(string),
                );
            },
            None => elem.RemoveAttribute(DOMString::from(modification.attribute_name)),
        }
    }
}

pub fn handle_wants_live_notifications(global: &GlobalScope, send_notifications: bool) {
    global.set_devtools_wants_updates(send_notifications);
}

pub fn handle_set_timeline_markers(
    documents: &Documents,
    pipeline: PipelineId,
    marker_types: Vec<TimelineMarkerType>,
    reply: IpcSender<Option<TimelineMarker>>,
) {
    match documents.find_window(pipeline) {
        None => reply.send(None).unwrap(),
        Some(window) => window.set_devtools_timeline_markers(marker_types, reply),
    }
}

pub fn handle_drop_timeline_markers(
    documents: &Documents,
    pipeline: PipelineId,
    marker_types: Vec<TimelineMarkerType>,
) {
    if let Some(window) = documents.find_window(pipeline) {
        window.drop_devtools_timeline_markers(marker_types);
    }
}

pub fn handle_request_animation_frame(documents: &Documents, id: PipelineId, actor_name: String) {
    if let Some(doc) = documents.find_document(id) {
        doc.request_animation_frame(AnimationFrameCallback::DevtoolsFramerateTick { actor_name });
    }
}

pub fn handle_reload(documents: &Documents, id: PipelineId) {
    if let Some(win) = documents.find_window(id) {
        win.Location().reload_without_origin_check();
    }
}
