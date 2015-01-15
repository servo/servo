/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use azure::azure_hl::{DrawTarget, SurfaceFormat, BackendType, StrokeOptions, DrawOptions};
use azure::azure_hl::{ColorPattern, PatternRef, JoinStyle, CapStyle, DrawSurfaceOptions, Filter};
use azure::AzFloat;
use geom::point::Point2D;
use geom::rect::Rect;
use geom::size::Size2D;
use gfx::color;
use util::task::spawn_named;
use util::vec::byte_swap;

use std::borrow::ToOwned;
use std::ops::Add;
use std::sync::mpsc::{channel, Sender};

#[derive(Clone)]
pub enum CanvasMsg {
    FillRect(Rect<f32>),
    ClearRect(Rect<f32>),
    StrokeRect(Rect<f32>),
    Recreate(Size2D<i32>),
    SendPixelContents(Sender<Vec<u8>>),
    GetImageData(Rect<i32>, Size2D<i32>, Sender<Vec<u8>>),
    PutImageData(Vec<u8>, Rect<i32>, Option<Rect<i32>>, Size2D<i32>),
    Close,
}

pub struct CanvasPaintTask<'a> {
    drawtarget: DrawTarget,
    fill_color: ColorPattern,
    stroke_color: ColorPattern,
    stroke_opts: StrokeOptions<'a>,
}

