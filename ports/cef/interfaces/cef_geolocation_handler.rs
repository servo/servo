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
// Callback structure used for asynchronous continuation of geolocation
// permission requests.
//
#[repr(C)]
pub struct _cef_geolocation_callback_t {
  //
  // Base structure.
  //
  pub base: types::cef_base_t,

  //
  // Call to allow or deny geolocation access.
  //
  pub cont: Option<extern "C" fn(this: *mut cef_geolocation_callback_t,
      allow: libc::c_int) -> ()>,

  //
  // The reference count. This will only be present for Rust instances!
  //
  pub ref_count: uint,

  //
  // Extra data. This will only be present for Rust instances!
  //
  pub extra: u8,
}

pub type cef_geolocation_callback_t = _cef_geolocation_callback_t;


//
// Callback structure used for asynchronous continuation of geolocation
// permission requests.
//
pub struct CefGeolocationCallback {
  c_object: *mut cef_geolocation_callback_t,
}

impl Clone for CefGeolocationCallback {
  fn clone(&self) -> CefGeolocationCallback{
    unsafe {
      if !self.c_object.is_null() {
        ((*self.c_object).base.add_ref.unwrap())(&mut (*self.c_object).base);
      }
      CefGeolocationCallback {
        c_object: self.c_object,
      }
    }
  }
}

impl Drop for CefGeolocationCallback {
  fn drop(&mut self) {
    unsafe {
      if !self.c_object.is_null() {
        ((*self.c_object).base.release.unwrap())(&mut (*self.c_object).base);
      }
    }
  }
}

impl CefGeolocationCallback {
  pub unsafe fn from_c_object(c_object: *mut cef_geolocation_callback_t) -> CefGeolocationCallback {
    CefGeolocationCallback {
      c_object: c_object,
    }
  }

  pub unsafe fn from_c_object_addref(c_object: *mut cef_geolocation_callback_t) -> CefGeolocationCallback {
    if !c_object.is_null() {
      ((*c_object).base.add_ref.unwrap())(&mut (*c_object).base);
    }
    CefGeolocationCallback {
      c_object: c_object,
    }
  }

  pub fn c_object(&self) -> *mut cef_geolocation_callback_t {
    self.c_object
  }

  pub fn c_object_addrefed(&self) -> *mut cef_geolocation_callback_t {
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
  // Call to allow or deny geolocation access.
  //
  pub fn cont(&self, allow: libc::c_int) -> () {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).cont.unwrap())(
          self.c_object,
          CefWrap::to_c(allow)))
    }
  }
}

impl CefWrap<*mut cef_geolocation_callback_t> for CefGeolocationCallback {
  fn to_c(rust_object: CefGeolocationCallback) -> *mut cef_geolocation_callback_t {
    rust_object.c_object_addrefed()
  }
  unsafe fn to_rust(c_object: *mut cef_geolocation_callback_t) -> CefGeolocationCallback {
    CefGeolocationCallback::from_c_object_addref(c_object)
  }
}
impl CefWrap<*mut cef_geolocation_callback_t> for Option<CefGeolocationCallback> {
  fn to_c(rust_object: Option<CefGeolocationCallback>) -> *mut cef_geolocation_callback_t {
    match rust_object {
      None => ptr::null_mut(),
      Some(rust_object) => rust_object.c_object_addrefed(),
    }
  }
  unsafe fn to_rust(c_object: *mut cef_geolocation_callback_t) -> Option<CefGeolocationCallback> {
    if c_object.is_null() {
      None
    } else {
      Some(CefGeolocationCallback::from_c_object_addref(c_object))
    }
  }
}


//
// Implement this structure to handle events related to geolocation permission
// requests. The functions of this structure will be called on the browser
// process UI thread.
//
#[repr(C)]
pub struct _cef_geolocation_handler_t {
  //
  // Base structure.
  //
  pub base: types::cef_base_t,

  //
  // Called when a page requests permission to access geolocation information.
  // |requesting_url| is the URL requesting permission and |request_id| is the
  // unique ID for the permission request. Return true (1) and call
  // cef_geolocation_callback_t::cont() either in this function or at a later
  // time to continue or cancel the request. Return false (0) to cancel the
  // request immediately.
  //
  pub on_request_geolocation_permission: Option<extern "C" fn(
      this: *mut cef_geolocation_handler_t,
      browser: *mut interfaces::cef_browser_t,
      requesting_url: *const types::cef_string_t, request_id: libc::c_int,
      callback: *mut interfaces::cef_geolocation_callback_t) -> libc::c_int>,

