/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;
use std::default::Default;
use std::hash::{BuildHasherDefault, Hash, Hasher};
use std::sync::Arc;

use app_units::Au;
use fnv::FnvHasher;
use log::debug;
use parking_lot::{Mutex, RwLock};
use servo_arc::Arc as ServoArc;
use style::computed_values::font_variant_caps::T as FontVariantCaps;
use style::properties::style_structs::Font as FontStyleStruct;
use webrender_api::{FontInstanceFlags, FontInstanceKey};

use crate::font::{Font, FontDescriptor, FontFamilyDescriptor, FontGroup, FontRef};
use crate::font_cache_thread::FontIdentifier;
use crate::font_template::{FontTemplateRef, FontTemplateRefMethods};
#[cfg(target_os = "macos")]
use crate::platform::core_text_font_cache::CoreTextFontCache;

static SMALL_CAPS_SCALE_FACTOR: f32 = 0.8; // Matches FireFox (see gfxFont.h)

pub trait FontSource {
    fn get_font_instance(
        &mut self,
        font_identifier: FontIdentifier,
        size: Au,
        flags: FontInstanceFlags,
    ) -> FontInstanceKey;
    fn find_matching_font_templates(
        &mut self,
        descriptor_to_match: &FontDescriptor,
        family_descriptor: FontFamilyDescriptor,
    ) -> Vec<FontTemplateRef>;
}

/// The FontContext represents the per-thread/thread state necessary for
/// working with fonts. It is the public API used by the layout and
/// paint code. It talks directly to the font cache thread where
/// required.
#[derive(Debug)]
pub struct FontContext<S: FontSource> {
    font_source: Mutex<S>,

    // TODO: The font context holds a strong ref to the cached fonts
    // so they will never be released. Find out a good time to drop them.
    // See bug https://github.com/servo/servo/issues/3300
    font_cache: RwLock<HashMap<FontCacheKey, Option<FontRef>>>,
    font_template_cache: RwLock<HashMap<FontTemplateCacheKey, Vec<FontTemplateRef>>>,
    font_group_cache:
        RwLock<HashMap<FontGroupCacheKey, Arc<RwLock<FontGroup>>, BuildHasherDefault<FnvHasher>>>,
}

impl<S: FontSource> FontContext<S> {
    pub fn new(font_source: S) -> FontContext<S> {
        #[allow(clippy::default_constructed_unit_structs)]
        FontContext {
            font_source: Mutex::new(font_source),
            font_cache: RwLock::default(),
            font_template_cache: RwLock::default(),
            font_group_cache: RwLock::default(),
        }
    }

    /// Invalidate all caches that this [`FontContext`] holds and any in-process platform-specific
    /// caches.
    ///
    /// # Safety
    ///
    /// This should never be called when more than one thread is using the [`FontContext`] or it
    /// may leave the context in an inconsistent state.
    pub fn invalidate_caches(&self) {
        #[cfg(target_os = "macos")]
        CoreTextFontCache::clear_core_text_font_cache();

        self.font_cache.write().clear();
        self.font_template_cache.write().clear();
        self.font_group_cache.write().clear();
    }

    /// Returns a `FontGroup` representing fonts which can be used for layout, given the `style`.
    /// Font groups are cached, so subsequent calls with the same `style` will return a reference
    /// to an existing `FontGroup`.
    pub fn font_group(&self, style: ServoArc<FontStyleStruct>) -> Arc<RwLock<FontGroup>> {
        let font_size = style.font_size.computed_size().into();
        self.font_group_with_size(style, font_size)
    }

    /// Like [`Self::font_group`], but overriding the size found in the [`FontStyleStruct`] with the given size
    /// in pixels.
    pub fn font_group_with_size(
        &self,
        style: ServoArc<FontStyleStruct>,
        size: Au,
    ) -> Arc<RwLock<FontGroup>> {
        let cache_key = FontGroupCacheKey { size, style };

        if let Some(font_group) = self.font_group_cache.read().get(&cache_key) {
            return font_group.clone();
        }

        let font_group = Arc::new(RwLock::new(FontGroup::new(&cache_key.style)));
        self.font_group_cache
            .write()
            .insert(cache_key, font_group.clone());
        font_group
    }

