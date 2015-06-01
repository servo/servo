/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::conversions::FromJSValConvertible;
use dom::bindings::conversions::StringificationBehavior;
use dom::bindings::codegen::InheritTypes::{NodeCast, ElementCast, HTMLIFrameElementCast};
use dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use dom::bindings::codegen::Bindings::ElementBinding::ElementMethods;
use dom::bindings::codegen::Bindings::HTMLIFrameElementBinding::HTMLIFrameElementMethods;
use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::codegen::Bindings::NodeListBinding::NodeListMethods;
use dom::bindings::js::{OptionalRootable, Rootable, Temporary};
use dom::node::{Node, NodeHelpers};
use dom::window::{ScriptHelpers, WindowHelpers};
use dom::document::DocumentHelpers;
use js::jsapi::JSContext;
use js::jsval::JSVal;
use page::Page;
use msg::constellation_msg::{PipelineId, SubpageId};
use msg::webdriver_msg::{WebDriverJSValue, WebDriverJSError, WebDriverJSResult, WebDriverFrameId};
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

pub fn jsval_to_webdriver(cx: *mut JSContext, val: JSVal) -> WebDriverJSResult {
    if val.is_undefined() {
        Ok(WebDriverJSValue::Undefined)
    } else if val.is_boolean() {
        Ok(WebDriverJSValue::Boolean(val.to_boolean()))
    } else if val.is_double() {
        Ok(WebDriverJSValue::Number(FromJSValConvertible::from_jsval(cx, val, ()).unwrap()))
    } else if val.is_string() {
        //FIXME: use jsstring_to_str when jsval grows to_jsstring
        Ok(
            WebDriverJSValue::String(
                FromJSValConvertible::from_jsval(cx, val, StringificationBehavior::Default).unwrap()))
    } else if val.is_null() {
        Ok(WebDriverJSValue::Null)
    } else {
        Err(WebDriverJSError::UnknownType)
    }
}

pub fn handle_execute_script(page: &Rc<Page>, pipeline: PipelineId, eval: String, reply: Sender<WebDriverJSResult>) {
    let page = get_page(&*page, pipeline);
    let window = page.window().root();
    let cx = window.r().get_cx();
    let rval = window.r().evaluate_js_on_global_with_result(&eval);

    reply.send(jsval_to_webdriver(cx, rval)).unwrap();
}

pub fn handle_execute_async_script(page: &Rc<Page>, pipeline: PipelineId, eval: String,
                                   reply: Sender<WebDriverJSResult>) {
    let page = get_page(&*page, pipeline);
    let window = page.window().root();
    window.r().set_webdriver_script_chan(Some(reply));
    window.r().evaluate_js_on_global_with_result(&eval);
}

pub fn handle_get_frame_id(page: &Rc<Page>,
                           pipeline: PipelineId,
                           webdriver_frame_id: WebDriverFrameId,
                           reply: Sender<Result<Option<(PipelineId, SubpageId)>, ()>>) {
    let window = match webdriver_frame_id {
        WebDriverFrameId::Short(_) => {
            // This isn't supported yet
            Ok(None)
        },
        WebDriverFrameId::Element(x) => {
            match find_node_by_unique_id(page, pipeline, x) {
                Some(ref node) => {
                    match HTMLIFrameElementCast::to_ref(node.root().r()) {
                        Some(ref elem) => Ok(elem.GetContentWindow()),
                        None => Err(())
                    }
                },
                None => Err(())
            }
        },
        WebDriverFrameId::Parent => {
            let window = page.window();
            Ok(window.root().r().parent())
        }
    };

    let frame_id = window.map(|x| x.and_then(|x| x.root().r().parent_info()));
    reply.send(frame_id).unwrap()
}

pub fn handle_find_element_css(page: &Rc<Page>, _pipeline: PipelineId, selector: String,
                               reply: Sender<Result<Option<String>, ()>>) {
    reply.send(match page.document().root().r().QuerySelector(selector.clone()) {
        Ok(node) => {
            let result = node.map(|x| NodeCast::from_ref(x.root().r()).get_unique_id());
            Ok(result)
        }
        Err(_) => Err(())
    }).unwrap();
}

pub fn handle_find_elements_css(page: &Rc<Page>, _pipeline: PipelineId, selector: String,
                                reply: Sender<Result<Vec<String>, ()>>) {
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
