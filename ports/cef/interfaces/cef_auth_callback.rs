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
// Callback structure used for asynchronous continuation of authentication
// requests.
//
#[repr(C)]
pub struct _cef_auth_callback_t {
  //
  // Base structure.
  //
  pub base: types::cef_base_t,

  //
  // Continue the authentication request.
  //
  pub cont: Option<extern "C" fn(this: *mut cef_auth_callback_t,
      username: *const types::cef_string_t,
      password: *const types::cef_string_t) -> ()>,

  //
  // Cancel the authentication request.
  //
  pub cancel: Option<extern "C" fn(this: *mut cef_auth_callback_t) -> ()>,

  //
  // The reference count. This will only be present for Rust instances!
  //
  pub ref_count: uint,

  //
  // Extra data. This will only be present for Rust instances!
  //
  pub extra: u8,
}

pub type cef_auth_callback_t = _cef_auth_callback_t;


//
// Callback structure used for asynchronous continuation of authentication
// requests.
//
pub struct CefAuthCallback {
  c_object: *mut cef_auth_callback_t,
}

impl Clone for CefAuthCallback {
  fn clone(&self) -> CefAuthCallback{
    unsafe {
      if !self.c_object.is_null() {
        ((*self.c_object).base.add_ref.unwrap())(&mut (*self.c_object).base);
      }
      CefAuthCallback {
        c_object: self.c_object,
      }
    }
  }
}

impl Drop for CefAuthCallback {
  fn drop(&mut self) {
    unsafe {
      if !self.c_object.is_null() {
        ((*self.c_object).base.release.unwrap())(&mut (*self.c_object).base);
      }
    }
  }
}

impl CefAuthCallback {
  pub unsafe fn from_c_object(c_object: *mut cef_auth_callback_t) -> CefAuthCallback {
    CefAuthCallback {
      c_object: c_object,
    }
  }

  pub unsafe fn from_c_object_addref(c_object: *mut cef_auth_callback_t) -> CefAuthCallback {
    if !c_object.is_null() {
      ((*c_object).base.add_ref.unwrap())(&mut (*c_object).base);
    }
    CefAuthCallback {
      c_object: c_object,
    }
  }

  pub fn c_object(&self) -> *mut cef_auth_callback_t {
    self.c_object
  }

  pub fn c_object_addrefed(&self) -> *mut cef_auth_callback_t {
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
  // Continue the authentication request.
  //
  pub fn cont(&self, username: &[u16], password: &[u16]) -> () {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).cont.unwrap())(
          self.c_object,
          CefWrap::to_c(username),
          CefWrap::to_c(password)))
    }
  }

  //
  // Cancel the authentication request.
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

impl CefWrap<*mut cef_auth_callback_t> for CefAuthCallback {
  fn to_c(rust_object: CefAuthCallback) -> *mut cef_auth_callback_t {
    rust_object.c_object_addrefed()
  }
  unsafe fn to_rust(c_object: *mut cef_auth_callback_t) -> CefAuthCallback {
    CefAuthCallback::from_c_object_addref(c_object)
  }
}
impl CefWrap<*mut cef_auth_callback_t> for Option<CefAuthCallback> {
  fn to_c(rust_object: Option<CefAuthCallback>) -> *mut cef_auth_callback_t {
    match rust_object {
      None => ptr::null_mut(),
      Some(rust_object) => rust_object.c_object_addrefed(),
    }
  }
  unsafe fn to_rust(c_object: *mut cef_auth_callback_t) -> Option<CefAuthCallback> {
    if c_object.is_null() {
      None
    } else {
      Some(CefAuthCallback::from_c_object_addref(c_object))
    }
  }
}

