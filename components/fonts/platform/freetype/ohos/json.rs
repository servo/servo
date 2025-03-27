/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;
use std::fs;
use std::path::PathBuf;
use std::string::String;
use std::vec::Vec;

use icu_locid::{LanguageIdentifier, langid};
use serde_json::Value;

use super::font_list::enumerate_font_files;

static FONT_CONFIG_PATH: &str = "/etc/fontconfig.json";

/* This file contains custom functions for json parsing to avoid
   blind reliance on serde_json::from_*. Even if user make some mistake in
   fontconfig we want to inform him about the problem. All correct information
   should be preserved and used in servo engine.

   Example of recoverable user mistake:
   "alias": [
       {
          "HarmonyOS-Sans": 0,
          "HarmonyOS-Sans-Light": 100
       },
   ]
   We get modified fontconfig file and entry of generic font alias contains several
   PostScript names to font-weight associations instead of one association.
   We will take first one and report about error in the fontconfig.js

   Example of correct alias bellow:
   "alias": [
       {
          "HarmonyOS-Sans": 0,
       },
       {
          "HarmonyOS-Sans-Light": 100
       }
   ]

   Example of non recoverable user mistake:
   "alias": [
       {
          "HarmonyOS-Sans: 0,
       },
   ]
   "HarmonyOS-Sans <- missing quotation mark at the end of the string
   currently I do not want to fix errors in JSON TEXT. Probably that is a good task for the future.
*/

/// Represents individual entry in vector of generic font families supported by
/// OpenHarmony
#[derive(Clone, Debug)]
pub(super) struct GenericFontFamilyOHOS {
    pub family: String,
    pub alias: Vec<(String, i32)>,
    #[allow(unused)]
    pub adjust: Vec<[(String, i32); 2]>,
    pub font_variations: Vec<[(String, i32); 2]>,
}

/// Represents individual entry in named fallback
#[derive(Clone, Debug)]
pub(super) struct FallbackEntryOHOS {
    /// lang_srcipt contains pair of language-script value of content
    /// and family name for it visual representation
    /// ("zh-Hans": "HarmonyOS Sans SC")
    pub lang_id_to_family: (LanguageIdentifier, String),
    /// family may support; Currently used only for font-weight values!
    pub font_variations: Vec<[(String, i32); 2]>,
}

/// Representation of OpenHarmony fontconfig.json
/// in Rust structure. We don't have clear requirements for object
/// definition inside this structure. Just remember, even if there is
/// some links to CSS specifications here, it doesn't mean that object
/// inside structure should follow CSS spec. This just convenient rust
/// representation of some OpenHarmony specific objects.
#[derive(Debug)]
pub(super) struct FontconfigOHOS {
    /// Array that contains all folders where we should search fonts
    /// on OpenHarmony OS.
    #[allow(unused)]
    pub fontdir: Vec<String>,
    /// Fonts that should be used as generic on HarmonyOS
    /// <https://www.w3.org/TR/css-fonts-4/#generic-family-name-syntax>
    /// (we extract ui-serif, ui-sans-serif, ui-monospace, ui-rounded)
    pub generic: Vec<GenericFontFamilyOHOS>,
    /// Information that provides default font fallbacks (first fonts in
    /// installed font list in CSS notation that we should try if we was unable
    /// to match family with all family names specified by user)
    pub fallback: Vec<(String, Vec<FallbackEntryOHOS>)>,
    /// Table that provides association between font-family and font file path.
    pub font_file_map: Vec<(String, String)>,
}

// ##########################################################
// Block of functions that is responsible for general types #
// ##########################################################
#[inline]
fn convert_to_i32_value(serde_val: &serde_json::Value) -> Option<i32> {
    // serde_val example
    // "_": 100
    // we working with 100
    match serde_val {
        Value::Number(number) => {
            if let Some(alias_val) = number.as_i64() {
                let alias_val = i32::try_from(alias_val);
                match alias_val {
                    Err(e) => {
                        log::warn!("Value in {:?} overflows i32\n {:?}", FONT_CONFIG_PATH, e);
                        return None;
                    },
                    Ok(data) => return Some(data),
                }
            }
            None
        },
        _ => {
            log::warn!("Unexpected value in supposedly i32 value");
            None
        },
    }
}

