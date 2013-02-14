extern mod core_foundation;
extern mod core_text;

use native;
use quartz;
use quartz::font_list::core_foundation::array::CFArray;
use quartz::font_list::core_foundation::base::CFWrapper;
use quartz::font_list::core_foundation::string::{CFString, CFStringRef};

use quartz::font_list::core_text::font::{CTFont, debug_font_names, debug_font_traits};
use quartz::font_list::core_text::font_collection::CTFontCollection;
use quartz::font_list::core_text::font_descriptor::{CTFontDescriptor, CTFontDescriptorRef};
use quartz::font_list::core_text::font_descriptor::{debug_descriptor};

use quartz::font::QuartzFontHandle;
use quartz::font_context::QuartzFontContextHandle;
use gfx_font::{FontHandle, FontHandleMethods};
use gfx_font_context::FontContextHandleMethods;
use gfx_font_list::{FontEntry, FontFamily, FontFamilyMap};

use core::dvec::DVec;
use core::hashmap::linear::LinearMap;

pub struct QuartzFontListHandle {
    fctx: QuartzFontContextHandle,
}

pub impl QuartzFontListHandle {
    static fn new(fctx: &native::FontContextHandle) -> QuartzFontListHandle {
        QuartzFontListHandle { fctx: fctx.clone() }
    }

    fn get_available_families() -> FontFamilyMap {
        let family_names: CFArray<CFStringRef> =
            quartz::font_list::core_text::font_collection::get_family_names();
        let mut family_map : FontFamilyMap = LinearMap::new();
        for family_names.each |strref: &CFStringRef| {
            /*let family_name = CFWrapper::wrap_shared(strref).to_str();
            debug!("Creating new FontFamily for family: %s", family_name);

            let new_family = @FontFamily::new(family_name);
            family_map.insert(move family_name, new_family);*/
        }
        return move family_map;
    }

    fn load_variations_for_family(family: @FontFamily) {
        let family_name = &family.family_name;
        debug!("Looking for faces of family: %s", *family_name);

        let family_collection =
            quartz::font_list::core_text::font_collection::create_for_family(*family_name);
        for family_collection.get_descriptors().each |descref: &CTFontDescriptorRef| {
            let desc = CFWrapper::wrap_shared(*descref);
            let font = quartz::font_list::core_text::font::new_from_descriptor(&desc, 0.0);
            let handle = result::unwrap(QuartzFontHandle::new_from_CTFont(&self.fctx, move font));

            debug!("Creating new FontEntry for face: %s", handle.face_name());
            let entry = @FontEntry::new(family, move handle);
            family.entries.push(entry);
        }
    }
}
