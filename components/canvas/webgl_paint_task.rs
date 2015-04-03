/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use azure::azure::AzFloat;
use azure::azure_hl::{DrawTarget, SurfaceFormat, BackendType, StrokeOptions, DrawOptions, Pattern, Filter, DrawSurfaceOptions};
use compositing::windowing::{WindowMethods};
use geom::point::Point2D;
use geom::rect::Rect;
use geom::size::Size2D;
use gleam::gl;

use layers::platform::surface::{NativeGraphicsMetadata, NativePaintingGraphicsContext, NativeCompositingGraphicsContext};
use layers::rendergl;
use layers::rendergl::RenderContext;
use util::task::spawn_named;

use cssparser::RGBA;
use skia::SkiaGrGLNativeContextRef;
use std::borrow::ToOwned;
use std::sync::mpsc::{channel, Sender};


#[derive(Clone)]
pub enum WebGLMsg {
    Clear(u32),
    ClearColor(f32, f32, f32, f32),
    Render,
}

pub struct WebGLPaintTask {
    size: Size2D<i32>,
    drawtarget: DrawTarget,
    native_graphics_context: NativePaintingGraphicsContext,
}

impl WebGLPaintTask {
    fn new(size: Size2D<i32>, native_graphics_metadata: &NativeGraphicsMetadata) -> WebGLPaintTask {
        let native_graphics_context = NativePaintingGraphicsContext::from_metadata(&native_graphics_metadata);
        let native_compositing_context = create_graphics_context(native_graphics_metadata);
        let draw_target = WebGLPaintTask::create(size, &native_graphics_context, &native_compositing_context);
        WebGLPaintTask {
            size: size,
            drawtarget: draw_target,
            native_graphics_context: native_graphics_context,
        }
        // // GPU painting path:
        // draw_target.make_current();

        // // We mark the native surface as not leaking in case the surfaces
        // // die on their way to the compositor task.
        // let mut native_surface: NativeSurface =
        //     NativeSurface::from_draw_target_backing(draw_target.steal_draw_target_backing());
        // native_surface.mark_wont_leak();
    }

    // fn create(size: Size2D<i32>) -> DrawTarget {
    //     DrawTarget::new(BackendType::Skia, size, SurfaceFormat::B8G8R8A8)
    // }

    fn create(size: Size2D<i32>,
              native_graphics_context: &NativePaintingGraphicsContext,
              native_compositing_context: &NativeCompositingGraphicsContext) -> DrawTarget {
        let graphics_context = native_graphics_context as *const _ as SkiaGrGLNativeContextRef;
        let draw_target = DrawTarget::new_with_fbo(BackendType::Skia,
                                                   graphics_context,
                                                   size,
                                                   SurfaceFormat::B8G8R8A8);
        draw_target.make_current();
        let context = Some(rendergl::RenderContext::new(*native_compositing_context, false));
        draw_target
    }

    pub fn start(size: Size2D<i32>,
                 native_metadata: NativeGraphicsMetadata) -> Sender<WebGLMsg> {
            let (chan, port) = channel::<WebGLMsg>();
        spawn_named("WebGLTask".to_owned(), move || {
            loop {
                let native_metadata = native_metadata;
                let mut painter = WebGLPaintTask::new(size, &native_metadata);
                match port.recv().unwrap() {
                    WebGLMsg::Clear(mask) => {
                      painter.clear(mask);
                    },
                    WebGLMsg::ClearColor(r, g, b, a) => {
                      painter.clear_color(r, g, b, a);
                    },
                    WebGLMsg::Render => {
                      painter.render();
                    },
                }
            }
        });
        chan
    }

    fn clear(&self, mask: u32) {
        gl::clear(mask);
        self.render();
    }

    fn clear_color(&self, r: f32, g: f32, b: f32, a: f32) {
        gl::clear_color(r, g, b, a);
        self.render();
    }

    fn render(&self) {
        //rendergl::render();
        let pixels = gl::read_pixels(0, 0,
                                    self.size.width as gl::GLsizei,
                                    self.size.height as gl::GLsizei,
                                    gl::RGB, gl::UNSIGNED_BYTE);
        let source_surface = self.drawtarget.create_source_surface_from_data(&pixels,
            self.size, self.size.width * 4, SurfaceFormat::B8G8R8A8);

        let canvas_rect = Rect(Point2D(0i32, 0i32), self.size);
        let draw_surface_options = DrawSurfaceOptions::new(Filter::Linear, true);
        let draw_options = DrawOptions::new(1.0f64 as AzFloat, 0);

        self.drawtarget.draw_surface(source_surface,
                                     canvas_rect.to_azfloat(),
                                     canvas_rect.to_azfloat(),
                                     draw_surface_options, draw_options);

    }

}

#[cfg(target_os="linux")]
pub fn create_graphics_context(native_metadata: &NativeGraphicsMetadata)
                                -> NativeCompositingGraphicsContext {
    NativeCompositingGraphicsContext::from_display(native_metadata.display)
}
#[cfg(not(target_os="linux"))]
pub fn create_graphics_context(_: &NativeGraphicsMetadata)
                                -> NativeCompositingGraphicsContext {
    NativeCompositingGraphicsContext::new()
}

pub trait ToAzFloat {
    fn to_azfloat(&self) -> Rect<AzFloat>;
}

impl ToAzFloat for Rect<i32> {
    fn to_azfloat(&self) -> Rect<AzFloat> {
        Rect(Point2D(self.origin.x as AzFloat, self.origin.y as AzFloat),
             Size2D(self.size.width as AzFloat, self.size.height as AzFloat))
    }
}