#[inline]
fn convert_string_value(serde_val: &serde_json::Value) -> Option<String> {
    // serde_val example:
    // "_": "HarmonyOS Sans"
    // we working with "HarmonyOS Sans"
    match serde_val {
        Value::String(string) => Some(string.to_string()),
        _ => {
            log::warn!("Unexpected value in supposedly string value");
            None
        },
    }
}

#[inline]
fn convert_lang_script(lang_script_sequence: &str) -> Vec<LanguageIdentifier> {
    let mut result = Vec::<LanguageIdentifier>::new();
    if lang_script_sequence.is_empty() {
        result.push(langid!("und"));
        return result;
    }
    for lang_script in lang_script_sequence.split(',') {
        // Use `LanguageIdentifier` cause locale contains unnecessary extensions
        // This is just author decision. Can be changed to `Locale` in the future;
        match LanguageIdentifier::try_from_bytes(lang_script.as_bytes()) {
            Err(e) => {
                if log::log_enabled!(log::Level::Error) {
                    log::error!(
                        "Error during parsing of locale in fallback array!\nEntry: {}, error: {}",
                        lang_script,
                        e
                    );
                }
                return result;
            },
            Ok(lang_id) => result.push(lang_id),
        };
    }

    result
}

#[inline]
fn convert_to_two_string_int_pair_value(
    obj_entry: &serde_json::Value,
) -> Option<[(String, i32); 2]> {
    // obj_entry example
    // {
    //   "weight": 50, "to": 100
    // },
    // or
    // {
    //   "weight": 100, "wght": 100
    // },
    match obj_entry {
        Value::Object(map) => {
            let mut map_iter = map.iter();
            let mut first: Option<(String, i32)> = None;
            let mut second: Option<(String, i32)> = None;
            if let Some((string_ref, serde_val)) = map_iter.next() {
                if let Some(processed_value) = convert_to_i32_value(serde_val) {
                    first = Some((string_ref.to_string(), processed_value));
                }
            }

            if let Some((string_ref, serde_val)) = map_iter.next() {
                if let Some(processed_value) = convert_to_i32_value(serde_val) {
                    second = Some((string_ref.to_string(), processed_value));
                }
            }

            let unexpected_entry = map_iter.next();
            if unexpected_entry.is_some() {
                log::warn!("We expect exactly 2 (String, Value) pairs in the type");
                return None;
            }

            match (first, second) {
                (Some(first_internal), Some(second_internal)) => {
                    Some([first_internal, second_internal])
                },
                (_, _) => None,
            }
        },
        _ => {
            log::warn!(
                "Entry in adjust of font-variations array should be representable as Object(Map<String,Value>)"
            );
            None
        },
    }
}

// ####################################################################
// Block of functions that is responsible for FontconfigOHOS::fontdir #
// ####################################################################
fn detect_fonts_specified_in_fontdir(paths: &[String]) -> Vec<PathBuf> {
    let mut found_font_files = Vec::<PathBuf>::new();
    for directory in paths {
        match enumerate_font_files(directory.as_str()) {
            Err(_) => {},
            Ok(data) => {
                found_font_files.extend(data.into_iter());
            },
        }
    }
    found_font_files
}
fn convert_and_verify_fontdir(
    serde_val: &serde_json::Value,
) -> Option<(Vec<String>, Vec<PathBuf>)> {
    // serde_val example
    //"fontdir": ["/system/fonts/"],

    // convert to more convenient Rust Structure
    let fontdir: Option<Vec<String>> = match serde_val {
        Value::Array(array) => Some(array.iter().filter_map(convert_string_value).collect()),
        _ => {
            log::warn!("Fontdir value should be array representable");
            None
        },
    };

    // verify correctness of the value
    if let Some(fontdir) = fontdir {
        let found_font_files = detect_fonts_specified_in_fontdir(&fontdir);
        if found_font_files.is_empty() {
            log::warn!(
                r#"
            Not a single .ttf or .ttc file was not found under the paths
            specified in fontdir field of fontconfig.json! Please check values!
            "#
            );
            return None;
        }
        return Some((fontdir, found_font_files));
    }
    None
}

