/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::ops;
use std::ptr;
use winapi::Interface;
use winapi::ctypes::c_void;
use winapi::shared::guiddef::GUID;
use winapi::shared::winerror::HRESULT;
use winapi::shared::winerror::SUCCEEDED;
use winapi::um::unknwnbase::IUnknown;

pub fn as_ptr<T>(x: &T) -> *mut T {
    x as *const T as _
}

pub trait CheckHResult {
    fn check_hresult(self);
}

impl CheckHResult for HRESULT {
    fn check_hresult(self) {
        if !SUCCEEDED(self) {
            panic_com(self)
        }
    }
}

fn panic_com(hresult: HRESULT) -> ! {
    panic!("COM error 0x{:08X}", hresult as u32)
}

/// Forked from <https://github.com/retep998/wio-rs/blob/44093f7db8/src/com.rs>
#[derive(PartialEq, Debug)]
pub struct ComPtr<T>(*mut T) where T: Interface;

impl<T> ComPtr<T> where T: Interface {
    /// Creates a `ComPtr` to wrap a raw pointer.
    /// It takes ownership over the pointer which means it does __not__ call `AddRef`.
    /// `T` __must__ be a COM interface that inherits from `IUnknown`.
    pub unsafe fn from_raw(ptr: *mut T) -> ComPtr<T> {
        assert!(!ptr.is_null());
        ComPtr(ptr)
    }

    /// For use with APIs that take an interface UUID and
    /// "return" a new COM object through a `*mut *mut c_void` out-parameter.
    pub unsafe fn new_with_uuid<F>(f: F) -> Self
        where F: FnOnce(&GUID, *mut *mut c_void) -> HRESULT
    {
        Self::new_with(|ptr| f(&T::uuidof(), ptr as _))
    }

    /// For use with APIs that "return" a new COM object through a `*mut *mut T` out-parameter.
    pub unsafe fn new_with<F>(f: F) -> Self
        where F: FnOnce(*mut *mut T) -> HRESULT
    {
        let mut ptr = ptr::null_mut();
        let hresult = f(&mut ptr);
        if SUCCEEDED(hresult) {
            ComPtr::from_raw(ptr)
        } else {
            if !ptr.is_null() {
                let ptr = ptr as *mut IUnknown;
                (*ptr).Release();
            }
            panic_com(hresult)
        }
    }

    pub fn as_raw(&self) -> *mut T {
        self.0
    }

    fn as_unknown(&self) -> &IUnknown {
        unsafe {
            &*(self.0 as *mut IUnknown)
        }
    }

    /// Performs QueryInterface fun.
    pub fn cast<U>(&self) -> ComPtr<U> where U: Interface {
        unsafe {
            ComPtr::<U>::new_with_uuid(|uuid, ptr| self.as_unknown().QueryInterface(uuid, ptr))
        }
    }
}

impl<T> ops::Deref for ComPtr<T> where T: Interface {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.0 }
    }
}

impl<T> Clone for ComPtr<T> where T: Interface {
    fn clone(&self) -> Self {
        unsafe {
            self.as_unknown().AddRef();
            ComPtr(self.0)
        }
    }
}

impl<T> Drop for ComPtr<T> where T: Interface {
    fn drop(&mut self) {
        unsafe {
            self.as_unknown().Release();
        }
    }
}
