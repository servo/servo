/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashSet;

// Proper locale handling
use icu_locid::subtags::{language, script};
use icu_locid::LanguageIdentifier;

use super::FallbackOptionsKey;
use crate::platform::font_list::{
    FallbackAssociations, FontAlias, FontFamily, OpenHarmonyFontDescriptor, convert_script,
};
use crate::platform::freetype::ohos::json::{
    FallbackEntryOHOS, FontconfigOHOS, GenericFontFamilyOHOS,
};

/* Functions bellow is used to extract Generic and Fallback font families
   from OpenHarmony fontconfig.json file. Results of their work used in
   constructor of FONT_LIST object.
*/

fn get_generic_family_font_file_path_from_ohos_fontconfig<'a>(
    full_family_name: &'a str,
    config: &'a FontconfigOHOS,
) -> Option<&'a str> {
    let font_full_name_to_filepath = &config.font_file_map;
    let mut family_name_key = full_family_name.to_string();

    // Awfull performance. Rewrite this.
    if let Some(res) = font_full_name_to_filepath
        .iter()
        .find(|entry| family_name_key == entry.0)
    {
        return Some(&res.1);
    } else {
        if log::log_enabled!(log::Level::Debug) {
            log::warn!("Was unable to find font file with canonicalized naming");
            log::warn!("Will try regular variant");
        }
        family_name_key = full_family_name.to_string() + " Regular";
        if let Some(res) = font_full_name_to_filepath
            .iter()
            .find(|entry| family_name_key == entry.0)
        {
            return Some(&res.1);
        } else {
            if log::log_enabled!(log::Level::Error) {
                log::error!(
                    r#"
                    Unable to find fontfile path for family in verified config!
                    Check OHOS fontconfig verification code!
                    family name: {}
                    "#,
                    full_family_name
                );
            }
        }
    }
    None
}

fn get_all_family_font_file_paths_from_ohos_fontconfig<'a>(
    full_family_name: &'a str,
    config: &'a FontconfigOHOS,
) -> Vec<&'a str> {
    let mut result = Vec::<&'a str>::new();
    let font_full_name_to_filepath = &config.font_file_map;
    for (font_full_name, font_file_path) in font_full_name_to_filepath.iter() {
        if font_full_name.contains(full_family_name) {
            result.push(font_file_path)
        }
    }
    result
}

fn get_family_weight_from_font_variations_entry(variation: &[(String, i32); 2]) -> Option<i32> {
    if variation[0].0.contains("weight") {
        return Some(variation[0].1);
    }
    if variation[1].0.contains("weight") {
        return Some(variation[1].1);
    }
    if log::log_enabled!(log::Level::Error) {
        log::error!(
            r#"
            Unable to get font weight from font-variations in verified config!
            Check OHOS fontconfig verification code!
        "#
        );
    }
    None
}

fn process_generic_entry_from_ohos_config(
    generic_font_family: &GenericFontFamilyOHOS,
    config: &FontconfigOHOS,
) -> Option<(FontFamily, Vec<FontAlias>)> {
    let family_name = &generic_font_family.family;
    let mut family_fonts = Vec::<OpenHarmonyFontDescriptor>::new();

    let res = get_generic_family_font_file_path_from_ohos_fontconfig(family_name, config);
    if res.is_none() {
        return None;
    }
    let filepath = res.unwrap().to_string();

    let font_variations = &generic_font_family.font_variations;
    // Code for common script
    // script!("Zyyy") == Script::Common
    if font_variations.is_empty() {
        family_fonts.push(OpenHarmonyFontDescriptor {
            filepath: filepath.clone(),
            ..Default::default()
        });
    }
    for variation in font_variations {
        let weight = get_family_weight_from_font_variations_entry(variation);
        family_fonts.push(OpenHarmonyFontDescriptor {
            filepath: filepath.clone(),
            weight,
            ..Default::default()
        });
    }

    let family = FontFamily {
        name: family_name.to_string(),
        fonts: family_fonts,
    };

    let list_of_aliases_in_config = &generic_font_family.alias;
    let family_aliases = list_of_aliases_in_config
        .iter()
        .map(|(alias, weight)| {
            let effective_weight: Option<i32> = match *weight {
                // CSS weight is allowed from 1 to 999
                0 => None,
                _ => Some(*weight),
            };
            // Convert system-alias notation. To CSS alias.
            let resulting_alias = match alias.as_str() {
                "serif" => "ui-serif",
                "sans-serif" => "ui-sans-serif",
                "monospace" => "ui-monospace",
                "rounded" => "ui-rounded",
                _ => alias,
            };
            FontAlias {
                from: resulting_alias.to_string(),
                to: family_name.to_string(),
                weight: effective_weight,
            }
        })
        .collect();

    Some((family, family_aliases))
}

pub fn generic_font_families_from_ohos_fontconfig(
    config: &FontconfigOHOS,
) -> (Vec<FontFamily>, Vec<FontAlias>) {
    let mut result_fonts = Vec::<FontFamily>::new();
    let mut result_aliases = Vec::<FontAlias>::new();
    for generic_entry in &config.generic {
        let candidate = process_generic_entry_from_ohos_config(generic_entry, config);
        if let Some((generic_family, generic_family_aliases)) = candidate {
            result_fonts.push(generic_family);
            result_aliases.extend(generic_family_aliases);
        }
    }
    (result_fonts, result_aliases)
}

