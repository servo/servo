/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use js::jsapi::{JSContext, JSBool};
use js::jsapi::{JS_IsExceptionPending};

use js::glue::{ReportError};

#[deriving(ToStr)]
pub enum Error {
    FailureUnknown,
    NotFound,
    HierarchyRequest,
    InvalidCharacter,
    NotSupported,
    InvalidState,
    NamespaceError
}

pub type Fallible<T> = Result<T, Error>;

pub type ErrorResult = Fallible<()>;

pub fn throw_method_failed_with_details<T>(cx: *JSContext,
                                           result: Result<T, Error>,
                                           interface: &'static str,
                                           member: &'static str) -> JSBool {
    assert!(result.is_err());
    assert!(unsafe { JS_IsExceptionPending(cx) } == 0);
    let message = format!("Method failed: {}.{}", interface, member);
    message.with_c_str(|string| {
        unsafe { ReportError(cx, string) };
    });
    return 0;
}

pub fn throw_not_in_union(cx: *JSContext, names: &'static str) -> JSBool {
    assert!(unsafe { JS_IsExceptionPending(cx) } == 0);
    let message = format!("argument could not be converted to any of: {}", names);
    message.with_c_str(|string| {
        unsafe { ReportError(cx, string) };
    });
    return 0;
}
