/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use dom::bindings::codegen::Bindings::ElementBinding::ElementMethods;
use dom::bindings::codegen::Bindings::HTMLElementBinding::HTMLElementMethods;
use dom::bindings::codegen::Bindings::HTMLIFrameElementBinding::HTMLIFrameElementMethods;
use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::codegen::Bindings::NodeListBinding::NodeListMethods;
use dom::bindings::conversions::{FromJSValConvertible, StringificationBehavior};
use dom::bindings::inheritance::Castable;
use dom::bindings::js::Root;
use dom::element::Element;
use dom::htmlelement::HTMLElement;
use dom::htmliframeelement::HTMLIFrameElement;
use dom::node::Node;
use dom::window::ScriptHelpers;
use ipc_channel::ipc::IpcSender;
use js::jsapi::JSContext;
use js::jsapi::{HandleValue, RootedValue};
use js::jsval::UndefinedValue;
use msg::constellation_msg::PipelineId;
use msg::webdriver_msg::{WebDriverFrameId, WebDriverJSError, WebDriverJSResult, WebDriverJSValue};
use page::Page;
use script_task::get_page;
use std::rc::Rc;
use url::Url;
use util::str::DOMString;

fn find_node_by_unique_id(page: &Rc<Page>, pipeline: PipelineId, node_id: String) -> Option<Root<Node>> {
    let page = get_page(&*page, pipeline);
    let document = page.document();
    let node = document.upcast::<Node>();

    for candidate in node.traverse_preorder() {
        if candidate.get_unique_id() == node_id {
            return Some(candidate);
        }
    }

    None
}

#[allow(unsafe_code)]
pub unsafe fn jsval_to_webdriver(cx: *mut JSContext, val: HandleValue) -> WebDriverJSResult {
    if val.get().is_undefined() {
        Ok(WebDriverJSValue::Undefined)
    } else if val.get().is_boolean() {
        Ok(WebDriverJSValue::Boolean(val.get().to_boolean()))
    } else if val.get().is_double() || val.get().is_int32() {
        Ok(WebDriverJSValue::Number(FromJSValConvertible::from_jsval(cx, val, ()).unwrap()))
    } else if val.get().is_string() {
        //FIXME: use jsstring_to_str when jsval grows to_jsstring
        let string: DOMString = FromJSValConvertible::from_jsval(cx, val, StringificationBehavior::Default).unwrap();
        Ok(WebDriverJSValue::String(String::from(string)))
    } else if val.get().is_null() {
        Ok(WebDriverJSValue::Null)
    } else {
        Err(WebDriverJSError::UnknownType)
    }
}

#[allow(unsafe_code)]
pub fn handle_execute_script(page: &Rc<Page>,
                             pipeline: PipelineId,
                             eval: String,
                             reply: IpcSender<WebDriverJSResult>) {
    let page = get_page(&*page, pipeline);
    let window = page.window();
    let result = unsafe {
        let cx = window.get_cx();
        let mut rval = RootedValue::new(cx, UndefinedValue());
        window.evaluate_js_on_global_with_result(&eval, rval.handle_mut());
        jsval_to_webdriver(cx, rval.handle())
    };
    reply.send(result).unwrap();
}

pub fn handle_execute_async_script(page: &Rc<Page>,
                                   pipeline: PipelineId,
                                   eval: String,
                                   reply: IpcSender<WebDriverJSResult>) {
    let page = get_page(&*page, pipeline);
    let window = page.window();
    let cx = window.get_cx();
    window.set_webdriver_script_chan(Some(reply));
    let mut rval = RootedValue::new(cx, UndefinedValue());
    window.evaluate_js_on_global_with_result(&eval, rval.handle_mut());
}

