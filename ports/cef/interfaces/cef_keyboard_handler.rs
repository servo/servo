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
// Implement this structure to handle events related to keyboard input. The
// functions of this structure will be called on the UI thread.
//
#[repr(C)]
pub struct _cef_keyboard_handler_t {
  //
  // Base structure.
  //
  pub base: types::cef_base_t,

  // Called before a keyboard event is sent to the renderer. |event| contains
  // information about the keyboard event. |os_event| is the operating system
  // event message, if any. Return true (1) if the event was handled or false
  // (0) otherwise. If the event will be handled in on_key_event() as a keyboard
  // shortcut set |is_keyboard_shortcut| to true (1) and return false (0).
  pub on_pre_key_event: Option<extern "C" fn(this: *mut cef_keyboard_handler_t,
      browser: *mut interfaces::cef_browser_t,
      event: *const interfaces::cef_key_event_t,
      os_event: types::cef_event_handle_t,
      is_keyboard_shortcut: *mut libc::c_int) -> libc::c_int>,

  //
  // Called after the renderer and JavaScript in the page has had a chance to
  // handle the event. |event| contains information about the keyboard event.
  // |os_event| is the operating system event message, if any. Return true (1)
  // if the keyboard event was handled or false (0) otherwise.
  //
  pub on_key_event: Option<extern "C" fn(this: *mut cef_keyboard_handler_t,
      browser: *mut interfaces::cef_browser_t,
      event: *const interfaces::cef_key_event_t,
      os_event: types::cef_event_handle_t) -> libc::c_int>,

  //
  // The reference count. This will only be present for Rust instances!
  //
  pub ref_count: uint,

  //
  // Extra data. This will only be present for Rust instances!
  //
  pub extra: u8,
}

pub type cef_keyboard_handler_t = _cef_keyboard_handler_t;


//
// Implement this structure to handle events related to keyboard input. The
// functions of this structure will be called on the UI thread.
//
pub struct CefKeyboardHandler {
  c_object: *mut cef_keyboard_handler_t,
}

impl Clone for CefKeyboardHandler {
  fn clone(&self) -> CefKeyboardHandler{
    unsafe {
      if !self.c_object.is_null() {
        ((*self.c_object).base.add_ref.unwrap())(&mut (*self.c_object).base);
      }
      CefKeyboardHandler {
        c_object: self.c_object,
      }
    }
  }
}

impl Drop for CefKeyboardHandler {
  fn drop(&mut self) {
    unsafe {
      if !self.c_object.is_null() {
        ((*self.c_object).base.release.unwrap())(&mut (*self.c_object).base);
      }
    }
  }
}

impl CefKeyboardHandler {
  pub unsafe fn from_c_object(c_object: *mut cef_keyboard_handler_t) -> CefKeyboardHandler {
    CefKeyboardHandler {
      c_object: c_object,
    }
  }

  pub unsafe fn from_c_object_addref(c_object: *mut cef_keyboard_handler_t) -> CefKeyboardHandler {
    if !c_object.is_null() {
      ((*c_object).base.add_ref.unwrap())(&mut (*c_object).base);
    }
    CefKeyboardHandler {
      c_object: c_object,
    }
  }

  pub fn c_object(&self) -> *mut cef_keyboard_handler_t {
    self.c_object
  }

  pub fn c_object_addrefed(&self) -> *mut cef_keyboard_handler_t {
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

  // Called before a keyboard event is sent to the renderer. |event| contains
  // information about the keyboard event. |os_event| is the operating system
  // event message, if any. Return true (1) if the event was handled or false
  // (0) otherwise. If the event will be handled in on_key_event() as a keyboard
  // shortcut set |is_keyboard_shortcut| to true (1) and return false (0).
  pub fn on_pre_key_event(&self, browser: interfaces::CefBrowser,
      event: &interfaces::CefKeyEvent, os_event: types::cef_event_handle_t,
      is_keyboard_shortcut: &mut libc::c_int) -> libc::c_int {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).on_pre_key_event.unwrap())(
          self.c_object,
          CefWrap::to_c(browser),
          CefWrap::to_c(event),
          CefWrap::to_c(os_event),
          CefWrap::to_c(is_keyboard_shortcut)))
    }
  }

  //
  // Called after the renderer and JavaScript in the page has had a chance to
  // handle the event. |event| contains information about the keyboard event.
  // |os_event| is the operating system event message, if any. Return true (1)
  // if the keyboard event was handled or false (0) otherwise.
  //
  pub fn on_key_event(&self, browser: interfaces::CefBrowser,
      event: &interfaces::CefKeyEvent,
      os_event: types::cef_event_handle_t) -> libc::c_int {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).on_key_event.unwrap())(
          self.c_object,
          CefWrap::to_c(browser),
          CefWrap::to_c(event),
          CefWrap::to_c(os_event)))
    }
  }
}

impl CefWrap<*mut cef_keyboard_handler_t> for CefKeyboardHandler {
  fn to_c(rust_object: CefKeyboardHandler) -> *mut cef_keyboard_handler_t {
    rust_object.c_object_addrefed()
  }
  unsafe fn to_rust(c_object: *mut cef_keyboard_handler_t) -> CefKeyboardHandler {
    CefKeyboardHandler::from_c_object_addref(c_object)
  }
}
impl CefWrap<*mut cef_keyboard_handler_t> for Option<CefKeyboardHandler> {
  fn to_c(rust_object: Option<CefKeyboardHandler>) -> *mut cef_keyboard_handler_t {
    match rust_object {
      None => ptr::null_mut(),
      Some(rust_object) => rust_object.c_object_addrefed(),
    }
  }
  unsafe fn to_rust(c_object: *mut cef_keyboard_handler_t) -> Option<CefKeyboardHandler> {
    if c_object.is_null() {
      None
    } else {
      Some(CefKeyboardHandler::from_c_object_addref(c_object))
    }
  }
}

