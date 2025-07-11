/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Common interfaces for Canvas Contexts

use euclid::default::Size2D;
use layout_api::HTMLCanvasData;
use pixels::Snapshot;
use script_bindings::root::{Dom, DomRoot};
use webrender_api::ImageKey;

use crate::dom::bindings::codegen::UnionTypes::HTMLCanvasElementOrOffscreenCanvas;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::htmlcanvaselement::HTMLCanvasElement;
use crate::dom::node::{Node, NodeDamage};
#[cfg(feature = "webgpu")]
use crate::dom::types::GPUCanvasContext;
use crate::dom::types::{
    CanvasRenderingContext2D, ImageBitmapRenderingContext, OffscreenCanvas,
    OffscreenCanvasRenderingContext2D, WebGL2RenderingContext, WebGLRenderingContext,
};

pub(crate) trait LayoutCanvasRenderingContextHelpers {
    /// `None` is rendered as transparent black (cleared canvas)
    fn canvas_data_source(self) -> Option<ImageKey>;
}

pub(crate) trait LayoutHTMLCanvasElementHelpers {
    fn data(self) -> HTMLCanvasData;
}

pub(crate) trait CanvasContext {
    type ID;

    fn context_id(&self) -> Self::ID;

    fn canvas(&self) -> Option<HTMLCanvasElementOrOffscreenCanvas>;

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

    fn mark_as_dirty(&self) {
        if let Some(HTMLCanvasElementOrOffscreenCanvas::HTMLCanvasElement(canvas)) = &self.canvas()
        {
            canvas.upcast::<Node>().dirty(NodeDamage::Other);
        }
    }

    fn update_rendering(&self) {}

