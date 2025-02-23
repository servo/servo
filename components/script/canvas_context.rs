/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Common interfaces for Canvas Contexts

use canvas_traits::canvas::CanvasId;
use euclid::default::Size2D;
use ipc_channel::ipc::IpcSharedMemory;
use script_layout_interface::{HTMLCanvasData, HTMLCanvasDataSource};

use crate::dom::bindings::codegen::UnionTypes::HTMLCanvasElementOrOffscreenCanvas;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::htmlcanvaselement::HTMLCanvasElement;
use crate::dom::node::{Node, NodeDamage};

pub(crate) trait LayoutCanvasRenderingContextHelpers {
    fn canvas_data_source(self) -> HTMLCanvasDataSource;
}

pub(crate) trait LayoutHTMLCanvasElementHelpers {
    fn data(self) -> HTMLCanvasData;
    fn get_canvas_id_for_layout(self) -> CanvasId;
}

pub(crate) trait CanvasContext {
    type ID;

    fn context_id(&self) -> Self::ID;

    fn canvas(&self) -> HTMLCanvasElementOrOffscreenCanvas;

    fn resize(&self);

    fn get_image_data_as_shared_memory(&self) -> Option<IpcSharedMemory>;

    fn get_image_data(&self) -> Option<Vec<u8>> {
        self.get_image_data_as_shared_memory().map(|sm| sm.to_vec())
    }

    fn origin_is_clean(&self) -> bool {
        true
    }

    fn size(&self) -> Size2D<u64> {
        self.canvas().size()
    }

    fn mark_as_dirty(&self) {
        if let HTMLCanvasElementOrOffscreenCanvas::HTMLCanvasElement(canvas) = &self.canvas() {
            canvas.upcast::<Node>().dirty(NodeDamage::OtherNodeDamage);
        }
    }

    fn update_rendering(&self) {}

    fn onscreen(&self) -> bool {
        match self.canvas() {
            HTMLCanvasElementOrOffscreenCanvas::HTMLCanvasElement(ref canvas) => {
                canvas.upcast::<Node>().is_connected()
            },
            // FIXME(34628): Offscreen canvases should be considered offscreen if a placeholder is set.
            // <https://www.w3.org/TR/webgpu/#abstract-opdef-updating-the-rendering-of-a-webgpu-canvas>
            HTMLCanvasElementOrOffscreenCanvas::OffscreenCanvas(_) => false,
        }
    }
}

impl HTMLCanvasElementOrOffscreenCanvas {
    pub(crate) fn size(&self) -> Size2D<u64> {
        match self {
            HTMLCanvasElementOrOffscreenCanvas::HTMLCanvasElement(canvas) => {
                canvas.get_size().cast()
            },
            HTMLCanvasElementOrOffscreenCanvas::OffscreenCanvas(canvas) => canvas.get_size(),
        }
    }

    pub(crate) fn canvas(&self) -> Option<&HTMLCanvasElement> {
        match self {
            HTMLCanvasElementOrOffscreenCanvas::HTMLCanvasElement(canvas) => Some(canvas),
            HTMLCanvasElementOrOffscreenCanvas::OffscreenCanvas(canvas) => canvas.placeholder(),
        }
    }
}
