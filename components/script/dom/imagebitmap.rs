/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::vec::Vec;

use dom_struct::dom_struct;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::ImageBitmapBinding::ImageBitmapMethods;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct ImageBitmap {
    reflector_: Reflector,
    width: u32,
    height: u32,
    /// The actual pixel data of the bitmap
    ///
    /// If this is `None`, then the bitmap data has been released by calling
    /// [`close`](https://html.spec.whatwg.org/multipage/#dom-imagebitmap-close)
    bitmap_data: DomRefCell<Option<Vec<u8>>>,
    origin_clean: Cell<bool>,
}

impl ImageBitmap {
    fn new_inherited(width_arg: u32, height_arg: u32) -> ImageBitmap {
        ImageBitmap {
            reflector_: Reflector::new(),
            width: width_arg,
            height: height_arg,
            bitmap_data: DomRefCell::new(Some(vec![])),
            origin_clean: Cell::new(true),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn new(
        global: &GlobalScope,
        width: u32,
        height: u32,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<ImageBitmap>> {
        //assigning to a variable the return object of new_inherited
        let imagebitmap = Box::new(ImageBitmap::new_inherited(width, height));

        Ok(reflect_dom_object(imagebitmap, global, can_gc))
    }

    pub(crate) fn set_bitmap_data(&self, data: Vec<u8>) {
        *self.bitmap_data.borrow_mut() = Some(data);
    }

    pub(crate) fn set_origin_clean(&self, origin_is_clean: bool) {
        self.origin_clean.set(origin_is_clean);
    }

    /// Return the value of the [`[[Detached]]`](https://html.spec.whatwg.org/multipage/#detached)
    /// internal slot
    fn is_detached(&self) -> bool {
        self.bitmap_data.borrow().is_none()
    }
}

impl ImageBitmapMethods<crate::DomTypeHolder> for ImageBitmap {
    /// <https://html.spec.whatwg.org/multipage/#dom-imagebitmap-height>
    fn Height(&self) -> u32 {
        // Step 1. If this's [[Detached]] internal slot's value is true, then return 0.
        if self.is_detached() {
            return 0;
        }

        // Step 2. Return this's height, in CSS pixels.
        self.height
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-imagebitmap-width>
    fn Width(&self) -> u32 {
        // Step 1. If this's [[Detached]] internal slot's value is true, then return 0.
        if self.is_detached() {
            return 0;
        }

        // Step 2. Return this's width, in CSS pixels.
        self.width
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-imagebitmap-close>
    fn Close(&self) {
        // Step 1. Set this's [[Detached]] internal slot value to true.
        // Step 2. Unset this's bitmap data.
        // NOTE: The existence of the bitmap data is the internal slot in our implementation
        self.bitmap_data.borrow_mut().take();
    }
}
