/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Common interfaces for Canvas Contexts

use euclid::default::Size2D;
use script_bindings::root::Dom;
use script_layout_interface::{HTMLCanvasData, HTMLCanvasDataSource};
use snapshot::Snapshot;

use crate::dom::bindings::codegen::UnionTypes::HTMLCanvasElementOrOffscreenCanvas;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::htmlcanvaselement::HTMLCanvasElement;
use crate::dom::node::{Node, NodeDamage};
use crate::dom::types::{
    CanvasRenderingContext2D, GPUCanvasContext, OffscreenCanvas, OffscreenCanvasRenderingContext2D,
    WebGL2RenderingContext, WebGLRenderingContext,
};

pub(crate) trait LayoutCanvasRenderingContextHelpers {
    fn canvas_data_source(self) -> HTMLCanvasDataSource;
}

pub(crate) trait LayoutHTMLCanvasElementHelpers {
    fn data(self) -> HTMLCanvasData;
}

pub(crate) trait CanvasContext {
    type ID;

    fn context_id(&self) -> Self::ID;

    fn canvas(&self) -> HTMLCanvasElementOrOffscreenCanvas;

    fn resize(&self);

    /// Returns none if area of canvas is zero.
    ///
    /// In case of other errors it returns cleared snapshot
    fn get_image_data(&self) -> Option<Snapshot>;

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

pub(crate) trait CanvasHelpers {
    fn size(&self) -> Size2D<u64>;
    fn canvas(&self) -> Option<&HTMLCanvasElement>;
}

impl CanvasHelpers for HTMLCanvasElementOrOffscreenCanvas {
    fn size(&self) -> Size2D<u64> {
        match self {
            HTMLCanvasElementOrOffscreenCanvas::HTMLCanvasElement(canvas) => {
                canvas.get_size().cast()
            },
            HTMLCanvasElementOrOffscreenCanvas::OffscreenCanvas(canvas) => canvas.get_size(),
        }
    }

    fn canvas(&self) -> Option<&HTMLCanvasElement> {
        match self {
            HTMLCanvasElementOrOffscreenCanvas::HTMLCanvasElement(canvas) => Some(canvas),
            HTMLCanvasElementOrOffscreenCanvas::OffscreenCanvas(canvas) => canvas.placeholder(),
        }
    }
}

/// Non rooted variant of [`crate::dom::bindings::codegen::Bindings::HTMLCanvasElementBinding::RenderingContext`]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
#[derive(Clone, JSTraceable, MallocSizeOf)]
pub(crate) enum RenderingContext {
    Placeholder(Dom<OffscreenCanvas>),
    Context2d(Dom<CanvasRenderingContext2D>),
    WebGL(Dom<WebGLRenderingContext>),
    WebGL2(Dom<WebGL2RenderingContext>),
    #[cfg(feature = "webgpu")]
    WebGPU(Dom<GPUCanvasContext>),
}

impl CanvasContext for RenderingContext {
    type ID = ();

    fn context_id(&self) -> Self::ID {}

    fn canvas(&self) -> HTMLCanvasElementOrOffscreenCanvas {
        match self {
            RenderingContext::Placeholder(context) => (*context.context().unwrap()).canvas(),
            RenderingContext::Context2d(context) => context.canvas(),
            RenderingContext::WebGL(context) => context.canvas(),
            RenderingContext::WebGL2(context) => context.canvas(),
            #[cfg(feature = "webgpu")]
            RenderingContext::WebGPU(context) => context.canvas(),
        }
    }

    fn resize(&self) {
        match self {
            RenderingContext::Placeholder(context) => (*context.context().unwrap()).resize(),
            RenderingContext::Context2d(context) => context.resize(),
            RenderingContext::WebGL(context) => context.resize(),
            RenderingContext::WebGL2(context) => context.resize(),
            #[cfg(feature = "webgpu")]
            RenderingContext::WebGPU(context) => context.resize(),
        }
    }

    fn get_image_data(&self) -> Option<Snapshot> {
        match self {
            RenderingContext::Placeholder(context) => {
                (*context.context().unwrap()).get_image_data()
            },
            RenderingContext::Context2d(context) => context.get_image_data(),
            RenderingContext::WebGL(context) => context.get_image_data(),
            RenderingContext::WebGL2(context) => context.get_image_data(),
            #[cfg(feature = "webgpu")]
            RenderingContext::WebGPU(context) => context.get_image_data(),
        }
    }

