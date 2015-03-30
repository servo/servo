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
// Implement this structure to receive notification when tracing has completed.
// The functions of this structure will be called on the browser process UI
// thread.
//
#[repr(C)]
pub struct _cef_end_tracing_callback_t {
  //
  // Base structure.
  //
  pub base: types::cef_base_t,

  //
  // Called after all processes have sent their trace data. |tracing_file| is
  // the path at which tracing data was written. The client is responsible for
  // deleting |tracing_file|.
  //
  pub on_end_tracing_complete: Option<extern "C" fn(
      this: *mut cef_end_tracing_callback_t,
      tracing_file: *const types::cef_string_t) -> ()>,

  //
  // The reference count. This will only be present for Rust instances!
  //
  pub ref_count: uint,

  //
  // Extra data. This will only be present for Rust instances!
  //
  pub extra: u8,
}

pub type cef_end_tracing_callback_t = _cef_end_tracing_callback_t;


//
// Implement this structure to receive notification when tracing has completed.
// The functions of this structure will be called on the browser process UI
// thread.
//
pub struct CefEndTracingCallback {
  c_object: *mut cef_end_tracing_callback_t,
}

impl Clone for CefEndTracingCallback {
  fn clone(&self) -> CefEndTracingCallback{
    unsafe {
      if !self.c_object.is_null() {
        ((*self.c_object).base.add_ref.unwrap())(&mut (*self.c_object).base);
      }
      CefEndTracingCallback {
        c_object: self.c_object,
      }
    }
  }
}

impl Drop for CefEndTracingCallback {
  fn drop(&mut self) {
    unsafe {
      if !self.c_object.is_null() {
        ((*self.c_object).base.release.unwrap())(&mut (*self.c_object).base);
      }
    }
  }
}

impl CefEndTracingCallback {
  pub unsafe fn from_c_object(c_object: *mut cef_end_tracing_callback_t) -> CefEndTracingCallback {
    CefEndTracingCallback {
      c_object: c_object,
    }
  }

  pub unsafe fn from_c_object_addref(c_object: *mut cef_end_tracing_callback_t) -> CefEndTracingCallback {
    if !c_object.is_null() {
      ((*c_object).base.add_ref.unwrap())(&mut (*c_object).base);
    }
    CefEndTracingCallback {
      c_object: c_object,
    }
  }

  pub fn c_object(&self) -> *mut cef_end_tracing_callback_t {
    self.c_object
  }

  pub fn c_object_addrefed(&self) -> *mut cef_end_tracing_callback_t {
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
  // Called after all processes have sent their trace data. |tracing_file| is
  // the path at which tracing data was written. The client is responsible for
  // deleting |tracing_file|.
  //
  pub fn on_end_tracing_complete(&self, tracing_file: &[u16]) -> () {
    if self.c_object.is_null() {
      panic!("called a CEF method on a null object")
    }
    unsafe {
      CefWrap::to_rust(
        ((*self.c_object).on_end_tracing_complete.unwrap())(
          self.c_object,
          CefWrap::to_c(tracing_file)))
    }
  }
}

impl CefWrap<*mut cef_end_tracing_callback_t> for CefEndTracingCallback {
  fn to_c(rust_object: CefEndTracingCallback) -> *mut cef_end_tracing_callback_t {
    rust_object.c_object_addrefed()
  }
  unsafe fn to_rust(c_object: *mut cef_end_tracing_callback_t) -> CefEndTracingCallback {
    CefEndTracingCallback::from_c_object_addref(c_object)
  }
}
impl CefWrap<*mut cef_end_tracing_callback_t> for Option<CefEndTracingCallback> {
  fn to_c(rust_object: Option<CefEndTracingCallback>) -> *mut cef_end_tracing_callback_t {
    match rust_object {
      None => ptr::null_mut(),
      Some(rust_object) => rust_object.c_object_addrefed(),
    }
  }
  unsafe fn to_rust(c_object: *mut cef_end_tracing_callback_t) -> Option<CefEndTracingCallback> {
    if c_object.is_null() {
      None
    } else {
      Some(CefEndTracingCallback::from_c_object_addref(c_object))
    }
  }
}

