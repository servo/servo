import gfx::geom::*;

enum item_type {
    solid_color
}

enum display_item = {
    item_type: item_type,
    bounds: rect<au>
};

type display_list = [display_item];
