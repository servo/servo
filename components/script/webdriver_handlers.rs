/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::CSSStyleDeclarationBinding::CSSStyleDeclarationMethods;
use crate::dom::bindings::codegen::Bindings::DOMRectBinding::DOMRectMethods;
use crate::dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use crate::dom::bindings::codegen::Bindings::ElementBinding::ElementMethods;
use crate::dom::bindings::codegen::Bindings::HTMLElementBinding::HTMLElementMethods;
use crate::dom::bindings::codegen::Bindings::HTMLInputElementBinding::HTMLInputElementMethods;
use crate::dom::bindings::codegen::Bindings::HTMLOptionElementBinding::HTMLOptionElementMethods;
use crate::dom::bindings::codegen::Bindings::HTMLSelectElementBinding::HTMLSelectElementMethods;
use crate::dom::bindings::codegen::Bindings::NodeBinding::{GetRootNodeOptions, NodeMethods};
use crate::dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use crate::dom::bindings::codegen::Bindings::XMLSerializerBinding::XMLSerializerMethods;
use crate::dom::bindings::conversions::{
    get_property, get_property_jsval, is_array_like, root_from_object,
};
use crate::dom::bindings::conversions::{
    ConversionBehavior, ConversionResult, FromJSValConvertible, StringificationBehavior,
};
use crate::dom::bindings::error::{throw_dom_exception, Error};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::DomObject;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::element::Element;
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::htmldatalistelement::HTMLDataListElement;
use crate::dom::htmlelement::HTMLElement;
use crate::dom::htmliframeelement::HTMLIFrameElement;
use crate::dom::htmlinputelement::{HTMLInputElement, InputType};
use crate::dom::htmloptionelement::HTMLOptionElement;
use crate::dom::htmlselectelement::HTMLSelectElement;
use crate::dom::node::{window_from_node, Node, ShadowIncluding};
use crate::dom::nodelist::NodeList;
use crate::dom::window::Window;
use crate::dom::xmlserializer::XMLSerializer;
use crate::realms::enter_realm;
use crate::script_runtime::JSContext as SafeJSContext;
use crate::script_thread::{Documents, ScriptThread};
use cookie::Cookie;
use euclid::default::{Point2D, Rect, Size2D};
use hyper_serde::Serde;
use ipc_channel::ipc::{self, IpcSender};
use js::jsapi::{HandleValueArray, JSAutoRealm, JSContext, JSType, JS_IsExceptionPending};
use js::jsval::UndefinedValue;
use js::rust::wrappers::{JS_CallFunctionName, JS_GetProperty, JS_HasOwnProperty, JS_TypeOfValue};
use js::rust::{Handle, HandleObject, HandleValue};
use msg::constellation_msg::BrowsingContextId;
use msg::constellation_msg::PipelineId;
use net_traits::CookieSource::{NonHTTP, HTTP};
use net_traits::CoreResourceMsg::{DeleteCookies, GetCookiesDataForUrl, SetCookieForUrl};
use net_traits::IpcSend;
use script_traits::webdriver_msg::WebDriverCookieError;
use script_traits::webdriver_msg::{
    WebDriverFrameId, WebDriverJSError, WebDriverJSResult, WebDriverJSValue,
};
use servo_url::ServoUrl;
use std::cmp;
use std::collections::HashMap;
use std::ffi::CString;
use webdriver::common::{WebElement, WebFrame, WebWindow};
use webdriver::error::ErrorStatus;

fn find_node_by_unique_id(
    documents: &Documents,
    pipeline: PipelineId,
    node_id: String,
) -> Result<DomRoot<Node>, ErrorStatus> {
    match documents.find_document(pipeline).and_then(|document| {
        document
            .upcast::<Node>()
            .traverse_preorder(ShadowIncluding::Yes)
            .find(|node| node.unique_id() == node_id)
    }) {
        Some(node) => Ok(node),
        None => {
            if ScriptThread::has_node_id(&node_id) {
                Err(ErrorStatus::StaleElementReference)
            } else {
                Err(ErrorStatus::NoSuchElement)
            }
        },
    }
}

fn matching_links<'a>(
    links: &'a NodeList,
    link_text: String,
    partial: bool,
) -> impl Iterator<Item = String> + 'a {
    links
        .iter()
        .filter(move |node| {
            let content = node
                .GetTextContent()
                .map_or("".to_owned(), String::from)
                .trim()
                .to_owned();
            if partial {
                content.contains(&link_text)
            } else {
                content == link_text
            }
        })
        .map(|node| node.upcast::<Node>().unique_id())
}