// ###########################################################################
// Block of functions that is responsible for FontconfigOHOS::generic::alias #
// ###########################################################################
#[inline]
fn convert_to_alias_entry(serde_val: &serde_json::Value) -> Option<(String, i32)> {
    // serde_val example
    // {
    //   "HarmonyOS-Sans": 0
    // },
    match serde_val {
        Value::Object(map) => {
            let mut map_iter = map.iter();
            if let Some((alias_str, alias_val)) = map_iter.next() {
                let next_entry = map_iter.next();
                if next_entry.is_some() {
                    log::warn!(
                        "Alias array entry map should have only one {{string}}:{{number}} pair"
                    );
                }
                if let Some(data) = convert_to_i32_value(alias_val) {
                    return Some((alias_str.to_string(), data));
                }
            }
            None
        },
        _ => {
            log::warn!(
                "Entry in allias array should be representable as Object(Map<String,Value>)"
            );
            None
        },
    }
}

fn convert_alias(serde_val: &serde_json::Value) -> Vec<(String, i32)> {
    // serde_val example
    // "alias": [
    //     {
    //       "HarmonyOS-Sans": 0
    //     },
    //     {
    //       "HarmonyOS-Sans-Light": 100
    //     }
    // ]

    match serde_val {
        Value::Array(array) => array.iter().filter_map(convert_to_alias_entry).collect(),
        _ => {
            log::warn!("Alias value should be array representable");
            Vec::<(String, i32)>::new()
        },
    }
}

// ############################################################################
// Block of functions that is responsible for FontconfigOHOS::generic::adjust #
// and FontconfigOHOS::generic::font_variation                                #
// ############################################################################
fn convert_adjust_or_font_variations(serde_val: &serde_json::Value) -> Vec<[(String, i32); 2]> {
    // serde_val example
    // "adjust": [
    //     {
    //       "weight": 50, "to": 100
    //     },
    //     {
    //       "weight": 80, "to": 400
    //     },
    // ]
    // or
    // "font-variations": [
    //     {
    //       "weight": 100, "wght": 100
    //     },
    //     {
    //       "weight": 300, "wght": 247
    //     },
    // ]

    match serde_val {
        Value::Array(array) => array
            .iter()
            .filter_map(convert_to_two_string_int_pair_value)
            .collect(),
        _ => {
            log::warn!(
                "adjust or font-variation values in fontconfig.json should be array representable"
            );
            Vec::<[(String, i32); 2]>::new()
        },
    }
}

// ####################################################################
// Block of functions that is responsible for FontconfigOHOS::generic #
// ####################################################################
fn convert_generic_array_entry(serde_val: &serde_json::Value) -> Option<GenericFontFamilyOHOS> {
    // serde_val is entry of config["generic"]
    match serde_val {
        Value::Object(map) => {
            let mut family = String::new();
            let mut alias = Vec::<(String, i32)>::new();
            let mut adjust = Vec::<[(String, i32); 2]>::new();
            let mut font_variations = Vec::<[(String, i32); 2]>::new();
            for (string_ref, serde_val) in map.iter() {
                match string_ref.as_str() {
                    "family" => {
                        if let Some(result) = convert_string_value(serde_val) {
                            family.extend(result.chars());
                        } else {
                            return None;
                        }
                    },
                    "alias" => {
                        alias.extend(convert_alias(serde_val).into_iter());
                    },
                    "adjust" => {
                        adjust.extend(convert_adjust_or_font_variations(serde_val).into_iter());
                    },
                    "font-variations" => {
                        font_variations
                            .extend(convert_adjust_or_font_variations(serde_val).into_iter());
                    },
                    _ => {
                        log::warn!("Unexpected key in generic array entry");
                    },
                }
            }
            // If we was able to recover family field we consider that we will be able to use
            // information from config partially
            return Some(GenericFontFamilyOHOS {
                family,
                alias,
                adjust,
                font_variations,
            });
        },
        _ => {
            log::warn!("Unexpected value in generic array entry value");
        },
    }
    None
}

fn convert_and_verify_generic(serde_val: &serde_json::Value) -> Vec<GenericFontFamilyOHOS> {
    // serde_val is config["generic"]

    // convert to more convenient Rust Structure
    match serde_val {
        Value::Array(array) => array
            .iter()
            .filter_map(convert_generic_array_entry)
            .collect(),
        _ => {
            log::warn!("Unexpected value in generic entry");
            Vec::<GenericFontFamilyOHOS>::new()
        },
    }
}