impl<'a> CanvasPaintTask<'a> {
    fn new(size: Size2D<i32>) -> CanvasPaintTask<'a> {
        CanvasPaintTask {
            drawtarget: CanvasPaintTask::create(size),
            fill_color: ColorPattern::new(color::black()),
            stroke_color: ColorPattern::new(color::black()),
            stroke_opts: StrokeOptions::new(1.0, JoinStyle::MiterOrBevel, CapStyle::Butt, 1.0, &[]),
        }
    }

    pub fn start(size: Size2D<i32>) -> Sender<CanvasMsg> {
        let (chan, port) = channel::<CanvasMsg>();
        spawn_named("CanvasTask".to_owned(), move || {
            let mut painter = CanvasPaintTask::new(size);

            loop {
                match port.recv().unwrap() {
                    CanvasMsg::FillRect(ref rect) => painter.fill_rect(rect),
                    CanvasMsg::StrokeRect(ref rect) => painter.stroke_rect(rect),
                    CanvasMsg::ClearRect(ref rect) => painter.clear_rect(rect),
                    CanvasMsg::Recreate(size) => painter.recreate(size),
                    CanvasMsg::SendPixelContents(chan) => painter.send_pixel_contents(chan),
                    CanvasMsg::GetImageData(dest_rect, canvas_size, chan) => painter.get_image_data(dest_rect, canvas_size, chan),
                    CanvasMsg::PutImageData(imagedata, image_data_rect, dirty_rect, canvas_size)
                        => painter.put_image_data(imagedata, image_data_rect, dirty_rect, canvas_size),
                    CanvasMsg::Close => break,
                }
            }
        });
        chan
    }

    fn fill_rect(&self, rect: &Rect<f32>) {
        let drawopts = DrawOptions::new(1.0, 0);
        self.drawtarget.fill_rect(rect, PatternRef::Color(&self.fill_color), Some(&drawopts));
    }

    fn clear_rect(&self, rect: &Rect<f32>) {
        self.drawtarget.clear_rect(rect);
    }

    fn stroke_rect(&self, rect: &Rect<f32>) {
        let drawopts = DrawOptions::new(1.0, 0);
        self.drawtarget.stroke_rect(rect, &self.stroke_color, &self.stroke_opts, &drawopts);
    }

    fn create(size: Size2D<i32>) -> DrawTarget {
        DrawTarget::new(BackendType::Skia, size, SurfaceFormat::B8G8R8A8)
    }

    fn recreate(&mut self, size: Size2D<i32>) {
        self.drawtarget = CanvasPaintTask::create(size);
    }

    fn send_pixel_contents(&mut self, chan: Sender<Vec<u8>>) {
        self.drawtarget.snapshot().get_data_surface().with_data(|element| {
            chan.send(element.to_vec()).unwrap();
        })
    }

    fn get_image_data(&self, mut dest_rect: Rect<i32>, canvas_size: Size2D<i32>, chan: Sender<Vec<u8>>) {
        if dest_rect.size.width < 0 {
            dest_rect.size.width = -dest_rect.size.width;
            dest_rect.origin.x -= dest_rect.size.width;
        }
        if dest_rect.size.height < 0 {
            dest_rect.size.height = -dest_rect.size.height;
            dest_rect.origin.y -= dest_rect.size.height;
        }
        if dest_rect.size.width == 0 {
            dest_rect.size.width = 1;
        }
        if dest_rect.size.height == 0 {
            dest_rect.size.height = 1;
        }

        let canvas_rect = Rect(Point2D(0i32, 0i32), canvas_size);
        let src_read_rect = canvas_rect.intersection(&dest_rect).unwrap_or(Rect::zero());

        let mut dest_data = Vec::new();
        //load the canvas data to the source vector
        if !src_read_rect.is_empty() && canvas_size.width != 0 && canvas_size.height != 0 {
            let data_surface = self.drawtarget.snapshot().get_data_surface();
            let mut src_data = Vec::new();
            data_surface.with_data(|element| {
                src_data = element.to_vec();
            });

            let stride = data_surface.stride();

            //start offset of the copyable rectangle
            let mut src = (src_read_rect.origin.y * stride + src_read_rect.origin.x * 4) as usize;
            //copy the data to the destination vector
            for _ in range(0, src_read_rect.size.height) {
                let row = &src_data[src .. src + (4 * src_read_rect.size.width) as usize];
                dest_data.push_all(row);
                src += stride as usize;
            }
        }
        // bgra -> rgba
        byte_swap(dest_data.as_mut_slice());
        chan.send(dest_data).unwrap();
    }

    fn put_image_data(&mut self, mut imagedata: Vec<u8>, image_data_rect: Rect<i32>,
                      dirty_rect: Option<Rect<i32>>, canvas_size: Size2D<i32>) {

        if image_data_rect.size.width <= 0 || image_data_rect.size.height <= 0 {
            return
        }

        assert!(image_data_rect.size.width * image_data_rect.size.height * 4 == imagedata.len() as i32);
        // rgba -> bgra
        byte_swap(imagedata.as_mut_slice());

        let new_image_data_rect = Rect(Point2D(0i32, 0i32),
            Size2D(image_data_rect.size.width, image_data_rect.size.height));

        let new_dirty_rect = match dirty_rect {
            Some(mut dirty_rect) => {
                if dirty_rect.size.width < 0 {
                    dirty_rect.origin.x = dirty_rect.origin.x + dirty_rect.size.width;
                    dirty_rect.size.width = -dirty_rect.size.width;
                }
                if dirty_rect.size.height < 0 {
                    dirty_rect.origin.y = dirty_rect.origin.y + dirty_rect.size.height;
                    dirty_rect.size.height = -dirty_rect.size.height;
                }
                new_image_data_rect.intersection(&dirty_rect)
            },
            None => Some(new_image_data_rect)
        };

        if let Some(new_dirty_rect) = new_dirty_rect {
            let moved_dirty_rect = Rect(new_dirty_rect.origin.add(image_data_rect.origin),
                                        new_dirty_rect.size).intersection(&Rect(Point2D(0i32, 0i32),
                                        canvas_size)).unwrap_or(Rect::zero());
            if moved_dirty_rect.is_empty() {
                return
            }

            let source_surface = self.drawtarget.create_source_surface_from_data(imagedata.as_slice(),
                image_data_rect.size, image_data_rect.size.width * 4, SurfaceFormat::B8G8R8A8);

            let draw_surface_options = DrawSurfaceOptions::new(Filter::Linear, true);
            let draw_options = DrawOptions::new(1.0f64 as AzFloat, 0);

            self.drawtarget.draw_surface(source_surface,
                Rect(Point2D(moved_dirty_rect.origin.x as AzFloat, moved_dirty_rect.origin.y as AzFloat),
                     Size2D(moved_dirty_rect.size.width as AzFloat, moved_dirty_rect.size.height as AzFloat)),
                Rect(Point2D((moved_dirty_rect.origin.x - image_data_rect.origin.x) as AzFloat,
                             (moved_dirty_rect.origin.y - image_data_rect.origin.y) as AzFloat),
                     Size2D(moved_dirty_rect.size.width as AzFloat, moved_dirty_rect.size.height as AzFloat)),
                draw_surface_options, draw_options);
        }
    }
}