fn all_matching_links(
    root_node: &Node,
    link_text: String,
    partial: bool,
) -> Result<Vec<String>, ErrorStatus> {
    root_node
        .query_selector_all(DOMString::from("a"))
        .map_err(|_| ErrorStatus::UnknownError)
        .map(|nodes| matching_links(&nodes, link_text, partial).collect())
}

fn first_matching_link(
    root_node: &Node,
    link_text: String,
    partial: bool,
) -> Result<Option<String>, ErrorStatus> {
    root_node
        .query_selector_all(DOMString::from("a"))
        .map_err(|_| ErrorStatus::UnknownError)
        .map(|nodes| matching_links(&nodes, link_text, partial).take(1).next())
}

#[allow(unsafe_code)]
unsafe fn object_has_to_json_property(
    cx: *mut JSContext,
    global_scope: &GlobalScope,
    object: HandleObject,
) -> bool {
    let name = CString::new("toJSON").unwrap();
    let mut found = false;
    if JS_HasOwnProperty(cx, object, name.as_ptr(), &mut found) && found {
        rooted!(in(cx) let mut value = UndefinedValue());
        let result = JS_GetProperty(cx, object, name.as_ptr(), value.handle_mut());
        if !result {
            throw_dom_exception(SafeJSContext::from_ptr(cx), global_scope, Error::JSFailed);
            false
        } else {
            result && JS_TypeOfValue(cx, value.handle()) == JSType::JSTYPE_FUNCTION
        }
    } else if JS_IsExceptionPending(cx) {
        throw_dom_exception(SafeJSContext::from_ptr(cx), global_scope, Error::JSFailed);
        false
    } else {
        false
    }
}

#[allow(unsafe_code)]
pub unsafe fn jsval_to_webdriver(
    cx: *mut JSContext,
    global_scope: &GlobalScope,
    val: HandleValue,
) -> WebDriverJSResult {
    let _ac = enter_realm(global_scope);
    if val.get().is_undefined() {
        Ok(WebDriverJSValue::Undefined)
    } else if val.get().is_null() {
        Ok(WebDriverJSValue::Null)
    } else if val.get().is_boolean() {
        Ok(WebDriverJSValue::Boolean(val.get().to_boolean()))
    } else if val.get().is_double() || val.get().is_int32() {
        Ok(WebDriverJSValue::Number(
            match FromJSValConvertible::from_jsval(cx, val, ()).unwrap() {
                ConversionResult::Success(c) => c,
                _ => unreachable!(),
            },
        ))
    } else if val.get().is_string() {
        //FIXME: use jsstring_to_str when jsval grows to_jsstring
        let string: DOMString =
            match FromJSValConvertible::from_jsval(cx, val, StringificationBehavior::Default)
                .unwrap()
            {
                ConversionResult::Success(c) => c,
                _ => unreachable!(),
            };
        Ok(WebDriverJSValue::String(String::from(string)))
    } else if val.get().is_object() {
        rooted!(in(cx) let object = match FromJSValConvertible::from_jsval(cx, val, ()).unwrap() {
            ConversionResult::Success(object) => object,
            _ => unreachable!(),
        });
        let _ac = JSAutoRealm::new(cx, *object);

        if is_array_like(cx, val) {
            let mut result: Vec<WebDriverJSValue> = Vec::new();

            let length = match get_property::<u32>(
                cx,
                object.handle(),
                "length",
                ConversionBehavior::Default,
            ) {
                Ok(length) => match length {
                    Some(length) => length,
                    _ => return Err(WebDriverJSError::UnknownType),
                },
                Err(error) => {
                    throw_dom_exception(SafeJSContext::from_ptr(cx), global_scope, error);
                    return Err(WebDriverJSError::JSError);
                },
            };

            for i in 0..length {
                rooted!(in(cx) let mut item = UndefinedValue());
                match get_property_jsval(cx, object.handle(), &i.to_string(), item.handle_mut()) {
                    Ok(_) => match jsval_to_webdriver(cx, global_scope, item.handle()) {
                        Ok(converted_item) => result.push(converted_item),
                        err @ Err(_) => return err,
                    },
                    Err(error) => {
                        throw_dom_exception(SafeJSContext::from_ptr(cx), global_scope, error);
                        return Err(WebDriverJSError::JSError);
                    },
                }
            }

            Ok(WebDriverJSValue::ArrayLike(result))
        } else if let Ok(element) = root_from_object::<Element>(*object, cx) {
            Ok(WebDriverJSValue::Element(WebElement(
                element.upcast::<Node>().unique_id(),
            )))
        } else if let Ok(window) = root_from_object::<Window>(*object, cx) {
            let window_proxy = window.window_proxy();
            if window_proxy.is_browsing_context_discarded() {
                Err(WebDriverJSError::StaleElementReference)
            } else if window_proxy.browsing_context_id() ==
                window_proxy.top_level_browsing_context_id()
            {
                Ok(WebDriverJSValue::Window(WebWindow(
                    window.Document().upcast::<Node>().unique_id(),
                )))
            } else {
                Ok(WebDriverJSValue::Frame(WebFrame(
                    window.Document().upcast::<Node>().unique_id(),
                )))
            }
        } else if object_has_to_json_property(cx, global_scope, object.handle()) {
            let name = CString::new("toJSON").unwrap();
            rooted!(in(cx) let mut value = UndefinedValue());
            if JS_CallFunctionName(
                cx,
                object.handle(),
                name.as_ptr(),
                &mut HandleValueArray::new(),
                value.handle_mut(),
            ) {
                jsval_to_webdriver(cx, global_scope, Handle::new(&value))
            } else {
                throw_dom_exception(SafeJSContext::from_ptr(cx), global_scope, Error::JSFailed);
                Err(WebDriverJSError::JSError)
            }
        } else {
            let mut result = HashMap::new();

            let common_properties = vec!["x", "y", "width", "height", "key"];
            for property in common_properties.iter() {
                rooted!(in(cx) let mut item = UndefinedValue());
                if let Ok(_) = get_property_jsval(cx, object.handle(), property, item.handle_mut())
                {
                    if !item.is_undefined() {
                        if let Ok(value) = jsval_to_webdriver(cx, global_scope, item.handle()) {
                            result.insert(property.to_string(), value);
                        }
                    }
                } else {
                    throw_dom_exception(SafeJSContext::from_ptr(cx), global_scope, Error::JSFailed);
                    return Err(WebDriverJSError::JSError);
                }
            }

            Ok(WebDriverJSValue::Object(result))
        }
    } else {
        Err(WebDriverJSError::UnknownType)
    }
}

