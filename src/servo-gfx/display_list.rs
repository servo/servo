use color::{Color, rgb};
use geometry::Au;
use image::base::Image;
use render_context::RenderContext;
use text::SendableTextRun;
use util::range::Range;

use azure::azure_hl::DrawTarget;
use core::dvec::DVec;
use clone_arc = std::arc::clone;
use geom::Rect;
use geom::Point2D;
use std::arc::ARC;

struct DisplayItemData {
    bounds : Rect<Au>, // TODO: whose coordinate system should this use?
}

impl DisplayItemData {
    static pure fn new(bounds: &Rect<Au>) -> DisplayItemData {
        DisplayItemData { bounds: copy *bounds }
    }
}

pub enum DisplayItem {
    SolidColor(DisplayItemData, Color),
    // TODO: need to provide spacing data for text run.
    // (i.e, to support rendering of CSS 'word-spacing' and 'letter-spacing')
    // TODO: don't copy text runs, ever.
    Text(DisplayItemData, ~SendableTextRun, Range, Color),
    Image(DisplayItemData, ARC<~image::base::Image>),
    Border(DisplayItemData, Au, Color)
}

impl DisplayItem {
    pure fn d(&self) -> &self/DisplayItemData {
        match *self {
            SolidColor(ref d, _) => d,
            Text(ref d, _, _, _) => d,
            Image(ref d, _) => d,
            Border(ref d, _, _) => d
        }
    }
    
    fn draw_into_context(&self, ctx: &RenderContext) {
        match *self {
            SolidColor(_, color) => ctx.draw_solid_color(&self.d().bounds, color),
            Text(_, run, ref range, color) => {
                let new_run = @run.deserialize(ctx.font_ctx);
                let font = new_run.font;
                let origin = self.d().bounds.origin;
                let baseline_origin = Point2D(origin.x, origin.y + font.metrics.ascent);
                font.draw_text_into_context(ctx, new_run, range, baseline_origin, color);
            },
            Image(_, ref img) => {
                debug!("drawing image at %?", self.d().bounds);
                ctx.draw_image(self.d().bounds, clone_arc(img));
            }
            Border(_, width, color) => ctx.draw_border(&self.d().bounds, width, color),
        }

        debug!("%?", {
        ctx.draw_border(&self.d().bounds, Au::from_px(1), rgb(150, 150, 150));
        () });
    }

    static pure fn new_SolidColor(bounds: &Rect<Au>, color: Color) -> DisplayItem {
        SolidColor(DisplayItemData::new(bounds), color)
    }

    static pure fn new_Border(bounds: &Rect<Au>, width: Au, color: Color) -> DisplayItem {
        Border(DisplayItemData::new(bounds), width, color)
    }

    static pure fn new_Text(bounds: &Rect<Au>,
                            run: ~SendableTextRun,
                            range: Range,
                            color: Color) -> DisplayItem {
        Text(DisplayItemData::new(bounds), move run, move range, color)
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
