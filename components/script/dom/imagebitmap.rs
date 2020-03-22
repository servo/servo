/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// the following are the statements
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::CanvasGradientBinding;
use crate::dom::bindings::codegen::Bindings::CanvasRenderingContext2DBinding::CanvasImageSource;
use crate::dom::bindings::codegen::Bindings::ImageBitmapBinding;
use crate::dom::bindings::codegen::Bindings::ImageBitmapBinding::ImageBitmapMethods;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::root::DomRoot;
use crate::dom::blob::Blob;
use crate::dom::globalscope::GlobalScope;
use crate::dom::imagedata::ImageData;

use crate::dom::bindings::callback::ExceptionHandling;
// for the reflector to be used
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
//not sure if these two work
use crate::dom::bindings::codegen::Bindings::CanvasPatternBinding; //--don't know why I did this!
use dom_struct::dom_struct;
use js::jsapi::JSObject;
use std::vec::Vec;

//as mentioned in bluetoothuuid.rs
//pub type ImageBitmapSource = CanvasImageSourceorImageDataorBlob;

#[dom_struct]
pub struct ImageBitmap {
    reflector_: Reflector,
    width: u32,
    height: u32,
    ibm_vector: DomRefCell<Vec<u8>>,
}

//#[allow (dead_code)]
impl ImageBitmap {
    #[allow(unsafe_code)]
    pub fn new_inherited(width_arg: u32, height_arg: u32) -> ImageBitmap {
        ImageBitmap {
            reflector_: Reflector::new(),
            width: width_arg,
            height: height_arg,
            ibm_vector: DomRefCell::new(vec![]),
        }
    }

    pub fn new(global: &GlobalScope, width: u32, height: u32) -> Fallible<DomRoot<ImageBitmap>> {
        let imagebitmap = Box::new(ImageBitmap::new_inherited(width, height));

        //reflect_dom_object(box ImageBitMap::new_inherited(width, height), global, ImageBitMapBinding::Wrap);
        Ok(reflect_dom_object(
            imagebitmap,
            global,
            ImageBitmapBinding::Wrap,
        ))
    }
}

// uncomment when working on it
impl ImageBitmapMethods for ImageBitmap {
    fn Height(&self) -> u32 {
        //to do: add a condition for checking detached internal slot
        self.height
    }

    fn Width(&self) -> u32 {
        //to do: add a condition to check detached internal slot
        self.width
    }
}
