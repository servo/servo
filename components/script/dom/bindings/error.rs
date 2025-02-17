/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Utilities to throw exceptions from Rust bindings.

use std::slice::from_raw_parts;

#[cfg(feature = "js_backtrace")]
use backtrace::Backtrace;
use js::error::{throw_range_error, throw_type_error};
#[cfg(feature = "js_backtrace")]
use js::jsapi::StackFormat as JSStackFormat;
use js::jsapi::{ExceptionStackBehavior, JS_ClearPendingException, JS_IsExceptionPending};
use js::jsval::UndefinedValue;
use js::rust::wrappers::{JS_ErrorFromException, JS_GetPendingException, JS_SetPendingException};
use js::rust::{HandleObject, HandleValue, MutableHandleValue};
use libc::c_uint;

#[cfg(feature = "js_backtrace")]
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::PrototypeList::proto_id_to_name;
use crate::dom::bindings::conversions::{
    root_from_object, ConversionResult, FromJSValConvertible, ToJSValConvertible,
};
use crate::dom::bindings::str::USVString;
use crate::dom::domexception::{DOMErrorName, DOMException};
use crate::dom::globalscope::GlobalScope;
use crate::realms::InRealm;
use crate::script_runtime::{CanGc, JSContext as SafeJSContext};

#[cfg(feature = "js_backtrace")]
thread_local! {
    /// An optional stringified JS backtrace and stringified native backtrace from the
    /// the last DOM exception that was reported.
    static LAST_EXCEPTION_BACKTRACE: DomRefCell<Option<(Option<String>, String)>> = DomRefCell::new(None);
}

/// DOM exceptions that can be thrown by a native DOM method.
#[derive(Clone, Debug, MallocSizeOf)]
pub(crate) enum Error {
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
    /// NotReadableError DOMException
    NotReadable,
    /// DataError DOMException
    Data,
    /// OperationError DOMException
    Operation,

    /// TypeError JavaScript Error
    Type(String),
    /// RangeError JavaScript Error
    Range(String),

    /// A JavaScript exception is already pending.
    JSFailed,
}

/// The return type for IDL operations that can throw DOM exceptions.
pub(crate) type Fallible<T> = Result<T, Error>;

/// The return type for IDL operations that can throw DOM exceptions and
/// return `()`.
pub(crate) type ErrorResult = Fallible<()>;

/// Set a pending exception for the given `result` on `cx`.
pub(crate) fn throw_dom_exception(cx: SafeJSContext, global: &GlobalScope, result: Error) {
    #[cfg(feature = "js_backtrace")]
    unsafe {
        capture_stack!(in(*cx) let stack);
        let js_stack = stack.and_then(|s| s.as_string(None, JSStackFormat::Default));
        let rust_stack = Backtrace::new();
        LAST_EXCEPTION_BACKTRACE.with(|backtrace| {
            *backtrace.borrow_mut() = Some((js_stack, format!("{:?}", rust_stack)));
        });
    }

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
        Error::NotReadable => DOMErrorName::NotReadableError,
        Error::Data => DOMErrorName::DataError,
        Error::Operation => DOMErrorName::OperationError,
        Error::Type(message) => unsafe {
            assert!(!JS_IsExceptionPending(*cx));
            throw_type_error(*cx, &message);
            return;
        },
        Error::Range(message) => unsafe {
            assert!(!JS_IsExceptionPending(*cx));
            throw_range_error(*cx, &message);
            return;
        },
        Error::JSFailed => unsafe {
            assert!(JS_IsExceptionPending(*cx));
            return;
        },
    };

    unsafe {
        assert!(!JS_IsExceptionPending(*cx));
        let exception = DOMException::new(global, code, CanGc::note());
        rooted!(in(*cx) let mut thrown = UndefinedValue());
        exception.to_jsval(*cx, thrown.handle_mut());
        JS_SetPendingException(*cx, thrown.handle(), ExceptionStackBehavior::Capture);
    }
}

/// A struct encapsulating information about a runtime script error.
#[derive(Default)]
pub(crate) struct ErrorInfo {
    /// The error message.
    pub(crate) message: String,
    /// The file name.
    pub(crate) filename: String,
    /// The line number.
    pub(crate) lineno: c_uint,
    /// The column number.
    pub(crate) column: c_uint,
}

impl ErrorInfo {
    fn from_native_error(object: HandleObject, cx: SafeJSContext) -> Option<ErrorInfo> {
        let report = unsafe { JS_ErrorFromException(*cx, object) };
        if report.is_null() {
            return None;
        }

        let filename = {
            let filename = unsafe { (*report)._base.filename.data_ as *const u8 };
            if !filename.is_null() {
                let filename = unsafe {
                    let length = (0..).find(|idx| *filename.offset(*idx) == 0).unwrap();
                    from_raw_parts(filename, length as usize)
                };
                String::from_utf8_lossy(filename).into_owned()
            } else {
                "none".to_string()
            }
        };

        let lineno = unsafe { (*report)._base.lineno };
        let column = unsafe { (*report)._base.column._base };

        let message = {
            let message = unsafe { (*report)._base.message_.data_ as *const u8 };
            let message = unsafe {
                let length = (0..).find(|idx| *message.offset(*idx) == 0).unwrap();
                from_raw_parts(message, length as usize)
            };
            String::from_utf8_lossy(message).into_owned()
        };

        Some(ErrorInfo {
            filename,
            message,
            lineno,
            column,
        })
    }

