/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::CanvasRenderingContext2DBinding;
use dom::bindings::codegen::Bindings::CanvasRenderingContext2DBinding::CanvasRenderingContext2DMethods;
use dom::bindings::global::{GlobalRef, GlobalField};
use dom::bindings::js::{JS, JSRef, Temporary};
use dom::bindings::trace::Untraceable;
use dom::bindings::utils::{Reflector, Reflectable, reflect_dom_object};
use dom::htmlcanvaselement::HTMLCanvasElement;

use geom::point::Point2D;
use geom::rect::Rect;
use geom::size::Size2D;

use canvas::canvas_render_task::{CanvasMsg, CanvasRenderTask, ClearRect, Close, FillRect, Recreate, StrokeRect};

#[jstraceable]
#[must_root]
pub struct CanvasRenderingContext2D {
    reflector_: Reflector,
    global: GlobalField,
    renderer: Untraceable<Sender<CanvasMsg>>,
    canvas: JS<HTMLCanvasElement>,
}

impl CanvasRenderingContext2D {
    pub fn new_inherited(global: &GlobalRef, canvas: JSRef<HTMLCanvasElement>, size: Size2D<i32>) -> CanvasRenderingContext2D {
        CanvasRenderingContext2D {
            reflector_: Reflector::new(),
            global: GlobalField::from_rooted(global),
            renderer: Untraceable::new(CanvasRenderTask::start(size)),
            canvas: JS::from_rooted(canvas),
        }
    }

    pub fn new(global: &GlobalRef, canvas: JSRef<HTMLCanvasElement>, size: Size2D<i32>) -> Temporary<CanvasRenderingContext2D> {
        reflect_dom_object(box CanvasRenderingContext2D::new_inherited(global, canvas, size),
                           global, CanvasRenderingContext2DBinding::Wrap)
    }

    pub fn recreate(&self, size: Size2D<i32>) {
        self.renderer.deref().send(Recreate(size));
    }
}

impl<'a> CanvasRenderingContext2DMethods for JSRef<'a, CanvasRenderingContext2D> {
    fn Canvas(self) -> Temporary<HTMLCanvasElement> {
        Temporary::new(self.canvas)
    }

    fn FillRect(self, x: f64, y: f64, width: f64, height: f64) {
        let rect = Rect(Point2D(x as f32, y as f32), Size2D(width as f32, height as f32));
        self.renderer.deref().send(FillRect(rect));
    }

    fn ClearRect(self, x: f64, y: f64, width: f64, height: f64) {
        let rect = Rect(Point2D(x as f32, y as f32), Size2D(width as f32, height as f32));
        self.renderer.deref().send(ClearRect(rect));
    }

    fn StrokeRect(self, x: f64, y: f64, width: f64, height: f64) {
        let rect = Rect(Point2D(x as f32, y as f32), Size2D(width as f32, height as f32));
        self.renderer.deref().send(StrokeRect(rect));
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
