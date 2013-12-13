/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::cast;
use std::unstable::raw::Box;
use dom::bindings::utils::{Reflector, Reflectable};
use dom::window;
use js::jsapi::{JSContext, JSObject};

pub struct JSManaged<T> {
    ptr: *mut T
}
 
impl<T: Reflectable> JSManaged<T> {
    pub fn new(mut obj: ~T,
               window:  &window::Window,
               wrap_fn: extern "Rust" fn(*JSContext, *JSObject, ~T) -> *JSObject) -> JSManaged<T> {
        let cx = window.get_cx();
        let scope = window.reflector().get_jsobject();
        let raw: *mut T = &mut *obj;
        if wrap_fn(cx, scope, obj).is_null() {
            fail!("Could not eagerly wrap object");
        }
        JSManaged {
            ptr: raw
        }
    }

    pub unsafe fn from_box(box_: *mut Box<T>) -> JSManaged<T> {
        JSManaged {
            ptr: &mut (*box_).data
        }
    }
}

impl<T: Reflectable> Reflectable for JSManaged<T> {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.value().reflector()
    }

    fn mut_reflector<'a>(&'a mut self) -> &'a mut Reflector {
        self.mut_value().mut_reflector()
    }
}
 
impl<T> JSManaged<T> {
    pub fn value<'a>(&'a self) -> &'a T {
        unsafe {
            &(*self.ptr)
        }
    }
 
    pub fn mut_value<'a>(&'a mut self) -> &'a mut T {
        unsafe {
            &mut (*self.ptr)
        }
    }
}
 
impl<From, To> JSManaged<From> {
    //XXXjdm It would be lovely if this could be private.
    pub unsafe fn transmute(self) -> JSManaged<To> {
        cast::transmute(self)
    }
}