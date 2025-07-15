/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use dom_struct::dom_struct;
use euclid::default::Size2D;
use pixels::Snapshot;
use webrender_api::ImageKey;

use crate::canvas_context::{CanvasContext, CanvasHelpers, LayoutCanvasRenderingContextHelpers};
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::ImageBitmapBinding::ImageBitmapMethods;
use crate::dom::bindings::codegen::Bindings::ImageBitmapRenderingContextBinding::ImageBitmapRenderingContextMethods;
use crate::dom::bindings::codegen::UnionTypes::HTMLCanvasElementOrOffscreenCanvas;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::reflector::{Reflector, reflect_dom_object};
use crate::dom::bindings::root::{DomRoot, LayoutDom};
use crate::dom::globalscope::GlobalScope;
use crate::dom::imagebitmap::ImageBitmap;
use crate::script_runtime::CanGc;

/// <https://html.spec.whatwg.org/multipage/#imagebitmaprenderingcontext>
#[dom_struct]
pub(crate) struct ImageBitmapRenderingContext {
    reflector_: Reflector,
    /// <https://html.spec.whatwg.org/multipage/#dom-imagebitmaprenderingcontext-canvas>
    canvas: HTMLCanvasElementOrOffscreenCanvas,
    /// Represents both the [output bitmap] and the [bitmap mode] of the context.
    /// <https://html.spec.whatwg.org/multipage/#concept-imagebitmaprenderingcontext-output-bitmap>
    /// <https://html.spec.whatwg.org/multipage/#concept-imagebitmaprenderingcontext-bitmap-mode>
    #[no_trace]
    bitmap: DomRefCell<Option<Snapshot>>,
    origin_clean: Cell<bool>,
}

impl ImageBitmapRenderingContext {
    /// <https://html.spec.whatwg.org/multipage/#imagebitmaprenderingcontext-creation-algorithm>
    fn new_inherited(canvas: HTMLCanvasElementOrOffscreenCanvas) -> ImageBitmapRenderingContext {
        ImageBitmapRenderingContext {
            reflector_: Reflector::new(),
            canvas,
            bitmap: DomRefCell::new(None),
            origin_clean: Cell::new(true),
        }
    }

    pub(crate) fn new(
        global: &GlobalScope,
        canvas: HTMLCanvasElementOrOffscreenCanvas,
        can_gc: CanGc,
    ) -> DomRoot<ImageBitmapRenderingContext> {
        reflect_dom_object(
            Box::new(ImageBitmapRenderingContext::new_inherited(canvas)),
            global,
            can_gc,
        )
    }

    /// <https://html.spec.whatwg.org/multipage/#set-an-imagebitmaprenderingcontext's-output-bitmap>
    fn set_bitmap(&self, image_bitmap: Option<&ImageBitmap>) {
        match image_bitmap {
            Some(image_bitmap) => {
                // Step 2.1. Set context's bitmap mode to valid.
                // Step 2.2. Set context's output bitmap to refer to the same
                // underlying bitmap data as bitmap, without making a copy.
                *self.bitmap.borrow_mut() = image_bitmap.bitmap_data().clone();

                // The origin-clean flag of bitmap is included in the bitmap
                // data to be referenced by context's output bitmap.
                self.origin_clean.set(image_bitmap.origin_is_clean());
            },
            None => {
                // Step 1.1. Set context's bitmap mode to blank.
                // Step 1.2. Let canvas be the canvas element to which context is bound.
                // Step 1.3. Set context's output bitmap to be transparent black
                // with a natural width equal to the numeric value of canvas's
                // width attribute and a natural height equal to the numeric
                // value of canvas's height attribute, those values being
                // interpreted in CSS pixels.
                *self.bitmap.borrow_mut() = None;

                // Step 1.4. Set the output bitmap's origin-clean flag to true.
                self.origin_clean.set(true);
            },
        }
    }
}

impl LayoutCanvasRenderingContextHelpers for LayoutDom<'_, ImageBitmapRenderingContext> {
    fn canvas_data_source(self) -> Option<ImageKey> {
        None
    }
}

impl CanvasContext for ImageBitmapRenderingContext {
    type ID = ();

    fn context_id(&self) -> Self::ID {}

    fn canvas(&self) -> Option<HTMLCanvasElementOrOffscreenCanvas> {
        Some(self.canvas.clone())
    }

    /// <https://html.spec.whatwg.org/multipage/#the-canvas-element:concept-canvas-bitmaprenderer>
    fn resize(&self) {
        // The absence of the bitmap is the context's blank bitmap mode so the
        // steps to set output bitmap could be omitted.
    }

    fn reset_bitmap(&self) {
        // The newly created bitmap should be of the same dimensions as the
        // previous bitmap if the context's bitmap mode is valid.
        if self.bitmap.borrow().is_none() {
            return;
        }

        let size = self.bitmap.borrow().as_ref().unwrap().size();
        *self.bitmap.borrow_mut() = Some(Snapshot::cleared(size));
    }

    fn get_image_data(&self) -> Option<Snapshot> {
        match self.bitmap.borrow().as_ref() {
            Some(bitmap) => Some(bitmap.clone()),
            None => {
                let size = self.canvas.size();
                if size.is_empty() ||
                    pixels::compute_rgba8_byte_length_if_within_limit(
                        size.width as usize,
                        size.height as usize,
                    )
                    .is_none()
                {
                    None
                } else {
                    Some(Snapshot::cleared(size))
                }
            },
        }
    }

    fn origin_is_clean(&self) -> bool {
        self.origin_clean.get()
    }

    fn size(&self) -> Size2D<u32> {
        self.bitmap
            .borrow()
            .as_ref()
            .map_or_else(|| self.canvas.size(), |bitmap| bitmap.size())
    }
}

impl ImageBitmapRenderingContextMethods<crate::DomTypeHolder> for ImageBitmapRenderingContext {
    /// <https://html.spec.whatwg.org/multipage/#dom-imagebitmaprenderingcontext-canvas>
    fn Canvas(&self) -> HTMLCanvasElementOrOffscreenCanvas {
        self.canvas.clone()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-imagebitmaprenderingcontext-transferfromimagebitmap>
    fn TransferFromImageBitmap(&self, image_bitmap: Option<&ImageBitmap>) -> Fallible<()> {
        let Some(image_bitmap) = image_bitmap else {
            // Step 2. If bitmap is null, then run the steps to set an
            // ImageBitmapRenderingContext's output bitmap, with
            // bitmapContext as the context argument and no bitmap argument,
            // then return.
            self.set_bitmap(None);

            return Ok(());
        };

        // Step 3. If the value of bitmap's [[Detached]] internal slot
        // is set to true, then throw an "InvalidStateError"
        // DOMException.
        if image_bitmap.is_detached() {
            return Err(Error::InvalidState);
        }

        // Step 4. Run the steps to set an ImageBitmapRenderingContext's
        // output bitmap, with the context argument equal to
        // bitmapContext, and the bitmap argument referring to bitmap's
        // underlying bitmap data.
        self.set_bitmap(Some(image_bitmap));

        // Step 5. Set the value of bitmap's [[Detached]] internal slot
        // to true.
        // Step 6. Unset bitmap's bitmap data.
        image_bitmap.Close();

        Ok(())
    }
}
