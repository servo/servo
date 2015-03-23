/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Utilities to throw exceptions from Rust bindings.

use dom::bindings::conversions::ToJSValConvertible;
use dom::bindings::global::GlobalRef;
use dom::domexception::{DOMException, DOMErrorName};

use util::str::DOMString;

use js::jsapi::{JSContext, JSBool, JSObject};
use js::jsapi::{JS_IsExceptionPending, JS_SetPendingException, JS_ReportPendingException};
use js::jsapi::{JS_ReportErrorNumber, JSErrorFormatString, JSEXN_TYPEERR};
use js::jsapi::{JS_SaveFrameChain, JS_RestoreFrameChain};
use js::glue::{ReportError};
use js::rust::with_compartment;

use libc;
use std::ffi::CString;
use std::ptr;

/// DOM exceptions that can be thrown by a native DOM method.
#[derive(Debug, Clone)]
pub enum Error {
    /// IndexSizeError DOMException
    IndexSize,
    /// NotFoundError DOMException
    NotFound,
    /// HierarchyRequestError DOMException
    HierarchyRequest,
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
    /// DataCloneError DOMException
    DataClone,
    /// NoModificationAllowedError DOMException
    NoModificationAllowed,

    /// TypeError JavaScript Error
    Type(DOMString),

    /// A JavaScript exception is already pending.
    JSFailed,
}

/// The return type for IDL operations that can throw DOM exceptions.
pub type Fallible<T> = Result<T, Error>;

/// The return type for IDL operations that can throw DOM exceptions and
/// return `()`.
pub type ErrorResult = Fallible<()>;

/// Set a pending exception for the given `result` on `cx`.
pub fn throw_dom_exception(cx: *mut JSContext, global: GlobalRef,
                           result: Error) {
    let code = match result {
        Error::IndexSize => DOMErrorName::IndexSizeError,
        Error::NotFound => DOMErrorName::NotFoundError,
        Error::HierarchyRequest => DOMErrorName::HierarchyRequestError,
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
        Error::DataClone => DOMErrorName::DataCloneError,
        Error::NoModificationAllowed => DOMErrorName::NoModificationAllowedError,
        Error::Type(message) => {
            assert!(unsafe { JS_IsExceptionPending(cx) } == 0);
            throw_type_error(cx, &message);
            return;
        }
        Error::JSFailed => {
            assert!(unsafe { JS_IsExceptionPending(cx) } == 1);
            return;
        }
    };

    assert!(unsafe { JS_IsExceptionPending(cx) } == 0);
    let exception = DOMException::new(global, code).root();
    let thrown = exception.to_jsval(cx);
    unsafe {
        JS_SetPendingException(cx, thrown);
    }
}

/// Report a pending exception, thereby clearing it.
pub fn report_pending_exception(cx: *mut JSContext, obj: *mut JSObject) {
    unsafe {
        if JS_IsExceptionPending(cx) != 0 {
            let saved = JS_SaveFrameChain(cx);
            with_compartment(cx, obj, || {
                JS_ReportPendingException(cx);
            });
            if saved != 0 {
                JS_RestoreFrameChain(cx);
            }
        }
    }
}

/// Throw an exception to signal that a `JSVal` can not be converted to any of
/// the types in an IDL union type.
pub fn throw_not_in_union(cx: *mut JSContext, names: &'static str) -> JSBool {
    assert!(unsafe { JS_IsExceptionPending(cx) } == 0);
    let message = format!("argument could not be converted to any of: {}", names);
    let string = CString::new(message).unwrap();
    unsafe { ReportError(cx, string.as_ptr()) };
    return 0;
}

/// Format string used to throw `TypeError`s.
static ERROR_FORMAT_STRING_STRING: [libc::c_char; 4] = [
    '{' as libc::c_char,
    '0' as libc::c_char,
    '}' as libc::c_char,
    0 as libc::c_char,
];

/// Format string struct used to throw `TypeError`s.
static mut ERROR_FORMAT_STRING: JSErrorFormatString = JSErrorFormatString {
    format: &ERROR_FORMAT_STRING_STRING as *const libc::c_char,
    argCount: 1,
    exnType: JSEXN_TYPEERR as i16,
};

/// Callback used to throw `TypeError`s.
unsafe extern fn get_error_message(_user_ref: *mut libc::c_void,
                            _locale: *const libc::c_char,
                            error_number: libc::c_uint) -> *const JSErrorFormatString
{
    assert_eq!(error_number, 0);
    &ERROR_FORMAT_STRING as *const JSErrorFormatString
}

/// Throw a `TypeError` with the given message.
pub fn throw_type_error(cx: *mut JSContext, error: &str) {
    let error = CString::new(error).unwrap();
    unsafe {
        JS_ReportErrorNumber(cx,
            Some(get_error_message as
                unsafe extern "C" fn(*mut libc::c_void, *const libc::c_char,
                                     libc::c_uint) -> *const JSErrorFormatString),
            ptr::null_mut(), 0, error.as_ptr());
    }
}
