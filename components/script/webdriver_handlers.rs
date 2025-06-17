/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cmp;
use std::collections::{HashMap, HashSet};
use std::ffi::CString;
use std::ptr::NonNull;

use base::id::{BrowsingContextId, PipelineId};
use cookie::Cookie;
use embedder_traits::{WebDriverFrameId, WebDriverJSError, WebDriverJSResult, WebDriverJSValue};
use euclid::default::{Point2D, Rect, Size2D};
use hyper_serde::Serde;
use ipc_channel::ipc::{self, IpcSender};
use js::jsapi::{
    self, GetPropertyKeys, HandleValueArray, JS_GetOwnPropertyDescriptorById, JS_GetPropertyById,
    JS_IsExceptionPending, JSAutoRealm, JSContext, JSType, PropertyDescriptor,
};
use js::jsval::UndefinedValue;
use js::rust::wrappers::{JS_CallFunctionName, JS_GetProperty, JS_HasOwnProperty, JS_TypeOfValue};
use js::rust::{HandleObject, HandleValue, IdVector, ToString};
use net_traits::CookieSource::{HTTP, NonHTTP};
use net_traits::CoreResourceMsg::{
    DeleteCookie, DeleteCookies, GetCookiesDataForUrl, SetCookieForUrl,
};
use net_traits::IpcSend;
use script_bindings::conversions::is_array_like;
use servo_url::ServoUrl;
use webdriver::common::{WebElement, WebFrame, WebWindow};
use webdriver::error::ErrorStatus;

use crate::document_collection::DocumentCollection;
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
    ConversionBehavior, ConversionResult, FromJSValConvertible, StringificationBehavior,
    get_property, get_property_jsval, jsid_to_string, jsstring_to_str, root_from_object,
};
use crate::dom::bindings::error::{Error, report_pending_exception, throw_dom_exception};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::{DomGlobal, DomObject};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::document::Document;
use crate::dom::element::Element;
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::htmldatalistelement::HTMLDataListElement;
use crate::dom::htmlelement::HTMLElement;
use crate::dom::htmliframeelement::HTMLIFrameElement;
use crate::dom::htmlinputelement::{HTMLInputElement, InputType};
use crate::dom::htmloptgroupelement::HTMLOptGroupElement;
use crate::dom::htmloptionelement::HTMLOptionElement;
use crate::dom::htmlselectelement::HTMLSelectElement;
use crate::dom::node::{Node, NodeTraits, ShadowIncluding};
use crate::dom::nodelist::NodeList;
use crate::dom::window::Window;
use crate::dom::xmlserializer::XMLSerializer;
use crate::realms::{AlreadyInRealm, InRealm, enter_realm};
use crate::script_module::ScriptFetchOptions;
use crate::script_runtime::{CanGc, JSContext as SafeJSContext};
use crate::script_thread::ScriptThread;

/// <https://w3c.github.io/webdriver/#dfn-get-a-known-element>
fn get_known_element(
    documents: &DocumentCollection,
    pipeline: PipelineId,
    node_id: String,
) -> Result<DomRoot<Element>, ErrorStatus> {
    let doc = documents
        .find_document(pipeline)
        .expect("webdriver_handlers::Document should exists");
    // Step 3. If node is not null and node does not implement Element
    // return error with error code no such element.
    find_node_by_unique_id_in_document(&doc, node_id).and_then(|node| {
        node.downcast::<Element>()
            .map(DomRoot::from_ref)
            .ok_or(ErrorStatus::NoSuchElement)
    })
}

// This is also used by `dom/window.rs`
pub(crate) fn find_node_by_unique_id_in_document(
    document: &Document,
    node_id: String,
) -> Result<DomRoot<Node>, ErrorStatus> {
    let pipeline = document.window().pipeline_id();
    match document
        .upcast::<Node>()
        .traverse_preorder(ShadowIncluding::Yes)
        .find(|node| node.unique_id(pipeline) == node_id)
    {
        Some(node) => Ok(node),
        None => {
            if ScriptThread::has_node_id(pipeline, &node_id) {
                Err(ErrorStatus::StaleElementReference)
            } else {
                Err(ErrorStatus::NoSuchElement)
            }
        },
    }
}

/// <https://w3c.github.io/webdriver/#dfn-link-text-selector>
fn matching_links(
    links: &NodeList,
    link_text: String,
    partial: bool,
    can_gc: CanGc,
) -> impl Iterator<Item = String> + '_ {
    links
        .iter()
        .filter(move |node| {
            let content = node
                .downcast::<HTMLElement>()
                .map(|element| element.InnerText(can_gc))
                .map_or("".to_owned(), String::from)
                .trim()
                .to_owned();
            if partial {
                content.contains(&link_text)
            } else {
                content == link_text
            }
        })
        .map(|node| node.unique_id(node.owner_doc().window().pipeline_id()))
}

fn all_matching_links(
    root_node: &Node,
    link_text: String,
    partial: bool,
    can_gc: CanGc,
) -> Result<Vec<String>, ErrorStatus> {
    // <https://w3c.github.io/webdriver/#dfn-find>
    // Step 7.2. If a DOMException, SyntaxError, XPathException, or other error occurs
    // during the execution of the element location strategy, return error invalid selector.
    root_node
        .query_selector_all(DOMString::from("a"))
        .map_err(|_| ErrorStatus::InvalidSelector)
        .map(|nodes| matching_links(&nodes, link_text, partial, can_gc).collect())
}

