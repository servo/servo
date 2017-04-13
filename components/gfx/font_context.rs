/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use app_units::Au;
use fnv::FnvHasher;
use font::{Font, FontGroup, FontHandleMethods};
use font_cache_thread::FontCacheThread;
use font_template::FontTemplateDescriptor;
use heapsize::HeapSizeOf;
use platform::font::FontHandle;
use platform::font_context::FontContextHandle;
use platform::font_template::FontTemplateData;
use smallvec::SmallVec;
use std::cell::RefCell;
use std::collections::HashMap;
use std::default::Default;
use std::hash::{BuildHasherDefault, Hash, Hasher};
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering, ATOMIC_USIZE_INIT};
use style::computed_values::{font_style, font_variant_caps};
use style::properties::style_structs;
use webrender_traits;

static SMALL_CAPS_SCALE_FACTOR: f32 = 0.8;      // Matches FireFox (see gfxFont.h)

#[derive(Debug)]
struct LayoutFontCacheEntry {
    family: String,
    font: Option<Rc<RefCell<Font>>>,
}

#[derive(Debug)]
struct FallbackFontCacheEntry {
    font: Rc<RefCell<Font>>,
}

/// An epoch for the font context cache. The cache is flushed if the current epoch does not match
/// this one.
static FONT_CACHE_EPOCH: AtomicUsize = ATOMIC_USIZE_INIT;

/// The FontContext represents the per-thread/thread state necessary for
/// working with fonts. It is the public API used by the layout and
/// paint code. It talks directly to the font cache thread where
/// required.
#[derive(Debug)]
pub struct FontContext {
    platform_handle: FontContextHandle,
    font_cache_thread: FontCacheThread,

    /// TODO: See bug https://github.com/servo/servo/issues/3300.
    layout_font_cache: Vec<LayoutFontCacheEntry>,
    fallback_font_cache: Vec<FallbackFontCacheEntry>,

    layout_font_group_cache:
        HashMap<LayoutFontGroupCacheKey, Rc<FontGroup>, BuildHasherDefault<FnvHasher>>,

    epoch: usize,
}

impl FontContext {
    pub fn new(font_cache_thread: FontCacheThread) -> FontContext {
        let handle = FontContextHandle::new();
        FontContext {
            platform_handle: handle,
            font_cache_thread: font_cache_thread,
            layout_font_cache: vec!(),
            fallback_font_cache: vec!(),
            layout_font_group_cache: HashMap::with_hasher(Default::default()),
            epoch: 0,
        }
    }

    /// Create a font for use in layout calculations.
    fn create_layout_font(&self,
                          template: Arc<FontTemplateData>,
                          descriptor: FontTemplateDescriptor,
                          pt_size: Au,
                          variant: font_variant_caps::T,
                          font_key: webrender_traits::FontKey) -> Result<Font, ()> {
        // TODO: (Bug #3463): Currently we only support fake small-caps
        // painting. We should also support true small-caps (where the
        // font supports it) in the future.
        let actual_pt_size = match variant {
            font_variant_caps::T::small_caps => pt_size.scale_by(SMALL_CAPS_SCALE_FACTOR),
            font_variant_caps::T::normal => pt_size,
        };

        let handle = try!(FontHandle::new_from_template(&self.platform_handle,
                                                        template,
                                                        Some(actual_pt_size)));

        Ok(Font::new(handle, variant, descriptor, pt_size, actual_pt_size, font_key))
    }

    fn expire_font_caches_if_necessary(&mut self) {
        let current_epoch = FONT_CACHE_EPOCH.load(Ordering::SeqCst);
        if current_epoch == self.epoch {
            return
        }

        self.layout_font_cache.clear();
        self.fallback_font_cache.clear();
        self.layout_font_group_cache.clear();
        self.epoch = current_epoch
    }

