/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::sync::{Arc, Mutex};

use app_units::Au;
use fonts::{
    FallbackFontSelectionOptions, FontContext, FontDescriptor, FontFamilyDescriptor,
    FontSearchScope, fallback_font_families,
};
use icu_locale_core::subtags::Language;
use net_traits::image_cache::FontResolver;
use resvg::usvg::{Font, FontFamily, FontStretch, FontStyle, fontdb};
use rustc_hash::FxHashMap;
use style::computed_values::font_optical_sizing::T as FontOpticalSizing;
use style::properties::longhands::font_variant_caps::computed_value::T as FontVariantCaps;
use style::values::computed::font::{
    FamilyName, FontFamilyNameSyntax, GenericFontFamily, SingleFontFamily,
};
use style::values::computed::{
    FontStyle as ServoFontStyle, FontSynthesis, FontWeight, FontWidth as ServoFontWidth,
};
use webrender_api::FontVariation;

/// Used to dynamically query fonts used in SVGs and insert them into the fontDB used when rasterizing.
#[derive(MallocSizeOf)]
pub struct SvgFontResolver {
    /// Cache for Font to ID
    font_id_cache: Mutex<FxHashMap<Font, fontdb::ID>>,
    fallback_id_cache: Mutex<FxHashMap<char, Vec<fontdb::ID>>>,
    #[ignore_malloc_size_of = "Arc"]
    database: Mutex<Arc<fontdb::Database>>,
    #[conditional_malloc_size_of]
    context: Arc<FontContext>,
}

impl SvgFontResolver {
    pub(crate) fn new(context: Arc<FontContext>) -> Self {
        Self {
            font_id_cache: Mutex::new(FxHashMap::default()),
            fallback_id_cache: Mutex::new(FxHashMap::default()),
            database: Mutex::new(Arc::new(fontdb::Database::new())),
            context,
        }
    }

    /// Insert the font into the database in [`SvgFontResolver`] and into the cache.
    fn insert_into_database(&self, font: &Font) -> Option<fontdb::ID> {
        let font_descriptor = font_to_fontdescriptor(font);

        for family in font.families() {
            let family_descriptor = FontFamilyDescriptor::new(
                fontfamily_to_singlefontfamily(family),
                FontSearchScope::Any,
            );

            let Some(font_template) = self
                .context
                .matching_templates(&font_descriptor, &family_descriptor)
                .into_iter()
                .next()
            else {
                log::debug!(
                    "Cannot find matching font_template from font {font:?}, font-descriptor {font_descriptor:?}, family_descriptor: {family_descriptor:?} for this family {family:?}"
                );
                continue;
            };

            let Some(font_ref) = self.context.font(font_template, &font_descriptor) else {
                continue;
            };

            let Ok(data_and_index) = font_ref.font_data_and_index() else {
                continue;
            };
            let mut shared_database = self.database.lock().unwrap();
            let ids = Arc::make_mut(&mut *shared_database).load_font_source(
                fontdb::Source::Binary(Arc::new(data_and_index.data.clone())),
            );

            if let Some(id) = ids.get(data_and_index.index as usize).copied() {
                self.font_id_cache.lock().unwrap().insert(font.clone(), id);
                return Some(id);
            }
        }

        None
    }
}

fn font_to_fontdescriptor(font: &Font) -> FontDescriptor {
    let style = match font.style() {
        FontStyle::Normal => ServoFontStyle::normal(),
        FontStyle::Italic => ServoFontStyle::ITALIC,
        FontStyle::Oblique => ServoFontStyle::OBLIQUE,
    };

    let width = match font.stretch() {
        FontStretch::UltraCondensed => ServoFontWidth::ULTRA_CONDENSED,
        FontStretch::ExtraCondensed => ServoFontWidth::EXTRA_CONDENSED,
        FontStretch::Condensed => ServoFontWidth::CONDENSED,
        FontStretch::SemiCondensed => ServoFontWidth::SEMI_CONDENSED,
        FontStretch::Normal => ServoFontWidth::NORMAL,
        FontStretch::SemiExpanded => ServoFontWidth::SEMI_EXPANDED,
        FontStretch::Expanded => ServoFontWidth::EXPANDED,
        FontStretch::ExtraExpanded => ServoFontWidth::EXTRA_EXPANDED,
        FontStretch::UltraExpanded => ServoFontWidth::ULTRA_EXPANDED,
    };

    let variation_settings = font
        .variations()
        .iter()
        .map(|variation| FontVariation {
            tag: u32::from_be_bytes(variation.tag),
            value: variation.value,
        })
        .collect();

    FontDescriptor {
        weight: FontWeight::from_float(font.weight() as f32),
        width,
        style,
        variant: FontVariantCaps::Normal,
        pt_size: Au::from_px(16),
        variation_settings,
        synthesis_weight: FontSynthesis::Auto,
        optical_sizing: FontOpticalSizing::Auto,
    }
}

