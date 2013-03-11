use geom::rect::Rect;
use gfx::font_context::FontContext;
use gfx::geometry::Au;
use gfx::resource::local_image_cache::LocalImageCache;
use std::net::url::Url;

/* Represents layout task context. */

pub struct LayoutContext {
    font_ctx: @mut FontContext,
    image_cache: @mut LocalImageCache,
    doc_url: Url,
    screen_size: Rect<Au>
}