// #####################################################################
// Block of functions that is responsible for FontconfigOHOS::fallback #
// #####################################################################
#[inline]
fn convert_fallback_array_entry_value(obj_entry: &serde_json::Value) -> Vec<FallbackEntryOHOS> {
    // obj_entry example
    // "font-variations": [
    //         {
    //           "weight": 100, "wght": 100
    //         },
    // ]
    // or
    // "und-Arab": "HarmonyOS Sans Naskh Arabic UI"
    let mut result = Vec::<FallbackEntryOHOS>::new();
    match obj_entry {
        Value::Object(map) => {
            let mut lang_id_to_family_vec = Vec::<(LanguageIdentifier, String)>::new();
            let mut font_variations = Vec::<[(String, i32); 2]>::new();
            for (string_ref, serde_val) in map.iter() {
                match string_ref.as_str() {
                    "font-variations" => {
                        font_variations
                            .extend(convert_adjust_or_font_variations(serde_val).into_iter());
                    },
                    // Is it possible to match against string pattern here?
                    _ => {
                        let lang_id_vector = convert_lang_script(string_ref);
                        let script_specific_family_name_candidate = convert_string_value(serde_val);
                        if script_specific_family_name_candidate.is_none() {
                            return result;
                        }

                        lang_id_to_family_vec.extend(lang_id_vector.into_iter().map(|lang_id| {
                            let script_specific_family_name =
                                script_specific_family_name_candidate.clone().unwrap();
                            (lang_id, script_specific_family_name)
                        }));
                    },
                }
            }
            result.extend(lang_id_to_family_vec.into_iter().map(|lang_id_to_family| {
                FallbackEntryOHOS {
                    lang_id_to_family,
                    font_variations: font_variations.clone(),
                }
            }));
        },
        _ => {
            log::warn!("Unexpected value in fallback entry");
        },
    }
    result
}

#[inline]
fn convert_fallback_array_entry(
    obj_entry: &serde_json::Value,
) -> Option<(String, Vec<FallbackEntryOHOS>)> {
    // obj_entry is entry in config["fallback"] array
    // Object(Map<String, Value>)
    // "IMPORTANT_IN_FONTCONFIG_EMPTY_STR_HERE": {
    //     "font-variations": [
    //         {
    //           "weight": 300, "wght": 247
    //         },
    //         {
    //           "weight": 400, "wght": 400
    //         },
    //         {
    //           "weight": 900, "wght": 844
    //         }
    //       ],
    //     "und-Arab": "HarmonyOS Sans Naskh Arabic UI"
    // }
    match obj_entry {
        Value::Object(map) => {
            if let Some((string_ref, serde_val)) = map.iter().next() {
                if !serde_val.is_array() {
                    log::warn!("Unexpected value of fallback entry object in fontconfig.json");
                    return None;
                }
                // unwrap is safe because of check above | code is panic free
                let serde_val = serde_val.as_array().unwrap();

                let data: Vec<FallbackEntryOHOS> = serde_val
                    .iter()
                    .flat_map(convert_fallback_array_entry_value)
                    .collect();
                return Some((string_ref.to_string(), data));
            }
            None
        },
        _ => {
            log::warn!("Unexpected value inside entry of fallback array");
            None
        },
    }
}

fn convert_and_verify_fallback(
    serde_val: &serde_json::Value,
    verified_font_files: &[(String, String)],
) -> Vec<(String, Vec<FallbackEntryOHOS>)> {
    // serde_val is config["fallback"]

    // convert to more convenient Rust Structure
    let fallback = match serde_val {
        Value::Array(array) => array
            .iter()
            .filter_map(convert_fallback_array_entry)
            .collect(),
        _ => {
            log::warn!("Unexpected value in fallback entry");
            Vec::<(String, Vec<FallbackEntryOHOS>)>::new()
        },
    };

    let verify_fallback_family = |entry: &FallbackEntryOHOS| -> Option<FallbackEntryOHOS> {
        let (_lang_script, font_family) = &entry.lang_id_to_family;
        if verified_font_files
            .iter()
            .find(|(font_full_name, _font_file_name)| font_full_name.contains(font_family))
            .is_some()
        {
            return Some(entry.clone());
        }
        None
    };

    fallback
        .into_iter()
        .filter_map(|entry: (String, Vec<FallbackEntryOHOS>)| {
            let (name, fallback_fonts) = entry;
            let valid_fallback_fonts: Vec<FallbackEntryOHOS> = fallback_fonts
                .iter()
                .filter_map(verify_fallback_family)
                .collect();
            if valid_fallback_fonts.is_empty() {
                return None;
            }
            Some((name, valid_fallback_fonts))
        })
        .collect()
}