    fn onscreen(&self) -> bool {
        let Some(canvas) = self.canvas() else {
            return false;
        };

        match canvas {
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
    fn size(&self) -> Size2D<u32>;
    fn canvas(&self) -> Option<DomRoot<HTMLCanvasElement>>;
}

impl CanvasHelpers for HTMLCanvasElementOrOffscreenCanvas {
    fn size(&self) -> Size2D<u32> {
        match self {
            HTMLCanvasElementOrOffscreenCanvas::HTMLCanvasElement(canvas) => {
                canvas.get_size().cast()
            },
            HTMLCanvasElementOrOffscreenCanvas::OffscreenCanvas(canvas) => canvas.get_size(),
        }
    }

    fn canvas(&self) -> Option<DomRoot<HTMLCanvasElement>> {
        match self {
            HTMLCanvasElementOrOffscreenCanvas::HTMLCanvasElement(canvas) => Some(canvas.clone()),
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
    BitmapRenderer(Dom<ImageBitmapRenderingContext>),
    WebGL(Dom<WebGLRenderingContext>),
    WebGL2(Dom<WebGL2RenderingContext>),
    #[cfg(feature = "webgpu")]
    WebGPU(Dom<GPUCanvasContext>),
}

impl CanvasContext for RenderingContext {
    type ID = ();

    fn context_id(&self) -> Self::ID {}

    fn canvas(&self) -> Option<HTMLCanvasElementOrOffscreenCanvas> {
        match self {
            RenderingContext::Placeholder(offscreen_canvas) => offscreen_canvas.context()?.canvas(),
            RenderingContext::Context2d(context) => context.canvas(),
            RenderingContext::BitmapRenderer(context) => context.canvas(),
            RenderingContext::WebGL(context) => context.canvas(),
            RenderingContext::WebGL2(context) => context.canvas(),
            #[cfg(feature = "webgpu")]
            RenderingContext::WebGPU(context) => context.canvas(),
        }
    }

    fn resize(&self) {
        match self {
            RenderingContext::Placeholder(offscreen_canvas) => {
                if let Some(context) = offscreen_canvas.context() {
                    context.resize()
                }
            },
            RenderingContext::Context2d(context) => context.resize(),
            RenderingContext::BitmapRenderer(context) => context.resize(),
            RenderingContext::WebGL(context) => context.resize(),
            RenderingContext::WebGL2(context) => context.resize(),
            #[cfg(feature = "webgpu")]
            RenderingContext::WebGPU(context) => context.resize(),
        }
    }

    fn reset_bitmap(&self) {
        match self {
            RenderingContext::Placeholder(offscreen_canvas) => {
                if let Some(context) = offscreen_canvas.context() {
                    context.reset_bitmap()
                }
            },
            RenderingContext::Context2d(context) => context.reset_bitmap(),
            RenderingContext::BitmapRenderer(context) => context.reset_bitmap(),
            RenderingContext::WebGL(context) => context.reset_bitmap(),
            RenderingContext::WebGL2(context) => context.reset_bitmap(),
            #[cfg(feature = "webgpu")]
            RenderingContext::WebGPU(context) => context.reset_bitmap(),
        }
    }

    fn get_image_data(&self) -> Option<Snapshot> {
        match self {
            RenderingContext::Placeholder(offscreen_canvas) => {
                offscreen_canvas.context()?.get_image_data()
            },
            RenderingContext::Context2d(context) => context.get_image_data(),
            RenderingContext::BitmapRenderer(context) => context.get_image_data(),
            RenderingContext::WebGL(context) => context.get_image_data(),
            RenderingContext::WebGL2(context) => context.get_image_data(),
            #[cfg(feature = "webgpu")]
            RenderingContext::WebGPU(context) => context.get_image_data(),
        }
    }

    fn origin_is_clean(&self) -> bool {
        match self {
            RenderingContext::Placeholder(offscreen_canvas) => offscreen_canvas
                .context()
                .is_none_or(|context| context.origin_is_clean()),
            RenderingContext::Context2d(context) => context.origin_is_clean(),
            RenderingContext::BitmapRenderer(context) => context.origin_is_clean(),
            RenderingContext::WebGL(context) => context.origin_is_clean(),
            RenderingContext::WebGL2(context) => context.origin_is_clean(),
            #[cfg(feature = "webgpu")]
            RenderingContext::WebGPU(context) => context.origin_is_clean(),
        }
    }

    fn size(&self) -> Size2D<u32> {
        match self {
            RenderingContext::Placeholder(offscreen_canvas) => offscreen_canvas
                .context()
                .map(|context| context.size())
                .unwrap_or_default(),
            RenderingContext::Context2d(context) => context.size(),
            RenderingContext::BitmapRenderer(context) => context.size(),
            RenderingContext::WebGL(context) => context.size(),
            RenderingContext::WebGL2(context) => context.size(),
            #[cfg(feature = "webgpu")]
            RenderingContext::WebGPU(context) => context.size(),
        }
    }

    fn mark_as_dirty(&self) {
        match self {
            RenderingContext::Placeholder(offscreen_canvas) => {
                if let Some(context) = offscreen_canvas.context() {
                    context.mark_as_dirty()
                }
            },
            RenderingContext::Context2d(context) => context.mark_as_dirty(),
            RenderingContext::BitmapRenderer(context) => context.mark_as_dirty(),
            RenderingContext::WebGL(context) => context.mark_as_dirty(),
            RenderingContext::WebGL2(context) => context.mark_as_dirty(),
            #[cfg(feature = "webgpu")]
            RenderingContext::WebGPU(context) => context.mark_as_dirty(),
        }
    }

    fn update_rendering(&self) {
        match self {
            RenderingContext::Placeholder(offscreen_canvas) => {
                if let Some(context) = offscreen_canvas.context() {
                    context.update_rendering()
                }
            },
            RenderingContext::Context2d(context) => context.update_rendering(),
            RenderingContext::BitmapRenderer(context) => context.update_rendering(),
            RenderingContext::WebGL(context) => context.update_rendering(),
            RenderingContext::WebGL2(context) => context.update_rendering(),
            #[cfg(feature = "webgpu")]
            RenderingContext::WebGPU(context) => context.update_rendering(),
        }
    }

    fn onscreen(&self) -> bool {
        match self {
            RenderingContext::Placeholder(offscreen_canvas) => offscreen_canvas
                .context()
                .is_some_and(|context| context.onscreen()),
            RenderingContext::Context2d(context) => context.onscreen(),
            RenderingContext::BitmapRenderer(context) => context.onscreen(),
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
    BitmapRenderer(Dom<ImageBitmapRenderingContext>),
    //WebGL(Dom<WebGLRenderingContext>),
    //WebGL2(Dom<WebGL2RenderingContext>),
    //#[cfg(feature = "webgpu")]
    //WebGPU(Dom<GPUCanvasContext>),
    Detached,
}

impl CanvasContext for OffscreenRenderingContext {
    type ID = ();

    fn context_id(&self) -> Self::ID {}

    fn canvas(&self) -> Option<HTMLCanvasElementOrOffscreenCanvas> {
        match self {
            OffscreenRenderingContext::Context2d(context) => context.canvas(),
            OffscreenRenderingContext::BitmapRenderer(context) => context.canvas(),
            OffscreenRenderingContext::Detached => None,
        }
    }

    fn resize(&self) {
        match self {
            OffscreenRenderingContext::Context2d(context) => context.resize(),
            OffscreenRenderingContext::BitmapRenderer(context) => context.resize(),
            OffscreenRenderingContext::Detached => {},
        }
    }

    fn reset_bitmap(&self) {
        match self {
            OffscreenRenderingContext::Context2d(context) => context.reset_bitmap(),
            OffscreenRenderingContext::BitmapRenderer(context) => context.reset_bitmap(),
            OffscreenRenderingContext::Detached => {},
        }
    }

    fn get_image_data(&self) -> Option<Snapshot> {
        match self {
            OffscreenRenderingContext::Context2d(context) => context.get_image_data(),
            OffscreenRenderingContext::BitmapRenderer(context) => context.get_image_data(),
            OffscreenRenderingContext::Detached => None,
        }
    }

    fn origin_is_clean(&self) -> bool {
        match self {
            OffscreenRenderingContext::Context2d(context) => context.origin_is_clean(),
            OffscreenRenderingContext::BitmapRenderer(context) => context.origin_is_clean(),
            OffscreenRenderingContext::Detached => true,
        }
    }

    fn size(&self) -> Size2D<u32> {
        match self {
            OffscreenRenderingContext::Context2d(context) => context.size(),
            OffscreenRenderingContext::BitmapRenderer(context) => context.size(),
            OffscreenRenderingContext::Detached => Size2D::default(),
        }
    }

    fn mark_as_dirty(&self) {
        match self {
            OffscreenRenderingContext::Context2d(context) => context.mark_as_dirty(),
            OffscreenRenderingContext::BitmapRenderer(context) => context.mark_as_dirty(),
            OffscreenRenderingContext::Detached => {},
        }
    }

    fn update_rendering(&self) {
        match self {
            OffscreenRenderingContext::Context2d(context) => context.update_rendering(),
            OffscreenRenderingContext::BitmapRenderer(context) => context.update_rendering(),
            OffscreenRenderingContext::Detached => {},
        }
    }

    fn onscreen(&self) -> bool {
        match self {
            OffscreenRenderingContext::Context2d(context) => context.onscreen(),
            OffscreenRenderingContext::BitmapRenderer(context) => context.onscreen(),
            OffscreenRenderingContext::Detached => false,
        }
    }
}
