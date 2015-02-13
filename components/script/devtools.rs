/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use devtools_traits::{EvaluateJSReply, NodeInfo, Modification};
use dom::bindings::conversions::FromJSValConvertible;
use dom::bindings::conversions::StringificationBehavior;
use dom::bindings::js::{JSRef, Temporary, OptionalRootable};
use dom::bindings::codegen::InheritTypes::{NodeCast, ElementCast};
use dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use dom::bindings::codegen::Bindings::DOMRectBinding::{DOMRectMethods};
use dom::bindings::codegen::Bindings::ElementBinding::{ElementMethods};
use dom::node::{Node, NodeHelpers};
use dom::window::{ScriptHelpers};
use dom::element::Element;
use dom::document::DocumentHelpers;
use page::Page;
use msg::constellation_msg::PipelineId;
use script_task::get_page;

use std::sync::mpsc::Sender;
use std::rc::Rc;


pub fn handle_evaluate_js(page: &Rc<Page>, pipeline: PipelineId, eval: String, reply: Sender<EvaluateJSReply>){
    let page = get_page(&*page, pipeline);
    let frame = page.frame();
    let window = frame.as_ref().unwrap().window.root();
    let cx = window.r().get_cx();
    let rval = window.r().evaluate_js_on_global_with_result(eval.as_slice());

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
    let frame = page.frame();
    let document = frame.as_ref().unwrap().document.root();

    let node: JSRef<Node> = NodeCast::from_ref(document.r());
    reply.send(node.summarize()).unwrap();
}

pub fn handle_get_document_element(page: &Rc<Page>, pipeline: PipelineId, reply: Sender<NodeInfo>) {
    let page = get_page(&*page, pipeline);
    let frame = page.frame();
    let document = frame.as_ref().unwrap().document.root();
    let document_element = document.r().GetDocumentElement().root().unwrap();

    let node: JSRef<Node> = NodeCast::from_ref(document_element.r());
    reply.send(node.summarize()).unwrap();
}

fn find_node_by_unique_id(page: &Rc<Page>, pipeline: PipelineId, node_id: String) -> Temporary<Node> {
    let page = get_page(&*page, pipeline);
    let frame = page.frame();
    let document = frame.as_ref().unwrap().document.root();
    let node: JSRef<Node> = NodeCast::from_ref(document.r());

    for candidate in node.traverse_preorder() {
        if candidate.get_unique_id().as_slice() == node_id.as_slice() {
            return Temporary::from_rooted(candidate);
        }
    }

    panic!("couldn't find node with unique id {}", node_id)
}

pub fn handle_get_children(page: &Rc<Page>, pipeline: PipelineId, node_id: String, reply: Sender<Vec<NodeInfo>>) {
    let parent = find_node_by_unique_id(&*page, pipeline, node_id).root();
    let children = parent.r().children().map(|child| child.summarize()).collect();
    reply.send(children).unwrap();
}

pub fn handle_get_layout(page: &Rc<Page>, pipeline: PipelineId, node_id: String, reply: Sender<(f32, f32)>) {
    let node = find_node_by_unique_id(&*page, pipeline, node_id).root();
    let elem: JSRef<Element> = ElementCast::to_ref(node.r()).expect("should be getting layout of element");
    let rect = elem.GetBoundingClientRect().root();
    reply.send((rect.r().Width(), rect.r().Height())).unwrap();
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
    page.devtools_wants_updates.set(send_notifications);
}