#[allow(unsafe_code)]
pub fn handle_execute_script(
    window: Option<DomRoot<Window>>,
    eval: String,
    reply: IpcSender<WebDriverJSResult>,
) {
    match window {
        Some(window) => {
            let result = unsafe {
                let cx = window.get_cx();
                rooted!(in(*cx) let mut rval = UndefinedValue());
                window
                    .upcast::<GlobalScope>()
                    .evaluate_js_on_global_with_result(&eval, rval.handle_mut());
                jsval_to_webdriver(*cx, &window.upcast::<GlobalScope>(), rval.handle())
            };

            reply.send(result).unwrap();
        },
        None => {
            reply
                .send(Err(WebDriverJSError::BrowsingContextNotFound))
                .unwrap();
        },
    }
}

pub fn handle_execute_async_script(
    window: Option<DomRoot<Window>>,
    eval: String,
    reply: IpcSender<WebDriverJSResult>,
) {
    match window {
        Some(window) => {
            let cx = window.get_cx();
            window.set_webdriver_script_chan(Some(reply));
            rooted!(in(*cx) let mut rval = UndefinedValue());
            window
                .upcast::<GlobalScope>()
                .evaluate_js_on_global_with_result(&eval, rval.handle_mut());
        },
        None => {
            reply
                .send(Err(WebDriverJSError::BrowsingContextNotFound))
                .unwrap();
        },
    }
}

pub fn handle_get_browsing_context_id(
    documents: &Documents,
    pipeline: PipelineId,
    webdriver_frame_id: WebDriverFrameId,
    reply: IpcSender<Result<BrowsingContextId, ErrorStatus>>,
) {
    reply
        .send(match webdriver_frame_id {
            WebDriverFrameId::Short(_) => {
                // This isn't supported yet
                Err(ErrorStatus::UnsupportedOperation)
            },
            WebDriverFrameId::Element(element_id) => {
                find_node_by_unique_id(documents, pipeline, element_id).and_then(|node| {
                    node.downcast::<HTMLIFrameElement>()
                        .and_then(|element| element.browsing_context_id())
                        .ok_or(ErrorStatus::NoSuchFrame)
                })
            },
            WebDriverFrameId::Parent => documents
                .find_window(pipeline)
                .and_then(|window| {
                    window
                        .window_proxy()
                        .parent()
                        .map(|parent| parent.browsing_context_id())
                })
                .ok_or(ErrorStatus::NoSuchFrame),
        })
        .unwrap();
}

