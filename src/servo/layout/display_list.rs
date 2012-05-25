import gfx::geom::*;

enum item_type {
    display_item_solid_color(u8, u8, u8),
    // FIXME: Shape code does not understand the alignment without this
    padding(u8, u8, u8, u8)
}

enum display_item = {
    item_type: item_type,
    bounds: rect<au>
};

type display_list = [display_item];
