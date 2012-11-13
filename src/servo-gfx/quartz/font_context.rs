extern mod core_foundation;
extern mod core_graphics;
extern mod core_text;

use ct = core_text;
use ct::font::CTFont;

use gfx_font::{FontHandle, UsedFontStyle};
use font::QuartzFontHandle;
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

pub impl QuartzFontContextHandle : FontContextHandleMethods {
    fn create_font_from_identifier(name: ~str, style: UsedFontStyle) -> Result<FontHandle, ()> {
        let ctfont_result = CTFont::new_from_name(move name, style.pt_size);
        do result::chain(move ctfont_result) |ctfont| {
            QuartzFontHandle::new_from_CTFont(&self, move ctfont)
        }
    }
}