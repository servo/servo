/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use devtools_traits::{EvaluateJSReply, NodeInfo, Modification, TimelineMarker, TimelineMarkerType};
use dom::bindings::conversions::FromJSValConvertible;
use dom::bindings::conversions::StringificationBehavior;
use dom::bindings::js::{JSRef, OptionalRootable, Rootable, Temporary};
use dom::bindings::codegen::InheritTypes::{NodeCast, ElementCast};
use dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use dom::bindings::codegen::Bindings::DOMRectBinding::{DOMRectMethods};
use dom::bindings::codegen::Bindings::ElementBinding::{ElementMethods};
use dom::node::{Node, NodeHelpers};
use dom::window::{WindowHelpers, ScriptHelpers};
use dom::element::Element;
use dom::document::DocumentHelpers;
use page::{IterablePage, Page};
use msg::constellation_msg::PipelineId;
use script_task::{get_page, ScriptTask};

use std::sync::mpsc::Sender;
use std::rc::Rc;


pub fn handle_evaluate_js(page: &Rc<Page>, pipeline: PipelineId, eval: String, reply: Sender<EvaluateJSReply>){
    let page = get_page(&*page, pipeline);
    let window = page.window().root();
    let cx = window.r().get_cx();
    let rval = window.r().evaluate_js_on_global_with_result(&eval);

    reply.send(if rval.is_undefined() {
        EvaluateJSReply::VoidValue
    } else if rval.is_boolean() {
        EvaluateJSReply::BooleanValue(rval.to_boolean())
    } else if rval.is_double() {
        EvaluateJSReply::NumberValue(FromJSValConvertible::from_jsval(cx, rval, ()).unwrap())
    } else if rval.is_string() {
        //FIXME: use jsstring_to_str when jsval grows to_jsstring
        EvaluateJSReply::StringValue(FromJSValConvertible::from_jsval(cx, rval, StringificationBehavior::Default).unwrap())
    } else if rval.is_null() {
        EvaluateJSReply::NullValue
    } else {
        //FIXME: jsvals don't have an is_int32/is_number yet
        assert!(rval.is_object());
        panic!("object values unimplemented")
    }).unwrap();
}

pub fn handle_get_root_node(page: &Rc<Page>, pipeline: PipelineId, reply: Sender<NodeInfo>) {
    let page = get_page(&*page, pipeline);
    let document = page.document().root();

    let node: JSRef<Node> = NodeCast::from_ref(document.r());
    reply.send(node.summarize()).unwrap();
}

pub fn handle_get_document_element(page: &Rc<Page>, pipeline: PipelineId, reply: Sender<NodeInfo>) {
    let page = get_page(&*page, pipeline);
    let document = page.document().root();
    let document_element = document.r().GetDocumentElement().root().unwrap();

    let node: JSRef<Node> = NodeCast::from_ref(document_element.r());
    reply.send(node.summarize()).unwrap();
}

fn find_node_by_unique_id(page: &Rc<Page>, pipeline: PipelineId, node_id: String) -> Temporary<Node> {
    let page = get_page(&*page, pipeline);
    let document = page.document().root();
    let node: JSRef<Node> = NodeCast::from_ref(document.r());

    for candidate in node.traverse_preorder() {
        if candidate.root().r().get_unique_id() == node_id {
            return candidate;
        }
    }

    panic!("couldn't find node with unique id {}", node_id)
}

pub fn handle_get_children(page: &Rc<Page>, pipeline: PipelineId, node_id: String, reply: Sender<Vec<NodeInfo>>) {
    let parent = find_node_by_unique_id(&*page, pipeline, node_id).root();
    let children = parent.r().children().map(|child| {
        let child = child.root();
        child.r().summarize()
    }).collect();
    reply.send(children).unwrap();
}

pub fn handle_get_layout(page: &Rc<Page>, pipeline: PipelineId, node_id: String, reply: Sender<(f32, f32)>) {
    let node = find_node_by_unique_id(&*page, pipeline, node_id).root();
    let elem: JSRef<Element> = ElementCast::to_ref(node.r()).expect("should be getting layout of element");
    let rect = elem.GetBoundingClientRect().root();
    let width = *rect.r().Width();
    let height = *rect.r().Height();
    reply.send((width, height)).unwrap();
}

pub fn handle_modify_attribute(page: &Rc<Page>, pipeline: PipelineId, node_id: String, modifications: Vec<Modification>) {
    let node = find_node_by_unique_id(&*page, pipeline, node_id).root();
    let elem: JSRef<Element> = ElementCast::to_ref(node.r()).expect("should be getting layout of element");

    for modification in modifications.iter(){
        match modification.newValue {
            Some(ref string) => {
                let _ = elem.SetAttribute(modification.attributeName.clone(), string.clone());
            },
            None => elem.RemoveAttribute(modification.attributeName.clone()),
        }
    }
}

pub fn handle_wants_live_notifications(page: &Rc<Page>, pipeline_id: PipelineId, send_notifications: bool) {
    let page = get_page(&*page, pipeline_id);
    let window = page.window().root();
    window.r().set_devtools_wants_updates(send_notifications);
}

pub fn handle_set_timeline_markers(page: &Rc<Page>,
                                   script_task: &ScriptTask,
                                   marker_types: Vec<TimelineMarkerType>,
                                   reply: Sender<TimelineMarker>) {
    for marker_type in &marker_types {
        match *marker_type {
            TimelineMarkerType::Reflow => {
                let window = page.window().root();
                window.r().set_devtools_timeline_marker(TimelineMarkerType::Reflow, reply.clone());
            }
            TimelineMarkerType::DOMEvent => {
                script_task.set_devtools_timeline_marker(TimelineMarkerType::DOMEvent, reply.clone());
            }
        }
    }
}

pub fn handle_drop_timeline_markers(page: &Rc<Page>,
                                    script_task: &ScriptTask,
                                    marker_types: Vec<TimelineMarkerType>) {
    let window = page.window().root();
    for marker_type in &marker_types {
        match *marker_type {
            TimelineMarkerType::Reflow => {
                window.r().drop_devtools_timeline_markers();
            }
            TimelineMarkerType::DOMEvent => {
                script_task.drop_devtools_timeline_markers();
            }
        }
    }
}

pub fn handle_request_animation_frame(page: &Rc<Page>, id: PipelineId, callback: Box<Fn(f64, )>) {
    let page = page.find(id).expect("There is no such page");
    let doc = page.document().root();
    doc.r().request_animation_frame(callback);
}
