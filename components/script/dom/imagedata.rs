/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use core::nonzero::NonZero;
use dom::bindings::codegen::Bindings::ImageDataBinding;
use dom::bindings::codegen::Bindings::ImageDataBinding::ImageDataMethods;
use dom::bindings::js::Root;
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::globalscope::GlobalScope;
use euclid::size::Size2D;
use js::jsapi::{Heap, JSContext, JSObject};
use js::jsapi::{JS_GetUint8ClampedArrayData, JS_NewUint8ClampedArray};
use libc::uint8_t;
use std::default::Default;
use std::ptr;
use std::slice;
use std::vec::Vec;

#[dom_struct]
pub struct ImageData {
    reflector_: Reflector,
    width: u32,
    height: u32,
    data: Heap<*mut JSObject>,
}

impl ImageData {
    #[allow(unsafe_code)]
    pub fn new(global: &GlobalScope, width: u32, height: u32, data: Option<Vec<u8>>) -> Root<ImageData> {
        let mut imagedata = box ImageData {
            reflector_: Reflector::new(),
            width: width,
            height: height,
            data: Heap::default(),
        };

        unsafe {
            let cx = global.get_cx();
            let js_object: *mut JSObject = JS_NewUint8ClampedArray(cx, width * height * 4);
            assert!(!js_object.is_null());

            if let Some(vec) = data {
                let mut is_shared = false;
                let js_object_data: *mut uint8_t = JS_GetUint8ClampedArrayData(js_object, &mut is_shared, ptr::null());
                assert!(!is_shared);
                ptr::copy_nonoverlapping(vec.as_ptr(), js_object_data, vec.len())
            }
            (*imagedata).data.set(js_object);
        }

        reflect_dom_object(imagedata,
                           global, ImageDataBinding::Wrap)
    }

    #[allow(unsafe_code)]
    pub fn get_data_array(&self) -> Vec<u8> {
        unsafe {
            let mut is_shared = false;
            assert!(!self.data.get().is_null());
            let data: *const uint8_t =
                JS_GetUint8ClampedArrayData(self.data.get(), &mut is_shared, ptr::null()) as *const uint8_t;
            assert!(!data.is_null());
            assert!(!is_shared);
            let len = self.Width() * self.Height() * 4;
            slice::from_raw_parts(data, len as usize).to_vec()
        }
    }

    pub fn get_size(&self) -> Size2D<i32> {
        Size2D::new(self.Width() as i32, self.Height() as i32)
    }
}

impl ImageDataMethods for ImageData {
    // https://html.spec.whatwg.org/multipage/#dom-imagedata-width
    fn Width(&self) -> u32 {
        self.width
    }

    // https://html.spec.whatwg.org/multipage/#dom-imagedata-height
    fn Height(&self) -> u32 {
        self.height
    }

    #[allow(unsafe_code)]
    // https://html.spec.whatwg.org/multipage/#dom-imagedata-data
    unsafe fn Data(&self, _: *mut JSContext) -> NonZero<*mut JSObject> {
        assert!(!self.data.get().is_null());
        NonZero::new(self.data.get())
    }
}
