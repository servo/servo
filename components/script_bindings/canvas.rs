/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use euclid::default::Size2D;
use pixels::Snapshot;

use crate::DomTypes;
use crate::codegen::GenericUnionTypes::HTMLCanvasElementOrOffscreenCanvas;
use crate::inheritance::Castable;

pub trait CanvasBindingTrait {
    fn size(&self) -> Size2D<u32>;
}

pub trait CanvasNodeTrait {
    fn is_connected(&self) -> bool;
}

pub trait CanvasContext<D>
where
    D: DomTypes,
    HTMLCanvasElementOrOffscreenCanvas<D>: CanvasBindingTrait,
    D::Node: CanvasNodeTrait,
{
    type ID;

    fn context_id(&self) -> Self::ID;

    fn canvas(&self) -> Option<HTMLCanvasElementOrOffscreenCanvas<D>>;

    fn resize(&self);

    // Resets the backing bitmap (to transparent or opaque black) without the
    // context state reset.
    // Used by OffscreenCanvas.transferToImageBitmap.
    fn reset_bitmap(&self);

    /// Returns none if area of canvas is zero.
    ///
    /// In case of other errors it returns cleared snapshot
    fn get_image_data(&self) -> Option<Snapshot>;

    fn origin_is_clean(&self) -> bool {
        true
    }

    fn size(&self) -> Size2D<u32> {
        self.canvas()
            .map(|canvas| canvas.size())
            .unwrap_or_default()
    }

    fn mark_as_dirty(&self);

    fn onscreen(&self) -> bool {
        let Some(canvas) = self.canvas() else {
            return false;
        };

        match canvas {
            HTMLCanvasElementOrOffscreenCanvas::HTMLCanvasElement(canvas) => {
                canvas.upcast::<D::Node>().is_connected()
            },
            // FIXME(34628): Offscreen canvases should be considered offscreen if a placeholder is set.
            // <https://www.w3.org/TR/webgpu/#abstract-opdef-updating-the-rendering-of-a-webgpu-canvas>
            HTMLCanvasElementOrOffscreenCanvas::OffscreenCanvas(_) => false,
        }
    }
}
