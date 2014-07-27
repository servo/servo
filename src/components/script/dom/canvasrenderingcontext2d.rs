/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::CanvasRenderingContext2DBinding;
use dom::bindings::codegen::Bindings::CanvasRenderingContext2DBinding::CanvasRenderingContext2DMethods;
use dom::bindings::global::{GlobalRef, GlobalField};
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::trace::Untraceable;
use dom::bindings::utils::{Reflector, Reflectable, reflect_dom_object};

use azure::azure_hl::{DrawTarget, Color, B8G8R8A8, SkiaBackend, StrokeOptions, DrawOptions};
use azure::azure_hl::ColorPattern;
use geom::point::Point2D;
use geom::rect::Rect;
use geom::size::Size2D;

use std::comm;
use std::task::TaskBuilder;

#[deriving(Encodable)]
pub struct CanvasRenderingContext2D {
    reflector_: Reflector,
    global: GlobalField,
    renderer: Untraceable<Sender<CanvasMsg>>,
}

enum CanvasMsg {
    FillRect(Rect<f32>),
    ClearRect(Rect<f32>),
    StrokeRect(Rect<f32>),
    Recreate(Size2D<i32>),
    Close,
}

struct CanvasRenderTask {
    drawtarget: DrawTarget,
    fill_color: ColorPattern,
    stroke_color: ColorPattern,
    stroke_opts: StrokeOptions,
}

impl CanvasRenderTask {
    fn new(size: Size2D<i32>) -> CanvasRenderTask {
        CanvasRenderTask {
            drawtarget: CanvasRenderTask::create(size),
            fill_color: ColorPattern::new(Color::new(0., 0., 0., 1.)),
            stroke_color: ColorPattern::new(Color::new(0., 0., 0., 1.)),
            stroke_opts: StrokeOptions::new(1.0, 1.0),
        }
    }

    fn start(size: Size2D<i32>) -> Sender<CanvasMsg> {
        let (chan, port) = comm::channel::<CanvasMsg>();
        let builder = TaskBuilder::new().named("CanvasTask");
        builder.spawn(proc() {
            let mut renderer = CanvasRenderTask::new(size);

            loop {
                match port.recv() {
                    FillRect(ref rect) => renderer.fill_rect(rect),
                    StrokeRect(ref rect) => renderer.stroke_rect(rect),
                    ClearRect(ref rect) => renderer.clear_rect(rect),
                    Recreate(size) => renderer.recreate(size),
                    Close => break,
                }
            }
        });
        chan
    }

    fn fill_rect(&self, rect: &Rect<f32>) {
        let drawopts = DrawOptions::new(1.0, 0);
        self.drawtarget.fill_rect(rect, &self.fill_color, Some(&drawopts));
    }

    fn clear_rect(&self, rect: &Rect<f32>) {
        self.drawtarget.clear_rect(rect);
    }

    fn stroke_rect(&self, rect: &Rect<f32>) {
        let drawopts = DrawOptions::new(1.0, 0);
        self.drawtarget.stroke_rect(rect, &self.stroke_color, &self.stroke_opts, &drawopts);
    }

    fn create(size: Size2D<i32>) -> DrawTarget {
        DrawTarget::new(SkiaBackend, size, B8G8R8A8)
    }

    fn recreate(&mut self, size: Size2D<i32>) {
        self.drawtarget = CanvasRenderTask::create(size);
    }
}

impl CanvasRenderingContext2D {
    pub fn new_inherited(global: &GlobalRef, size: Size2D<i32>) -> CanvasRenderingContext2D {
        CanvasRenderingContext2D {
            reflector_: Reflector::new(),
            global: GlobalField::from_rooted(global),
            renderer: Untraceable::new(CanvasRenderTask::start(size)),
        }
    }

    pub fn new(global: &GlobalRef, size: Size2D<i32>) -> Temporary<CanvasRenderingContext2D> {
        reflect_dom_object(box CanvasRenderingContext2D::new_inherited(global, size),
                           global, CanvasRenderingContext2DBinding::Wrap)
    }

    pub fn recreate(&self, size: Size2D<i32>) {
        self.renderer.send(Recreate(size));
    }
}

impl<'a> CanvasRenderingContext2DMethods for JSRef<'a, CanvasRenderingContext2D> {
    fn FillRect(&self, x: f64, y: f64, width: f64, height: f64) {
        let rect = Rect(Point2D(x as f32, y as f32), Size2D(width as f32, height as f32));
        self.renderer.send(FillRect(rect));
    }

    fn ClearRect(&self, x: f64, y: f64, width: f64, height: f64) {
        let rect = Rect(Point2D(x as f32, y as f32), Size2D(width as f32, height as f32));
        self.renderer.send(ClearRect(rect));
    }

    fn StrokeRect(&self, x: f64, y: f64, width: f64, height: f64) {
        let rect = Rect(Point2D(x as f32, y as f32), Size2D(width as f32, height as f32));
        self.renderer.send(StrokeRect(rect));
    }
}

impl Reflectable for CanvasRenderingContext2D {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        &self.reflector_
    }
}

#[unsafe_destructor]
impl Drop for CanvasRenderingContext2D {
    fn drop(&mut self) {
        self.renderer.send(Close);
    }
}