// https://w3c.github.io/webdriver/#dfn-center-point
fn get_element_in_view_center_point(element: &Element) -> Option<Point2D<i64>> {
    window_from_node(element.upcast::<Node>())
        .Document()
        .GetBody()
        .map(DomRoot::upcast::<Element>)
        .and_then(|body| {
            element
                .GetClientRects()
                .iter()
                // Step 1
                .next()
                .map(|rectangle| {
                    let x = rectangle.X().round() as i64;
                    let y = rectangle.Y().round() as i64;
                    let width = rectangle.Width().round() as i64;
                    let height = rectangle.Height().round() as i64;

                    let client_width = body.ClientWidth() as i64;
                    let client_height = body.ClientHeight() as i64;

                    // Steps 2 - 5
                    let left = cmp::max(0, cmp::min(x, x + width));
                    let right = cmp::min(client_width, cmp::max(x, x + width));
                    let top = cmp::max(0, cmp::min(y, y + height));
                    let bottom = cmp::min(client_height, cmp::max(y, y + height));

                    // Steps 6 - 7
                    let x = (left + right) / 2;
                    let y = (top + bottom) / 2;

                    // Step 8
                    Point2D::new(x, y)
                })
        })
}

pub fn handle_get_element_in_view_center_point(
    documents: &Documents,
    pipeline: PipelineId,
    element_id: String,
    reply: IpcSender<Result<Option<(i64, i64)>, ErrorStatus>>,
) {
    reply
        .send(
            find_node_by_unique_id(documents, pipeline, element_id).map(|node| {
                get_element_in_view_center_point(node.downcast::<Element>().unwrap())
                    .map(|point| (point.x, point.y))
            }),
        )
        .unwrap();
}

pub fn handle_find_element_css(
    documents: &Documents,
    pipeline: PipelineId,
    selector: String,
    reply: IpcSender<Result<Option<String>, ErrorStatus>>,
) {
    reply
        .send(
            documents
                .find_document(pipeline)
                .ok_or(ErrorStatus::UnknownError)
                .and_then(|document| {
                    document
                        .QuerySelector(DOMString::from(selector))
                        .map_err(|_| ErrorStatus::InvalidSelector)
                })
                .map(|node| node.map(|x| x.upcast::<Node>().unique_id())),
        )
        .unwrap();
}

pub fn handle_find_element_link_text(
    documents: &Documents,
    pipeline: PipelineId,
    selector: String,
    partial: bool,
    reply: IpcSender<Result<Option<String>, ErrorStatus>>,
) {
    reply
        .send(
            documents
                .find_document(pipeline)
                .ok_or(ErrorStatus::UnknownError)
                .and_then(|document| {
                    first_matching_link(&document.upcast::<Node>(), selector.clone(), partial)
                }),
        )
        .unwrap();
}

pub fn handle_find_element_tag_name(
    documents: &Documents,
    pipeline: PipelineId,
    selector: String,
    reply: IpcSender<Result<Option<String>, ErrorStatus>>,
) {
    reply
        .send(
            documents
                .find_document(pipeline)
                .ok_or(ErrorStatus::UnknownError)
                .and_then(|document| {
                    Ok(document
                        .GetElementsByTagName(DOMString::from(selector))
                        .elements_iter()
                        .next())
                })
                .map(|node| node.map(|x| x.upcast::<Node>().unique_id())),
        )
        .unwrap();
}

pub fn handle_find_elements_css(
    documents: &Documents,
    pipeline: PipelineId,
    selector: String,
    reply: IpcSender<Result<Vec<String>, ErrorStatus>>,
) {
    reply
        .send(
            documents
                .find_document(pipeline)
                .ok_or(ErrorStatus::UnknownError)
                .and_then(|document| {
                    document
                        .QuerySelectorAll(DOMString::from(selector))
                        .map_err(|_| ErrorStatus::InvalidSelector)
                })
                .map(|nodes| {
                    nodes
                        .iter()
                        .map(|x| x.upcast::<Node>().unique_id())
                        .collect()
                }),
        )
        .unwrap();
}

pub fn handle_find_elements_link_text(
    documents: &Documents,
    pipeline: PipelineId,
    selector: String,
    partial: bool,
    reply: IpcSender<Result<Vec<String>, ErrorStatus>>,
) {
    reply
        .send(
            documents
                .find_document(pipeline)
                .ok_or(ErrorStatus::UnknownError)
                .and_then(|document| {
                    all_matching_links(&document.upcast::<Node>(), selector.clone(), partial)
                }),
        )
        .unwrap();
}

pub fn handle_find_elements_tag_name(
    documents: &Documents,
    pipeline: PipelineId,
    selector: String,
    reply: IpcSender<Result<Vec<String>, ErrorStatus>>,
) {
    reply
        .send(
            documents
                .find_document(pipeline)
                .ok_or(ErrorStatus::UnknownError)
                .and_then(|document| Ok(document.GetElementsByTagName(DOMString::from(selector))))
                .map(|nodes| {
                    nodes
                        .elements_iter()
                        .map(|x| x.upcast::<Node>().unique_id())
                        .collect::<Vec<String>>()
                }),
        )
        .unwrap();
}

