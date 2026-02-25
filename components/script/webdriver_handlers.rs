/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::{HashMap, HashSet};
use std::ffi::CString;
use std::ptr::NonNull;

use base::generic_channel::{GenericOneshotSender, GenericSend, GenericSender};
use base::id::{BrowsingContextId, PipelineId};
use cookie::Cookie;
use embedder_traits::{
    CustomHandlersAutomationMode, JSValue, JavaScriptEvaluationError,
    JavaScriptEvaluationResultSerializationError, WebDriverFrameId, WebDriverJSResult,
    WebDriverLoadStatus,
};
use euclid::default::{Point2D, Rect, Size2D};
use hyper_serde::Serde;
use ipc_channel::ipc::{self};
use js::context::JSContext;
use js::conversions::jsstr_to_string;
use js::jsapi::{
    self, GetPropertyKeys, HandleValueArray, JS_GetOwnPropertyDescriptorById, JS_GetPropertyById,
    JS_IsExceptionPending, JSAutoRealm, JSObject, JSType, PropertyDescriptor,
};
use js::jsval::UndefinedValue;
use js::realm::CurrentRealm;
use js::rust::wrappers::{JS_CallFunctionName, JS_GetProperty, JS_HasOwnProperty, JS_TypeOfValue};
use js::rust::{Handle, HandleObject, HandleValue, IdVector, ToString};
use net_traits::CookieSource::{HTTP, NonHTTP};
use net_traits::CoreResourceMsg::{
    DeleteCookie, DeleteCookies, GetCookiesDataForUrl, SetCookieForUrl,
};
use script_bindings::codegen::GenericBindings::ShadowRootBinding::ShadowRootMethods;
use script_bindings::conversions::is_array_like;
use script_bindings::num::Finite;
use script_bindings::settings_stack::run_a_script;
use webdriver::error::ErrorStatus;

use crate::DomTypeHolder;
use crate::document_collection::DocumentCollection;
use crate::dom::attr::is_boolean_attribute;
use crate::dom::bindings::codegen::Bindings::CSSStyleDeclarationBinding::CSSStyleDeclarationMethods;
use crate::dom::bindings::codegen::Bindings::DOMRectBinding::DOMRectMethods;
use crate::dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use crate::dom::bindings::codegen::Bindings::ElementBinding::{
    ElementMethods, ScrollIntoViewOptions, ScrollLogicalPosition,
};
use crate::dom::bindings::codegen::Bindings::HTMLElementBinding::HTMLElementMethods;
use crate::dom::bindings::codegen::Bindings::HTMLInputElementBinding::HTMLInputElementMethods;
use crate::dom::bindings::codegen::Bindings::HTMLOptionElementBinding::HTMLOptionElementMethods;
use crate::dom::bindings::codegen::Bindings::HTMLOrSVGElementBinding::FocusOptions;
use crate::dom::bindings::codegen::Bindings::HTMLSelectElementBinding::HTMLSelectElementMethods;
use crate::dom::bindings::codegen::Bindings::HTMLTextAreaElementBinding::HTMLTextAreaElementMethods;
use crate::dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use crate::dom::bindings::codegen::Bindings::WindowBinding::{
    ScrollBehavior, ScrollOptions, WindowMethods,
};
use crate::dom::bindings::codegen::Bindings::XMLSerializerBinding::XMLSerializerMethods;
use crate::dom::bindings::codegen::Bindings::XPathResultBinding::{
    XPathResultConstants, XPathResultMethods,
};
use crate::dom::bindings::codegen::UnionTypes::BooleanOrScrollIntoViewOptions;
use crate::dom::bindings::conversions::{
    ConversionBehavior, ConversionResult, FromJSValConvertible, get_property, get_property_jsval,
    jsid_to_string, root_from_object,
};
use crate::dom::bindings::error::{Error, report_pending_exception, throw_dom_exception};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::{DomGlobal, DomObject};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::document::Document;
use crate::dom::domrect::DOMRect;
use crate::dom::element::Element;
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::html::htmlbodyelement::HTMLBodyElement;
use crate::dom::html::htmldatalistelement::HTMLDataListElement;
use crate::dom::html::htmlelement::HTMLElement;
use crate::dom::html::htmlformelement::FormControl;
use crate::dom::html::htmliframeelement::HTMLIFrameElement;
use crate::dom::html::htmlinputelement::{HTMLInputElement, InputType};
use crate::dom::html::htmloptgroupelement::HTMLOptGroupElement;
use crate::dom::html::htmloptionelement::HTMLOptionElement;
use crate::dom::html::htmlselectelement::HTMLSelectElement;
use crate::dom::html::htmltextareaelement::HTMLTextAreaElement;
use crate::dom::node::{Node, NodeTraits, ShadowIncluding};
use crate::dom::nodelist::NodeList;
use crate::dom::types::ShadowRoot;
use crate::dom::validitystate::ValidationFlags;
use crate::dom::window::Window;
use crate::dom::xmlserializer::XMLSerializer;
use crate::realms::{InRealm, enter_auto_realm, enter_realm};
use crate::script_runtime::{CanGc, JSContext as SafeJSContext};
use crate::script_thread::ScriptThread;

/// <https://w3c.github.io/webdriver/#dfn-is-stale>
fn is_stale(element: &Element) -> bool {
    // An element is stale if its node document is not the active document
    // or if it is not connected.
    !element.owner_document().is_active() || !element.is_connected()
}

/// <https://w3c.github.io/webdriver/#dfn-is-detached>
fn is_detached(shadow_root: &ShadowRoot) -> bool {
    // A shadow root is detached if its node document is not the active document
    // or if the element node referred to as its host is stale.
    !shadow_root.owner_document().is_active() || is_stale(&shadow_root.Host())
}

/// <https://w3c.github.io/webdriver/#dfn-disabled>
fn is_disabled(element: &Element) -> bool {
    // Step 1. If element is an option element or element is an optgroup element
    if element.is::<HTMLOptionElement>() || element.is::<HTMLOptGroupElement>() {
        // Step 1.1. For each inclusive ancestor `ancestor` of element
        let disabled = element
            .upcast::<Node>()
            .inclusive_ancestors(ShadowIncluding::No)
            .any(|node| {
                if node.is::<HTMLOptGroupElement>() || node.is::<HTMLSelectElement>() {
                    // Step 1.1.1. If `ancestor` is an optgroup element or `ancestor` is a select element,
                    // and `ancestor` is actually disabled, return true.
                    node.downcast::<Element>().unwrap().is_actually_disabled()
                } else {
                    false
                }
            });

        // Step 1.2
        // The spec suggests that we immediately return false if the above is not true.
        // However, it causes disabled option element to not be considered as disabled.
        // Hence, here we also check if the element itself is actually disabled.
        if disabled {
            return true;
        }
    }
    // Step 2. Return element is actually disabled.
    element.is_actually_disabled()
}

pub(crate) fn handle_get_known_window(
    documents: &DocumentCollection,
    pipeline: PipelineId,
    webview_id: String,
    reply: GenericSender<Result<(), ErrorStatus>>,
) {
    if reply
        .send(
            documents
                .find_window(pipeline)
                .map_or(Err(ErrorStatus::NoSuchWindow), |window| {
                    let window_proxy = window.window_proxy();
                    // Step 3-4: Window must be top level browsing context.
                    if window_proxy.browsing_context_id() != window_proxy.webview_id() ||
                        window_proxy.webview_id().to_string() != webview_id
                    {
                        Err(ErrorStatus::NoSuchWindow)
                    } else {
                        Ok(())
                    }
                }),
        )
        .is_err()
    {
        error!("Webdriver get known window reply failed");
    }
}

pub(crate) fn handle_get_known_shadow_root(
    documents: &DocumentCollection,
    pipeline: PipelineId,
    shadow_root_id: String,
    reply: GenericSender<Result<(), ErrorStatus>>,
) {
    let result = get_known_shadow_root(documents, pipeline, shadow_root_id).map(|_| ());
    if reply.send(result).is_err() {
        error!("Webdriver get known shadow root reply failed");
    }
}

