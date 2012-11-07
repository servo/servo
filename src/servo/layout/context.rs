use geom::rect::Rect;
use gfx::{
    Au,
    FontContext,
};
use resource::local_image_cache::LocalImageCache;
use std::net::url::Url;

/* Represents layout task context. */

struct LayoutContext {
    font_ctx: @FontContext,
    image_cache: @LocalImageCache,
    doc_url: Url,
    screen_size: Rect<Au>
}
