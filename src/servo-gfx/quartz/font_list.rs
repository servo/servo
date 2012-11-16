extern mod core_foundation;
extern mod core_text;

use cf = core_foundation;
use cf::array::CFArray;
use cf::base::CFWrapper;
use cf::string::{CFString, CFStringRef};

use ct = core_text;
use ct::font::{CTFont, debug_font_names, debug_font_traits};
use ct::font_collection::CTFontCollection;
use ct::font_descriptor::{CTFontDescriptor, CTFontDescriptorRef, debug_descriptor};

use quartz::font::QuartzFontHandle;
use quartz::font_context::QuartzFontContextHandle;
use gfx_font::FontHandle;
use gfx_font_list::{FontEntry, FontFamily, FontFamilyMap};

use core::dvec::DVec;
use core::send_map::{linear, SendMap};

pub struct QuartzFontListHandle {
    fctx: QuartzFontContextHandle,
}

pub impl QuartzFontListHandle {
    static fn new(fctx: &native::FontContextHandle) -> QuartzFontListHandle {
        QuartzFontListHandle { fctx: fctx.clone() }
    }

    fn get_available_families() -> FontFamilyMap {
        let family_names = ct::font_collection::get_family_names();
        let mut family_map : FontFamilyMap = linear::LinearMap();
        for family_names.each |strref: &CFStringRef| {
            let family_name = CFWrapper::wrap_shared(*strref).to_str();
            debug!("Creating new FontFamily for family: %s", family_name);

            let new_family = @FontFamily::new(family_name);
            family_map.insert(move family_name, new_family);
        }
        return move family_map;
    }

    fn load_variations_for_family(family: @FontFamily) {
        let family_name = &family.family_name;
        debug!("Looking for faces of family: %s", *family_name);

        let family_collection = ct::font_collection::create_for_family(*family_name);
        for family_collection.get_descriptors().each |descref: &CTFontDescriptorRef| {
            let desc = CFWrapper::wrap_shared(*descref);
            let font = ct::font::new_from_descriptor(&desc, 0.0);
            let handle = result::unwrap(QuartzFontHandle::new_from_CTFont(&self.fctx, move font));

            debug!("Creating new FontEntry for face: %s", handle.face_name());
            let entry = @FontEntry::new(family, move handle);
            family.entries.push(entry);
        }
    }
}