/// <https://w3c.github.io/webdriver/#dfn-get-a-known-shadow-root>
fn get_known_shadow_root(
    documents: &DocumentCollection,
    pipeline: PipelineId,
    node_id: String,
) -> Result<DomRoot<ShadowRoot>, ErrorStatus> {
    let doc = documents
        .find_document(pipeline)
        .ok_or(ErrorStatus::NoSuchWindow)?;
    // Step 1. If not node reference is known with session, session's current browsing context,
    // and reference return error with error code no such shadow root.
    if !ScriptThread::has_node_id(pipeline, &node_id) {
        return Err(ErrorStatus::NoSuchShadowRoot);
    }

    // Step 2. Let node be the result of get a node with session,
    // session's current browsing context, and reference.
    let node = find_node_by_unique_id_in_document(&doc, node_id);

    // Step 3. If node is not null and node does not implement ShadowRoot
    // return error with error code no such shadow root.
    if let Some(ref node) = node {
        if !node.is::<ShadowRoot>() {
            return Err(ErrorStatus::NoSuchShadowRoot);
        }
    }

    // Step 4.1. If node is null return error with error code detached shadow root.
    let Some(node) = node else {
        return Err(ErrorStatus::DetachedShadowRoot);
    };

    // Step 4.2. If node is detached return error with error code detached shadow root.
    // A shadow root is detached if its node document is not the active document
    // or if the element node referred to as its host is stale.
    let shadow_root = DomRoot::downcast::<ShadowRoot>(node).unwrap();
    if is_detached(&shadow_root) {
        return Err(ErrorStatus::DetachedShadowRoot);
    }
    // Step 5. Return success with data node.
    Ok(shadow_root)
}

pub(crate) fn handle_get_known_element(
    documents: &DocumentCollection,
    pipeline: PipelineId,
    element_id: String,
    reply: GenericSender<Result<(), ErrorStatus>>,
) {
    let result = get_known_element(documents, pipeline, element_id).map(|_| ());
    if reply.send(result).is_err() {
        error!("Webdriver get known element reply failed");
    }
}

/// <https://w3c.github.io/webdriver/#dfn-get-a-known-element>
fn get_known_element(
    documents: &DocumentCollection,
    pipeline: PipelineId,
    node_id: String,
) -> Result<DomRoot<Element>, ErrorStatus> {
    let doc = documents
        .find_document(pipeline)
        .ok_or(ErrorStatus::NoSuchWindow)?;
    // Step 1. If not node reference is known with session, session's current browsing context,
    // and reference return error with error code no such element.
    if !ScriptThread::has_node_id(pipeline, &node_id) {
        return Err(ErrorStatus::NoSuchElement);
    }
    // Step 2.Let node be the result of get a node with session,
    // session's current browsing context, and reference.
    let node = find_node_by_unique_id_in_document(&doc, node_id);

    // Step 3. If node is not null and node does not implement Element
    // return error with error code no such element.
    if let Some(ref node) = node {
        if !node.is::<Element>() {
            return Err(ErrorStatus::NoSuchElement);
        }
    }
    // Step 4.1. If node is null return error with error code stale element reference.
    let Some(node) = node else {
        return Err(ErrorStatus::StaleElementReference);
    };
    // Step 4.2. If node is stale return error with error code stale element reference.
    let element = DomRoot::downcast::<Element>(node).unwrap();
    if is_stale(&element) {
        return Err(ErrorStatus::StaleElementReference);
    }
    // Step 5. Return success with data node.
    Ok(element)
}

// This is also used by `dom/window.rs`
pub(crate) fn find_node_by_unique_id_in_document(
    document: &Document,
    node_id: String,
) -> Option<DomRoot<Node>> {
    let pipeline = document.window().pipeline_id();
    document
        .upcast::<Node>()
        .traverse_preorder(ShadowIncluding::Yes)
        .find(|node| node.unique_id(pipeline) == node_id)
}

/// <https://w3c.github.io/webdriver/#dfn-link-text-selector>
fn matching_links(
    links: &NodeList,
    link_text: String,
    partial: bool,
) -> impl Iterator<Item = String> + '_ {
    links
        .iter()
        .filter(move |node| {
            let content = node
                .downcast::<HTMLElement>()
                .map(|element| element.InnerText())
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
) -> Result<Vec<String>, ErrorStatus> {
    // <https://w3c.github.io/webdriver/#dfn-find>
    // Step 7.2. If a DOMException, SyntaxError, XPathException, or other error occurs
    // during the execution of the element location strategy, return error invalid selector.
    root_node
        .query_selector_all(DOMString::from("a"))
        .map_err(|_| ErrorStatus::InvalidSelector)
        .map(|nodes| matching_links(&nodes, link_text, partial).collect())
}

#[expect(unsafe_code)]
fn object_has_to_json_property(
    cx: SafeJSContext,
    global_scope: &GlobalScope,
    object: HandleObject,
) -> bool {
    let name = CString::new("toJSON").unwrap();
    let mut found = false;
    if unsafe { JS_HasOwnProperty(*cx, object, name.as_ptr(), &mut found) } && found {
        rooted!(in(*cx) let mut value = UndefinedValue());
        let result = unsafe { JS_GetProperty(*cx, object, name.as_ptr(), value.handle_mut()) };
        if !result {
            throw_dom_exception(cx, global_scope, Error::JSFailed, CanGc::note());
            false
        } else {
            result && unsafe { JS_TypeOfValue(*cx, value.handle()) } == JSType::JSTYPE_FUNCTION
        }
    } else if unsafe { JS_IsExceptionPending(*cx) } {
        throw_dom_exception(cx, global_scope, Error::JSFailed, CanGc::note());
        false
    } else {
        false
    }
}

#[expect(unsafe_code)]
/// <https://w3c.github.io/webdriver/#dfn-collection>
fn is_arguments_object(cx: SafeJSContext, value: HandleValue) -> bool {
    rooted!(in(*cx) let class_name = unsafe { ToString(*cx, value) });
    let Some(class_name) = NonNull::new(class_name.get()) else {
        return false;
    };
    let class_name = unsafe { jsstr_to_string(*cx, class_name) };
    class_name == "[object Arguments]"
}

#[derive(Clone, Eq, Hash, PartialEq)]
struct HashableJSVal(u64);

impl From<HandleValue<'_>> for HashableJSVal {
    fn from(v: HandleValue<'_>) -> HashableJSVal {
        HashableJSVal(v.get().asBits_)
    }
}

/// <https://w3c.github.io/webdriver/#dfn-json-clone>
pub(crate) fn jsval_to_webdriver(
    cx: &mut CurrentRealm,
    global_scope: &GlobalScope,
    val: HandleValue,
) -> WebDriverJSResult {
    run_a_script::<DomTypeHolder, _>(global_scope, || {
        let mut seen = HashSet::new();
        let result = jsval_to_webdriver_inner(cx.into(), global_scope, val, &mut seen);

        let in_realm_proof = cx.into();
        let in_realm = InRealm::Already(&in_realm_proof);

        if result.is_err() {
            report_pending_exception(cx.into(), true, in_realm, CanGc::from_cx(cx));
        }
        result
    })
}

