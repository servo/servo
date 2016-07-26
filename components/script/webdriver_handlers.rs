/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cookie_rs::Cookie;
use dom::bindings::codegen::Bindings::CSSStyleDeclarationBinding::CSSStyleDeclarationMethods;
use dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use dom::bindings::codegen::Bindings::ElementBinding::ElementMethods;
use dom::bindings::codegen::Bindings::HTMLElementBinding::HTMLElementMethods;
use dom::bindings::codegen::Bindings::HTMLInputElementBinding::HTMLInputElementMethods;
use dom::bindings::codegen::Bindings::HTMLOptionElementBinding::HTMLOptionElementMethods;
use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::codegen::Bindings::NodeListBinding::NodeListMethods;
use dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use dom::bindings::conversions::{FromJSValConvertible, StringificationBehavior};
use dom::bindings::inheritance::Castable;
use dom::bindings::js::Root;
use dom::bindings::str::DOMString;
use dom::browsingcontext::BrowsingContext;
use dom::element::Element;
use dom::htmlelement::HTMLElement;
use dom::htmliframeelement::HTMLIFrameElement;
use dom::htmlinputelement::HTMLInputElement;
use dom::htmloptionelement::HTMLOptionElement;
use dom::node::Node;
use dom::window::ScriptHelpers;
use euclid::point::Point2D;
use euclid::rect::Rect;
use euclid::size::Size2D;
use ipc_channel::ipc::{self, IpcSender};
use js::jsapi::{JSContext, HandleValue};
use js::jsval::UndefinedValue;
use msg::constellation_msg::PipelineId;
use net_traits::CookieSource::{HTTP, NonHTTP};
use net_traits::CoreResourceMsg::{GetCookiesDataForUrl, SetCookiesForUrlWithData};
use net_traits::IpcSend;
use script_traits::webdriver_msg::WebDriverCookieError;
use script_traits::webdriver_msg::{WebDriverFrameId, WebDriverJSError, WebDriverJSResult, WebDriverJSValue};
use url::Url;

