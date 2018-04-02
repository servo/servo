/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use app_units::Au;
use fnv::FnvHasher;
use font::{Font, FontDescriptor, FontFamilyDescriptor, FontGroup, FontHandleMethods, FontRef};
use font_cache_thread::FontTemplateInfo;
use font_template::FontTemplateDescriptor;
use malloc_size_of::{MallocSizeOf, MallocSizeOfOps};
use platform::font::FontHandle;
pub use platform::font_context::FontContextHandle;
use servo_arc::Arc;
use std::cell::RefCell;
use std::collections::HashMap;
use std::default::Default;
use std::hash::{BuildHasherDefault, Hash, Hasher};
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering, ATOMIC_USIZE_INIT};
use style::computed_values::font_variant_caps::T as FontVariantCaps;
use style::properties::style_structs::Font as FontStyleStruct;
use webrender_api;

static SMALL_CAPS_SCALE_FACTOR: f32 = 0.8;      // Matches FireFox (see gfxFont.h)

/// An epoch for the font context cache. The cache is flushed if the current epoch does not match
/// this one.
static FONT_CACHE_EPOCH: AtomicUsize = ATOMIC_USIZE_INIT;

pub trait FontSource {
    fn get_font_instance(&mut self, key: webrender_api::FontKey, size: Au) -> webrender_api::FontInstanceKey;

    fn font_template(
        &mut self,
        template_descriptor: FontTemplateDescriptor,
        family_descriptor: FontFamilyDescriptor,
    ) -> Option<FontTemplateInfo>;
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
    font_cache: HashMap<FontCacheKey, Option<FontRef>>,
    font_template_cache: HashMap<FontTemplateCacheKey, Option<FontTemplateInfo>>,

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
            font_cache: HashMap::new(),
            font_template_cache: HashMap::new(),
            font_group_cache: HashMap::with_hasher(Default::default()),
            epoch: 0,
        }
    }

    fn expire_font_caches_if_necessary(&mut self) {
        let current_epoch = FONT_CACHE_EPOCH.load(Ordering::SeqCst);
        if current_epoch == self.epoch {
            return
        }

        self.font_cache.clear();
        self.font_template_cache.clear();
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

    /// Returns a font matching the parameters. Fonts are cached, so repeated calls will return a
    /// reference to the same underlying `Font`.
    pub fn font(
        &mut self,
        font_descriptor: &FontDescriptor,
        family_descriptor: &FontFamilyDescriptor,
    ) -> Option<FontRef>
    {
        let cache_key = FontCacheKey {
            font_descriptor: font_descriptor.clone(),
            family_descriptor: family_descriptor.clone(),
        };

        self.font_cache.get(&cache_key).map(|v| v.clone()).unwrap_or_else(|| {
            debug!(
                "FontContext::font cache miss for font_descriptor={:?} family_descriptor={:?}",
                font_descriptor,
                family_descriptor
            );

            let font =
                self.font_template(&font_descriptor.template_descriptor, family_descriptor)
                    .and_then(|template_info| self.create_font(template_info, font_descriptor.to_owned()).ok())
                    .map(|font| Rc::new(RefCell::new(font)));

            self.font_cache.insert(cache_key, font.clone());
            font
        })
    }

    fn font_template(
        &mut self,
        template_descriptor: &FontTemplateDescriptor,
        family_descriptor: &FontFamilyDescriptor
    ) -> Option<FontTemplateInfo>
    {
        let cache_key = FontTemplateCacheKey {
            template_descriptor: template_descriptor.clone(),
            family_descriptor: family_descriptor.clone(),
        };

        self.font_template_cache.get(&cache_key).map(|v| v.clone()).unwrap_or_else(|| {
            debug!(
                "FontContext::font_template cache miss for template_descriptor={:?} family_descriptor={:?}",
                template_descriptor,
                family_descriptor
            );

            let template_info = self.font_source.font_template(
                template_descriptor.clone(),
                family_descriptor.clone(),
            );

            self.font_template_cache.insert(cache_key, template_info.clone());
            template_info
        })
    }

    /// Create a `Font` for use in layout calculations, from a `FontTemplateData` returned by the
    /// cache thread and a `FontDescriptor` which contains the styling parameters.
    fn create_font(
        &mut self,
        info: FontTemplateInfo,
        descriptor: FontDescriptor
    ) -> Result<Font, ()>
    {
        // TODO: (Bug #3463): Currently we only support fake small-caps
        // painting. We should also support true small-caps (where the
        // font supports it) in the future.
        let actual_pt_size = match descriptor.variant {
            FontVariantCaps::SmallCaps => descriptor.pt_size.scale_by(SMALL_CAPS_SCALE_FACTOR),
            FontVariantCaps::Normal => descriptor.pt_size,
        };

        let handle = FontHandle::new_from_template(
            &self.platform_handle,
            info.font_template,
            Some(actual_pt_size)
        )?;

        let font_instance_key = self.font_source.get_font_instance(info.font_key, actual_pt_size);
        Ok(Font::new(handle, descriptor.to_owned(), actual_pt_size, font_instance_key))
    }
}

impl<S: FontSource> MallocSizeOf for FontContext<S> {
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        // FIXME(njn): Measure other fields eventually.
        self.platform_handle.size_of(ops)
    }
}

#[derive(Debug, Eq, Hash, PartialEq)]
struct FontCacheKey {
    font_descriptor: FontDescriptor,
    family_descriptor: FontFamilyDescriptor,
}

#[derive(Debug, Eq, Hash, PartialEq)]
struct FontTemplateCacheKey {
    template_descriptor: FontTemplateDescriptor,
    family_descriptor: FontFamilyDescriptor,
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
