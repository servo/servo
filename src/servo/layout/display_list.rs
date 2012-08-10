import gfx::geometry::*;
import geom::rect::Rect;
import image::base::Image;
import servo_text::text_run::TextRun;

import arc::arc;
import dvec::dvec;

enum item_type {
    display_item_solid_color(u8, u8, u8),
    display_item_image(~arc<~Image>),
    display_item_text(TextRun),
    // FIXME: Shape code does not understand the alignment without this
    padding(u8, u8, u8, u8)
}

enum display_item = {
    item_type: item_type,
    bounds: Rect<au>
};

type display_list = dvec<display_item>;