// ##########################################################################
// Block of functions that is responsible for FontconfigOHOS::font_file_map #
// ##########################################################################
#[inline]
fn convert_font_file_map_entry(obj_entry: &serde_json::Value) -> Option<(String, String)> {
    // serde_val is entry in config["font_file_map"] array
    // serde_val example:
    // {
    // "DejaVuMathTeXGyre-Regular": "DejaVuMathTeXGyre.ttf"
    // },
    match obj_entry {
        Value::Object(map) => {
            let mut map_iter = map.iter();
            if let Some((string_ref, serde_val)) = map_iter.next() {
                let data = convert_string_value(serde_val);
                let next_entry = map_iter.next();
                if next_entry.is_some() {
                    log::warn!(
                        "Single entry that should contains {{family}} : {{path}} expected here"
                    );
                    return None;
                }
                if let Some(data) = data {
                    return Some((string_ref.to_string(), data));
                } else {
                    return None;
                }
            }
            None
        },
        _ => {
            log::warn!(
                "Entry in font_file_map array should be representable as Object(Map<String,Value>)"
            );
            None
        },
    }
}

fn convert_and_verify_font_file_map(
    serde_val: &serde_json::Value,
    found_device_fonts_names: &[PathBuf],
) -> Option<Vec<(String, String)>> {
    // serde_val is config["font_file_map"]
    // serde_val example:
    // "font_file_map" : [
    //     {
    //     "DejaVuMathTeXGyre-Regular": "DejaVuMathTeXGyre.ttf"
    //     },
    //     {
    //     "FTSymbol": "FTSymbol.ttf"
    //     }
    // ]

    // Improve code by replacing String to &str with correct lifetimes evrywhere in this function
    //
    // TODO(ddesyatkin): convert to more convenient Rust Structure Vec<(String, String)> lacks performance
    // Find good realizations of MultiMap and MultiSet (Hash based)
    // for Rust language. I have not considered the fact that actual object may have 2 identical keys
    // because in original JSON this is the entries of 2 different JSON_OBJECTS

    // TODO(symlinks): Remember that I included symlinks in enumerate_font_files()
    // If some software down the line couldn't handle symlinks, then we must fix it by rollback.
    let mut result = Vec::<(String, String)>::new();
    match serde_val {
        Value::Array(array) => {
            result.extend(array.iter().filter_map(convert_font_file_map_entry));
        },
        _ => {
            log::warn!("font_file_map should be representable as array");
        },
    }

    if result.is_empty() {
        return None;
    }

    let found_device_fonts_names_paths_map: HashMap<String, String> = found_device_fonts_names
        .iter()
        .filter_map(|file_path| -> Option<(String, String)> {
            let name = file_path.file_name()?.to_str()?.to_string();
            let path = file_path.to_str()?.to_string();
            Some((name, path))
        })
        .collect();

    let found_device_fonts_names_set: HashSet<&str> = found_device_fonts_names_paths_map
        .iter()
        .map(|(name, _path)| name.as_str())
        .collect();
    let config_font_names_set: HashSet<&str> = result
        .iter()
        .map(|(_full_name, font_file_name)| font_file_name.as_str())
        .collect();

    let correctly_defined_fonts = config_font_names_set.intersection(&found_device_fonts_names_set);
    let unspecified_fonts = found_device_fonts_names_set.difference(&config_font_names_set);
    if log::log_enabled!(log::Level::Debug) {
        let errors_in_config = config_font_names_set.difference(&found_device_fonts_names_set);

        if unspecified_fonts.clone().count() != 0 {
            log::warn!(
                r#"
                We found some fonts that is not specified in fontconfig!
                It could be normal cause you may specify symlinks instead of actual files!
                Please check the contents of fontdir folder stated in config!
                If there is symlinks you can ignore this warning.
                If you just placed some additional fonts into folder, please specify them in fontconfig.json!

                In servo engine this fonts would be considered as installed fonts.
            "#
            );

            for font in unspecified_fonts.clone() {
                log::warn!("{}", font);
            }
        }

        if errors_in_config.clone().count() != 0 {
            log::warn!(
                r#"
                Some entries that describe fonts that is not present on device found in fontconfig.json!
                Please check fontconfig fontdir folder and add files that is listed bellow:
            "#
            );
            for font in errors_in_config {
                log::warn!("{}", font);
            }
        }
    }

    let correct_subset: HashSet<String> = correctly_defined_fonts
        .map(|entry| -> String { entry.to_string() })
        .collect();

    let unspecified_subset: HashSet<String> = unspecified_fonts
        .map(|entry| -> String { entry.to_string() })
        .collect();

    if correct_subset.is_empty() {
        return None;
    }

    result = result
        .into_iter()
        .filter_map(|entry| {
            let (key, ref value) = entry;
            if correct_subset.contains(value) || unspecified_subset.contains(value) {
                Some((key, found_device_fonts_names_paths_map[value].clone()))
            } else {
                None
            }
        })
        .collect();

    // TODO(ddesyatkin) create special procedure for unspecified fonts
    // We must parse the files and extract info about family names, scripts, unicode ranges
    // for FONT_LIST.

    if log::log_enabled!(log::Level::Debug) {
        log::debug!("OHOS fontconfig provides following correctly specified fonts:");
        for font in correct_subset {
            log::debug!("{}", font);
        }
        log::debug!("OHOS fontconfig converted and verified font_file_map:");
        for (font_family, font_path) in result.clone() {
            log::debug!("{} : {}", font_family, font_path);
        }
    }

    Some(result)
}

