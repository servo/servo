/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::ImageDataBinding;
use dom::bindings::codegen::Bindings::ImageDataBinding::ImageDataMethods;
use dom::bindings::error::{Error, Fallible};
use dom::bindings::global::GlobalRef;
use dom::bindings::js::Root;
use dom::bindings::typedarray::{Uint8ClampedArray, TypedArrayRooter};
use dom::bindings::utils::{Reflector, reflect_dom_object};
use euclid::size::Size2D;
use js::jsapi::{JSContext, JSObject, Heap};
use std::vec::Vec;
use std::default::Default;

#[dom_struct]
#[allow(raw_pointer_derive)]
pub struct ImageData {
    reflector_: Reflector,
    width: u32,
    height: u32,
    data: Heap<*mut JSObject>,
}

impl ImageData {
    pub fn new(global: GlobalRef, width: u32, height: u32, data: Option<&[u8]>)
               -> Fallible<Root<ImageData>> {
        let mut imagedata = box ImageData {
            reflector_: Reflector::new(),
            width: width,
            height: height,
            data: Heap::default(),
        };

        match Uint8ClampedArray::create(global.get_cx(), width * height * 4, data) {
            Some(js_object) => imagedata.data.set(*js_object),
            None => return Err(Error::JSFailed)
        }

        Ok(reflect_dom_object(imagedata, global, ImageDataBinding::Wrap))
    }
}

pub trait ImageDataHelpers {
    fn get_data_array(self, global: &GlobalRef) -> Vec<u8>;
    fn get_size(self) -> Size2D<i32>;
}

impl<'a> ImageDataHelpers for &'a ImageData {
    fn get_data_array(self, _global: &GlobalRef) -> Vec<u8> {
        let mut rooter = TypedArrayRooter::new();
        let mut typed_array = Uint8ClampedArray::new(&mut rooter);
        if typed_array.init(self.data.get()).is_err() {
            return vec!();
        }
        typed_array.compute_length_and_data();
        typed_array.as_slice().to_vec()
    }

    fn get_size(self) -> Size2D<i32> {
        Size2D::new(self.Width() as i32, self.Height() as i32)
    }
}

impl<'a> ImageDataMethods for &'a ImageData {
    // https://html.spec.whatwg.org/multipage/#dom-imagedata-width
    fn Width(self) -> u32 {
        self.width
    }

    // https://html.spec.whatwg.org/multipage/#dom-imagedata-height
    fn Height(self) -> u32 {
        self.height
    }

    // https://html.spec.whatwg.org/multipage/#dom-imagedata-data
    fn Data(self, _: *mut JSContext) -> *mut JSObject {
        self.data.get()
    }
}
