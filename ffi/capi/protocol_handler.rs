/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ffi::{c_char, c_void};
use std::future::Future;
use std::pin::Pin;
use embedder_traits::WebResourceResponse;
use embedder_traits::WebResourceRequest;
use http::{HeaderMap, HeaderName, HeaderValue, Method, StatusCode};
use servo_api::ServoUrl;
use servo_api::protocol_handler::{
    DoneChannel, FetchContext, HttpStatus, NetworkError, ProtocolHandler, ProtocolRegistry, Request,
    ResourceFetchTiming, ResourceTimingType, Response, ResponseBody,
};
use url::Url;

use crate::ServoError;

/// Opaque handle for a protocol handler.
/// Created by `servo_builder_register_protocol_handler`.
/// The handler is owned by the `ProtocolRegistry` and lives until the `Servo` instance is destroyed.
// cbindgen:opaque
#[repr(C)]
pub struct ServoProtocolHandler {
    _private: (),
}

/// Callback signature for a custom protocol handler.
///
/// The callback receives the request details and should populate the response.
/// Return `true` to indicate the response was provided (intercept), `false` to let the load proceed normally.
///
/// # Safety
///
/// The embedder must ensure:
/// - `request` is a valid pointer to a `ServoWebResourceRequest` for the duration of the callback.
/// - `response` is a valid pointer to a `ServoWebResourceResponse` that the callback populates.
/// - The callback does not unwind across the FFI boundary.
/// - The callback does not retain or use `request` or `response` after returning.
pub type ServoProtocolHandlerCallback = unsafe extern "C" fn(
    request: *const ServoWebResourceRequest,
    response: *mut ServoWebResourceResponse,
    user_data: *mut c_void,
) -> bool;

/// Opaque handle for a web resource request.
/// Provided to the `intercept_request` delegate callback and protocol handler callback.
// cbindgen:opaque
pub struct ServoWebResourceRequest {
    method: Method,
    url: Url,
    headers: HeaderMap,
}

/// Opaque handle for a web resource response.
/// Populated by the embedder in the `intercept_request` delegate callback or protocol handler callback.
// cbindgen:opaque
pub struct ServoWebResourceResponse {
    url: Url,
    headers: HeaderMap,
    status_code: StatusCode,
    status_message: Vec<u8>,
    body: Vec<u8>,
}

/// Register a custom protocol handler for a scheme (e.g., "myapp").
///
/// `scheme` must be a valid, non-empty scheme name without the trailing colon.
/// Forbidden schemes (http, https, chrome, about) cannot be registered and will return an error.
///
/// # Safety
///
/// The caller must ensure that:
/// - `builder` is a non-null pointer to a `ServoBuilder` previously returned by `servo_builder_create`.
/// - `scheme` is a valid NUL-terminated UTF-8 string.
/// - `callback` is a valid function pointer matching `ServoProtocolHandlerCallback`.
/// - `user_data` is a pointer that remains valid for the lifetime of the `Servo` instance.
/// - The callback does not unwind across the FFI boundary.
///
/// # Errors
///
/// Returns `ServoError::NullArgument` if any pointer argument is null.
/// Returns `ServoError::InvalidState` if the scheme is invalid, forbidden, or already registered.
/// Returns `ServoError::InvalidState` if the protocol registry cannot be modified.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_builder_register_protocol_handler(
    builder: *mut crate::ServoBuilder,
    scheme: *const c_char,
    callback: ServoProtocolHandlerCallback,
    user_data: *mut c_void,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if builder.is_null() || scheme.is_null() || (callback as *const ()).is_null() {
            return ServoError::NullArgument;
        }

        let scheme_str = match unsafe { std::ffi::CStr::from_ptr(scheme) }.to_str() {
            Ok(s) => s.to_string(),
            Err(_) => return ServoError::InvalidState,
        };

        if scheme_str.is_empty() {
            return ServoError::InvalidState;
        }

        // SAFETY: The caller is assumed to uphold the safety requirements.
        let builder = unsafe { &mut *builder };

        // Get or create the protocol registry
        let protocol_registry = builder
            .protocol_registry
            .get_or_insert_with(ProtocolRegistry::default);

        // Create a wrapper protocol handler that calls the C callback
        let handler = CProtocolHandler {
            callback,
            user_data,
        };

        match protocol_registry.register(&scheme_str, handler) {
            Ok(()) => ServoError::Ok,
            Err(_) => ServoError::InvalidState,
        }
    })
}

/// Internal protocol handler that bridges to the C callback.
struct CProtocolHandler {
    callback: ServoProtocolHandlerCallback,
    user_data: *mut c_void,
}

// SAFETY: The handler is only ever called from the network task while it owns
// the request. The embedder-supplied callback and `user_data` are required by
// the C API contract to be thread-safe for the lifetime of the `Servo`
// instance; upholding that is the caller's responsibility.
unsafe impl Send for CProtocolHandler {}
unsafe impl Sync for CProtocolHandler {}

