/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Common interfaces for Canvas Contexts

use base::Epoch;
use euclid::default::Size2D;
use pixels::Snapshot;
use script_bindings::root::{Dom, DomRoot};
use webrender_api::ImageKey;

use crate::dom::bindings::codegen::UnionTypes::HTMLCanvasElementOrOffscreenCanvas as RootedHTMLCanvasElementOrOffscreenCanvas;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::node::{Node, NodeDamage};
#[cfg(feature = "webgpu")]
use crate::dom::types::GPUCanvasContext;
use crate::dom::types::{
    CanvasRenderingContext2D, HTMLCanvasElement, ImageBitmapRenderingContext, OffscreenCanvas,
    OffscreenCanvasRenderingContext2D, WebGL2RenderingContext, WebGLRenderingContext,
};

pub(crate) trait LayoutCanvasRenderingContextHelpers {
    /// `None` is rendered as transparent black (cleared canvas)
    fn canvas_data_source(self) -> Option<ImageKey>;
}

/// Non rooted variant of [`crate::dom::bindings::codegen::UnionTypes::HTMLCanvasElementOrOffscreenCanvas`]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
#[derive(Clone, JSTraceable, MallocSizeOf)]
pub(crate) enum HTMLCanvasElementOrOffscreenCanvas {
    HTMLCanvasElement(Dom<HTMLCanvasElement>),
    OffscreenCanvas(Dom<OffscreenCanvas>),
}

impl From<&RootedHTMLCanvasElementOrOffscreenCanvas> for HTMLCanvasElementOrOffscreenCanvas {
    /// Returns a traced version suitable for use as member of other DOM objects.
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    fn from(
        value: &RootedHTMLCanvasElementOrOffscreenCanvas,
    ) -> HTMLCanvasElementOrOffscreenCanvas {
        match value {
            RootedHTMLCanvasElementOrOffscreenCanvas::HTMLCanvasElement(canvas) => {
                HTMLCanvasElementOrOffscreenCanvas::HTMLCanvasElement(canvas.as_traced())
            },
            RootedHTMLCanvasElementOrOffscreenCanvas::OffscreenCanvas(canvas) => {
                HTMLCanvasElementOrOffscreenCanvas::OffscreenCanvas(canvas.as_traced())
            },
        }
    }
}

impl From<&HTMLCanvasElementOrOffscreenCanvas> for RootedHTMLCanvasElementOrOffscreenCanvas {
    /// Returns a rooted version suitable for use on the stack.
    fn from(
        value: &HTMLCanvasElementOrOffscreenCanvas,
    ) -> RootedHTMLCanvasElementOrOffscreenCanvas {
        match value {
            HTMLCanvasElementOrOffscreenCanvas::HTMLCanvasElement(canvas) => {
                RootedHTMLCanvasElementOrOffscreenCanvas::HTMLCanvasElement(canvas.as_rooted())
            },
            HTMLCanvasElementOrOffscreenCanvas::OffscreenCanvas(canvas) => {
                RootedHTMLCanvasElementOrOffscreenCanvas::OffscreenCanvas(canvas.as_rooted())
            },
        }
    }
}

pub(crate) trait CanvasContext {
    type ID;

    fn context_id(&self) -> Self::ID;

