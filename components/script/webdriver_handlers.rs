/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::CSSStyleDeclarationBinding::CSSStyleDeclarationMethods;
use crate::dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use crate::dom::bindings::codegen::Bindings::ElementBinding::ElementMethods;
use crate::dom::bindings::codegen::Bindings::HTMLElementBinding::HTMLElementMethods;
use crate::dom::bindings::codegen::Bindings::HTMLInputElementBinding::HTMLInputElementMethods;
use crate::dom::bindings::codegen::Bindings::HTMLOptionElementBinding::HTMLOptionElementMethods;
use crate::dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use crate::dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use crate::dom::bindings::codegen::Bindings::XMLSerializerBinding::XMLSerializerMethods;
use crate::dom::bindings::conversions::{
    get_property, get_property_jsval, is_array_like, root_from_object,
};
use crate::dom::bindings::conversions::{
    ConversionBehavior, ConversionResult, FromJSValConvertible, StringificationBehavior,
};
use crate::dom::bindings::error::throw_dom_exception;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::DomObject;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::element::Element;
use crate::dom::globalscope::GlobalScope;
use crate::dom::htmlelement::HTMLElement;
use crate::dom::htmliframeelement::HTMLIFrameElement;
use crate::dom::htmlinputelement::HTMLInputElement;
use crate::dom::htmloptionelement::HTMLOptionElement;
use crate::dom::node::{window_from_node, Node, ShadowIncluding};
use crate::dom::nodelist::NodeList;
use crate::dom::window::Window;
use crate::dom::xmlserializer::XMLSerializer;
use crate::script_runtime::JSContext as SafeJSContext;
use crate::script_thread::{Documents, ScriptThread};
use cookie::Cookie;
use euclid::default::{Point2D, Rect, Size2D};
use hyper_serde::Serde;
use ipc_channel::ipc::{self, IpcSender};
use js::jsapi::{JSAutoRealm, JSContext};
use js::jsval::UndefinedValue;
use js::rust::HandleValue;
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
use webdriver::common::WebElement;
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
pub unsafe fn jsval_to_webdriver(
    cx: *mut JSContext,
    global_scope: &GlobalScope,
    val: HandleValue,
) -> WebDriverJSResult {
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

        if let Ok(element) = root_from_object::<HTMLElement>(*object, cx) {
            return Ok(WebDriverJSValue::Element(WebElement(
                element.upcast::<Node>().unique_id(),
            )));
        }

        if !is_array_like(cx, val) {
            return Err(WebDriverJSError::UnknownType);
        }

        let mut result: Vec<WebDriverJSValue> = Vec::new();

        let length =
            match get_property::<u32>(cx, object.handle(), "length", ConversionBehavior::Default) {
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
                let cx = documents.find_document(pipeline).unwrap().window().get_cx();

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
