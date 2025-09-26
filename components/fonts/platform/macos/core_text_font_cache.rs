/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

use app_units::Au;
use core_foundation::base::TCFType;
use core_foundation::data::CFData;
use core_foundation::number::CFNumber;
use core_foundation::string::{CFString, CFStringRef};
use core_foundation::url::{CFURL, kCFURLPOSIXPathStyle};
use core_graphics::display::CFDictionary;
use core_text::font_descriptor::{kCTFontURLAttribute, kCTFontVariationAttribute};
use fonts_traits::FontIdentifier;
use parking_lot::RwLock;
use webrender_api::FontVariation;

use crate::FontData;
use crate::platform::font::PlatformFont;

/// A cache of `CTFont` to avoid having to create `CTFont` instances over and over. It is
/// always possible to create a `CTFont` using a `FontTemplate` even if it isn't in this
/// cache.
pub(crate) struct CoreTextFontCache(OnceLock<RwLock<HashMap<FontIdentifier, CachedCTFont>>>);

/// The global [`CoreTextFontCache`].
static CACHE: CoreTextFontCache = CoreTextFontCache(OnceLock::new());

/// A [`HashMap`] of cached [`CTFont`] for a single [`FontIdentifier`]. There is one [`CTFont`]
/// for each cached font size.
type CachedCTFont = HashMap<CoreTextFontCacheKey, PlatformFont>;

#[derive(Eq, Hash, PartialEq)]
struct CoreTextFontCacheKey {
    size: Au,
    variations: Vec<FontVariation>,
}

impl CoreTextFontCache {
    pub(crate) fn core_text_font(
        font_identifier: FontIdentifier,
        data: Option<&FontData>,
        pt_size: f64,
        variations: &[FontVariation],
    ) -> Option<PlatformFont> {
        //// If you pass a zero font size to one of the Core Text APIs, it'll replace it with
        //// 12.0. We don't want that! (Issue #10492.)
        let clamped_pt_size = pt_size.max(0.01);
        let au_size = Au::from_f64_px(clamped_pt_size);

        let key = CoreTextFontCacheKey {
            size: au_size,
            variations: variations.to_owned(),
        };

        let cache = CACHE.0.get_or_init(Default::default);
        {
            let cache = cache.read();
            if let Some(platform_font) = cache
                .get(&font_identifier)
                .and_then(|identifier_cache| identifier_cache.get(&key))
            {
                return Some(platform_font.clone());
            }
        }

        if !key.variations.is_empty() {
            let core_text_font_no_variations =
                Self::core_text_font(font_identifier.clone(), data, clamped_pt_size, &[])?;
            let mut cache = cache.write();
            let entry = cache.entry(font_identifier.clone()).or_default();

            // It could be that between the time of the cache miss above and now, after the write lock
            // on the cache has been acquired, the cache was populated with the data that we need. Thus
            // check again and return the CTFont if it is is already cached.
            if let Some(core_text_font) = entry.get(&key) {
                return Some(core_text_font.clone());
            }

            let platform_font = Self::add_variations_to_font(
                core_text_font_no_variations,
                &key.variations,
                clamped_pt_size,
            );
            entry.insert(key, platform_font.clone());
            return Some(platform_font);
        }

        let mut cache = cache.write();
        let identifier_cache = cache.entry(font_identifier.clone()).or_default();

        // It could be that between the time of the cache miss above and now, after the write lock
        // on the cache has been acquired, the cache was populated with the data that we need. Thus
        // check again and return the CTFont if it is is already cached.
        if let Some(platform_font) = identifier_cache.get(&key) {
            return Some(platform_font.clone());
        }

        let platform_font =
            Self::create_font_without_variations(font_identifier, data, clamped_pt_size)?;
        identifier_cache.insert(key, platform_font.clone());
        Some(platform_font)
    }

