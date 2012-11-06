use geom::rect::Rect;
use gfx::{
    Au,
    FontCache,
};
use resource::local_image_cache::LocalImageCache;
use std::net::url::Url;

/* Represents layout task context. */

struct LayoutContext {
    font_cache: @FontCache,
    image_cache: @LocalImageCache,
    doc_url: Url,
    screen_size: Rect<Au>
}
