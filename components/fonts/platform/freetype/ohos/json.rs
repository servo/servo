/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;
use std::error::Error;
use std::path::PathBuf;
use std::result::Result as StdResult;
use std::string::String;
use std::vec::Vec;
use std::{fmt, fs, panic};

use itertools::Itertools;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Number, Result, Value};

use super::font_list::enumerate_font_files;

static FONT_CONFIG_PATH: &str = "/etc/fontconfig.json";

// This file contains custom functions for json parsing to avoid
// blind reliance on serde_json::from_*. Even if user make some mistake in
// fontconfig we want to inform him about the problem, but rest of usefull information
// should be preserved and used in servo engine.

/// Represents individual entry in vector of generic font families supported by
/// OpenHarmony
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(super) struct GenericFontFamilyOHOS {
    pub family: String,
    pub alias: Vec<(String, i32)>,
    pub adjust: Vec<[(String, i32); 2]>,
    pub font_variations: Vec<[(String, i32); 2]>,
}

/// Represents individual entry in named fallback
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(super) struct FallbackEntryOHOS {
    /// lang_srcipt contains pair of language-script value of content
    /// and family name for it visual representation
    /// ("zh-Hans": "HarmonyOS Sans SC")
    pub lang_script: HashMap<String, String>,
    // Hashmap instead of Vec<[String; 2]> is used because I understood
    // that I want to rewrite file to serde compatible structures parse_ohos_fontconfig
    // should be interchangeble with serde_json::from_string
    /// font-variations allow to setup several styles that fallback font
    /// family may support; Currently used only for font-weight values!
    pub font_variations: Vec<[(String, i32); 2]>,
}

/// Representation of OpenHarmony fontconfig.json
/// in Rust structure
#[derive(Debug, Deserialize, Serialize)]
pub(super) struct FontconfigOHOS {
    pub fontdir: Vec<String>,
    pub generic: Vec<GenericFontFamilyOHOS>,
    pub fallback: Vec<(String, Vec<FallbackEntryOHOS>)>,
    pub font_file_map: HashMap<String, String>,
}

// ##########################################################################
// Block of structures and functions that is responsible for Error Handling #
// ##########################################################################

// TODO(ddesyatkin): Rewrite everything to make module return recoverable errors instead
// of Option.
//
// Maybe preserve error messages in stack?
//
// struct FontconfigOHOSParsingError {
//     kind: FontconfigOHOSParsingErrorKind,
//     recoverable_data: RecoverableData
// }
// bitflags! {
//     struct FontconfigOHOSParsingErrorKind: u8 {
//         const FONT_DIR_PARSING_ERROR = 1 << 0;
//         const GENERIC_FONT_FAMILY_PARSING_ERROR = 1 << 2;
//         const FALLBACK_PARSING_ERROR = 1 << 3;
//         const FONT_FILE_MAP_PARSING_ERROR = 1 << 4;
//     }
// }
// enum RecoverableData {
// Fontconfig(Box<FontconfigOHOS>),
// GenericFamilies(Box<Vec<GenericFontFamilyOHOS>>),
// GenericFamily(Box<GenericFontFamilyOHOS>),
// Fallback(Box<Vec<(String, Vec<FallbackEntryOHOS>)>>),
// FontFileMap(Box<HashMap<String, String>>)
//
// }

#[derive(Debug)]
struct FontconfigOHOSParsingError {
    kind: FontconfigOHOSParsingErrorKind,
}

impl fmt::Display for FontconfigOHOSParsingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FontconfigOHOSParsingError")
    }
}

impl Error for FontconfigOHOSParsingError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.kind)
    }
}

#[derive(Debug)]
enum FontconfigOHOSParsingErrorKind {
    PlaceholderError,
    PlaceholderError1,
    FontDirParsingError,
    GenericFontFamilyParsingError,
    FallbackParsingError,
    FontFileMapParsingError,
}