#[expect(unsafe_code)]
/// <https://w3c.github.io/webdriver/#dfn-internal-json-clone>
fn jsval_to_webdriver_inner(
    cx: SafeJSContext,
    global_scope: &GlobalScope,
    val: HandleValue,
    seen: &mut HashSet<HashableJSVal>,
) -> WebDriverJSResult {
    let _ac = enter_realm(global_scope);
    if val.get().is_undefined() {
        Ok(JSValue::Undefined)
    } else if val.get().is_null() {
        Ok(JSValue::Null)
    } else if val.get().is_boolean() {
        Ok(JSValue::Boolean(val.get().to_boolean()))
    } else if val.get().is_number() {
        Ok(JSValue::Number(val.to_number()))
    } else if val.get().is_string() {
        let string = NonNull::new(val.to_string()).expect("Should have a non-Null String");
        let string = unsafe { jsstr_to_string(*cx, string) };
        Ok(JSValue::String(string))
    } else if val.get().is_object() {
        rooted!(in(*cx) let object = match unsafe { FromJSValConvertible::from_jsval(*cx, val, ())}.unwrap() {
            ConversionResult::Success(object) => object,
            _ => unreachable!(),
        });
        let _ac = JSAutoRealm::new(*cx, *object);

        if let Ok(element) = unsafe { root_from_object::<Element>(*object, *cx) } {
            // If the element is stale, return error with error code stale element reference.
            if is_stale(&element) {
                Err(JavaScriptEvaluationError::SerializationError(
                    JavaScriptEvaluationResultSerializationError::StaleElementReference,
                ))
            } else {
                Ok(JSValue::Element(
                    element
                        .upcast::<Node>()
                        .unique_id(element.owner_window().pipeline_id()),
                ))
            }
        } else if let Ok(shadow_root) = unsafe { root_from_object::<ShadowRoot>(*object, *cx) } {
            // If the shadow root is detached, return error with error code detached shadow root.
            if is_detached(&shadow_root) {
                Err(JavaScriptEvaluationError::SerializationError(
                    JavaScriptEvaluationResultSerializationError::DetachedShadowRoot,
                ))
            } else {
                Ok(JSValue::ShadowRoot(
                    shadow_root
                        .upcast::<Node>()
                        .unique_id(shadow_root.owner_window().pipeline_id()),
                ))
            }
        } else if let Ok(window) = unsafe { root_from_object::<Window>(*object, *cx) } {
            let window_proxy = window.window_proxy();
            if window_proxy.is_browsing_context_discarded() {
                Err(JavaScriptEvaluationError::SerializationError(
                    JavaScriptEvaluationResultSerializationError::StaleElementReference,
                ))
            } else if window_proxy.browsing_context_id() == window_proxy.webview_id() {
                Ok(JSValue::Window(window.webview_id().to_string()))
            } else {
                Ok(JSValue::Frame(
                    window_proxy.browsing_context_id().to_string(),
                ))
            }
        } else if object_has_to_json_property(cx, global_scope, object.handle()) {
            let name = CString::new("toJSON").unwrap();
            rooted!(in(*cx) let mut value = UndefinedValue());
            let call_result = unsafe {
                JS_CallFunctionName(
                    *cx,
                    object.handle(),
                    name.as_ptr(),
                    &HandleValueArray::empty(),
                    value.handle_mut(),
                )
            };

            if call_result {
                Ok(jsval_to_webdriver_inner(
                    cx,
                    global_scope,
                    value.handle(),
                    seen,
                )?)
            } else {
                throw_dom_exception(cx, global_scope, Error::JSFailed, CanGc::note());
                Err(JavaScriptEvaluationError::SerializationError(
                    JavaScriptEvaluationResultSerializationError::OtherJavaScriptError,
                ))
            }
        } else {
            clone_an_object(cx, global_scope, val, seen, object.handle())
        }
    } else {
        Err(JavaScriptEvaluationError::SerializationError(
            JavaScriptEvaluationResultSerializationError::UnknownType,
        ))
    }
}

#[expect(unsafe_code)]
/// <https://w3c.github.io/webdriver/#dfn-clone-an-object>
fn clone_an_object(
    cx: SafeJSContext,
    global_scope: &GlobalScope,
    val: HandleValue,
    seen: &mut HashSet<HashableJSVal>,
    object_handle: Handle<'_, *mut JSObject>,
) -> WebDriverJSResult {
    let hashable = val.into();
    // Step 1. If value is in `seen`, return error with error code javascript error.
    if seen.contains(&hashable) {
        return Err(JavaScriptEvaluationError::SerializationError(
            JavaScriptEvaluationResultSerializationError::OtherJavaScriptError,
        ));
    }
    // Step 2. Append value to `seen`.
    seen.insert(hashable.clone());

    let return_val = if unsafe {
        is_array_like::<crate::DomTypeHolder>(*cx, val) || is_arguments_object(cx, val)
    } {
        let mut result: Vec<JSValue> = Vec::new();

        let get_property_result =
            get_property::<u32>(cx, object_handle, c"length", ConversionBehavior::Default);
        let length = match get_property_result {
            Ok(length) => match length {
                Some(length) => length,
                _ => {
                    return Err(JavaScriptEvaluationError::SerializationError(
                        JavaScriptEvaluationResultSerializationError::UnknownType,
                    ));
                },
            },
            Err(error) => {
                throw_dom_exception(cx, global_scope, error, CanGc::note());
                return Err(JavaScriptEvaluationError::SerializationError(
                    JavaScriptEvaluationResultSerializationError::OtherJavaScriptError,
                ));
            },
        };
        // Step 4. For each enumerable property in value, run the following substeps:
        for i in 0..length {
            rooted!(in(*cx) let mut item = UndefinedValue());
            let cname = CString::new(i.to_string()).unwrap();
            let get_property_result =
                get_property_jsval(cx, object_handle, &cname, item.handle_mut());
            match get_property_result {
                Ok(_) => {
                    let conversion_result =
                        jsval_to_webdriver_inner(cx, global_scope, item.handle(), seen);
                    match conversion_result {
                        Ok(converted_item) => result.push(converted_item),
                        err @ Err(_) => return err,
                    }
                },
                Err(error) => {
                    throw_dom_exception(cx, global_scope, error, CanGc::note());
                    return Err(JavaScriptEvaluationError::SerializationError(
                        JavaScriptEvaluationResultSerializationError::OtherJavaScriptError,
                    ));
                },
            }
        }
        Ok(JSValue::Array(result))
    } else {
        let mut result = HashMap::new();

        let mut ids = unsafe { IdVector::new(*cx) };
        let succeeded = unsafe {
            GetPropertyKeys(
                *cx,
                object_handle.into(),
                jsapi::JSITER_OWNONLY,
                ids.handle_mut(),
            )
        };
        if !succeeded {
            return Err(JavaScriptEvaluationError::SerializationError(
                JavaScriptEvaluationResultSerializationError::OtherJavaScriptError,
            ));
        }
        for id in ids.iter() {
            rooted!(in(*cx) let id = *id);
            rooted!(in(*cx) let mut desc = PropertyDescriptor::default());

            let mut is_none = false;
            let succeeded = unsafe {
                JS_GetOwnPropertyDescriptorById(
                    *cx,
                    object_handle.into(),
                    id.handle().into(),
                    desc.handle_mut().into(),
                    &mut is_none,
                )
            };
            if !succeeded {
                return Err(JavaScriptEvaluationError::SerializationError(
                    JavaScriptEvaluationResultSerializationError::OtherJavaScriptError,
                ));
            }

            rooted!(in(*cx) let mut property = UndefinedValue());
            let succeeded = unsafe {
                JS_GetPropertyById(
                    *cx,
                    object_handle.into(),
                    id.handle().into(),
                    property.handle_mut().into(),
                )
            };
            if !succeeded {
                return Err(JavaScriptEvaluationError::SerializationError(
                    JavaScriptEvaluationResultSerializationError::OtherJavaScriptError,
                ));
            }

            if !property.is_undefined() {
                let name = unsafe { jsid_to_string(*cx, id.handle()) };
                let Some(name) = name else {
                    return Err(JavaScriptEvaluationError::SerializationError(
                        JavaScriptEvaluationResultSerializationError::OtherJavaScriptError,
                    ));
                };

                let value = jsval_to_webdriver_inner(cx, global_scope, property.handle(), seen)?;
                result.insert(name.into(), value);
            }
        }
        Ok(JSValue::Object(result))
    };
    // Step 5. Remove the last element of `seen`.
    seen.remove(&hashable);
    // Step 6. Return success with data `result`.
    return_val
}

