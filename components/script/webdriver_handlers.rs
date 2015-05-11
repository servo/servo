/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use webdriver_traits::{EvaluateJSReply};
use dom::bindings::conversions::FromJSValConvertible;
use dom::bindings::conversions::StringificationBehavior;
use dom::bindings::codegen::InheritTypes::{NodeCast, ElementCast};
use dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use dom::bindings::codegen::Bindings::ElementBinding::ElementMethods;
use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::codegen::Bindings::NodeListBinding::NodeListMethods;
use dom::bindings::js::{OptionalRootable, Rootable, Temporary};
use dom::node::{Node, NodeHelpers};
use dom::window::ScriptHelpers;
use dom::document::DocumentHelpers;
use page::Page;
use msg::constellation_msg::PipelineId;
use script_task::get_page;

use std::rc::Rc;
use std::sync::mpsc::Sender;

fn find_node_by_unique_id(page: &Rc<Page>, pipeline: PipelineId, node_id: String) -> Option<Temporary<Node>> {
    let page = get_page(&*page, pipeline);
    let document = page.document().root();
    let node = NodeCast::from_ref(document.r());

    for candidate in node.traverse_preorder() {
        if candidate.root().r().get_unique_id() == node_id {
            return Some(candidate);
        }
    }

    None
}

pub fn handle_evaluate_js(page: &Rc<Page>, pipeline: PipelineId, eval: String, reply: Sender<Result<EvaluateJSReply, ()>>) {
    let page = get_page(&*page, pipeline);
    let window = page.window().root();
    let cx = window.r().get_cx();
    let rval = window.r().evaluate_js_on_global_with_result(&eval);

    reply.send(if rval.is_undefined() {
        Ok(EvaluateJSReply::VoidValue)
    } else if rval.is_boolean() {
        Ok(EvaluateJSReply::BooleanValue(rval.to_boolean()))
    } else if rval.is_double() {
        Ok(EvaluateJSReply::NumberValue(FromJSValConvertible::from_jsval(cx, rval, ()).unwrap()))
    } else if rval.is_string() {
        //FIXME: use jsstring_to_str when jsval grows to_jsstring
        Ok(EvaluateJSReply::StringValue(FromJSValConvertible::from_jsval(cx, rval, StringificationBehavior::Default).unwrap()))
    } else if rval.is_null() {
        Ok(EvaluateJSReply::NullValue)
    } else {
        Err(())
    }).unwrap();
}

pub fn handle_find_element_css(page: &Rc<Page>, _pipeline: PipelineId, selector: String, reply: Sender<Result<Option<String>, ()>>) {
    reply.send(match page.document().root().r().QuerySelector(selector.clone()) {
        Ok(node) => {
            let result = node.map(|x| NodeCast::from_ref(x.root().r()).get_unique_id());
            Ok(result)
        }
        Err(_) => Err(())
    }).unwrap();
}

pub fn handle_find_elements_css(page: &Rc<Page>, _pipeline: PipelineId, selector: String, reply: Sender<Result<Vec<String>, ()>>) {
    reply.send(match page.document().root().r().QuerySelectorAll(selector.clone()) {
        Ok(ref node_list) => {
            let nodes = node_list.root();
            let mut result = Vec::with_capacity(nodes.r().Length() as usize);
            for i in 0..nodes.r().Length() {
                if let Some(ref node) = nodes.r().Item(i) {
                    result.push(node.root().r().get_unique_id());
                }
            }
            Ok(result)
        },
        Err(_) => {
            Err(())
        }
    }).unwrap();
}

pub fn handle_get_active_element(page: &Rc<Page>, _pipeline: PipelineId, reply: Sender<Option<String>>) {
    reply.send(page.document().root().r().GetActiveElement().map(
        |elem| NodeCast::from_ref(elem.root().r()).get_unique_id())).unwrap();
}

pub fn handle_get_title(page: &Rc<Page>, _pipeline: PipelineId, reply: Sender<String>) {
    reply.send(page.document().root().r().Title()).unwrap();
}

pub fn handle_get_text(page: &Rc<Page>, pipeline: PipelineId, node_id: String, reply: Sender<Result<String, ()>>) {
    reply.send(match find_node_by_unique_id(&*page, pipeline, node_id) {
        Some(ref node) => {
            Ok(node.root().r().GetTextContent().unwrap_or("".to_owned()))
        },
        None => Err(())
    }).unwrap();
}

pub fn handle_get_name(page: &Rc<Page>, pipeline: PipelineId, node_id: String, reply: Sender<Result<String, ()>>) {
    reply.send(match find_node_by_unique_id(&*page, pipeline, node_id) {
        Some(tmp_node) => {
            let node = tmp_node.root();
            let element = ElementCast::to_ref(node.r()).unwrap();
            Ok(element.TagName())
        },
        None => Err(())
    }).unwrap();
}
