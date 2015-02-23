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
// Implement this structure to handle events related to dragging. The functions
// of this structure will be called on the UI thread.
//
#[repr(C)]
pub struct _cef_drag_handler_t {
  //
  // Base structure.
  //
  pub base: types::cef_base_t,

  //
  // Called when an external drag event enters the browser window. |dragData|
  // contains the drag event data and |mask| represents the type of drag
  // operation. Return false (0) for default drag handling behavior or true (1)
  // to cancel the drag event.
  //
  pub on_drag_enter: Option<extern "C" fn(this: *mut cef_drag_handler_t,
      browser: *mut interfaces::cef_browser_t,
      dragData: *mut interfaces::cef_drag_data_t,
      mask: types::cef_drag_operations_mask_t) -> libc::c_int>,

  //
  // The reference count. This will only be present for Rust instances!
  //
  pub ref_count: uint,

  //
  // Extra data. This will only be present for Rust instances!
  //
  pub extra: u8,
}

pub type cef_drag_handler_t = _cef_drag_handler_t;


//
// Implement this structure to handle events related to dragging. The functions
// of this structure will be called on the UI thread.
//
pub struct CefDragHandler {
  c_object: *mut cef_drag_handler_t,
}

impl Clone for CefDragHandler {
  fn clone(&self) -> CefDragHandler{
    unsafe {
      if !self.c_object.is_null() {
        ((*self.c_object).base.add_ref.unwrap())(&mut (*self.c_object).base);
      }
      CefDragHandler {
        c_object: self.c_object,
      }
    }
  }
}

impl Drop for CefDragHandler {
  fn drop(&mut self) {
    unsafe {
      if !self.c_object.is_null() {
        ((*self.c_object).base.release.unwrap())(&mut (*self.c_object).base);
      }
    }
  }
}

impl CefDragHandler {
  pub unsafe fn from_c_object(c_object: *mut cef_drag_handler_t) -> CefDragHandler {
    CefDragHandler {
      c_object: c_object,
    }
  }

  pub unsafe fn from_c_object_addref(c_object: *mut cef_drag_handler_t) -> CefDragHandler {
    if !c_object.is_null() {
      ((*c_object).base.add_ref.unwrap())(&mut (*c_object).base);
    }
    CefDragHandler {
      c_object: c_object,
    }
  }

  pub fn c_object(&self) -> *mut cef_drag_handler_t {
    self.c_object
  }

  pub fn c_object_addrefed(&self) -> *mut cef_drag_handler_t {
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
  // Called when an external drag event enters the browser window. |dragData|
  // contains the drag event data and |mask| represents the type of drag
  // operation. Return false (0) for default drag handling behavior or true (1)
  // to cancel the drag event.
  //
  pub fn on_drag_enter(&self, browser: interfaces::CefBrowser,
      dragData: interfaces::CefDragData,
      mask: types::cef_drag_operations_mask_t) -> libc::c_int {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).on_drag_enter.unwrap())(
          self.c_object,
          CefWrap::to_c(browser),
          CefWrap::to_c(dragData),
          CefWrap::to_c(mask)))
    }
  }
}

impl CefWrap<*mut cef_drag_handler_t> for CefDragHandler {
  fn to_c(rust_object: CefDragHandler) -> *mut cef_drag_handler_t {
    rust_object.c_object_addrefed()
  }
  unsafe fn to_rust(c_object: *mut cef_drag_handler_t) -> CefDragHandler {
    CefDragHandler::from_c_object_addref(c_object)
  }
}
impl CefWrap<*mut cef_drag_handler_t> for Option<CefDragHandler> {
  fn to_c(rust_object: Option<CefDragHandler>) -> *mut cef_drag_handler_t {
    match rust_object {
      None => ptr::null_mut(),
      Some(rust_object) => rust_object.c_object_addrefed(),
    }
  }
  unsafe fn to_rust(c_object: *mut cef_drag_handler_t) -> Option<CefDragHandler> {
    if c_object.is_null() {
      None
    } else {
      Some(CefDragHandler::from_c_object_addref(c_object))
    }
  }
}

