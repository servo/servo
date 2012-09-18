use resource::image_cache_task::ImageCacheTask;
use servo_text::font_cache::FontCache;
use std::net::url::Url;
use geom::rect::Rect;
use au = gfx::geometry::au;

/* Represents layout task context. */

struct LayoutContext {
    font_cache: @FontCache,
    image_cache: ImageCacheTask,
    doc_url: Url,
    screen_size: Rect<au>
}