  //
  // Called when a geolocation access request is canceled. |requesting_url| is
  // the URL that originally requested permission and |request_id| is the unique
  // ID for the permission request.
  //
  pub on_cancel_geolocation_permission: Option<extern "C" fn(
      this: *mut cef_geolocation_handler_t,
      browser: *mut interfaces::cef_browser_t,
      requesting_url: *const types::cef_string_t, request_id: libc::c_int) -> (
      )>,

  //
  // The reference count. This will only be present for Rust instances!
  //
  pub ref_count: uint,

  //
  // Extra data. This will only be present for Rust instances!
  //
  pub extra: u8,
}

pub type cef_geolocation_handler_t = _cef_geolocation_handler_t;


//
// Implement this structure to handle events related to geolocation permission
// requests. The functions of this structure will be called on the browser
// process UI thread.
//
pub struct CefGeolocationHandler {
  c_object: *mut cef_geolocation_handler_t,
}

impl Clone for CefGeolocationHandler {
  fn clone(&self) -> CefGeolocationHandler{
    unsafe {
      if !self.c_object.is_null() {
        ((*self.c_object).base.add_ref.unwrap())(&mut (*self.c_object).base);
      }
      CefGeolocationHandler {
        c_object: self.c_object,
      }
    }
  }
}

impl Drop for CefGeolocationHandler {
  fn drop(&mut self) {
    unsafe {
      if !self.c_object.is_null() {
        ((*self.c_object).base.release.unwrap())(&mut (*self.c_object).base);
      }
    }
  }
}

impl CefGeolocationHandler {
  pub unsafe fn from_c_object(c_object: *mut cef_geolocation_handler_t) -> CefGeolocationHandler {
    CefGeolocationHandler {
      c_object: c_object,
    }
  }

  pub unsafe fn from_c_object_addref(c_object: *mut cef_geolocation_handler_t) -> CefGeolocationHandler {
    if !c_object.is_null() {
      ((*c_object).base.add_ref.unwrap())(&mut (*c_object).base);
    }
    CefGeolocationHandler {
      c_object: c_object,
    }
  }

  pub fn c_object(&self) -> *mut cef_geolocation_handler_t {
    self.c_object
  }

  pub fn c_object_addrefed(&self) -> *mut cef_geolocation_handler_t {
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
  // Called when a page requests permission to access geolocation information.
  // |requesting_url| is the URL requesting permission and |request_id| is the
  // unique ID for the permission request. Return true (1) and call
  // cef_geolocation_callback_t::cont() either in this function or at a later
  // time to continue or cancel the request. Return false (0) to cancel the
  // request immediately.
  //
  pub fn on_request_geolocation_permission(&self,
      browser: interfaces::CefBrowser, requesting_url: &[u16],
      request_id: libc::c_int,
      callback: interfaces::CefGeolocationCallback) -> libc::c_int {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).on_request_geolocation_permission.unwrap())(
          self.c_object,
          CefWrap::to_c(browser),
          CefWrap::to_c(requesting_url),
          CefWrap::to_c(request_id),
          CefWrap::to_c(callback)))
    }
  }

  //
  // Called when a geolocation access request is canceled. |requesting_url| is
  // the URL that originally requested permission and |request_id| is the unique
  // ID for the permission request.
  //
  pub fn on_cancel_geolocation_permission(&self,
      browser: interfaces::CefBrowser, requesting_url: &[u16],
      request_id: libc::c_int) -> () {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).on_cancel_geolocation_permission.unwrap())(
          self.c_object,
          CefWrap::to_c(browser),
          CefWrap::to_c(requesting_url),
          CefWrap::to_c(request_id)))
    }
  }
}

impl CefWrap<*mut cef_geolocation_handler_t> for CefGeolocationHandler {
  fn to_c(rust_object: CefGeolocationHandler) -> *mut cef_geolocation_handler_t {
    rust_object.c_object_addrefed()
  }
  unsafe fn to_rust(c_object: *mut cef_geolocation_handler_t) -> CefGeolocationHandler {
    CefGeolocationHandler::from_c_object_addref(c_object)
  }
}
impl CefWrap<*mut cef_geolocation_handler_t> for Option<CefGeolocationHandler> {
  fn to_c(rust_object: Option<CefGeolocationHandler>) -> *mut cef_geolocation_handler_t {
    match rust_object {
      None => ptr::null_mut(),
      Some(rust_object) => rust_object.c_object_addrefed(),
    }
  }
  unsafe fn to_rust(c_object: *mut cef_geolocation_handler_t) -> Option<CefGeolocationHandler> {
    if c_object.is_null() {
      None
    } else {
      Some(CefGeolocationHandler::from_c_object_addref(c_object))
    }
  }
}