pub fn handle_get_frame_id(page: &Rc<Page>,
                           pipeline: PipelineId,
                           webdriver_frame_id: WebDriverFrameId,
                           reply: IpcSender<Result<Option<PipelineId>, ()>>) {
    let window = match webdriver_frame_id {
        WebDriverFrameId::Short(_) => {
            // This isn't supported yet
            Ok(None)
        },
        WebDriverFrameId::Element(x) => {
            match find_node_by_unique_id(page, pipeline, x) {
                Some(ref node) => {
                    match node.downcast::<HTMLIFrameElement>() {
                        Some(ref elem) => Ok(elem.GetContentWindow()),
                        None => Err(())
                    }
                },
                None => Err(())
            }
        },
        WebDriverFrameId::Parent => {
            let window = page.window();
            Ok(window.parent())
        }
    };

    let frame_id = window.map(|x| x.map(|x| x.pipeline()));
    reply.send(frame_id).unwrap()
}

pub fn handle_find_element_css(page: &Rc<Page>, _pipeline: PipelineId, selector: String,
                               reply: IpcSender<Result<Option<String>, ()>>) {
    reply.send(match page.document().QuerySelector(DOMString::from(selector)) {
        Ok(node) => {
            Ok(node.map(|x| x.upcast::<Node>().get_unique_id()))
        }
        Err(_) => Err(())
    }).unwrap();
}

pub fn handle_find_elements_css(page: &Rc<Page>,
                                _pipeline: PipelineId,
                                selector: String,
                                reply: IpcSender<Result<Vec<String>, ()>>) {
    reply.send(match page.document().QuerySelectorAll(DOMString::from(selector)) {
        Ok(ref nodes) => {
            let mut result = Vec::with_capacity(nodes.Length() as usize);
            for i in 0..nodes.Length() {
                if let Some(ref node) = nodes.Item(i) {
                    result.push(node.get_unique_id());
                }
            }
            Ok(result)
        },
        Err(_) => {
            Err(())
        }
    }).unwrap();
}

pub fn handle_focus_element(page: &Rc<Page>,
                            pipeline: PipelineId,
                            element_id: String,
                            reply: IpcSender<Result<(), ()>>) {
    reply.send(match find_node_by_unique_id(page, pipeline, element_id) {
        Some(ref node) => {
            match node.downcast::<HTMLElement>() {
                Some(ref elem) => {
                    // Need a way to find if this actually succeeded
                    elem.Focus();
                    Ok(())
                }
                None => Err(())
            }
        },
        None => Err(())
    }).unwrap();
}

pub fn handle_get_active_element(page: &Rc<Page>,
                                 _pipeline: PipelineId,
                                 reply: IpcSender<Option<String>>) {
    reply.send(page.document().GetActiveElement().map(
        |elem| elem.upcast::<Node>().get_unique_id())).unwrap();
}

pub fn handle_get_title(page: &Rc<Page>, _pipeline: PipelineId, reply: IpcSender<String>) {
    reply.send(String::from(page.document().Title())).unwrap();
}

pub fn handle_get_text(page: &Rc<Page>,
                       pipeline: PipelineId,
                       node_id: String,
                       reply: IpcSender<Result<String, ()>>) {
    reply.send(match find_node_by_unique_id(&*page, pipeline, node_id) {
        Some(ref node) => {
            Ok(node.GetTextContent().map(String::from).unwrap_or("".to_owned()))
        },
        None => Err(())
    }).unwrap();
}

pub fn handle_get_name(page: &Rc<Page>,
                       pipeline: PipelineId,
                       node_id: String,
                       reply: IpcSender<Result<String, ()>>) {
    reply.send(match find_node_by_unique_id(&*page, pipeline, node_id) {
        Some(node) => {
            Ok(String::from(node.downcast::<Element>().unwrap().TagName()))
        },
        None => Err(())
    }).unwrap();
}

pub fn handle_get_url(page: &Rc<Page>,
                      _pipeline: PipelineId,
                      reply: IpcSender<Url>) {
    let document = page.document();
    let url = document.url();
    reply.send((*url).clone()).unwrap();
}
