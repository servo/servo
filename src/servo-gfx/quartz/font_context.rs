extern mod core_foundation;
extern mod core_graphics;
extern mod core_text;

use quartz;
use quartz::font::QuartzFontHandle;
use quartz::font_context::core_text::font::CTFont;

use gfx_font::{FontHandle, UsedFontStyle};
use gfx_font_context::FontContextHandleMethods;

pub struct QuartzFontContextHandle {
    ctx: (),

    drop { }
}

pub impl QuartzFontContextHandle {
    // this is a placeholder until NSFontManager or whatever is bound in here.
    static pub fn new() -> QuartzFontContextHandle {
        QuartzFontContextHandle { ctx: () }
    }
}

pub impl FontContextHandleMethods for QuartzFontContextHandle {
    pure fn clone(&const self) -> QuartzFontContextHandle {
        QuartzFontContextHandle { ctx: self.ctx }
    }

    fn create_font_from_identifier(name: ~str, style: UsedFontStyle) -> Result<FontHandle, ()> {
        let ctfont_result = quartz::font_context::core_text::font::new_from_name(name,
                                                                                 style.pt_size);
        do result::chain(ctfont_result) |ctfont| {
            QuartzFontHandle::new_from_CTFont(&self, ctfont)
        }
    }
}
