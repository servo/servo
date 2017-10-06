/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use euclid::Size2D;
use fnv::FnvHasher;
use gleam::gl;
#[cfg(feature = "vector-rasterizer")]
use nanovg::{self, Color, Context, Solidity, Transform, Winding};
use std::collections::HashMap;
use std::hash::BuildHasherDefault;
use std::os::raw::c_int;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread;
use api::DeviceUintSize;
use api::{Command, GeometryKey, Geometry, GeometryItem};
use api::{ImageData, ImageDescriptor, ImageFormat};
use offscreen_gl_context::{ColorAttachmentType, GLContext, GLContextAttributes};
use offscreen_gl_context::NativeGLContext;

static VIEWPORT_WIDTH: i32 = 2048;
static VIEWPORT_HEIGHT: i32 = 2048;

pub enum GeometryRequestMsg {
    Update(GeometryKey, Geometry),
    Delete(GeometryKey),
    Request(GeometryKey, DeviceUintSize),
    EndFrame
}

pub enum GeometryResultMsg {
    Image(GeometryKey, ImageData, ImageDescriptor),
    EndFrame
}

#[cfg(feature = "vector-rasterizer")]
pub fn spawn_svg_renderer(device_pixel_ratio: f32)
                          -> (Sender<GeometryRequestMsg>, Receiver<GeometryResultMsg>) {
    // Used for messages from resource cache -> geometry rendering thread.
    let (msg_tx, msg_rx) = channel();
    // Used for returning results from geometry rendering thread -> resource cache.
    let (result_tx, result_rx) = channel();

    thread::Builder::new()
        .name("SvgThread".to_owned())
        .spawn(move || {
            let mut svg_geometries: HashMap<GeometryKey,
                                            Geometry,
                                            BuildHasherDefault<FnvHasher>> = HashMap::default();
            let size = Size2D::new(VIEWPORT_WIDTH, VIEWPORT_HEIGHT);
            let context_attributes = GLContextAttributes {
                alpha: true,
                depth: true,
                stencil: true,
                antialias: true,
                premultiplied_alpha: true,
                preserve_drawing_buffer: false
            };
            let context = GLContext::<NativeGLContext>::new(size,
                                                            context_attributes,
                                                            ColorAttachmentType::Texture,
                                                            gl::GlType::default(),
                                                            None).unwrap();
            context.make_current().unwrap();
            let vg = Context::create_gl2(nanovg::ANTIALIAS | nanovg::STENCIL_STROKES);

            while let Ok(msg) = msg_rx.recv() {
                match msg {
                    GeometryRequestMsg::Update(key, geometry) => {
                        svg_geometries.insert(key, geometry);
                    }
                    GeometryRequestMsg::Delete(key) => {
                        svg_geometries.remove(&key);
                    }
                    GeometryRequestMsg::Request(key, dimensions) => {
                        let geometry = svg_geometries.get(&key)
                            .expect("Requested geometry wasn't added!");
                        let image_data = paint_svg(&context,
                                                   &vg,
                                                   geometry,
                                                   dimensions,
                                                   device_pixel_ratio);
                        let data = ImageData::Raw(Arc::new(image_data));
                        let descriptor = ImageDescriptor {
                            format: ImageFormat::BGRA8,
                            width: dimensions.width,
                            height: dimensions.height,
                            stride: None,
                            offset: 0,
                            is_opaque: false,
                        };
                        let msg = GeometryResultMsg::Image(key, data, descriptor);
                        result_tx.send(msg).unwrap();
                    }
                    GeometryRequestMsg::EndFrame => {
                        result_tx.send(GeometryResultMsg::EndFrame).unwrap();
                    }
                }
            }
        })
        .expect("Thread spawning failed");
    (msg_tx, result_rx)
}

fn paint_svg(context: &GLContext<NativeGLContext>,
             vg: &Context,
             geometry: &[GeometryItem],
             dimensions: DeviceUintSize,
             device_pixel_ratio: f32)
             -> Vec<u8> {
    context.gl().clear_color(0.0, 0.0, 0.0, 0.0);
    context.gl().clear(gl::COLOR_BUFFER_BIT);

    let canvas_width = (VIEWPORT_WIDTH as f32 / device_pixel_ratio) as u32;
    let canvas_height = (VIEWPORT_HEIGHT as f32 / device_pixel_ratio) as u32;
    vg.begin_frame(canvas_width, canvas_height, device_pixel_ratio);
    // the opengl y-coordinate is inverse with svg, so apply a transform here first.
    let transform = Transform::new(1.0, 0., 0., -1.0, 0., canvas_height as f32);
    vg.transform(transform);
    for item in geometry {
        match item {
            &GeometryItem::Shape(ref shape) => {
                vg.begin_path();
                for command in &shape.path {
                    match command {
                        &Command::MoveTo(point) => vg.move_to(point.x, point.y),
                        &Command::LineTo(point) => vg.line_to(point.x, point.y),
                        &Command::Arc(center, radius, start, end) => {
                            let direction = if start < end {
                                Winding::CCW
                            } else {
                                Winding::CW
                            };
                            vg.arc(center.x, center.y, radius, start, end, direction);
                        }
                    }
                }
                vg.close_path();
                let color = Color::rgba(shape.fill.r, shape.fill.g, shape.fill.b, shape.fill.a);
                vg.path_winding(Solidity::HOLE);
                vg.fill_color(color);
                vg.fill();
            }
        }
    }
    vg.end_frame();
    context.gl().read_pixels(0,
                             0,
                             dimensions.width as c_int,
                             dimensions.height as c_int,
                             gl::BGRA,
                             gl::UNSIGNED_BYTE)
}