impl ProtocolHandler for CProtocolHandler {
    fn load<'a>(
        &'a self,
        request: &'a mut Request,
        _done_chan: &mut DoneChannel,
        _context: &FetchContext,
    ) -> Pin<Box<dyn Future<Output = Response> + Send + 'a>> {
        // Convert the Rust request to a C request
        let c_request = ServoWebResourceRequest::from_request(request);

        // We need to create a response object that the callback can populate
        let request_url = request.current_url().as_url().clone();
        let mut c_response = ServoWebResourceResponse::new(request_url);

        // Call the C callback
        let intercepted = unsafe {
            (self.callback)(
                &c_request as *const _,
                &mut c_response as *mut _,
                self.user_data,
            )
        };

        if intercepted {
            // Convert the C response back to a Rust Response
            Box::pin(async move { c_response.into_response() })
        } else {
            // Not intercepted, proceed with normal fetch (return error to trigger normal handling)
            Box::pin(async move {
                Response::network_error(NetworkError::ProtocolHandlerSubstitutionError)
            })
        }
    }

    fn is_fetchable(&self) -> bool {
        true
    }
}

impl ServoWebResourceRequest {
    pub(crate) fn from_request(request: &Request) -> Self {
        Self {
            method: request.method.clone(),
            url: request.current_url().as_url().clone(),
            headers: request.headers.clone(),
        }
    }

    pub(crate) fn from_web_resource_request(request: &WebResourceRequest) -> Self {
        Self {
            method: request.method.clone(),
            url: request.url.as_url().clone(),
            headers: request.headers.clone(),
        }
    }
}

impl ServoWebResourceResponse {
    pub(crate) fn new(url: Url) -> Self {
        Self {
            url,
            headers: HeaderMap::new(),
            status_code: StatusCode::OK,
            status_message: b"OK".to_vec(),
            body: Vec::new(),
        }
    }

    fn into_response(self) -> Response {
        let mut response = Response::new(
            ServoUrl::from_url(self.url),
            ResourceFetchTiming::new(ResourceTimingType::None),
        );
        response.headers = self.headers;
        response.status = HttpStatus::new_raw(self.status_code.as_u16(), self.status_message);
        response.body = servo_arc::Arc::new(parking_lot::Mutex::new(ResponseBody::Done(self.body)));
        response
    }

    pub(crate) fn into_web_resource_response(self) -> WebResourceResponse {
        let mut response = WebResourceResponse::new(self.url);
        response.headers = self.headers;
        response.status_code = self.status_code;
        response.status_message = self.status_message;
        response
    }
}

/// Get the HTTP method of the request (e.g., "GET", "POST").
///
/// Returns a pointer to a static string that is valid for the lifetime of the request.
/// The caller must not free the returned string.
#[unsafe(no_mangle)]
pub extern "C" fn servo_web_resource_request_get_method(
    request: *const ServoWebResourceRequest,
) -> *const c_char {
    if request.is_null() {
        return std::ptr::null();
    }
    let request = unsafe { &*request };
    static GET: &[u8] = b"GET\0";
    static POST: &[u8] = b"POST\0";
    static PUT: &[u8] = b"PUT\0";
    static DELETE: &[u8] = b"DELETE\0";
    static HEAD: &[u8] = b"HEAD\0";
    static OPTIONS: &[u8] = b"OPTIONS\0";
    static PATCH: &[u8] = b"PATCH\0";
    static OTHER: &[u8] = b"OTHER\0";

    match request.method {
        Method::GET => GET.as_ptr() as *const c_char,
        Method::POST => POST.as_ptr() as *const c_char,
        Method::PUT => PUT.as_ptr() as *const c_char,
        Method::DELETE => DELETE.as_ptr() as *const c_char,
        Method::HEAD => HEAD.as_ptr() as *const c_char,
        Method::OPTIONS => OPTIONS.as_ptr() as *const c_char,
        Method::PATCH => PATCH.as_ptr() as *const c_char,
        _ => OTHER.as_ptr() as *const c_char,
    }
}

/// Get the URL of the request.
///
/// Returns the number of bytes required (including terminating NUL) if `buffer` is NULL or `buffer_len` is 0.
/// Otherwise, writes the URL into `buffer` and returns the number of bytes written (including terminating NUL).
#[unsafe(no_mangle)]
pub extern "C" fn servo_web_resource_request_get_url(
    request: *const ServoWebResourceRequest,
    buffer: *mut c_char,
    buffer_len: usize,
) -> usize {
    if request.is_null() {
        return 0;
    }
    let request = unsafe { &*request };
    let url_str = request.url.as_str();
    let len = url_str.len() + 1; // +1 for NUL

    if buffer.is_null() || buffer_len == 0 {
        return len;
    }

    let write_len = std::cmp::min(len, buffer_len);
    unsafe {
        std::ptr::copy_nonoverlapping(url_str.as_ptr(), buffer as *mut u8, write_len - 1);
        *buffer.add(write_len - 1) = 0;
    }
    len
}

