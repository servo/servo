/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ffi::{CStr, CString, c_char, c_void};
use std::future::Future;
use std::pin::Pin;

use http::{HeaderName, HeaderValue, StatusCode};
use servo_api::protocol_handler::{
    DoneChannel, FetchContext, HttpStatus, NetworkError, ProtocolHandler, Request,
    ResourceFetchTiming, Response, ResponseBody,
};

/// An opaque struct representing a request for a resource of a scheme
/// registered with `servo_builder_add_protocol_handler`.
///
/// Handles to requests are only passed to a `ServoProtocolHandlerCallback`
/// and are valid only for the duration of that callback, as are all
/// strings reached from them.
// cbindgen:opaque
pub struct ServoProtocolRequest {
    url: CString,
    method: CString,
}

/// An opaque struct representing the response to a `ServoProtocolRequest`,
/// which the `ServoProtocolHandlerCallback` it is passed to fills in.
///
/// Unless set, the status is 200 and the body is empty.
///
/// Handles to responses are only passed to a `ServoProtocolHandlerCallback`
/// and are valid only for the duration of that callback.
// cbindgen:opaque
pub struct ServoProtocolResponse {
    response: Response,
    body: Vec<u8>,
}

/// A callback used by Servo to load a resource of a scheme registered with
/// `servo_builder_add_protocol_handler`.
///
/// The callback receives:
/// - `request`: a handle to the request. Inspect it with the
///              `servo_protocol_request_*` functions.
/// - `response`: a handle to the response. Fill it in with the
///               `servo_protocol_response_*` functions before returning.
/// - `user_data`: the user data pointer that was passed to
///                `servo_builder_add_protocol_handler`.
///
/// The callback is invoked on one of Servo's internal threads, not on the
/// embedder thread that runs `servo_spin_event_loop`.
pub type ServoProtocolHandlerCallback = unsafe extern "C" fn(
    request: *const ServoProtocolRequest,
    response: *mut ServoProtocolResponse,
    user_data: *mut c_void,
);

/// Gets the URL of `request`.
///
/// Returns a NUL terminated UTF-8 string, which is owned by `request` and
/// valid for as long as it is.
///
/// # Safety
///
/// The caller must ensure that `request` is a non-null pointer to a
/// `ServoProtocolRequest` that is still valid, as documented on
/// `ServoProtocolRequest`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_protocol_request_url(
    request: *const ServoProtocolRequest,
) -> *const c_char {
    assert!(!request.is_null(), "request pointer must not be null");

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `request` documented above.
    unsafe { &*request }.url.as_ptr()
}

/// Gets the method of `request`, such as `GET`.
///
/// Returns a NUL terminated UTF-8 string, which is owned by `request` and
/// valid for as long as it is.
///
/// # Safety
///
/// The caller must ensure that `request` is a non-null pointer to a
/// `ServoProtocolRequest` that is still valid, as documented on
/// `ServoProtocolRequest`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_protocol_request_method(
    request: *const ServoProtocolRequest,
) -> *const c_char {
    assert!(!request.is_null(), "request pointer must not be null");

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `request` documented above.
    unsafe { &*request }.method.as_ptr()
}

/// Sets the HTTP status code of `response`.
///
/// # Safety
///
/// The caller must ensure that `response` is a non-null pointer to a
/// `ServoProtocolResponse` that is still valid, as documented on
/// `ServoProtocolResponse`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_protocol_response_set_status(
    response: *mut ServoProtocolResponse,
    status: u16,
) {
    assert!(!response.is_null(), "response pointer must not be null");

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `response` documented above.
    let response = unsafe { &mut *response };

    let reason = StatusCode::from_u16(status)
        .ok()
        .and_then(|code| code.canonical_reason())
        .unwrap_or_default();
    response.response.status = HttpStatus::new_raw(status, reason.as_bytes().to_vec());
}

