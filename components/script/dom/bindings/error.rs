/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Utilities to throw exceptions from Rust bindings.

use dom::bindings::codegen::Bindings::DOMExceptionBinding::DOMExceptionMethods;
use dom::bindings::codegen::PrototypeList::proto_id_to_name;
use dom::bindings::conversions::{ConversionResult, FromJSValConvertible, ToJSValConvertible};
use dom::bindings::conversions::root_from_object;
use dom::bindings::str::USVString;
use dom::domexception::{DOMErrorName, DOMException};
use dom::globalscope::GlobalScope;
use js::error::{throw_range_error, throw_type_error};
use js::jsapi;
use js::jsval::UndefinedValue;
use libc::c_uint;
use std::slice::from_raw_parts;

/// DOM exceptions that can be thrown by a native DOM method.
#[derive(Debug, Clone, HeapSizeOf)]
pub enum Error {
    /// IndexSizeError DOMException
    IndexSize,
    /// NotFoundError DOMException
    NotFound,
    /// HierarchyRequestError DOMException
    HierarchyRequest,
    /// WrongDocumentError DOMException
    WrongDocument,
    /// InvalidCharacterError DOMException
    InvalidCharacter,
    /// NotSupportedError DOMException
    NotSupported,
    /// InUseAttributeError DOMException
    InUseAttribute,
    /// InvalidStateError DOMException
    InvalidState,
    /// SyntaxError DOMException
    Syntax,
    /// NamespaceError DOMException
    Namespace,
    /// InvalidAccessError DOMException
    InvalidAccess,
    /// SecurityError DOMException
    Security,
    /// NetworkError DOMException
    Network,
    /// AbortError DOMException
    Abort,
    /// TimeoutError DOMException
    Timeout,
    /// InvalidNodeTypeError DOMException
    InvalidNodeType,
    /// DataCloneError DOMException
    DataClone,
    /// NoModificationAllowedError DOMException
    NoModificationAllowed,
    /// QuotaExceededError DOMException
    QuotaExceeded,
    /// TypeMismatchError DOMException
    TypeMismatch,
    /// InvalidModificationError DOMException
    InvalidModification,

    /// TypeError JavaScript Error
    Type(String),
    /// RangeError JavaScript Error
    Range(String),

    /// A JavaScript exception is already pending.
    JSFailed,
}

/// The return type for IDL operations that can throw DOM exceptions.
pub type Fallible<T> = Result<T, Error>;

/// The return type for IDL operations that can throw DOM exceptions and
/// return `()`.
pub type ErrorResult = Fallible<()>;

/// Set a pending exception for the given `result` on `cx`.
pub unsafe fn throw_dom_exception(cx: *mut jsapi::JSContext, global: &GlobalScope, result: Error) {
    let code = match result {
        Error::IndexSize => DOMErrorName::IndexSizeError,
        Error::NotFound => DOMErrorName::NotFoundError,
        Error::HierarchyRequest => DOMErrorName::HierarchyRequestError,
        Error::WrongDocument => DOMErrorName::WrongDocumentError,
        Error::InvalidCharacter => DOMErrorName::InvalidCharacterError,
        Error::NotSupported => DOMErrorName::NotSupportedError,
        Error::InUseAttribute => DOMErrorName::InUseAttributeError,
        Error::InvalidState => DOMErrorName::InvalidStateError,
        Error::Syntax => DOMErrorName::SyntaxError,
        Error::Namespace => DOMErrorName::NamespaceError,
        Error::InvalidAccess => DOMErrorName::InvalidAccessError,
        Error::Security => DOMErrorName::SecurityError,
        Error::Network => DOMErrorName::NetworkError,
        Error::Abort => DOMErrorName::AbortError,
        Error::Timeout => DOMErrorName::TimeoutError,
        Error::InvalidNodeType => DOMErrorName::InvalidNodeTypeError,
        Error::DataClone => DOMErrorName::DataCloneError,
        Error::NoModificationAllowed => DOMErrorName::NoModificationAllowedError,
        Error::QuotaExceeded => DOMErrorName::QuotaExceededError,
        Error::TypeMismatch => DOMErrorName::TypeMismatchError,
        Error::InvalidModification => DOMErrorName::InvalidModificationError,
        Error::Type(message) => {
            assert!(!jsapi::JS_IsExceptionPending(cx));
            throw_type_error(cx, &message);
            return;
        },
        Error::Range(message) => {
            assert!(!jsapi::JS_IsExceptionPending(cx));
            throw_range_error(cx, &message);
            return;
        },
        Error::JSFailed => {
            assert!(jsapi::JS_IsExceptionPending(cx));
            return;
        }
    };

    assert!(!jsapi::JS_IsExceptionPending(cx));
    let exception = DOMException::new(global, code);
    rooted!(in(cx) let mut thrown = UndefinedValue());
    exception.to_jsval(cx, thrown.handle_mut());
    jsapi::JS_SetPendingException(cx, thrown.handle());
}

