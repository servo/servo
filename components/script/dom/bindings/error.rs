/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Utilities to throw exceptions from Rust bindings.

use std::ffi::CString;
use std::ptr::NonNull;
use std::slice::from_raw_parts;

#[cfg(feature = "js_backtrace")]
use backtrace::Backtrace;
use embedder_traits::JavaScriptErrorInfo;
use js::context::JSContext;
use js::conversions::jsstr_to_string;
use js::error::{throw_range_error, throw_type_error};
#[cfg(feature = "js_backtrace")]
use js::jsapi::StackFormat as JSStackFormat;
use js::jsapi::{ExceptionStackBehavior, JS_ClearPendingException, JS_IsExceptionPending};
use js::jsval::UndefinedValue;
use js::realm::CurrentRealm;
use js::rust::wrappers::{JS_ErrorFromException, JS_GetPendingException, JS_SetPendingException};
use js::rust::wrappers2::JS_GetProperty;
use js::rust::{HandleObject, HandleValue, MutableHandleValue, describe_scripted_caller};
use libc::c_uint;
use script_bindings::conversions::SafeToJSValConvertible;
pub(crate) use script_bindings::error::*;
use script_bindings::root::DomRoot;
use script_bindings::str::DOMString;

#[cfg(feature = "js_backtrace")]
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::conversions::{
    ConversionResult, SafeFromJSValConvertible, root_from_object,
};
use crate::dom::bindings::str::USVString;
use crate::dom::domexception::{DOMErrorName, DOMException};
use crate::dom::globalscope::GlobalScope;
use crate::dom::types::QuotaExceededError;
use crate::realms::InRealm;
use crate::script_runtime::{CanGc, JSContext as SafeJSContext};

#[cfg(feature = "js_backtrace")]
thread_local! {
    /// An optional stringified JS backtrace and stringified native backtrace from the
    /// the last DOM exception that was reported.
    static LAST_EXCEPTION_BACKTRACE: DomRefCell<Option<(Option<String>, String)>> = DomRefCell::new(None);
}

/// Error values that have no equivalent DOMException representation.
pub(crate) enum JsEngineError {
    /// An EMCAScript TypeError.
    Type(CString),
    /// An ECMAScript RangeError.
    Range(CString),
    /// The JS engine reported a thrown exception.
    JSFailed,
}

/// Set a pending exception for the given `result` on `cx`.
pub(crate) fn throw_dom_exception(
    cx: SafeJSContext,
    global: &GlobalScope,
    result: Error,
    can_gc: CanGc,
) {
    #[cfg(feature = "js_backtrace")]
    unsafe {
        capture_stack!(in(*cx) let stack);
        let js_stack = stack.and_then(|s| s.as_string(None, JSStackFormat::Default));
        let rust_stack = Backtrace::new();
        LAST_EXCEPTION_BACKTRACE.with(|backtrace| {
            *backtrace.borrow_mut() = Some((js_stack, format!("{:?}", rust_stack)));
        });
    }

    match create_dom_exception(global, result, can_gc) {
        Ok(exception) => unsafe {
            assert!(!JS_IsExceptionPending(*cx));
            rooted!(in(*cx) let mut thrown = UndefinedValue());
            exception.safe_to_jsval(cx, thrown.handle_mut(), can_gc);
            JS_SetPendingException(*cx, thrown.handle(), ExceptionStackBehavior::Capture);
        },

        Err(JsEngineError::Type(message)) => unsafe {
            assert!(!JS_IsExceptionPending(*cx));
            throw_type_error(*cx, &message);
        },

        Err(JsEngineError::Range(message)) => unsafe {
            assert!(!JS_IsExceptionPending(*cx));
            throw_range_error(*cx, &message);
        },

        Err(JsEngineError::JSFailed) => unsafe {
            assert!(JS_IsExceptionPending(*cx));
        },
    }
}