fn find_node_by_unique_id(context: &BrowsingContext,
                          pipeline: PipelineId,
                          node_id: String)
                          -> Option<Root<Node>> {
    let context = match context.find(pipeline) {
        Some(context) => context,
        None => return None
    };

    let document = context.active_document();
    document.upcast::<Node>().traverse_preorder().find(|candidate| candidate.unique_id() == node_id)
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
pub fn handle_execute_script(context: &BrowsingContext,
                             pipeline: PipelineId,
                             eval: String,
                             reply: IpcSender<WebDriverJSResult>) {
    let context = match context.find(pipeline) {
        Some(context) => context,
        None => return reply.send(Err(WebDriverJSError::BrowsingContextNotFound)).unwrap()
    };

    let window = context.active_window();
    let result = unsafe {
        let cx = window.get_cx();
        rooted!(in(cx) let mut rval = UndefinedValue());
        window.evaluate_js_on_global_with_result(&eval, rval.handle_mut());
        jsval_to_webdriver(cx, rval.handle())
    };
    reply.send(result).unwrap();
}

pub fn handle_execute_async_script(context: &BrowsingContext,
                                   pipeline: PipelineId,
                                   eval: String,
                                   reply: IpcSender<WebDriverJSResult>) {
    let context = match context.find(pipeline) {
       Some(context) => context,
       None => return reply.send(Err(WebDriverJSError::BrowsingContextNotFound)).unwrap()
   };

    let window = context.active_window();
    let cx = window.get_cx();
    window.set_webdriver_script_chan(Some(reply));
    rooted!(in(cx) let mut rval = UndefinedValue());
    window.evaluate_js_on_global_with_result(&eval, rval.handle_mut());
}

pub fn handle_get_frame_id(context: &BrowsingContext,
                           pipeline: PipelineId,
                           webdriver_frame_id: WebDriverFrameId,
                           reply: IpcSender<Result<Option<PipelineId>, ()>>) {
    let window = match webdriver_frame_id {
        WebDriverFrameId::Short(_) => {
            // This isn't supported yet
            Ok(None)
        },
        WebDriverFrameId::Element(x) => {
            match find_node_by_unique_id(context, pipeline, x) {
                Some(ref node) => {
                    match node.downcast::<HTMLIFrameElement>() {
                        Some(ref elem) => Ok(elem.get_content_window()),
                        None => Err(())
                    }
                },
                None => Err(())
            }
        },
        WebDriverFrameId::Parent => {
            let window = context.active_window();
            Ok(window.parent())
        }
    };

    let frame_id = window.map(|x| x.map(|x| x.pipeline()));
    reply.send(frame_id).unwrap()
}

pub fn handle_find_element_css(context: &BrowsingContext, _pipeline: PipelineId, selector: String,
                               reply: IpcSender<Result<Option<String>, ()>>) {
    reply.send(match context.active_document().QuerySelector(DOMString::from(selector)) {
        Ok(node) => {
            Ok(node.map(|x| x.upcast::<Node>().unique_id()))
        }
        Err(_) => Err(())
    }).unwrap();
}

pub fn handle_find_elements_css(context: &BrowsingContext,
                                _pipeline: PipelineId,
                                selector: String,
                                reply: IpcSender<Result<Vec<String>, ()>>) {
    reply.send(match context.active_document().QuerySelectorAll(DOMString::from(selector)) {
        Ok(ref nodes) => {
            let mut result = Vec::with_capacity(nodes.Length() as usize);
            for i in 0..nodes.Length() {
                if let Some(ref node) = nodes.Item(i) {
                    result.push(node.unique_id());
                }
            }
            Ok(result)
        },
        Err(_) => {
            Err(())
        }
    }).unwrap();
}

pub fn handle_focus_element(context: &BrowsingContext,
                            pipeline: PipelineId,
                            element_id: String,
                            reply: IpcSender<Result<(), ()>>) {
    reply.send(match find_node_by_unique_id(context, pipeline, element_id) {
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

pub fn handle_get_active_element(context: &BrowsingContext,
                                 _pipeline: PipelineId,
                                 reply: IpcSender<Option<String>>) {
    reply.send(context.active_document().GetActiveElement().map(
        |elem| elem.upcast::<Node>().unique_id())).unwrap();
}

pub fn handle_get_cookies(context: &BrowsingContext,
                         _pipeline: PipelineId,
                         reply: IpcSender<Vec<Cookie>>) {
    let document = context.active_document();
    let url = document.url();
    let (sender, receiver) = ipc::channel().unwrap();
    let _ = document.window().resource_threads().send(
        GetCookiesDataForUrl(url.clone(), sender, NonHTTP)
        );
    let cookies = receiver.recv().unwrap();
    reply.send(cookies).unwrap();
}

// https://w3c.github.io/webdriver/webdriver-spec.html#get-cookie
pub fn handle_get_cookie(context: &BrowsingContext,
                         _pipeline: PipelineId,
                         name: String,
                         reply: IpcSender<Vec<Cookie>>) {
    let document = context.active_document();
    let url = document.url();
    let (sender, receiver) = ipc::channel().unwrap();
    let _ = document.window().resource_threads().send(
        GetCookiesDataForUrl(url.clone(), sender, NonHTTP)
        );
    let cookies = receiver.recv().unwrap();
    reply.send(cookies.into_iter().filter(|c| c.name == &*name).collect()).unwrap();
}

// https://w3c.github.io/webdriver/webdriver-spec.html#add-cookie
pub fn handle_add_cookie(context: &BrowsingContext,
                         _pipeline: PipelineId,
                         cookie: Cookie,
                         reply: IpcSender<Result<(), WebDriverCookieError>>) {
    let document = context.active_document();
    let url = document.url();
    let method = if cookie.httponly {
        HTTP
    } else {
        NonHTTP
    };
    reply.send(match (document.is_cookie_averse(), cookie.domain.clone()) {
        (true, _) => Err(WebDriverCookieError::InvalidDomain),
        (false, Some(ref domain)) if url.host_str().map(|x| { x == &**domain }).unwrap_or(false) => {
            let _ = document.window().resource_threads().send(
                SetCookiesForUrlWithData(url.clone(), cookie, method)
                );
            Ok(())
        },
        (false, None) => {
            let _ = document.window().resource_threads().send(
                SetCookiesForUrlWithData(url.clone(), cookie, method)
                );
            Ok(())
        },
        (_, _) => {
            Err(WebDriverCookieError::UnableToSetCookie)
        },
    }).unwrap();
}

pub fn handle_get_title(context: &BrowsingContext, _pipeline: PipelineId, reply: IpcSender<String>) {
    reply.send(String::from(context.active_document().Title())).unwrap();
}

pub fn handle_get_rect(context: &BrowsingContext,
                       pipeline: PipelineId,
                       element_id: String,
                       reply: IpcSender<Result<Rect<f64>, ()>>) {
    reply.send(match find_node_by_unique_id(context, pipeline, element_id) {
        Some(elem) => {
            // https://w3c.github.io/webdriver/webdriver-spec.html#dfn-calculate-the-absolute-position
            match elem.downcast::<HTMLElement>() {
                Some(html_elem) => {
                    // Step 1
                    let mut x = 0;
                    let mut y = 0;

                    let mut offset_parent = html_elem.GetOffsetParent();

                    // Step 2
                    while let Some(element) = offset_parent {
                        offset_parent = match element.downcast::<HTMLElement>() {
                            Some(elem) => {
                                x += elem.OffsetLeft();
                                y += elem.OffsetTop();
                                elem.GetOffsetParent()
                            },
                            None => None
                        };
                    }
                    // Step 3
                    Ok(Rect::new(Point2D::new(x as f64, y as f64),
                                 Size2D::new(html_elem.OffsetWidth() as f64,
                                             html_elem.OffsetHeight() as f64)))
                },
                None => Err(())
            }
        },
        None => Err(())
    }).unwrap();
}

pub fn handle_get_text(context: &BrowsingContext,
                       pipeline: PipelineId,
                       node_id: String,
                       reply: IpcSender<Result<String, ()>>) {
    reply.send(match find_node_by_unique_id(context, pipeline, node_id) {
        Some(ref node) => {
            Ok(node.GetTextContent().map_or("".to_owned(), String::from))
        },
        None => Err(())
    }).unwrap();
}

pub fn handle_get_name(context: &BrowsingContext,
                       pipeline: PipelineId,
                       node_id: String,
                       reply: IpcSender<Result<String, ()>>) {
    reply.send(match find_node_by_unique_id(context, pipeline, node_id) {
        Some(node) => {
            Ok(String::from(node.downcast::<Element>().unwrap().TagName()))
        },
        None => Err(())
    }).unwrap();
}

pub fn handle_get_attribute(context: &BrowsingContext,
                            pipeline: PipelineId,
                            node_id: String,
                            name: String,
                            reply: IpcSender<Result<Option<String>, ()>>) {
    reply.send(match find_node_by_unique_id(context, pipeline, node_id) {
        Some(node) => {
            Ok(node.downcast::<Element>().unwrap().GetAttribute(DOMString::from(name))
               .map(String::from))
        },
        None => Err(())
    }).unwrap();
}

pub fn handle_get_css(context: &BrowsingContext,
                      pipeline: PipelineId,
                      node_id: String,
                      name: String,
                      reply: IpcSender<Result<String, ()>>) {
    reply.send(match find_node_by_unique_id(context, pipeline, node_id) {
        Some(node) => {
            let window = context.active_window();
            let elem = node.downcast::<Element>().unwrap();
            Ok(String::from(
                window.GetComputedStyle(&elem, None).GetPropertyValue(DOMString::from(name))))
        },
        None => Err(())
    }).unwrap();
}

pub fn handle_get_url(context: &BrowsingContext,
                      _pipeline: PipelineId,
                      reply: IpcSender<Url>) {
    let document = context.active_document();
    let url = document.url();
    reply.send((*url).clone()).unwrap();
}

pub fn handle_is_enabled(context: &BrowsingContext,
                         pipeline: PipelineId,
                         element_id: String,
                         reply: IpcSender<Result<bool, ()>>) {
    reply.send(match find_node_by_unique_id(&context, pipeline, element_id) {
        Some(ref node) => {
            match node.downcast::<Element>() {
                Some(elem) => Ok(elem.enabled_state()),
                None => Err(())
            }
        },
        None => Err(())
    }).unwrap();
}

pub fn handle_is_selected(context: &BrowsingContext,
                          pipeline: PipelineId,
                          element_id: String,
                          reply: IpcSender<Result<bool, ()>>) {
    reply.send(match find_node_by_unique_id(context, pipeline, element_id) {
        Some(ref node) => {
            if let Some(input_element) = node.downcast::<HTMLInputElement>() {
                Ok(input_element.Checked())
            }
            else if let Some(option_element) = node.downcast::<HTMLOptionElement>() {
                Ok(option_element.Selected())
            }
            else if let Some(_) = node.downcast::<HTMLElement>() {
                Ok(false) // regular elements are not selectable
            } else {
                Err(())
            }
        },
        None => Err(())
    }).unwrap();
}
