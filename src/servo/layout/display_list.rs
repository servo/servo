use azure::azure_hl::DrawTarget;
use gfx::render_task::{draw_solid_color, draw_image, draw_text};
use gfx::geometry::*;
use geom::rect::Rect;
use image::base::Image;
use servo_text::text_run::TextRun;

use std::arc::{ARC, clone};
use dvec::DVec;

struct DisplayItem {
    draw: ~fn((&DisplayItem), (&DrawTarget)),
    bounds : Rect<au>, // TODO: whose coordinate system should this use?
    data : DisplayItemData
}

enum DisplayItemData {
    SolidColorData(u8, u8, u8),
    TextData(TextRun),
    ImageData(ARC<~image::base::Image>),
    PaddingData(u8, u8, u8, u8) // This is a hack to make fonts work (?)
}

fn draw_SolidColor(self: &DisplayItem, ctx: &DrawTarget) {
    match self.data {
        SolidColorData(r,g,b) => draw_solid_color(ctx, &self.bounds, r, g, b),
        _ => fail
    }        
}

fn draw_Text(self: &DisplayItem, ctx: &DrawTarget) {
    match self.data {
        TextData(run) => draw_text(ctx, self.bounds, &run),
        _ => fail
    }        
}

fn draw_Image(self: &DisplayItem, ctx: &DrawTarget) {
    match self.data {
        ImageData(img) => draw_image(ctx, self.bounds, img),
        _ => fail
    }        
}

fn SolidColor(bounds: Rect<au>, r: u8, g: u8, b: u8) -> DisplayItem {
    DisplayItem { 
        // TODO: this seems wrong.
        draw: |self, ctx| draw_SolidColor(self, ctx),
        bounds: bounds,
        data: SolidColorData(r, g, b)
    }
}

fn Text(bounds: Rect<au>, run: TextRun) -> DisplayItem {
    DisplayItem {
        draw: |self, ctx| draw_Text(self, ctx),
        bounds: bounds,
        data: TextData(run)
    }
}

// ARC should be cloned into ImageData, but Images are not sendable
fn Image(bounds: Rect<au>, image: ARC<~image::base::Image>) -> DisplayItem {
    DisplayItem {
        // TODO: this seems wrong.
        draw: |self, ctx| draw_Image(self, ctx),
        bounds: bounds,
        data: ImageData(clone(&image))
    }
}

type DisplayList = DVec<~DisplayItem>;

trait DisplayListMethods {
    fn draw(ctx: &DrawTarget);
}

impl DisplayList : DisplayListMethods {
    fn draw(ctx: &DrawTarget) {
        for self.each |item| {
            #debug["drawing %?", item];
            item.draw(item, ctx);
        }
    }
}