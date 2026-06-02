/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;
use std::path::Path;
use std::sync::OnceLock;

use app_units::Au;
use fonts_traits::FontIdentifier;
use objc2_core_foundation::{
    CFData, CFDictionary, CFNumber, CFRetained, CFString, CFType, CFURL, CGFloat,
};
use objc2_core_text::{
    CTFont, CTFontDescriptor, CTFontManagerCreateFontDescriptorFromData, kCTFontURLAttribute,
    kCTFontVariationAttribute, kCTFontVariationAxisDefaultValueKey,
    kCTFontVariationAxisIdentifierKey, kCTFontVariationAxisMaximumValueKey,
    kCTFontVariationAxisMinimumValueKey,
};
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
    synthetic_bold: bool,
    variations: Vec<FontVariation>,
}

impl CoreTextFontCache {
    pub(crate) fn core_text_font(
        font_identifier: FontIdentifier,
        data: Option<&FontData>,
        pt_size: f64,
        variations: &[FontVariation],
        synthetic_bold: bool,
    ) -> Option<PlatformFont> {
        //// If you pass a zero font size to one of the Core Text APIs, it'll replace it with
        //// 12.0. We don't want that! (Issue #10492.)
        let clamped_pt_size = pt_size.max(0.01);
        let au_size = Au::from_f64_px(clamped_pt_size);

        let key = CoreTextFontCacheKey {
            size: au_size,
            synthetic_bold,
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
            let core_text_font_no_variations = Self::core_text_font(
                font_identifier.clone(),
                data,
                clamped_pt_size,
                &[],
                synthetic_bold,
            )?;
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
                synthetic_bold,
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

        let platform_font = Self::create_font_without_variations(
            font_identifier,
            data,
            clamped_pt_size,
            synthetic_bold,
        )?;
        identifier_cache.insert(key, platform_font.clone());
        Some(platform_font)
    }

    pub(crate) fn create_font_without_variations(
        font_identifier: FontIdentifier,
        data: Option<&FontData>,
        clamped_pt_size: f64,
        synthetic_bold: bool,
    ) -> Option<PlatformFont> {
        let descriptor = match font_identifier {
            FontIdentifier::Local(local_font_identifier) => {
                // Other platforms can instantiate a platform font by loading the data
                // from a file and passing an index in the case the file is a TTC bundle.
                // The only way to reliably load the correct font from a TTC bundle on
                // macOS is to create the font using a descriptor with both the PostScript
                // name and path.
                let postscript_name = CFString::from_str(&local_font_identifier.postscript_name);
                let descriptor =
                    unsafe { CTFontDescriptor::with_name_and_size(&postscript_name, 0.0) };

                let cf_url =
                    CFURL::from_file_path(Path::new(&local_font_identifier.path.to_string()))?;
                let attributes: CFRetained<CFDictionary<CFString, CFType>> =
                    CFDictionary::from_slices(
                        &[unsafe { kCTFontURLAttribute }],
                        &[cf_url.as_ref()],
                    );
                unsafe { descriptor.copy_with_attributes(attributes.as_opaque()) }
            },
            FontIdentifier::Web(_) => {
                let data = data
                    .expect("Should always have FontData for web fonts")
                    .clone();
                let data = CFData::from_bytes(data.as_ref());
                unsafe { CTFontManagerCreateFontDescriptorFromData(&data)? }
            },
        };

        let ctfont = unsafe {
            CTFont::with_font_descriptor(&descriptor, clamped_pt_size as CGFloat, std::ptr::null())
        };
        Some(PlatformFont::new_with_ctfont(ctfont, synthetic_bold))
    }

    fn add_variations_to_font(
        platform_font: PlatformFont,
        specified_variations: &[FontVariation],
        pt_size: f64,
        synthetic_bold: bool,
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

        let variation_keys: Vec<_> = variations
            .iter()
            .map(|variation| CFNumber::new_i64(variation.tag as i64))
            .collect();
        let variation_values: Vec<_> = variations
            .iter()
            .map(|variation| CFNumber::new_f32(variation.value))
            .collect();
        let values_dict = CFDictionary::<CFNumber, CFNumber>::from_slices(
            &variation_keys
                .iter()
                .map(CFRetained::as_ref)
                .collect::<Vec<_>>(),
            &variation_values
                .iter()
                .map(CFRetained::as_ref)
                .collect::<Vec<_>>(),
        );

        let attributes = CFDictionary::<CFString, CFType>::from_slices(
            &[unsafe { kCTFontVariationAttribute }],
            &[values_dict.as_ref()],
        );
        let descriptor_with_variations = unsafe {
            platform_font
                .ctfont
                .font_descriptor()
                .copy_with_attributes(attributes.as_opaque())
        };

        let ctfont = unsafe {
            CTFont::with_font_descriptor(&descriptor_with_variations, pt_size, std::ptr::null())
        };
        PlatformFont::new_with_ctfont_and_variations(ctfont, variations, synthetic_bold)
    }

    fn get_variation_axis_information(
        platform_font: &PlatformFont,
    ) -> Option<Vec<VariationAxisInformation>> {
        let variation_axes = unsafe { platform_font.ctfont.variation_axes()? };
        let traits = unsafe { variation_axes.cast_unchecked::<CFDictionary<CFString, CFNumber>>() };

        Some(
            traits
                .iter()
                .filter_map(|axes| {
                    let tag = unsafe { axes.get(kCTFontVariationAxisIdentifierKey) }?;
                    let max_value = unsafe { axes.get(kCTFontVariationAxisMaximumValueKey) }?;
                    let min_value = unsafe { axes.get(kCTFontVariationAxisMinimumValueKey) }?;
                    let default_value = unsafe { axes.get(kCTFontVariationAxisDefaultValueKey) }?;
                    Some(VariationAxisInformation {
                        tag: tag.as_i64()?,
                        max_value: max_value.as_f64()?,
                        min_value: min_value.as_f64()?,
                        default_value: default_value.as_f64()?,
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
