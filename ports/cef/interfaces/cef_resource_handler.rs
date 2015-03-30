/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
#![allow(non_snake_case, unused_imports)]

use eutil;
use interfaces;
use types;
use wrappers::CefWrap;

use libc;
use std::collections::HashMap;
use std::ptr;

//
// Structure used to implement a custom request handler structure. The functions
// of this structure will always be called on the IO thread.
//
#[repr(C)]
pub struct _cef_resource_handler_t {
  //
  // Base structure.
  //
  pub base: types::cef_base_t,

  //
  // Begin processing the request. To handle the request return true (1) and
  // call cef_callback_t::cont() once the response header information is
  // available (cef_callback_t::cont() can also be called from inside this
  // function if header information is available immediately). To cancel the
  // request return false (0).
  //
  pub process_request: Option<extern "C" fn(this: *mut cef_resource_handler_t,
      request: *mut interfaces::cef_request_t,
      callback: *mut interfaces::cef_callback_t) -> libc::c_int>,

  //
  // Retrieve response header information. If the response length is not known
  // set |response_length| to -1 and read_response() will be called until it
  // returns false (0). If the response length is known set |response_length| to
  // a positive value and read_response() will be called until it returns false
  // (0) or the specified number of bytes have been read. Use the |response|
  // object to set the mime type, http status code and other optional header
  // values. To redirect the request to a new URL set |redirectUrl| to the new
  // URL.
  //
  pub get_response_headers: Option<extern "C" fn(
      this: *mut cef_resource_handler_t,
      response: *mut interfaces::cef_response_t, response_length: *mut i64,
      redirectUrl: *mut types::cef_string_t) -> ()>,

  //
  // Read response data. If data is available immediately copy up to
  // |bytes_to_read| bytes into |data_out|, set |bytes_read| to the number of
  // bytes copied, and return true (1). To read the data at a later time set
  // |bytes_read| to 0, return true (1) and call cef_callback_t::cont() when the
  // data is available. To indicate response completion return false (0).
  //
  pub read_response: Option<extern "C" fn(this: *mut cef_resource_handler_t,
      data_out: *mut (), bytes_to_read: libc::c_int,
      bytes_read: *mut libc::c_int,
      callback: *mut interfaces::cef_callback_t) -> libc::c_int>,

  //
  // Return true (1) if the specified cookie can be sent with the request or
  // false (0) otherwise. If false (0) is returned for any cookie then no
  // cookies will be sent with the request.
  //
  pub can_get_cookie: Option<extern "C" fn(this: *mut cef_resource_handler_t,
      cookie: *const interfaces::cef_cookie_t) -> libc::c_int>,

  //
  // Return true (1) if the specified cookie returned with the response can be
  // set or false (0) otherwise.
  //
  pub can_set_cookie: Option<extern "C" fn(this: *mut cef_resource_handler_t,
      cookie: *const interfaces::cef_cookie_t) -> libc::c_int>,

  //
  // Request processing has been canceled.
  //
  pub cancel: Option<extern "C" fn(this: *mut cef_resource_handler_t) -> ()>,

  //
  // The reference count. This will only be present for Rust instances!
  //
  pub ref_count: uint,

  //
  // Extra data. This will only be present for Rust instances!
  //
  pub extra: u8,
}

pub type cef_resource_handler_t = _cef_resource_handler_t;


//
// Structure used to implement a custom request handler structure. The functions
// of this structure will always be called on the IO thread.
//
pub struct CefResourceHandler {
  c_object: *mut cef_resource_handler_t,
}

impl Clone for CefResourceHandler {
  fn clone(&self) -> CefResourceHandler{
    unsafe {
      if !self.c_object.is_null() {
        ((*self.c_object).base.add_ref.unwrap())(&mut (*self.c_object).base);
      }
      CefResourceHandler {
        c_object: self.c_object,
      }
    }
  }
}

impl Drop for CefResourceHandler {
  fn drop(&mut self) {
    unsafe {
      if !self.c_object.is_null() {
        ((*self.c_object).base.release.unwrap())(&mut (*self.c_object).base);
      }
    }
  }
}

impl CefResourceHandler {
  pub unsafe fn from_c_object(c_object: *mut cef_resource_handler_t) -> CefResourceHandler {
    CefResourceHandler {
      c_object: c_object,
    }
  }

  pub unsafe fn from_c_object_addref(c_object: *mut cef_resource_handler_t) -> CefResourceHandler {
    if !c_object.is_null() {
      ((*c_object).base.add_ref.unwrap())(&mut (*c_object).base);
    }
    CefResourceHandler {
      c_object: c_object,
    }
  }

