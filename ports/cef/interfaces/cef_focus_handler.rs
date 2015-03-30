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
// Implement this structure to handle events related to focus. The functions of
// this structure will be called on the UI thread.
//
#[repr(C)]
pub struct _cef_focus_handler_t {
  //
  // Base structure.
  //
  pub base: types::cef_base_t,

  //
  // Called when the browser component is about to loose focus. For instance, if
  // focus was on the last HTML element and the user pressed the TAB key. |next|
  // will be true (1) if the browser is giving focus to the next component and
  // false (0) if the browser is giving focus to the previous component.
  //
  pub on_take_focus: Option<extern "C" fn(this: *mut cef_focus_handler_t,
      browser: *mut interfaces::cef_browser_t, next: libc::c_int) -> ()>,

  //
  // Called when the browser component is requesting focus. |source| indicates
  // where the focus request is originating from. Return false (0) to allow the
  // focus to be set or true (1) to cancel setting the focus.
  //
  pub on_set_focus: Option<extern "C" fn(this: *mut cef_focus_handler_t,
      browser: *mut interfaces::cef_browser_t,
      source: types::cef_focus_source_t) -> libc::c_int>,

  //
  // Called when the browser component has received focus.
  //
  pub on_got_focus: Option<extern "C" fn(this: *mut cef_focus_handler_t,
      browser: *mut interfaces::cef_browser_t) -> ()>,

  //
  // The reference count. This will only be present for Rust instances!
  //
  pub ref_count: uint,

  //
  // Extra data. This will only be present for Rust instances!
  //
  pub extra: u8,
}

pub type cef_focus_handler_t = _cef_focus_handler_t;


//
// Implement this structure to handle events related to focus. The functions of
// this structure will be called on the UI thread.
//
pub struct CefFocusHandler {
  c_object: *mut cef_focus_handler_t,
}

impl Clone for CefFocusHandler {
  fn clone(&self) -> CefFocusHandler{
    unsafe {
      if !self.c_object.is_null() {
        ((*self.c_object).base.add_ref.unwrap())(&mut (*self.c_object).base);
      }
      CefFocusHandler {
        c_object: self.c_object,
      }
    }
  }
}

impl Drop for CefFocusHandler {
  fn drop(&mut self) {
    unsafe {
      if !self.c_object.is_null() {
        ((*self.c_object).base.release.unwrap())(&mut (*self.c_object).base);
      }
    }
  }
}

impl CefFocusHandler {
  pub unsafe fn from_c_object(c_object: *mut cef_focus_handler_t) -> CefFocusHandler {
    CefFocusHandler {
      c_object: c_object,
    }
  }

  pub unsafe fn from_c_object_addref(c_object: *mut cef_focus_handler_t) -> CefFocusHandler {
    if !c_object.is_null() {
      ((*c_object).base.add_ref.unwrap())(&mut (*c_object).base);
    }
    CefFocusHandler {
      c_object: c_object,
    }
  }

  pub fn c_object(&self) -> *mut cef_focus_handler_t {
    self.c_object
  }

  pub fn c_object_addrefed(&self) -> *mut cef_focus_handler_t {
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
  // Called when the browser component is about to loose focus. For instance, if
  // focus was on the last HTML element and the user pressed the TAB key. |next|
  // will be true (1) if the browser is giving focus to the next component and
  // false (0) if the browser is giving focus to the previous component.
  //
  pub fn on_take_focus(&self, browser: interfaces::CefBrowser,
      next: libc::c_int) -> () {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).on_take_focus.unwrap())(
          self.c_object,
          CefWrap::to_c(browser),
          CefWrap::to_c(next)))
    }
  }

  //
  // Called when the browser component is requesting focus. |source| indicates
  // where the focus request is originating from. Return false (0) to allow the
  // focus to be set or true (1) to cancel setting the focus.
  //
  pub fn on_set_focus(&self, browser: interfaces::CefBrowser,
      source: types::cef_focus_source_t) -> libc::c_int {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).on_set_focus.unwrap())(
          self.c_object,
          CefWrap::to_c(browser),
          CefWrap::to_c(source)))
    }
  }

  //
  // Called when the browser component has received focus.
  //
  pub fn on_got_focus(&self, browser: interfaces::CefBrowser) -> () {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).on_got_focus.unwrap())(
          self.c_object,
          CefWrap::to_c(browser)))
    }
  }
}

impl CefWrap<*mut cef_focus_handler_t> for CefFocusHandler {
  fn to_c(rust_object: CefFocusHandler) -> *mut cef_focus_handler_t {
    rust_object.c_object_addrefed()
  }
  unsafe fn to_rust(c_object: *mut cef_focus_handler_t) -> CefFocusHandler {
    CefFocusHandler::from_c_object_addref(c_object)
  }
}
impl CefWrap<*mut cef_focus_handler_t> for Option<CefFocusHandler> {
  fn to_c(rust_object: Option<CefFocusHandler>) -> *mut cef_focus_handler_t {
    match rust_object {
      None => ptr::null_mut(),
      Some(rust_object) => rust_object.c_object_addrefed(),
    }
  }
  unsafe fn to_rust(c_object: *mut cef_focus_handler_t) -> Option<CefFocusHandler> {
    if c_object.is_null() {
      None
    } else {
      Some(CefFocusHandler::from_c_object_addref(c_object))
    }
  }
}

