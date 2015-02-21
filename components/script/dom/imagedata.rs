/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::ImageDataBinding;
use dom::bindings::codegen::Bindings::ImageDataBinding::ImageDataMethods;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflector, reflect_dom_object};
use geom::size::Size2D;
use js::jsapi::{JSContext, JSObject};
use js::jsfriendapi::bindgen::{JS_NewUint8ClampedArray, JS_GetUint8ClampedArrayData};
use libc::uint8_t;
use std::vec::Vec;
use collections::slice::from_raw_buf;
use std::ptr;

#[dom_struct]
#[allow(raw_pointer_derive)]
pub struct ImageData {
    reflector_: Reflector,
    width: u32,
    height: u32,
    data: *mut JSObject,
}

impl ImageData {
    #[allow(unsafe_blocks)]
    fn new_inherited(width: u32, height: u32, data: Option<Vec<u8>>, global: GlobalRef) -> ImageData {
        unsafe {
            let cx = global.get_cx();
            let js_object: *mut JSObject = JS_NewUint8ClampedArray(cx, width * height * 4);

            if let Some(vec) = data {
                let js_object_data: *mut uint8_t = JS_GetUint8ClampedArrayData(js_object, cx);
                ptr::copy_nonoverlapping_memory(js_object_data, vec.as_ptr(), vec.len())
            }

            ImageData {
                reflector_: Reflector::new(),
                width: width,
                height: height,
                data: js_object,
            }
        }
    }

    pub fn new(global: GlobalRef, width: u32, height: u32, data: Option<Vec<u8>>) -> Temporary<ImageData> {
        reflect_dom_object(box ImageData::new_inherited(width, height, data, global),
                           global, ImageDataBinding::Wrap)
    }
}

pub trait ImageDataHelpers {
    fn get_data_array(self, global: &GlobalRef) -> Vec<u8>;
    fn get_size(&self) -> Size2D<i32>;
}

impl<'a> ImageDataHelpers for JSRef<'a, ImageData> {
    #[allow(unsafe_blocks)]
    fn get_data_array(self, global: &GlobalRef) -> Vec<u8> {
        unsafe {
            let cx = global.get_cx();
            let data: *const uint8_t = JS_GetUint8ClampedArrayData(self.Data(cx), cx) as *const uint8_t;
            let len = self.Width() * self.Height() * 4;
            from_raw_buf(&data, len as uint).to_vec()
        }
    }

    fn get_size(&self) -> Size2D<i32> {
        Size2D(self.Width() as i32, self.Height() as i32)
    }
}

impl<'a> ImageDataMethods for JSRef<'a, ImageData> {
    fn Width(self) -> u32 {
        self.width
    }

    fn Height(self) -> u32 {
        self.height
    }

    fn Data(self, _: *mut JSContext) -> *mut JSObject {
        self.data
    }
}
