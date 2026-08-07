/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::sync::{Arc, Mutex};

use app_units::Au;
use fonts::{
    FallbackFontSelectionOptions, FontContext, FontDescriptor, FontFamilyDescriptor,
    FontSearchScope, fallback_font_families,
};
use net_traits::image_cache::FontResolver;
use resvg::usvg::{Font, FontFamily, FontStretch, FontStyle, fontdb};
use rustc_hash::FxHashMap;
use style::computed_values::font_optical_sizing::T as FontOpticalSizing;
use style::properties::longhands::font_variant_caps::computed_value::T as FontVariantCaps;
use style::values::computed::font::{
    FamilyName, FontFamilyNameSyntax, GenericFontFamily, SingleFontFamily,
};
use style::values::computed::{
    FontStretch as ServoFontStretch, FontStyle as ServoFontStyle, FontSynthesis, FontWeight,
};
use webrender_api::FontVariation;

/// Used to dynamically query fonts used in SVGs and insert them into the fontDB used when rasterizing.
pub struct SvgFontResolver {
    /// Cache for Font to ID
    font_id_cache: Mutex<FxHashMap<Font, fontdb::ID>>,
    fallback_id_cache: Mutex<FxHashMap<char, Vec<fontdb::ID>>>,
    context: Arc<FontContext>,
}

impl SvgFontResolver {
    pub(crate) fn new(context: Arc<FontContext>) -> Self {
        Self {
            font_id_cache: Mutex::new(FxHashMap::default()),
            fallback_id_cache: Mutex::new(FxHashMap::default()),
            context,
        }
    }

    /// Insert the font into the database in [`SvgFontResolver`] and into the cache.
    fn insert_into_database(
        &self,
        font: &Font,
        database: &mut Arc<fontdb::Database>,
    ) -> Option<fontdb::ID> {
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
            let ids = Arc::make_mut(database).load_font_source(fontdb::Source::Binary(
                data_and_index.data.as_ipc_shared_memory(),
            ));

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

    let stretch = match font.stretch() {
        FontStretch::UltraCondensed => ServoFontStretch::ULTRA_CONDENSED,
        FontStretch::ExtraCondensed => ServoFontStretch::EXTRA_CONDENSED,
        FontStretch::Condensed => ServoFontStretch::CONDENSED,
        FontStretch::SemiCondensed => ServoFontStretch::SEMI_CONDENSED,
        FontStretch::Normal => ServoFontStretch::NORMAL,
        FontStretch::SemiExpanded => ServoFontStretch::SEMI_EXPANDED,
        FontStretch::Expanded => ServoFontStretch::EXPANDED,
        FontStretch::ExtraExpanded => ServoFontStretch::EXTRA_EXPANDED,
        FontStretch::UltraExpanded => ServoFontStretch::ULTRA_EXPANDED,
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
        stretch,
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
        stretch: ServoFontStretch::hundred(),
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
        {
            let id_cache = self.font_id_cache.lock().unwrap();
            if let Some(font_id) = id_cache.get(font) {
                return Some(*font_id);
            }
        }
        self.insert_into_database(font, database)
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
                return Some(*font_id);
            }
        }
        let fallback_options =
            FallbackFontSelectionOptions::new(character, None, icu_locid::subtags::Language::UND);
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

                let ids = Arc::make_mut(database).load_font_source(fontdb::Source::Binary(
                    data_and_index.data.as_ipc_shared_memory(),
                ));

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
                return Some(*id);
            }
        }
        None
    }
}