    fn from_dom_exception(object: HandleObject, cx: SafeJSContext) -> Option<ErrorInfo> {
        let exception = match root_from_object::<DOMException>(object.get(), *cx) {
            Ok(exception) => exception,
            Err(_) => return None,
        };

        Some(ErrorInfo {
            filename: "".to_string(),
            message: exception.stringifier().into(),
            lineno: 0,
            column: 0,
        })
    }

    fn from_object(object: HandleObject, cx: SafeJSContext) -> Option<ErrorInfo> {
        if let Some(info) = ErrorInfo::from_native_error(object, cx) {
            return Some(info);
        }
        if let Some(info) = ErrorInfo::from_dom_exception(object, cx) {
            return Some(info);
        }
        None
    }

    fn from_value(value: HandleValue, cx: SafeJSContext) -> ErrorInfo {
        if value.is_object() {
            rooted!(in(*cx) let object = value.to_object());
            if let Some(info) = ErrorInfo::from_object(object.handle(), cx) {
                return info;
            }
        }

        match unsafe { USVString::from_jsval(*cx, value, ()) } {
            Ok(ConversionResult::Success(USVString(string))) => ErrorInfo {
                message: format!("uncaught exception: {}", string),
                filename: String::new(),
                lineno: 0,
                column: 0,
            },
            _ => {
                panic!("uncaught exception: failed to stringify primitive");
            },
        }
    }
}

/// Report a pending exception, thereby clearing it.
///
/// The `dispatch_event` argument is temporary and non-standard; passing false
/// prevents dispatching the `error` event.
pub(crate) fn report_pending_exception(
    cx: SafeJSContext,
    dispatch_event: bool,
    realm: InRealm,
    can_gc: CanGc,
) {
    unsafe {
        if !JS_IsExceptionPending(*cx) {
            return;
        }
    }
    rooted!(in(*cx) let mut value = UndefinedValue());

    unsafe {
        if !JS_GetPendingException(*cx, value.handle_mut()) {
            JS_ClearPendingException(*cx);
            error!("Uncaught exception: JS_GetPendingException failed");
            return;
        }

        JS_ClearPendingException(*cx);
    }
    let error_info = ErrorInfo::from_value(value.handle(), cx);

    error!(
        "Error at {}:{}:{} {}",
        error_info.filename, error_info.lineno, error_info.column, error_info.message
    );

    #[cfg(feature = "js_backtrace")]
    {
        LAST_EXCEPTION_BACKTRACE.with(|backtrace| {
            if let Some((js_backtrace, rust_backtrace)) = backtrace.borrow_mut().take() {
                if let Some(stack) = js_backtrace {
                    eprintln!("JS backtrace:\n{}", stack);
                }
                eprintln!("Rust backtrace:\n{}", rust_backtrace);
            }
        });
    }

    if dispatch_event {
        GlobalScope::from_safe_context(cx, realm).report_an_error(
            error_info,
            value.handle(),
            can_gc,
        );
    }
}

/// Throw an exception to signal that a `JSObject` can not be converted to a
/// given DOM type.
pub(crate) fn throw_invalid_this(cx: SafeJSContext, proto_id: u16) {
    debug_assert!(unsafe { !JS_IsExceptionPending(*cx) });
    let error = format!(
        "\"this\" object does not implement interface {}.",
        proto_id_to_name(proto_id)
    );
    unsafe { throw_type_error(*cx, &error) };
}

pub(crate) fn throw_constructor_without_new(cx: SafeJSContext, name: &str) {
    debug_assert!(unsafe { !JS_IsExceptionPending(*cx) });
    let error = format!("{} constructor: 'new' is required", name);
    unsafe { throw_type_error(*cx, &error) };
}

impl Error {
    /// Convert this error value to a JS value, consuming it in the process.
    #[allow(clippy::wrong_self_convention)]
    pub(crate) fn to_jsval(
        self,
        cx: SafeJSContext,
        global: &GlobalScope,
        rval: MutableHandleValue,
    ) {
        match self {
            Error::JSFailed => (),
            _ => unsafe { assert!(!JS_IsExceptionPending(*cx)) },
        }
        throw_dom_exception(cx, global, self);
        unsafe {
            assert!(JS_IsExceptionPending(*cx));
            assert!(JS_GetPendingException(*cx, rval));
            JS_ClearPendingException(*cx);
        }
    }
}
