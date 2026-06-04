/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use euclid::default::Size2D;

use crate::DomTypes;
use crate::codegen::GenericUnionTypes::HTMLCanvasElementOrOffscreenCanvas;

pub trait CanvasSizeTrait {
    fn get_size(&self) -> Size2D<u32>;
}

impl<D: DomTypes> HTMLCanvasElementOrOffscreenCanvas<D>
where
    D::HTMLCanvasElement: CanvasSizeTrait,
    D::OffscreenCanvas: CanvasSizeTrait,
{
    pub fn size(&self) -> Size2D<u32> {
        match self {
            HTMLCanvasElementOrOffscreenCanvas::HTMLCanvasElement(root) => root.get_size(),
            HTMLCanvasElementOrOffscreenCanvas::OffscreenCanvas(root) => root.get_size(),
        }
    }
}
