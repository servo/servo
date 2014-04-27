/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::{Reflector, Reflectable};
use dom::window::Window;
use js::jsapi::JSContext;
use layout_interface::TrustedNodeAddress;

use std::cast;
use std::cell::RefCell;

pub struct JS<T> {
    ptr: RefCell<*mut T>
}

impl<T> Eq for JS<T> {
    fn eq(&self, other: &JS<T>) -> bool {
        self.ptr == other.ptr
    }
}

impl <T> Clone for JS<T> {
    #[inline]
    fn clone(&self) -> JS<T> {
        JS {
            ptr: self.ptr.clone()
        }
    }
}

impl<T: Reflectable> JS<T> {
    pub fn new(obj: ~T,
               window:  &JS<Window>,
               wrap_fn: extern "Rust" fn(*JSContext, &JS<Window>, ~T) -> JS<T>) -> JS<T> {
        wrap_fn(window.get().get_cx(), window, obj)
    }

    pub unsafe fn from_raw(raw: *mut T) -> JS<T> {
        JS {
            ptr: RefCell::new(raw)
        }
    }


    pub unsafe fn from_trusted_node_address(inner: TrustedNodeAddress) -> JS<T> {
        let TrustedNodeAddress(addr) = inner;
        JS {
            ptr: RefCell::new(addr as *mut T)
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
            &**borrowed
        }
    }

    pub fn get_mut<'a>(&'a mut self) -> &'a mut T {
        let mut borrowed = self.ptr.borrow_mut();
        unsafe {
            &mut **borrowed
        }
    }

    /// Returns an unsafe pointer to the interior of this JS object without touching the borrow
    /// flags. This is the only method that be safely accessed from layout. (The fact that this
    /// is unsafe is what necessitates the layout wrappers.)
    pub unsafe fn unsafe_get(&self) -> *mut T {
        cast::transmute_copy(&self.ptr)
    }
}

impl<From, To> JS<From> {
    //XXXjdm It would be lovely if this could be private.
    pub unsafe fn transmute(self) -> JS<To> {
        cast::transmute(self)
    }

    pub unsafe fn transmute_copy(&self) -> JS<To> {
        cast::transmute_copy(self)
    }
}
