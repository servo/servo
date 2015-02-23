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
// A request context provides request handling for a set of related browser
// objects. A request context is specified when creating a new browser object
// via the cef_browser_host_t static factory functions. Browser objects with
// different request contexts will never be hosted in the same render process.
// Browser objects with the same request context may or may not be hosted in the
// same render process depending on the process model. Browser objects created
// indirectly via the JavaScript window.open function or targeted links will
// share the same render process and the same request context as the source
// browser. When running in single-process mode there is only a single render
// process (the main process) and so all browsers created in single-process mode
// will share the same request context. This will be the first request context
// passed into a cef_browser_host_t static factory function and all other
// request context objects will be ignored.
//
#[repr(C)]
pub struct _cef_request_context_t {
  //
  // Base structure.
  //
  pub base: types::cef_base_t,

  //
  // Returns true (1) if this object is pointing to the same context as |that|
  // object.
  //
  pub is_same: Option<extern "C" fn(this: *mut cef_request_context_t,
      other: *mut interfaces::cef_request_context_t) -> libc::c_int>,

  //
  // Returns true (1) if this object is the global context.
  //
  pub is_global: Option<extern "C" fn(
      this: *mut cef_request_context_t) -> libc::c_int>,

  //
  // Returns the handler for this context if any.
  //
  pub get_handler: Option<extern "C" fn(
      this: *mut cef_request_context_t) -> *mut interfaces::cef_request_context_handler_t>,

  //
  // The reference count. This will only be present for Rust instances!
  //
  pub ref_count: uint,

  //
  // Extra data. This will only be present for Rust instances!
  //
  pub extra: u8,
}

pub type cef_request_context_t = _cef_request_context_t;


//
// A request context provides request handling for a set of related browser
// objects. A request context is specified when creating a new browser object
// via the cef_browser_host_t static factory functions. Browser objects with
// different request contexts will never be hosted in the same render process.
// Browser objects with the same request context may or may not be hosted in the
// same render process depending on the process model. Browser objects created
// indirectly via the JavaScript window.open function or targeted links will
// share the same render process and the same request context as the source
// browser. When running in single-process mode there is only a single render
// process (the main process) and so all browsers created in single-process mode
// will share the same request context. This will be the first request context
// passed into a cef_browser_host_t static factory function and all other
// request context objects will be ignored.
//
pub struct CefRequestContext {
  c_object: *mut cef_request_context_t,
}

impl Clone for CefRequestContext {
  fn clone(&self) -> CefRequestContext{
    unsafe {
      if !self.c_object.is_null() {
        ((*self.c_object).base.add_ref.unwrap())(&mut (*self.c_object).base);
      }
      CefRequestContext {
        c_object: self.c_object,
      }
    }
  }
}

impl Drop for CefRequestContext {
  fn drop(&mut self) {
    unsafe {
      if !self.c_object.is_null() {
        ((*self.c_object).base.release.unwrap())(&mut (*self.c_object).base);
      }
    }
  }
}

impl CefRequestContext {
  pub unsafe fn from_c_object(c_object: *mut cef_request_context_t) -> CefRequestContext {
    CefRequestContext {
      c_object: c_object,
    }
  }

  pub unsafe fn from_c_object_addref(c_object: *mut cef_request_context_t) -> CefRequestContext {
    if !c_object.is_null() {
      ((*c_object).base.add_ref.unwrap())(&mut (*c_object).base);
    }
    CefRequestContext {
      c_object: c_object,
    }
  }

  pub fn c_object(&self) -> *mut cef_request_context_t {
    self.c_object
  }

  pub fn c_object_addrefed(&self) -> *mut cef_request_context_t {
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
  // Returns true (1) if this object is pointing to the same context as |that|
  // object.
  //
  pub fn is_same(&self, other: interfaces::CefRequestContext) -> libc::c_int {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).is_same.unwrap())(
          self.c_object,
          CefWrap::to_c(other)))
    }
  }

  //
  // Returns true (1) if this object is the global context.
  //
  pub fn is_global(&self) -> libc::c_int {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).is_global.unwrap())(
          self.c_object))
    }
  }

  //
  // Returns the handler for this context if any.
  //
  pub fn get_handler(&self) -> interfaces::CefRequestContextHandler {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).get_handler.unwrap())(
          self.c_object))
    }
  }

  //
  // Returns the global context object.
  //
  pub fn get_global_context() -> interfaces::CefRequestContext {
    unsafe {
      CefWrap::to_rust(
        ::request_context::cef_request_context_get_global_context(
))
    }
  }

  //
  // Creates a new context object with the specified handler.
  //
  pub fn create_context(
      handler: interfaces::CefRequestContextHandler) -> interfaces::CefRequestContext {
    unsafe {
      CefWrap::to_rust(
        ::request_context::cef_request_context_create_context(
          CefWrap::to_c(handler)))
    }
  }
}

impl CefWrap<*mut cef_request_context_t> for CefRequestContext {
  fn to_c(rust_object: CefRequestContext) -> *mut cef_request_context_t {
    rust_object.c_object_addrefed()
  }
  unsafe fn to_rust(c_object: *mut cef_request_context_t) -> CefRequestContext {
    CefRequestContext::from_c_object_addref(c_object)
  }
}
impl CefWrap<*mut cef_request_context_t> for Option<CefRequestContext> {
  fn to_c(rust_object: Option<CefRequestContext>) -> *mut cef_request_context_t {
    match rust_object {
      None => ptr::null_mut(),
      Some(rust_object) => rust_object.c_object_addrefed(),
    }
  }
  unsafe fn to_rust(c_object: *mut cef_request_context_t) -> Option<CefRequestContext> {
    if c_object.is_null() {
      None
    } else {
      Some(CefRequestContext::from_c_object_addref(c_object))
    }
  }
}

