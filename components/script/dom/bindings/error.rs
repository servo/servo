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
use js::jsapi::{
    ExceptionStackBehavior, JSContext, JS_ClearPendingException, JS_IsExceptionPending,
};
use js::jsval::UndefinedValue;
use js::rust::wrappers::{JS_ErrorFromException, JS_GetPendingException, JS_SetPendingException};
use js::rust::{HandleObject, HandleValue, MutableHandleValue};
use libc::c_uint;
#[cfg(feature = "js_backtrace")]
use script_bindings::error::LAST_EXCEPTION_BACKTRACE;
pub use script_bindings::error::*;

use crate::script_runtime::CanGc;

pub unsafe fn report_pending_exception(
    cx: *mut JSContext,
    dispatch_event: bool,
    realm: InRealm,
    can_gc: CanGc,
) {
    script_bindings::error::report_pending_exception::<crate::DomTypeHolder>(
        cx,
        dispatch_event,
        realm,
        can_gc,
    )
}

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
use crate::script_runtime::JSContext as SafeJSContext;

/// Set a pending exception for the given `result` on `cx`.
pub fn throw_dom_exception(cx: SafeJSContext, global: &GlobalScope, result: Error) {
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
        let exception = DOMException::new(global, code);
        rooted!(in(*cx) let mut thrown = UndefinedValue());
        exception.to_jsval(*cx, thrown.handle_mut());
        JS_SetPendingException(*cx, thrown.handle(), ExceptionStackBehavior::Capture);
    }
}