fn find_full_name_to_generic_family_name_association<'a>(
    full_name: &'a str,
    generic_families: &'a mut Vec<FontFamily>,
) -> Option<&'a mut FontFamily> {
    let mut candidate: Option<&mut FontFamily> = None;
    for font_family in generic_families {
        let family_name = &font_family.name;
        if full_name.contains(family_name) {
            // Process first ever found candidate
            if candidate.is_none() {
                candidate = Some(font_family);
                continue;
            }
            // We will return longest candidate
            // Decide between 2 candidates
            if let Some(ref cur_candidate) = candidate {
                if family_name.len() > cur_candidate.name.len() {
                    candidate = Some(font_family);
                }
            }
        }
    }
    candidate
}

fn process_fallback_list_from_ohos_config(
    fallback_list: &[FallbackEntryOHOS],
    generic_families: &mut Vec<FontFamily>,
    config: &FontconfigOHOS,
) -> (Vec<FontFamily>, FallbackAssociations) {
    let mut result_fonts = Vec::<FontFamily>::new();
    let mut result_fallback_associations = FallbackAssociations::new();
    let mut processed_filepaths = HashSet::<String>::new();

    for fallback_font in fallback_list {
        let mut family_fonts = Vec::<OpenHarmonyFontDescriptor>::new();
        // TODO(ddesyatkin): Think about feasibility of introduction of the following function
        // fn conversion_lang_id_to_unicode_block(lang_id: LanguageIdentifier) -> UnicodeBlock

        let (lang_id, font_family_with_script) = &fallback_font.lang_id_to_family;
        let font_variations = &fallback_font.font_variations;

        // Try to find generic system family that will match with current font_family_with_script
        let mut generic_family_candidate = find_full_name_to_generic_family_name_association(
            font_family_with_script,
            generic_families,
        );

        if let Some(ref generic_family) = generic_family_candidate {
            generic_family.fonts.iter().for_each(|font| {
                processed_filepaths.insert(font.filepath.to_string());
            });
        }

        // Get all posible candidates for new system font file paths;
        let res =
            get_all_family_font_file_paths_from_ohos_fontconfig(font_family_with_script, config);
        if res.is_empty() {
            continue;
        }

        let original_family_filepaths = res.clone();

        // Filter paths from all generic (system) families that we was able to process before
        let filtered_filepaths: Vec<&str> = res
            .into_iter()
            .filter_map(|filepath| {
                if processed_filepaths.contains(filepath) {
                    return None;
                }
                Some(filepath)
            })
            .collect();

        let key = FallbackOptionsKey::new_from_lang_id(lang_id.clone());

        if filtered_filepaths.is_empty() {
            // If all filepaths was filtered it means that currently we process part of segmented font
            // And it became the part of the GenericFontFamily list. We must properly mark the script within
            // font family though.
            if let Some(ref mut generic_family) = generic_family_candidate {
                result_fallback_associations
                    .add_value_to_set_on_key(key, generic_family.name.clone());
                generic_family.fonts.iter_mut().for_each(|font| {
                    if original_family_filepaths.contains(&font.filepath.as_str()) {
                        font.language = lang_id.clone();
                    }
                });
            }
            continue;
        }

        for filepath in filtered_filepaths {
            if font_variations.is_empty() {
                family_fonts.push(OpenHarmonyFontDescriptor {
                    filepath: filepath.to_string(),
                    language: lang_id.clone(),
                    ..Default::default()
                });
            }

            for variation in font_variations {
                let weight = get_family_weight_from_font_variations_entry(variation);
                family_fonts.push(OpenHarmonyFontDescriptor {
                    filepath: filepath.to_string(),
                    weight,
                    language: lang_id.clone(),
                    ..Default::default()
                });
            }
        }

        // Add fallback fonts that corresponds to generic font family into
        // existing font family.
        if let Some(ref mut generic_family) = generic_family_candidate {
            result_fallback_associations.add_value_to_set_on_key(key, generic_family.name.clone());
            generic_family.fonts.extend(family_fonts.clone());
            continue;
        }

        // If we met some family that doesn't have clear lang_script instructions,
        // that particular family should become default fallback family if we was unable to match
        // against any style that user asked (GenericFontFamily::None)

        // So we should add it to generic system families cause only they are visible through
        // default_system_generic_font_family function
        if lang_id.language == language!("und") &&
            convert_script(lang_id.script) == convert_script(script!("Zzzz").into())
        // script can not be created from alias now Zyyy == Common | Zzzz == Unknown
        {
            result_fallback_associations
                .add_value_to_set_on_key(key, font_family_with_script.to_string());

            family_fonts.iter_mut().for_each(|font| {
                font.language = LanguageIdentifier::default();
            });

            generic_families.push(FontFamily {
                name: font_family_with_script.to_string(),
                fonts: family_fonts.clone(),
            });
            continue;
        }

        // If we was unable to find family in generic families, create new fallback font family.
        result_fallback_associations
            .add_value_to_set_on_key(key, font_family_with_script.to_string());
        result_fonts.push(FontFamily {
            name: font_family_with_script.to_string(),
            fonts: family_fonts,
        });
    }
    (result_fonts, result_fallback_associations)
}

pub fn fallback_font_families_from_ohos_fontconfig(
    generic_families: &mut Vec<FontFamily>,
    config: &FontconfigOHOS,
) -> (Vec<FontFamily>, FallbackAssociations) {
    let mut result = Vec::<FontFamily>::new();
    let mut result_associations = FallbackAssociations::new();
    for (_fallback_name, fallback_list) in &config.fallback {
        // _fallback_name now ohos fontconfig has only one fallback strategy.
        let (strategy_families_vec, strategy_families_associations) =
            process_fallback_list_from_ohos_config(&fallback_list, generic_families, config);
        result.extend(strategy_families_vec);
        result_associations.extend(strategy_families_associations)
    }
    (result, result_associations)
}