    fn canvas(&self) -> Option<RootedHTMLCanvasElementOrOffscreenCanvas>;

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
        if let Some(RootedHTMLCanvasElementOrOffscreenCanvas::HTMLCanvasElement(canvas)) =
            self.canvas()
        {
            canvas.upcast::<Node>().dirty(NodeDamage::Other);
        }
    }

    /// The WebRender [`ImageKey`] of this [`CanvasContext`] if any.
    fn image_key(&self) -> Option<ImageKey>;

    /// Request that the [`CanvasContext`] update the rendering of its contents,
    /// returning `true` if new image was produced.
    ///
    /// Note: If this function returns `true`, script will wait for all images to be updated
    /// before updating the rendering again. Be sure that image updates are always sent
    /// even in the failure case by sending transparent black image or return `false` in the
    /// case of failure.
    fn update_rendering(&self, _canvas_epoch: Epoch) -> bool {
        false
    }

    fn onscreen(&self) -> bool {
        let Some(canvas) = self.canvas() else {
            return false;
        };

        match canvas {
            RootedHTMLCanvasElementOrOffscreenCanvas::HTMLCanvasElement(canvas) => {
                canvas.upcast::<Node>().is_connected()
            },
            // FIXME(34628): Offscreen canvases should be considered offscreen if a placeholder is set.
            // <https://www.w3.org/TR/webgpu/#abstract-opdef-updating-the-rendering-of-a-webgpu-canvas>
            RootedHTMLCanvasElementOrOffscreenCanvas::OffscreenCanvas(_) => false,
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
            HTMLCanvasElementOrOffscreenCanvas::HTMLCanvasElement(canvas) => {
                Some(canvas.as_rooted())
            },
            HTMLCanvasElementOrOffscreenCanvas::OffscreenCanvas(canvas) => canvas.placeholder(),
        }
    }
}

impl CanvasHelpers for RootedHTMLCanvasElementOrOffscreenCanvas {
    fn size(&self) -> Size2D<u32> {
        match self {
            RootedHTMLCanvasElementOrOffscreenCanvas::HTMLCanvasElement(canvas) => {
                canvas.get_size().cast()
            },
            RootedHTMLCanvasElementOrOffscreenCanvas::OffscreenCanvas(canvas) => canvas.get_size(),
        }
    }

    fn canvas(&self) -> Option<DomRoot<HTMLCanvasElement>> {
        match self {
            RootedHTMLCanvasElementOrOffscreenCanvas::HTMLCanvasElement(canvas) => {
                Some(canvas.clone())
            },
            RootedHTMLCanvasElementOrOffscreenCanvas::OffscreenCanvas(canvas) => {
                canvas.placeholder()
            },
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

    fn canvas(&self) -> Option<RootedHTMLCanvasElementOrOffscreenCanvas> {
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

    fn image_key(&self) -> Option<ImageKey> {
        match self {
            RenderingContext::Placeholder(offscreen_canvas) => offscreen_canvas
                .context()
                .and_then(|context| context.image_key()),
            RenderingContext::Context2d(context) => context.image_key(),
            RenderingContext::BitmapRenderer(context) => context.image_key(),
            RenderingContext::WebGL(context) => context.image_key(),
            RenderingContext::WebGL2(context) => context.image_key(),
            #[cfg(feature = "webgpu")]
            RenderingContext::WebGPU(context) => context.image_key(),
        }
    }

    fn update_rendering(&self, canvas_epoch: Epoch) -> bool {
        match self {
            RenderingContext::Placeholder(offscreen_canvas) => offscreen_canvas
                .context()
                .is_some_and(|context| context.update_rendering(canvas_epoch)),
            RenderingContext::Context2d(context) => context.update_rendering(canvas_epoch),
            RenderingContext::BitmapRenderer(context) => context.update_rendering(canvas_epoch),
            RenderingContext::WebGL(context) => context.update_rendering(canvas_epoch),
            RenderingContext::WebGL2(context) => context.update_rendering(canvas_epoch),
            #[cfg(feature = "webgpu")]
            RenderingContext::WebGPU(context) => context.update_rendering(canvas_epoch),
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
    // WebGL(Dom<WebGLRenderingContext>),
    // WebGL2(Dom<WebGL2RenderingContext>),
    // #[cfg(feature = "webgpu")]
    // WebGPU(Dom<GPUCanvasContext>),
    Detached,
}

impl CanvasContext for OffscreenRenderingContext {
    type ID = ();

    fn context_id(&self) -> Self::ID {}

    fn canvas(&self) -> Option<RootedHTMLCanvasElementOrOffscreenCanvas> {
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

    fn image_key(&self) -> Option<ImageKey> {
        None
    }

    fn update_rendering(&self, canvas_epoch: Epoch) -> bool {
        match self {
            OffscreenRenderingContext::Context2d(context) => context.update_rendering(canvas_epoch),
            OffscreenRenderingContext::BitmapRenderer(context) => {
                context.update_rendering(canvas_epoch)
            },
            OffscreenRenderingContext::Detached => false,
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
