/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::sync::{Arc, Mutex};

use app_units::Au;
use fonts::{FontContext, FontDescriptor, FontFamilyDescriptor, FontSearchScope};
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

pub struct SvgFontResolver {
    /// Cache for Font to ID
    font_id_cache: Mutex<FxHashMap<Font, fontdb::ID>>,
    context: Arc<FontContext>,
}

impl SvgFontResolver {
    pub(crate) fn new(context: Arc<FontContext>) -> Self {
        Self {
            font_id_cache: Mutex::new(FxHashMap::default()),
            context,
        }
    }
}

fn convert_font_descriptor(font: &Font) -> FontDescriptor {
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

fn convert_font_family(family: &FontFamily) -> SingleFontFamily {
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

/// Insert the font into the database in [`SvgFontResolver`] and into the cache.
fn insert_into_database(
    resolver: &SvgFontResolver,
    font: &Font,
    database: &mut Arc<fontdb::Database>,
) -> Option<fontdb::ID> {
    let font_descriptor = convert_font_descriptor(font);

    for family in font.families() {
        let family_descriptor =
            FontFamilyDescriptor::new(convert_font_family(family), FontSearchScope::Any);
        let Some(font_template) = resolver
            .context
            .matching_templates(&font_descriptor, &family_descriptor)
            .into_iter()
            .next()
        else {
            continue;
        };
        let Some(font) = resolver.context.font(font_template, &font_descriptor) else {
            continue;
        };
        let Ok(data_and_index) = font.font_data_and_index() else {
            continue;
        };
        let ids = Arc::make_mut(database).load_font_source(fontdb::Source::Binary(
            data_and_index.data.as_ipc_shared_memory(),
        ));
        if let Some(id) = ids.get(data_and_index.index as usize).copied() {
            return Some(id);
        }
    }

    None
}

impl FontResolver for SvgFontResolver {
    fn resolve(&self, font: &Font, database: &mut Arc<fontdb::Database>) -> Option<fontdb::ID> {
        let id_cache = self.font_id_cache.lock().unwrap();
        if let Some(font_id) = id_cache.get(font) {
            Some(*font_id)
        } else {
            insert_into_database(self, font, database)
        }
    }
}
