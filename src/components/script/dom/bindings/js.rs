/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::{Reflector, Reflectable};
use dom::window;
use js::jsapi::{JSContext, JSObject};
use layout_interface::TrustedNodeAddress;

use std::cast;
use std::cell::RefCell;
use std::unstable::raw::Box;

pub struct JS<T> {
    priv ptr: RefCell<*mut T>
}

impl<T> Eq for JS<T> {
    fn eq(&self, other: &JS<T>) -> bool {
        self.ptr == other.ptr
    }
}

impl <T> Clone for JS<T> {
    fn clone(&self) -> JS<T> {
        JS {
            ptr: self.ptr.clone()
        }
    }
}

impl<T: Reflectable> JS<T> {
    pub fn new(mut obj: ~T,
               window:  &window::Window,
               wrap_fn: extern "Rust" fn(*JSContext, *JSObject, ~T) -> *JSObject) -> JS<T> {
        let cx = window.get_cx();
        let scope = window.reflector().get_jsobject();
        let raw: *mut T = &mut *obj;
        if wrap_fn(cx, scope, obj).is_null() {
            fail!("Could not eagerly wrap object");
        }
        JS {
            ptr: RefCell::new(raw)
        }
    }

    pub unsafe fn from_raw(raw: *mut T) -> JS<T> {
        JS {
            ptr: RefCell::new(raw)
        }
    }


    pub unsafe fn from_box(box_: *mut Box<T>) -> JS<T> {
        let raw: *mut T = &mut (*box_).data;
        JS {
            ptr: RefCell::new(raw)
        }
    }

    pub unsafe fn from_trusted_node_address(inner: TrustedNodeAddress) -> JS<T> {
        JS {
            ptr: RefCell::new(inner as *mut T)
        }
    }
}

impl<T: Reflectable> Reflectable for JS<T> {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.get().reflector()
    }

    fn mut_reflector<'a>(&'a mut self) -> &'a mut Reflector {
        self.get_mut().mut_reflector()
    }
}

impl<T> JS<T> {
    pub fn get<'a>(&'a self) -> &'a T {
        let borrowed = self.ptr.borrow();
        unsafe {
            &(**borrowed.get())
        }
    }

    pub fn get_mut<'a>(&'a mut self) -> &'a mut T {
        let mut borrowed = self.ptr.borrow_mut();
        unsafe {
            &mut (**borrowed.get())
        }
    }
}

impl<From, To> JS<From> {
    //XXXjdm It would be lovely if this could be private.
    pub unsafe fn transmute(self) -> JS<To> {
        cast::transmute(self)
    }
}
