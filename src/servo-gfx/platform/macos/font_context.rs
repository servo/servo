extern mod core_foundation;
extern mod core_graphics;
extern mod core_text;

use gfx_font::UsedFontStyle;
use gfx_font_context::FontContextHandleMethods;

use platform::macos::font::FontHandle;
use platform;

pub struct FontContextHandle {
    ctx: ()
}

pub impl FontContextHandle {
    // this is a placeholder until NSFontManager or whatever is bound in here.
    pub fn new() -> FontContextHandle {
        FontContextHandle { ctx: () }
    }
}

impl FontContextHandleMethods for FontContextHandle {
    fn clone(&self) -> FontContextHandle {
        FontContextHandle {
            ctx: self.ctx
        }
    }

    fn create_font_from_identifier(&self,
                                   name: ~str,
                                   style: UsedFontStyle)
                                -> Result<FontHandle, ()> {
        let ctfont_result = platform::macos::font_context::core_text::font::new_from_name(
            name,
            style.pt_size);
        do result::chain(ctfont_result) |ctfont| {
            FontHandle::new_from_CTFont(self, ctfont)
        }
    }
}