pub(crate) fn handle_execute_async_script(
    window: Option<DomRoot<Window>>,
    eval: String,
    reply: GenericSender<WebDriverJSResult>,
    cx: &mut JSContext,
) {
    match window {
        Some(window) => {
            let reply_sender = reply.clone();
            window.set_webdriver_script_chan(Some(reply));
            rooted!(&in(cx) let mut rval = UndefinedValue());

            let global_scope = window.as_global_scope();

            let mut realm = enter_auto_realm(cx, global_scope);
            let mut realm = realm.current_realm();
            if let Err(error) = global_scope.evaluate_js_on_global(
                &mut realm,
                eval.into(),
                "",
                None, // No known `introductionType` for JS code from WebDriver
                rval.handle_mut(),
            ) {
                reply_sender.send(Err(error)).unwrap_or_else(|error| {
                    error!("ExecuteAsyncScript Failed to send reply: {error}");
                });
            }
        },
        None => {
            reply
                .send(Err(JavaScriptEvaluationError::DocumentNotFound))
                .unwrap_or_else(|error| {
                    error!("ExecuteAsyncScript Failed to send reply: {error}");
                });
        },
    }
}

/// Get BrowsingContextId for <https://w3c.github.io/webdriver/#switch-to-parent-frame>
pub(crate) fn handle_get_parent_frame_id(
    documents: &DocumentCollection,
    pipeline: PipelineId,
    reply: GenericSender<Result<BrowsingContextId, ErrorStatus>>,
) {
    // Step 2. If session's current parent browsing context is no longer open,
    // return error with error code no such window.
    reply
        .send(
            documents
                .find_window(pipeline)
                .and_then(|window| {
                    window
                        .window_proxy()
                        .parent()
                        .map(|parent| parent.browsing_context_id())
                })
                .ok_or(ErrorStatus::NoSuchWindow),
        )
        .unwrap();
}

/// Get the BrowsingContextId for <https://w3c.github.io/webdriver/#dfn-switch-to-frame>
pub(crate) fn handle_get_browsing_context_id(
    documents: &DocumentCollection,
    pipeline: PipelineId,
    webdriver_frame_id: WebDriverFrameId,
    reply: GenericSender<Result<BrowsingContextId, ErrorStatus>>,
) {
    reply
        .send(match webdriver_frame_id {
            WebDriverFrameId::Short(id) => {
                // Step 5. If id is not a supported property index of window,
                // return error with error code no such frame.
                documents
                    .find_document(pipeline)
                    .ok_or(ErrorStatus::NoSuchWindow)
                    .and_then(|document| {
                        document
                            .iframes()
                            .iter()
                            .nth(id as usize)
                            .and_then(|iframe| iframe.browsing_context_id())
                            .ok_or(ErrorStatus::NoSuchFrame)
                    })
            },
            WebDriverFrameId::Element(element_id) => {
                get_known_element(documents, pipeline, element_id).and_then(|element| {
                    element
                        .downcast::<HTMLIFrameElement>()
                        .and_then(|element| element.browsing_context_id())
                        .ok_or(ErrorStatus::NoSuchFrame)
                })
            },
        })
        .unwrap();
}

/// <https://w3c.github.io/webdriver/#dfn-center-point>
fn get_element_in_view_center_point(element: &Element, can_gc: CanGc) -> Option<Point2D<i64>> {
    let doc = element.owner_document();
    // Step 1: Let rectangle be the first element of the DOMRect sequence
    // returned by calling getClientRects() on element.
    element.GetClientRects(can_gc).first().map(|rectangle| {
        let x = rectangle.X();
        let y = rectangle.Y();
        let width = rectangle.Width();
        let height = rectangle.Height();
        debug!(
            "get_element_in_view_center_point: Element rectangle at \
            (x: {x}, y: {y}, width: {width}, height: {height})",
        );
        let window = doc.window();
        // Steps 2. Let left be max(0, min(x coordinate, x coordinate + width dimension)).
        let left = (x.min(x + width)).max(0.0);
        // Step 3. Let right be min(innerWidth, max(x coordinate, x coordinate + width dimension)).
        let right = f64::min(window.InnerWidth() as f64, x.max(x + width));
        // Step 4. Let top be max(0, min(y coordinate, y coordinate + height dimension)).
        let top = (y.min(y + height)).max(0.0);
        // Step 5. Let bottom be
        // min(innerHeight, max(y coordinate, y coordinate + height dimension)).
        let bottom = f64::min(window.InnerHeight() as f64, y.max(y + height));
        debug!(
            "get_element_in_view_center_point: Computed rectangle is \
            (left: {left}, right: {right}, top: {top}, bottom: {bottom})",
        );
        // Step 6. Let x be floor((left + right) ÷ 2.0).
        let center_x = ((left + right) / 2.0).floor() as i64;
        // Step 7. Let y be floor((top + bottom) ÷ 2.0).
        let center_y = ((top + bottom) / 2.0).floor() as i64;

        debug!(
            "get_element_in_view_center_point: Element center point at ({center_x}, {center_y})",
        );
        // Step 8
        Point2D::new(center_x, center_y)
    })
}

