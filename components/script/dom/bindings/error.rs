/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Utilities to throw exceptions from Rust bindings.

use dom::bindings::codegen::PrototypeList::proto_id_to_name;
use dom::bindings::conversions::ToJSValConvertible;
use dom::bindings::global::GlobalRef;
use dom::domexception::{DOMException, DOMErrorName};

use util::mem::HeapSizeOf;
use util::str::DOMString;

use js::jsapi::JSAutoCompartment;
use js::jsapi::{JSContext, JSObject, RootedValue};
use js::jsapi::{JS_IsExceptionPending, JS_SetPendingException, JS_ReportPendingException};
use js::jsapi::{JS_ReportErrorNumber1, JSErrorFormatString, JSExnType};
use js::jsapi::{JS_SaveFrameChain, JS_RestoreFrameChain};
use js::jsval::UndefinedValue;

use libc;
use std::ffi::CString;
use std::mem;
use std::ptr;

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

    /// TypeError JavaScript Error
    Type(DOMString),
    /// RangeError JavaScript Error
    Range(DOMString),

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
        Error::Type(message) => {
            assert!(unsafe { JS_IsExceptionPending(cx) } == 0);
            throw_type_error(cx, &message);
            return;
        },
        Error::Range(message) => {
            assert!(unsafe { JS_IsExceptionPending(cx) } == 0);
            throw_range_error(cx, &message);
            return;
        },
        Error::JSFailed => {
            assert!(unsafe { JS_IsExceptionPending(cx) } == 1);
            return;
        }
    };

    assert!(unsafe { JS_IsExceptionPending(cx) } == 0);
    let exception = DOMException::new(global, code);
    let mut thrown = RootedValue::new(cx, UndefinedValue());
    exception.to_jsval(cx, thrown.handle_mut());
    unsafe {
        JS_SetPendingException(cx, thrown.handle());
    }
}

/// Report a pending exception, thereby clearing it.
pub fn report_pending_exception(cx: *mut JSContext, obj: *mut JSObject) {
    unsafe {
        if JS_IsExceptionPending(cx) != 0 {
            let saved = JS_SaveFrameChain(cx);
            {
                let _ac = JSAutoCompartment::new(cx, obj);
                JS_ReportPendingException(cx);
            }
            if saved != 0 {
                JS_RestoreFrameChain(cx);
            }
        }
    }
}

/// Throw an exception to signal that a `JSVal` can not be converted to any of
/// the types in an IDL union type.
pub fn throw_not_in_union(cx: *mut JSContext, names: &'static str) {
    assert!(unsafe { JS_IsExceptionPending(cx) } == 0);
    let error = format!("argument could not be converted to any of: {}", names);
    throw_type_error(cx, &error);
}

/// Throw an exception to signal that a `JSObject` can not be converted to a
/// given DOM type.
pub fn throw_invalid_this(cx: *mut JSContext, proto_id: u16) {
    debug_assert!(unsafe { JS_IsExceptionPending(cx) } == 0);
    let error = format!("\"this\" object does not implement interface {}.",
                        proto_id_to_name(proto_id));
    throw_type_error(cx, &error);
}

/// Format string used to throw javascript errors.
static ERROR_FORMAT_STRING_STRING: [libc::c_char; 4] = [
    '{' as libc::c_char,
    '0' as libc::c_char,
    '}' as libc::c_char,
    0 as libc::c_char,
];

/// Format string struct used to throw `TypeError`s.
static mut TYPE_ERROR_FORMAT_STRING: JSErrorFormatString = JSErrorFormatString {
    format: &ERROR_FORMAT_STRING_STRING as *const libc::c_char,
    argCount: 1,
    exnType: JSExnType::JSEXN_TYPEERR as i16,
};

/// Format string struct used to throw `RangeError`s.
static mut RANGE_ERROR_FORMAT_STRING: JSErrorFormatString = JSErrorFormatString {
    format: &ERROR_FORMAT_STRING_STRING as *const libc::c_char,
    argCount: 1,
    exnType: JSExnType::JSEXN_RANGEERR as i16,
};

/// Callback used to throw javascript errors.
/// See throw_js_error for info about error_number.
unsafe extern fn get_error_message(_user_ref: *mut libc::c_void,
                                   error_number: libc::c_uint)
    -> *const JSErrorFormatString
{
    let num: JSExnType = mem::transmute(error_number);
    match num {
      JSExnType::JSEXN_TYPEERR => &TYPE_ERROR_FORMAT_STRING as *const JSErrorFormatString,
      JSExnType::JSEXN_RANGEERR => &RANGE_ERROR_FORMAT_STRING as *const JSErrorFormatString,
      _ => panic!("Bad js error number given to get_error_message: {}", error_number)
    }
}

/// Helper fn to throw a javascript error with the given message and number.
/// Reuse the jsapi error codes to distinguish the error_number
/// passed back to the get_error_message callback.
/// c_uint is u32, so this cast is safe, as is casting to/from i32 from there.
fn throw_js_error(cx: *mut JSContext, error: &str, error_number: u32) {
    let error = CString::new(error).unwrap();
    unsafe {
        JS_ReportErrorNumber1(cx,
            Some(get_error_message),
            ptr::null_mut(), error_number, error.as_ptr());
    }
}

/// Throw a `TypeError` with the given message.
pub fn throw_type_error(cx: *mut JSContext, error: &str) {
    throw_js_error(cx, error, JSExnType::JSEXN_TYPEERR as u32);
}

/// Throw a `RangeError` with the given message.
pub fn throw_range_error(cx: *mut JSContext, error: &str) {
    throw_js_error(cx, error, JSExnType::JSEXN_RANGEERR as u32);
}
