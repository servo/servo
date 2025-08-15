mod harfbuzz;
pub use harfbuzz::{ShapedGlyphData, ShapedGlyphEntry, Shaper};

fn unicode_to_hb_script(script: unicode_script::Script) -> harfbuzz_sys::hb_script_t {
    let bytes: [u8; 4] = match script {
        unicode_script::Script::Unknown => *b"Zzzz",
        _ => {
            let short_name = script.short_name();
            short_name.as_bytes().try_into().unwrap()
        },
    };

    u32::from_be_bytes(bytes) as core::ffi::c_uint
}
