/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Utilities to throw exceptions from Rust bindings.

#[cfg(feature = "js_backtrace")]
use std::cell::RefCell;
use std::slice::from_raw_parts;

#[cfg(feature = "js_backtrace")]
use backtrace::Backtrace;
use js::error::{throw_range_error, throw_type_error};
#[cfg(feature = "js_backtrace")]
use js::jsapi::StackFormat as JSStackFormat;
use js::jsapi::{
    ExceptionStackBehavior, JSContext, JS_ClearPendingException, JS_IsExceptionPending,
};
use js::jsval::UndefinedValue;
use js::rust::wrappers::{JS_ErrorFromException, JS_GetPendingException, JS_SetPendingException};
use js::rust::{HandleObject, HandleValue, MutableHandleValue};
use libc::c_uint;

use crate::codegen::DomTypes::DomTypes;
use crate::dom::bindings::codegen::PrototypeList::proto_id_to_name;
use crate::dom::bindings::conversions::{
    root_from_object, ConversionResult, FromJSValConvertible, ToJSValConvertible,
};
use crate::dom::bindings::str::USVString;
use crate::dom::bindings::utils::DomHelpers;
use crate::realms::InRealm;
use crate::script_runtime::{CanGc, JSContext as SafeJSContext};

#[cfg(feature = "js_backtrace")]
thread_local! {
    /// An optional stringified JS backtrace and stringified native backtrace from the
    /// the last DOM exception that was reported.
    pub static LAST_EXCEPTION_BACKTRACE: RefCell<Option<(Option<String>, String)>> = RefCell::new(None);
}

/// DOM exceptions that can be thrown by a native DOM method.
#[derive(Clone, Debug, MallocSizeOf)]
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
pub type Fallible<T> = Result<T, Error>;

/// The return type for IDL operations that can throw DOM exceptions and
/// return `()`.
pub type ErrorResult = Fallible<()>;

/// A struct encapsulating information about a runtime script error.
#[derive(Default)]
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
    unsafe fn from_native_error(object: HandleObject, cx: *mut JSContext) -> Option<ErrorInfo> {
        let report = JS_ErrorFromException(cx, object);
        if report.is_null() {
            return None;
        }

        let filename = {
            let filename = (*report)._base.filename.data_ as *const u8;
            if !filename.is_null() {
                let length = (0..).find(|idx| *filename.offset(*idx) == 0).unwrap();
                let filename = from_raw_parts(filename, length as usize);
                String::from_utf8_lossy(filename).into_owned()
            } else {
                "none".to_string()
            }
        };

        let lineno = (*report)._base.lineno;
        let column = (*report)._base.column._base;

        let message = {
            let message = (*report)._base.message_.data_ as *const u8;
            let length = (0..).find(|idx| *message.offset(*idx) == 0).unwrap();
            let message = from_raw_parts(message, length as usize);
            String::from_utf8_lossy(message).into_owned()
        };

        Some(ErrorInfo {
            filename,
            message,
            lineno,
            column,
        })
    }

    fn from_dom_exception<D: DomTypes>(
        object: HandleObject,
        cx: *mut JSContext,
    ) -> Option<ErrorInfo> {
        let exception = match root_from_object::<D::DOMException>(object.get(), cx) {
            Ok(exception) => exception,
            Err(_) => return None,
        };

        Some(ErrorInfo {
            filename: "".to_string(),
            message: <D as DomHelpers<D>>::DOMException_stringifier(&exception).into(),
            lineno: 0,
            column: 0,
        })
    }

    unsafe fn from_object<D: DomTypes>(
        object: HandleObject,
        cx: *mut JSContext,
    ) -> Option<ErrorInfo> {
        if let Some(info) = ErrorInfo::from_native_error(object, cx) {
            return Some(info);
        }
        if let Some(info) = ErrorInfo::from_dom_exception::<D>(object, cx) {
            return Some(info);
        }
        None
    }

    unsafe fn from_value<D: DomTypes>(value: HandleValue, cx: *mut JSContext) -> ErrorInfo {
        if value.is_object() {
            rooted!(in(cx) let object = value.to_object());
            if let Some(info) = ErrorInfo::from_object::<D>(object.handle(), cx) {
                return info;
            }
        }

        match USVString::from_jsval(cx, value, ()) {
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
pub unsafe fn report_pending_exception<D: crate::DomTypes>(
    cx: *mut JSContext,
    dispatch_event: bool,
    realm: InRealm,
    can_gc: CanGc,
) {
    if !JS_IsExceptionPending(cx) {
        return;
    }

    rooted!(in(cx) let mut value = UndefinedValue());
    if !JS_GetPendingException(cx, value.handle_mut()) {
        JS_ClearPendingException(cx);
        error!("Uncaught exception: JS_GetPendingException failed");
        return;
    }

    JS_ClearPendingException(cx);
    let error_info = ErrorInfo::from_value::<D>(value.handle(), cx);

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
        let global = <D as crate::DomHelpers<D>>::GlobalScope_from_context(cx, realm);
        D::GlobalScope_report_an_error(&global, error_info, value.handle(), can_gc);
    }
}

/// Throw an exception to signal that a `JSObject` can not be converted to a
/// given DOM type.
pub unsafe fn throw_invalid_this(cx: *mut JSContext, proto_id: u16) {
    debug_assert!(!JS_IsExceptionPending(cx));
    let error = format!(
        "\"this\" object does not implement interface {}.",
        proto_id_to_name(proto_id)
    );
    throw_type_error(cx, &error);
}

pub unsafe fn throw_constructor_without_new(cx: *mut JSContext, name: &str) {
    debug_assert!(!JS_IsExceptionPending(cx));
    let error = format!("{} constructor: 'new' is required", name);
    throw_type_error(cx, &error);
}

impl Error {
    /// Convert this error value to a JS value, consuming it in the process.
    #[allow(clippy::wrong_self_convention)]
    pub unsafe fn to_jsval<D: DomTypes>(
        self,
        cx: *mut JSContext,
        global: &D::GlobalScope,
        rval: MutableHandleValue,
    ) {
        match self {
            Error::JSFailed => (),
            _ => assert!(!JS_IsExceptionPending(cx)),
        }
        <D as DomHelpers<D>>::throw_dom_exception(SafeJSContext::from_ptr(cx), global, self);
        assert!(JS_IsExceptionPending(cx));
        assert!(JS_GetPendingException(cx, rval));
        JS_ClearPendingException(cx);
    }
}