pub fn handle_find_element_element_css(
    documents: &Documents,
    pipeline: PipelineId,
    element_id: String,
    selector: String,
    reply: IpcSender<Result<Option<String>, ErrorStatus>>,
) {
    reply
        .send(
            find_node_by_unique_id(documents, pipeline, element_id).and_then(|node| {
                node.query_selector(DOMString::from(selector))
                    .map_err(|_| ErrorStatus::InvalidSelector)
                    .map(|node| node.map(|x| x.upcast::<Node>().unique_id()))
            }),
        )
        .unwrap();
}

pub fn handle_find_element_element_link_text(
    documents: &Documents,
    pipeline: PipelineId,
    element_id: String,
    selector: String,
    partial: bool,
    reply: IpcSender<Result<Option<String>, ErrorStatus>>,
) {
    reply
        .send(
            find_node_by_unique_id(documents, pipeline, element_id)
                .and_then(|node| first_matching_link(&node, selector.clone(), partial)),
        )
        .unwrap();
}

pub fn handle_find_element_element_tag_name(
    documents: &Documents,
    pipeline: PipelineId,
    element_id: String,
    selector: String,
    reply: IpcSender<Result<Option<String>, ErrorStatus>>,
) {
    reply
        .send(
            find_node_by_unique_id(documents, pipeline, element_id).and_then(|node| match node
                .downcast::<Element>(
            ) {
                Some(element) => Ok(element
                    .GetElementsByTagName(DOMString::from(selector))
                    .elements_iter()
                    .next()
                    .map(|x| x.upcast::<Node>().unique_id())),
                None => Err(ErrorStatus::UnknownError),
            }),
        )
        .unwrap();
}

pub fn handle_find_element_elements_css(
    documents: &Documents,
    pipeline: PipelineId,
    element_id: String,
    selector: String,
    reply: IpcSender<Result<Vec<String>, ErrorStatus>>,
) {
    reply
        .send(
            find_node_by_unique_id(documents, pipeline, element_id).and_then(|node| {
                node.query_selector_all(DOMString::from(selector))
                    .map_err(|_| ErrorStatus::InvalidSelector)
                    .map(|nodes| {
                        nodes
                            .iter()
                            .map(|x| x.upcast::<Node>().unique_id())
                            .collect()
                    })
            }),
        )
        .unwrap();
}

pub fn handle_find_element_elements_link_text(
    documents: &Documents,
    pipeline: PipelineId,
    element_id: String,
    selector: String,
    partial: bool,
    reply: IpcSender<Result<Vec<String>, ErrorStatus>>,
) {
    reply
        .send(
            find_node_by_unique_id(documents, pipeline, element_id)
                .and_then(|node| all_matching_links(&node, selector.clone(), partial)),
        )
        .unwrap();
}

pub fn handle_find_element_elements_tag_name(
    documents: &Documents,
    pipeline: PipelineId,
    element_id: String,
    selector: String,
    reply: IpcSender<Result<Vec<String>, ErrorStatus>>,
) {
    reply
        .send(
            find_node_by_unique_id(documents, pipeline, element_id).and_then(|node| match node
                .downcast::<Element>(
            ) {
                Some(element) => Ok(element
                    .GetElementsByTagName(DOMString::from(selector))
                    .elements_iter()
                    .map(|x| x.upcast::<Node>().unique_id())
                    .collect::<Vec<String>>()),
                None => Err(ErrorStatus::UnknownError),
            }),
        )
        .unwrap();
}

pub fn handle_focus_element(
    documents: &Documents,
    pipeline: PipelineId,
    element_id: String,
    reply: IpcSender<Result<(), ErrorStatus>>,
) {
    reply
        .send(
            find_node_by_unique_id(documents, pipeline, element_id).and_then(|node| {
                match node.downcast::<HTMLElement>() {
                    Some(element) => {
                        // Need a way to find if this actually succeeded
                        element.Focus();
                        Ok(())
                    },
                    None => Err(ErrorStatus::UnknownError),
                }
            }),
        )
        .unwrap();
}

pub fn handle_get_active_element(
    documents: &Documents,
    pipeline: PipelineId,
    reply: IpcSender<Option<String>>,
) {
    reply
        .send(
            documents
                .find_document(pipeline)
                .and_then(|document| document.GetActiveElement())
                .map(|element| element.upcast::<Node>().unique_id()),
        )
        .unwrap();
}