/// A struct encapsulating information about a runtime script error.
pub struct ErrorInfo {
    /// The error message.
    pub message: String,
    /// The file name.
    pub filename: String,
    /// The line number.
    pub lineno: c_uint,
    /// The column number.
    pub column: c_uint,
}

impl ErrorInfo {
    unsafe fn from_native_error(cx: *mut jsapi::JSContext, object: jsapi::JS::HandleObject)
                                -> Option<ErrorInfo> {
        let report = jsapi::JS_ErrorFromException(cx, object);
        if report.is_null() {
            return None;
        }

        // TODO(fitzgen): Errors now have zero or more "notes", each of which
        // may have a filename, line, and column. However, these notes are
        // behind `UniquePtr`s that are too complicated for bindgen to
        // understand at the moment.
        let filename = "none".to_string();
        let lineno = 0;
        let column = 0;

        let message = {
            let message = (*report)._base.message_.data_;
            let length = (0..).find(|idx| *message.offset(*idx) == 0).unwrap();
            let message = from_raw_parts(message as _, length as usize);
            String::from_utf8_lossy(message)
        };

        Some(ErrorInfo {
            filename: filename,
            message: message.to_string(),
            lineno: lineno,
            column: column,
        })
    }

    fn from_dom_exception(object: jsapi::JS::HandleObject) -> Option<ErrorInfo> {
        let exception = match root_from_object::<DOMException>(object.get()) {
            Ok(exception) => exception,
            Err(_) => return None,
        };

        Some(ErrorInfo {
            filename: "".to_string(),
            message: exception.Stringifier().into(),
            lineno: 0,
            column: 0,
        })
    }
}

/// Report a pending exception, thereby clearing it.
///
/// The `dispatch_event` argument is temporary and non-standard; passing false
/// prevents dispatching the `error` event.
pub unsafe fn report_pending_exception(cx: *mut jsapi::JSContext, dispatch_event: bool) {
    if !jsapi::JS_IsExceptionPending(cx) { return; }

    rooted!(in(cx) let mut value = UndefinedValue());
    if !jsapi::JS_GetPendingException(cx, value.handle_mut()) {
        jsapi::JS_ClearPendingException(cx);
        error!("Uncaught exception: jsapi::JS_GetPendingException failed");
        return;
    }

    jsapi::JS_ClearPendingException(cx);
    let error_info = if value.is_object() {
        rooted!(in(cx) let object = value.to_object());
        ErrorInfo::from_native_error(cx, object.handle())
            .or_else(|| ErrorInfo::from_dom_exception(object.handle()))
            .unwrap_or_else(|| {
                ErrorInfo {
                    message: format!("uncaught exception: unknown (can't convert to string)"),
                    filename: String::new(),
                    lineno: 0,
                    column: 0,
                }
            })
    } else {
        match USVString::from_jsval(cx, value.handle(), ()) {
            Ok(ConversionResult::Success(USVString(string))) => {
                ErrorInfo {
                    message: format!("uncaught exception: {}", string),
                    filename: String::new(),
                    lineno: 0,
                    column: 0,
                }
            },
            _ => {
                panic!("Uncaught exception: failed to stringify primitive");
            },
        }
    };

    error!("Error at {}:{}:{} {}",
           error_info.filename,
           error_info.lineno,
           error_info.column,
           error_info.message);

    if dispatch_event {
        GlobalScope::from_context(cx)
            .report_an_error(error_info, value.handle());
    }
}

/// Throw an exception to signal that a `jsapi::JS::Value` can not be converted to any of
/// the types in an IDL union type.
pub unsafe fn throw_not_in_union(cx: *mut jsapi::JSContext, names: &'static str) {
    assert!(!jsapi::JS_IsExceptionPending(cx));
    let error = format!("argument could not be converted to any of: {}", names);
    throw_type_error(cx, &error);
}

/// Throw an exception to signal that a `jsapi::JSObject` can not be converted to a
/// given DOM type.
pub unsafe fn throw_invalid_this(cx: *mut jsapi::JSContext, proto_id: u16) {
    debug_assert!(!jsapi::JS_IsExceptionPending(cx));
    let error = format!("\"this\" object does not implement interface {}.",
                        proto_id_to_name(proto_id));
    throw_type_error(cx, &error);
}

impl Error {
    /// Convert this error value to a JS value, consuming it in the process.
    pub unsafe fn to_jsval(self, cx: *mut jsapi::JSContext, global: &GlobalScope, rval: jsapi::JS::MutableHandleValue) {
        assert!(!jsapi::JS_IsExceptionPending(cx));
        throw_dom_exception(cx, global, self);
        assert!(jsapi::JS_IsExceptionPending(cx));
        assert!(jsapi::JS_GetPendingException(cx, rval));
        jsapi::JS_ClearPendingException(cx);
    }
}
