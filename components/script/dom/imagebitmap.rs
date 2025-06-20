/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::{Cell, Ref};
use std::collections::HashMap;

use base::id::{ImageBitmapId, ImageBitmapIndex};
use constellation_traits::SerializableImageBitmap;
use dom_struct::dom_struct;
use euclid::default::{Point2D, Rect, Size2D};
use pixels::{Snapshot, SnapshotAlphaMode, SnapshotPixelFormat};

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::ImageBitmapBinding::{
    ImageBitmapMethods, ImageBitmapOptions, ImageOrientation, PremultiplyAlpha, ResizeQuality,
};
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
    /// The actual pixel data of the bitmap
    ///
    /// If this is `None`, then the bitmap data has been released by calling
    /// [`close`](https://html.spec.whatwg.org/multipage/#dom-imagebitmap-close)
    #[no_trace]
    bitmap_data: DomRefCell<Option<Snapshot>>,
    origin_clean: Cell<bool>,
}

impl ImageBitmap {
    fn new_inherited(bitmap_data: Snapshot) -> ImageBitmap {
        ImageBitmap {
            reflector_: Reflector::new(),
            bitmap_data: DomRefCell::new(Some(bitmap_data)),
            origin_clean: Cell::new(true),
        }
    }

    pub(crate) fn new(
        global: &GlobalScope,
        bitmap_data: Snapshot,
        can_gc: CanGc,
    ) -> DomRoot<ImageBitmap> {
        reflect_dom_object(
            Box::new(ImageBitmap::new_inherited(bitmap_data)),
            global,
            can_gc,
        )
    }

    #[allow(dead_code)]
    pub(crate) fn bitmap_data(&self) -> Ref<Option<Snapshot>> {
        self.bitmap_data.borrow()
    }

    pub(crate) fn origin_is_clean(&self) -> bool {
        self.origin_clean.get()
    }

    pub(crate) fn set_origin_clean(&self, origin_is_clean: bool) {
        self.origin_clean.set(origin_is_clean);
    }

    /// Return the value of the [`[[Detached]]`](https://html.spec.whatwg.org/multipage/#detached)
    /// internal slot
    pub(crate) fn is_detached(&self) -> bool {
        self.bitmap_data.borrow().is_none()
    }

