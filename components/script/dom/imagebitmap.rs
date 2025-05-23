/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::collections::HashMap;
use std::vec::Vec;

use base::id::{ImageBitmapId, ImageBitmapIndex};
use constellation_traits::SerializableImageBitmap;
use dom_struct::dom_struct;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::ImageBitmapBinding::ImageBitmapMethods;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::reflector::{Reflector, reflect_dom_object};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::serializable::Serializable;
use crate::dom::bindings::structuredclone::StructuredData;
use crate::dom::bindings::transferable::Transferable;
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
    fn new_inherited(width: u32, height: u32) -> ImageBitmap {
        ImageBitmap {
            reflector_: Reflector::new(),
            width,
            height,
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

    pub(crate) fn origin_is_clean(&self) -> bool {
        self.origin_clean.get()
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

impl Serializable for ImageBitmap {
    type Index = ImageBitmapIndex;
    type Data = SerializableImageBitmap;

    /// <https://html.spec.whatwg.org/multipage/#the-imagebitmap-interface:serialization-steps>
    fn serialize(&self) -> Result<(ImageBitmapId, Self::Data), ()> {
        // Step 1. If value's origin-clean flag is not set, then throw a "DataCloneError" DOMException.
        if !self.origin_is_clean() {
            return Err(());
        }

        // If value has a [[Detached]] internal slot whose value is true,
        // then throw a "DataCloneError" DOMException.
        if self.is_detached() {
            return Err(());
        }

        // Step 2. Set serialized.[[BitmapData]] to a copy of value's bitmap data.
        let serialized = SerializableImageBitmap {
            width: self.width,
            height: self.height,
            bitmap_data: self.bitmap_data.borrow().clone().unwrap(),
        };

        Ok((ImageBitmapId::new(), serialized))
    }

    /// <https://html.spec.whatwg.org/multipage/#the-imagebitmap-interface:deserialization-steps>
    fn deserialize(
        owner: &GlobalScope,
        serialized: Self::Data,
        can_gc: CanGc,
    ) -> Result<DomRoot<Self>, ()> {
        let image_bitmap =
            ImageBitmap::new(owner, serialized.width, serialized.height, can_gc).unwrap();

        // Step 1. Set value's bitmap data to serialized.[[BitmapData]].
        image_bitmap.set_bitmap_data(serialized.bitmap_data);

        Ok(image_bitmap)
    }

    fn serialized_storage<'a>(
        reader: StructuredData<'a, '_>,
    ) -> &'a mut Option<HashMap<ImageBitmapId, Self::Data>> {
        match reader {
            StructuredData::Reader(r) => &mut r.image_bitmaps,
            StructuredData::Writer(w) => &mut w.image_bitmaps,
        }
    }
}

impl Transferable for ImageBitmap {
    type Index = ImageBitmapIndex;
    type Data = SerializableImageBitmap;

    fn can_transfer(&self) -> bool {
        if !self.origin_is_clean() || self.is_detached() {
            return false;
        }
        true
    }

    /// <https://html.spec.whatwg.org/multipage/#the-imagebitmap-interface:transfer-steps>
    fn transfer(&self) -> Result<(ImageBitmapId, SerializableImageBitmap), ()> {
        // Step 1. If value's origin-clean flag is not set, then throw a "DataCloneError" DOMException.
        if !self.origin_is_clean() {
            return Err(());
        }

        // If value has a [[Detached]] internal slot whose value is true,
        // then throw a "DataCloneError" DOMException.
        if self.is_detached() {
            return Err(());
        }

        // Step 2. Set dataHolder.[[BitmapData]] to value's bitmap data.
        // Step 3. Unset value's bitmap data.
        let serialized = SerializableImageBitmap {
            width: self.width,
            height: self.height,
            bitmap_data: self.bitmap_data.borrow_mut().take().unwrap(),
        };

        Ok((ImageBitmapId::new(), serialized))
    }

    /// <https://html.spec.whatwg.org/multipage/#the-imagebitmap-interface:transfer-receiving-steps>
    fn transfer_receive(
        owner: &GlobalScope,
        _: ImageBitmapId,
        serialized: SerializableImageBitmap,
    ) -> Result<DomRoot<Self>, ()> {
        let image_bitmap =
            ImageBitmap::new(owner, serialized.width, serialized.height, CanGc::note()).unwrap();

        // Step 1. Set value's bitmap data to serialized.[[BitmapData]].
        image_bitmap.set_bitmap_data(serialized.bitmap_data);

        Ok(image_bitmap)
    }

    fn serialized_storage<'a>(
        data: StructuredData<'a, '_>,
    ) -> &'a mut Option<HashMap<ImageBitmapId, Self::Data>> {
        match data {
            StructuredData::Reader(r) => &mut r.transferred_image_bitmaps,
            StructuredData::Writer(w) => &mut w.transferred_image_bitmaps,
        }
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