fn first_matching_link(
    root_node: &Node,
    link_text: String,
    partial: bool,
    can_gc: CanGc,
) -> Result<Option<String>, ErrorStatus> {
    // <https://w3c.github.io/webdriver/#dfn-find>
    // Step 7.2. If a DOMException, SyntaxError, XPathException, or other error occurs
    // during the execution of the element location strategy, return error invalid selector.
    root_node
        .query_selector_all(DOMString::from("a"))
        .map_err(|_| ErrorStatus::InvalidSelector)
        .map(|nodes| {
            matching_links(&nodes, link_text, partial, can_gc)
                .take(1)
                .next()
        })
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
            throw_dom_exception(
                SafeJSContext::from_ptr(cx),
                global_scope,
                Error::JSFailed,
                CanGc::note(),
            );
            false
        } else {
            result && JS_TypeOfValue(cx, value.handle()) == JSType::JSTYPE_FUNCTION
        }
    } else if JS_IsExceptionPending(cx) {
        throw_dom_exception(
            SafeJSContext::from_ptr(cx),
            global_scope,
            Error::JSFailed,
            CanGc::note(),
        );
        false
    } else {
        false
    }
}

#[allow(unsafe_code)]
/// <https://w3c.github.io/webdriver/#dfn-collection>
unsafe fn is_arguments_object(cx: *mut JSContext, value: HandleValue) -> bool {
    rooted!(in(cx) let class_name = ToString(cx, value));
    let Some(class_name) = NonNull::new(class_name.get()) else {
        return false;
    };
    jsstring_to_str(cx, class_name) == "[object Arguments]"
}

#[derive(Clone, Eq, Hash, PartialEq)]
struct HashableJSVal(u64);

impl From<HandleValue<'_>> for HashableJSVal {
    fn from(v: HandleValue<'_>) -> HashableJSVal {
        HashableJSVal(v.get().asBits_)
    }
}

#[allow(unsafe_code)]
/// <https://w3c.github.io/webdriver/#dfn-json-deserialize>
pub(crate) fn jsval_to_webdriver(
    cx: SafeJSContext,
    global_scope: &GlobalScope,
    val: HandleValue,
    realm: InRealm,
    can_gc: CanGc,
) -> WebDriverJSResult {
    let mut seen = HashSet::new();
    let result = unsafe { jsval_to_webdriver_inner(*cx, global_scope, val, &mut seen) };
    if result.is_err() {
        report_pending_exception(cx, true, realm, can_gc);
    }
    result
}

