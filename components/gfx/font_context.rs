/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use app_units::Au;
use fnv::FnvHasher;
use font::{Font, FontDescriptor, FontGroup, FontHandleMethods, FontRef};
use font_cache_thread::FontTemplateInfo;
use font_template::FontTemplateDescriptor;
use malloc_size_of::{MallocSizeOf, MallocSizeOfOps};
use platform::font::FontHandle;
pub use platform::font_context::FontContextHandle;
use servo_arc::Arc;
use servo_atoms::Atom;
use std::cell::RefCell;
use std::collections::HashMap;
use std::default::Default;
use std::hash::{BuildHasherDefault, Hash, Hasher};
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering, ATOMIC_USIZE_INIT};
use style::computed_values::font_variant_caps::T as FontVariantCaps;
use style::properties::style_structs::Font as FontStyleStruct;
use style::values::computed::font::SingleFontFamily;
use webrender_api;

static SMALL_CAPS_SCALE_FACTOR: f32 = 0.8;      // Matches FireFox (see gfxFont.h)

#[derive(Debug)]
struct FontCacheEntry {
    family: Atom,
    font: Option<FontRef>,
}

impl FontCacheEntry {
    fn matches(&self, descriptor: &FontDescriptor, family: &SingleFontFamily) -> bool {
        if self.family != *family.atom() {
            return false
        }

        if let Some(ref font) = self.font {
            (*font).borrow().descriptor == *descriptor
        } else {
            true
        }
    }
}

#[derive(Debug)]
struct FallbackFontCacheEntry {
    font: FontRef,
}

impl FallbackFontCacheEntry {
    fn matches(&self, descriptor: &FontDescriptor) -> bool {
        self.font.borrow().descriptor == *descriptor
    }
}

/// An epoch for the font context cache. The cache is flushed if the current epoch does not match
/// this one.
static FONT_CACHE_EPOCH: AtomicUsize = ATOMIC_USIZE_INIT;

pub trait FontSource {
    fn get_font_instance(&mut self, key: webrender_api::FontKey, size: Au) -> webrender_api::FontInstanceKey;

    fn find_font_template(
        &mut self,
        family: SingleFontFamily,
        desc: FontTemplateDescriptor
    ) -> Option<FontTemplateInfo>;

    fn last_resort_font_template(&mut self, desc: FontTemplateDescriptor) -> FontTemplateInfo;
}

/// The FontContext represents the per-thread/thread state necessary for
/// working with fonts. It is the public API used by the layout and
/// paint code. It talks directly to the font cache thread where
/// required.
#[derive(Debug)]
pub struct FontContext<S: FontSource> {
    platform_handle: FontContextHandle,
    font_source: S,

    // TODO: The font context holds a strong ref to the cached fonts
    // so they will never be released. Find out a good time to drop them.
    // See bug https://github.com/servo/servo/issues/3300
    //
    // GWTODO: Check on real pages if this is faster as Vec() or HashMap().
    font_cache: Vec<FontCacheEntry>,
    fallback_font_cache: Vec<FallbackFontCacheEntry>,

    font_group_cache:
        HashMap<FontGroupCacheKey, Rc<RefCell<FontGroup>>, BuildHasherDefault<FnvHasher>>,

    epoch: usize,
}

impl<S: FontSource> FontContext<S> {
    pub fn new(font_source: S) -> FontContext<S> {
        let handle = FontContextHandle::new();
        FontContext {
            platform_handle: handle,
            font_source,
            font_cache: vec!(),
            fallback_font_cache: vec!(),
            font_group_cache: HashMap::with_hasher(Default::default()),
            epoch: 0,
        }
    }

    /// Create a `Font` for use in layout calculations, from a `FontTemplateInfo` returned by the
    /// cache thread (which contains the underlying font data) and a `FontDescriptor` which
    /// contains the styling parameters.
    fn create_font(&mut self, info: FontTemplateInfo, descriptor: FontDescriptor) -> Result<Font, ()> {
        // TODO: (Bug #3463): Currently we only support fake small-caps
        // painting. We should also support true small-caps (where the
        // font supports it) in the future.
        let actual_pt_size = match descriptor.variant {
            FontVariantCaps::SmallCaps => descriptor.pt_size.scale_by(SMALL_CAPS_SCALE_FACTOR),
            FontVariantCaps::Normal => descriptor.pt_size,
        };

        let handle = FontHandle::new_from_template(&self.platform_handle,
                                                        info.font_template,
                                                        Some(actual_pt_size))?;

        let font_instance_key = self.font_source.get_font_instance(info.font_key, actual_pt_size);
        Ok(Font::new(handle, descriptor.to_owned(), actual_pt_size, font_instance_key))
    }