pub fn handle_get_page_source(
    documents: &Documents,
    pipeline: PipelineId,
    reply: IpcSender<Result<String, ErrorStatus>>,
) {
    reply
        .send(
            documents
                .find_document(pipeline)
                .ok_or(ErrorStatus::UnknownError)
                .and_then(|document| match document.GetDocumentElement() {
                    Some(element) => match element.GetOuterHTML() {
                        Ok(source) => Ok(source.to_string()),
                        Err(_) => {
                            match XMLSerializer::new(document.window())
                                .SerializeToString(element.upcast::<Node>())
                            {
                                Ok(source) => Ok(source.to_string()),
                                Err(_) => Err(ErrorStatus::UnknownError),
                            }
                        },
                    },
                    None => Err(ErrorStatus::UnknownError),
                }),
        )
        .unwrap();
}

pub fn handle_get_cookies(
    documents: &Documents,
    pipeline: PipelineId,
    reply: IpcSender<Vec<Serde<Cookie<'static>>>>,
) {
    reply
        .send(
            // TODO: Return an error if the pipeline doesn't exist
            match documents.find_document(pipeline) {
                Some(document) => {
                    let url = document.url();
                    let (sender, receiver) = ipc::channel().unwrap();
                    let _ = document
                        .window()
                        .upcast::<GlobalScope>()
                        .resource_threads()
                        .send(GetCookiesDataForUrl(url, sender, NonHTTP));
                    receiver.recv().unwrap()
                },
                None => Vec::new(),
            },
        )
        .unwrap();
}

// https://w3c.github.io/webdriver/webdriver-spec.html#get-cookie
pub fn handle_get_cookie(
    documents: &Documents,
    pipeline: PipelineId,
    name: String,
    reply: IpcSender<Vec<Serde<Cookie<'static>>>>,
) {
    reply
        .send(
            // TODO: Return an error if the pipeline doesn't exist
            match documents.find_document(pipeline) {
                Some(document) => {
                    let url = document.url();
                    let (sender, receiver) = ipc::channel().unwrap();
                    let _ = document
                        .window()
                        .upcast::<GlobalScope>()
                        .resource_threads()
                        .send(GetCookiesDataForUrl(url, sender, NonHTTP));
                    let cookies = receiver.recv().unwrap();
                    cookies
                        .into_iter()
                        .filter(|cookie| cookie.name() == &*name)
                        .collect()
                },
                None => Vec::new(),
            },
        )
        .unwrap();
}

// https://w3c.github.io/webdriver/webdriver-spec.html#add-cookie
pub fn handle_add_cookie(
    documents: &Documents,
    pipeline: PipelineId,
    cookie: Cookie<'static>,
    reply: IpcSender<Result<(), WebDriverCookieError>>,
) {
    // TODO: Return a different error if the pipeline doesn't exist
    let document = match documents.find_document(pipeline) {
        Some(document) => document,
        None => {
            return reply
                .send(Err(WebDriverCookieError::UnableToSetCookie))
                .unwrap();
        },
    };
    let url = document.url();
    let method = if cookie.http_only().unwrap_or(false) {
        HTTP
    } else {
        NonHTTP
    };

    let domain = cookie.domain().map(ToOwned::to_owned);
    reply
        .send(match (document.is_cookie_averse(), domain) {
            (true, _) => Err(WebDriverCookieError::InvalidDomain),
            (false, Some(ref domain)) if url.host_str().map(|x| x == domain).unwrap_or(false) => {
                let _ = document
                    .window()
                    .upcast::<GlobalScope>()
                    .resource_threads()
                    .send(SetCookieForUrl(url, Serde(cookie), method));
                Ok(())
            },
            (false, None) => {
                let _ = document
                    .window()
                    .upcast::<GlobalScope>()
                    .resource_threads()
                    .send(SetCookieForUrl(url, Serde(cookie), method));
                Ok(())
            },
            (_, _) => Err(WebDriverCookieError::UnableToSetCookie),
        })
        .unwrap();
}

pub fn handle_delete_cookies(
    documents: &Documents,
    pipeline: PipelineId,
    reply: IpcSender<Result<(), ErrorStatus>>,
) {
    let document = match documents.find_document(pipeline) {
        Some(document) => document,
        None => {
            return reply.send(Err(ErrorStatus::UnknownError)).unwrap();
        },
    };
    let url = document.url();
    document
        .window()
        .upcast::<GlobalScope>()
        .resource_threads()
        .send(DeleteCookies(url))
        .unwrap();
    reply.send(Ok(())).unwrap();
}

pub fn handle_get_title(documents: &Documents, pipeline: PipelineId, reply: IpcSender<String>) {
    reply
        .send(
            // TODO: Return an error if the pipeline doesn't exist
            documents
                .find_document(pipeline)
                .map(|document| String::from(document.Title()))
                .unwrap_or_default(),
        )
        .unwrap();
}