fn fallback_descriptor() -> FontDescriptor {
    FontDescriptor {
        weight: FontWeight::normal(),
        width: ServoFontWidth::hundred(),
        style: ServoFontStyle::normal(),
        variant: FontVariantCaps::Normal,
        pt_size: Au::from_px(16),
        variation_settings: vec![],
        synthesis_weight: FontSynthesis::Auto,
        optical_sizing: FontOpticalSizing::Auto,
    }
}

fn fontfamily_to_singlefontfamily(family: &FontFamily) -> SingleFontFamily {
    match family {
        FontFamily::Serif => SingleFontFamily::Generic(GenericFontFamily::Serif),
        FontFamily::SansSerif => SingleFontFamily::Generic(GenericFontFamily::SansSerif),
        FontFamily::Cursive => SingleFontFamily::Generic(GenericFontFamily::Cursive),
        FontFamily::Fantasy => SingleFontFamily::Generic(GenericFontFamily::Fantasy),
        FontFamily::Monospace => SingleFontFamily::Generic(GenericFontFamily::Monospace),
        FontFamily::Named(name) => SingleFontFamily::FamilyName(FamilyName {
            name: name.as_str().into(),
            syntax: FontFamilyNameSyntax::Quoted,
        }),
    }
}

impl FontResolver for SvgFontResolver {
    fn resolve(&self, font: &Font, database: &mut Arc<fontdb::Database>) -> Option<fontdb::ID> {
        let font_id = {
            let id_cache = self.font_id_cache.lock().unwrap();
            id_cache.get(font).copied()
        };

        if let Some(font_id) = font_id {
            *database = self.database.lock().unwrap().clone();
            return Some(font_id);
        }

        let id = self.insert_into_database(font)?;
        *database = self.database.lock().unwrap().clone();
        Some(id)
    }

    fn resolve_fallback(
        &self,
        character: char,
        excluded: &[fontdb::ID],
        database: &mut Arc<fontdb::Database>,
    ) -> Option<fontdb::ID> {
        {
            let id_cache = self.fallback_id_cache.lock().unwrap();
            if let Some(font_id) = id_cache
                .get(&character)
                .and_then(|font_ids| font_ids.iter().find(|font_id| !excluded.contains(font_id)))
            {
                *database = self.database.lock().unwrap().clone();
                return Some(*font_id);
            }
        }
        let fallback_options =
            FallbackFontSelectionOptions::new(character, None, Language::UNKNOWN);

        for family in fallback_font_families(fallback_options) {
            let family = FontFamilyDescriptor::new(
                SingleFontFamily::FamilyName(FamilyName {
                    name: family.into(),
                    syntax: FontFamilyNameSyntax::Quoted,
                }),
                FontSearchScope::Any,
            );
            let fallback_descriptor = fallback_descriptor();

            let font_templates = self
                .context
                .matching_templates(&fallback_descriptor, &family);

            for font_template in font_templates {
                let Some(font_ref) = self.context.font(font_template, &fallback_descriptor) else {
                    continue;
                };
                if !font_ref.has_glyph_for(character) {
                    continue;
                }

                let Ok(data_and_index) = font_ref.font_data_and_index() else {
                    continue;
                };

                let mut shared_database = self.database.lock().unwrap();
                let ids = Arc::make_mut(&mut *shared_database).load_font_source(
                    fontdb::Source::Binary(Arc::new(data_and_index.data.clone())),
                );

                let Some(id) = ids.get(data_and_index.index as usize) else {
                    continue;
                };

                if excluded.contains(id) {
                    continue;
                }

                self.fallback_id_cache
                    .lock()
                    .unwrap()
                    .entry(character)
                    .or_default()
                    .push(*id);
                *database = shared_database.clone();
                return Some(*id);
            }
        }
        None
    }
}