  pub fn c_object(&self) -> *mut cef_resource_handler_t {
    self.c_object
  }

  pub fn c_object_addrefed(&self) -> *mut cef_resource_handler_t {
    unsafe {
      if !self.c_object.is_null() {
        eutil::add_ref(self.c_object as *mut types::cef_base_t);
      }
      self.c_object
    }
  }

  pub fn is_null_cef_object(&self) -> bool {
    self.c_object.is_null()
  }
  pub fn is_not_null_cef_object(&self) -> bool {
    !self.c_object.is_null()
  }

  //
  // Begin processing the request. To handle the request return true (1) and
  // call cef_callback_t::cont() once the response header information is
  // available (cef_callback_t::cont() can also be called from inside this
  // function if header information is available immediately). To cancel the
  // request return false (0).
  //
  pub fn process_request(&self, request: interfaces::CefRequest,
      callback: interfaces::CefCallback) -> libc::c_int {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).process_request.unwrap())(
          self.c_object,
          CefWrap::to_c(request),
          CefWrap::to_c(callback)))
    }
  }

  //
  // Retrieve response header information. If the response length is not known
  // set |response_length| to -1 and read_response() will be called until it
  // returns false (0). If the response length is known set |response_length| to
  // a positive value and read_response() will be called until it returns false
  // (0) or the specified number of bytes have been read. Use the |response|
  // object to set the mime type, http status code and other optional header
  // values. To redirect the request to a new URL set |redirectUrl| to the new
  // URL.
  //
  pub fn get_response_headers(&self, response: interfaces::CefResponse,
      response_length: &mut i64, redirectUrl: *mut types::cef_string_t) -> (
      ) {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).get_response_headers.unwrap())(
          self.c_object,
          CefWrap::to_c(response),
          CefWrap::to_c(response_length),
          CefWrap::to_c(redirectUrl)))
    }
  }

  //
  // Read response data. If data is available immediately copy up to
  // |bytes_to_read| bytes into |data_out|, set |bytes_read| to the number of
  // bytes copied, and return true (1). To read the data at a later time set
  // |bytes_read| to 0, return true (1) and call cef_callback_t::cont() when the
  // data is available. To indicate response completion return false (0).
  //
  pub fn read_response(&self, data_out: &mut (), bytes_to_read: libc::c_int,
      bytes_read: &mut libc::c_int,
      callback: interfaces::CefCallback) -> libc::c_int {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).read_response.unwrap())(
          self.c_object,
          CefWrap::to_c(data_out),
          CefWrap::to_c(bytes_to_read),
          CefWrap::to_c(bytes_read),
          CefWrap::to_c(callback)))
    }
  }

  //
  // Return true (1) if the specified cookie can be sent with the request or
  // false (0) otherwise. If false (0) is returned for any cookie then no
  // cookies will be sent with the request.
  //
  pub fn can_get_cookie(&self, cookie: &interfaces::CefCookie) -> libc::c_int {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).can_get_cookie.unwrap())(
          self.c_object,
          CefWrap::to_c(cookie)))
    }
  }

  //
  // Return true (1) if the specified cookie returned with the response can be
  // set or false (0) otherwise.
  //
  pub fn can_set_cookie(&self, cookie: &interfaces::CefCookie) -> libc::c_int {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).can_set_cookie.unwrap())(
          self.c_object,
          CefWrap::to_c(cookie)))
    }
  }

  //
  // Request processing has been canceled.
  //
  pub fn cancel(&self) -> () {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).cancel.unwrap())(
          self.c_object))
    }
  }
}

impl CefWrap<*mut cef_resource_handler_t> for CefResourceHandler {
  fn to_c(rust_object: CefResourceHandler) -> *mut cef_resource_handler_t {
    rust_object.c_object_addrefed()
  }
  unsafe fn to_rust(c_object: *mut cef_resource_handler_t) -> CefResourceHandler {
    CefResourceHandler::from_c_object_addref(c_object)
  }
}
impl CefWrap<*mut cef_resource_handler_t> for Option<CefResourceHandler> {
  fn to_c(rust_object: Option<CefResourceHandler>) -> *mut cef_resource_handler_t {
    match rust_object {
      None => ptr::null_mut(),
      Some(rust_object) => rust_object.c_object_addrefed(),
    }
  }
  unsafe fn to_rust(c_object: *mut cef_resource_handler_t) -> Option<CefResourceHandler> {
    if c_object.is_null() {
      None
    } else {
      Some(CefResourceHandler::from_c_object_addref(c_object))
    }
  }
}