pub fn handle_get_rect(
    documents: &Documents,
    pipeline: PipelineId,
    element_id: String,
    reply: IpcSender<Result<Rect<f64>, ErrorStatus>>,
) {
    reply
        .send(
            find_node_by_unique_id(documents, pipeline, element_id).and_then(|node| {
                // https://w3c.github.io/webdriver/webdriver-spec.html#dfn-calculate-the-absolute-position
                match node.downcast::<HTMLElement>() {
                    Some(html_element) => {
                        // Step 1
                        let mut x = 0;
                        let mut y = 0;

                        let mut offset_parent = html_element.GetOffsetParent();

                        // Step 2
                        while let Some(element) = offset_parent {
                            offset_parent = match element.downcast::<HTMLElement>() {
                                Some(elem) => {
                                    x += elem.OffsetLeft();
                                    y += elem.OffsetTop();
                                    elem.GetOffsetParent()
                                },
                                None => None,
                            };
                        }
                        // Step 3
                        Ok(Rect::new(
                            Point2D::new(x as f64, y as f64),
                            Size2D::new(
                                html_element.OffsetWidth() as f64,
                                html_element.OffsetHeight() as f64,
                            ),
                        ))
                    },
                    None => Err(ErrorStatus::UnknownError),
                }
            }),
        )
        .unwrap();
}

pub fn handle_get_bounding_client_rect(
    documents: &Documents,
    pipeline: PipelineId,
    element_id: String,
    reply: IpcSender<Result<Rect<f32>, ErrorStatus>>,
) {
    reply
        .send(
            find_node_by_unique_id(documents, pipeline, element_id).and_then(|node| match node
                .downcast::<Element>(
            ) {
                Some(element) => {
                    let rect = element.GetBoundingClientRect();
                    Ok(Rect::new(
                        Point2D::new(rect.X() as f32, rect.Y() as f32),
                        Size2D::new(rect.Width() as f32, rect.Height() as f32),
                    ))
                },
                None => Err(ErrorStatus::UnknownError),
            }),
        )
        .unwrap();
}

pub fn handle_get_text(
    documents: &Documents,
    pipeline: PipelineId,
    node_id: String,
    reply: IpcSender<Result<String, ErrorStatus>>,
) {
    reply
        .send(
            find_node_by_unique_id(documents, pipeline, node_id)
                .and_then(|node| Ok(node.GetTextContent().map_or("".to_owned(), String::from))),
        )
        .unwrap();
}

pub fn handle_get_name(
    documents: &Documents,
    pipeline: PipelineId,
    node_id: String,
    reply: IpcSender<Result<String, ErrorStatus>>,
) {
    reply
        .send(
            find_node_by_unique_id(documents, pipeline, node_id)
                .and_then(|node| Ok(String::from(node.downcast::<Element>().unwrap().TagName()))),
        )
        .unwrap();
}

pub fn handle_get_attribute(
    documents: &Documents,
    pipeline: PipelineId,
    node_id: String,
    name: String,
    reply: IpcSender<Result<Option<String>, ErrorStatus>>,
) {
    reply
        .send(
            find_node_by_unique_id(documents, pipeline, node_id).and_then(|node| {
                Ok(node
                    .downcast::<Element>()
                    .unwrap()
                    .GetAttribute(DOMString::from(name))
                    .map(String::from))
            }),
        )
        .unwrap();
}

#[allow(unsafe_code)]
pub fn handle_get_property(
    documents: &Documents,
    pipeline: PipelineId,
    node_id: String,
    name: String,
    reply: IpcSender<Result<WebDriverJSValue, ErrorStatus>>,
) {
    reply
        .send(
            find_node_by_unique_id(documents, pipeline, node_id).and_then(|node| {
                let document = documents.find_document(pipeline).unwrap();
                let _ac = enter_realm(&*document);
                let cx = document.window().get_cx();

                rooted!(in(*cx) let mut property = UndefinedValue());
                match unsafe {
                    get_property_jsval(
                        *cx,
                        node.reflector().get_jsobject(),
                        &name,
                        property.handle_mut(),
                    )
                } {
                    Ok(_) => match unsafe {
                        jsval_to_webdriver(*cx, &node.reflector().global(), property.handle())
                    } {
                        Ok(property) => Ok(property),
                        Err(_) => Ok(WebDriverJSValue::Undefined),
                    },
                    Err(error) => {
                        throw_dom_exception(cx, &node.reflector().global(), error);
                        Ok(WebDriverJSValue::Undefined)
                    },
                }
            }),
        )
        .unwrap();
}

pub fn handle_get_css(
    documents: &Documents,
    pipeline: PipelineId,
    node_id: String,
    name: String,
    reply: IpcSender<Result<String, ErrorStatus>>,
) {
    reply
        .send(
            find_node_by_unique_id(documents, pipeline, node_id).and_then(|node| {
                let window = window_from_node(&*node);
                let element = node.downcast::<Element>().unwrap();
                Ok(String::from(
                    window
                        .GetComputedStyle(&element, None)
                        .GetPropertyValue(DOMString::from(name)),
                ))
            }),
        )
        .unwrap();
}