    /// <https://html.spec.whatwg.org/multipage/#cropped-to-the-source-rectangle-with-formatting>
    pub(crate) fn crop_and_transform_bitmap_data(
        input: Snapshot,
        mut sx: i32,
        mut sy: i32,
        sw: Option<i32>,
        sh: Option<i32>,
        options: &ImageBitmapOptions,
    ) -> Option<Snapshot> {
        let input_size = input.size().to_i32();

        // Step 2. If sx, sy, sw and sh are specified, let sourceRectangle be a rectangle whose corners
        // are the four points (sx, sy), (sx+sw, sy), (sx+sw, sy+sh), (sx, sy+sh). Otherwise,
        // let sourceRectangle be a rectangle whose corners are the four points (0, 0), (width of input, 0),
        // (width of input, height of input), (0, height of input). If either sw or sh are negative,
        // then the top-left corner of this rectangle will be to the left or above the (sx, sy) point.
        let sw = sw.map_or(input_size.width, |width| {
            if width < 0 {
                sx = sx.saturating_add(width);
                width.saturating_abs()
            } else {
                width
            }
        });

        let sh = sh.map_or(input_size.height, |height| {
            if height < 0 {
                sy = sy.saturating_add(height);
                height.saturating_abs()
            } else {
                height
            }
        });

        let source_rect = Rect::new(Point2D::new(sx, sy), Size2D::new(sw, sh));

        // Whether the byte length of the source bitmap exceeds the supported range.
        // In the case the source is too large, we should fail, and that is not defined.
        // <https://github.com/whatwg/html/issues/3323>
        let Some(source_byte_length) = pixels::compute_rgba8_byte_length_if_within_limit(
            source_rect.size.width as usize,
            source_rect.size.height as usize,
        ) else {
            log::warn!(
                "Failed to allocate bitmap of size {:?}, too large",
                source_rect.size
            );
            return None;
        };

        // Step 3. Let outputWidth be determined as follows:
        // Step 4. Let outputHeight be determined as follows:
        let output_size = match (options.resizeWidth, options.resizeHeight) {
            (Some(width), Some(height)) => Size2D::new(width, height),
            (Some(width), None) => {
                let height =
                    source_rect.size.height as f64 * width as f64 / source_rect.size.width as f64;
                Size2D::new(width, height.round() as u32)
            },
            (None, Some(height)) => {
                let width =
                    source_rect.size.width as f64 * height as f64 / source_rect.size.height as f64;
                Size2D::new(width.round() as u32, height)
            },
            (None, None) => source_rect.size.to_u32(),
        };

        // Whether the byte length of the output bitmap exceeds the supported range.
        // In the case the output is too large, we should fail, and that is not defined.
        // <https://github.com/whatwg/html/issues/3323>
        let Some(output_byte_length) = pixels::compute_rgba8_byte_length_if_within_limit(
            output_size.width as usize,
            output_size.height as usize,
        ) else {
            log::warn!(
                "Failed to allocate bitmap of size {:?}, too large",
                output_size
            );
            return None;
        };

        // TODO: Take into account the image orientation (such as EXIF metadata).

        // Step 5. Place input on an infinite transparent black grid plane, positioned so that
        // its top left corner is at the origin of the plane, with the x-coordinate increasing to the right,
        // and the y-coordinate increasing down, and with each pixel in the input image data occupying a cell
        // on the plane's grid.
        let input_rect = Rect::new(Point2D::zero(), input_size);

        let input_rect_cropped = source_rect
            .intersection(&input_rect)
            .unwrap_or(Rect::zero());

        // Early out for empty tranformations.
        if input_rect_cropped.is_empty() {
            return Some(Snapshot::cleared(output_size));
        }

        // Step 6. Let output be the rectangle on the plane denoted by sourceRectangle.
        let mut source: Snapshot = Snapshot::from_vec(
            source_rect.size.cast(),
            input.format(),
            input.alpha_mode(),
            vec![0; source_byte_length],
        );

        let source_rect_cropped = Rect::new(
            Point2D::new(
                input_rect_cropped.origin.x - source_rect.origin.x,
                input_rect_cropped.origin.y - source_rect.origin.y,
            ),
            input_rect_cropped.size,
        );

        pixels::copy_rgba8_image(
            input.size(),
            input_rect_cropped.cast(),
            input.data(),
            source.size(),
            source_rect_cropped.cast(),
            source.data_mut(),
        );

        // Step 7. Scale output to the size specified by outputWidth and outputHeight.
        let mut output = if source.size() != output_size {
            let quality = match options.resizeQuality {
                ResizeQuality::Pixelated => pixels::FilterQuality::None,
                ResizeQuality::Low => pixels::FilterQuality::Low,
                ResizeQuality::Medium => pixels::FilterQuality::Medium,
                ResizeQuality::High => pixels::FilterQuality::High,
            };

            let Some(output_data) =
                pixels::scale_rgba8_image(source.size(), source.data(), output_size, quality)
            else {
                log::warn!(
                    "Failed to scale the bitmap of size {:?} to required size {:?}",
                    source.size(),
                    output_size
                );
                return None;
            };

            debug_assert_eq!(output_data.len(), output_byte_length);

            Snapshot::from_vec(
                output_size,
                source.format(),
                source.alpha_mode(),
                output_data,
            )
        } else {
            source
        };

        // Step 8. If the value of the imageOrientation member of options is "flipY",
        // output must be flipped vertically, disregarding any image orientation metadata
        // of the source (such as EXIF metadata), if any.
        if options.imageOrientation == ImageOrientation::FlipY {
            pixels::flip_y_rgba8_image_inplace(output.size(), output.data_mut());
        }

        // TODO: Step 9. If image is an img element or a Blob object, let val be the value
        // of the colorSpaceConversion member of options, and then run these substeps:

        // Step 10. Let val be the value of premultiplyAlpha member of options,
        // and then run these substeps:
        // TODO: Preserve the original input pixel format and perform conversion on demand.
        match options.premultiplyAlpha {
            PremultiplyAlpha::Default | PremultiplyAlpha::Premultiply => {
                output.transform(
                    SnapshotAlphaMode::Transparent {
                        premultiplied: true,
                    },
                    SnapshotPixelFormat::BGRA,
                );
            },
            PremultiplyAlpha::None => {
                output.transform(
                    SnapshotAlphaMode::Transparent {
                        premultiplied: false,
                    },
                    SnapshotPixelFormat::BGRA,
                );
            },
        }

        // Step 11. Return output.
        Some(output)
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
        // Step 1. Set value's bitmap data to serialized.[[BitmapData]].
        Ok(ImageBitmap::new(owner, serialized.bitmap_data, can_gc))
    }

    fn serialized_storage<'a>(
        data: StructuredData<'a, '_>,
    ) -> &'a mut Option<HashMap<ImageBitmapId, Self::Data>> {
        match data {
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
        // Step 1. Set value's bitmap data to serialized.[[BitmapData]].
        Ok(ImageBitmap::new(
            owner,
            serialized.bitmap_data,
            CanGc::note(),
        ))
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
        self.bitmap_data
            .borrow()
            .as_ref()
            .unwrap()
            .size()
            .cast()
            .height
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-imagebitmap-width>
    fn Width(&self) -> u32 {
        // Step 1. If this's [[Detached]] internal slot's value is true, then return 0.
        if self.is_detached() {
            return 0;
        }

        // Step 2. Return this's width, in CSS pixels.
        self.bitmap_data
            .borrow()
            .as_ref()
            .unwrap()
            .size()
            .cast()
            .width
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-imagebitmap-close>
    fn Close(&self) {
        // Step 1. Set this's [[Detached]] internal slot value to true.
        // Step 2. Unset this's bitmap data.
        // NOTE: The existence of the bitmap data is the internal slot in our implementation
        self.bitmap_data.borrow_mut().take();
    }
}
