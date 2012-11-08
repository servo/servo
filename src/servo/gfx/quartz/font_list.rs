extern mod core_foundation;
extern mod core_text;

use cf = core_foundation;
use cf::array::CFArray;
use ct = core_text;
use ct::font_collection::CTFontCollection;
use ct::font_descriptor::{
    CTFontDescriptor,
    CTFontDescriptorRef,
    debug_descriptor,
};

use gfx::font::FontFamily;

use dvec::DVec;

pub struct QuartzFontListHandle {
    collection: CTFontCollection,
}

pub impl QuartzFontListHandle {
    static pub fn new(_fctx: &native::FontContextHandle) -> QuartzFontListHandle {
        QuartzFontListHandle { collection: CTFontCollection::new() }
    }

    fn get_available_families(_fctx: &native::FontContextHandle) -> ~[@FontFamily] {
        // TODO: make a hashtable from family name to DVec<FontEntry>
        let descriptors : CFArray<CTFontDescriptorRef, CTFontDescriptor>;
        descriptors = self.collection.get_descriptors();
        for descriptors.each |desc: &CTFontDescriptor| {
            debug!("%?", { debug_descriptor(desc); () });
            // TODO: for each descriptor, make a FontEntry.
            // TODO: append FontEntry to hashtable value
        }

        let families: DVec<@FontFamily> = DVec();
        // TODO: iterate over (key,val) pairs and create FontFamily instances

        return move dvec::unwrap(move families);
    }
}