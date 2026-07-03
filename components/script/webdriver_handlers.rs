/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::ffi::{CStr, CString};
use std::ops::ControlFlow;
use std::ptr::{self, NonNull, null_mut};
use std::rc::Rc;

use cookie::Cookie;
use embedder_traits::{
    CustomHandlersAutomationMode, JSValue, JavaScriptEvaluationError,
    JavaScriptEvaluationResultSerializationError, WebDriverFrameId, WebDriverJSResult,
    WebDriverLoadStatus,
};
use euclid::default::{Point2D, Rect, Size2D};
use hyper_serde::Serde;
use js::context::JSContext;
use js::conversions::{FromJSValConvertible, jsstr_to_string};
use js::error::throw_type_error;
use js::gc::{MutableHandleValue, RootedVec};
use js::glue::IsProxyHandlerFamily;
use js::jsapi::{
    CallArgs, ESClass, GetFunctionNativeReserved, HandleValueArray, IdentifyStandardPrototype,
    IsArrayBufferObject, IsCallable, IsWeakMapObject, IsWindowProxy, JS_GetFunctionObject,
    JS_IsTypedArrayObject, JSITER_OWNONLY, JSObject, JSProtoKey, JSType, PropertyDescriptor,
    SetFunctionNativeReserved, ToWindowIfWindowProxy, Value,
};
use js::jsval::{self, NullValue, ObjectValue, UndefinedValue};
use js::panic::{maybe_resume_unwind, wrap_panic};
use js::realm::CurrentRealm;
use js::rust::wrappers2::{
    BigIntToString, Call, Compile1, Construct1, GetBuiltinClass, GetPropertyKeys, IsArray,
    IsPromiseObject, JS_CallFunctionName, JS_DefineProperty, JS_GetClassObject, JS_GetElement,
    JS_GetOwnPropertyDescriptorById, JS_GetProperty, JS_GetPropertyById, JS_GetPrototype,
    JS_HasOwnProperty, JS_IsExceptionPending, JS_NewPlainObject, JS_NewStringCopyUTF8N,
    JS_TypeOfValue, NewArrayObject, NewFunctionWithReserved, ObjectIsDate, ObjectIsRegExp,
    ReportErrorASCII, ToBigInt,
};
use js::rust::{
    HandleObject, HandleValue, IdVector, ToString, error_info_from_exception_stack, for_of,
    transform_str_to_source_text,
};
use net_traits::CookieSource::{HTTP, NonHTTP};
use net_traits::CoreResourceMsg::{DeleteCookie, DeleteCookies, GetCookiesForUrl, SetCookieForUrl};
use script_bindings::callback::ThisReflector;
use script_bindings::codegen::GenericBindings::ShadowRootBinding::{
    ShadowRootMethods, ShadowRootMode,
};
use script_bindings::conversions::{get_dom_class, is_array_like};
use script_bindings::num::Finite;
use script_bindings::reflector::DomObject;
use script_bindings::settings_stack::run_a_script;
use servo_base::generic_channel::{self, GenericOneshotSender, GenericSend, GenericSender};
use servo_base::id::{BrowsingContextId, PipelineId};
use servo_url::ServoUrl;
use webdriver::error::ErrorStatus;
use webdriver_traits::bidi::script::{
    ArrayBufferRemoteValue, ArrayRemoteValue, BigIntValue, BooleanValue, ChannelProperties,
    ChannelValue, DateLocalValue, DateRemoteValue, ErrorRemoteValue, ExceptionDetails,
    FunctionRemoteValue, GeneratorRemoteValue, HtmlCollectionRemoteValue, IncludeShadowTree,
    ListLocalValue, LocalValue, LocalValueOrText, MapRemoteValue, MappingLocalValue,
    NodeListRemoteValue, NodeProperties, NodeRemoteValue, NumberValue, NumberValueKind,
    ObjectRemoteValue, PrimitiveProtocolValue, PromiseRemoteValue, ProxyRemoteValue,
    RegExpLocalValue, RegExpRemoteValue, RegExpValue, RemoteObjectReference, RemoteReference,
    RemoteValue, RemoteValueOrText, ResultOwnership, SerializationOptions, SetRemoteValue,
    SharedId, SharedReference, SpecialNumber, StackTrace, StringValue, SymbolRemoteValue,
    TypedArrayRemoteValue, WeakMapRemoteValue, WeakSetRemoteValue, WindowProxyProperties,
    WindowProxyRemoteValue,
};
use webdriver_traits::bidi::{ErrorCode, script};
use webdriver_traits::ids::{HandleId, InternalId, ResumeId};
use webdriver_traits::messages::{
    CallFunctionBody, EvaluateBody, EvaluationResultBody, MessageBody, PreloadScriptBody,
    ScriptToWebDriverMessage,
};

