/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::conversions::ToJSValConvertible;
use dom::bindings::global::GlobalRef;
use dom::domexception::DOMException;

use js::jsapi::{JSContext, JSBool, JSObject};
use js::jsapi::{JS_IsExceptionPending, JS_SetPendingException, JS_ReportPendingException};
use js::jsapi::{JS_ReportErrorNumber, JSErrorFormatString, JSEXN_TYPEERR};
use js::jsapi::{JS_SaveFrameChain, JS_RestoreFrameChain};
use js::glue::{ReportError};
use js::rust::with_compartment;

use libc;
use std::ptr;

#[deriving(Show)]
pub enum Error {
    IndexSize,
    FailureUnknown,
    NotFound,
    HierarchyRequest,
    InvalidCharacter,
    NotSupported,
    InvalidState,
    Syntax,
    NamespaceError,
    InvalidAccess,
    Security,
    Network,
    Abort,
    Timeout
}

pub type Fallible<T> = Result<T, Error>;

pub type ErrorResult = Fallible<()>;

pub fn throw_dom_exception(cx: *mut JSContext, global: &GlobalRef,
                           result: Error) {
    assert!(unsafe { JS_IsExceptionPending(cx) } == 0);
    let exception = DOMException::new_from_error(global, result).root();
    let thrown = exception.to_jsval(cx);
    unsafe {
        JS_SetPendingException(cx, thrown);
    }
}

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

pub fn throw_not_in_union(cx: *mut JSContext, names: &'static str) -> JSBool {
    assert!(unsafe { JS_IsExceptionPending(cx) } == 0);
    let message = format!("argument could not be converted to any of: {}", names);
    message.with_c_str(|string| {
        unsafe { ReportError(cx, string) };
    });
    return 0;
}

static ERROR_FORMAT_STRING_STRING: [libc::c_char, ..4] = [
    '{' as libc::c_char,
    '0' as libc::c_char,
    '}' as libc::c_char,
    0 as libc::c_char,
];

static ERROR_FORMAT_STRING: JSErrorFormatString = JSErrorFormatString {
    format: &ERROR_FORMAT_STRING_STRING as *libc::c_char,
    argCount: 1,
    exnType: JSEXN_TYPEERR as i16,
};

extern fn get_error_message(_user_ref: *mut libc::c_void,
                            _locale: *libc::c_char,
                            error_number: libc::c_uint) -> *JSErrorFormatString
{
    assert_eq!(error_number, 0);
    &ERROR_FORMAT_STRING as *JSErrorFormatString
}

pub fn throw_type_error(cx: *mut JSContext, error: &str) {
    let error = error.to_c_str();
    error.with_ref(|error| unsafe {
        JS_ReportErrorNumber(cx, Some(get_error_message), ptr::mut_null(), 0, error);
    });
}