    pub(crate) fn create_font_without_variations(
        font_identifier: FontIdentifier,
        data: Option<&FontData>,
        clamped_pt_size: f64,
    ) -> Option<PlatformFont> {
        let descriptor = match font_identifier {
            FontIdentifier::Local(local_font_identifier) => {
                // Other platforms can instantiate a platform font by loading the data
                // from a file and passing an index in the case the file is a TTC bundle.
                // The only way to reliably load the correct font from a TTC bundle on
                // macOS is to create the font using a descriptor with both the PostScript
                // name and path.
                let cf_name = CFString::new(&local_font_identifier.postscript_name);
                let descriptor = core_text::font_descriptor::new_from_postscript_name(&cf_name);

                let cf_path = CFString::new(&local_font_identifier.path);
                let url_attribute = unsafe { CFString::wrap_under_get_rule(kCTFontURLAttribute) };
                let attributes = CFDictionary::from_CFType_pairs(&[(
                    url_attribute,
                    CFURL::from_file_system_path(cf_path, kCFURLPOSIXPathStyle, false),
                )]);

                descriptor.create_copy_with_attributes(attributes.to_untyped())
            },
            FontIdentifier::Web(_) => {
                let data = data
                    .expect("Should always have FontData for web fonts")
                    .clone();
                let cf_data = CFData::from_arc(Arc::new(data));
                core_text::font_manager::create_font_descriptor_with_data(cf_data)
            },
        };

        Some(PlatformFont::new_with_ctfont(
            core_text::font::new_from_descriptor(&descriptor.ok()?, clamped_pt_size),
        ))
    }

    fn add_variations_to_font(
        platform_font: PlatformFont,
        specified_variations: &[FontVariation],
        pt_size: f64,
    ) -> PlatformFont {
        if specified_variations.is_empty() {
            return platform_font;
        }
        let Some(variations) = Self::get_variation_axis_information(&platform_font) else {
            return platform_font;
        };

        let mut modified_variations = false;
        let variations: Vec<_> = variations
            .iter()
            .map(|variation| {
                let value = specified_variations
                    .iter()
                    .find_map(|specified_variation| {
                        if variation.tag == specified_variation.tag as i64 {
                            Some(specified_variation.value as f64)
                        } else {
                            None
                        }
                    })
                    .unwrap_or(variation.default_value)
                    .clamp(variation.min_value, variation.max_value);
                if value != variation.default_value {
                    modified_variations = true;
                }
                FontVariation {
                    tag: variation.tag as u32,
                    value: value as f32,
                }
            })
            .collect();

        if !modified_variations {
            return platform_font;
        }

        let cftype_variations: Vec<_> = variations
            .iter()
            .map(|variation| {
                (
                    CFNumber::from(variation.tag as i64),
                    CFNumber::from(variation.value as f64),
                )
            })
            .collect();
        let values_dict = CFDictionary::from_CFType_pairs(&cftype_variations);

        let variation_attribute =
            unsafe { CFString::wrap_under_get_rule(kCTFontVariationAttribute) };
        let attrs_dict = CFDictionary::from_CFType_pairs(&[(variation_attribute, values_dict)]);
        let ct_var_font_desc = platform_font
            .ctfont
            .copy_descriptor()
            .create_copy_with_attributes(attrs_dict.to_untyped())
            .unwrap();

        let ctfont = core_text::font::new_from_descriptor(&ct_var_font_desc, pt_size);
        PlatformFont::new_with_ctfont_and_variations(ctfont, variations)
    }

    fn get_variation_axis_information(
        platform_font: &PlatformFont,
    ) -> Option<Vec<VariationAxisInformation>> {
        Some(
            platform_font
                .ctfont
                .get_variation_axes()?
                .iter()
                .filter_map(|axes| {
                    let tag = unsafe { axes.find(kCTFontVariationAxisIdentifierKey) }
                        .and_then(|tag| tag.downcast::<CFNumber>())?;
                    let max_value = unsafe { axes.find(kCTFontVariationAxisMaximumValueKey) }
                        .and_then(|tag| tag.downcast::<CFNumber>())?;
                    let min_value = unsafe { axes.find(kCTFontVariationAxisMinimumValueKey) }
                        .and_then(|tag| tag.downcast::<CFNumber>())?;
                    let default_value = unsafe { axes.find(kCTFontVariationAxisDefaultValueKey) }
                        .and_then(|tag| tag.downcast::<CFNumber>())?;
                    Some(VariationAxisInformation {
                        tag: tag.to_i64()?,
                        max_value: max_value.to_f64()?,
                        min_value: min_value.to_f64()?,
                        default_value: default_value.to_f64()?,
                    })
                })
                .collect(),
        )
    }
}

#[derive(Clone, Default, Debug)]
struct VariationAxisInformation {
    tag: i64,
    max_value: f64,
    min_value: f64,
    default_value: f64,
}

unsafe extern "C" {
    static kCTFontVariationAxisDefaultValueKey: CFStringRef;
    static kCTFontVariationAxisIdentifierKey: CFStringRef;
    static kCTFontVariationAxisMaximumValueKey: CFStringRef;
    static kCTFontVariationAxisMinimumValueKey: CFStringRef;
}