    /// Create a group of fonts for use in layout calculations. May return
    /// a cached font if this font instance has already been used by
    /// this context.
    pub fn layout_font_group_for_style(&mut self, style: Arc<style_structs::Font>)
                                       -> Rc<FontGroup> {
        self.expire_font_caches_if_necessary();

        let layout_font_group_cache_key = LayoutFontGroupCacheKey {
            pointer: style.clone(),
            size: style.font_size,
        };
        if let Some(ref cached_font_group) = self.layout_font_group_cache.get(
                &layout_font_group_cache_key) {
            return (*cached_font_group).clone()
        }

        // TODO: The font context holds a strong ref to the cached fonts
        // so they will never be released. Find out a good time to drop them.

        let desc = FontTemplateDescriptor::new(style.font_weight,
                                               style.font_stretch,
                                               style.font_style == font_style::T::italic ||
                                                style.font_style == font_style::T::oblique);

        let mut fonts: SmallVec<[Rc<RefCell<Font>>; 8]> = SmallVec::new();

        for family in &style.font_family.0 {
            // GWTODO: Check on real pages if this is faster as Vec() or HashMap().
            let mut cache_hit = false;
            for cached_font_entry in &self.layout_font_cache {
                if cached_font_entry.family == family.name() {
                    match cached_font_entry.font {
                        None => {
                            cache_hit = true;
                            break;
                        }
                        Some(ref cached_font_ref) => {
                            let cached_font = (*cached_font_ref).borrow();
                            if cached_font.descriptor == desc &&
                               cached_font.requested_pt_size == style.font_size &&
                               cached_font.variant == style.font_variant_caps {
                                fonts.push((*cached_font_ref).clone());
                                cache_hit = true;
                                break;
                            }
                        }
                    }
                }
            }

            if !cache_hit {
                let template_info = self.font_cache_thread.find_font_template(family.clone(),
                                                                             desc.clone());
                match template_info {
                    Some(template_info) => {
                        let layout_font = self.create_layout_font(template_info.font_template,
                                                                  desc.clone(),
                                                                  style.font_size,
                                                                  style.font_variant_caps,
                                                                  template_info.font_key
                                                                               .expect("No font key present!"));
                        let font = match layout_font {
                            Ok(layout_font) => {
                                let layout_font = Rc::new(RefCell::new(layout_font));
                                fonts.push(layout_font.clone());

                                Some(layout_font)
                            }
                            Err(_) => None
                        };

                        self.layout_font_cache.push(LayoutFontCacheEntry {
                            family: family.name().to_owned(),
                            font: font
                        });
                    }
                    None => {
                        self.layout_font_cache.push(LayoutFontCacheEntry {
                            family: family.name().to_owned(),
                            font: None,
                        });
                    }
                }
            }
        }

        // Add a last resort font as a fallback option.
        let mut cache_hit = false;
        for cached_font_entry in &self.fallback_font_cache {
            let cached_font = cached_font_entry.font.borrow();
            if cached_font.descriptor == desc &&
                        cached_font.requested_pt_size == style.font_size &&
                        cached_font.variant == style.font_variant_caps {
                fonts.push(cached_font_entry.font.clone());
                cache_hit = true;
                break;
            }
        }

        if !cache_hit {
            let template_info = self.font_cache_thread.last_resort_font_template(desc.clone());
            let layout_font = self.create_layout_font(template_info.font_template,
                                                      desc.clone(),
                                                      style.font_size,
                                                      style.font_variant_caps,
                                                      template_info.font_key.expect("No font key present!"));
            match layout_font {
                Ok(layout_font) => {
                    let layout_font = Rc::new(RefCell::new(layout_font));
                    self.fallback_font_cache.push(FallbackFontCacheEntry {
                        font: layout_font.clone(),
                    });
                    fonts.push(layout_font);
                }
                Err(_) => debug!("Failed to create fallback layout font!")
            }
        }

        let font_group = Rc::new(FontGroup::new(fonts));
        self.layout_font_group_cache.insert(layout_font_group_cache_key, font_group.clone());
        font_group
    }
}

impl HeapSizeOf for FontContext {
    fn heap_size_of_children(&self) -> usize {
        // FIXME(njn): Measure other fields eventually.
        self.platform_handle.heap_size_of_children()
    }
}

#[derive(Debug)]
struct LayoutFontGroupCacheKey {
    pointer: Arc<style_structs::Font>,
    size: Au,
}

impl PartialEq for LayoutFontGroupCacheKey {
    fn eq(&self, other: &LayoutFontGroupCacheKey) -> bool {
        self.pointer == other.pointer && self.size == other.size
    }
}

impl Eq for LayoutFontGroupCacheKey {}

impl Hash for LayoutFontGroupCacheKey {
    fn hash<H>(&self, hasher: &mut H) where H: Hasher {
        self.pointer.hash.hash(hasher)
    }
}

#[inline]
pub fn invalidate_font_caches() {
    FONT_CACHE_EPOCH.fetch_add(1, Ordering::SeqCst);
}