    fn origin_is_clean(&self) -> bool {
        match self {
            RenderingContext::Placeholder(context) => {
                (*context.context().unwrap()).origin_is_clean()
            },
            RenderingContext::Context2d(context) => context.origin_is_clean(),
            RenderingContext::WebGL(context) => context.origin_is_clean(),
            RenderingContext::WebGL2(context) => context.origin_is_clean(),
            #[cfg(feature = "webgpu")]
            RenderingContext::WebGPU(context) => context.origin_is_clean(),
        }
    }

    fn size(&self) -> Size2D<u64> {
        match self {
            RenderingContext::Placeholder(context) => (*context.context().unwrap()).size(),
            RenderingContext::Context2d(context) => context.size(),
            RenderingContext::WebGL(context) => context.size(),
            RenderingContext::WebGL2(context) => context.size(),
            #[cfg(feature = "webgpu")]
            RenderingContext::WebGPU(context) => context.size(),
        }
    }

    fn mark_as_dirty(&self) {
        match self {
            RenderingContext::Placeholder(context) => (*context.context().unwrap()).mark_as_dirty(),
            RenderingContext::Context2d(context) => context.mark_as_dirty(),
            RenderingContext::WebGL(context) => context.mark_as_dirty(),
            RenderingContext::WebGL2(context) => context.mark_as_dirty(),
            #[cfg(feature = "webgpu")]
            RenderingContext::WebGPU(context) => context.mark_as_dirty(),
        }
    }

    fn update_rendering(&self) {
        match self {
            RenderingContext::Placeholder(context) => {
                (*context.context().unwrap()).update_rendering()
            },
            RenderingContext::Context2d(context) => context.update_rendering(),
            RenderingContext::WebGL(context) => context.update_rendering(),
            RenderingContext::WebGL2(context) => context.update_rendering(),
            #[cfg(feature = "webgpu")]
            RenderingContext::WebGPU(context) => context.update_rendering(),
        }
    }

    fn onscreen(&self) -> bool {
        match self {
            RenderingContext::Placeholder(context) => (*context.context().unwrap()).onscreen(),
            RenderingContext::Context2d(context) => context.onscreen(),
            RenderingContext::WebGL(context) => context.onscreen(),
            RenderingContext::WebGL2(context) => context.onscreen(),
            #[cfg(feature = "webgpu")]
            RenderingContext::WebGPU(context) => context.onscreen(),
        }
    }
}

/// Non rooted variant of [`crate::dom::bindings::codegen::Bindings::OffscreenCanvasBinding::OffscreenRenderingContext`]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
#[derive(Clone, JSTraceable, MallocSizeOf)]
pub(crate) enum OffscreenRenderingContext {
    Context2d(Dom<OffscreenCanvasRenderingContext2D>),
    //WebGL(Dom<WebGLRenderingContext>),
    //WebGL2(Dom<WebGL2RenderingContext>),
    //#[cfg(feature = "webgpu")]
    //WebGPU(Dom<GPUCanvasContext>),
}

impl CanvasContext for OffscreenRenderingContext {
    type ID = ();

    fn context_id(&self) -> Self::ID {}

    fn canvas(&self) -> HTMLCanvasElementOrOffscreenCanvas {
        match self {
            OffscreenRenderingContext::Context2d(context) => context.canvas(),
        }
    }

    fn resize(&self) {
        match self {
            OffscreenRenderingContext::Context2d(context) => context.resize(),
        }
    }

    fn get_image_data(&self) -> Option<Snapshot> {
        match self {
            OffscreenRenderingContext::Context2d(context) => context.get_image_data(),
        }
    }

    fn origin_is_clean(&self) -> bool {
        match self {
            OffscreenRenderingContext::Context2d(context) => context.origin_is_clean(),
        }
    }

    fn size(&self) -> Size2D<u64> {
        match self {
            OffscreenRenderingContext::Context2d(context) => context.size(),
        }
    }

    fn mark_as_dirty(&self) {
        match self {
            OffscreenRenderingContext::Context2d(context) => context.mark_as_dirty(),
        }
    }

    fn update_rendering(&self) {
        match self {
            OffscreenRenderingContext::Context2d(context) => context.update_rendering(),
        }
    }

    fn onscreen(&self) -> bool {
        match self {
            OffscreenRenderingContext::Context2d(context) => context.onscreen(),
        }
    }
}