pub fn handle_get_url(documents: &Documents, pipeline: PipelineId, reply: IpcSender<ServoUrl>) {
    reply
        .send(
            // TODO: Return an error if the pipeline doesn't exist.
            documents
                .find_document(pipeline)
                .map(|document| document.url())
                .unwrap_or_else(|| ServoUrl::parse("about:blank").expect("infallible")),
        )
        .unwrap();
}

// https://w3c.github.io/webdriver/#element-click
pub fn handle_element_click(
    documents: &Documents,
    pipeline: PipelineId,
    element_id: String,
    reply: IpcSender<Result<Option<String>, ErrorStatus>>,
) {
    reply
        .send(
            // Step 3
            find_node_by_unique_id(documents, pipeline, element_id).and_then(|node| {
                // Step 4
                if let Some(input_element) = node.downcast::<HTMLInputElement>() {
                    if input_element.input_type() == InputType::File {
                        return Err(ErrorStatus::InvalidArgument);
                    }
                }

                // Step 5
                // TODO: scroll into view

                // Step 6
                // TODO: return error if still not in view

                // Step 7
                // TODO: return error if obscured

                // Step 8
                match node.downcast::<HTMLOptionElement>() {
                    Some(option_element) => {
                        // https://w3c.github.io/webdriver/#dfn-container
                        let root_node = node.GetRootNode(&GetRootNodeOptions::empty());
                        let datalist_parent = node
                            .preceding_nodes(&root_node)
                            .find(|preceding| preceding.is::<HTMLDataListElement>());
                        let select_parent = node
                            .preceding_nodes(&root_node)
                            .find(|preceding| preceding.is::<HTMLSelectElement>());

                        // Step 8.1
                        let parent_node = match datalist_parent {
                            Some(datalist_parent) => datalist_parent,
                            None => match select_parent {
                                Some(select_parent) => select_parent,
                                None => return Err(ErrorStatus::UnknownError),
                            },
                        };

                        // Steps 8.2 - 8.4
                        let event_target = parent_node.upcast::<EventTarget>();
                        event_target.fire_event(atom!("mouseover"));
                        event_target.fire_event(atom!("mousemove"));
                        event_target.fire_event(atom!("mousedown"));

                        // Step 8.5
                        match parent_node.downcast::<HTMLElement>() {
                            Some(html_element) => html_element.Focus(),
                            None => return Err(ErrorStatus::UnknownError),
                        }

                        // Step 8.6
                        if !option_element.Disabled() {
                            // Step 8.6.1
                            event_target.fire_event(atom!("input"));

                            // Steps 8.6.2
                            let previous_selectedness = option_element.Selected();

                            // Step 8.6.3
                            match parent_node.downcast::<HTMLSelectElement>() {
                                Some(select_element) => {
                                    if select_element.Multiple() {
                                        option_element.SetSelected(!option_element.Selected());
                                    }
                                },
                                None => option_element.SetSelected(true),
                            }

                            // Step 8.6.4
                            if !previous_selectedness {
                                event_target.fire_event(atom!("change"));
                            }
                        }

                        // Steps 8.7 - 8.8
                        event_target.fire_event(atom!("mouseup"));
                        event_target.fire_event(atom!("click"));

                        Ok(None)
                    },
                    None => Ok(Some(node.unique_id())),
                }
            }),
        )
        .unwrap();
}

pub fn handle_is_enabled(
    documents: &Documents,
    pipeline: PipelineId,
    element_id: String,
    reply: IpcSender<Result<bool, ErrorStatus>>,
) {
    reply
        .send(
            find_node_by_unique_id(&documents, pipeline, element_id).and_then(|node| match node
                .downcast::<Element>(
            ) {
                Some(element) => Ok(element.enabled_state()),
                None => Err(ErrorStatus::UnknownError),
            }),
        )
        .unwrap();
}

pub fn handle_is_selected(
    documents: &Documents,
    pipeline: PipelineId,
    element_id: String,
    reply: IpcSender<Result<bool, ErrorStatus>>,
) {
    reply
        .send(
            find_node_by_unique_id(documents, pipeline, element_id).and_then(|node| {
                if let Some(input_element) = node.downcast::<HTMLInputElement>() {
                    Ok(input_element.Checked())
                } else if let Some(option_element) = node.downcast::<HTMLOptionElement>() {
                    Ok(option_element.Selected())
                } else if node.is::<HTMLElement>() {
                    Ok(false) // regular elements are not selectable
                } else {
                    Err(ErrorStatus::UnknownError)
                }
            }),
        )
        .unwrap();
}
