/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::CanvasRenderingContext2DBinding;
use dom::bindings::js::JS;
use dom::bindings::utils::{Reflector, Reflectable, reflect_dom_object};
use dom::window::Window;

use azure::azure_hl::{DrawTarget, Color, B8G8R8A8, SkiaBackend, StrokeOptions, DrawOptions};
use azure::azure_hl::ColorPattern;
use geom::point::Point2D;
use geom::rect::Rect;
use geom::size::Size2D;

use std::comm;
use std::task;

use serialize::{Encodable, Encoder};

#[deriving(Encodable)]
pub struct CanvasRenderingContext2D {
    priv owner: JS<Window>,
    priv reflector_: Reflector,
    priv extra: Untraceable,
}

struct Untraceable {
    renderer: Sender<CanvasMsg>,
}

impl<S: Encoder> Encodable<S> for Untraceable {
    fn encode(&self, _: &mut S) {
    }
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
            fill_color: ColorPattern(Color(0., 0., 0., 1.)),
            stroke_color: ColorPattern(Color(0., 0., 0., 1.)),
            stroke_opts: StrokeOptions(1.0, 1.0),
        }
    }

    fn start(size: Size2D<i32>) -> Sender<CanvasMsg> {
        let (chan, port) = comm::channel::<CanvasMsg>();
        let builder = task::task().named("CanvasTask");
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
        let drawopts = DrawOptions(1.0, 0);
        self.drawtarget.fill_rect(rect, &self.fill_color, Some(&drawopts));
    }

    fn clear_rect(&self, rect: &Rect<f32>) {
        self.drawtarget.clear_rect(rect);
    }

    fn stroke_rect(&self, rect: &Rect<f32>) {
        let drawopts = DrawOptions(1.0, 0);
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
    pub fn new_inherited(owner: JS<Window>, size: Size2D<i32>) -> CanvasRenderingContext2D {
        CanvasRenderingContext2D {
            owner: owner,
            reflector_: Reflector::new(),
            extra: Untraceable {
                renderer: CanvasRenderTask::start(size),
            }
        }
    }

    pub fn new(owner: &JS<Window>, size: Size2D<i32>) -> JS<CanvasRenderingContext2D> {
        reflect_dom_object(~CanvasRenderingContext2D::new_inherited(owner.clone(), size),
                           owner, CanvasRenderingContext2DBinding::Wrap)
    }

    pub fn recreate(&mut self, size: Size2D<i32>) {
        self.extra.renderer.send(Recreate(size));
    }

    pub fn FillRect(&self, x: f32, y: f32, width: f32, height: f32) {
        let rect = Rect(Point2D(x, y), Size2D(width, height));
        self.extra.renderer.send(FillRect(rect));
    }

    pub fn ClearRect(&self, x: f32, y: f32, width: f32, height: f32) {
        let rect = Rect(Point2D(x, y), Size2D(width, height));
        self.extra.renderer.send(ClearRect(rect));
    }

    pub fn StrokeRect(&self, x: f32, y: f32, width: f32, height: f32) {
        let rect = Rect(Point2D(x, y), Size2D(width, height));
        self.extra.renderer.send(StrokeRect(rect));
    }
}

impl Reflectable for CanvasRenderingContext2D {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        &self.reflector_
    }

    fn mut_reflector<'a>(&'a mut self) -> &'a mut Reflector {
        &mut self.reflector_
    }
}

#[unsafe_destructor]
impl Drop for CanvasRenderingContext2D {
    fn drop(&mut self) {
        self.extra.renderer.send(Close);
    }
}