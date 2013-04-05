extern mod core_foundation;
extern mod core_text;

use gfx_font::FontHandleMethods;
use gfx_font_context::FontContextHandleMethods;
use gfx_font_list::{FontEntry, FontFamily, FontFamilyMap};

use native;
use platform::macos::font::QuartzFontHandle;
use platform::macos::font_context::QuartzFontContextHandle;
use platform::macos::font_list::core_foundation::array::CFArray;
use platform::macos::font_list::core_foundation::base::CFWrapper;
use platform::macos::font_list::core_foundation::string::{CFString, CFStringRef};
use platform::macos::font_list::core_text::font_collection::CTFontCollectionMethods;
use platform::macos::font_list::core_text::font_descriptor::CTFontDescriptorRef;
use platform;

use core::hashmap::HashMap;

pub struct QuartzFontListHandle {
    fctx: QuartzFontContextHandle,
}

pub impl QuartzFontListHandle {
    fn new(fctx: &native::FontContextHandle) -> QuartzFontListHandle {
        QuartzFontListHandle { fctx: fctx.clone() }
    }

    fn get_available_families(&self) -> FontFamilyMap {
        let family_names: CFArray<CFStringRef> =
            platform::macos::font_list::core_text::font_collection::get_family_names();
        let mut family_map : FontFamilyMap = HashMap::new();
        for family_names.each |&strref: &CFStringRef| {
            let family_name = CFString::wrap_extern(strref).to_str();
            debug!("Creating new FontFamily for family: %s", family_name);

            let new_family = @mut FontFamily::new(family_name);
            family_map.insert(family_name, new_family);
        }
        return family_map;
    }

    fn load_variations_for_family(&self, family: @mut FontFamily) {
        let fam : &mut FontFamily = family; // FIXME: borrow checker workaround
        let family_name = &fam.family_name;
        debug!("Looking for faces of family: %s", *family_name);

        let family_collection =
            platform::macos::font_list::core_text::font_collection::create_for_family(
                *family_name);
        for family_collection.get_descriptors().each |descref: &CTFontDescriptorRef| {
            let desc = CFWrapper::wrap_shared(*descref);
            let font = platform::macos::font_list::core_text::font::new_from_descriptor(&desc,
                                                                                        0.0);
            let handle = result::unwrap(QuartzFontHandle::new_from_CTFont(&self.fctx, font));

            debug!("Creating new FontEntry for face: %s", handle.face_name());
            let entry = @FontEntry::new(family, handle);
            family.entries.push(entry);
        }
    }
}