    /// Returns a font matching the parameters. Fonts are cached, so repeated calls will return a
    /// reference to the same underlying `Font`.
    pub fn font(
        &self,
        font_template: FontTemplateRef,
        font_descriptor: &FontDescriptor,
    ) -> Option<FontRef> {
        self.get_font_maybe_synthesizing_small_caps(
            font_template,
            font_descriptor,
            true, /* synthesize_small_caps */
        )
    }

    fn get_font_maybe_synthesizing_small_caps(
        &self,
        font_template: FontTemplateRef,
        font_descriptor: &FontDescriptor,
        synthesize_small_caps: bool,
    ) -> Option<FontRef> {
        // TODO: (Bug #3463): Currently we only support fake small-caps
        // painting. We should also support true small-caps (where the
        // font supports it) in the future.
        let synthesized_small_caps_font =
            if font_descriptor.variant == FontVariantCaps::SmallCaps && synthesize_small_caps {
                let mut small_caps_descriptor = font_descriptor.clone();
                small_caps_descriptor.pt_size =
                    font_descriptor.pt_size.scale_by(SMALL_CAPS_SCALE_FACTOR);
                self.get_font_maybe_synthesizing_small_caps(
                    font_template.clone(),
                    &small_caps_descriptor,
                    false, /* synthesize_small_caps */
                )
            } else {
                None
            };

        let cache_key = FontCacheKey {
            font_identifier: font_template.identifier(),
            font_descriptor: font_descriptor.clone(),
        };

        if let Some(font) = self.font_cache.read().get(&cache_key).cloned() {
            return font;
        }

        debug!(
            "FontContext::font cache miss for font_template={:?} font_descriptor={:?}",
            font_template, font_descriptor
        );

        let font = self
            .create_font(
                font_template,
                font_descriptor.to_owned(),
                synthesized_small_caps_font,
            )
            .ok();
        self.font_cache.write().insert(cache_key, font.clone());
        font
    }

    pub fn matching_templates(
        &self,
        descriptor_to_match: &FontDescriptor,
        family_descriptor: &FontFamilyDescriptor,
    ) -> Vec<FontTemplateRef> {
        let cache_key = FontTemplateCacheKey {
            font_descriptor: descriptor_to_match.clone(),
            family_descriptor: family_descriptor.clone(),
        };

        if let Some(templates) = self.font_template_cache.read().get(&cache_key).cloned() {
            return templates;
        }

        debug!(
            "FontContext::font_template cache miss for template_descriptor={:?} family_descriptor={:?}",
            descriptor_to_match,
            family_descriptor
        );

        let templates = self
            .font_source
            .lock()
            .find_matching_font_templates(descriptor_to_match, family_descriptor.clone());

        self.font_template_cache
            .write()
            .insert(cache_key, templates.clone());
        templates
    }

    /// Create a `Font` for use in layout calculations, from a `FontTemplateData` returned by the
    /// cache thread and a `FontDescriptor` which contains the styling parameters.
    fn create_font(
        &self,
        font_template: FontTemplateRef,
        font_descriptor: FontDescriptor,
        synthesized_small_caps: Option<FontRef>,
    ) -> Result<FontRef, &'static str> {
        let mut font = Font::new(
            font_template.clone(),
            font_descriptor.clone(),
            synthesized_small_caps,
        )?;
        font.font_key = self.font_source.lock().get_font_instance(
            font_template.identifier(),
            font_descriptor.pt_size,
            font.webrender_font_instance_flags(),
        );

        Ok(Arc::new(font))
    }
}

#[derive(Debug, Eq, Hash, PartialEq)]
struct FontCacheKey {
    font_identifier: FontIdentifier,
    font_descriptor: FontDescriptor,
}

#[derive(Debug, Eq, Hash, PartialEq)]
struct FontTemplateCacheKey {
    font_descriptor: FontDescriptor,
    family_descriptor: FontFamilyDescriptor,
}

#[derive(Debug)]
struct FontGroupCacheKey {
    style: ServoArc<FontStyleStruct>,
    size: Au,
}

impl PartialEq for FontGroupCacheKey {
    fn eq(&self, other: &FontGroupCacheKey) -> bool {
        self.style == other.style && self.size == other.size
    }
}

impl Eq for FontGroupCacheKey {}

impl Hash for FontGroupCacheKey {
    fn hash<H>(&self, hasher: &mut H)
    where
        H: Hasher,
    {
        self.style.hash.hash(hasher)
    }
}
