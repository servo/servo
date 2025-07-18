/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::{Cell, Ref};
use std::collections::HashMap;
use std::rc::Rc;

use base::id::{ImageBitmapId, ImageBitmapIndex};
use constellation_traits::SerializableImageBitmap;
use dom_struct::dom_struct;
use euclid::default::{Point2D, Rect, Size2D};
use pixels::{CorsStatus, PixelFormat, Snapshot, SnapshotAlphaMode, SnapshotPixelFormat};
use script_bindings::error::{Error, Fallible};
use script_bindings::realms::{AlreadyInRealm, InRealm};

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::ImageBitmapBinding::{
    ImageBitmapMethods, ImageBitmapOptions, ImageBitmapSource, ImageOrientation, PremultiplyAlpha,
    ResizeQuality,
};
use crate::dom::bindings::refcounted::{Trusted, TrustedPromise};
use crate::dom::bindings::reflector::{Reflector, reflect_dom_object};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::serializable::Serializable;
use crate::dom::bindings::structuredclone::StructuredData;
use crate::dom::bindings::transferable::Transferable;
use crate::dom::globalscope::GlobalScope;
use crate::dom::types::Promise;
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
            input.as_raw_bytes(),
            source.size(),
            source_rect_cropped.cast(),
            source.as_raw_bytes_mut(),
        );

        // Step 7. Scale output to the size specified by outputWidth and outputHeight.
        let mut output = if source.size() != output_size {
            let quality = match options.resizeQuality {
                ResizeQuality::Pixelated => pixels::FilterQuality::None,
                ResizeQuality::Low => pixels::FilterQuality::Low,
                ResizeQuality::Medium => pixels::FilterQuality::Medium,
                ResizeQuality::High => pixels::FilterQuality::High,
            };

            let Some(output_data) = pixels::scale_rgba8_image(
                source.size(),
                source.as_raw_bytes(),
                output_size,
                quality,
            ) else {
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
            pixels::flip_y_rgba8_image_inplace(output.size(), output.as_raw_bytes_mut());
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

    /// <https://html.spec.whatwg.org/multipage/#dom-createimagebitmap>
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn create_image_bitmap(
        global_scope: &GlobalScope,
        image: ImageBitmapSource,
        sx: i32,
        sy: i32,
        sw: Option<i32>,
        sh: Option<i32>,
        options: &ImageBitmapOptions,
        can_gc: CanGc,
    ) -> Rc<Promise> {
        let in_realm_proof = AlreadyInRealm::assert::<crate::DomTypeHolder>();
        let p = Promise::new_in_current_realm(InRealm::Already(&in_realm_proof), can_gc);

        // Step 1. If either sw or sh is given and is 0, then return a promise rejected with a RangeError.
        if sw.is_some_and(|w| w == 0) {
            p.reject_error(
                Error::Range("'sw' must be a non-zero value".to_owned()),
                can_gc,
            );
            return p;
        }

        if sh.is_some_and(|h| h == 0) {
            p.reject_error(
                Error::Range("'sh' must be a non-zero value".to_owned()),
                can_gc,
            );
            return p;
        }

        // Step 2. If either options's resizeWidth or options's resizeHeight is present and is 0,
        // then return a promise rejected with an "InvalidStateError" DOMException.
        if options.resizeWidth.is_some_and(|w| w == 0) {
            p.reject_error(Error::InvalidState, can_gc);
            return p;
        }

        if options.resizeHeight.is_some_and(|h| h == 0) {
            p.reject_error(Error::InvalidState, can_gc);
            return p;
        }

        // The promise with image bitmap should be fulfilled on the the bitmap task source.
        let fullfill_promise_on_bitmap_task_source =
            |promise: &Rc<Promise>, image_bitmap: &ImageBitmap| {
                let trusted_promise = TrustedPromise::new(promise.clone());
                let trusted_image_bitmap = Trusted::new(image_bitmap);

                global_scope.task_manager().bitmap_task_source().queue(
                    task!(resolve_promise: move || {
                        let promise = trusted_promise.root();
                        let image_bitmap = trusted_image_bitmap.root();

                        promise.resolve_native(&image_bitmap, CanGc::note());
                    }),
                );
            };

        // The promise with "InvalidStateError" DOMException should be rejected
        // on the the bitmap task source.
        let reject_promise_on_bitmap_task_source = |promise: &Rc<Promise>| {
            let trusted_promise = TrustedPromise::new(promise.clone());

            global_scope
                .task_manager()
                .bitmap_task_source()
                .queue(task!(reject_promise: move || {
                    let promise = trusted_promise.root();

                    promise.reject_error(Error::InvalidState, CanGc::note());
                }));
        };

        // Step 3. Check the usability of the image argument. If this throws an exception or returns bad,
        // then return a promise rejected with an "InvalidStateError" DOMException.
        // Step 6. Switch on image:
        match image {
            ImageBitmapSource::HTMLImageElement(ref image) => {
                // <https://html.spec.whatwg.org/multipage/#check-the-usability-of-the-image-argument>
                if !image.is_usable().is_ok_and(|u| u) {
                    p.reject_error(Error::InvalidState, can_gc);
                    return p;
                }

                // If no ImageBitmap object can be constructed, then the promise
                // is rejected instead.
                let Some(snapshot) = image.get_raster_image_data() else {
                    p.reject_error(Error::InvalidState, can_gc);
                    return p;
                };

                // Step 6.3. Set imageBitmap's bitmap data to a copy of image's media data,
                // cropped to the source rectangle with formatting.
                let Some(bitmap_data) =
                    ImageBitmap::crop_and_transform_bitmap_data(snapshot, sx, sy, sw, sh, options)
                else {
                    p.reject_error(Error::InvalidState, can_gc);
                    return p;
                };

                let image_bitmap = Self::new(global_scope, bitmap_data, can_gc);
                // Step 6.4. If image is not origin-clean, then set the origin-clean flag
                // of imageBitmap's bitmap to false.
                image_bitmap.set_origin_clean(image.same_origin(GlobalScope::entry().origin()));

                // Step 6.5. Queue a global task, using the bitmap task source,
                // to resolve promise with imageBitmap.
                fullfill_promise_on_bitmap_task_source(&p, &image_bitmap);
            },
            ImageBitmapSource::HTMLVideoElement(ref video) => {
                // <https://html.spec.whatwg.org/multipage/#check-the-usability-of-the-image-argument>
                if !video.is_usable() {
                    p.reject_error(Error::InvalidState, can_gc);
                    return p;
                }

                // Step 6.1. If image's networkState attribute is NETWORK_EMPTY, then return
                // a promise rejected with an "InvalidStateError" DOMException.
                if video.is_network_state_empty() {
                    p.reject_error(Error::InvalidState, can_gc);
                    return p;
                }

                // If no ImageBitmap object can be constructed, then the promise is rejected instead.
                let Some(snapshot) = video.get_current_frame_data() else {
                    p.reject_error(Error::InvalidState, can_gc);
                    return p;
                };

                // Step 6.2. Set imageBitmap's bitmap data to a copy of the frame at the current
                // playback position, at the media resource's natural width and natural height
                // (i.e., after any aspect-ratio correction has been applied),
                // cropped to the source rectangle with formatting.
                let Some(bitmap_data) =
                    ImageBitmap::crop_and_transform_bitmap_data(snapshot, sx, sy, sw, sh, options)
                else {
                    p.reject_error(Error::InvalidState, can_gc);
                    return p;
                };

                let image_bitmap = Self::new(global_scope, bitmap_data, can_gc);
                // Step 6.3. If image is not origin-clean, then set the origin-clean flag
                // of imageBitmap's bitmap to false.
                image_bitmap.set_origin_clean(video.origin_is_clean());

                // Step 6.4. Queue a global task, using the bitmap task source,
                // to resolve promise with imageBitmap.
                fullfill_promise_on_bitmap_task_source(&p, &image_bitmap);
            },
            ImageBitmapSource::HTMLCanvasElement(ref canvas) => {
                // <https://html.spec.whatwg.org/multipage/#check-the-usability-of-the-image-argument>
                if canvas.get_size().is_empty() {
                    p.reject_error(Error::InvalidState, can_gc);
                    return p;
                }

                // If no ImageBitmap object can be constructed, then the promise is rejected instead.
                let Some(snapshot) = canvas.get_image_data() else {
                    p.reject_error(Error::InvalidState, can_gc);
                    return p;
                };

                // Step 6.1. Set imageBitmap's bitmap data to a copy of image's bitmap data,
                // cropped to the source rectangle with formatting.
                let Some(bitmap_data) =
                    ImageBitmap::crop_and_transform_bitmap_data(snapshot, sx, sy, sw, sh, options)
                else {
                    p.reject_error(Error::InvalidState, can_gc);
                    return p;
                };

                let image_bitmap = Self::new(global_scope, bitmap_data, can_gc);
                // Step 6.2. Set the origin-clean flag of the imageBitmap's bitmap to the same value
                // as the origin-clean flag of image's bitmap.
                image_bitmap.set_origin_clean(canvas.origin_is_clean());

                // Step 6.3. Queue a global task, using the bitmap task source,
                // to resolve promise with imageBitmap.
                fullfill_promise_on_bitmap_task_source(&p, &image_bitmap);
            },
            ImageBitmapSource::ImageBitmap(ref bitmap) => {
                // <https://html.spec.whatwg.org/multipage/#check-the-usability-of-the-image-argument>
                if bitmap.is_detached() {
                    p.reject_error(Error::InvalidState, can_gc);
                    return p;
                }

                // If no ImageBitmap object can be constructed, then the promise is rejected instead.
                let Some(snapshot) = bitmap.bitmap_data().clone() else {
                    p.reject_error(Error::InvalidState, can_gc);
                    return p;
                };

                // Step 6.1. Set imageBitmap's bitmap data to a copy of image's bitmap data,
                // cropped to the source rectangle with formatting.
                let Some(bitmap_data) =
                    ImageBitmap::crop_and_transform_bitmap_data(snapshot, sx, sy, sw, sh, options)
                else {
                    p.reject_error(Error::InvalidState, can_gc);
                    return p;
                };

                let image_bitmap = Self::new(global_scope, bitmap_data, can_gc);
                // Step 6.2. Set the origin-clean flag of imageBitmap's bitmap to the same value
                // as the origin-clean flag of image's bitmap.
                image_bitmap.set_origin_clean(bitmap.origin_is_clean());

                // Step 6.3. Queue a global task, using the bitmap task source,
                // to resolve promise with imageBitmap.
                fullfill_promise_on_bitmap_task_source(&p, &image_bitmap);
            },
            ImageBitmapSource::OffscreenCanvas(ref canvas) => {
                // <https://html.spec.whatwg.org/multipage/#check-the-usability-of-the-image-argument>
                if canvas.get_size().is_empty() {
                    p.reject_error(Error::InvalidState, can_gc);
                    return p;
                }

                // If no ImageBitmap object can be constructed, then the promise is rejected instead.
                let Some(snapshot) = canvas.get_image_data() else {
                    p.reject_error(Error::InvalidState, can_gc);
                    return p;
                };

                // Step 6.1. Set imageBitmap's bitmap data to a copy of image's bitmap data,
                // cropped to the source rectangle with formatting.
                let Some(bitmap_data) =
                    ImageBitmap::crop_and_transform_bitmap_data(snapshot, sx, sy, sw, sh, options)
                else {
                    p.reject_error(Error::InvalidState, can_gc);
                    return p;
                };

                let image_bitmap = Self::new(global_scope, bitmap_data, can_gc);
                // Step 6.2. Set the origin-clean flag of the imageBitmap's bitmap to the same value
                // as the origin-clean flag of image's bitmap.
                image_bitmap.set_origin_clean(canvas.origin_is_clean());

                // Step 6.3. Queue a global task, using the bitmap task source,
                // to resolve promise with imageBitmap.
                fullfill_promise_on_bitmap_task_source(&p, &image_bitmap);
            },
            ImageBitmapSource::Blob(ref blob) => {
                // Step 6.1. Let imageData be the result of reading image's data.
                // If an error occurs during reading of the object, then queue
                // a global task, using the bitmap task source, to reject promise
                // with an "InvalidStateError" DOMException and abort these steps.
                let Ok(bytes) = blob.get_bytes() else {
                    reject_promise_on_bitmap_task_source(&p);
                    return p;
                };

                // Step 6.2. Apply the image sniffing rules to determine the file
                // format of imageData, with MIME type of image (as given by
                // image's type attribute) giving the official type.
                // Step 6.3. If imageData is not in a supported image file format
                // (e.g., it's not an image at all), or if imageData is corrupted
                // in some fatal way such that the image dimensions cannot be obtained
                // (e.g., a vector graphic with no natural size), then queue
                // a global task, using the bitmap task source, to reject promise
                // with an "InvalidStateError" DOMException and abort these steps.
                let Some(img) = pixels::load_from_memory(&bytes, CorsStatus::Safe) else {
                    reject_promise_on_bitmap_task_source(&p);
                    return p;
                };

                let size = Size2D::new(img.metadata.width, img.metadata.height);
                let format = match img.format {
                    PixelFormat::BGRA8 => SnapshotPixelFormat::BGRA,
                    PixelFormat::RGBA8 => SnapshotPixelFormat::RGBA,
                    pixel_format => {
                        unimplemented!("unsupported pixel format ({:?})", pixel_format)
                    },
                };
                let alpha_mode = SnapshotAlphaMode::Transparent {
                    premultiplied: false,
                };

                let snapshot = Snapshot::from_vec(
                    size.cast(),
                    format,
                    alpha_mode,
                    img.first_frame().bytes.to_vec(),
                );

                // Step 6.4. Set imageBitmap's bitmap data to imageData, cropped
                // to the source rectangle with formatting.
                let Some(bitmap_data) =
                    ImageBitmap::crop_and_transform_bitmap_data(snapshot, sx, sy, sw, sh, options)
                else {
                    reject_promise_on_bitmap_task_source(&p);
                    return p;
                };

                let image_bitmap = Self::new(global_scope, bitmap_data, can_gc);

                // Step 6.5. Queue a global task, using the bitmap task source,
                // to resolve promise with imageBitmap.
                fullfill_promise_on_bitmap_task_source(&p, &image_bitmap);
            },
            ImageBitmapSource::ImageData(ref image_data) => {
                // Step 6.1. Let buffer be image's data attribute value's [[ViewedArrayBuffer]] internal slot.
                // Step 6.2. If IsDetachedBuffer(buffer) is true, then return a promise rejected
                // with an "InvalidStateError" DOMException.
                if image_data.is_detached() {
                    p.reject_error(Error::InvalidState, can_gc);
                    return p;
                }

                let alpha_mode = SnapshotAlphaMode::Transparent {
                    premultiplied: false,
                };

                let snapshot = Snapshot::from_vec(
                    image_data.get_size().cast(),
                    SnapshotPixelFormat::RGBA,
                    alpha_mode,
                    image_data.to_vec(),
                );

                // Step 6.3. Set imageBitmap's bitmap data to image's image data,
                // cropped to the source rectangle with formatting.
                let Some(bitmap_data) =
                    ImageBitmap::crop_and_transform_bitmap_data(snapshot, sx, sy, sw, sh, options)
                else {
                    p.reject_error(Error::InvalidState, can_gc);
                    return p;
                };

                let image_bitmap = Self::new(global_scope, bitmap_data, can_gc);

                // Step 6.4. Queue a global task, using the bitmap task source,
                // to resolve promise with imageBitmap.
                fullfill_promise_on_bitmap_task_source(&p, &image_bitmap);
            },
            ImageBitmapSource::CSSStyleValue(_) => {
                // TODO: CSSStyleValue is not part of ImageBitmapSource
                // <https://html.spec.whatwg.org/multipage/#imagebitmapsource>
                p.reject_error(Error::NotSupported, can_gc);
            },
        }

        // Step 7. Return promise.
        p
    }
}

impl Serializable for ImageBitmap {
    type Index = ImageBitmapIndex;
    type Data = SerializableImageBitmap;

    /// <https://html.spec.whatwg.org/multipage/#the-imagebitmap-interface:serialization-steps>
    fn serialize(&self) -> Result<(ImageBitmapId, Self::Data), ()> {
        // <https://html.spec.whatwg.org/multipage/#structuredserializeinternal>
        // Step 19.1. If value has a [[Detached]] internal slot whose value is
        // true, then throw a "DataCloneError" DOMException.
        if self.is_detached() {
            return Err(());
        }

        // Step 1. If value's origin-clean flag is not set, then throw a
        // "DataCloneError" DOMException.
        if !self.origin_is_clean() {
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

    /// <https://html.spec.whatwg.org/multipage/#the-imagebitmap-interface:transfer-steps>
    fn transfer(&self) -> Fallible<(ImageBitmapId, SerializableImageBitmap)> {
        // <https://html.spec.whatwg.org/multipage/#structuredserializewithtransfer>
        // Step 5.2. If transferable has a [[Detached]] internal slot and
        // transferable.[[Detached]] is true, then throw a "DataCloneError"
        // DOMException.
        if self.is_detached() {
            return Err(Error::DataClone(None));
        }

        // Step 1. If value's origin-clean flag is not set, then throw a
        // "DataCloneError" DOMException.
        if !self.origin_is_clean() {
            return Err(Error::DataClone(None));
        }

        // Step 2. Set dataHolder.[[BitmapData]] to value's bitmap data.
        // Step 3. Unset value's bitmap data.
        let transferred = SerializableImageBitmap {
            bitmap_data: self.bitmap_data.borrow_mut().take().unwrap(),
        };

        Ok((ImageBitmapId::new(), transferred))
    }

    /// <https://html.spec.whatwg.org/multipage/#the-imagebitmap-interface:transfer-receiving-steps>
    fn transfer_receive(
        owner: &GlobalScope,
        _: ImageBitmapId,
        transferred: SerializableImageBitmap,
    ) -> Result<DomRoot<Self>, ()> {
        // Step 1. Set value's bitmap data to serialized.[[BitmapData]].
        Ok(ImageBitmap::new(
            owner,
            transferred.bitmap_data,
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
