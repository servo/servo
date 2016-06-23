/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::refcounted::Trusted;
use dom::bindings::reflector::Reflectable;
use dom::bindings::str::DOMString;
use dom::bindings::structuredclone::StructuredCloneData;
use js::jsapi::{JSRuntime, JS_RequestInterruptCallback};
use js::rust::Runtime;
use script_runtime::CommonScriptMsg;

/// Messages used to control the worker event loops
pub enum WorkerScriptMsg {
    /// Common variants associated with the script messages
    Common(CommonScriptMsg),
    /// Message sent through Worker.postMessage
    DOMMessage(StructuredCloneData),
}

pub struct SimpleWorkerErrorHandler<T: Reflectable> {
    pub addr: Trusted<T>,
}

impl<T: Reflectable> SimpleWorkerErrorHandler<T> {
    pub fn new(addr: Trusted<T>) -> SimpleWorkerErrorHandler<T> {
        SimpleWorkerErrorHandler {
            addr: addr
        }
    }
}

pub struct WorkerErrorHandler<T: Reflectable> {
    pub addr: Trusted<T>,
    pub msg: DOMString,
    pub file_name: DOMString,
    pub line_num: u32,
    pub col_num: u32,
}

impl<T: Reflectable> WorkerErrorHandler<T> {
    pub fn new(addr: Trusted<T>, msg: DOMString, file_name: DOMString, line_num: u32, col_num: u32)
            -> WorkerErrorHandler<T> {
        WorkerErrorHandler {
            addr: addr,
            msg: msg,
            file_name: file_name,
            line_num: line_num,
            col_num: col_num,
        }
    }
}

#[derive(Copy, Clone)]
pub struct SharedRt {
    pub rt: *mut JSRuntime
}

impl SharedRt {
    pub fn new(rt: &Runtime) -> SharedRt {
        SharedRt {
            rt: rt.rt()
        }
    }

    #[allow(unsafe_code)]
    pub fn request_interrupt(&self) {
        unsafe {
            JS_RequestInterruptCallback(self.rt);
        }
    }

    pub fn rt(&self) -> *mut JSRuntime {
        self.rt
    }
}
#[allow(unsafe_code)]
unsafe impl Send for SharedRt {}
