use font::{Font, FontDescriptor, FontGroup, FontStyle, SelectorPlatformIdentifier, SelectorStubDummy};
use font::{SpecifiedFontStyle, UsedFontStyle};
use font_list::FontList;
use native::FontHandle;
use util::cache;

use azure::azure_hl::BackendType;
use core::dvec::DVec;

// TODO(Issue #164): delete, and get default font from font list
const TEST_FONT: [u8 * 33004] = #include_bin("JosefinSans-SemiBold.ttf");

fn test_font_bin() -> ~[u8] {
    return vec::from_fn(33004, |i| TEST_FONT[i]);
}

// TODO(Rust #3934): creating lots of new dummy styles is a workaround
// for not being able to store symbolic enums in top-level constants.
pub fn dummy_style() -> FontStyle {
    use font::FontWeight300;
    return FontStyle {
        pt_size: 20f,
        weight: FontWeight300,
        italic: false,
        oblique: false,
        families: ~"Helvetica, serif",
    }
}

#[cfg(target_os = "macos")]
type FontContextHandle/& = quartz::font_context::QuartzFontContextHandle;

#[cfg(target_os = "linux")]
type FontContextHandle/& = freetype::font_context::FreeTypeFontContextHandle;

trait FontContextHandleMethods {
    fn create_font_from_identifier(~str, UsedFontStyle) -> Result<FontHandle, ()>;
}

// TODO(Issue #163): this is a workaround for static methods, traits,
// and typedefs not working well together. It should be removed.
pub impl FontContextHandle {
    #[cfg(target_os = "macos")]
    static pub fn new() -> FontContextHandle {
        quartz::font_context::QuartzFontContextHandle::new()
    }

    #[cfg(target_os = "linux")]
    static pub fn new() -> FontContextHandle {
        freetype::font_context::FreeTypeFontContextHandle::new()
    }
}

pub struct FontContext {
    instance_cache: cache::MonoCache<FontDescriptor, @Font>,
    font_list: Option<FontList>, // only needed by layout
    handle: FontContextHandle,
    backend: BackendType,
}

pub impl FontContext {
    static fn new(backend: BackendType, needs_font_list: bool) -> FontContext {
        let handle = FontContextHandle::new();
        let font_list = if needs_font_list { Some(FontList::new(&handle)) } else { None };
        FontContext { 
            // TODO(Rust #3902): remove extraneous type parameters once they are inferred correctly.
            instance_cache: cache::new::<FontDescriptor, @Font, cache::MonoCache<FontDescriptor, @Font>>(10),
            font_list: move font_list,
            handle: move handle,
            backend: backend
        }
    }

    priv pure fn get_font_list(&self) -> &self/FontList {
        option::get_ref(&self.font_list)
    }

    fn get_resolved_font_for_style(style: &SpecifiedFontStyle) -> @FontGroup {
        // TODO(Issue #178, E): implement a cache of FontGroup instances.
        self.create_font_group(style)
    }

    fn get_font_by_descriptor(desc: &FontDescriptor) -> Result<@Font, ()> {
        match self.instance_cache.find(desc) {
            Some(f) => Ok(f),
            None => { 
                let result = self.create_font_instance(desc);
                match result {
                    Ok(font) => {
                        self.instance_cache.insert(desc, font);
                    }, _ => {}
                };
                result
            }
        }
    }

    // TODO:(Issue #196): cache font groups on the font context.
    priv fn create_font_group(style: &SpecifiedFontStyle) -> @FontGroup {
        let fonts = DVec();

        // TODO(Issue #193): make iteration over 'font-family' more robust.
        for str::split_char_each(style.families, ',') |family| {
            let family_name = str::trim(family);
            let list = self.get_font_list();

            let result = list.find_font_in_family(family_name, style);
            do result.iter |font_entry| {
                // TODO(Issue #203): route this instantion through FontContext's Font instance cache.
                let instance = Font::new_from_existing_handle(&self, &font_entry.handle, style, self.backend);
                do result::iter(&instance) |font: &@Font| { fonts.push(*font); }
            };
        }

        // TODO(Issue #194): *always* attach a fallback font to the
        // font list, so that this assertion will never fail.

        // assert fonts.len() > 0;
        if fonts.len() == 0 {
            let desc = FontDescriptor::new(font_context::dummy_style(), SelectorStubDummy);
            match self.get_font_by_descriptor(&desc) {
                Ok(instance) => fonts.push(instance),
                Err(()) => {}
            }
        }
        assert fonts.len() > 0;
        // TODO(Issue #179): Split FontStyle into specified and used styles
        let used_style = copy *style;

        @FontGroup::new(style.families.to_managed(), &used_style, dvec::unwrap(move fonts))
    }

    priv fn create_font_instance(desc: &FontDescriptor) -> Result<@Font, ()> {
        return match desc.selector {
            SelectorStubDummy => {
                Font::new_from_buffer(&self, test_font_bin(), &desc.style, self.backend)
            },
            // TODO(Issue #174): implement by-platform-name font selectors.
            SelectorPlatformIdentifier(identifier) => { 
                let result_handle = self.handle.create_font_from_identifier(identifier, copy desc.style);
                result::chain(move result_handle, |handle| {
                    Ok(Font::new_from_adopted_handle(&self, move handle, &desc.style, self.backend))
                })
            }
        };
    }
}