pub(crate) fn handle_get_element_in_view_center_point(
    documents: &DocumentCollection,
    pipeline: PipelineId,
    element_id: String,
    reply: GenericOneshotSender<Result<Option<(i64, i64)>, ErrorStatus>>,
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

pub(crate) fn handle_find_elements_css_selector(
    documents: &DocumentCollection,
    pipeline: PipelineId,
    selector: String,
    reply: GenericSender<Result<Vec<String>, ErrorStatus>>,
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
    reply: GenericSender<Result<Vec<String>, ErrorStatus>>,
) {
    match retrieve_document_and_check_root_existence(documents, pipeline) {
        Ok(document) => reply
            .send(all_matching_links(
                document.upcast::<Node>(),
                selector.clone(),
                partial,
            ))
            .unwrap(),
        Err(error) => reply.send(Err(error)).unwrap(),
    }
}

pub(crate) fn handle_find_elements_tag_name(
    documents: &DocumentCollection,
    pipeline: PipelineId,
    selector: String,
    reply: GenericSender<Result<Vec<String>, ErrorStatus>>,
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

/// <https://w3c.github.io/webdriver/#xpath>
fn find_elements_xpath_strategy(
    document: &Document,
    start_node: &Node,
    selector: String,
    pipeline: PipelineId,
    can_gc: CanGc,
) -> Result<Vec<String>, ErrorStatus> {
    // Step 1. Let evaluateResult be the result of calling evaluate,
    // with arguments selector, start node, null, ORDERED_NODE_SNAPSHOT_TYPE, and null.

    // A snapshot is used to promote operation atomicity.
    let evaluate_result = match document.Evaluate(
        DOMString::from(selector),
        start_node,
        None,
        XPathResultConstants::ORDERED_NODE_SNAPSHOT_TYPE,
        None,
        can_gc,
    ) {
        Ok(res) => res,
        Err(_) => return Err(ErrorStatus::InvalidSelector),
    };
    // Step 2. Let index be 0. (Handled altogether in Step 5.)

    // Step 3: Let length be the result of getting the property "snapshotLength"
    // from evaluateResult.

    let length = match evaluate_result.GetSnapshotLength() {
        Ok(len) => len,
        Err(_) => return Err(ErrorStatus::InvalidSelector),
    };

    // Step 4: Prepare result vector
    let mut result = Vec::new();

    // Step 5: Repeat, while index is less than length:
    for index in 0..length {
        // Step 5.1. Let node be the result of calling snapshotItem with
        // evaluateResult as this and index as the argument.
        let node = match evaluate_result.SnapshotItem(index) {
            Ok(node) => node.expect(
                "Node should always exist as ORDERED_NODE_SNAPSHOT_TYPE \
                                gives static result and we verified the length!",
            ),
            Err(_) => return Err(ErrorStatus::InvalidSelector),
        };

        // Step 5.2. If node is not an element return an error with error code invalid selector.
        if !node.is::<Element>() {
            return Err(ErrorStatus::InvalidSelector);
        }

        // Step 5.3. Append node to result.
        result.push(node.unique_id(pipeline));
    }
    // Step 6. Return success with data result.
    Ok(result)
}

pub(crate) fn handle_find_elements_xpath_selector(
    documents: &DocumentCollection,
    pipeline: PipelineId,
    selector: String,
    reply: GenericSender<Result<Vec<String>, ErrorStatus>>,
    can_gc: CanGc,
) {
    match retrieve_document_and_check_root_existence(documents, pipeline) {
        Ok(document) => reply
            .send(find_elements_xpath_strategy(
                &document,
                document.upcast::<Node>(),
                selector,
                pipeline,
                can_gc,
            ))
            .unwrap(),
        Err(error) => reply.send(Err(error)).unwrap(),
    }
}

pub(crate) fn handle_find_element_elements_css_selector(
    documents: &DocumentCollection,
    pipeline: PipelineId,
    element_id: String,
    selector: String,
    reply: GenericSender<Result<Vec<String>, ErrorStatus>>,
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
    reply: GenericSender<Result<Vec<String>, ErrorStatus>>,
) {
    reply
        .send(
            get_known_element(documents, pipeline, element_id).and_then(|element| {
                all_matching_links(element.upcast::<Node>(), selector.clone(), partial)
            }),
        )
        .unwrap();
}

pub(crate) fn handle_find_element_elements_tag_name(
    documents: &DocumentCollection,
    pipeline: PipelineId,
    element_id: String,
    selector: String,
    reply: GenericSender<Result<Vec<String>, ErrorStatus>>,
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

pub(crate) fn handle_find_element_elements_xpath_selector(
    documents: &DocumentCollection,
    pipeline: PipelineId,
    element_id: String,
    selector: String,
    reply: GenericSender<Result<Vec<String>, ErrorStatus>>,
    can_gc: CanGc,
) {
    reply
        .send(
            get_known_element(documents, pipeline, element_id).and_then(|element| {
                find_elements_xpath_strategy(
                    &documents
                        .find_document(pipeline)
                        .expect("Document existence guaranteed by `get_known_element`"),
                    element.upcast::<Node>(),
                    selector,
                    pipeline,
                    can_gc,
                )
            }),
        )
        .unwrap();
}

/// <https://w3c.github.io/webdriver/#find-elements-from-shadow-root>
pub(crate) fn handle_find_shadow_elements_css_selector(
    documents: &DocumentCollection,
    pipeline: PipelineId,
    shadow_root_id: String,
    selector: String,
    reply: GenericSender<Result<Vec<String>, ErrorStatus>>,
) {
    reply
        .send(
            get_known_shadow_root(documents, pipeline, shadow_root_id).and_then(|shadow_root| {
                shadow_root
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

pub(crate) fn handle_find_shadow_elements_link_text(
    documents: &DocumentCollection,
    pipeline: PipelineId,
    shadow_root_id: String,
    selector: String,
    partial: bool,
    reply: GenericSender<Result<Vec<String>, ErrorStatus>>,
) {
    reply
        .send(
            get_known_shadow_root(documents, pipeline, shadow_root_id).and_then(|shadow_root| {
                all_matching_links(shadow_root.upcast::<Node>(), selector.clone(), partial)
            }),
        )
        .unwrap();
}

pub(crate) fn handle_find_shadow_elements_tag_name(
    documents: &DocumentCollection,
    pipeline: PipelineId,
    shadow_root_id: String,
    selector: String,
    reply: GenericSender<Result<Vec<String>, ErrorStatus>>,
) {
    // According to spec, we should use `getElementsByTagName`. But it is wrong, as only
    // Document and Element implement this method. So we use `querySelectorAll` instead.
    // But we should not return InvalidSelector error if the selector is not valid,
    // as `getElementsByTagName` won't.
    // See https://github.com/w3c/webdriver/issues/1903
    reply
        .send(
            get_known_shadow_root(documents, pipeline, shadow_root_id).map(|shadow_root| {
                shadow_root
                    .upcast::<Node>()
                    .query_selector_all(DOMString::from(selector))
                    .map(|nodes| {
                        nodes
                            .iter()
                            .map(|x| x.upcast::<Node>().unique_id(pipeline))
                            .collect()
                    })
                    .unwrap_or_default()
            }),
        )
        .unwrap();
}

pub(crate) fn handle_find_shadow_elements_xpath_selector(
    documents: &DocumentCollection,
    pipeline: PipelineId,
    shadow_root_id: String,
    selector: String,
    reply: GenericSender<Result<Vec<String>, ErrorStatus>>,
    can_gc: CanGc,
) {
    reply
        .send(
            get_known_shadow_root(documents, pipeline, shadow_root_id).and_then(|shadow_root| {
                find_elements_xpath_strategy(
                    &documents
                        .find_document(pipeline)
                        .expect("Document existence guaranteed by `get_known_shadow_root`"),
                    shadow_root.upcast::<Node>(),
                    selector,
                    pipeline,
                    can_gc,
                )
            }),
        )
        .unwrap();
}

/// <https://www.w3.org/TR/webdriver2/#dfn-get-element-shadow-root>
pub(crate) fn handle_get_element_shadow_root(
    documents: &DocumentCollection,
    pipeline: PipelineId,
    element_id: String,
    reply: GenericSender<Result<Option<String>, ErrorStatus>>,
) {
    reply
        .send(
            get_known_element(documents, pipeline, element_id).map(|element| {
                element
                    .shadow_root()
                    .map(|x| x.upcast::<Node>().unique_id(pipeline))
            }),
        )
        .unwrap();
}

/// <https://w3c.github.io/webdriver/#dfn-keyboard-interactable>
fn is_keyboard_interactable(element: &Element) -> bool {
    element.is_focusable_area() || element.is::<HTMLBodyElement>() || element.is_document_element()
}

fn handle_send_keys_file(
    file_input: &HTMLInputElement,
    text: &str,
    reply_sender: GenericSender<Result<bool, ErrorStatus>>,
) {
    // Step 1. Let files be the result of splitting text
    // on the newline (\n) character.
    //
    // Be sure to also remove empty strings, as "" always splits to a single string.
    let files: Vec<DOMString> = text
        .split("\n")
        .filter_map(|string| {
            if string.is_empty() {
                None
            } else {
                Some(string.into())
            }
        })
        .collect();

    // Step 2. If files is of 0 length, return ErrorStatus::InvalidArgument.
    if files.is_empty() {
        let _ = reply_sender.send(Err(ErrorStatus::InvalidArgument));
        return;
    }

    // Step 3. Let multiple equal the result of calling hasAttribute() with "multiple" on
    // element. Step 4. If multiple is false and the length of files is not equal to 1,
    // return ErrorStatus::InvalidArgument.
    if !file_input.Multiple() && files.len() > 1 {
        let _ = reply_sender.send(Err(ErrorStatus::InvalidArgument));
        return;
    }

    // Step 5. Return ErrorStatus::InvalidArgument if the files does not exist.
    // Step 6. Set the selected files on the input event.
    // TODO: If multiple is true files are be appended to element's selected files.
    // Step 7. Fire input and change event (should already be fired in `htmlinputelement.rs`)
    // Step 8. Return success with data null.
    //
    // Do not reply to the response yet, as we are waiting for the files to arrive
    // asynchronously.
    file_input.select_files_for_webdriver(files, reply_sender);
}

/// We have verify previously that input element is not textual.
fn handle_send_keys_non_typeable(
    input_element: &HTMLInputElement,
    text: &str,
    can_gc: CanGc,
) -> Result<bool, ErrorStatus> {
    // Step 1. If element does not have an own property named value,
    // Return ErrorStatus::ElementNotInteractable.
    // Currently, we only support HTMLInputElement for non-typeable
    // form controls. Hence, it should always have value property.

    // Step 2. If element is not mutable, return ErrorStatus::ElementNotInteractable.
    if !input_element.is_mutable() {
        return Err(ErrorStatus::ElementNotInteractable);
    }

    // Step 3. Set a property value to text on element.
    if let Err(error) = input_element.SetValue(text.into(), can_gc) {
        error!(
            "Failed to set value on non-typeable input element: {:?}",
            error
        );
        return Err(ErrorStatus::UnknownError);
    }

    // Step 4. If element is suffering from bad input, return ErrorStatus::InvalidArgument.
    if input_element
        .Validity(can_gc)
        .invalid_flags()
        .contains(ValidationFlags::BAD_INPUT)
    {
        return Err(ErrorStatus::InvalidArgument);
    }

    // Step 5. Return success with data null.
    // This is done in `webdriver_server:lib.rs`
    Ok(false)
}

/// Implementing step 5 - 7, plus part of step 8 of "Element Send Keys"
/// where element is input element in the file upload state.
/// This function will send a boolean back to webdriver_server,
/// indicating whether the dispatching of the key and
/// composition event is still needed or not.
pub(crate) fn handle_will_send_keys(
    documents: &DocumentCollection,
    pipeline: PipelineId,
    element_id: String,
    text: String,
    strict_file_interactability: bool,
    reply: GenericSender<Result<bool, ErrorStatus>>,
    can_gc: CanGc,
) {
    // Set 5. Let element be the result of trying to get a known element.
    let element = match get_known_element(documents, pipeline, element_id) {
        Ok(element) => element,
        Err(error) => {
            let _ = reply.send(Err(error));
            return;
        },
    };

    let input_element = element.downcast::<HTMLInputElement>();
    let mut element_has_focus = false;

    // Step 6: Let file be true if element is input element
    // in the file upload state, or false otherwise
    let is_file_input = input_element.is_some_and(|e| e.input_type() == InputType::File);

    // Step 7. If file is false or the session's strict file interactability
    if !is_file_input || strict_file_interactability {
        // Step 7.1. Scroll into view the element
        scroll_into_view(&element, documents, &pipeline, can_gc);

        // TODO: Step 7.2 - 7.5
        // Wait until element become keyboard-interactable

        // Step 7.6. If element is not keyboard-interactable,
        // return ErrorStatus::ElementNotInteractable.
        if !is_keyboard_interactable(&element) {
            let _ = reply.send(Err(ErrorStatus::ElementNotInteractable));
            return;
        }

        // Step 7.7. If element is not the active element
        // run the focusing steps for the element.
        let Some(html_element) = element.downcast::<HTMLElement>() else {
            let _ = reply.send(Err(ErrorStatus::UnknownError));
            return;
        };

        if !element.is_active_element() {
            html_element.Focus(
                &FocusOptions {
                    preventScroll: true,
                },
                can_gc,
            );
        } else {
            element_has_focus = element.focus_state();
        }
    }

    if let Some(input_element) = input_element {
        // Step 8 (Handle file upload)
        if is_file_input {
            handle_send_keys_file(input_element, &text, reply);
            return;
        }

        // Step 8 (Handle non-typeable form control)
        if input_element.is_nontypeable() {
            let _ = reply.send(handle_send_keys_non_typeable(input_element, &text, can_gc));
            return;
        }
    }

    // TODO: Check content editable

    // Step 8 (Other type of elements)
    // Step 8.1. If element does not currently have focus,
    // let current text length be the length of element's API value.
    // Step 8.2. Set the text insertion caret using set selection range
    // using current text length for both the start and end parameters.
    if !element_has_focus {
        if let Some(input_element) = input_element {
            let length = input_element.Value().len() as u32;
            let _ = input_element.SetSelectionRange(length, length, None);
        } else if let Some(textarea_element) = element.downcast::<HTMLTextAreaElement>() {
            let length = textarea_element.Value().len() as u32;
            let _ = textarea_element.SetSelectionRange(length, length, None);
        }
    }

    let _ = reply.send(Ok(true));
}

pub(crate) fn handle_get_active_element(
    documents: &DocumentCollection,
    pipeline: PipelineId,
    reply: GenericSender<Option<String>>,
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
    reply: GenericSender<Result<Option<String>, ErrorStatus>>,
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
    reply: GenericSender<Result<String, ErrorStatus>>,
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
    reply: GenericSender<Result<Vec<Serde<Cookie<'static>>>, ErrorStatus>>,
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
    reply: GenericSender<Result<Vec<Serde<Cookie<'static>>>, ErrorStatus>>,
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
    reply: GenericSender<Result<(), ErrorStatus>>,
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
            (false, Some(ref domain)) if url.host_str().is_some_and(|host| host == domain) => {
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
    reply: GenericSender<Result<(), ErrorStatus>>,
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
        .send(DeleteCookies(Some(url), None))
        .unwrap();
    reply.send(Ok(())).unwrap();
}

// https://w3c.github.io/webdriver/#delete-cookie
pub(crate) fn handle_delete_cookie(
    documents: &DocumentCollection,
    pipeline: PipelineId,
    name: String,
    reply: GenericSender<Result<(), ErrorStatus>>,
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
    reply: GenericSender<String>,
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

/// <https://w3c.github.io/webdriver/#dfn-calculate-the-absolute-position>
fn calculate_absolute_position(
    documents: &DocumentCollection,
    pipeline: &PipelineId,
    rect: &DOMRect,
) -> Result<(f64, f64), ErrorStatus> {
    // Step 1
    // We already pass the rectangle here, see `handle_get_rect`.

    // Step 2
    let document = match documents.find_document(*pipeline) {
        Some(document) => document,
        None => return Err(ErrorStatus::UnknownError),
    };
    let win = match document.GetDefaultView() {
        Some(win) => win,
        None => return Err(ErrorStatus::UnknownError),
    };

    // Step 3 - 5
    let x = win.ScrollX() as f64 + rect.X();
    let y = win.ScrollY() as f64 + rect.Y();

    Ok((x, y))
}

/// <https://w3c.github.io/webdriver/#get-element-rect>
pub(crate) fn handle_get_rect(
    documents: &DocumentCollection,
    pipeline: PipelineId,
    element_id: String,
    reply: GenericSender<Result<Rect<f64>, ErrorStatus>>,
    can_gc: CanGc,
) {
    reply
        .send(
            get_known_element(documents, pipeline, element_id).and_then(|element| {
                // Step 4-5
                // We pass the rect instead of element so we don't have to
                // call `GetBoundingClientRect` twice.
                let rect = element.GetBoundingClientRect(can_gc);
                let (x, y) = calculate_absolute_position(documents, &pipeline, &rect)?;

                // Step 6-7
                Ok(Rect::new(
                    Point2D::new(x, y),
                    Size2D::new(rect.Width(), rect.Height()),
                ))
            }),
        )
        .unwrap();
}

pub(crate) fn handle_scroll_and_get_bounding_client_rect(
    documents: &DocumentCollection,
    pipeline: PipelineId,
    element_id: String,
    reply: GenericSender<Result<Rect<f32>, ErrorStatus>>,
    can_gc: CanGc,
) {
    reply
        .send(
            get_known_element(documents, pipeline, element_id).map(|element| {
                scroll_into_view(&element, documents, &pipeline, can_gc);

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
    reply: GenericSender<Result<String, ErrorStatus>>,
) {
    reply
        .send(
            get_known_element(documents, pipeline, node_id).map(|element| {
                element
                    .downcast::<HTMLElement>()
                    .map(|htmlelement| htmlelement.InnerText().to_string())
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
    reply: GenericSender<Result<String, ErrorStatus>>,
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
    reply: GenericSender<Result<Option<String>, ErrorStatus>>,
) {
    reply
        .send(
            get_known_element(documents, pipeline, node_id).map(|element| {
                if is_boolean_attribute(&name) {
                    if element.HasAttribute(DOMString::from(name)) {
                        Some(String::from("true"))
                    } else {
                        None
                    }
                } else {
                    element
                        .GetAttribute(DOMString::from(name))
                        .map(String::from)
                }
            }),
        )
        .unwrap();
}

pub(crate) fn handle_get_property(
    documents: &DocumentCollection,
    pipeline: PipelineId,
    node_id: String,
    name: String,
    reply: GenericSender<Result<JSValue, ErrorStatus>>,
    cx: &mut JSContext,
) {
    reply
        .send(
            get_known_element(documents, pipeline, node_id).map(|element| {
                let document = documents.find_document(pipeline).unwrap();

                let Ok(cname) = CString::new(name) else {
                    return JSValue::Undefined;
                };

                let mut realm = enter_auto_realm(cx, &*document);
                let cx = &mut realm.current_realm();

                rooted!(&in(cx) let mut property = UndefinedValue());
                match get_property_jsval(
                    cx.into(),
                    element.reflector().get_jsobject(),
                    &cname,
                    property.handle_mut(),
                ) {
                    Ok(_) => match jsval_to_webdriver(cx, &element.global(), property.handle()) {
                        Ok(property) => property,
                        Err(_) => JSValue::Undefined,
                    },
                    Err(error) => {
                        throw_dom_exception(
                            cx.into(),
                            &element.global(),
                            error,
                            CanGc::from_cx(cx),
                        );
                        JSValue::Undefined
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
    reply: GenericSender<Result<String, ErrorStatus>>,
) {
    reply
        .send(
            get_known_element(documents, pipeline, node_id).map(|element| {
                let window = element.owner_window();
                String::from(
                    window
                        .GetComputedStyle(&element, None)
                        .GetPropertyValue(DOMString::from(name)),
                )
            }),
        )
        .unwrap();
}

pub(crate) fn handle_get_url(
    documents: &DocumentCollection,
    pipeline: PipelineId,
    reply: GenericSender<String>,
    _can_gc: CanGc,
) {
    reply
        .send(
            // TODO: Return an error if the pipeline doesn't exist.
            documents
                .find_document(pipeline)
                .map(|document| document.url().into_string())
                .unwrap_or_else(|| "about:blank".to_string()),
        )
        .unwrap();
}

/// <https://w3c.github.io/webdriver/#dfn-mutable-form-control-element>
fn element_is_mutable_form_control(element: &Element) -> bool {
    if let Some(input_element) = element.downcast::<HTMLInputElement>() {
        input_element.is_mutable() &&
            matches!(
                input_element.input_type(),
                InputType::Text |
                    InputType::Search |
                    InputType::Url |
                    InputType::Tel |
                    InputType::Email |
                    InputType::Password |
                    InputType::Date |
                    InputType::Month |
                    InputType::Week |
                    InputType::Time |
                    InputType::DatetimeLocal |
                    InputType::Number |
                    InputType::Range |
                    InputType::Color |
                    InputType::File
            )
    } else if let Some(textarea_element) = element.downcast::<HTMLTextAreaElement>() {
        textarea_element.is_mutable()
    } else {
        false
    }
}

/// <https://w3c.github.io/webdriver/#dfn-clear-a-resettable-element>
fn clear_a_resettable_element(element: &Element, can_gc: CanGc) -> Result<(), ErrorStatus> {
    let html_element = element
        .downcast::<HTMLElement>()
        .ok_or(ErrorStatus::UnknownError)?;

    // Step 1 - 2. if element is a candidate for constraint
    // validation and value is empty, abort steps.
    if html_element.is_candidate_for_constraint_validation() {
        if let Some(input_element) = element.downcast::<HTMLInputElement>() {
            if input_element.Value().is_empty() {
                return Ok(());
            }
        } else if let Some(textarea_element) = element.downcast::<HTMLTextAreaElement>() {
            if textarea_element.Value().is_empty() {
                return Ok(());
            }
        }
    }

    // Step 3. Invoke the focusing steps for the element.
    html_element.Focus(
        &FocusOptions {
            preventScroll: true,
        },
        can_gc,
    );

    // Step 4. Run clear algorithm for element.
    if let Some(input_element) = element.downcast::<HTMLInputElement>() {
        input_element.clear(can_gc);
    } else if let Some(textarea_element) = element.downcast::<HTMLTextAreaElement>() {
        textarea_element.clear();
    } else {
        unreachable!("We have confirm previously that element is mutable form control");
    }

    let event_target = element.upcast::<EventTarget>();
    event_target.fire_bubbling_event(atom!("input"), can_gc);
    event_target.fire_bubbling_event(atom!("change"), can_gc);

    // Step 5. Run the unfocusing steps for the element.
    html_element.Blur(can_gc);

    Ok(())
}

/// <https://w3c.github.io/webdriver/#element-clear>
pub(crate) fn handle_element_clear(
    documents: &DocumentCollection,
    pipeline: PipelineId,
    element_id: String,
    reply: GenericSender<Result<(), ErrorStatus>>,
    can_gc: CanGc,
) {
    reply
        .send(
            get_known_element(documents, pipeline, element_id).and_then(|element| {
                // Step 4. If element is not editable, return ErrorStatus::InvalidElementState.
                // TODO: editing hosts and content editable elements are not implemented yet,
                // hence we currently skip the check
                if !element_is_mutable_form_control(&element) {
                    return Err(ErrorStatus::InvalidElementState);
                }

                // Step 5. Scroll Into View
                scroll_into_view(&element, documents, &pipeline, can_gc);

                // TODO: Step 6 - 9: Implicit wait. In another PR.
                // Wait until element become interactable and check.

                // Step 10. If element is not keyboard-interactable or not pointer-interactable,
                // return error with error code element not interactable.
                if !is_keyboard_interactable(&element) {
                    return Err(ErrorStatus::ElementNotInteractable);
                }

                let paint_tree = get_element_pointer_interactable_paint_tree(
                    &element,
                    &documents
                        .find_document(pipeline)
                        .expect("Document existence guaranteed by `get_known_element`"),
                    can_gc,
                );
                if !is_element_in_view(&element, &paint_tree) {
                    return Err(ErrorStatus::ElementNotInteractable);
                }

                // Step 11
                // TODO: Clear content editable elements
                clear_a_resettable_element(&element, can_gc)
            }),
        )
        .unwrap();
}

fn get_option_parent(node: &Node) -> Option<DomRoot<Element>> {
    // Get parent for `<option>` or `<optiongrp>` based on container spec:
    // > 1. Let datalist parent be the first datalist element reached by traversing the tree
    // >    in reverse order from element, or undefined if the root of the tree is reached.
    // > 2. Let select parent be the first select element reached by traversing the tree in
    // >    reverse order from element, or undefined if the root of the tree is reached.
    // > 3. If datalist parent is undefined, the element context is select parent.
    // >    Otherwise, the element context is datalist parent.
    let mut candidate_select = None;

    for ancestor in node.ancestors() {
        if ancestor.is::<HTMLDataListElement>() {
            return Some(DomRoot::downcast::<Element>(ancestor).unwrap());
        } else if candidate_select.is_none() && ancestor.is::<HTMLSelectElement>() {
            candidate_select = Some(ancestor);
        }
    }

    candidate_select.map(|ancestor| DomRoot::downcast::<Element>(ancestor).unwrap())
}

/// <https://w3c.github.io/webdriver/#dfn-container>
fn get_container(element: &Element) -> Option<DomRoot<Element>> {
    if element.is::<HTMLOptionElement>() {
        return get_option_parent(element.upcast::<Node>());
    }
    if element.is::<HTMLOptGroupElement>() {
        return get_option_parent(element.upcast::<Node>())
            .or_else(|| Some(DomRoot::from_ref(element)));
    }
    Some(DomRoot::from_ref(element))
}

// https://w3c.github.io/webdriver/#element-click
pub(crate) fn handle_element_click(
    documents: &DocumentCollection,
    pipeline: PipelineId,
    element_id: String,
    reply: GenericSender<Result<Option<String>, ErrorStatus>>,
    can_gc: CanGc,
) {
    reply
        .send(
            // Step 3
            get_known_element(documents, pipeline, element_id).and_then(|element| {
                // Step 4. If the element is an input element in the file upload state
                // return error with error code invalid argument.
                if let Some(input_element) = element.downcast::<HTMLInputElement>() {
                    if input_element.input_type() == InputType::File {
                        return Err(ErrorStatus::InvalidArgument);
                    }
                }

                let Some(container) = get_container(&element) else {
                    return Err(ErrorStatus::UnknownError);
                };

                // Step 5. Scroll into view the element's container.
                scroll_into_view(&container, documents, &pipeline, can_gc);

                // Step 6. If element's container is still not in view
                // return error with error code element not interactable.
                let paint_tree = get_element_pointer_interactable_paint_tree(
                    &container,
                    &documents
                        .find_document(pipeline)
                        .expect("Document existence guaranteed by `get_known_element`"),
                    can_gc,
                );

                if !is_element_in_view(&container, &paint_tree) {
                    return Err(ErrorStatus::ElementNotInteractable);
                }

                // Step 7. If element's container is obscured by another element,
                // return error with error code element click intercepted.
                // https://w3c.github.io/webdriver/#dfn-obscuring
                // An element is obscured if the pointer-interactable paint tree is empty,
                // or the first element in this tree is not an inclusive descendant of itself.
                // `paint_tree` is guaranteed not empty as element is "in view".
                if !container
                    .upcast::<Node>()
                    .is_shadow_including_inclusive_ancestor_of(paint_tree[0].upcast::<Node>())
                {
                    return Err(ErrorStatus::ElementClickIntercepted);
                }

                // Step 8 for <option> element.
                match element.downcast::<HTMLOptionElement>() {
                    Some(option_element) => {
                        // Steps 8.2 - 8.4
                        let event_target = container.upcast::<EventTarget>();
                        event_target.fire_event(atom!("mouseover"), can_gc);
                        event_target.fire_event(atom!("mousemove"), can_gc);
                        event_target.fire_event(atom!("mousedown"), can_gc);

                        // Step 8.5
                        match container.downcast::<HTMLElement>() {
                            Some(html_element) => {
                                html_element.Focus(
                                    &FocusOptions {
                                        preventScroll: true,
                                    },
                                    can_gc,
                                );
                            },
                            None => return Err(ErrorStatus::UnknownError),
                        }

                        // Step 8.6
                        if !is_disabled(&element) {
                            // Step 8.6.1
                            event_target.fire_event(atom!("input"), can_gc);

                            // Steps 8.6.2
                            let previous_selectedness = option_element.Selected();

                            // Step 8.6.3
                            match container.downcast::<HTMLSelectElement>() {
                                Some(select_element) => {
                                    if select_element.Multiple() {
                                        option_element
                                            .SetSelected(!option_element.Selected(), can_gc);
                                    }
                                },
                                None => option_element.SetSelected(true, can_gc),
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

/// <https://w3c.github.io/webdriver/#dfn-in-view>
fn is_element_in_view(element: &Element, paint_tree: &[DomRoot<Element>]) -> bool {
    // An element is in view if it is a member of its own pointer-interactable paint tree,
    // given the pretense that its pointer events are not disabled.
    if !paint_tree.contains(&DomRoot::from_ref(element)) {
        return false;
    }
    use style::computed_values::pointer_events::T as PointerEvents;
    // https://w3c.github.io/webdriver/#dfn-pointer-events-are-not-disabled
    // An element is said to have pointer events disabled
    // if the resolved value of its "pointer-events" style property is "none".
    element
        .style()
        .is_none_or(|style| style.get_inherited_ui().pointer_events != PointerEvents::None)
}

/// <https://w3c.github.io/webdriver/#dfn-pointer-interactable-paint-tree>
fn get_element_pointer_interactable_paint_tree(
    element: &Element,
    document: &Document,
    can_gc: CanGc,
) -> Vec<DomRoot<Element>> {
    // Step 1. If element is not in the same tree as session's
    // current browsing context's active document, return an empty sequence.
    if !element.is_connected() {
        return Vec::new();
    }

    // Step 2 - 5: Return "elements from point" w.r.t. in-view center point of element.
    // Spec has bugs in description and can be simplified.
    // The original step 4 "compute in-view center point" takes an element as argument
    // which internally computes first DOMRect of getClientRects

    get_element_in_view_center_point(element, can_gc).map_or(Vec::new(), |center_point| {
        document.ElementsFromPoint(
            Finite::wrap(center_point.x as f64),
            Finite::wrap(center_point.y as f64),
        )
    })
}

/// <https://w3c.github.io/webdriver/#is-element-enabled>
pub(crate) fn handle_is_enabled(
    documents: &DocumentCollection,
    pipeline: PipelineId,
    element_id: String,
    reply: GenericSender<Result<bool, ErrorStatus>>,
) {
    reply
        .send(
            // Step 3. Let element be the result of trying to get a known element
            get_known_element(documents, pipeline, element_id).map(|element| {
                // In `get_known_element`, we confirmed that document exists
                let document = documents.find_document(pipeline).unwrap();

                // Step 4
                // Let enabled be a boolean initially set to true if session's
                // current browsing context's active document's type is not "xml".
                // Otherwise, let enabled to false and jump to the last step of this algorithm.
                // Step 5. Set enabled to false if a form control is disabled.
                if document.is_html_document() || document.is_xhtml_document() {
                    !is_disabled(&element)
                } else {
                    false
                }
            }),
        )
        .unwrap();
}

pub(crate) fn handle_is_selected(
    documents: &DocumentCollection,
    pipeline: PipelineId,
    element_id: String,
    reply: GenericSender<Result<bool, ErrorStatus>>,
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

pub(crate) fn handle_add_load_status_sender(
    documents: &DocumentCollection,
    pipeline: PipelineId,
    reply: GenericSender<WebDriverLoadStatus>,
) {
    if let Some(document) = documents.find_document(pipeline) {
        let window = document.window();
        window.set_webdriver_load_status_sender(Some(reply));
    }
}

pub(crate) fn handle_remove_load_status_sender(
    documents: &DocumentCollection,
    pipeline: PipelineId,
) {
    if let Some(document) = documents.find_document(pipeline) {
        let window = document.window();
        window.set_webdriver_load_status_sender(None);
    }
}

/// <https://w3c.github.io/webdriver/#dfn-scrolls-into-view>
fn scroll_into_view(
    element: &Element,
    documents: &DocumentCollection,
    pipeline: &PipelineId,
    can_gc: CanGc,
) {
    // Check if element is already in view
    let paint_tree = get_element_pointer_interactable_paint_tree(
        element,
        &documents
            .find_document(*pipeline)
            .expect("Document existence guaranteed by `get_known_element`"),
        can_gc,
    );
    if is_element_in_view(element, &paint_tree) {
        return;
    }

    // Step 1. Let options be the following ScrollIntoViewOptions:
    // - "behavior": instant
    // - Logical scroll position "block": end
    // - Logical scroll position "inline": nearest
    let options = BooleanOrScrollIntoViewOptions::ScrollIntoViewOptions(ScrollIntoViewOptions {
        parent: ScrollOptions {
            behavior: ScrollBehavior::Instant,
        },
        block: ScrollLogicalPosition::End,
        inline: ScrollLogicalPosition::Nearest,
        container: Default::default(),
    });
    // Step 2. Run scrollIntoView
    element.ScrollIntoView(options);
}

pub(crate) fn set_protocol_handler_automation_mode(
    documents: &DocumentCollection,
    pipeline: PipelineId,
    mode: CustomHandlersAutomationMode,
) {
    if let Some(document) = documents.find_document(pipeline) {
        document.set_protocol_handler_automation_mode(mode);
    }
}