#[allow(unsafe_code)]
unsafe fn jsval_to_webdriver_inner(
    cx: *mut JSContext,
    global_scope: &GlobalScope,
    val: HandleValue,
    seen: &mut HashSet<HashableJSVal>,
) -> WebDriverJSResult {
    let _ac = enter_realm(global_scope);
    if val.get().is_undefined() {
        Ok(WebDriverJSValue::Undefined)
    } else if val.get().is_null() {
        Ok(WebDriverJSValue::Null)
    } else if val.get().is_boolean() {
        Ok(WebDriverJSValue::Boolean(val.get().to_boolean()))
    } else if val.get().is_int32() {
        Ok(WebDriverJSValue::Int(
            match FromJSValConvertible::from_jsval(cx, val, ConversionBehavior::Default).unwrap() {
                ConversionResult::Success(c) => c,
                _ => unreachable!(),
            },
        ))
    } else if val.get().is_double() {
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
    }
    // https://w3c.github.io/webdriver/#dfn-clone-an-object
    else if val.get().is_object() {
        let hashable = val.into();
        // Step 1. If value is in `seen`, return error with error code javascript error.
        if seen.contains(&hashable) {
            return Err(WebDriverJSError::JSError);
        }
        //Step 2. Append value to `seen`.
        seen.insert(hashable.clone());

        rooted!(in(cx) let object = match FromJSValConvertible::from_jsval(cx, val, ()).unwrap() {
            ConversionResult::Success(object) => object,
            _ => unreachable!(),
        });
        let _ac = JSAutoRealm::new(cx, *object);

        let return_val = if is_array_like::<crate::DomTypeHolder>(cx, val) ||
            is_arguments_object(cx, val)
        {
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
                    throw_dom_exception(
                        SafeJSContext::from_ptr(cx),
                        global_scope,
                        error,
                        CanGc::note(),
                    );
                    return Err(WebDriverJSError::JSError);
                },
            };
            // Step 4. For each enumerable property in value, run the following substeps:
            for i in 0..length {
                rooted!(in(cx) let mut item = UndefinedValue());
                match get_property_jsval(cx, object.handle(), &i.to_string(), item.handle_mut()) {
                    Ok(_) => {
                        match jsval_to_webdriver_inner(cx, global_scope, item.handle(), seen) {
                            Ok(converted_item) => result.push(converted_item),
                            err @ Err(_) => return err,
                        }
                    },
                    Err(error) => {
                        throw_dom_exception(
                            SafeJSContext::from_ptr(cx),
                            global_scope,
                            error,
                            CanGc::note(),
                        );
                        return Err(WebDriverJSError::JSError);
                    },
                }
            }
            Ok(WebDriverJSValue::ArrayLike(result))
        } else if let Ok(element) = root_from_object::<Element>(*object, cx) {
            Ok(WebDriverJSValue::Element(WebElement(
                element
                    .upcast::<Node>()
                    .unique_id(element.owner_document().window().pipeline_id()),
            )))
        } else if let Ok(window) = root_from_object::<Window>(*object, cx) {
            let window_proxy = window.window_proxy();
            if window_proxy.is_browsing_context_discarded() {
                return Err(WebDriverJSError::StaleElementReference);
            } else {
                let pipeline = window.pipeline_id();
                if window_proxy.browsing_context_id() == window_proxy.webview_id() {
                    Ok(WebDriverJSValue::Window(WebWindow(
                        window.Document().upcast::<Node>().unique_id(pipeline),
                    )))
                } else {
                    Ok(WebDriverJSValue::Frame(WebFrame(
                        window.Document().upcast::<Node>().unique_id(pipeline),
                    )))
                }
            }
        } else if object_has_to_json_property(cx, global_scope, object.handle()) {
            let name = CString::new("toJSON").unwrap();
            rooted!(in(cx) let mut value = UndefinedValue());
            if JS_CallFunctionName(
                cx,
                object.handle(),
                name.as_ptr(),
                &HandleValueArray::empty(),
                value.handle_mut(),
            ) {
                Ok(jsval_to_webdriver_inner(
                    cx,
                    global_scope,
                    value.handle(),
                    seen,
                )?)
            } else {
                throw_dom_exception(
                    SafeJSContext::from_ptr(cx),
                    global_scope,
                    Error::JSFailed,
                    CanGc::note(),
                );
                return Err(WebDriverJSError::JSError);
            }
        } else {
            let mut result = HashMap::new();

            let mut ids = IdVector::new(cx);
            if !GetPropertyKeys(
                cx,
                object.handle().into(),
                jsapi::JSITER_OWNONLY,
                ids.handle_mut(),
            ) {
                return Err(WebDriverJSError::JSError);
            }
            for id in ids.iter() {
                rooted!(in(cx) let id = *id);
                rooted!(in(cx) let mut desc = PropertyDescriptor::default());

                let mut is_none = false;
                if !JS_GetOwnPropertyDescriptorById(
                    cx,
                    object.handle().into(),
                    id.handle().into(),
                    desc.handle_mut().into(),
                    &mut is_none,
                ) {
                    return Err(WebDriverJSError::JSError);
                }

                rooted!(in(cx) let mut property = UndefinedValue());
                if !JS_GetPropertyById(
                    cx,
                    object.handle().into(),
                    id.handle().into(),
                    property.handle_mut().into(),
                ) {
                    return Err(WebDriverJSError::JSError);
                }
                if !property.is_undefined() {
                    let Some(name) = jsid_to_string(cx, id.handle()) else {
                        return Err(WebDriverJSError::JSError);
                    };

                    if let Ok(value) =
                        jsval_to_webdriver_inner(cx, global_scope, property.handle(), seen)
                    {
                        result.insert(name.into(), value);
                    } else {
                        return Err(WebDriverJSError::JSError);
                    }
                }
            }
            Ok(WebDriverJSValue::Object(result))
        };
        // Step 5. Remove the last element of `seen`.
        seen.remove(&hashable);
        // Step 6. Return success with data `result`.
        return_val
    } else {
        Err(WebDriverJSError::UnknownType)
    }
}

