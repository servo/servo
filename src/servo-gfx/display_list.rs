/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use color::{Color, rgb};
use geometry::Au;
use render_context::RenderContext;
use text::SendableTextRun;

use clone_arc = std::arc::clone;
use geom::Rect;
use geom::Point2D;
use std::arc::ARC;
use servo_net::image::base::Image;
use servo_util::range::Range;

struct DisplayItemData {
    bounds : Rect<Au>, // TODO: whose coordinate system should this use?
}

pub impl DisplayItemData {
    fn new(bounds: &Rect<Au>) -> DisplayItemData {
        DisplayItemData { bounds: copy *bounds }
    }
}

pub enum DisplayItem {
    SolidColor(DisplayItemData, Color),
    // TODO: need to provide spacing data for text run.
    // (i.e, to support rendering of CSS 'word-spacing' and 'letter-spacing')
    // TODO: don't copy text runs, ever.
    Text(DisplayItemData, ~SendableTextRun, Range, Color),
    Image(DisplayItemData, ARC<~Image>),
    Border(DisplayItemData, Au, Color)
}

pub impl<'self> DisplayItem {
    fn d(&'self self) -> &'self DisplayItemData {
        match *self {
            SolidColor(ref d, _) => d,
            Text(ref d, _, _, _) => d,
            Image(ref d, _) => d,
            Border(ref d, _, _) => d
        }
    }
    
    fn draw_into_context(&self, ctx: &RenderContext) {
        match self {
            &SolidColor(_, color) => {
                ctx.draw_solid_color(&self.d().bounds, color)
            }
            &Text(_, ref run, ref range, color) => {
                debug!("drawing text at %?", self.d().bounds);
                let new_run = @run.deserialize(ctx.font_ctx);
                let font = new_run.font;
                let origin = self.d().bounds.origin;
                let baseline_origin = Point2D(origin.x, origin.y + font.metrics.ascent);
                font.draw_text_into_context(ctx, new_run, range, baseline_origin, color);
            },
            &Image(_, ref img) => {
                debug!("drawing image at %?", self.d().bounds);
                ctx.draw_image(self.d().bounds, clone_arc(img));
            }
            &Border(_, width, color) => {
                ctx.draw_border(&self.d().bounds, width, color)
            }
        }

        debug!("%?", {
        ctx.draw_border(&self.d().bounds, Au::from_px(1), rgb(150, 150, 150));
        () });
    }

    fn new_SolidColor(bounds: &Rect<Au>, color: Color) -> DisplayItem {
        SolidColor(DisplayItemData::new(bounds), color)
    }

    fn new_Border(bounds: &Rect<Au>, width: Au, color: Color) -> DisplayItem {
        Border(DisplayItemData::new(bounds), width, color)
    }

    fn new_Text(bounds: &Rect<Au>,
                run: ~SendableTextRun,
                range: Range,
                color: Color) -> DisplayItem {
        Text(DisplayItemData::new(bounds), run, range, color)
    }

    // ARC should be cloned into ImageData, but Images are not sendable
    fn new_Image(bounds: &Rect<Au>, image: ARC<~Image>) -> DisplayItem {
        Image(DisplayItemData::new(bounds), image)
    }
}

// Dual-mode/freezable.
pub struct DisplayList {
    list: ~[~DisplayItem]
}

pub impl DisplayList {
    fn new() -> DisplayList {
        DisplayList { list: ~[] }
    }

    fn append_item(&mut self, item: ~DisplayItem) {
        // FIXME(Issue #150): crashes
        //debug!("Adding display item %u: %?", self.len(), item);
        self.list.push(item);
    }

    fn draw_into_context(&self, ctx: &RenderContext) {
        debug!("beginning display list");
        for self.list.each |item| {
            // FIXME(Issue #150): crashes
            //debug!("drawing %?", *item);
            item.draw_into_context(ctx);
        }
        debug!("ending display list");
    }
}
