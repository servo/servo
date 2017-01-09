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
use js::rust::Runtime;
use js::typedarray::Uint8ClampedArray;
use std::default::Default;
use std::ptr;
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
            rooted!(in (cx) let mut js_object = ptr::null_mut());
            let data = data.as_ref().map(|d| &d[..]);
            Uint8ClampedArray::create(cx, width * height * 4, data, js_object.handle_mut()).unwrap();
            (*imagedata).data.set(js_object.get());
        }

        reflect_dom_object(imagedata,
                           global, ImageDataBinding::Wrap)
    }

    #[allow(unsafe_code)]
    pub fn get_data_array(&self) -> Vec<u8> {
        unsafe {
            assert!(!self.data.get().is_null());
            let cx = Runtime::get();
            assert!(!cx.is_null());
            typedarray!(in(cx) let array: Uint8ClampedArray = self.data.get());
            let vec = array.unwrap().as_slice().to_vec();
            vec
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