#[allow(unsafe_code)]
pub(crate) fn handle_execute_script(
    window: Option<DomRoot<Window>>,
    eval: String,
    reply: IpcSender<WebDriverJSResult>,
    can_gc: CanGc,
) {
    match window {
        Some(window) => {
            let cx = window.get_cx();
            let realm = AlreadyInRealm::assert_for_cx(cx);
            let realm = InRealm::already(&realm);

            rooted!(in(*cx) let mut rval = UndefinedValue());
            let global = window.as_global_scope();
            let result = if global.evaluate_js_on_global_with_result(
                &eval,
                rval.handle_mut(),
                ScriptFetchOptions::default_classic_script(global),
                global.api_base_url(),
                can_gc,
            ) {
                jsval_to_webdriver(cx, global, rval.handle(), realm, can_gc)
            } else {
                Err(WebDriverJSError::JSError)
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

pub(crate) fn handle_execute_async_script(
    window: Option<DomRoot<Window>>,
    eval: String,
    reply: IpcSender<WebDriverJSResult>,
    can_gc: CanGc,
) {
    match window {
        Some(window) => {
            let cx = window.get_cx();
            let reply_sender = reply.clone();
            window.set_webdriver_script_chan(Some(reply));
            rooted!(in(*cx) let mut rval = UndefinedValue());

            let global_scope = window.as_global_scope();
            if !global_scope.evaluate_js_on_global_with_result(
                &eval,
                rval.handle_mut(),
                ScriptFetchOptions::default_classic_script(global_scope),
                global_scope.api_base_url(),
                can_gc,
            ) {
                reply_sender.send(Err(WebDriverJSError::JSError)).unwrap();
            }
        },
        None => {
            reply
                .send(Err(WebDriverJSError::BrowsingContextNotFound))
                .unwrap();
        },
    }
}

pub(crate) fn handle_get_browsing_context_id(
    documents: &DocumentCollection,
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
                get_known_element(documents, pipeline, element_id).and_then(|element| {
                    element
                        .downcast::<HTMLIFrameElement>()
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
fn get_element_in_view_center_point(element: &Element, can_gc: CanGc) -> Option<Point2D<i64>> {
    element
        .owner_document()
        .GetBody()
        .map(DomRoot::upcast::<Element>)
        .and_then(|body| {
            // Step 1: Let rectangle be the first element of the DOMRect sequence
            // returned by calling getClientRects() on element.
            element.GetClientRects(can_gc).first().map(|rectangle| {
                let x = rectangle.X().round() as i64;
                let y = rectangle.Y().round() as i64;
                let width = rectangle.Width().round() as i64;
                let height = rectangle.Height().round() as i64;

                let client_width = body.ClientWidth(can_gc) as i64;
                let client_height = body.ClientHeight(can_gc) as i64;

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

pub(crate) fn handle_get_element_in_view_center_point(
    documents: &DocumentCollection,
    pipeline: PipelineId,
    element_id: String,
    reply: IpcSender<Result<Option<(i64, i64)>, ErrorStatus>>,
    can_gc: CanGc,
) {
    reply
        .send(
            get_known_element(documents, pipeline, element_id).map(|element| {
                get_element_in_view_center_point(&element, can_gc).map(|point| (point.x, point.y))
            }),
        )
        .unwrap();
}

fn retrieve_document_and_check_root_existence(
    documents: &DocumentCollection,
    pipeline: PipelineId,
) -> Result<DomRoot<Document>, ErrorStatus> {
    let document = documents
        .find_document(pipeline)
        .ok_or(ErrorStatus::NoSuchWindow)?;

    // <https://w3c.github.io/webdriver/#find-element>
    // <https://w3c.github.io/webdriver/#find-elements>
    // Step 7 - 8. If current browsing context's document element is null,
    // return error with error code no such element.
    if document.GetDocumentElement().is_none() {
        Err(ErrorStatus::NoSuchElement)
    } else {
        Ok(document)
    }
}

pub(crate) fn handle_find_element_css(
    documents: &DocumentCollection,
    pipeline: PipelineId,
    selector: String,
    reply: IpcSender<Result<Option<String>, ErrorStatus>>,
) {
    match retrieve_document_and_check_root_existence(documents, pipeline) {
        Ok(document) => reply
            .send(
                document
                    .QuerySelector(DOMString::from(selector))
                    .map_err(|_| ErrorStatus::InvalidSelector)
                    .map(|element| element.map(|x| x.upcast::<Node>().unique_id(pipeline))),
            )
            .unwrap(),
        Err(error) => reply.send(Err(error)).unwrap(),
    }
}

pub(crate) fn handle_find_element_link_text(
    documents: &DocumentCollection,
    pipeline: PipelineId,
    selector: String,
    partial: bool,
    reply: IpcSender<Result<Option<String>, ErrorStatus>>,
    can_gc: CanGc,
) {
    match retrieve_document_and_check_root_existence(documents, pipeline) {
        Ok(document) => reply
            .send(first_matching_link(
                document.upcast::<Node>(),
                selector.clone(),
                partial,
                can_gc,
            ))
            .unwrap(),
        Err(error) => reply.send(Err(error)).unwrap(),
    }
}

pub(crate) fn handle_find_element_tag_name(
    documents: &DocumentCollection,
    pipeline: PipelineId,
    selector: String,
    reply: IpcSender<Result<Option<String>, ErrorStatus>>,
    can_gc: CanGc,
) {
    match retrieve_document_and_check_root_existence(documents, pipeline) {
        Ok(document) => reply
            .send(Ok(document
                .GetElementsByTagName(DOMString::from(selector), can_gc)
                .elements_iter()
                .next()
                .map(|element| element.upcast::<Node>().unique_id(pipeline))))
            .unwrap(),
        Err(error) => reply.send(Err(error)).unwrap(),
    }
}

pub(crate) fn handle_find_elements_css(
    documents: &DocumentCollection,
    pipeline: PipelineId,
    selector: String,
    reply: IpcSender<Result<Vec<String>, ErrorStatus>>,
) {
    match retrieve_document_and_check_root_existence(documents, pipeline) {
        Ok(document) => reply
            .send(
                document
                    .QuerySelectorAll(DOMString::from(selector))
                    .map_err(|_| ErrorStatus::InvalidSelector)
                    .map(|nodes| {
                        nodes
                            .iter()
                            .map(|x| x.upcast::<Node>().unique_id(pipeline))
                            .collect()
                    }),
            )
            .unwrap(),
        Err(error) => reply.send(Err(error)).unwrap(),
    }
}

pub(crate) fn handle_find_elements_link_text(
    documents: &DocumentCollection,
    pipeline: PipelineId,
    selector: String,
    partial: bool,
    reply: IpcSender<Result<Vec<String>, ErrorStatus>>,
    can_gc: CanGc,
) {
    match retrieve_document_and_check_root_existence(documents, pipeline) {
        Ok(document) => reply
            .send(all_matching_links(
                document.upcast::<Node>(),
                selector.clone(),
                partial,
                can_gc,
            ))
            .unwrap(),
        Err(error) => reply.send(Err(error)).unwrap(),
    }
}

pub(crate) fn handle_find_elements_tag_name(
    documents: &DocumentCollection,
    pipeline: PipelineId,
    selector: String,
    reply: IpcSender<Result<Vec<String>, ErrorStatus>>,
    can_gc: CanGc,
) {
    match retrieve_document_and_check_root_existence(documents, pipeline) {
        Ok(document) => reply
            .send(Ok(document
                .GetElementsByTagName(DOMString::from(selector), can_gc)
                .elements_iter()
                .map(|x| x.upcast::<Node>().unique_id(pipeline))
                .collect::<Vec<String>>()))
            .unwrap(),
        Err(error) => reply.send(Err(error)).unwrap(),
    }
}

pub(crate) fn handle_find_element_element_css(
    documents: &DocumentCollection,
    pipeline: PipelineId,
    element_id: String,
    selector: String,
    reply: IpcSender<Result<Option<String>, ErrorStatus>>,
) {
    reply
        .send(
            get_known_element(documents, pipeline, element_id).and_then(|element| {
                element
                    .upcast::<Node>()
                    .query_selector(DOMString::from(selector))
                    .map_err(|_| ErrorStatus::InvalidSelector)
                    .map(|element| element.map(|x| x.upcast::<Node>().unique_id(pipeline)))
            }),
        )
        .unwrap();
}

pub(crate) fn handle_find_element_element_link_text(
    documents: &DocumentCollection,
    pipeline: PipelineId,
    element_id: String,
    selector: String,
    partial: bool,
    reply: IpcSender<Result<Option<String>, ErrorStatus>>,
    can_gc: CanGc,
) {
    reply
        .send(
            get_known_element(documents, pipeline, element_id).and_then(|element| {
                first_matching_link(element.upcast::<Node>(), selector.clone(), partial, can_gc)
            }),
        )
        .unwrap();
}

pub(crate) fn handle_find_element_element_tag_name(
    documents: &DocumentCollection,
    pipeline: PipelineId,
    element_id: String,
    selector: String,
    reply: IpcSender<Result<Option<String>, ErrorStatus>>,
    can_gc: CanGc,
) {
    reply
        .send(
            get_known_element(documents, pipeline, element_id).map(|element| {
                element
                    .GetElementsByTagName(DOMString::from(selector), can_gc)
                    .elements_iter()
                    .next()
                    .map(|x| x.upcast::<Node>().unique_id(pipeline))
            }),
        )
        .unwrap();
}

pub(crate) fn handle_find_element_elements_css(
    documents: &DocumentCollection,
    pipeline: PipelineId,
    element_id: String,
    selector: String,
    reply: IpcSender<Result<Vec<String>, ErrorStatus>>,
) {
    reply
        .send(
            get_known_element(documents, pipeline, element_id).and_then(|element| {
                element
                    .upcast::<Node>()
                    .query_selector_all(DOMString::from(selector))
                    .map_err(|_| ErrorStatus::InvalidSelector)
                    .map(|nodes| {
                        nodes
                            .iter()
                            .map(|x| x.upcast::<Node>().unique_id(pipeline))
                            .collect()
                    })
            }),
        )
        .unwrap();
}

pub(crate) fn handle_find_element_elements_link_text(
    documents: &DocumentCollection,
    pipeline: PipelineId,
    element_id: String,
    selector: String,
    partial: bool,
    reply: IpcSender<Result<Vec<String>, ErrorStatus>>,
    can_gc: CanGc,
) {
    reply
        .send(
            get_known_element(documents, pipeline, element_id).and_then(|element| {
                all_matching_links(element.upcast::<Node>(), selector.clone(), partial, can_gc)
            }),
        )
        .unwrap();
}

pub(crate) fn handle_find_element_elements_tag_name(
    documents: &DocumentCollection,
    pipeline: PipelineId,
    element_id: String,
    selector: String,
    reply: IpcSender<Result<Vec<String>, ErrorStatus>>,
    can_gc: CanGc,
) {
    reply
        .send(
            get_known_element(documents, pipeline, element_id).map(|element| {
                element
                    .GetElementsByTagName(DOMString::from(selector), can_gc)
                    .elements_iter()
                    .map(|x| x.upcast::<Node>().unique_id(pipeline))
                    .collect::<Vec<String>>()
            }),
        )
        .unwrap();
}

/// <https://www.w3.org/TR/webdriver2/#dfn-get-element-shadow-root>
pub(crate) fn handle_get_element_shadow_root(
    documents: &DocumentCollection,
    pipeline: PipelineId,
    element_id: String,
    reply: IpcSender<Result<Option<String>, ErrorStatus>>,
) {
    reply
        .send(
            get_known_element(documents, pipeline, element_id).map(|element| {
                element
                    .GetShadowRoot()
                    .map(|x| x.upcast::<Node>().unique_id(pipeline))
            }),
        )
        .unwrap();
}

pub(crate) fn handle_will_send_keys(
    documents: &DocumentCollection,
    pipeline: PipelineId,
    element_id: String,
    text: String,
    strict_file_interactability: bool,
    reply: IpcSender<Result<bool, ErrorStatus>>,
    can_gc: CanGc,
) {
    reply
        .send(
            get_known_element(documents, pipeline, element_id).and_then(|element| {
                // Step 6: Let file be true if element is input element
                // in the file upload state, or false otherwise
                let file_input = element
                    .downcast::<HTMLInputElement>()
                    .filter(|&input_element| input_element.input_type() == InputType::File);

                // Step 7: If file is false or the session's strict file interactability
                if file_input.is_none() || strict_file_interactability {
                    match element.downcast::<HTMLElement>() {
                        Some(element) => {
                            // Need a way to find if this actually succeeded
                            element.Focus(can_gc);
                        },
                        None => return Err(ErrorStatus::UnknownError),
                    }
                }

                // Step 8 (file input)
                if let Some(file_input) = file_input {
                    // Step 8.1: Let files be the result of splitting text
                    // on the newline (\n) character.
                    let files: Vec<DOMString> = text.split("\n").map(|s| s.into()).collect();

                    // Step 8.2
                    if files.is_empty() {
                        return Err(ErrorStatus::InvalidArgument);
                    }

                    // Step 8.3 - 8.4
                    if !file_input.Multiple() && files.len() > 1 {
                        return Err(ErrorStatus::InvalidArgument);
                    }

                    // Step 8.5
                    // TODO: Should return invalid argument error if file doesn't exist

                    // Step 8.6 - 8.7
                    // Input and change event already fired in `htmlinputelement.rs`.
                    file_input.SelectFiles(files, can_gc);

                    // Step 8.8
                    return Ok(false);
                }

                // TODO: Check non-typeable form control
                // TODO: Check content editable

                Ok(true)
            }),
        )
        .unwrap();
}

pub(crate) fn handle_get_active_element(
    documents: &DocumentCollection,
    pipeline: PipelineId,
    reply: IpcSender<Option<String>>,
) {
    reply
        .send(
            documents
                .find_document(pipeline)
                .and_then(|document| document.GetActiveElement())
                .map(|element| element.upcast::<Node>().unique_id(pipeline)),
        )
        .unwrap();
}

pub(crate) fn handle_get_computed_role(
    documents: &DocumentCollection,
    pipeline: PipelineId,
    node_id: String,
    reply: IpcSender<Result<Option<String>, ErrorStatus>>,
) {
    reply
        .send(
            get_known_element(documents, pipeline, node_id)
                .map(|element| element.GetRole().map(String::from)),
        )
        .unwrap();
}

pub(crate) fn handle_get_page_source(
    documents: &DocumentCollection,
    pipeline: PipelineId,
    reply: IpcSender<Result<String, ErrorStatus>>,
    can_gc: CanGc,
) {
    reply
        .send(
            documents
                .find_document(pipeline)
                .ok_or(ErrorStatus::UnknownError)
                .and_then(|document| match document.GetDocumentElement() {
                    Some(element) => match element.outer_html(can_gc) {
                        Ok(source) => Ok(source.to_string()),
                        Err(_) => {
                            match XMLSerializer::new(document.window(), None, can_gc)
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

pub(crate) fn handle_get_cookies(
    documents: &DocumentCollection,
    pipeline: PipelineId,
    reply: IpcSender<Result<Vec<Serde<Cookie<'static>>>, ErrorStatus>>,
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
                        .as_global_scope()
                        .resource_threads()
                        .send(GetCookiesDataForUrl(url, sender, NonHTTP));
                    Ok(receiver.recv().unwrap())
                },
                None => Ok(Vec::new()),
            },
        )
        .unwrap();
}

// https://w3c.github.io/webdriver/webdriver-spec.html#get-cookie
pub(crate) fn handle_get_cookie(
    documents: &DocumentCollection,
    pipeline: PipelineId,
    name: String,
    reply: IpcSender<Result<Vec<Serde<Cookie<'static>>>, ErrorStatus>>,
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
                        .as_global_scope()
                        .resource_threads()
                        .send(GetCookiesDataForUrl(url, sender, NonHTTP));
                    let cookies = receiver.recv().unwrap();
                    Ok(cookies
                        .into_iter()
                        .filter(|cookie| cookie.name() == &*name)
                        .collect())
                },
                None => Ok(Vec::new()),
            },
        )
        .unwrap();
}

// https://w3c.github.io/webdriver/webdriver-spec.html#add-cookie
pub(crate) fn handle_add_cookie(
    documents: &DocumentCollection,
    pipeline: PipelineId,
    cookie: Cookie<'static>,
    reply: IpcSender<Result<(), ErrorStatus>>,
) {
    // TODO: Return a different error if the pipeline doesn't exist
    let document = match documents.find_document(pipeline) {
        Some(document) => document,
        None => {
            return reply.send(Err(ErrorStatus::UnableToSetCookie)).unwrap();
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
            (true, _) => Err(ErrorStatus::InvalidCookieDomain),
            (false, Some(ref domain)) if url.host_str().map(|x| x == domain).unwrap_or(false) => {
                let _ = document
                    .window()
                    .as_global_scope()
                    .resource_threads()
                    .send(SetCookieForUrl(url, Serde(cookie), method));
                Ok(())
            },
            (false, None) => {
                let _ = document
                    .window()
                    .as_global_scope()
                    .resource_threads()
                    .send(SetCookieForUrl(url, Serde(cookie), method));
                Ok(())
            },
            (_, _) => Err(ErrorStatus::UnableToSetCookie),
        })
        .unwrap();
}

// https://w3c.github.io/webdriver/#delete-all-cookies
pub(crate) fn handle_delete_cookies(
    documents: &DocumentCollection,
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
        .as_global_scope()
        .resource_threads()
        .send(DeleteCookies(url))
        .unwrap();
    reply.send(Ok(())).unwrap();
}

// https://w3c.github.io/webdriver/#delete-cookie
pub(crate) fn handle_delete_cookie(
    documents: &DocumentCollection,
    pipeline: PipelineId,
    name: String,
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
        .as_global_scope()
        .resource_threads()
        .send(DeleteCookie(url, name))
        .unwrap();
    reply.send(Ok(())).unwrap();
}

pub(crate) fn handle_get_title(
    documents: &DocumentCollection,
    pipeline: PipelineId,
    reply: IpcSender<String>,
) {
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

pub(crate) fn handle_get_rect(
    documents: &DocumentCollection,
    pipeline: PipelineId,
    element_id: String,
    reply: IpcSender<Result<Rect<f64>, ErrorStatus>>,
    can_gc: CanGc,
) {
    reply
        .send(
            get_known_element(documents, pipeline, element_id).and_then(|element| {
                // https://w3c.github.io/webdriver/webdriver-spec.html#dfn-calculate-the-absolute-position
                match element.downcast::<HTMLElement>() {
                    Some(html_element) => {
                        // Step 1
                        let mut x = 0;
                        let mut y = 0;

                        let mut offset_parent = html_element.GetOffsetParent(can_gc);

                        // Step 2
                        while let Some(element) = offset_parent {
                            offset_parent = match element.downcast::<HTMLElement>() {
                                Some(elem) => {
                                    x += elem.OffsetLeft(can_gc);
                                    y += elem.OffsetTop(can_gc);
                                    elem.GetOffsetParent(can_gc)
                                },
                                None => None,
                            };
                        }
                        // Step 3
                        Ok(Rect::new(
                            Point2D::new(x as f64, y as f64),
                            Size2D::new(
                                html_element.OffsetWidth(can_gc) as f64,
                                html_element.OffsetHeight(can_gc) as f64,
                            ),
                        ))
                    },
                    None => Err(ErrorStatus::UnknownError),
                }
            }),
        )
        .unwrap();
}

pub(crate) fn handle_get_bounding_client_rect(
    documents: &DocumentCollection,
    pipeline: PipelineId,
    element_id: String,
    reply: IpcSender<Result<Rect<f32>, ErrorStatus>>,
    can_gc: CanGc,
) {
    reply
        .send(
            get_known_element(documents, pipeline, element_id).map(|element| {
                let rect = element.GetBoundingClientRect(can_gc);
                Rect::new(
                    Point2D::new(rect.X() as f32, rect.Y() as f32),
                    Size2D::new(rect.Width() as f32, rect.Height() as f32),
                )
            }),
        )
        .unwrap();
}

/// <https://w3c.github.io/webdriver/#dfn-get-element-text>
pub(crate) fn handle_get_text(
    documents: &DocumentCollection,
    pipeline: PipelineId,
    node_id: String,
    reply: IpcSender<Result<String, ErrorStatus>>,
    can_gc: CanGc,
) {
    reply
        .send(
            get_known_element(documents, pipeline, node_id).map(|element| {
                element
                    .downcast::<HTMLElement>()
                    .map(|htmlelement| htmlelement.InnerText(can_gc).to_string())
                    .unwrap_or_else(|| {
                        element
                            .upcast::<Node>()
                            .GetTextContent()
                            .map_or("".to_owned(), String::from)
                    })
            }),
        )
        .unwrap();
}

pub(crate) fn handle_get_name(
    documents: &DocumentCollection,
    pipeline: PipelineId,
    node_id: String,
    reply: IpcSender<Result<String, ErrorStatus>>,
) {
    reply
        .send(
            get_known_element(documents, pipeline, node_id)
                .map(|element| String::from(element.TagName())),
        )
        .unwrap();
}

pub(crate) fn handle_get_attribute(
    documents: &DocumentCollection,
    pipeline: PipelineId,
    node_id: String,
    name: String,
    reply: IpcSender<Result<Option<String>, ErrorStatus>>,
) {
    reply
        .send(
            get_known_element(documents, pipeline, node_id).map(|element| {
                element
                    .GetAttribute(DOMString::from(name))
                    .map(String::from)
            }),
        )
        .unwrap();
}

#[allow(unsafe_code)]
pub(crate) fn handle_get_property(
    documents: &DocumentCollection,
    pipeline: PipelineId,
    node_id: String,
    name: String,
    reply: IpcSender<Result<WebDriverJSValue, ErrorStatus>>,
    can_gc: CanGc,
) {
    reply
        .send(
            get_known_element(documents, pipeline, node_id).map(|element| {
                let document = documents.find_document(pipeline).unwrap();
                let realm = enter_realm(&*document);
                let cx = document.window().get_cx();

                rooted!(in(*cx) let mut property = UndefinedValue());
                match unsafe {
                    get_property_jsval(
                        *cx,
                        element.reflector().get_jsobject(),
                        &name,
                        property.handle_mut(),
                    )
                } {
                    Ok(_) => {
                        match jsval_to_webdriver(
                            cx,
                            &element.global(),
                            property.handle(),
                            InRealm::entered(&realm),
                            can_gc,
                        ) {
                            Ok(property) => property,
                            Err(_) => WebDriverJSValue::Undefined,
                        }
                    },
                    Err(error) => {
                        throw_dom_exception(cx, &element.global(), error, can_gc);
                        WebDriverJSValue::Undefined
                    },
                }
            }),
        )
        .unwrap();
}

pub(crate) fn handle_get_css(
    documents: &DocumentCollection,
    pipeline: PipelineId,
    node_id: String,
    name: String,
    reply: IpcSender<Result<String, ErrorStatus>>,
    can_gc: CanGc,
) {
    reply
        .send(
            get_known_element(documents, pipeline, node_id).map(|element| {
                let window = element.owner_window();
                String::from(
                    window
                        .GetComputedStyle(&element, None)
                        .GetPropertyValue(DOMString::from(name), can_gc),
                )
            }),
        )
        .unwrap();
}

pub(crate) fn handle_get_url(
    documents: &DocumentCollection,
    pipeline: PipelineId,
    reply: IpcSender<ServoUrl>,
    _can_gc: CanGc,
) {
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

fn get_option_parent(node: &Node) -> Option<DomRoot<Node>> {
    // Get parent for `<option>` or `<optiongrp>` based on container spec:
    // > 1. Let datalist parent be the first datalist element reached by traversing the tree
    // >    in reverse order from element, or undefined if the root of the tree is reached.
    // > 2. Let select parent be the first select element reached by traversing the tree in
    // >    reverse order from element, or undefined if the root of the tree is reached.
    // > 3. If datalist parent is undefined, the element context is select parent.
    // >    Otherwise, the element context is datalist parent.
    let root_node = node.GetRootNode(&GetRootNodeOptions::empty());
    node.preceding_nodes(&root_node)
        .find(|preceding| preceding.is::<HTMLDataListElement>())
        .or_else(|| {
            node.preceding_nodes(&root_node)
                .find(|preceding| preceding.is::<HTMLSelectElement>())
        })
}

// https://w3c.github.io/webdriver/#dfn-container
fn get_container(node: &Node) -> Option<DomRoot<Node>> {
    if node.is::<HTMLOptionElement>() {
        return get_option_parent(node);
    }
    if node.is::<HTMLOptGroupElement>() {
        let option_parent = get_option_parent(node);
        return option_parent.or_else(|| Some(DomRoot::from_ref(node)));
    }
    Some(DomRoot::from_ref(node))
}

// https://w3c.github.io/webdriver/#element-click
pub(crate) fn handle_element_click(
    documents: &DocumentCollection,
    pipeline: PipelineId,
    element_id: String,
    reply: IpcSender<Result<Option<String>, ErrorStatus>>,
    can_gc: CanGc,
) {
    reply
        .send(
            // Step 3
            get_known_element(documents, pipeline, element_id).and_then(|element| {
                // Step 4
                if let Some(input_element) = element.downcast::<HTMLInputElement>() {
                    if input_element.input_type() == InputType::File {
                        return Err(ErrorStatus::InvalidArgument);
                    }
                }

                let Some(container) = get_container(element.upcast::<Node>()) else {
                    return Err(ErrorStatus::UnknownError);
                };

                // Step 5
                // TODO: scroll into view

                // Step 6
                // TODO: return error if still not in view

                // Step 7
                // TODO: return error if obscured

                // Step 8
                match element.downcast::<HTMLOptionElement>() {
                    Some(option_element) => {
                        // Steps 8.2 - 8.4
                        let event_target = container.upcast::<EventTarget>();
                        event_target.fire_event(atom!("mouseover"), can_gc);
                        event_target.fire_event(atom!("mousemove"), can_gc);
                        event_target.fire_event(atom!("mousedown"), can_gc);

                        // Step 8.5
                        match container.downcast::<HTMLElement>() {
                            Some(html_element) => html_element.Focus(can_gc),
                            None => return Err(ErrorStatus::UnknownError),
                        }

                        // Step 8.6
                        if !option_element.Disabled() {
                            // Step 8.6.1
                            event_target.fire_event(atom!("input"), can_gc);

                            // Steps 8.6.2
                            let previous_selectedness = option_element.Selected();

                            // Step 8.6.3
                            match container.downcast::<HTMLSelectElement>() {
                                Some(select_element) => {
                                    if select_element.Multiple() {
                                        option_element.SetSelected(!option_element.Selected());
                                    }
                                },
                                None => option_element.SetSelected(true),
                            }

                            // Step 8.6.4
                            if !previous_selectedness {
                                event_target.fire_event(atom!("change"), can_gc);
                            }
                        }

                        // Steps 8.7 - 8.8
                        event_target.fire_event(atom!("mouseup"), can_gc);
                        event_target.fire_event(atom!("click"), can_gc);

                        Ok(None)
                    },
                    None => Ok(Some(element.upcast::<Node>().unique_id(pipeline))),
                }
            }),
        )
        .unwrap();
}

pub(crate) fn handle_is_enabled(
    documents: &DocumentCollection,
    pipeline: PipelineId,
    element_id: String,
    reply: IpcSender<Result<bool, ErrorStatus>>,
) {
    reply
        .send(
            get_known_element(documents, pipeline, element_id)
                .map(|element| element.enabled_state()),
        )
        .unwrap();
}

pub(crate) fn handle_is_selected(
    documents: &DocumentCollection,
    pipeline: PipelineId,
    element_id: String,
    reply: IpcSender<Result<bool, ErrorStatus>>,
) {
    reply
        .send(
            get_known_element(documents, pipeline, element_id).and_then(|element| {
                if let Some(input_element) = element.downcast::<HTMLInputElement>() {
                    Ok(input_element.Checked())
                } else if let Some(option_element) = element.downcast::<HTMLOptionElement>() {
                    Ok(option_element.Selected())
                } else if element.is::<HTMLElement>() {
                    Ok(false) // regular elements are not selectable
                } else {
                    Err(ErrorStatus::UnknownError)
                }
            }),
        )
        .unwrap();
}
