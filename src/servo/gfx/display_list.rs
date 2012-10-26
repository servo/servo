use azure::azure_hl::DrawTarget;
use au = gfx::geometry;
use au::Au;
use geom::rect::Rect;
use geom::point::Point2D;
use image::base::Image;
use render_context::RenderContext;
use servo_text::text_run;
use text::text_run::SendableTextRun;
use util::range::Range;

use std::arc::ARC;
use clone_arc = std::arc::clone;
use dvec::DVec;

pub use layout::display_list_builder::DisplayListBuilder;

struct DisplayItemData {
    bounds : Rect<Au>, // TODO: whose coordinate system should this use?
}

impl DisplayItemData {
    static pure fn new(bounds: &Rect<Au>) -> DisplayItemData {
        DisplayItemData { bounds: copy *bounds }
    }
}

pub enum DisplayItem {
    SolidColor(DisplayItemData, u8, u8, u8),
    // TODO: need to provide spacing data for text run.
    // (i.e, to support rendering of CSS 'word-spacing' and 'letter-spacing')
    // TODO: don't copy text runs, ever.
    Text(DisplayItemData, ~SendableTextRun, Range),
    Image(DisplayItemData, ARC<~image::base::Image>),
    Border(DisplayItemData, Au, u8, u8, u8)
}

impl DisplayItem {
    pure fn d(&self) -> &self/DisplayItemData {
        match *self {
            SolidColor(ref d, _, _, _) => d,
            Text(ref d, _, _) => d,
            Image(ref d, _) => d,
            Border(ref d, _, _, _, _) => d
        }
    }
    
    fn draw_into_context(&self, ctx: &RenderContext) {
        match *self {
            SolidColor(_, r,g,b) => ctx.draw_solid_color(&self.d().bounds, r, g, b),
            Text(_, run, range) => {
                let new_run = @run.deserialize(ctx.font_cache);
                let font = new_run.font;
                let origin = self.d().bounds.origin;
                let baseline_origin = Point2D(origin.x, origin.y + font.metrics.ascent);
                font.draw_text_into_context(ctx, new_run, range, baseline_origin);
            },
            Image(_, ref img) => ctx.draw_image(self.d().bounds, clone_arc(img)),
            Border(_, width, r, g, b) => ctx.draw_border(&self.d().bounds, width, r, g, b),
        }

        debug!("%?", {
        ctx.draw_border(&self.d().bounds, au::from_px(1), 150, 150, 150);
        () });
    }

    static pure fn new_SolidColor(bounds: &Rect<Au>, r: u8, g: u8, b: u8) -> DisplayItem {
        SolidColor(DisplayItemData::new(bounds), r, g, b)
    }

    static pure fn new_Border(bounds: &Rect<Au>, width: Au, r: u8, g: u8, b: u8) -> DisplayItem {
        Border(DisplayItemData::new(bounds), width, r, g, b)
    }

    static pure fn new_Text(bounds: &Rect<Au>, run: ~SendableTextRun, range: Range) -> DisplayItem {
        Text(DisplayItemData::new(bounds), move run, range)
    }

    // ARC should be cloned into ImageData, but Images are not sendable
    static pure fn new_Image(bounds: &Rect<Au>, image: ARC<~image::base::Image>) -> DisplayItem {
        Image(DisplayItemData::new(bounds), move image)
    }
}

// Dual-mode/freezable.
pub struct DisplayList {
    list: ~[~DisplayItem]
}

trait DisplayListMethods {
    fn append_item(&mut self, item: ~DisplayItem);
    fn draw_into_context(ctx: &RenderContext);
}

impl DisplayList {
    static fn new() -> DisplayList {
        DisplayList { list: ~[] }
    }
}

impl DisplayList : DisplayListMethods {
    fn append_item(&mut self, item: ~DisplayItem) {
        // FIXME(Issue #150): crashes
        //debug!("Adding display item %u: %?", self.len(), item);
        self.list.push(move item);
    }

    fn draw_into_context(ctx: &RenderContext) {
        debug!("beginning display list");
        for self.list.each |item| {
            // FIXME(Issue #150): crashes
            //debug!("drawing %?", *item);
            item.draw_into_context(ctx);
        }
        debug!("ending display list");
    }
}
