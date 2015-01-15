/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::CanvasRenderingContext2DBinding;
use dom::bindings::codegen::Bindings::CanvasRenderingContext2DBinding::CanvasRenderingContext2DMethods;
use dom::bindings::codegen::Bindings::ImageDataBinding::ImageDataMethods;
use dom::bindings::error::Error::IndexSize;
use dom::bindings::error::Fallible;
use dom::bindings::global::{GlobalRef, GlobalField};
use dom::bindings::js::{JS, JSRef, LayoutJS, Temporary};
use dom::bindings::utils::{Reflector, reflect_dom_object};
use dom::htmlcanvaselement::{HTMLCanvasElement, HTMLCanvasElementHelpers};
use dom::imagedata::{ImageData, ImageDataHelpers};

use geom::point::Point2D;
use geom::rect::Rect;
use geom::size::Size2D;

use canvas::canvas_paint_task::{CanvasMsg, CanvasPaintTask};
use canvas::canvas_paint_task::CanvasMsg::{ClearRect, Close, FillRect, Recreate, StrokeRect, GetImageData, PutImageData};

use std::num::{Float, ToPrimitive};
use std::sync::mpsc::{channel, Sender};

#[dom_struct]
pub struct CanvasRenderingContext2D {
    reflector_: Reflector,
    global: GlobalField,
    renderer: Sender<CanvasMsg>,
    canvas: JS<HTMLCanvasElement>,
}

impl CanvasRenderingContext2D {
    fn new_inherited(global: GlobalRef, canvas: JSRef<HTMLCanvasElement>, size: Size2D<i32>) -> CanvasRenderingContext2D {
        CanvasRenderingContext2D {
            reflector_: Reflector::new(),
            global: GlobalField::from_rooted(&global),
            renderer: CanvasPaintTask::start(size),
            canvas: JS::from_rooted(canvas),
        }
    }

    pub fn new(global: GlobalRef, canvas: JSRef<HTMLCanvasElement>, size: Size2D<i32>) -> Temporary<CanvasRenderingContext2D> {
        reflect_dom_object(box CanvasRenderingContext2D::new_inherited(global, canvas, size),
                           global, CanvasRenderingContext2DBinding::Wrap)
    }

    pub fn recreate(&self, size: Size2D<i32>) {
        self.renderer.send(Recreate(size)).unwrap();
    }
}

pub trait LayoutCanvasRenderingContext2DHelpers {
    unsafe fn get_renderer(&self) -> Sender<CanvasMsg>;
}

impl LayoutCanvasRenderingContext2DHelpers for LayoutJS<CanvasRenderingContext2D> {
    unsafe fn get_renderer(&self) -> Sender<CanvasMsg> {
        (*self.unsafe_get()).renderer.clone()
    }
}

impl<'a> CanvasRenderingContext2DMethods for JSRef<'a, CanvasRenderingContext2D> {
    fn Canvas(self) -> Temporary<HTMLCanvasElement> {
        Temporary::new(self.canvas)
    }

    fn FillRect(self, x: f64, y: f64, width: f64, height: f64) {
        let rect = Rect(Point2D(x as f32, y as f32), Size2D(width as f32, height as f32));
        self.renderer.send(FillRect(rect)).unwrap();
    }

    fn ClearRect(self, x: f64, y: f64, width: f64, height: f64) {
        let rect = Rect(Point2D(x as f32, y as f32), Size2D(width as f32, height as f32));
        self.renderer.send(ClearRect(rect)).unwrap();
    }

    fn StrokeRect(self, x: f64, y: f64, width: f64, height: f64) {
        let rect = Rect(Point2D(x as f32, y as f32), Size2D(width as f32, height as f32));
        self.renderer.send(StrokeRect(rect)).unwrap();
    }

    fn CreateImageData(self, sw: f64, sh: f64) -> Fallible<Temporary<ImageData>> {
        if sw == 0.0 || sh == 0.0 {
            return Err(IndexSize)
        }

        Ok(ImageData::new(self.global.root().r(), sw.abs().to_u32().unwrap(), sh.abs().to_u32().unwrap(), None))
    }

    fn CreateImageData_(self, imagedata: JSRef<ImageData>) -> Fallible<Temporary<ImageData>> {
        Ok(ImageData::new(self.global.root().r(), imagedata.Width(), imagedata.Height(), None))
    }

    fn GetImageData(self, sx: f64, sy: f64, sw: f64, sh: f64) -> Fallible<Temporary<ImageData>> {
        if sw == 0.0 || sh == 0.0 {
            return Err(IndexSize)
        }

        let (sender, receiver) = channel::<Vec<u8>>();
        let dest_rect = Rect(Point2D(sx.to_i32().unwrap(), sy.to_i32().unwrap()), Size2D(sw.to_i32().unwrap(), sh.to_i32().unwrap()));
        let canvas_size = self.canvas.root().r().get_size();
        self.renderer.send(GetImageData(dest_rect, canvas_size, sender)).unwrap();
        let data = receiver.recv().unwrap();
        Ok(ImageData::new(self.global.root().r(), sw.abs().to_u32().unwrap(), sh.abs().to_u32().unwrap(), Some(data)))
    }

    fn PutImageData(self, imagedata: JSRef<ImageData>, dx: f64, dy: f64) {
        let data = imagedata.get_data_array(&self.global.root().r());
        let image_data_rect = Rect(Point2D(dx.to_i32().unwrap(), dy.to_i32().unwrap()), imagedata.get_size());
        let dirty_rect = None;
        let canvas_size = self.canvas.root().r().get_size();
        self.renderer.send(PutImageData(data, image_data_rect, dirty_rect, canvas_size)).unwrap()
    }

    fn PutImageData_(self, imagedata: JSRef<ImageData>, dx: f64, dy: f64,
                     dirtyX: f64, dirtyY: f64, dirtyWidth: f64, dirtyHeight: f64) {
        let data = imagedata.get_data_array(&self.global.root().r());
        let image_data_rect = Rect(Point2D(dx.to_i32().unwrap(), dy.to_i32().unwrap()),
                                   Size2D(imagedata.Width().to_i32().unwrap(),
                                          imagedata.Height().to_i32().unwrap()));
        let dirty_rect = Some(Rect(Point2D(dirtyX.to_i32().unwrap(), dirtyY.to_i32().unwrap()),
                                   Size2D(dirtyWidth.to_i32().unwrap(),
                                          dirtyHeight.to_i32().unwrap())));
        let canvas_size = self.canvas.root().r().get_size();
        self.renderer.send(PutImageData(data, image_data_rect, dirty_rect, canvas_size)).unwrap()
    }
}

#[unsafe_destructor]
impl Drop for CanvasRenderingContext2D {
    fn drop(&mut self) {
        self.renderer.send(Close).unwrap();
    }
}
