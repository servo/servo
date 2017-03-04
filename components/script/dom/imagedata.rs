/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use core::nonzero::NonZero;
use dom::bindings::codegen::Bindings::ImageDataBinding;
use dom::bindings::codegen::Bindings::ImageDataBinding::ImageDataMethods;
use dom::bindings::error::{Fallible, Error};
use dom::bindings::js::Root;
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::globalscope::GlobalScope;
use dom_struct::dom_struct;
use euclid::size::Size2D;
use js::jsapi::{Heap, JSContext, JSObject};
use js::rust::Runtime;
use js::typedarray::{Uint8ClampedArray, CreateWith};
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
    pub fn new(global: &GlobalScope, width: u32, height: u32, mut data: Option<Vec<u8>>) -> Root<ImageData> {
        let imagedata = box ImageData {
            reflector_: Reflector::new(),
            width: width,
            height: height,
            data: Heap::default(),
        };
        let len = width * height * 4;

        unsafe {
            let cx = global.get_cx();
            rooted!(in (cx) let mut js_object = ptr::null_mut());
            let data = match data {
                Some(ref mut d) => {
                    d.resize(len as usize, 0);
                    CreateWith::Slice(&d[..])
                },
                None => CreateWith::Length(len),
            };
            Uint8ClampedArray::create(cx, data, js_object.handle_mut()).unwrap();
            (*imagedata).data.set(js_object.get());
        }

        reflect_dom_object(imagedata,
                           global, ImageDataBinding::Wrap)
    }

    #[allow(unsafe_code)]
    unsafe fn new_with_jsobject(global: &GlobalScope, width: u32, height: u32, data: *mut JSObject) -> Root<ImageData> {
        let imagedata = box ImageData {
            reflector_: Reflector::new(),
            width: width,
            height: height,
            data: Heap::default(),
        };

        (*imagedata).data.set(data);
        reflect_dom_object(imagedata, global, ImageDataBinding::Wrap)
    }

    //https://html.spec.whatwg.org/multipage/#pixel-manipulation:dom-imagedata-3
    #[allow(unsafe_code)]
    pub fn Constructor(global: &GlobalScope, width: u32, height: u32) -> Fallible<Root<Self>> {
        if width == 0 || height == 0 {
            return Err(Error::IndexSize);
        }
        unsafe {
            let len = width * height * 4;
            let cx = global.get_cx();
            rooted!(in (cx) let mut array = ptr::null_mut());
            Uint8ClampedArray::create(cx, CreateWith::Length(len), array.handle_mut()).unwrap();
            Ok(Self::new_with_jsobject(global, width, height, array.get()))
        }
    }

    //https://html.spec.whatwg.org/multipage/#pixel-manipulation:dom-imagedata-4
    #[allow(unsafe_code)]
    pub unsafe fn Constructor_(cx: *mut JSContext,
                        global: &GlobalScope,
                        data: *mut JSObject,
                        width: u32,
                        opt_height: Option<u32>)
                        -> Fallible<Root<Self>> {
        typedarray!(in(cx) let data_res: Uint8ClampedArray = data);
        let mut array = match data_res {
            Ok(data) => data,
            Err(_)   => {
                return Err(Error::Type("Argument to Image data is not an ArrayBufferView".to_owned()));
            }
        };

        let byte_len = array.as_slice().len() as u32;
        if byte_len % 4 != 0 {
            return Err(Error::InvalidState);
        }

        let len = byte_len / 4;
        if width == 0 || len % width != 0 {
            return Err(Error::IndexSize);
        }

        let height = len / width;
        if opt_height.map_or(false, |x| height != x) {
            return Err(Error::IndexSize);
        }

        Ok(Self::new_with_jsobject(global, width, height, data))
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