impl fmt::Display for FontconfigOHOSParsingErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FontconfigOHOSParsingErrorKind::PlaceholderError => {
                write!(f, "PlaceholderError")
            },
            FontconfigOHOSParsingErrorKind::PlaceholderError1 => {
                write!(f, "PlaceholderError1")
            },
            FontconfigOHOSParsingErrorKind::FontDirParsingError => {
                write!(f, "FontDirParsingError")
            },
            FontconfigOHOSParsingErrorKind::GenericFontFamilyParsingError => {
                write!(f, "GenericFontFamilyParsingError")
            },
            FontconfigOHOSParsingErrorKind::FallbackParsingError => {
                write!(f, "FallbackParsingError")
            },
            FontconfigOHOSParsingErrorKind::FontFileMapParsingError => {
                write!(f, "FontFileMapParsingError")
            },
        }
    }
}

impl Error for FontconfigOHOSParsingErrorKind {}

// TODO(ddesyatkin): Rewrite functions bellow to make them panic free!
// We should just return None instead of the config if we ever meet some
// irrecoverable error

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
                        log::warn!("Value in {:?} overflows i32", FONT_CONFIG_PATH);
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
        Value::String(string) => {
            return Some(string.to_string());
        },
        _ => {
            log::warn!("Unexpected value in supposedly string value");
            None
        },
    }
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
            log::warn!("Entry in adjust of font-variations array should be representable as Object(Map<String,Value>)");
            None
        },
    }
}

