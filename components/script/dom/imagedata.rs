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
    pub fn from_vec(global: &GlobalScope, width: u32, height: u32, mut data: Option<Vec<u8>>) -> Root<ImageData> {
        unsafe {
            let len = width * height * 4;
            let cx = global.get_cx();
            rooted!(in (cx) let mut typedarray = ptr::null_mut());
            let data = match data {
                Some(ref mut d) => {
                    d.resize(len as usize, 0);
                    CreateWith::Slice(&d[..])
                },
                None => CreateWith::Length(len),
            };
            Uint8ClampedArray::create(cx, data, typedarray.handle_mut()).unwrap();
            Self::new(global, width, height, Some(typedarray.get()))
        }
    }

    #[allow(unsafe_code)]
    pub fn new(global: &GlobalScope, width: u32, height: u32, data: Option<*mut JSObject>) -> Root<ImageData> {
        let imagedata = box ImageData {
            reflector_: Reflector::new(),
            width: width,
            height: height,
            data: Heap::default(),
        };

        let len = width * height * 4;
        match data {
            Some(jsobject) => (*imagedata).data.set(jsobject),
            None => {
                unsafe {
                    let cx = global.get_cx();
                    rooted!(in (cx) let mut typedarray = ptr::null_mut());
                    Uint8ClampedArray::create(cx, CreateWith::Length(len), typedarray.handle_mut()).unwrap();
                    (*imagedata).data.set(typedarray.get());
                }
            },
        };
        reflect_dom_object(imagedata, global, ImageDataBinding::Wrap)
    }

    #[allow(unsafe_code)]
    pub fn get_data_array(&self) -> Vec<u8> {
        unsafe {
            assert!(!self.data.get().is_null());
            let cx = Runtime::get();
            assert!(!cx.is_null());
            typedarray!(in(cx) let typedarray: Uint8ClampedArray = self.data.get());
            let vec = typedarray.unwrap().as_slice().to_vec();
            vec
        }
    }

    pub fn get_size(&self) -> Size2D<i32> {
        Size2D::new(self.Width() as i32, self.Height() as i32)
    }

    pub fn Constructor(global: &GlobalScope, width: u32, height: u32) -> Fallible<Root<Self>> {
        if width == 0 || height == 0 {
            Err(Error::IndexSize)
        } else {
            Ok(Self::new(global, width, height, None))
        }
    }

    #[allow(unsafe_code)]
    pub fn Constructor_(cx: *mut JSContext,
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
        let mut len;
        unsafe {
            len = array.as_slice().len() as u32;
        }
        if len % 4 != 0 {
            Err(Error::InvalidState)
        } else {
            len /= 4;
            let height = len / width;
            if len % width != 0 {
                Err(Error::IndexSize)
            } else if opt_height.is_some() && height != opt_height.unwrap() {
                Err(Error::IndexSize)
            } else {
                Ok(Self::new(global, width, height, Some(data)))
            }
        }
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