/// Adds a header to `response`.
///
/// `name` and `value` are NUL terminated strings.
/// The function panics if they are not a valid header name and value.
/// The ownership of `name` and `value` remains with the caller after the
/// call.
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `response` is a non-null pointer to a `ServoProtocolResponse` that is
///   still valid, as documented on `ServoProtocolResponse`.
/// - `name` and `value` are non-null pointers to C strings that remain
///   unmodified for the duration of the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_protocol_response_add_header(
    response: *mut ServoProtocolResponse,
    name: *const c_char,
    value: *const c_char,
) {
    assert!(!response.is_null(), "response pointer must not be null");
    assert!(!name.is_null(), "name pointer must not be null");
    assert!(!value.is_null(), "value pointer must not be null");

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `response` documented above.
    let response = unsafe { &mut *response };

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `name` and `value` documented above.
    let (name, value) = unsafe { (CStr::from_ptr(name), CStr::from_ptr(value)) };
    let name = HeaderName::from_bytes(name.to_bytes()).expect("invalid header name");
    let value = HeaderValue::from_bytes(value.to_bytes()).expect("invalid header value");
    response.response.headers.append(name, value);
}

/// Sets the body of `response` to `length` bytes from `data`, which are
/// copied.
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `response` is a non-null pointer to a `ServoProtocolResponse` that is
///   still valid, as documented on `ServoProtocolResponse`.
/// - `data` is a pointer to `length` bytes that remain unmodified for the
///   duration of the call, and is only `NULL` if `length` is 0.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_protocol_response_set_body(
    response: *mut ServoProtocolResponse,
    data: *const u8,
    length: usize,
) {
    assert!(!response.is_null(), "response pointer must not be null");
    assert!(
        !data.is_null() || length == 0,
        "data pointer must not be null unless length is 0"
    );

    // SAFETY: The caller is assumed to uphold the safety requirements
    // for `response` documented above.
    let response = unsafe { &mut *response };

    response.body = if length == 0 {
        Vec::new()
    } else {
        // SAFETY: The caller is assumed to uphold the safety requirements
        // for `data` documented above.
        unsafe { std::slice::from_raw_parts(data, length) }.to_vec()
    };
}

pub(crate) struct CallbackProtocolHandler {
    callback: ServoProtocolHandlerCallback,
    user_data: *mut c_void,
}

// SAFETY: The embedder guarantees, as documented on
// `servo_builder_add_protocol_handler`, that the callback and its user data
// can be used from any thread.
unsafe impl Send for CallbackProtocolHandler {}
// SAFETY: See above.
unsafe impl Sync for CallbackProtocolHandler {}

impl CallbackProtocolHandler {
    pub(crate) fn new(callback: ServoProtocolHandlerCallback, user_data: *mut c_void) -> Self {
        Self {
            callback,
            user_data,
        }
    }
}

impl ProtocolHandler for CallbackProtocolHandler {
    fn load(
        &self,
        request: &mut Request,
        _done_chan: &mut DoneChannel,
        _context: &FetchContext,
    ) -> Pin<Box<dyn Future<Output = Response> + Send>> {
        let url = request.current_url();
        let (Ok(url_string), Ok(method)) = (
            CString::new(url.as_str()),
            CString::new(request.method.as_str()),
        ) else {
            return Box::pin(std::future::ready(Response::network_error(
                NetworkError::ResourceLoadError("Invalid URL or method".to_owned()),
            )));
        };
        let callback_request = ServoProtocolRequest {
            url: url_string,
            method,
        };
        let mut callback_response = ServoProtocolResponse {
            response: Response::new(url, ResourceFetchTiming::new(request.timing_type())),
            body: Vec::new(),
        };

        // SAFETY: The embedder is assumed to uphold the safety requirements
        // for `callback` and `user_data` documented on
        // `servo_builder_add_protocol_handler`. `callback_request` and
        // `callback_response` are valid for the call as they are dropped
        // only after `callback` returns.
        unsafe { (self.callback)(&callback_request, &mut callback_response, self.user_data) };

        let ServoProtocolResponse { response, body } = callback_response;
        *response.body.lock() = ResponseBody::Done(body);
        Box::pin(std::future::ready(response))
    }

    fn is_fetchable(&self) -> bool {
        true
    }
}