// ####################################################################
// Block of functions that is responsible for FontconfigOHOS::fontdir #
// ####################################################################
fn detect_fonts_specified_in_fontdir(paths: &Vec<String>) -> Vec<PathBuf> {
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
    // "alias": [
    //     {
    //       "HarmonyOS-Sans": 0
    //     },
    //     {
    //       "HarmonyOS-Sans-Light": 100
    //     }
    // ]

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
fn convert_fallback_array_entry_value(obj_entry: &serde_json::Value) -> Option<FallbackEntryOHOS> {
    // obj_entry example
    // "font-variations": [
    //         {
    //           "weight": 100, "wght": 100
    //         },
    // ]
    // or
    // "und-Arab": "HarmonyOS Sans Naskh Arabic UI"
    match obj_entry {
        Value::Object(map) => {
            let mut lang_script = HashMap::<String, String>::new();
            let mut font_variations = Vec::<[(String, i32); 2]>::new();
            for (string_ref, serde_val) in map.iter() {
                match string_ref.as_str() {
                    "font-variations" => {
                        font_variations
                            .extend(convert_adjust_or_font_variations(serde_val).into_iter());
                    },
                    _ => {
                        // TODO (ddesyatkin)
                        // check_valid_lang_script(string_ref)
                        let data = convert_string_value(serde_val);
                        if let Some(data) = data {
                            lang_script.insert(string_ref.to_string(), data);
                        }
                    },
                }
            }
            Some(FallbackEntryOHOS {
                lang_script,
                font_variations,
            })
        },
        _ => {
            log::warn!("Unexpected value in fallback entry");
            None
        },
    }
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
                    .filter_map(convert_fallback_array_entry_value)
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
    verified_font_files: &HashMap<String, String>,
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

    let convert_and_verify_fallback = |entry: &FallbackEntryOHOS| -> Option<FallbackEntryOHOS> {
        let mut iter = entry.lang_script.values();
        let value = iter.next();
        if iter.next().is_some() {
            // Call once log bellow
            log::warn!(
                r#"
                Problem on FontconfigOHOS verification.
                Despite lang_script is HasMap logically it is pair.
                We expect single <String>, <String> value here.
                Additional values will be discarded.
            "#
            );
        }
        if let Some(value) = value {
            if verified_font_files.contains_key(value) {
                return Some(entry.clone());
            }
        }
        None
    };

    fallback
        .into_iter()
        .filter_map(|entry: (String, Vec<FallbackEntryOHOS>)| {
            let (name, fallback_fonts) = entry;
            let valid_fallback_fonts: Vec<FallbackEntryOHOS> = fallback_fonts
                .iter()
                .filter_map(convert_and_verify_fallback)
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

                return Some((string_ref.to_string(), serde_val.to_string()));
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
    found_device_fonts_names: &Vec<PathBuf>,
) -> Option<HashMap<String, String>> {
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

    // convert to more convenient Rust Structure
    let mut result = HashMap::<String, String>::new();
    match serde_val {
        Value::Array(array) => {
            let iter_of_valid_entries = array.iter().filter_map(convert_font_file_map_entry);
            result.extend(iter_of_valid_entries);
        },
        _ => {
            log::warn!("font_file_map should be representable as array");
        },
    }

    if result.is_empty() {
        return None;
    }

    let found_device_fonts_names_set: HashSet<String> = found_device_fonts_names
        .iter()
        .filter_map(|file_path| -> Option<String> {
            Some(file_path.file_name()?.to_str()?.to_string())
        })
        .collect();
    // Aggregated all font_files under all paths
    // now we should compare values with one parsed from fontconfig.json
    let config_font_names_set: HashSet<String> = result
        .values()
        .map(|entry| -> String {
            // TODO (ddesyatkin): Decide what is the right place for that replacement operation
            // json convertion or verification.
            let entry = entry.replace(&['"'][..], "");
            entry.to_string()
        })
        .collect();
    let unspecified_fonts = found_device_fonts_names_set.difference(&config_font_names_set);
    let errors_in_config = config_font_names_set.difference(&found_device_fonts_names_set);
    let correctly_defined_fonts = config_font_names_set.intersection(&found_device_fonts_names_set);

    if unspecified_fonts.clone().count() != 0 {
        log::warn!(
            r#"
            We found some fonts that is not specified in fontconfig!
            It could be normal cause you may specify symlinks instead of actual files!
            Please check the contents of fontdir folder stated in config!
            If there is symlinks you can ignore this warning.
            If you just placed some additional fonts into folder, please specify them in fontconfig.json!

            Found device font that doesn't described in OHOS fontconfig:
        "#
        );

        for font in unspecified_fonts {
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

    log::debug!("OHOS fontconfig provides following correctly specified fonts:");
    for font in correctly_defined_fonts.clone() {
        log::debug!("{}", font);
    }

    let correct_subset: HashSet<String> = correctly_defined_fonts
        .map(|entry| -> String { entry.to_string() })
        .collect();

    if correct_subset.is_empty() {
        return None;
    }

    result = result
        .into_iter()
        .filter_map(|entry| {
            let (ref key, ref value) = entry;
            if correct_subset.contains(value) {
                Some(entry)
            } else {
                None
            }
        })
        .collect();

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

    let mut config: serde_json::Value = Value::Null;
    match serde_json::from_str(&content_string) {
        Err(e) => {
            log::warn!(
                r#"
            Unable to deserialize OpenHarmony fontconfig!
            serde_json produced following error {}
            "#,
                e
            );
        },
        Ok(data) => config = data,
    };
    // Prohibit further modifications.
    let config = config;
    // Fontconfig is loaded.
    // Now we should verify it before providing it to user.

    if let Some((fontdir, found_font_files)) = convert_and_verify_fontdir(&config["fontdir"]) {
        // Config is completely useless for us if we don't know where to search fonts...
        // Should I consider return Err with partial inforamtion here,
        // And use it to resolve fonts that we will search on some predefined paths?
        if let Some(font_file_map) =
            convert_and_verify_font_file_map(&config["font_file_map"], &found_font_files)
        {
            let generic = convert_and_verify_generic(&config["generic"]);
            let fallback = convert_and_verify_fallback(&config["fallback"], &font_file_map);

            let result = FontconfigOHOS {
                fontdir,
                generic,
                fallback,
                font_file_map,
            };

            match serde_json::to_string(&result) {
                Err(e) => {
                    log::warn!("Life is hard {}", e);
                },
                Ok(data) => {
                    log::warn!("Test Serialization {:#?}", data);
                },
            };

            return Some((result, found_font_files));
        }
    }
    None
}

// TODO(ddesyatkin): Add propper tests for all parsing functions;
// 2 variants correct and incorrect input;

#[test]
fn test_generic_generic_font_family_ohos_serde_parsing() {
    let data = r#"{
        "family": "HarmonyOS Sans",
        "alias": [
          {
            "HarmonyOS-Sans": 0
          },
          {
            "HarmonyOS-Sans-Light": 100
          },
          {
            "HarmonyOS-Sans-Regular": 400
          },
          {
            "HarmonyOS-Sans-Bold": 900
          }
        ],
        "adjust": [
          {
            "weight": 50, "to": 100
          },
          {
            "weight": 80, "to": 400
          },
          {
            "weight": 100, "to": 700
          },
          {
            "weight": 200, "to": 900
          }
        ],
        "font-variations": [
          {
            "weight": 100, "wght": 100
          },
          {
            "weight": 400, "wght": 400
          },
          {
            "weight": 900, "wght": 844
          }
        ]
      }"#;
    let family_object: GenericFontFamilyOHOS = serde_json::from_str(data)?;
}

#[test]
fn test_generic_generic_font_family_ohos_custom_parsing() {
    let data = r#"{
      "family": "HarmonyOS Sans",
      "alias": [
        {
          "HarmonyOS-Sans": 0
        },
        {
          "HarmonyOS-Sans-Light": 100
        },
        {
          "HarmonyOS-Sans-Regular": 400
        },
        {
          "HarmonyOS-Sans-Bold": 900
        }
      ],
      "adjust": [
        {
          "weight": 50, "to": 100
        },
        {
          "weight": 80, "to": 400
        },
        {
          "weight": 100, "to": 700
        },
        {
          "weight": 200, "to": 900
        }
      ],
      "font-variations": [
        {
          "weight": 100, "wght": 100
        },
        {
          "weight": 400, "wght": 400
        },
        {
          "weight": 900, "wght": 844
        }
      ]
    }"#;
    let mock_family_string = "HarmonyOS Sans".to_string();
    let mock_alias_object = Vec::<(String, i32)>::new();
    mock_alias_object.push(("HarmonyOS-Sans", 0));
    mock_alias_object.push(("HarmonyOS-Sans-Light", 100));
    mock_alias_object.push(("HarmonyOS-Sans-Regular", 400));
    mock_alias_object.push(("HarmonyOS-Sans-Bold", 900));
    let mock_adjust_object = Vec::<[(String, i32); 2]>::new();
    mock_adjust_object.push([("weight", 50), ("to", 100)]);
    mock_adjust_object.push([("weight", 80), ("to", 400)]);
    mock_adjust_object.push([("weight", 100), ("to", 700)]);
    mock_adjust_object.push([("weight", 200), ("to", 900)]);
    let mock_font_variations_object = Vec::<[(String, i32); 2]>::new();
    mock_font_variations_object.push([("weight", 100), ("wght", 100)]);
    mock_font_variations_object.push([("weight", 400), ("wght", 400)]);
    mock_font_variations_object.push([("weight", 900), ("wght", 844)]);

    let mock_family_object = GenericFontFamilyOHOS {
        family: mock_family_string,
        alias: mock_alias_object,
        adjust: mock_adjust_object,
        font_variations: mock_font_variations_object,
    };

    let family_object: serde_json::Value = serde_json::from_str(data)?;
    let family = family_object["family"]
        .as_str()
        .expect("Unexpected value of generic::family in fontconfig.json")
        .to_string();
    let alias = convert_alias(&family_object["alias"]);
    let adjust = convert_adjust_or_font_variations(&family_object["adjust"]);
    let font_variations = convert_adjust_or_font_variations(&family_object["font-variations"]);
    GenericFontFamilyOHOS {
        family,
        alias,
        adjust,
        font_variations,
    };

    println!(
        "Please call {} at the number {}",
        v["family"], v["adjust"][0]
    );

    Ok(())
}
