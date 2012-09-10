import gfx::geometry::*;
import geom::rect::Rect;
import image::base::Image;
import servo_text::text_run::TextRun;

import std::arc::ARC;
import dvec::DVec;

// TODO: convert to DisplayItem trait with methods like bounds(), paint(), etc.
enum ItemKind {
    SolidColor(u8, u8, u8),
    Image(ARC<~Image>),
    Text(TextRun),
    // FIXME: Shape code does not understand the alignment without this
    Padding(u8, u8, u8, u8)
}

struct DisplayItem {
    item: ItemKind,
    bounds: Rect<au>
}

impl DisplayItem : Copy { }

type DisplayList = DVec<DisplayItem>;