/// Get a request header value by name.
///
/// Returns the number of bytes required (including terminating NUL) if `buffer` is NULL or `buffer_len` is 0.
/// Otherwise, writes the header value into `buffer` and returns the number of bytes written (including terminating NUL).
/// Returns 0 if the header is not present.
#[unsafe(no_mangle)]
pub extern "C" fn servo_web_resource_request_get_header(
    request: *const ServoWebResourceRequest,
    name: *const c_char,
    buffer: *mut c_char,
    buffer_len: usize,
) -> usize {
    if request.is_null() || name.is_null() {
        return 0;
    }
    let request = unsafe { &*request };
    let name_str = match unsafe { std::ffi::CStr::from_ptr(name) }.to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };

    let header_name = match HeaderName::from_bytes(name_str.as_bytes()) {
        Ok(n) => n,
        Err(_) => return 0,
    };

    let Some(value) = request.headers.get(&header_name) else {
        return 0;
    };

    let value_bytes = value.as_bytes();
    let len = value_bytes.len() + 1;

    if buffer.is_null() || buffer_len == 0 {
        return len;
    }

    let write_len = std::cmp::min(len, buffer_len);
    unsafe {
        std::ptr::copy_nonoverlapping(value_bytes.as_ptr(), buffer as *mut u8, write_len - 1);
        *buffer.add(write_len - 1) = 0;
    }
    len
}

/// Get the request body data.
///
/// Returns the number of bytes in the body if `buffer` is NULL or `buffer_len` is 0.
/// Otherwise, copies up to `buffer_len` bytes into `buffer` and returns the total body length.
#[unsafe(no_mangle)]
pub extern "C" fn servo_web_resource_request_get_body(
    request: *const ServoWebResourceRequest,
    _buffer: *mut u8,
    _buffer_len: usize,
) -> usize {
    if request.is_null() {
        return 0;
    }
    // Note: The current Request type doesn't expose body directly in a simple way.
    // This would need to be extended when body access is needed.
    // For now, return 0.
    0
}

/// Set the response URL.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_web_resource_response_set_url(
    response: *mut ServoWebResourceResponse,
    url: *const c_char,
) -> i32 {
    if response.is_null() || url.is_null() {
        return -1;
    }
    let response = unsafe { &mut *response };
    let url_str = match unsafe { std::ffi::CStr::from_ptr(url) }.to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };
    match Url::parse(url_str) {
        Ok(u) => {
            response.url = u;
            0
        }
        Err(_) => -1,
    }
}

/// Set the response status code.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_web_resource_response_set_status(
    response: *mut ServoWebResourceResponse,
    status_code: u16,
    status_message: *const c_char,
) -> i32 {
    if response.is_null() {
        return -1;
    }
    let response = unsafe { &mut *response };
    match StatusCode::from_u16(status_code) {
        Ok(sc) => {
            response.status_code = sc;
            if !status_message.is_null() {
                let msg = unsafe { std::ffi::CStr::from_ptr(status_message) }.to_bytes();
                response.status_message = msg.to_vec();
            }
            0
        }
        Err(_) => -1,
    }
}

/// Set a response header.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_web_resource_response_set_header(
    response: *mut ServoWebResourceResponse,
    name: *const c_char,
    value: *const c_char,
) -> i32 {
    if response.is_null() || name.is_null() || value.is_null() {
        return -1;
    }
    let response = unsafe { &mut *response };
    let name_str = match unsafe { std::ffi::CStr::from_ptr(name) }.to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };
    let value_str = match unsafe { std::ffi::CStr::from_ptr(value) }.to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };

    let header_name = match HeaderName::from_bytes(name_str.as_bytes()) {
        Ok(n) => n,
        Err(_) => return -1,
    };
    let header_value = match HeaderValue::from_bytes(value_str.as_bytes()) {
        Ok(v) => v,
        Err(_) => return -1,
    };

    response.headers.insert(header_name, header_value);
    0
}

/// Set the response body data.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_web_resource_response_set_body(
    response: *mut ServoWebResourceResponse,
    data: *const u8,
    data_len: usize,
) -> i32 {
    if response.is_null() {
        return -1;
    }
    let response = unsafe { &mut *response };
    if data.is_null() && data_len > 0 {
        return -1;
    }
    response.body = if data.is_null() {
        Vec::new()
    } else {
        unsafe { std::slice::from_raw_parts(data, data_len).to_vec() }
    };
    0
}

/// Free a web resource request (no-op, request is owned by Servo).
#[unsafe(no_mangle)]
pub extern "C" fn servo_web_resource_request_free(_request: *mut ServoWebResourceRequest) {
    // No-op: the request is owned by Servo and should not be freed by the embedder.
}

/// Free a web resource response (no-op, response is owned by the embedder but allocated on stack).
#[unsafe(no_mangle)]
pub extern "C" fn servo_web_resource_response_free(_response: *mut ServoWebResourceResponse) {
    // No-op: the response is typically stack-allocated or owned by the embedder.
}