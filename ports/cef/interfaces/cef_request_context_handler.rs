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
// Implement this structure to provide handler implementations.
//
#[repr(C)]
pub struct _cef_request_context_handler_t {
  //
  // Base structure.
  //
  pub base: types::cef_base_t,

  //
  // Called on the IO thread to retrieve the cookie manager. The global cookie
  // manager will be used if this function returns NULL.
  //
  pub get_cookie_manager: Option<extern "C" fn(
      this: *mut cef_request_context_handler_t) -> *mut interfaces::cef_cookie_manager_t>,

  //
  // The reference count. This will only be present for Rust instances!
  //
  pub ref_count: uint,

  //
  // Extra data. This will only be present for Rust instances!
  //
  pub extra: u8,
}

pub type cef_request_context_handler_t = _cef_request_context_handler_t;


//
// Implement this structure to provide handler implementations.
//
pub struct CefRequestContextHandler {
  c_object: *mut cef_request_context_handler_t,
}

impl Clone for CefRequestContextHandler {
  fn clone(&self) -> CefRequestContextHandler{
    unsafe {
      if !self.c_object.is_null() {
        ((*self.c_object).base.add_ref.unwrap())(&mut (*self.c_object).base);
      }
      CefRequestContextHandler {
        c_object: self.c_object,
      }
    }
  }
}

impl Drop for CefRequestContextHandler {
  fn drop(&mut self) {
    unsafe {
      if !self.c_object.is_null() {
        ((*self.c_object).base.release.unwrap())(&mut (*self.c_object).base);
      }
    }
  }
}

impl CefRequestContextHandler {
  pub unsafe fn from_c_object(c_object: *mut cef_request_context_handler_t) -> CefRequestContextHandler {
    CefRequestContextHandler {
      c_object: c_object,
    }
  }

  pub unsafe fn from_c_object_addref(c_object: *mut cef_request_context_handler_t) -> CefRequestContextHandler {
    if !c_object.is_null() {
      ((*c_object).base.add_ref.unwrap())(&mut (*c_object).base);
    }
    CefRequestContextHandler {
      c_object: c_object,
    }
  }

  pub fn c_object(&self) -> *mut cef_request_context_handler_t {
    self.c_object
  }

  pub fn c_object_addrefed(&self) -> *mut cef_request_context_handler_t {
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
  // Called on the IO thread to retrieve the cookie manager. The global cookie
  // manager will be used if this function returns NULL.
  //
  pub fn get_cookie_manager(&self) -> interfaces::CefCookieManager {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).get_cookie_manager.unwrap())(
          self.c_object))
    }
  }
}

impl CefWrap<*mut cef_request_context_handler_t> for CefRequestContextHandler {
  fn to_c(rust_object: CefRequestContextHandler) -> *mut cef_request_context_handler_t {
    rust_object.c_object_addrefed()
  }
  unsafe fn to_rust(c_object: *mut cef_request_context_handler_t) -> CefRequestContextHandler {
    CefRequestContextHandler::from_c_object_addref(c_object)
  }
}
impl CefWrap<*mut cef_request_context_handler_t> for Option<CefRequestContextHandler> {
  fn to_c(rust_object: Option<CefRequestContextHandler>) -> *mut cef_request_context_handler_t {
    match rust_object {
      None => ptr::null_mut(),
      Some(rust_object) => rust_object.c_object_addrefed(),
    }
  }
  unsafe fn to_rust(c_object: *mut cef_request_context_handler_t) -> Option<CefRequestContextHandler> {
    if c_object.is_null() {
      None
    } else {
      Some(CefRequestContextHandler::from_c_object_addref(c_object))
    }
  }
}