use crate::DomTypeHolder;
use crate::document_collection::DocumentCollection;
use crate::dom::attr::{Attr, is_boolean_attribute};
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
    ConversionBehavior, ConversionResult, get_property, get_property_jsval, jsid_to_string,
    root_from_handleobject, root_from_object,
};
use crate::dom::bindings::error::{Error, report_pending_exception, throw_dom_exception};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::DomGlobal;
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
use crate::dom::html::htmloptgroupelement::HTMLOptGroupElement;
use crate::dom::html::htmloptionelement::HTMLOptionElement;
use crate::dom::html::htmlselectelement::HTMLSelectElement;
use crate::dom::html::htmltextareaelement::HTMLTextAreaElement;
use crate::dom::html::input_element::HTMLInputElement;
use crate::dom::input_element::input_type::InputType;
use crate::dom::iterators::ShadowIncluding;
use crate::dom::node::{Node, NodeTraits};
use crate::dom::nodelist::NodeList;
use crate::dom::script_execution::{ErrorReporting, evaluate_script, fill_compile_options};
use crate::dom::types::{HTMLCollection, ShadowRoot};
use crate::dom::validitystate::ValidationFlags;
use crate::dom::window::Window;
use crate::dom::xmlserializer::XMLSerializer;
use crate::realms::enter_auto_realm;
use crate::script_module::ScriptFetchOptions;
use crate::script_runtime::CanGc;
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
                    if window_proxy.browsing_context_id() != window_proxy.webview_id()
                        || window_proxy.webview_id().to_string() != webview_id
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
    if let Some(ref node) = node
        && !node.is::<ShadowRoot>()
    {
        return Err(ErrorStatus::NoSuchShadowRoot);
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
    if let Some(ref node) = node
        && !node.is::<Element>()
    {
        return Err(ErrorStatus::NoSuchElement);
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
    cx: &mut JSContext,
    root_node: &Node,
    link_text: String,
    partial: bool,
) -> Result<Vec<String>, ErrorStatus> {
    // <https://w3c.github.io/webdriver/#dfn-find>
    // Step 7.2. If a DOMException, SyntaxError, XPathException, or other error occurs
    // during the execution of the element location strategy, return error invalid selector.
    root_node
        .query_selector_all(cx.no_gc(), DOMString::from("a"))
        .map_err(|_| ErrorStatus::InvalidSelector)
        .map(|nodes| matching_links(&nodes, link_text, partial).collect())
}

#[expect(unsafe_code)]
fn object_has_to_json_property(
    cx: &mut JSContext,
    global_scope: &GlobalScope,
    object: HandleObject,
) -> bool {
    let name = CString::new("toJSON").unwrap();
    let mut found = false;
    if unsafe { JS_HasOwnProperty(cx, object, name.as_ptr(), &mut found) } && found {
        rooted!(&in(cx) let mut value = UndefinedValue());
        let result = unsafe { JS_GetProperty(cx, object, name.as_ptr(), value.handle_mut()) };
        if !result {
            throw_dom_exception(cx, global_scope, Error::JSFailed);
            false
        } else {
            result && unsafe { JS_TypeOfValue(cx, value.handle()) } == JSType::JSTYPE_FUNCTION
        }
    } else if unsafe { JS_IsExceptionPending(cx) } {
        throw_dom_exception(cx, global_scope, Error::JSFailed);
        false
    } else {
        false
    }
}

#[expect(unsafe_code)]
/// <https://w3c.github.io/webdriver/#dfn-collection>
fn is_arguments_object(cx: &mut JSContext, value: HandleValue) -> bool {
    rooted!(&in(cx) let class_name = unsafe { ToString(cx, value) });
    let Some(class_name) = NonNull::new(class_name.get()) else {
        return false;
    };
    let class_name = unsafe { jsstr_to_string(cx, class_name) };
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
    run_a_script::<DomTypeHolder, _, _>(cx, global_scope, |cx| {
        let mut seen = HashSet::new();
        let result = jsval_to_webdriver_inner(cx, global_scope, val, &mut seen);

        if result.is_err() {
            report_pending_exception(cx);
        }
        result
    })
}

#[expect(unsafe_code)]
/// <https://w3c.github.io/webdriver/#dfn-internal-json-clone>
fn jsval_to_webdriver_inner(
    cx: &mut CurrentRealm,
    global_scope: &GlobalScope,
    val: HandleValue,
    seen: &mut HashSet<HashableJSVal>,
) -> WebDriverJSResult {
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
        let string = unsafe { jsstr_to_string(cx, string) };
        Ok(JSValue::String(string))
    } else if val.get().is_object() {
        rooted!(&in(cx) let object = match FromJSValConvertible::safe_from_jsval(cx, val, ()).unwrap() {
            ConversionResult::Success(object) => object,
            _ => unreachable!(),
        });

        if let Ok(element) = unsafe { root_from_object::<Element>(*object, cx.raw_cx()) } {
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
        } else if let Ok(shadow_root) =
            unsafe { root_from_object::<ShadowRoot>(*object, cx.raw_cx()) }
        {
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
        } else if let Ok(window) = unsafe { root_from_object::<Window>(*object, cx.raw_cx()) } {
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
            rooted!(&in(cx) let mut value = UndefinedValue());
            let call_result = unsafe {
                JS_CallFunctionName(
                    cx,
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
                throw_dom_exception(cx, global_scope, Error::JSFailed);
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
    cx: &mut CurrentRealm,
    global_scope: &GlobalScope,
    val: HandleValue,
    seen: &mut HashSet<HashableJSVal>,
    object_handle: HandleObject,
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

    let return_val = if is_array_like::<crate::DomTypeHolder>(cx, val)
        || is_arguments_object(cx, val)
    {
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
                throw_dom_exception(cx, global_scope, error);
                return Err(JavaScriptEvaluationError::SerializationError(
                    JavaScriptEvaluationResultSerializationError::OtherJavaScriptError,
                ));
            },
        };
        // Step 4. For each enumerable property in value, run the following substeps:
        for i in 0..length {
            rooted!(&in(cx) let mut item = UndefinedValue());
            let cname = CString::new(i.to_string()).unwrap();
            let get_property_result =
                get_property_jsval(cx, object_handle, &cname, item.handle_mut());
            match get_property_result {
                Ok(_) => {
                    let converted_item =
                        jsval_to_webdriver_inner(cx, global_scope, item.handle(), seen)?;

                    result.push(converted_item);
                },
                Err(error) => {
                    throw_dom_exception(cx, global_scope, error);
                    return Err(JavaScriptEvaluationError::SerializationError(
                        JavaScriptEvaluationResultSerializationError::OtherJavaScriptError,
                    ));
                },
            }
        }
        Ok(JSValue::Array(result))
    } else {
        let mut result = HashMap::new();

        let mut ids = unsafe { IdVector::new(cx.raw_cx()) };
        let succeeded =
            unsafe { GetPropertyKeys(cx, object_handle, JSITER_OWNONLY, ids.handle_mut()) };
        if !succeeded {
            return Err(JavaScriptEvaluationError::SerializationError(
                JavaScriptEvaluationResultSerializationError::OtherJavaScriptError,
            ));
        }
        for id in ids.iter() {
            rooted!(&in(cx) let id = *id);
            rooted!(&in(cx) let mut desc = PropertyDescriptor::default());

            let mut is_none = false;
            let succeeded = unsafe {
                JS_GetOwnPropertyDescriptorById(
                    cx,
                    object_handle,
                    id.handle(),
                    desc.handle_mut(),
                    &mut is_none,
                )
            };
            if !succeeded {
                return Err(JavaScriptEvaluationError::SerializationError(
                    JavaScriptEvaluationResultSerializationError::OtherJavaScriptError,
                ));
            }

            rooted!(&in(cx) let mut property = UndefinedValue());
            let succeeded = unsafe {
                JS_GetPropertyById(cx, object_handle, id.handle(), property.handle_mut())
            };
            if !succeeded {
                return Err(JavaScriptEvaluationError::SerializationError(
                    JavaScriptEvaluationResultSerializationError::OtherJavaScriptError,
                ));
            }

            if !property.is_undefined() {
                let name = jsid_to_string(cx, id.handle());
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

            let global_scope = window.as_global_scope();

            let mut realm = enter_auto_realm(cx, global_scope);
            let mut realm = realm.current_realm();
            if let Err(error) = global_scope.evaluate_js_on_global(
                &mut realm,
                eval.into(),
                "",
                None, // No known `introductionType` for JS code from WebDriver
                None,
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
fn get_element_in_view_center_point(cx: &mut JSContext, element: &Element) -> Option<Point2D<i64>> {
    let doc = element.owner_document();
    // Step 1: Let rectangle be the first element of the DOMRect sequence
    // returned by calling getClientRects() on element.
    element.GetClientRects(cx).first().map(|rectangle| {
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
    cx: &mut JSContext,
    documents: &DocumentCollection,
    pipeline: PipelineId,
    element_id: String,
    reply: GenericOneshotSender<Result<Option<(i64, i64)>, ErrorStatus>>,
) {
    reply
        .send(
            get_known_element(documents, pipeline, element_id).map(|element| {
                get_element_in_view_center_point(cx, &element).map(|point| (point.x, point.y))
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
    cx: &mut JSContext,
    documents: &DocumentCollection,
    pipeline: PipelineId,
    selector: String,
    reply: GenericSender<Result<Vec<String>, ErrorStatus>>,
) {
    match retrieve_document_and_check_root_existence(documents, pipeline) {
        Ok(document) => reply
            .send(
                document
                    .QuerySelectorAll(cx, DOMString::from(selector))
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
    cx: &mut JSContext,
    documents: &DocumentCollection,
    pipeline: PipelineId,
    selector: String,
    partial: bool,
    reply: GenericSender<Result<Vec<String>, ErrorStatus>>,
) {
    match retrieve_document_and_check_root_existence(documents, pipeline) {
        Ok(document) => reply
            .send(all_matching_links(
                cx,
                document.upcast::<Node>(),
                selector,
                partial,
            ))
            .unwrap(),
        Err(error) => reply.send(Err(error)).unwrap(),
    }
}

pub(crate) fn handle_find_elements_tag_name(
    cx: &mut JSContext,
    documents: &DocumentCollection,
    pipeline: PipelineId,
    selector: String,
    reply: GenericSender<Result<Vec<String>, ErrorStatus>>,
) {
    match retrieve_document_and_check_root_existence(documents, pipeline) {
        Ok(document) => reply
            .send(Ok(document
                .GetElementsByTagName(cx, DOMString::from(selector))
                .elements_iter(cx.no_gc())
                .map(|x| x.upcast::<Node>().unique_id(pipeline))
                .collect::<Vec<String>>()))
            .unwrap(),
        Err(error) => reply.send(Err(error)).unwrap(),
    }
}

/// <https://w3c.github.io/webdriver/#xpath>
fn find_elements_xpath_strategy(
    cx: &mut JSContext,
    document: &Document,
    start_node: &Node,
    selector: String,
    pipeline: PipelineId,
) -> Result<Vec<String>, ErrorStatus> {
    // Step 1. Let evaluateResult be the result of calling evaluate,
    // with arguments selector, start node, null, ORDERED_NODE_SNAPSHOT_TYPE, and null.

    // A snapshot is used to promote operation atomicity.
    let evaluate_result = match document.Evaluate(
        cx,
        DOMString::from(selector),
        start_node,
        None,
        XPathResultConstants::ORDERED_NODE_SNAPSHOT_TYPE,
        None,
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
    cx: &mut JSContext,
    documents: &DocumentCollection,
    pipeline: PipelineId,
    selector: String,
    reply: GenericSender<Result<Vec<String>, ErrorStatus>>,
) {
    match retrieve_document_and_check_root_existence(documents, pipeline) {
        Ok(document) => reply
            .send(find_elements_xpath_strategy(
                cx,
                &document,
                document.upcast::<Node>(),
                selector,
                pipeline,
            ))
            .unwrap(),
        Err(error) => reply.send(Err(error)).unwrap(),
    }
}

pub(crate) fn handle_find_element_elements_css_selector(
    cx: &mut JSContext,
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
                    .query_selector_all(cx.no_gc(), DOMString::from(selector))
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
    cx: &mut JSContext,
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
                all_matching_links(cx, element.upcast::<Node>(), selector.clone(), partial)
            }),
        )
        .unwrap();
}

pub(crate) fn handle_find_element_elements_tag_name(
    cx: &mut JSContext,
    documents: &DocumentCollection,
    pipeline: PipelineId,
    element_id: String,
    selector: String,
    reply: GenericSender<Result<Vec<String>, ErrorStatus>>,
) {
    reply
        .send(
            get_known_element(documents, pipeline, element_id).map(|element| {
                element
                    .GetElementsByTagName(cx, DOMString::from(selector))
                    .elements_iter(cx.no_gc())
                    .map(|x| x.upcast::<Node>().unique_id(pipeline))
                    .collect::<Vec<String>>()
            }),
        )
        .unwrap();
}

pub(crate) fn handle_find_element_elements_xpath_selector(
    cx: &mut JSContext,
    documents: &DocumentCollection,
    pipeline: PipelineId,
    element_id: String,
    selector: String,
    reply: GenericSender<Result<Vec<String>, ErrorStatus>>,
) {
    reply
        .send(
            get_known_element(documents, pipeline, element_id).and_then(|element| {
                find_elements_xpath_strategy(
                    cx,
                    &documents
                        .find_document(pipeline)
                        .expect("Document existence guaranteed by `get_known_element`"),
                    element.upcast::<Node>(),
                    selector,
                    pipeline,
                )
            }),
        )
        .unwrap();
}

/// <https://w3c.github.io/webdriver/#find-elements-from-shadow-root>
pub(crate) fn handle_find_shadow_elements_css_selector(
    cx: &mut JSContext,
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
                    .query_selector_all(cx.no_gc(), DOMString::from(selector))
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
    cx: &mut JSContext,
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
                all_matching_links(cx, shadow_root.upcast::<Node>(), selector.clone(), partial)
            }),
        )
        .unwrap();
}

pub(crate) fn handle_find_shadow_elements_tag_name(
    cx: &mut JSContext,
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
                    .query_selector_all(cx.no_gc(), DOMString::from(selector))
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
    cx: &mut JSContext,
    documents: &DocumentCollection,
    pipeline: PipelineId,
    shadow_root_id: String,
    selector: String,
    reply: GenericSender<Result<Vec<String>, ErrorStatus>>,
) {
    reply
        .send(
            get_known_shadow_root(documents, pipeline, shadow_root_id).and_then(|shadow_root| {
                find_elements_xpath_strategy(
                    cx,
                    &documents
                        .find_document(pipeline)
                        .expect("Document existence guaranteed by `get_known_shadow_root`"),
                    shadow_root.upcast::<Node>(),
                    selector,
                    pipeline,
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

impl Element {
    /// <https://w3c.github.io/webdriver/#dfn-keyboard-interactable>
    fn is_keyboard_interactable(&self) -> bool {
        self.is_focusable_area() || self.is::<HTMLBodyElement>() || self.is_document_element()
    }
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
    cx: &mut JSContext,
    input_element: &HTMLInputElement,
    text: &str,
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
    if let Err(error) = input_element.SetValue(cx, text.into()) {
        error!(
            "Failed to set value on non-typeable input element: {:?}",
            error
        );
        return Err(ErrorStatus::UnknownError);
    }

    // Step 4. If element is suffering from bad input, return ErrorStatus::InvalidArgument.
    if input_element
        .Validity(cx)
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
    cx: &mut JSContext,
    documents: &DocumentCollection,
    pipeline: PipelineId,
    element_id: String,
    text: String,
    strict_file_interactability: bool,
    reply: GenericSender<Result<bool, ErrorStatus>>,
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
    let is_file_input =
        input_element.is_some_and(|e| matches!(*e.input_type(), InputType::File(_)));

    // Step 7. If file is false or the session's strict file interactability
    if !is_file_input || strict_file_interactability {
        // Step 7.1. Scroll into view the element
        scroll_into_view(cx, &element);

        // TODO: Step 7.2 - 7.5
        // Wait until element become keyboard-interactable

        // Step 7.6. If element is not keyboard-interactable,
        // return ErrorStatus::ElementNotInteractable.
        if !element.is_keyboard_interactable() {
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
                cx,
                &FocusOptions {
                    preventScroll: true,
                },
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
            let _ = reply.send(handle_send_keys_non_typeable(cx, input_element, &text));
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
                // FIXME: Actually compute the role instead of using WAI-ARIA role.
                // <https://github.com/servo/servo/issues/43734>
                // The logic can then be shared with devtools accessibility inspector.
                .map(|element| element.GetRole().map(String::from)),
        )
        .unwrap();
}

pub(crate) fn handle_get_page_source(
    cx: &mut JSContext,
    documents: &DocumentCollection,
    pipeline: PipelineId,
    reply: GenericSender<Result<String, ErrorStatus>>,
) {
    reply
        .send(
            documents
                .find_document(pipeline)
                .ok_or(ErrorStatus::UnknownError)
                .and_then(|document| match document.GetDocumentElement() {
                    Some(element) => match element.outer_html(cx) {
                        Ok(source) => Ok(String::from(source)),
                        Err(_) => {
                            match XMLSerializer::new(document.window(), None, CanGc::from_cx(cx))
                                .SerializeToString(element.upcast::<Node>())
                            {
                                Ok(source) => Ok(String::from(source)),
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
                    let (sender, receiver) = generic_channel::channel().unwrap();
                    let _ = document
                        .window()
                        .as_global_scope()
                        .resource_threads()
                        .send(GetCookiesForUrl(url, sender, NonHTTP));
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
                    let (sender, receiver) = generic_channel::channel().unwrap();
                    let _ = document
                        .window()
                        .as_global_scope()
                        .resource_threads()
                        .send(GetCookiesForUrl(url, sender, NonHTTP));
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
            return reply.send(Err(ErrorStatus::NoSuchWindow)).unwrap();
        },
    };
    let url = document.url();
    let method = if cookie.http_only().unwrap_or(false) {
        HTTP
    } else {
        NonHTTP
    };

    let domain = cookie.domain().map(ToOwned::to_owned);
    // Step 6.
    reply
        .send(match (document.is_cookie_averse(), domain) {
            // If session's current browsing context's document element is a
            // cookie-averse Document object, return error with error code invalid cookie domain.
            (true, _) => Err(ErrorStatus::InvalidCookieDomain),
            (false, Some(ref domain)) if url.host_str().is_some_and(|host| host == domain) => {
                let _ = document
                    .window()
                    .as_global_scope()
                    .resource_threads()
                    .send(SetCookieForUrl(url, Serde(cookie), method, None));
                Ok(())
            },
            // If cookie domain is not equal to session's current browsing context's
            // active document's domain, return error with error code invalid cookie domain.
            (false, Some(_)) => Err(ErrorStatus::InvalidCookieDomain),
            (false, None) => {
                let _ = document
                    .window()
                    .as_global_scope()
                    .resource_threads()
                    .send(SetCookieForUrl(url, Serde(cookie), method, None));
                Ok(())
            },
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
    cx: &mut JSContext,
    documents: &DocumentCollection,
    pipeline: PipelineId,
    element_id: String,
    reply: GenericSender<Result<Rect<f64>, ErrorStatus>>,
) {
    reply
        .send(
            get_known_element(documents, pipeline, element_id).and_then(|element| {
                // Step 4-5
                // We pass the rect instead of element so we don't have to
                // call `GetBoundingClientRect` twice.
                let rect = element.GetBoundingClientRect(cx);
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
    cx: &mut JSContext,
    documents: &DocumentCollection,
    pipeline: PipelineId,
    element_id: String,
    reply: GenericSender<Result<Rect<f32>, ErrorStatus>>,
) {
    reply
        .send(
            get_known_element(documents, pipeline, element_id).map(|element| {
                scroll_into_view(cx, &element);

                let rect = element.GetBoundingClientRect(cx);
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
                    .map(|htmlelement| String::from(htmlelement.InnerText()))
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
    cx: &mut JSContext,
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
                    if element.HasAttribute(cx, DOMString::from(name)) {
                        Some(String::from("true"))
                    } else {
                        None
                    }
                } else {
                    element
                        .GetAttribute(cx, DOMString::from(name))
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
                    cx,
                    element.reflector().get_jsobject(),
                    &cname,
                    property.handle_mut(),
                ) {
                    Ok(_) => match jsval_to_webdriver(cx, &element.global(), property.handle()) {
                        Ok(property) => property,
                        Err(_) => JSValue::Undefined,
                    },
                    Err(error) => {
                        throw_dom_exception(cx, &element.global(), error);
                        JSValue::Undefined
                    },
                }
            }),
        )
        .unwrap();
}

pub(crate) fn handle_get_css(
    cx: &mut JSContext,
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
                        .GetComputedStyle(cx, &element, None)
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
        input_element.is_mutable()
            && matches!(
                *input_element.input_type(),
                InputType::Text(_)
                    | InputType::Search(_)
                    | InputType::Url(_)
                    | InputType::Tel(_)
                    | InputType::Email(_)
                    | InputType::Password(_)
                    | InputType::Date(_)
                    | InputType::Month(_)
                    | InputType::Week(_)
                    | InputType::Time(_)
                    | InputType::DatetimeLocal(_)
                    | InputType::Number(_)
                    | InputType::Range(_)
                    | InputType::Color(_)
                    | InputType::File(_)
            )
    } else if let Some(textarea_element) = element.downcast::<HTMLTextAreaElement>() {
        textarea_element.is_mutable()
    } else {
        false
    }
}

/// <https://w3c.github.io/webdriver/#dfn-clear-a-resettable-element>
fn clear_a_resettable_element(cx: &mut JSContext, element: &Element) -> Result<(), ErrorStatus> {
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
        } else if let Some(textarea_element) = element.downcast::<HTMLTextAreaElement>()
            && textarea_element.Value().is_empty()
        {
            return Ok(());
        }
    }

    // Step 3. Invoke the focusing steps for the element.
    html_element.Focus(
        cx,
        &FocusOptions {
            preventScroll: true,
        },
    );

    // Step 4. Run clear algorithm for element.
    if let Some(input_element) = element.downcast::<HTMLInputElement>() {
        input_element.clear(cx);
    } else if let Some(textarea_element) = element.downcast::<HTMLTextAreaElement>() {
        textarea_element.clear();
    } else {
        unreachable!("We have confirm previously that element is mutable form control");
    }

    let event_target = element.upcast::<EventTarget>();
    event_target.fire_bubbling_event(cx, atom!("input"));
    event_target.fire_bubbling_event(cx, atom!("change"));

    // Step 5. Run the unfocusing steps for the element.
    html_element.Blur(cx);

    Ok(())
}

/// <https://w3c.github.io/webdriver/#element-clear>
pub(crate) fn handle_element_clear(
    cx: &mut JSContext,
    documents: &DocumentCollection,
    pipeline: PipelineId,
    element_id: String,
    reply: GenericSender<Result<(), ErrorStatus>>,
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
                scroll_into_view(cx, &element);

                // TODO: Step 6 - 9: Implicit wait. In another PR.
                // Wait until element become interactable and check.

                // Step 10. If element is not keyboard-interactable or not pointer-interactable,
                // return error with error code element not interactable.
                if !element.is_keyboard_interactable() {
                    return Err(ErrorStatus::ElementNotInteractable);
                }

                let paint_tree = get_element_pointer_interactable_paint_tree(cx, &element);
                if !is_element_in_view(&element, &paint_tree) {
                    return Err(ErrorStatus::ElementNotInteractable);
                }

                // Step 11
                // TODO: Clear content editable elements
                clear_a_resettable_element(cx, &element)
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
    cx: &mut JSContext,
    documents: &DocumentCollection,
    pipeline: PipelineId,
    element_id: String,
    reply: GenericSender<Result<Option<String>, ErrorStatus>>,
) {
    reply
        .send(
            // Step 3
            get_known_element(documents, pipeline, element_id).and_then(|element| {
                // Step 4. If the element is an input element in the file upload state
                // return error with error code invalid argument.
                if let Some(input_element) = element.downcast::<HTMLInputElement>()
                    && matches!(*input_element.input_type(), InputType::File(_))
                {
                    return Err(ErrorStatus::InvalidArgument);
                }

                let Some(container) = get_container(&element) else {
                    return Err(ErrorStatus::UnknownError);
                };

                // Step 5. Scroll into view the element's container.
                scroll_into_view(cx, &container);

                // Step 6. If element's container is still not in view
                // return error with error code element not interactable.
                let paint_tree = get_element_pointer_interactable_paint_tree(cx, &container);

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
                        event_target.fire_event(cx, atom!("mouseover"));
                        event_target.fire_event(cx, atom!("mousemove"));
                        event_target.fire_event(cx, atom!("mousedown"));

                        // Step 8.5
                        match container.downcast::<HTMLElement>() {
                            Some(html_element) => {
                                html_element.Focus(
                                    cx,
                                    &FocusOptions {
                                        preventScroll: true,
                                    },
                                );
                            },
                            None => return Err(ErrorStatus::UnknownError),
                        }

                        // Step 8.6
                        if !is_disabled(&element) {
                            // Step 8.6.1
                            event_target.fire_event(cx, atom!("input"));

                            // Steps 8.6.2
                            let previous_selectedness = option_element.Selected();

                            // Step 8.6.3
                            match container.downcast::<HTMLSelectElement>() {
                                Some(select_element) => {
                                    if select_element.Multiple() {
                                        option_element.SetSelected(cx, !option_element.Selected());
                                    }
                                },
                                None => option_element.SetSelected(cx, true),
                            }

                            // Step 8.6.4
                            if !previous_selectedness {
                                event_target.fire_event(cx, atom!("change"));
                            }
                        }

                        // Steps 8.7 - 8.8
                        event_target.fire_event(cx, atom!("mouseup"));
                        event_target.fire_event(cx, atom!("click"));

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
    cx: &mut JSContext,
    element: &Element,
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

    get_element_in_view_center_point(cx, element).map_or(Vec::new(), |center_point| {
        if let Some(shadow_root) = element.containing_shadow_root() {
            shadow_root.ElementsFromPoint(
                Finite::wrap(center_point.x as f64),
                Finite::wrap(center_point.y as f64),
            )
        } else {
            element.owner_document().ElementsFromPoint(
                Finite::wrap(center_point.x as f64),
                Finite::wrap(center_point.y as f64),
            )
        }
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
fn scroll_into_view(cx: &mut JSContext, element: &Element) {
    // Check if element is already in view
    let paint_tree = get_element_pointer_interactable_paint_tree(cx, element);
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
    element.ScrollIntoView(cx, options);
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

/// <https://www.w3.org/TR/webdriver-bidi/#run-webdriver-bidi-preload-scripts>.
/// Starting from 5.1.5
#[expect(unsafe_code)]
pub(crate) fn run_webdriver_preload_script(
    cx: &mut CurrentRealm,
    window: &Window,
    preload_script: Rc<PreloadScriptBody>,
) {
    let global_scope = window.upcast::<GlobalScope>();
    // Step 5.1.5.
    let arguments = &preload_script.arguments;
    // Step 5.1.6.
    rooted_vec!(let mut deserialized_arguments);
    // Step 5.1.7. create channel for each
    for argument in arguments {
        // Step 5.1.7.1
        rooted!(&in(cx) let mut channel = UndefinedValue());
        if let Err(_err) = create_a_channel(cx, argument, channel.handle_mut()) {
            report_pending_exception(cx);
            return;
        }
        // Step 5.1.7.2
        deserialized_arguments.push(channel.get());
    }

    // Step 5.1.{8,9}.
    let base_url = global_scope.api_base_url();
    let options = ScriptFetchOptions::default_classic_script();

    // Step 5.1.10
    let function_declaration = &preload_script.function_declaration;

    // Step 5.1.11.
    rooted!(&in(cx) let mut function_val = UndefinedValue());
    let function_body_evaluation_status = evaluate_function_body(
        cx,
        global_scope,
        base_url,
        options,
        function_declaration,
        // Step 5.1.13.
        function_val.handle_mut(),
    );

    // Step 5.1.12. if is abrupt
    if !function_body_evaluation_status {
        report_pending_exception(cx);
        return;
    }
    // Step 5.1.14.
    if function_val.is_object()
        && let function_obj = function_val.to_object()
        && unsafe { IsCallable(function_obj) }
    {
    } else {
        unsafe { throw_type_error(cx.raw_cx(), c"preload script is not an function") };
        return;
    };

    let mut evaluation_status = false;
    rooted!(&in(cx) let mut value = UndefinedValue());

    // Step 5.1.{15,17}.
    run_a_script::<DomTypeHolder, _, _>(cx, global_scope, |cx| {
        rooted!(&in(cx) let this = UndefinedValue());
        // Step 5.1.16.
        evaluation_status = unsafe {
            Call(
                cx.as_mut(),
                this.handle(),
                function_val.handle(),
                &(&deserialized_arguments).into(),
                value.handle_mut(),
            )
        };
    });
    // Step 5.1.18.
    if evaluation_status {
        report_pending_exception(cx);
    }
}

/// Part of <https://www.w3.org/TR/webdriver-bidi/#command-script-disown>.
pub(crate) fn handle_webdriver_bidi_script_disown(
    global_scope: &GlobalScope,
    resume_id: ResumeId,
    handles: Vec<HandleId>,
) {
    // Step 3.
    for handle in handles {
        global_scope.disown_handle(&handle);
    }
    // Step 4.
    if let Some(chan) = global_scope.webdriver_chan()
        && let Err(err) = chan.send(ScriptToWebDriverMessage::Disowned(resume_id))
    {
        warn!("Sending disown response to webdriver failed ({err:?})");
    }
}

/// Part of <https://www.w3.org/TR/webdriver-bidi/#command-script-evaluate>.
#[expect(unsafe_code)]
pub(crate) fn handle_webdriver_bidi_script_evaluate(
    cx: &mut JSContext,
    global_scope: &GlobalScope,
    resume_id: ResumeId,
    body: EvaluateBody,
) {
    let mut realm = enter_auto_realm(cx, global_scope);
    let cx = &mut realm.current_realm();

    let send_error = |err| {
        _ = global_scope
            .webdriver_chan()
            .as_ref()
            .unwrap()
            .send(ScriptToWebDriverMessage::Evaluated(resume_id, Err(err)));
    };

    // Step 3. skip abstract step

    // Step {4,5,6,7}.
    let source = body.expression;
    let await_promise = body.await_promise;
    let serialization_options = body.serialization_options;
    let result_ownership = body.result_ownership;

    // Step 8. default script fetch options
    let options = ScriptFetchOptions::default_classic_script();
    // Step 9. api base url
    let base_url = global_scope.api_base_url();
    // Step 10. not used in compile args
    let _bypass_disabled_scripting = true;

    // Step 11. create a classic script.
    let mut source = transform_str_to_source_text(&source);
    let compile_options = fill_compile_options(cx, "", None, ErrorReporting::Unmuted, false, 1);
    rooted!(&in(cx) let script = unsafe { Compile1(cx, compile_options.ptr, &mut source) });
    let Some(script) = NonNull::new(*script) else {
        debug!("error compiling Dom string");
        report_pending_exception(cx);

        send_error(ErrorCode::UnknownError);
        return;
    };

    let mut evaluation_status = false;
    rooted!(&in(cx) let mut value = UndefinedValue());

    // Step {13,16}. prepare and cleanup
    run_a_script::<DomTypeHolder, _, _>(cx, global_scope, |cx| {
        // Step 14. script evaluation
        evaluation_status = evaluate_script(cx, script, base_url, options, value.handle_mut());
        // Step 15. maybe await promise
        if evaluation_status && await_promise {
            // TODO: and check if is promise
            // NOTE: await should not be handled here, pending instead
        }

        maybe_resume_unwind();
    });

    // Step 17. if throw
    if !evaluation_status {
        // Step 17.1. get exception details
        let exception_details = get_exception_details(cx, evaluation_status, result_ownership);
        report_pending_exception(cx);
        // Step 17.2. return
        _ = global_scope.webdriver_chan().as_ref().unwrap().send(
            ScriptToWebDriverMessage::Evaluated(
                resume_id,
                exception_details.map(EvaluationResultBody::Exception),
            ),
        );
    }

    // Step 18. assert normal
    debug_assert!(evaluation_status);
    // Step 19. serialize
    let result =
        serialize_as_a_remote_value(cx, value.handle(), serialization_options, result_ownership);
    report_pending_exception(cx);
    // Step 20. return
    _ = global_scope
        .webdriver_chan()
        .as_ref()
        .unwrap()
        .send(ScriptToWebDriverMessage::Evaluated(
            resume_id,
            result.map(EvaluationResultBody::Success),
        ));
}

/// Part of <https://www.w3.org/TR/webdriver-bidi/#command-script-callFunction>.
#[expect(unsafe_code)]
pub(crate) fn handle_webdriver_bidi_script_call_function(
    cx: &mut JSContext,
    global_scope: &GlobalScope,
    resume_id: ResumeId,
    body: CallFunctionBody,
) {
    let mut realm = enter_auto_realm(cx, global_scope);
    let cx = &mut realm.current_realm();

    let send_error = |err| {
        _ = global_scope
            .webdriver_chan()
            .as_ref()
            .unwrap()
            .send(ScriptToWebDriverMessage::Evaluated(resume_id, Err(err)));
    };

    // Step {5,6}. deserialize arguments
    rooted_vec!(let mut deserialized_arguments);
    if let Err(err) = deserialize_arguments(cx, body.arguments, &mut deserialized_arguments) {
        report_pending_exception(cx);
        send_error(err);
        return;
    }

    // Step {7,8,9}. deserialize this
    rooted!(&in(cx) let mut this_object = UndefinedValue());
    if let Some(this_parameter) = body.this {
        if let Err(err) = deserialize_local_value(cx, this_parameter, this_object.handle_mut()) {
            report_pending_exception(cx);
            send_error(err);
            return;
        }
    }

    // Step {10,11,12,13}.
    let function_declaration = body.function_declaration;
    let await_promise = body.await_promise;
    let serialization_options = body.serialization_options;
    let result_ownership = body.result_ownership;

    // Step 14. api base url
    let base_url = global_scope.api_base_url();
    // Step 15. default script fetch options
    let options = ScriptFetchOptions::default_classic_script();

    rooted!(&in(cx) let mut function_val = UndefinedValue());
    // Step 16. "evaluate function body"
    let function_body_evaluation_status = evaluate_function_body(
        cx,
        global_scope,
        base_url,
        options,
        &function_declaration,
        function_val.handle_mut(),
    );

    // Step 17. if is throw
    if !function_body_evaluation_status {
        // Step 17.1. "get exception details"
        let exception_details =
            get_exception_details(cx, function_body_evaluation_status, result_ownership);
        report_pending_exception(cx);
        // Step 17.2. return
        _ = global_scope.webdriver_chan().as_ref().unwrap().send(
            ScriptToWebDriverMessage::Evaluated(
                resume_id,
                exception_details.map(EvaluationResultBody::Exception),
            ),
        );
    }
    // Step 18.
    let function_obj = if function_val.is_object() {
        function_val.to_object()
    } else {
        send_error(ErrorCode::UnknownError);
        return;
    };
    // Step 19. check callable
    if !unsafe { IsCallable(function_obj) } {
        // Step 19.1. error "invalid argument"
        send_error(ErrorCode::InvalidArgument);
        return;
    }
    // Step 20. user activation
    if body.user_activation {
        // TODO: run activation notification
    }

    let mut evaluation_status = false;
    rooted!(&in(cx) let mut value = UndefinedValue());

    // Step {21,24}. prepare and cleanup
    run_a_script::<DomTypeHolder, _, _>(cx, global_scope, |cx| {
        // Step 22. call function
        evaluation_status = unsafe {
            Call(
                cx.as_mut(),
                this_object.handle(),
                function_val.handle(),
                &(&deserialized_arguments).into(),
                value.handle_mut(),
            )
        };
        // Step 23. maybe await promise
        if evaluation_status && await_promise {
            // TODO: await to pend
        }
    });

    // Step 25. if throw
    if !evaluation_status {
        // Step 25.1. "get exception details"
        let exception_details =
            get_exception_details(cx, function_body_evaluation_status, result_ownership);
        report_pending_exception(cx);
        // Step 25.2. return
        _ = global_scope.webdriver_chan().as_ref().unwrap().send(
            ScriptToWebDriverMessage::Evaluated(
                resume_id,
                exception_details.map(EvaluationResultBody::Exception),
            ),
        );
    }

    // Step 26.
    debug_assert!(evaluation_status);
    // Step 27. serialize
    let result =
        serialize_as_a_remote_value(cx, value.handle(), serialization_options, result_ownership);
    report_pending_exception(cx);
    // Step 28. return
    _ = global_scope
        .webdriver_chan()
        .as_ref()
        .unwrap()
        .send(ScriptToWebDriverMessage::Evaluated(
            resume_id,
            result.map(EvaluationResultBody::Success),
        ));
}

/// <https://www.w3.org/TR/webdriver-bidi/#deserialize-arguments>
fn deserialize_arguments(
    cx: &mut CurrentRealm,
    serialized_arguments_list: Vec<LocalValue>,
    deserialize_arguments_list: &mut RootedVec<Value>,
) -> Result<(), ErrorCode> {
    // Step 2.
    for serialized_argument in serialized_arguments_list {
        // Step 2.1. deserialize local value
        rooted!(&in(cx) let mut deserialized_argument = UndefinedValue());
        deserialize_local_value(cx, serialized_argument, deserialized_argument.handle_mut())?;
        // Step 2.2. append
        deserialize_arguments_list.push(deserialized_argument.get());
    }
    // Step 3. return success.
    Ok(())
}

/// <https://www.w3.org/TR/webdriver-bidi/#evaluate-function-body>
#[expect(unsafe_code)]
fn evaluate_function_body(
    cx: &mut CurrentRealm,
    global_scope: &GlobalScope,
    base_url: ServoUrl,
    options: ScriptFetchOptions,
    function_declaration: &str,
    function_rval: MutableHandleValue,
) -> bool {
    // Step 1. not used
    let _bypass_disabled_scripting = true;
    // Step 2. paren
    let parenthesized_function_declaration = format!("({function_declaration})");

    // Step 3. create a classic script
    let mut source = transform_str_to_source_text(&parenthesized_function_declaration);
    let compile_options = fill_compile_options(cx, "", None, ErrorReporting::Unmuted, false, 1);
    rooted!(&in(cx) let script = unsafe { Compile1(cx, compile_options.ptr, &mut source) });
    let Some(script) = NonNull::new(*script) else {
        debug!("error compiling Dom string");
        report_pending_exception(cx);
        return false;
    };

    let mut function_body_evaluation_status = false;

    // Step {4,6}. prepare and cleanup
    run_a_script::<DomTypeHolder, _, _>(cx, global_scope, |cx| {
        // Step 5. ScriptEvaluation
        function_body_evaluation_status =
            evaluate_script(cx, script, base_url, options, function_rval);

        maybe_resume_unwind();
    });

    // Step 7. return
    function_body_evaluation_status
}

/// See <https://www.w3.org/TR/webdriver-bidi/#get-exception-details>
#[expect(unsafe_code)]
fn get_exception_details(
    cx: &mut CurrentRealm,
    record: bool,
    ownership_type: ResultOwnership,
) -> Result<ExceptionDetails, ErrorCode> {
    // Step 1.
    debug_assert!(!record);

    rooted!(&in(cx) let mut error = UndefinedValue());
    let error_info =
        unsafe { error_info_from_exception_stack(cx.raw_cx(), error.handle_mut().into()) }
            .ok_or(ErrorCode::UnknownError)?;

    // Step 2.
    let text = error_info.message;

    // Step 3.
    let serialization_options = SerializationOptions::default();

    // Step 4.
    let exception =
        serialize_as_a_remote_value(cx, error.handle(), serialization_options, ownership_type)?;

    // Step 5.
    // TODO: mozjs does not expose ExceptinStack::stack_, though
    // we can get it through raw jsapi, it should not be done at this
    // layer.
    let stack_trace = StackTrace {
        call_frames: vec![],
    };

    // Step 6.
    let line_number = error_info.line;
    let column_number = error_info.col;

    // Step 7.
    let exception_details = ExceptionDetails {
        column_number,
        exception,
        line_number,
        stack_trace,
        text,
    };

    // Step 8.
    Ok(exception_details)
}

pub(crate) fn notify_webdriver_realm_destroyed(global_scope: &GlobalScope) {
    if let Some(sender) = global_scope.webdriver_chan() {
        if let Err(err) = sender.send(ScriptToWebDriverMessage::RealmDestroyed(
            global_scope.realm_id(),
        )) {
            warn!("Sending RealmDestroyed event to webdriver failed ({err})");
        }
    }
}

/// See <https://www.w3.org/TR/webdriver-bidi/#serialize-as-a-remote-value>
pub(crate) fn serialize_as_a_remote_value(
    realm: &mut CurrentRealm,
    value: HandleValue,
    serialization_options: SerializationOptions,
    ownership_type: ResultOwnership,
) -> Result<RemoteValue, ErrorCode> {
    serialize_as_a_remote_value_partial(
        realm,
        value,
        serialization_options,
        ownership_type,
        &mut Default::default(),
    )
    .and_then(|v| v.to_remote_value())
}

/// See <https://www.w3.org/TR/webdriver-bidi/#serialize-as-a-remote-value>
#[expect(unsafe_code)]
fn serialize_as_a_remote_value_partial(
    cx: &mut CurrentRealm,
    value: HandleValue,
    serialization_options: SerializationOptions,
    ownership_type: ResultOwnership,
    serialization_internal_map: &mut InternalMap,
) -> Result<PartialRemoteValue, ErrorCode> {
    let global = GlobalScope::from_current_realm(cx);

    // Step 1.
    let remote_value = serialize_primitive_protocol_value(cx, value)?;
    // Step 2.
    if let Some(remote_value) = remote_value {
        return Ok(PartialRemoteValue::PrimitiveProtocol(remote_value));
    }

    // Step 3. "handle for an object"
    rooted!(&in(cx) let obj = value.to_object());
    let handle_id = global.handle_for_an_object(ownership_type, obj.handle().into());

    // Step 4. set ownership type to none
    let ownership_type = ResultOwnership::None;
    // Step 5. check known object
    let known_object = serialization_internal_map.contains_key(&value.get().asBits_);

    // Step 6.
    let mut cls = ESClass::Object;
    rooted!(&in(cx) let mut prototype = unsafe { JS_NewPlainObject(cx.as_mut()) });
    if prototype.is_null() {
        return Err(ErrorCode::UnknownError);
    }

    macro_rules! simple_value {
        ($variant:ident, $ty:ident) => {
            PartialRemoteValue::$variant(Rc::new(RefCell::new($ty {
                handle: handle_id,
                internal_id: None,
            })))
        };
    }
    macro_rules! array_like_value {
        ($ty:ident) => {
            serialize_an_array_like::<$ty>(
                cx,
                handle_id,
                known_object,
                value,
                serialization_options,
                ownership_type,
                serialization_internal_map,
            )?
        };
    }

    macro_rules! err {
        () => {
            return Err(ErrorCode::UnknownError);
        };
    }

    let remote_value;
    'm: {
        // Step 6.Symbol.
        if value.is_symbol() {
            remote_value = simple_value!(Symbol, SymbolRemoteValue);
            break 'm;
        }

        // Step 6.Array.
        let mut is_array = false;
        if unsafe { IsArray(cx.as_mut(), obj.handle(), &mut is_array) } {
            if is_array {
                remote_value = array_like_value!(PartialArrayRemoteValue);
                break 'm;
            }
        } else {
            err!();
        }

        // Step 6.RegExp.
        let mut is_reg_exp = false;
        if unsafe { ObjectIsRegExp(cx.as_mut(), obj.handle(), &mut is_reg_exp) } {
            if is_reg_exp {
                // Step 6.RegExp.1.
                let pattern = serialize_property_string(cx.as_mut(), obj.handle(), c"source")?
                    // the spec does not specify how to handle error
                    .unwrap_or_default();
                // Step 6.RegExp.2.
                let flags = serialize_property_string(cx.as_mut(), obj.handle(), c"flags")?;
                // Step 6.RegExp.3.
                let serialized = RegExpValue { pattern, flags };
                // Step 6.RegExp.4.
                remote_value =
                    PartialRemoteValue::RegExp(Rc::new(RefCell::new(RegExpRemoteValue {
                        local: RegExpLocalValue { value: serialized },
                        handle: handle_id,
                        internal_id: None,
                    })));
                break 'm;
            }
        } else {
            err!();
        }

        // Step 6.Date.
        let mut is_date = false;
        if unsafe { ObjectIsDate(cx.as_mut(), obj.handle(), &mut is_date) } {
            if is_date {
                // Step 6.Date.1.
                let serialized = {
                    rooted!(&in(cx) let mut out = UndefinedValue());
                    if unsafe {
                        JS_CallFunctionName(
                            cx.as_mut(),
                            obj.handle(),
                            c"toISOString".as_ptr(),
                            &HandleValueArray::empty(),
                            out.handle_mut(),
                        )
                    } {
                        serialize_string(cx, out.handle())?
                    } else {
                        err!();
                    }
                };
                // Step 6.Date.2. skip assert
                // Step 6.Date.3.
                remote_value = PartialRemoteValue::Date(Rc::new(RefCell::new(DateRemoteValue {
                    local: DateLocalValue { value: serialized },
                    handle: handle_id,
                    internal_id: None,
                })));
                break 'm;
            };
        } else {
            err!();
        }

        // Step 6.Map.
        if unsafe { GetBuiltinClass(cx.as_mut(), obj.handle(), &mut cls) } {
            if cls == ESClass::Map {
                // Step 6.Map.1.
                let map_remote_value = Rc::new(RefCell::new(PartialMapRemoteValue {
                    handle: handle_id,
                    internal_id: None,
                    value: None,
                }));

                // Step 6.Map.2. set internal
                set_internal_ids_if_needed(
                    serialization_internal_map,
                    PartialRemoteValue::Map(map_remote_value.clone()),
                    value.get(),
                );

                // Step 6.Map.3.
                let mut serialized = None;
                // Step 6.Map.4.
                if !known_object && serialization_options.max_object_depth != Some(0) {
                    // Step 6.Set.4.1. "serialize as a mapping"
                    serialized = Some(serialize_as_a_mapping(
                        cx,
                        value,
                        serialization_options,
                        ownership_type,
                        serialization_internal_map,
                    )?);
                }
                // Step 6.Map.5.
                if serialized.is_some() {
                    map_remote_value.borrow_mut().value = serialized;
                }
                remote_value = PartialRemoteValue::Map(map_remote_value);
                break 'm;
            };
        } else {
            err!();
        }

        // Step 6.Set.
        if unsafe { GetBuiltinClass(cx.as_mut(), obj.handle(), &mut cls) } {
            if cls == ESClass::Set {
                // Step 6.Set.1.
                let set_remote_value = Rc::new(RefCell::new(PartialSetRemoteValue {
                    handle: handle_id,
                    internal_id: None,
                    value: None,
                }));

                // Step 6.Set.2.
                set_internal_ids_if_needed(
                    serialization_internal_map,
                    PartialRemoteValue::Set(set_remote_value.clone()),
                    value.get(),
                );

                // Step 6.Set.3.
                let mut serialized = None;
                // Step 6.Set.4.
                if !known_object && serialization_options.max_object_depth != Some(0) {
                    // Step 6.Set.4.1. "serialize as a list"
                    serialized = Some(serialize_as_a_list(
                        cx,
                        value,
                        serialization_options,
                        ownership_type,
                        serialization_internal_map,
                    )?);
                }
                // Step 6.Set.5.
                if serialized.is_some() {
                    set_remote_value.borrow_mut().value = serialized;
                }
                remote_value = PartialRemoteValue::Set(set_remote_value);
                break 'm;
            }
        } else {
            err!();
        }

        // Step 6.WeakMap.
        if unsafe { IsWeakMapObject(obj.get()) } {
            remote_value = simple_value!(WeakMap, WeakMapRemoteValue);
            break 'm;
        }

        // Step 6.WeakSet
        if unsafe { JS_GetPrototype(cx.as_mut(), obj.handle(), prototype.handle_mut()) } {
            if unsafe { IdentifyStandardPrototype(prototype.get()) } == JSProtoKey::JSProto_WeakSet
            {
                remote_value = simple_value!(WeakSet, WeakSetRemoteValue);
                break 'm;
            }
        } else {
            err!();
        }

        // Step 6.Generator.
        if unsafe { JS_GetPrototype(cx.as_mut(), obj.handle(), prototype.handle_mut()) } {
            let proto_key = unsafe { IdentifyStandardPrototype(prototype.get()) };
            if proto_key == JSProtoKey::JSProto_GeneratorFunction
                || proto_key == JSProtoKey::JSProto_AsyncGeneratorFunction
            {
                remote_value = simple_value!(Generator, GeneratorRemoteValue);
                break 'm;
            }
        } else {
            err!();
        }

        // Step 6.Error.
        if unsafe { GetBuiltinClass(cx.as_mut(), obj.handle(), &mut cls) } {
            if cls == ESClass::Error {
                remote_value = simple_value!(Error, ErrorRemoteValue);
                break 'm;
            };
        } else {
            err!();
        }

        // Step 6.Proxy.
        if unsafe { IsProxyHandlerFamily(obj.get()) } {
            remote_value = simple_value!(Proxy, ProxyRemoteValue);
            break 'm;
        }

        // Step 6.Promise.
        if unsafe { IsPromiseObject(obj.handle()) } {
            remote_value = simple_value!(Promise, PromiseRemoteValue);
            break 'm;
        }

        // Step 6.TypedArray.
        if unsafe { JS_IsTypedArrayObject(obj.get()) } {
            remote_value = simple_value!(TypedArray, TypedArrayRemoteValue);
            break 'm;
        }

        // Step 6.ArrayBuffer.
        if unsafe { IsArrayBufferObject(obj.get()) } {
            remote_value = simple_value!(ArrayBuffer, ArrayBufferRemoteValue);
            break 'm;
        }

        // Step 6.NodeList.
        if unsafe { root_from_object::<NodeList>(obj.get(), cx.as_mut().raw_cx()) }.is_ok() {
            remote_value = array_like_value!(PartialNodeListRemoteValue);
            break 'm;
        }

        // Step 6.HTMLCollection.
        if unsafe { root_from_object::<HTMLCollection>(obj.get(), cx.as_mut().raw_cx()) }.is_ok() {
            remote_value = array_like_value!(PartialHtmlCollectionRemoteValue);
            break 'm;
        }

        // Step 6.Node.
        if let Ok(node) = root_from_handleobject::<Node>(obj.handle(), unsafe { cx.raw_cx() }) {
            remote_value = PartialRemoteValue::Node(serialize_node_partial(
                cx,
                value.get(),
                &node,
                handle_id,
                serialization_options,
                ownership_type,
                serialization_internal_map,
            )?);
            break 'm;
        };

        // Step 6.WindowProxy.
        if unsafe { IsWindowProxy(obj.get()) } {
            rooted!(&in(cx) let window = unsafe { ToWindowIfWindowProxy(obj.get()) });
            if window.is_null() {
                err!();
            }

            let window = root_from_handleobject::<Window>(window.handle(), unsafe { cx.raw_cx() })
                .map_err(|_| ErrorCode::UnknownError)?;

            let navigable_id = BrowsingContextId::from(window.webview_id());

            remote_value =
                PartialRemoteValue::Window(Rc::new(RefCell::new(WindowProxyRemoteValue {
                    value: WindowProxyProperties {
                        context: navigable_id,
                    },
                    handle: handle_id,
                    internal_id: None,
                })));
            break 'm;
        }

        // Step 6.platformobject
        if unsafe { get_dom_class(obj.get()) }.is_ok() {
            remote_value =
                PartialRemoteValue::Object(Rc::new(RefCell::new(PartialObjectRemoteValue {
                    handle: handle_id,
                    internal_id: None,
                    value: None,
                })));
            break 'm;
        }

        // Step 6.Callable.
        if unsafe { IsCallable(obj.get()) } {
            remote_value = simple_value!(Function, FunctionRemoteValue);
            break 'm;
        }

        // Step 6.Otherwise.
        // Step 6.Otherwise.1. skip assert
        // Step 6.Otherwise.2.
        let obj_remote_value = Rc::new(RefCell::new(PartialObjectRemoteValue {
            handle: handle_id,
            internal_id: None,
            value: None,
        }));

        // Step 6.Otherwise.3. set internal id
        set_internal_ids_if_needed(
            serialization_internal_map,
            PartialRemoteValue::Object(obj_remote_value.clone()),
            value.get(),
        );

        // Step 6.Otherwise.4.
        let mut serialized = None;
        // Step 6.Otherwise.5.
        if !known_object && serialization_options.max_object_depth != Some(0) {
            // Step 6.Otherwise.5.1. serialize enumerable properties
            // currently we use Object.entries
            // XXX: is there internal fn for `EnumerableOwnPropertyNames(value, key+value)`?
            rooted_vec!(let mut args);
            args.push(value.get());
            rooted!(&in(cx) let mut entries = UndefinedValue());
            constructor_or_static_method(
                cx,
                JSProtoKey::JSProto_Object,
                Some(c"entries"),
                (&args).into(),
                entries.handle_mut(),
            )?;
            serialized = Some(serialize_as_a_mapping(
                cx,
                entries.handle(),
                serialization_options,
                ownership_type,
                serialization_internal_map,
            )?);
        }
        // Step 6.Otherwise.6.
        if let Some(serialized) = serialized {
            obj_remote_value.borrow_mut().value = Some(serialized);
        }
        remote_value = PartialRemoteValue::Object(obj_remote_value)
    }

    // Step 7. return
    Ok(remote_value)
}

fn serialize_node_partial(
    realm: &mut CurrentRealm,
    value: Value,
    node: &Node,
    handle_id: Option<HandleId>,
    serialization_options: SerializationOptions,
    ownership_type: ResultOwnership,
    serialization_internal_map: &mut InternalMap,
) -> Result<Rc<RefCell<PartialNodeRemoteValue>>, ErrorCode> {
    // Step 6.Node.1.
    // TODO: shared id, blocked until we implement sandbox
    let shared_id = None;
    // Step 6.Node.2.
    let remote_value = Rc::new(RefCell::new(PartialNodeRemoteValue {
        shared_id: shared_id,
        handle: handle_id,
        internal_id: None,
        value: None,
    }));

    // Step 6.Node.3. set internal id
    set_internal_ids_if_needed(
        serialization_internal_map,
        PartialRemoteValue::Node(remote_value.clone()),
        value,
    );

    // Step 6.Node.{4,5}. serialize properties
    let known_object = serialization_internal_map.contains_key(&value.asBits_);
    if !known_object {
        // Step 6.Node.5.1.
        let mut serialized = PartialNodeProperties::default();

        // Step 6.Node.5.2. nodeType
        serialized.node_type = node.NodeType();
        // Step 6.Node.5.{3,4}. nodeValue
        serialized.node_value = node.GetNodeValue().map(String::from);

        // Step 6.Node.5.5.element
        if let Some(element) = node.downcast::<Element>() {
            // Step 6.Node.5.5.element.1.
            serialized.local_name = Some(String::from(&**element.local_name()));
            // Step 6.Node.5.5.element.2.
            serialized.namespace_uri = Some(String::from(&**element.namespace()));
        }

        // Step 6.Node.5.5.attr
        if let Some(attr) = node.downcast::<Attr>() {
            // Step 6.Node.5.5.attr.1.
            serialized.local_name = Some(String::from(&**attr.local_name()));
            // Step 6.Node.5.5.attr.2.
            serialized.namespace_uri = Some(String::from(&**attr.namespace()));
        }

        // Step 6.Node.5.{6,7}. child node count
        serialized.child_node_count = node.children_count();

        // Step 6.Node.5.8. shadow root
        let children = if serialization_options.max_dom_depth.unwrap_or_default() == 0
            || node.is::<ShadowRoot>()
                && serialization_options
                    .include_shadow_tree
                    .unwrap_or_default()
                    == script::IncludeShadowTree::None
        {
            None
        }
        // otherwise
        else {
            let mut children = vec![];
            for child in node.children() {
                // Step 6.Node.5.8.1. clone child serialization options
                let mut child_serialization_options = serialization_options.clone();
                // Step 6.Node.5.8.2. sub maxDomDepth
                if let Some(max_dom_depth) = &mut child_serialization_options.max_dom_depth {
                    *max_dom_depth = max_dom_depth.saturating_sub(1);
                }

                // Step 6.Node.5.8.3. serialize child
                let serialized = serialize_node_partial(
                    realm,
                    ObjectValue(child.jsobject()),
                    &child,
                    handle_id,
                    serialization_options.clone(),
                    ownership_type,
                    serialization_internal_map,
                )?;

                // Step 6.Node.5.8.4. append
                children.push(serialized);
            }
            Some(children)
        };

        // Step 6.Node.5.9.
        if children.is_some() {
            serialized.children = children;
        }

        // Step 6.Node.5.10. if element
        if let Some(element) = node.downcast::<Element>() {
            // Step 6.Node.5.10.1.
            let mut attributes = HashMap::new();
            // Step 6.Node.5.10.2.
            for attr in element.attrs().borrow().iter() {
                // Step 6.Node.5.10.2.1
                let name = String::from(&**attr.name());
                // Step 6.Node.5.10.2.2
                let value = String::from(&**attr.value());
                // Step 6.Node.5.10.2.3
                attributes.insert(name, value);
            }

            // Step 6.Node.5.10.3.
            serialized.attributes = Some(attributes);

            // Step 6.Node.5.10.4.
            let shadow_root = element.shadow_root();
            // Step 6.Node.5.10.5.
            let serialized_shadow = match shadow_root {
                None => None,
                // Step 6.Node.5.10.5.1
                Some(shadow_root) => {
                    let shadow_root_node = shadow_root.upcast::<Node>();
                    Some(serialize_node_partial(
                        realm,
                        ObjectValue(shadow_root_node.jsobject()),
                        node,
                        handle_id,
                        serialization_options,
                        ownership_type,
                        serialization_internal_map,
                    )?)
                },
            };

            // Step 6.Node.5.10.6.
            serialized.shadow_root = serialized_shadow;
        }

        // Step 6.Node.5.11. if shadow root
        if let Some(shadow_root) = node.downcast::<ShadowRoot>() {
            serialized.mode = Some(match shadow_root.shadow_root_mode() {
                ShadowRootMode::Open => script::ShadowRootMode::Open,
                ShadowRootMode::Closed => script::ShadowRootMode::Closed,
            });
        }

        // Step 6.Node.6.
        remote_value.borrow_mut().value = Some(serialized);
    }
    Ok(remote_value)
}

/// See <https://www.w3.org/TR/webdriver-bidi/#serialize-primitive-protocol-value>.
#[expect(unsafe_code)]
fn serialize_primitive_protocol_value(
    cx: &mut CurrentRealm,
    value: HandleValue,
) -> Result<Option<PrimitiveProtocolValue>, ErrorCode> {
    // Step 1
    let mut remote_value = None;

    // Step 2.undefined
    if value.is_undefined() {
        remote_value = Some(PrimitiveProtocolValue::Undefined);
    }

    // Step 2.null
    if value.is_null() {
        remote_value = Some(PrimitiveProtocolValue::Null);
    }

    // Step 2.string
    if value.is_string() {
        remote_value = Some(
            serialize_string(cx, value)
                .map(|s| PrimitiveProtocolValue::String(StringValue { value: s }))?,
        );
    }

    // Step 2.number
    if value.is_number() {
        let value = value.to_number();
        // Step 2.number.1.
        let serialized = match value {
            v if v.is_nan() => NumberValueKind::SpecialNumber(SpecialNumber::Nan),
            v if v == 0.0 && v.is_sign_negative() => {
                NumberValueKind::SpecialNumber(SpecialNumber::NegZero)
            },
            f64::INFINITY => NumberValueKind::SpecialNumber(SpecialNumber::Infinity),
            f64::NEG_INFINITY => NumberValueKind::SpecialNumber(SpecialNumber::NegInfinity),
            _ => NumberValueKind::Number(value),
        };
        // Step 2.number.2.
        remote_value = Some(PrimitiveProtocolValue::Number(NumberValue {
            value: serialized,
        }));
    }

    // Step 2.boolean
    if value.is_boolean() {
        let value = value.to_boolean();
        remote_value = Some(PrimitiveProtocolValue::Boolean(BooleanValue { value }));
    }

    // Step 2.bigint
    if value.is_bigint() {
        let value = value.to_bigint();
        rooted!(&in(cx) let value = value);
        let Some(jsstr) = NonNull::new(unsafe { BigIntToString(cx, value.handle(), 10) }) else {
            return Err(ErrorCode::UnknownError);
        };

        let value = unsafe { jsstr_to_string(cx, jsstr) };
        remote_value = Some(PrimitiveProtocolValue::Bigint(BigIntValue { value }));
    }

    // Step 3. return
    Ok(remote_value)
}

trait ArrayLike {
    fn new_with_handle(handle_id: Option<HandleId>) -> Self;
    fn value_mut(&mut self) -> &mut Option<Vec<PartialRemoteValue>>;
    fn into_value(self) -> PartialRemoteValue;
}

macro_rules! impl_arraylike {
    ($name:ident, $variant:ident) => {
        impl ArrayLike for $name {
            fn new_with_handle(handle_id: Option<HandleId>) -> Self {
                Self {
                    handle: handle_id,
                    internal_id: None,
                    value: None,
                }
            }
            fn value_mut(&mut self) -> &mut Option<Vec<PartialRemoteValue>> {
                &mut self.value
            }
            fn into_value(self) -> PartialRemoteValue {
                PartialRemoteValue::$variant(Rc::new(RefCell::new(self)))
            }
        }
    };
}

impl_arraylike!(PartialArrayRemoteValue, Array);
impl_arraylike!(PartialNodeListRemoteValue, NodeList);
impl_arraylike!(PartialHtmlCollectionRemoteValue, HtmlCollection);

/// See <https://www.w3.org/TR/webdriver-bidi/#serialize-an-array-like>.
fn serialize_an_array_like<T: ArrayLike>(
    realm: &mut CurrentRealm,
    handle_id: Option<HandleId>,
    known_object: bool,
    value: HandleValue,
    serialization_options: SerializationOptions,
    ownership_type: ResultOwnership,
    serialization_internal_map: &mut InternalMap,
) -> Result<PartialRemoteValue, ErrorCode> {
    // Step 1. production
    let mut remote_value = T::new_with_handle(handle_id);
    // Step 3.
    if !known_object && serialization_options.max_object_depth != Some(0) {
        // Step 3.1. serialize as a list
        let serialized = serialize_as_a_list(
            realm,
            value,
            serialization_options,
            ownership_type,
            serialization_internal_map,
        )?;
        // Step 3.2. set field
        *remote_value.value_mut() = Some(serialized);
    }
    // Step 2. set internal id
    let remote_value = remote_value.into_value();
    set_internal_ids_if_needed(
        serialization_internal_map,
        remote_value.clone(),
        value.get(),
    );
    Ok(remote_value)
}

/// See <https://www.w3.org/TR/webdriver-bidi/#serialize-as-a-list>.
#[expect(unsafe_code)]
fn serialize_as_a_list(
    realm: &mut CurrentRealm,
    iterable: HandleValue,
    serialization_options: SerializationOptions,
    ownership_type: ResultOwnership,
    serialization_internal_map: &mut InternalMap,
) -> Result<PartialListRemoteValue, ErrorCode> {
    // Step 1. check options
    if let Some(max_object_depth) = &serialization_options.max_object_depth {
        debug_assert!(*max_object_depth > 0);
    }
    // Step 2. let serialized
    let mut serialized = vec![];

    // Step 3. for each child value
    for_of(unsafe { realm.as_mut().raw_cx() }, iterable, |item| {
        // TODO:
        // Step 3.1. clone child serialization options
        let mut child_serialization_options = serialization_options.clone();
        // Step 3.2. sub maxObjectDepth
        if let Some(max_object_depth) = &mut child_serialization_options.max_object_depth {
            *max_object_depth = max_object_depth.saturating_sub(1);
        }
        // Step 3.3. serialize child
        let serialized_child = serialize_as_a_remote_value_partial(
            realm,
            item,
            child_serialization_options,
            ownership_type,
            serialization_internal_map,
        )?;
        // Step 3.4. append
        serialized.push(serialized_child);

        Ok(ControlFlow::Continue(()))
    })
    .map_err(|_| ErrorCode::UnknownError)?;

    Ok(serialized)
}

/// See <https://www.w3.org/TR/webdriver-bidi/#serialize-as-a-mapping>.
#[expect(unsafe_code)]
fn serialize_as_a_mapping(
    cx: &mut CurrentRealm,
    iterable: HandleValue,
    serialization_options: SerializationOptions,
    ownership_type: ResultOwnership,
    serialization_internal_map: &mut InternalMap,
) -> Result<PartialMappingRemoteValue, ErrorCode> {
    // Step 1. check options
    if let Some(max_object_depth) = &serialization_options.max_object_depth {
        debug_assert!(*max_object_depth > 0);
    }
    // Step 2. let serialized
    let mut serialized = vec![];

    macro_rules! report_and_err {
        () => {
            return Err(ErrorCode::UnknownError.into());
        };
    }

    // Step 3. iter
    for_of(unsafe { cx.as_mut().raw_cx() }, iterable, |item| {
        if !item.is_object() {
            return Err(ErrorCode::UnknownError.into());
        }
        rooted!(&in(cx) let item_obj = item.to_object());

        // Step 3.1. assert array
        let mut is_array = false;
        if !unsafe { IsArray(cx.as_mut(), item_obj.handle(), &mut is_array) } || !is_array {
            report_and_err!();
        }
        // Step 3.1. assert array
        let mut is_array = false;
        if !unsafe { IsArray(cx.as_mut(), item_obj.handle(), &mut is_array) } || !is_array {
            report_and_err!();
        }

        // Step 3.2 & 3.3. skip abstract step
        // Step 3.4. key and value
        rooted!(&in(cx) let mut key = UndefinedValue());
        if !unsafe { JS_GetElement(cx.as_mut(), item_obj.handle(), 0, key.handle_mut()) } {
            report_and_err!();
        }
        rooted!(&in(cx) let mut value = UndefinedValue());
        if !unsafe { JS_GetElement(cx.as_mut(), item_obj.handle(), 0, value.handle_mut()) } {
            report_and_err!();
        }
        // Step 3.5. clone child serialization options
        let mut child_serialization_options = serialization_options.clone();
        // Step 3.6. sub maxObjectDepth
        if let Some(max_object_depth) = &mut child_serialization_options.max_object_depth {
            *max_object_depth = max_object_depth.saturating_sub(1);
        }
        // Step 3.7. serialize key
        let serialized_key = if key.is_string() {
            PartialRemoteValueOrText::Text(serialize_string(cx, key.handle())?)
        } else {
            PartialRemoteValueOrText::Value(serialize_as_a_remote_value_partial(
                cx,
                key.handle(),
                serialization_options.clone(),
                ownership_type,
                serialization_internal_map,
            )?)
        };
        // Step 3.8. serialize value
        let serialized_value = serialize_as_a_remote_value_partial(
            cx,
            value.handle(),
            serialization_options.clone(),
            ownership_type,
            serialization_internal_map,
        )?;
        // Step 3.9.
        let serialized_child = (serialized_key, serialized_value);
        // Step 3.10. append
        serialized.push(serialized_child);

        Ok(ControlFlow::Continue(()))
    })
    .map_err(|_| ErrorCode::UnknownError)?;

    // Step 4. return
    Ok(serialized)
}

#[expect(unsafe_code)]
fn serialize_string(cx: &mut CurrentRealm, value: HandleValue) -> Result<String, ErrorCode> {
    let Some(jsstr) = NonNull::new(match value.is_string() {
        true => value.to_string(),
        false => unsafe { ToString(cx, value) },
    }) else {
        return Err(ErrorCode::UnknownError);
    };
    Ok(unsafe { jsstr_to_string(cx, jsstr) })
}

/// See <https://www.w3.org/TR/webdriver-bidi/#deserialize-local-value>.
fn deserialize_local_value(
    cx: &mut CurrentRealm,
    local_protocol_value: LocalValue,
    rval: MutableHandleValue,
) -> Result<(), ErrorCode> {
    match local_protocol_value {
        // Step 1. RemoteReference
        LocalValue::RemoteReference(val) => deserialize_remote_reference(cx, val, rval),
        // Step 2. Primitive
        LocalValue::PrimitiveProtocol(value) => {
            deserialize_primitive_protocol_value(cx, value, rval)
        },
        // Step 3. Channel
        LocalValue::Channel(val) => create_a_channel(cx, &val, rval),
        // Step 6.array.
        LocalValue::Array(value) => deserialize_value_list(cx, value.value, rval, false),
        // Step 6.date.
        LocalValue::Date(value) => {
            rooted_vec!(let mut args);
            rooted!(&in(cx) let mut s = UndefinedValue());
            deserialize_string(cx, &value.value, s.handle_mut().into())?;
            args.push(s.get());
            constructor_or_static_method(cx, JSProtoKey::JSProto_Date, None, (&args).into(), rval)
        },
        // Step 6.map.
        LocalValue::Map(value) => deserialize_key_value_list(cx, value.value, rval, true),
        // Step 6.object.
        LocalValue::Object(value) => deserialize_key_value_list(cx, value.value, rval, false),
        // Step 6.regexp.
        LocalValue::Regexp(value) => {
            let RegExpValue { pattern, flags } = value.value;
            rooted_vec!(let mut args);
            // Step 6.regexp.1. pattern
            rooted!(&in(cx) let mut pattern_rval = UndefinedValue());
            deserialize_string(cx, &pattern, pattern_rval.handle_mut().into())?;
            args.push(pattern_rval.get());
            // Step 6.regexp.1. flags
            if let Some(flags) = flags {
                rooted!(&in(cx) let mut flags_rval = UndefinedValue());
                deserialize_string(cx, &flags, flags_rval.handle_mut().into())?;
                args.push(pattern_rval.get());
            } else {
                args.push(UndefinedValue());
            }
            let args = HandleValueArray::from(&args);
            // Step 6.regexp.3.
            constructor_or_static_method(cx, JSProtoKey::JSProto_RegExp, None, args, rval)
        },
        // Step 6.set.
        LocalValue::Set(value) => deserialize_value_list(cx, value.value, rval, true),
    }
}

/// See <https://www.w3.org/TR/webdriver-bidi/#deserialize-primitive-protocol-value>.
#[expect(unsafe_code)]
pub(crate) fn deserialize_primitive_protocol_value(
    cx: &mut CurrentRealm,
    primitive_protocol_value: PrimitiveProtocolValue,
    mut rval: MutableHandleValue,
) -> Result<(), ErrorCode> {
    // Step 4.
    match primitive_protocol_value {
        PrimitiveProtocolValue::Undefined => {
            rval.set(UndefinedValue());
        },
        PrimitiveProtocolValue::Null => {
            rval.set(NullValue());
        },
        PrimitiveProtocolValue::String(value) => {
            deserialize_string(cx, &value.value, rval)?;
        },
        PrimitiveProtocolValue::Number(value) => {
            let value = match value.value {
                NumberValueKind::Number(value) => jsval::DoubleValue(value),
                NumberValueKind::SpecialNumber(special_number) => {
                    jsval::DoubleValue(match special_number {
                        SpecialNumber::Nan => f64::NAN,
                        SpecialNumber::NegZero => -0.0,
                        SpecialNumber::Infinity => f64::INFINITY,
                        SpecialNumber::NegInfinity => f64::NEG_INFINITY,
                    })
                },
            };
            rval.set(value);
        },
        PrimitiveProtocolValue::Boolean(value) => {
            rval.set(jsval::BooleanValue(value.value));
        },
        PrimitiveProtocolValue::Bigint(value) => {
            let s = js::conversions::Utf8Chars::from(&*value.value);
            rooted!(&in(cx) let mut val = UndefinedValue());
            let Some(jsstr) = NonNull::new(unsafe { JS_NewStringCopyUTF8N(cx.as_mut(), &*s) })
            else {
                return Err(ErrorCode::InvalidArgument);
            };
            val.set(jsval::StringValue(unsafe { jsstr.as_ref() }));
            if let Some(bigint) = NonNull::new(unsafe { ToBigInt(cx.as_mut(), val.handle()) }) {
                rval.set(jsval::BigIntValue(unsafe { bigint.as_ref() }));
            } else {
                return Err(ErrorCode::InvalidArgument);
            }
        },
    }
    Ok(())
}

/// See <https://www.w3.org/TR/webdriver-bidi/#deserialize-remote-reference>.
pub(crate) fn deserialize_remote_reference(
    realm: &mut CurrentRealm,
    local_protocol_value: RemoteReference,
    rval: MutableHandleValue,
) -> Result<(), ErrorCode> {
    match local_protocol_value {
        // Step 2.
        RemoteReference::RemoteObject(remote) => {
            deserialize_remote_object_reference(realm, remote, rval)
        },
        // Step 3.
        RemoteReference::Shared(shared) => deserialize_shared_reference(realm, shared, rval),
    }
}

/// See <https://www.w3.org/TR/webdriver-bidi/#deserialize-remote-object-reference>.
pub(crate) fn deserialize_remote_object_reference(
    realm: &mut CurrentRealm,
    remote_object_reference: RemoteObjectReference,
    rval: MutableHandleValue,
) -> Result<(), ErrorCode> {
    // Step 1.
    let handle_id = remote_object_reference.handle;
    // Step 2. get handle map
    let global_scope = GlobalScope::from_current_realm(realm);
    if !global_scope.get_handle(&handle_id, rval) {
        // Step 3.
        return Err(ErrorCode::NoSuchHandle);
    }
    // Step 4. return success
    Ok(())
}

/// See <https://www.w3.org/TR/webdriver-bidi/#deserialize-shared-reference>.
pub(crate) fn deserialize_shared_reference(
    _realm: &mut CurrentRealm,
    _shared_reference: SharedReference,
    mut _rval: MutableHandleValue,
) -> Result<(), ErrorCode> {
    // TODO: shared reference is not implemented in this version
    Err(ErrorCode::UnknownError)
}

/// See <https://www.w3.org/TR/webdriver-bidi/#deserialize-key-value-list>.
#[expect(unsafe_code)]
pub(crate) fn deserialize_key_value_list(
    cx: &mut CurrentRealm,
    serialized_key_value_list: MappingLocalValue,
    rval: MutableHandleValue,
    map_or_object: bool,
) -> Result<(), ErrorCode> {
    // Step 1.
    rooted_vec!(let mut deserialized_key_value_list);
    // Step 2. for each "serialized key-value"
    for serialzed_key_value in serialized_key_value_list {
        // Step 2.1. skip assert size
        // Step 2.2.
        let serialized_key = serialzed_key_value.0;
        // Step 2.3.
        rooted!(&in(cx) let mut deserialized_key = UndefinedValue());
        // Step 2.4.
        match serialized_key {
            LocalValueOrText::LocalValue(key) => {
                deserialize_local_value(cx, key, deserialized_key.handle_mut().into())?
            },
            LocalValueOrText::Text(s) => {
                deserialize_string(cx, &s, deserialized_key.handle_mut().into())?
            },
        };
        // Step 2.5.
        let serialized_value = serialzed_key_value.1;
        // Step 2.6.
        rooted!(&in(cx) let mut deserialized_value = UndefinedValue());
        deserialize_local_value(cx, serialized_value, deserialized_value.handle_mut().into())?;
        // Step 2.7.
        rooted_vec!(let mut pair);
        pair.push(deserialized_key.get());
        pair.push(deserialized_value.get());
        rooted!(&in(cx) let array_obj = unsafe { NewArrayObject(cx.as_mut(), &(&pair).into()) });
        if array_obj.is_null() {
            return Err(ErrorCode::UnknownError);
        }
        rooted!(&in(cx) let array = jsval::ObjectValue(array_obj.get()));
        deserialized_key_value_list.push(array.get());
    }
    // actual construct
    rooted_vec!(let mut args);
    rooted!(&in(cx) let arg0_obj = unsafe { NewArrayObject(cx.as_mut(), &(&deserialized_key_value_list).into()) });
    if arg0_obj.is_null() {
        return Err(ErrorCode::UnknownError);
    }
    rooted!(&in(cx) let arg0 = jsval::ObjectValue(arg0_obj.get()));
    args.push(arg0.get());
    match map_or_object {
        // map => new Map
        true => {
            constructor_or_static_method(cx, JSProtoKey::JSProto_Map, None, (&args).into(), rval)
        },
        // object => Object.fromEntries
        false => constructor_or_static_method(
            cx,
            JSProtoKey::JSProto_Object,
            Some(c"fromEntries"),
            (&args).into(),
            rval,
        ),
    }
}

/// See <https://www.w3.org/TR/webdriver-bidi/#deserialize-value-list>.
#[expect(unsafe_code)]
pub(crate) fn deserialize_value_list(
    cx: &mut CurrentRealm,
    serialized_value_list: ListLocalValue,
    mut rval: MutableHandleValue,
    set_or_array: bool,
) -> Result<(), ErrorCode> {
    // Step 1.
    rooted_vec!(let mut deserialized_values);
    // Step 2. for each "serialized value"
    for serialized_value in serialized_value_list {
        rooted!(&in(cx) let mut rval = UndefinedValue());
        deserialize_local_value(cx, serialized_value, rval.handle_mut())?;
        deserialized_values.push(rval.get());
    }
    // Step 3.
    // array
    rooted!(&in(cx) let array_obj = unsafe { NewArrayObject(cx.as_mut(), &(&deserialized_values).into()) });
    if array_obj.is_null() {
        return Err(ErrorCode::UnknownError);
    }
    if !set_or_array {
        rval.set(jsval::ObjectValue(array_obj.get()));
        Ok(())
    }
    // set
    else {
        rooted_vec!(let mut args);
        rooted!(&in(cx) let array = jsval::ObjectValue(array_obj.get()));
        args.push(array.get());
        constructor_or_static_method(cx, JSProtoKey::JSProto_Set, None, (&args).into(), rval)
    }
}

/// See <https://www.w3.org/TR/webdriver-bidi/#create-a-channel>.
#[expect(unsafe_code)]
pub(crate) fn create_a_channel(
    cx: &mut CurrentRealm,
    protocol_value: &ChannelValue,
    mut rval: MutableHandleValue,
) -> Result<(), ErrorCode> {
    // create native function
    let func =
        unsafe { NewFunctionWithReserved(cx.as_mut(), Some(channel_inner), 1, 0, c"".as_ptr()) };
    if func.is_null() {
        return Err(ErrorCode::UnknownError);
    }

    // serialize channel properties
    rooted!(&in(cx) let mut channel = UndefinedValue());
    serialize_channel_properties(cx, &protocol_value.value, channel.handle_mut())?;

    // store channel properties
    rooted!(&in(cx) let func_obj = unsafe { JS_GetFunctionObject(func) });
    unsafe {
        SetFunctionNativeReserved(func_obj.get(), 0, &channel.get());
    };

    rval.set(jsval::ObjectValue(func_obj.get()));
    Ok(())
}

#[expect(unsafe_code)]
unsafe extern "C" fn channel_inner(
    cx: *mut js::jsapi::JSContext,
    argc: u32,
    vp: *mut Value,
) -> bool {
    let mut result = false;
    wrap_panic(&mut || {
        result = (|| {
            let args = unsafe { CallArgs::from_vp(vp, argc) };
            let callee = args.callee();
            let global = unsafe { GlobalScope::from_object(callee) };

            let mut cx = unsafe { JSContext::from_ptr(ptr::NonNull::new(cx).unwrap()) };
            let mut auto_realm = enter_auto_realm(&mut cx, &*global);
            let mut realm = auto_realm.current_realm();

            if argc != 1 {
                unsafe {
                    ReportErrorASCII(
                        realm.as_mut(),
                        c"Channel function can only be called one argument".as_ptr(),
                    )
                };
                return false;
            }

            let reserved = unsafe { GetFunctionNativeReserved(callee, 0) as *mut Value };
            if reserved.is_null() {
                report_pending_exception(&mut realm);
                return false;
            }
            let Ok(channel_properties) = deserialize_channel_properties(&mut realm, reserved)
            else {
                report_pending_exception(&mut realm);
                return false;
            };

            let arg0 = args.get(0);
            let Ok(serialized) = serialize_as_a_remote_value(
                &mut realm,
                unsafe { HandleValue::from_raw(arg0) },
                channel_properties.serialization_options.unwrap_or_default(),
                channel_properties
                    .ownership
                    .unwrap_or(ResultOwnership::None),
            ) else {
                unsafe {
                    ReportErrorASCII(realm.as_mut(), c"Serializing remote value failed".as_ptr())
                };
                return false;
            };

            if let Some(chan) = global.webdriver_chan() {
                _ = chan.send(ScriptToWebDriverMessage::Message(MessageBody {
                    channel: channel_properties.channel.clone(),
                    data: serialized,
                    realm: global.realm_id(),
                }));
            }

            return true;
        })()
    });
    result
}

#[expect(unsafe_code)]
fn deserialize_string(
    cx: &mut CurrentRealm,
    s: &str,
    mut rval: MutableHandleValue,
) -> Result<(), ErrorCode> {
    let s = js::conversions::Utf8Chars::from(s);
    if let Some(jsstr) = NonNull::new(unsafe { JS_NewStringCopyUTF8N(cx.as_mut(), &*s) }) {
        rval.set(jsval::StringValue(unsafe { jsstr.as_ref() }));
        Ok(())
    } else {
        Err(ErrorCode::InvalidArgument)
    }
}

/// Construct a builtin class with either constructor or static method.
#[expect(unsafe_code)]
fn constructor_or_static_method(
    cx: &mut CurrentRealm,
    proto: JSProtoKey,
    // new or other static method
    method: Option<&CStr>,
    args: HandleValueArray,
    mut rval: MutableHandleValue,
) -> Result<(), ErrorCode> {
    rooted!(&in(cx) let mut ctor_obj = null_mut::<JSObject>());
    if !unsafe { JS_GetClassObject(cx.as_mut(), proto, ctor_obj.handle_mut()) } {
        return Err(ErrorCode::UnknownError);
    }
    match method {
        // new constructor
        None => {
            rooted!(&in(cx) let ctor = jsval::ObjectValue(ctor_obj.get()));
            rooted!(&in(cx) let mut obj = null_mut::<JSObject>());
            if !unsafe { Construct1(cx.as_mut(), ctor.handle(), &args, obj.handle_mut()) } {
                return Err(ErrorCode::InvalidArgument);
            }
            rval.set(jsval::ObjectValue(obj.get()));
        },
        // static method, e.g. Object.fromEntries
        Some(method) => {
            if !unsafe {
                JS_CallFunctionName(cx.as_mut(), ctor_obj.handle(), method.as_ptr(), &args, rval)
            } {
                return Err(ErrorCode::UnknownError);
            }
        },
    }
    Ok(())
}

/// Serialize [`ChannelProperties`] into js object so that we can
/// store that in reserved slot of function.
#[expect(unsafe_code)]
fn serialize_channel_properties(
    realm: &mut CurrentRealm,
    channel: &ChannelProperties,
    mut rval: MutableHandleValue,
) -> Result<(), ErrorCode> {
    rooted!(&in(realm) let obj = unsafe { JS_NewPlainObject(realm) });
    if obj.is_null() {
        return Err(ErrorCode::UnknownError);
    }
    // channel
    rooted!(&in(realm) let mut channel_id = UndefinedValue());
    deserialize_string(realm, &channel.channel, channel_id.handle_mut())?;
    if !unsafe {
        JS_DefineProperty(
            realm.as_mut(),
            obj.handle(),
            c"channel".as_ptr(),
            channel_id.handle(),
            0,
        )
    } {
        return Err(ErrorCode::UnknownError);
    }

    if let Some(serialization_options) = &channel.serialization_options {
        macro_rules! store_u32_field {
            ($name:ident, $cstr:literal) => {
                if let Some($name) = serialization_options.$name{
                    rooted!(&in(realm) let mut ownership = jsval::UInt32Value($name as u32));
                    if !unsafe {
                        JS_DefineProperty(
                            realm.as_mut(),
                            obj.handle(),
                            $cstr.as_ptr(),
                            ownership.handle(),
                            0,
                        )
                    } {
                        return Err(ErrorCode::UnknownError);
                    }
                }
            };
        }
        store_u32_field!(max_dom_depth, c"max_dom_depth");
        store_u32_field!(max_object_depth, c"max_object_depth");
        if let Some(include_shadow_tree) = serialization_options.include_shadow_tree {
            let val = match include_shadow_tree {
                IncludeShadowTree::None => 0,
                IncludeShadowTree::Open => 1,
                IncludeShadowTree::All => 2,
            };
            rooted!(&in(realm) let mut ownership = jsval::UInt32Value(val));
            if !unsafe {
                JS_DefineProperty(
                    realm.as_mut(),
                    obj.handle(),
                    c"include_shadow_tree".as_ptr(),
                    ownership.handle(),
                    0,
                )
            } {
                return Err(ErrorCode::UnknownError);
            }
        }
    }

    // ownership
    if let Some(ownership) = channel.ownership {
        let value = match ownership {
            ResultOwnership::Root => true,
            ResultOwnership::None => false,
        };
        rooted!(&in(realm) let mut ownership = jsval::BooleanValue(value));
        if !unsafe {
            JS_DefineProperty(
                realm.as_mut(),
                obj.handle(),
                c"ownership".as_ptr(),
                ownership.handle(),
                0,
            )
        } {
            return Err(ErrorCode::UnknownError);
        }
    }

    rval.set(ObjectValue(obj.get()));
    Ok(())
}

/// Deserialize js object into [`ChannelProperties`] from function's
/// reserved slot.
#[expect(unsafe_code)]
fn deserialize_channel_properties(
    cx: &mut CurrentRealm,
    value: *const Value,
) -> Result<ChannelProperties, ErrorCode> {
    let Some(value) = NonNull::new(value as *mut Value) else {
        return Err(ErrorCode::UnknownError);
    };
    let jsval = unsafe { value.as_ref() };
    if !jsval.is_object() {
        return Err(ErrorCode::UnknownError);
    }

    rooted!(&in(cx) let obj = jsval.to_object());

    // channel
    rooted!(&in(cx) let mut channel_val = UndefinedValue());
    if !unsafe {
        JS_GetProperty(
            cx,
            obj.handle(),
            c"channel".as_ptr(),
            channel_val.handle_mut(),
        )
    } || !channel_val.is_string()
    {
        return Err(ErrorCode::UnknownError);
    }
    let channel = serialize_string(cx, channel_val.handle())?;

    // serialization_options
    let mut serialization_options = SerializationOptions::default();
    macro_rules! load_u32_field {
        ($name:ident, $cstr:literal) => {
            rooted!(&in(cx) let mut val = UndefinedValue());
            if unsafe { JS_GetProperty(cx, obj.handle(), $cstr.as_ptr(), val.handle_mut()) }
                && val.is_number()
            {
                serialization_options.$name = Some(val.to_int32() as u64);
            }
        };
    }
    load_u32_field!(max_dom_depth, c"max_dom_depth");
    load_u32_field!(max_object_depth, c"max_object_depth");

    rooted!(&in(cx) let mut val = UndefinedValue());
    if unsafe {
        JS_GetProperty(
            cx,
            obj.handle(),
            c"include_shadow_tree".as_ptr(),
            val.handle_mut(),
        )
    } && val.is_number()
    {
        let include_shadow_tree = &mut serialization_options.include_shadow_tree;
        match val.to_int32() {
            0 => *include_shadow_tree = Some(IncludeShadowTree::None),
            1 => *include_shadow_tree = Some(IncludeShadowTree::Open),
            2 => *include_shadow_tree = Some(IncludeShadowTree::All),
            _ => {},
        }
    }

    rooted!(&in(cx) let mut ownership_val = UndefinedValue());
    let ownership = if unsafe {
        JS_GetProperty(
            cx,
            obj.handle(),
            c"ownership".as_ptr(),
            ownership_val.handle_mut(),
        )
    } && ownership_val.is_boolean()
    {
        Some(match ownership_val.to_boolean() {
            true => ResultOwnership::Root,
            false => ResultOwnership::None,
        })
    } else {
        None
    };

    Ok(ChannelProperties {
        channel,
        serialization_options: Some(serialization_options),
        ownership,
    })
}

// p => ToString(Get(value, p))
#[expect(unsafe_code)]
fn serialize_property_string(
    cx: &mut JSContext,
    obj: HandleObject,
    p: &CStr,
) -> Result<Option<String>, ErrorCode> {
    rooted!(&in(cx) let mut out = UndefinedValue());
    if !unsafe { JS_GetProperty(cx, obj, p.as_ptr(), out.handle_mut()) } {
        return Err(ErrorCode::UnknownError);
    }
    if out.is_undefined() {
        return Ok(None);
    }
    let jsstr =
        NonNull::new(unsafe { ToString(cx, out.handle()) }).ok_or(ErrorCode::UnknownError)?;
    Ok(Some(unsafe { jsstr_to_string(cx, jsstr) }))
}

type InternalMap = HashMap<u64, PartialRemoteValue>;

/// See <https://www.w3.org/TR/webdriver-bidi/#set-internal-ids-if-needed>.
fn set_internal_ids_if_needed(
    serialization_internal_map: &mut InternalMap,
    remote_value: PartialRemoteValue,
    value: Value,
) {
    let obj_id = value.asBits_;
    match serialization_internal_map.get(&obj_id) {
        // Step 1. set internal ids
        None => {
            serialization_internal_map.insert(obj_id, remote_value);
        },
        // Step 2. otherwise
        Some(previously_serialized_remote_value) => {
            // Step 2.2. new internal id
            let internal_id = InternalId::new();
            // Step 2.3. set internal id of previously serialized
            previously_serialized_remote_value.set_internal_id(internal_id);
        },
    }
}

/// The "serialize as a remote value" algorith is described
/// in a JavaScript manner, with shared ownership and mutability.
/// while in Rust we need to do that explicitly.
#[derive(Clone)]
enum PartialRemoteValue {
    Symbol(Rc<RefCell<SymbolRemoteValue>>),
    Array(Rc<RefCell<PartialArrayRemoteValue>>),
    Object(Rc<RefCell<PartialObjectRemoteValue>>),
    Function(Rc<RefCell<FunctionRemoteValue>>),
    RegExp(Rc<RefCell<RegExpRemoteValue>>),
    Date(Rc<RefCell<DateRemoteValue>>),
    Map(Rc<RefCell<PartialMapRemoteValue>>),
    Set(Rc<RefCell<PartialSetRemoteValue>>),
    WeakMap(Rc<RefCell<WeakMapRemoteValue>>),
    WeakSet(Rc<RefCell<WeakSetRemoteValue>>),
    Generator(Rc<RefCell<GeneratorRemoteValue>>),
    Error(Rc<RefCell<ErrorRemoteValue>>),
    Proxy(Rc<RefCell<ProxyRemoteValue>>),
    Promise(Rc<RefCell<PromiseRemoteValue>>),
    TypedArray(Rc<RefCell<TypedArrayRemoteValue>>),
    ArrayBuffer(Rc<RefCell<ArrayBufferRemoteValue>>),
    NodeList(Rc<RefCell<PartialNodeListRemoteValue>>),
    HtmlCollection(Rc<RefCell<PartialHtmlCollectionRemoteValue>>),
    Node(Rc<RefCell<PartialNodeRemoteValue>>),
    Window(Rc<RefCell<WindowProxyRemoteValue>>),
    PrimitiveProtocol(PrimitiveProtocolValue),
}

type PartialListRemoteValue = Vec<PartialRemoteValue>;

type PartialMappingRemoteValue = Vec<(PartialRemoteValueOrText, PartialRemoteValue)>;

enum PartialRemoteValueOrText {
    Value(PartialRemoteValue),
    Text(String),
}

macro_rules! define_partial {
    ($name:ident, $items:ty) => {
        struct $name {
            handle: Option<HandleId>,
            internal_id: Option<InternalId>,
            value: Option<$items>,
        }
    };
}

define_partial!(PartialArrayRemoteValue, PartialListRemoteValue);
define_partial!(PartialSetRemoteValue, PartialListRemoteValue);
define_partial!(PartialNodeListRemoteValue, PartialListRemoteValue);
define_partial!(PartialHtmlCollectionRemoteValue, PartialListRemoteValue);
define_partial!(PartialObjectRemoteValue, PartialMappingRemoteValue);
define_partial!(PartialMapRemoteValue, PartialMappingRemoteValue);

struct PartialNodeRemoteValue {
    shared_id: Option<SharedId>,
    handle: Option<HandleId>,
    internal_id: Option<InternalId>,
    value: Option<PartialNodeProperties>,
}

#[derive(Default)]
pub struct PartialNodeProperties {
    node_type: u16,
    child_node_count: u32,
    attributes: Option<HashMap<String, String>>,
    children: Option<Vec<Rc<RefCell<PartialNodeRemoteValue>>>>,
    local_name: Option<String>,
    mode: Option<script::ShadowRootMode>,
    namespace_uri: Option<String>,
    node_value: Option<String>,
    shadow_root: Option<Rc<RefCell<PartialNodeRemoteValue>>>,
}

impl PartialRemoteValue {
    fn set_internal_id(&self, id: InternalId) {
        macro_rules! match_internal_id {
            ($($name:ident,)*) => {
                match self {
                    $( Self::$name(val) => val.borrow_mut().internal_id = Some(id), )*
                    _ => { },
                }
            };
        }
        match_internal_id! {
            Symbol, Array, Object, Function, RegExp, Date,
            Map, Set, WeakMap, WeakSet, Generator, Error,
            Proxy, Promise, TypedArray, ArrayBuffer, NodeList,
            HtmlCollection, Node, Window,
        }
    }
}

impl PartialRemoteValue {
    fn to_remote_value(self) -> Result<RemoteValue, ErrorCode> {
        macro_rules! match_inner {
            (
                simple: [$($sname:ident,)*],
                list: {$($lname:ident:$lty:ident,)*},
                map: {$($mname:ident:$mty:ident,)*},
            ) => {
                Ok(match self {
                    // simple
                    $(
                        PartialRemoteValue::$sname(val) =>
                            RemoteValue::$sname(
                                Rc::into_inner(val)
                                    .ok_or(ErrorCode::UnknownError)?
                                    .into_inner()
                            ),
                    )*
                    // list
                    $(
                        PartialRemoteValue::$lname(val) => {
                            let val = Rc::into_inner(val)
                                .ok_or(ErrorCode::UnknownError)?
                                .into_inner();
                            RemoteValue::$lname($lty {
                                handle: val.handle,
                                internal_id: val.internal_id,
                                value: val
                                    .value
                                    .map(|v| {
                                        v.into_iter()
                                            .map(PartialRemoteValue::to_remote_value)
                                            .collect()
                                    })
                                    .transpose()?,
                            })
                        },
                    )*
                    // map
                    $(
                        PartialRemoteValue::$mname(val) => {
                            let val = Rc::into_inner(val)
                                .ok_or(ErrorCode::UnknownError)?
                                .into_inner();
                            RemoteValue::$mname($mty {
                                handle: val.handle,
                                internal_id: val.internal_id,
                                value: val
                                    .value
                                    .map(|v| {
                                        v.into_iter()
                                            .map(|(k, v)| Ok((
                                                k.to_remote_value()?,
                                                v.to_remote_value()?)
                                            ))
                                            .collect()
                                    })
                                    .transpose()?,
                            })
                        },
                    )*
                    // primitive, specially handled
                    PartialRemoteValue::PrimitiveProtocol(val) => {
                        RemoteValue::PrimitiveProtocol(val)
                    }
                    // node, specially handled
                    PartialRemoteValue::Node(val) => {
                        let val = Rc::into_inner(val)
                            .ok_or(ErrorCode::UnknownError)?
                            .into_inner();
                        RemoteValue::Node(val.to_remote_value()?)
                    }
                })
            }
        }
        match_inner! {
            simple: [
                Symbol, Function, RegExp, Date, WeakMap,
                WeakSet, Generator, Error, Proxy, Promise,
                TypedArray, ArrayBuffer, Window,
            ],
            list: {
                Array: ArrayRemoteValue,
                Set: SetRemoteValue,
                NodeList: NodeListRemoteValue,
                HtmlCollection: HtmlCollectionRemoteValue,
            },
            map: {
                Object: ObjectRemoteValue,
                Map: MapRemoteValue,
            },
        }
    }
}

impl PartialNodeRemoteValue {
    fn to_remote_value(self) -> Result<NodeRemoteValue, ErrorCode> {
        Ok(NodeRemoteValue {
            shared_id: self.shared_id,
            handle: self.handle,
            internal_id: self.internal_id,
            value: self
                .value
                .map(|p| {
                    Ok(NodeProperties {
                        node_type: p.node_type,
                        child_node_count: p.child_node_count,
                        attributes: p.attributes,
                        children: p
                            .children
                            .map(|children| {
                                children
                                    .into_iter()
                                    .map(|child| {
                                        Rc::into_inner(child)
                                            .ok_or(ErrorCode::UnknownError)?
                                            .into_inner()
                                            .to_remote_value()
                                    })
                                    .collect()
                            })
                            .transpose()?,
                        local_name: p.local_name,
                        mode: p.mode,
                        namespace_uri: p.namespace_uri,
                        node_value: p.node_value,
                        shadow_root: p
                            .shadow_root
                            .map(|r| {
                                Rc::into_inner(r)
                                    .ok_or(ErrorCode::UnknownError)?
                                    .into_inner()
                                    .to_remote_value()
                            })
                            .transpose()?
                            .map(Box::new),
                    })
                })
                .transpose()?,
        })
    }
}

impl PartialRemoteValueOrText {
    fn to_remote_value(self) -> Result<RemoteValueOrText, ErrorCode> {
        Ok(match self {
            PartialRemoteValueOrText::Value(k) => RemoteValueOrText::Value(k.to_remote_value()?),
            PartialRemoteValueOrText::Text(t) => RemoteValueOrText::Text(t),
        })
    }
}