    fn expire_font_caches_if_necessary(&mut self) {
        let current_epoch = FONT_CACHE_EPOCH.load(Ordering::SeqCst);
        if current_epoch == self.epoch {
            return
        }

        self.font_cache.clear();
        self.fallback_font_cache.clear();
        self.font_group_cache.clear();
        self.epoch = current_epoch
    }

    /// Returns a `FontGroup` representing fonts which can be used for layout, given the `style`.
    /// Font groups are cached, so subsequent calls with the same `style` will return a reference
    /// to an existing `FontGroup`.
    pub fn font_group(&mut self, style: Arc<FontStyleStruct>) -> Rc<RefCell<FontGroup>> {
        self.expire_font_caches_if_necessary();

        let cache_key = FontGroupCacheKey {
            size: style.font_size.size(),
            style,
        };

        if let Some(ref font_group) = self.font_group_cache.get(&cache_key) {
            return (*font_group).clone()
        }

        let font_group = Rc::new(RefCell::new(FontGroup::new(&cache_key.style)));
        self.font_group_cache.insert(cache_key, font_group.clone());
        font_group
    }

    /// Returns a reference to an existing font cache entry matching `descriptor` and `family`, if
    /// there is one.
    fn font_cache_entry(&self, descriptor: &FontDescriptor, family: &SingleFontFamily) -> Option<&FontCacheEntry> {
        self.font_cache.iter()
            .find(|cache_entry| cache_entry.matches(&descriptor, &family))
    }

    /// Creates a new font cache entry matching `descriptor` and `family`.
    fn create_font_cache_entry(&mut self, descriptor: &FontDescriptor, family: &SingleFontFamily) -> FontCacheEntry {
        let font =
            self.font_source.find_font_template(family.clone(), descriptor.template_descriptor.clone())
                .and_then(|template_info|
                    self.create_font(template_info, descriptor.to_owned()).ok()
                )
                .map(|font| Rc::new(RefCell::new(font)));

        FontCacheEntry { family: family.atom().to_owned(), font }
    }

    /// Returns a font from `family` matching the `descriptor`. Fonts are cached, so repeated calls
    /// will return a reference to the same underlying `Font`.
    pub fn font(&mut self, descriptor: &FontDescriptor, family: &SingleFontFamily) -> Option<FontRef> {
        if let Some(entry) = self.font_cache_entry(descriptor, family) {
            return entry.font.clone()
        }

        let entry = self.create_font_cache_entry(descriptor, family);
        let font = entry.font.clone();
        self.font_cache.push(entry);
        font
    }

    /// Returns a reference to an existing fallback font cache entry matching `descriptor`, if
    /// there is one.
    fn fallback_font_cache_entry(&self, descriptor: &FontDescriptor) -> Option<&FallbackFontCacheEntry> {
        self.fallback_font_cache.iter()
            .find(|cache_entry| cache_entry.matches(descriptor))
    }

    /// Creates a new fallback font cache entry matching `descriptor`.
    fn create_fallback_font_cache_entry(&mut self, descriptor: &FontDescriptor) -> Option<FallbackFontCacheEntry> {
        let template_info = self.font_source.last_resort_font_template(descriptor.template_descriptor.clone());

        match self.create_font(template_info, descriptor.to_owned()) {
            Ok(font) =>
                Some(FallbackFontCacheEntry {
                    font: Rc::new(RefCell::new(font))
                }),

            Err(_) => {
                debug!("Failed to create fallback font!");
                None
            }
        }
    }

    /// Returns a fallback font matching the `descriptor`. Fonts are cached, so repeated calls will
    /// return a reference to the same underlying `Font`.
    pub fn fallback_font(&mut self, descriptor: &FontDescriptor) -> Option<FontRef> {
        if let Some(cached_entry) = self.fallback_font_cache_entry(descriptor) {
            return Some(cached_entry.font.clone())
        };

        if let Some(entry) = self.create_fallback_font_cache_entry(descriptor) {
            let font = entry.font.clone();
            self.fallback_font_cache.push(entry);
            Some(font)
        } else {
            None
        }
    }
}

impl<S: FontSource> MallocSizeOf for FontContext<S> {
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        // FIXME(njn): Measure other fields eventually.
        self.platform_handle.size_of(ops)
    }
}

#[derive(Debug)]
struct FontGroupCacheKey {
    style: Arc<FontStyleStruct>,
    size: Au,
}

impl PartialEq for FontGroupCacheKey {
    fn eq(&self, other: &FontGroupCacheKey) -> bool {
        self.style == other.style && self.size == other.size
    }
}

impl Eq for FontGroupCacheKey {}

impl Hash for FontGroupCacheKey {
    fn hash<H>(&self, hasher: &mut H) where H: Hasher {
        self.style.hash.hash(hasher)
    }
}

#[inline]
pub fn invalidate_font_caches() {
    FONT_CACHE_EPOCH.fetch_add(1, Ordering::SeqCst);
}