/// If possible, create a new DOMException representing the provided error.
/// If no such DOMException exists, return a subset of the original error values
/// that may need additional handling.
pub(crate) fn create_dom_exception(
    global: &GlobalScope,
    result: Error,
    can_gc: CanGc,
) -> Result<DomRoot<DOMException>, JsEngineError> {
    let new_custom_exception = |error_name, message| {
        Ok(DOMException::new_with_custom_message(
            global, error_name, message, can_gc,
        ))
    };

    let code = match result {
        Error::IndexSize(Some(custom_message)) => {
            return new_custom_exception(DOMErrorName::IndexSizeError, custom_message);
        },
        Error::IndexSize(None) => DOMErrorName::IndexSizeError,
        Error::NotFound(Some(custom_message)) => {
            return new_custom_exception(DOMErrorName::NotFoundError, custom_message);
        },
        Error::NotFound(None) => DOMErrorName::NotFoundError,
        Error::HierarchyRequest(Some(custom_message)) => {
            return new_custom_exception(DOMErrorName::HierarchyRequestError, custom_message);
        },
        Error::HierarchyRequest(None) => DOMErrorName::HierarchyRequestError,
        Error::WrongDocument(Some(doc_err_custom_message)) => {
            return new_custom_exception(DOMErrorName::WrongDocumentError, doc_err_custom_message);
        },
        Error::WrongDocument(None) => DOMErrorName::WrongDocumentError,
        Error::InvalidCharacter(Some(custom_message)) => {
            return new_custom_exception(DOMErrorName::InvalidCharacterError, custom_message);
        },
        Error::InvalidCharacter(None) => DOMErrorName::InvalidCharacterError,
        Error::NotSupported(Some(custom_message)) => {
            return new_custom_exception(DOMErrorName::NotSupportedError, custom_message);
        },
        Error::NotSupported(None) => DOMErrorName::NotSupportedError,
        Error::InUseAttribute(Some(custom_message)) => {
            return new_custom_exception(DOMErrorName::InUseAttributeError, custom_message);
        },
        Error::InUseAttribute(None) => DOMErrorName::InUseAttributeError,
        Error::InvalidState(Some(custom_message)) => {
            return new_custom_exception(DOMErrorName::InvalidStateError, custom_message);
        },
        Error::InvalidState(None) => DOMErrorName::InvalidStateError,
        Error::Syntax(Some(custom_message)) => {
            return new_custom_exception(DOMErrorName::SyntaxError, custom_message);
        },
        Error::Syntax(None) => DOMErrorName::SyntaxError,
        Error::Namespace(Some(custom_message)) => {
            return new_custom_exception(DOMErrorName::NamespaceError, custom_message);
        },
        Error::Namespace(None) => DOMErrorName::NamespaceError,
        Error::InvalidAccess(Some(custom_message)) => {
            return new_custom_exception(DOMErrorName::InvalidAccessError, custom_message);
        },
        Error::InvalidAccess(None) => DOMErrorName::InvalidAccessError,
        Error::Security(Some(custom_message)) => {
            return new_custom_exception(DOMErrorName::SecurityError, custom_message);
        },
        Error::Security(None) => DOMErrorName::SecurityError,
        Error::Network(Some(custom_message)) => {
            return new_custom_exception(DOMErrorName::NetworkError, custom_message);
        },
        Error::Network(None) => DOMErrorName::NetworkError,
        Error::Abort(Some(custom_message)) => {
            return new_custom_exception(DOMErrorName::AbortError, custom_message);
        },
        Error::Abort(None) => DOMErrorName::AbortError,
        Error::Timeout(Some(custom_message)) => {
            return new_custom_exception(DOMErrorName::TimeoutError, custom_message);
        },
        Error::Timeout(None) => DOMErrorName::TimeoutError,
        Error::InvalidNodeType(Some(custom_message)) => {
            return new_custom_exception(DOMErrorName::InvalidNodeTypeError, custom_message);
        },
        Error::InvalidNodeType(None) => DOMErrorName::InvalidNodeTypeError,
        Error::DataClone(Some(custom_message)) => {
            return new_custom_exception(DOMErrorName::DataCloneError, custom_message);
        },
        Error::DataClone(None) => DOMErrorName::DataCloneError,
        Error::Data(Some(custom_message)) => {
            return new_custom_exception(DOMErrorName::DataError, custom_message);
        },
        Error::Data(None) => DOMErrorName::DataError,
        Error::TransactionInactive(Some(custom_message)) => {
            return new_custom_exception(DOMErrorName::TransactionInactiveError, custom_message);
        },
        Error::TransactionInactive(None) => DOMErrorName::TransactionInactiveError,
        Error::ReadOnly(Some(custom_message)) => {
            return new_custom_exception(DOMErrorName::ReadOnlyError, custom_message);
        },
        Error::ReadOnly(None) => DOMErrorName::ReadOnlyError,
        Error::Version(Some(custom_message)) => {
            return new_custom_exception(DOMErrorName::VersionError, custom_message);
        },
        Error::Version(None) => DOMErrorName::VersionError,
        Error::NoModificationAllowed(Some(custom_message)) => {
            return new_custom_exception(DOMErrorName::NoModificationAllowedError, custom_message);
        },
        Error::NoModificationAllowed(None) => DOMErrorName::NoModificationAllowedError,
        Error::QuotaExceeded { quota, requested } => {
            return Ok(DomRoot::upcast(QuotaExceededError::new(
                global,
                DOMString::new(),
                quota,
                requested,
                can_gc,
            )));
        },
        Error::TypeMismatch(Some(custom_message)) => {
            return new_custom_exception(DOMErrorName::TypeMismatchError, custom_message);
        },
        Error::TypeMismatch(None) => DOMErrorName::TypeMismatchError,
        Error::InvalidModification(Some(custom_message)) => {
            return new_custom_exception(DOMErrorName::InvalidModificationError, custom_message);
        },
        Error::InvalidModification(None) => DOMErrorName::InvalidModificationError,
        Error::NotReadable(Some(custom_message)) => {
            return new_custom_exception(DOMErrorName::NotReadableError, custom_message);
        },
        Error::NotReadable(None) => DOMErrorName::NotReadableError,
        Error::Operation(Some(custom_message)) => {
            return new_custom_exception(DOMErrorName::OperationError, custom_message);
        },
        Error::Operation(None) => DOMErrorName::OperationError,
        Error::NotAllowed(Some(custom_message)) => {
            return new_custom_exception(DOMErrorName::NotAllowedError, custom_message);
        },
        Error::NotAllowed(None) => DOMErrorName::NotAllowedError,
        Error::Encoding(Some(custom_message)) => {
            return new_custom_exception(DOMErrorName::EncodingError, custom_message);
        },
        Error::Encoding(None) => DOMErrorName::EncodingError,
        Error::Constraint(Some(custom_message)) => {
            return new_custom_exception(DOMErrorName::ConstraintError, custom_message);
        },
        Error::Constraint(None) => DOMErrorName::ConstraintError,
        Error::Type(message) => return Err(JsEngineError::Type(message)),
        Error::Range(message) => return Err(JsEngineError::Range(message)),
        Error::JSFailed => return Err(JsEngineError::JSFailed),
    };
    Ok(DOMException::new(global, code, can_gc))
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
        let exception = unsafe { root_from_object::<DOMException>(object.get(), *cx).ok()? };
        let scripted_caller = unsafe { describe_scripted_caller(*cx) }.unwrap_or_default();
        Some(ErrorInfo {
            message: exception.stringifier().into(),
            filename: scripted_caller.filename,
            lineno: scripted_caller.line,
            column: scripted_caller.col + 1,
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

    /// <https://html.spec.whatwg.org/multipage/#extract-error>
    pub(crate) fn from_value(value: HandleValue, cx: SafeJSContext, can_gc: CanGc) -> ErrorInfo {
        if value.is_object() {
            rooted!(in(*cx) let object = value.to_object());
            if let Some(info) = ErrorInfo::from_object(object.handle(), cx) {
                return info;
            }
        }

        match USVString::safe_from_jsval(cx, value, (), can_gc) {
            Ok(ConversionResult::Success(USVString(string))) => {
                let scripted_caller = unsafe { describe_scripted_caller(*cx) }.unwrap_or_default();
                ErrorInfo {
                    message: format!("uncaught exception: {}", string),
                    filename: scripted_caller.filename,
                    lineno: scripted_caller.line,
                    column: scripted_caller.col + 1,
                }
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
    rooted!(in(*cx) let mut value = UndefinedValue());
    if take_pending_exception(cx, value.handle_mut()) {
        let error_info = ErrorInfo::from_value(value.handle(), cx, can_gc);
        report_error(
            error_info,
            value.handle(),
            cx,
            dispatch_event,
            realm,
            can_gc,
        );
    }
}

fn take_pending_exception(cx: SafeJSContext, value: MutableHandleValue) -> bool {
    unsafe {
        if !JS_IsExceptionPending(*cx) {
            return false;
        }
    }

    unsafe {
        if !JS_GetPendingException(*cx, value) {
            JS_ClearPendingException(*cx);
            error!("Uncaught exception: JS_GetPendingException failed");
            return false;
        }

        JS_ClearPendingException(*cx);
    }
    true
}

fn report_error(
    error_info: ErrorInfo,
    value: HandleValue,
    cx: SafeJSContext,
    dispatch_event: bool,
    realm: InRealm,
    can_gc: CanGc,
) {
    error!(
        "Error at {}:{}:{} {}",
        error_info.filename, error_info.lineno, error_info.column, error_info.message
    );

    #[cfg(feature = "js_backtrace")]
    {
        LAST_EXCEPTION_BACKTRACE.with(|backtrace| {
            if let Some((js_backtrace, rust_backtrace)) = backtrace.borrow_mut().take() {
                if let Some(stack) = js_backtrace {
                    error!("JS backtrace:\n{}", stack);
                }
                error!("Rust backtrace:\n{}", rust_backtrace);
            }
        });
    }

    if dispatch_event {
        GlobalScope::from_safe_context(cx, realm).report_an_error(error_info, value, can_gc);
    }
}

pub(crate) fn javascript_error_info_from_error_info(
    cx: &mut JSContext,
    error_info: &ErrorInfo,
    value: HandleValue,
) -> JavaScriptErrorInfo {
    let mut stack = || {
        if !value.is_object() {
            return None;
        }

        rooted!(&in(cx) let object = value.to_object());
        rooted!(&in(cx) let mut stack_value = UndefinedValue());
        if unsafe {
            !JS_GetProperty(
                cx,
                object.handle(),
                c"stack".as_ptr(),
                stack_value.handle_mut(),
            )
        } {
            return None;
        }
        if !stack_value.is_string() {
            return None;
        }
        let stack_string = NonNull::new(stack_value.to_string())?;
        Some(unsafe { jsstr_to_string(cx.raw_cx(), stack_string) })
    };

    JavaScriptErrorInfo {
        message: error_info.message.clone(),
        filename: error_info.filename.clone(),
        line_number: error_info.lineno as u64,
        column: error_info.column as u64,
        stack: stack(),
    }
}

pub(crate) fn take_and_report_pending_exception_for_api(
    cx: &mut CurrentRealm,
) -> Option<JavaScriptErrorInfo> {
    let in_realm_proof = cx.into();
    let in_realm = InRealm::Already(&in_realm_proof);

    rooted!(&in(cx) let mut value = UndefinedValue());
    if !take_pending_exception(cx.into(), value.handle_mut()) {
        return None;
    }

    let error_info = ErrorInfo::from_value(value.handle(), cx.into(), CanGc::from_cx(cx));
    let return_value = javascript_error_info_from_error_info(cx, &error_info, value.handle());
    report_error(
        error_info,
        value.handle(),
        cx.into(),
        true, /* dispatch_event */
        in_realm,
        CanGc::from_cx(cx),
    );
    Some(return_value)
}

pub(crate) trait ErrorToJsval {
    fn to_jsval(
        self,
        cx: SafeJSContext,
        global: &GlobalScope,
        rval: MutableHandleValue,
        can_gc: CanGc,
    );
}

impl ErrorToJsval for Error {
    /// Convert this error value to a JS value, consuming it in the process.
    fn to_jsval(
        self,
        cx: SafeJSContext,
        global: &GlobalScope,
        rval: MutableHandleValue,
        can_gc: CanGc,
    ) {
        match self {
            Error::JSFailed => (),
            _ => unsafe { assert!(!JS_IsExceptionPending(*cx)) },
        }
        throw_dom_exception(cx, global, self, can_gc);
        unsafe {
            assert!(JS_IsExceptionPending(*cx));
            assert!(JS_GetPendingException(*cx, rval));
            JS_ClearPendingException(*cx);
        }
    }
}
