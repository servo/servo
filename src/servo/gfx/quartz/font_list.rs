extern mod core_foundation;
extern mod core_text;

use cf = core_foundation;
use cf::array::CFArray;
use ct = core_text;
use ct::font::{
    CTFont,
    debug_font_names,
    debug_font_traits,
};
use ct::font_collection::CTFontCollection;
use ct::font_descriptor::{
    CTFontDescriptor,
    CTFontDescriptorRef,
    debug_descriptor,
};

use gfx::font::{
    FontEntry,
    FontFamily,
    FontHandle,
};

use font::{QuartzFontHandle};

use dvec::DVec;
use send_map::{linear, SendMap};

pub struct QuartzFontListHandle {
    collection: CTFontCollection,
}

pub impl QuartzFontListHandle {
    static pub fn new(_fctx: &native::FontContextHandle) -> QuartzFontListHandle {
        QuartzFontListHandle { collection: CTFontCollection::new() }
    }

    fn get_available_families(&const self,fctx: &native::FontContextHandle) -> linear::LinearMap<~str, @FontFamily> {
        // since we mutate it repeatedly, must be mut variable.
        let mut family_map : linear::LinearMap<~str, @FontFamily> = linear::LinearMap();
        let descriptors : CFArray<CTFontDescriptorRef, CTFontDescriptor>;
        descriptors = self.collection.get_descriptors();
        for descriptors.each |desc: &CTFontDescriptor| {
            // TODO: for each descriptor, make a FontEntry.
            let font = CTFont::new_from_descriptor(desc, 0.0);
            let handle = result::unwrap(QuartzFontHandle::new_from_CTFont(fctx, move font));
            let family_name = handle.family_name();
            debug!("Looking for family name: %s", family_name);
            let family = match family_map.find(&family_name) {
                Some(fam) => fam,
                None => {
                    debug!("Creating new FontFamily for family: %s", family_name);
                    let new_family = @FontFamily::new(family_name);
                    family_map.insert(move family_name, new_family);
                    new_family
                }
            };

            debug!("Creating new FontEntry for face: %s", handle.face_name());
            let entry = @FontEntry::new(family, move handle);
            family.entries.push(entry);
            // TODO: append FontEntry to hashtable value
        }
        return move family_map;
    }
}