// ########################################################################
// Main parsing function. Represents public interface to contents of file #
// ########################################################################
pub(super) fn load_and_verify_ohos_fontconfig() -> Option<(FontconfigOHOS, Vec<PathBuf>)> {
    let mut content_string = String::new();
    let contents = fs::read_to_string(FONT_CONFIG_PATH);
    match contents {
        Err(e) => {
            log::warn!(
                r#"
                Unable to read OpenHarmony fontconfig
                fs::read_to_string returned following error {}
            "#,
                e
            );
            return None;
        },
        Ok(result_string) => {
            content_string.extend(result_string.chars());
        },
    }

    let config: serde_json::Value;
    match serde_json::from_str(&content_string) {
        Err(e) => {
            log::warn!(
                r#"
            Unable to deserialize OpenHarmony fontconfig!
            serde_json produced following error {}
            "#,
                e
            );
            return None;
        },
        Ok(data) => config = data,
    };
    // Prohibit further modifications.
    let config = config;
    // Fontconfig is loaded.
    // Now we should verify it before providing it to user.
    if let Some((fontdir, found_font_files)) = convert_and_verify_fontdir(&config["fontdir"]) {
        if log::log_enabled!(log::Level::Debug) {
            log::warn!("OHOS fontconfig converted fontdir:");
            for directory in &fontdir {
                log::warn!("{:?}", directory);
            }
        }
        if let Some(font_file_map) =
            convert_and_verify_font_file_map(&config["font_file_map"], &found_font_files)
        {
            let generic = convert_and_verify_generic(&config["generic"]);
            let fallback = convert_and_verify_fallback(&config["fallback"], &font_file_map);
            if log::log_enabled!(log::Level::Debug) {
                log::warn!("OHOS fontconfig converted and verified generic:");
                for font in &generic {
                    log::warn!("{:?}", font);
                }
                log::warn!("OHOS fontconfig converted and verified fallback:");
                for font in &fallback {
                    log::warn!("{:?}", font);
                }
            }

            let result = FontconfigOHOS {
                fontdir,
                generic,
                fallback,
                font_file_map,
            };

            return Some((result, found_font_files));
        }
        log::error!(
            r#"
                OHOS font_file_map value is incorrect in config! JSON TEXT should contain array of JSON_OBJECT!
            "#
        );
    }
    log::error!(
        r#"
            OHOS fontdir value is incorrect in config! JSON TEXT should contain array of JSON_STRING!
        "#
    );
    None
}

// TODO(ddesyatkin): Add propper tests for all json convertion and verification functions;
