use azure::azure_hl::DrawTarget;
use gfx::geometry::*;
use geom::rect::Rect;
use image::base::Image;
use render_context::RenderContext;
use servo_text::text_run;
use text::text_run::SendableTextRun;
use util::range::Range;

use std::arc::ARC;
use clone_arc = std::arc::clone;
use dvec::DVec;

pub use layout::display_list_builder::DisplayListBuilder;

// TODO: invert this so common data is nested inside each variant as first arg.
struct DisplayItem {
    draw: ~fn((&DisplayItem), (&RenderContext)),
    bounds : Rect<au>, // TODO: whose coordinate system should this use?
    data : DisplayItemData
}

pub enum DisplayItemData {
    SolidColorData(u8, u8, u8),
    // TODO: need to provide spacing data for text run.
    // (i.e, to support rendering of CSS 'word-spacing' and 'letter-spacing')
    // TODO: don't copy text runs, ever.
    TextData(~SendableTextRun, Range),
    ImageData(ARC<~image::base::Image>),
    BorderData(au, u8, u8, u8)
}

fn draw_SolidColor(self: &DisplayItem, ctx: &RenderContext) {
    match self.data {
        SolidColorData(r,g,b) => ctx.draw_solid_color(&self.bounds, r, g, b),
        _ => fail
    }        
}

fn draw_Text(self: &DisplayItem, ctx: &RenderContext) {
    match self.data {
        TextData(run, range) => {
            let new_run = text_run::deserialize(ctx.font_cache, run);
            ctx.draw_text(self.bounds, new_run, range)
        },
        _ => fail
    }        
}

fn draw_Image(self: &DisplayItem, ctx: &RenderContext) {
    match self.data {
        ImageData(ref img) => ctx.draw_image(self.bounds, clone_arc(img)),
        _ => fail
    }        
}

fn draw_Border(self: &DisplayItem, ctx: &RenderContext) {
    match self.data {
        BorderData(width, r, g, b) => ctx.draw_border(&self.bounds, width, r, g, b),
        _ => fail
    }
}

pub fn SolidColor(bounds: Rect<au>, r: u8, g: u8, b: u8) -> DisplayItem {
    DisplayItem { 
        draw: |self, ctx| draw_SolidColor(self, ctx),
        bounds: bounds,
        data: SolidColorData(r, g, b)
    }
}

pub fn Border(bounds: Rect<au>, width: au, r: u8, g: u8, b: u8) -> DisplayItem {
    DisplayItem {
        draw: |self, ctx| draw_Border(self, ctx),
        bounds: bounds,
        data: BorderData(width, r, g, b)
    }
}

pub fn Text(bounds: Rect<au>, run: ~SendableTextRun, range: Range) -> DisplayItem {
    DisplayItem {
        draw: |self, ctx| draw_Text(self, ctx),
        bounds: bounds,
        data: TextData(run, range)
    }
}

// ARC should be cloned into ImageData, but Images are not sendable
pub fn Image(bounds: Rect<au>, image: ARC<~image::base::Image>) -> DisplayItem {
    DisplayItem {
        draw: |self, ctx| draw_Image(self, ctx),
        bounds: bounds,
        data: ImageData(image)
    }
}

pub type DisplayList = DVec<~DisplayItem>;

trait DisplayListMethods {
    fn draw(ctx: &RenderContext);
}

impl DisplayList : DisplayListMethods {
    fn append_item(item: ~DisplayItem) {
        debug!("Adding display item %u: %?", self.len(), item);
        self.push(move item);
    }

    fn draw(ctx: &RenderContext) {
        debug!("beginning display list");
        for self.each |item| {
            debug!("drawing %?", *item);
            item.draw(*item, ctx);
        }
        debug!("ending display list");
    }
}
