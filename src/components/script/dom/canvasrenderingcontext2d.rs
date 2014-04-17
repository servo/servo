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

use serialize::{Encodable, Encoder};

#[deriving(Encodable)]
pub struct CanvasRenderingContext2D {
    priv owner: JS<Window>,
    priv reflector_: Reflector,
    priv extra: Untraceable,
}

struct Untraceable {
    drawtarget: DrawTarget,
}

impl<S: Encoder> Encodable<S> for Untraceable {
    fn encode(&self, _: &mut S) {
    }
}

impl CanvasRenderingContext2D {
    pub fn new_inherited(owner: JS<Window>) -> CanvasRenderingContext2D {
        CanvasRenderingContext2D {
            owner: owner,
            reflector_: Reflector::new(),
            extra: Untraceable {
                drawtarget: DrawTarget::new(SkiaBackend, Size2D(100i32, 100i32), B8G8R8A8),
            }
        }
    }

    pub fn new(owner: &JS<Window>) -> JS<CanvasRenderingContext2D> {
        reflect_dom_object(~CanvasRenderingContext2D::new_inherited(owner.clone()), owner,
                           CanvasRenderingContext2DBinding::Wrap)
    }

    pub fn recreate(&mut self, width: u32, height: u32) {
        self.extra.drawtarget = DrawTarget::new(SkiaBackend, Size2D(width as i32, height as i32), B8G8R8A8);
    }

    pub fn FillRect(&self, x: f32, y: f32, width: f32, height: f32) {
        let colorpattern = ColorPattern(Color(1.0, 0.0, 0.0, 0.0));
        let rect = Rect(Point2D(x, y), Size2D(width, height));
        let drawopts = DrawOptions(1.0, 0);
        self.extra.drawtarget.fill_rect(&rect, &colorpattern, Some(&drawopts));
    }

    pub fn ClearRect(&self, x: f32, y: f32, width: f32, height: f32) {
        let rect = Rect(Point2D(x, y), Size2D(width, height));
        self.extra.drawtarget.clear_rect(&rect);
    }

    pub fn StrokeRect(&self, x: f32, y: f32, width: f32, height: f32) {
        let colorpattern = ColorPattern(Color(1.0, 0.0, 0.0, 0.0));
        let rect = Rect(Point2D(x, y), Size2D(width, height));
        let strokeopts = StrokeOptions(10.0, 10.0);
        let drawopts = DrawOptions(1.0, 0);
        self.extra.drawtarget.stroke_rect(&rect, &colorpattern, &strokeopts, &drawopts);
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
