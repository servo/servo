/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;
use std::convert::TryFrom;
use std::error::Error;
use std::result::Result as StdResult;
use std::string::String;
use std::vec::Vec;
use std::{fmt, fs, panic};

use itertools::Itertools;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Number, Result, Value};

static FONT_CONFIG_PATH: &str = "/etc/fontconfig.json";

// Think do I really want to introduce this structures here?
// I want to do it only if it is possible to organize tight interreactions with
// serde_json crate. And will be able to directly Serialize Custom structures to the
// same format as fontconfig.json file

/// Represents individual entry in vector of generic font families supported by
/// OpenHarmony

#[derive(Debug)]
pub struct GenericFontFamilyOHOS {
    pub family: String,
    pub alias: Vec<(String, i32)>,
    pub adjust: Vec<[(String, i32); 2]>,
    pub font_variations: Vec<[(String, i32); 2]>,
}

/// Represents individual entry in named fallback
#[derive(Debug)]
pub struct FallbackEntry {
    /// lang_srcipt contains pair of language-script value of content
    /// and family name for it visual representation
    /// ("zh-Hans": "HarmonyOS Sans SC")
    pub lang_script: HashMap<String, String>,
    /// font-variations allow to setup several styles that fallback font
    /// family may support; Currently used only for font-weight values!
    pub font_variations: Vec<[(String, i32); 2]>,
}

/// Representation of OpenHarmony /etc/fontconfig.json
/// in Rust structure
#[derive(Debug)]
pub struct FontconfigOHOS {
    pub fontdir: String,
    pub generic: Vec<GenericFontFamilyOHOS>,
    pub fallback: Vec<(String, Vec<FallbackEntry>)>,
    pub font_file_map: HashMap<String, String>,
}

// #########################################################################
// Block of structures and functions that is responsible for Error Hndling #
// #########################################################################

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

// ###########################################################################
// Block of functions that is responsible for FontconfigOHOS::generic::alias #
// ###########################################################################
#[inline]
fn parse_alias_entry(alias_obj_entry: &serde_json::Value) -> Option<(String, i32)> {
    let alias_obj_entry_obj = alias_obj_entry
        .as_object()
        .expect("Entry in allias array should be representable as Object(Map<String,Value>)");

    if let Some((alias_str, alias_val)) = alias_obj_entry_obj.iter().next() {
        let alias_val: i64 = alias_val
            .as_i64()
            .expect("Unexpected value of generic::alias in fontconfig.json");
        let alias_val = i32::try_from(alias_val);
        match alias_val {
            Err(e) => {
                log::warn!("Alias value in {:?} overflows i32", FONT_CONFIG_PATH);
                return None;
            },
            Ok(data) => return Some((alias_str.to_string(), data)),
        }
    }
    None
}

fn parse_alias(alias_val: &serde_json::Value) -> Vec<(String, i32)> {
    let alias_obj = alias_val
        .as_array()
        .expect("Alias value should be array representable");
    alias_obj.iter().filter_map(parse_alias_entry).collect()
}

// ############################################################################
// Block of functions that is responsible for FontconfigOHOS::generic::adjust #
// and FontconfigOHOS::generic::font_variation                                #
// ############################################################################
#[inline]
fn parse_adjust_or_fv_entry(obj_entry: &serde_json::Value) -> Option<[(String, i32); 2]> {
    let obj_entry_obj = obj_entry
        .as_object()
        .expect("Entry in adjust of font-variations array should be representable as Object(Map<String,Value>)");

    let mut obj_entry_iter = obj_entry_obj.iter();

    let mut first: Option<(String, i32)> = None;
    let mut second: Option<(String, i32)> = None;

    if let Some((string_ref, serde_val)) = obj_entry_iter.next() {
        let serde_val: i64 = serde_val.as_i64().expect(
            "Unexpected value of generic::alias or generic::font-variations in fontconfig.json",
        );
        let serde_val = i32::try_from(serde_val);
        first = match serde_val {
            Err(e) => None,
            Ok(data) => Some((string_ref.to_string(), data)),
        }
    }

    if let Some((string_ref, serde_val)) = obj_entry_iter.next() {
        let serde_val: i64 = serde_val.as_i64().expect(
            "Unexpected value of generic::alias or generic::font-variations in fontconfig.json",
        );
        let serde_val = i32::try_from(serde_val);
        second = match serde_val {
            Err(e) => None,
            Ok(data) => Some((string_ref.to_string(), data)),
        }
    }

    match (first, second) {
        (Some(first_internal), Some(second_internal)) => Some([first_internal, second_internal]),
        (_, _) => None,
    }
}

fn parse_adjust_or_font_variations(serde_val: &serde_json::Value) -> Vec<[(String, i32); 2]> {
    let serde_obj = serde_val
        .as_array()
        .expect("adjust or font-variation value should be array representable");
    serde_obj
        .iter()
        .filter_map(parse_adjust_or_fv_entry)
        .collect()
}

// #####################################################################
// Block of functions that is responsible for FontconfigOHOS::fallback #
// #####################################################################
#[inline]
fn parse_fallback_array_entry_value(obj_entry: &serde_json::Value) -> Option<FallbackEntry> {
    // obj_entry is Value is
    // "font-variations": [
    //         {
    //           "weight": 100, "wght": 100
    //         },
    // ]
    // or
    // "und-Arab": "HarmonyOS Sans Naskh Arabic UI"

    let parse_lang_script = |entry: (&String, &Value)| -> Option<[String; 2]> {
        let (string_ref, serde_val) = entry;
        match serde_val {
            Value::String(value_string) => Some([string_ref.to_string(), value_string.to_string()]),
            // No log here
            // conflicts with parse_font_variations
            _ => None,
        }
    };

    let parse_font_variations = |entry: (&String, &Value)| -> Option<Vec<[(String, i32); 2]>> {
        let (string_ref, serde_val) = entry;
        if !string_ref.contains("font-variations") {
            return None;
        }

        match serde_val {
            Value::Array(array) => {
                Some(array.iter().filter_map(parse_adjust_or_fv_entry).collect())
            },
            // No log here
            // conflicts with parse_lang_script
            _ => None,
        }
    };

    match obj_entry {
        Value::Object(map) => {
            let mut lang_script = HashMap::<String, String>::new();
            for entry in map.iter().filter_map(parse_lang_script) {
                lang_script.insert(entry[0].clone(), entry[1].clone());
            }
            // TODO(ddesyatkin): find better realization for font_variations
            let mut font_variations = Vec::<[(String, i32); 2]>::new();
            for entry in map.iter().filter_map(parse_font_variations) {
                font_variations = entry;
            }
            Some(FallbackEntry {
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
fn parse_fallback_array_entry(
    obj_entry: &serde_json::Value,
) -> Option<(String, Vec<FallbackEntry>)> {
    // obj_entry is Object(Map<String, Value>)
    // "IMPORTANT_IN_FONTCONFIG_EMPTY_STR_HERE": {
    //     "font-variations": [
    //         {
    //           "weight": 100, "wght": 100
    //         },
    //         {
    //           "weight": 300, "wght": 247
    //         },
    //         {
    //           "weight": 400, "wght": 400
    //         },
    //         {
    //           "weight": 500, "wght": 500
    //         },
    //         {
    //           "weight": 700, "wght": 706
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
                let serde_val = serde_val
                    .as_array()
                    .expect("Unexpected value of fallback entry object in fontconfig.json");

                let data: Vec<FallbackEntry> = serde_val
                    .iter()
                    .filter_map(parse_fallback_array_entry_value)
                    .collect();
                // This return is fine cause we should have only one entry in this map,
                // because this function will be applied to each element independently
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

fn parse_fallback(serde_val: &serde_json::Value) -> Vec<(String, Vec<FallbackEntry>)> {
    // serde_val is config["fallback"]
    match serde_val {
        Value::Array(array) => array
            .iter()
            .filter_map(parse_fallback_array_entry)
            .collect(),
        _ => {
            log::warn!("Unexpected value in fallback entry");
            Vec::<(String, Vec<FallbackEntry>)>::new()
        },
    }
}

// ##########################################################################
// Block of functions that is responsible for FontconfigOHOS::font_file_map #
// ##########################################################################
#[inline]
fn parse_font_file_map_entry(obj_entry: &serde_json::Value) -> Option<[String; 2]> {
    let obj_entry_obj = obj_entry.as_object().expect(
        "Entry in font_file_map array should be representable as Object(Map<String,Value>)",
    );

    if let Some((string_ref, serde_val)) = obj_entry_obj.iter().next() {
        let serde_val = serde_val
            .as_str()
            .expect("Unexpected value of font_file_map in fontconfig.json");

        return Some([string_ref.to_string(), serde_val.to_string()]);
    }
    None
}

fn parse_font_file_map(serde_val: &serde_json::Value) -> HashMap<String, String> {
    // serde_val is config["font_file_map"]
    let mut result = HashMap::<String, String>::new();
    if let Some(serde_obj) = serde_val.as_array() {
        let array_of_valid_entries: Vec<[String; 2]> = serde_obj
            .iter()
            .filter_map(parse_font_file_map_entry)
            .collect();
        for entry in array_of_valid_entries.into_iter() {
            result.insert(entry[0].clone(), entry[1].clone());
        }
    }
    result
}

// ########################################################################
// Main parsing function. Represents public interface to contents of file #
// ########################################################################
pub(super) fn parse_ohos_fontconfig() -> Option<FontconfigOHOS> {
    let contents =
        fs::read_to_string(FONT_CONFIG_PATH).expect("Succsessfully read OpenHarmony fontconfig");
    let config: serde_json::Value =
        serde_json::from_str(&contents).expect("OpenHarmony fontconfig deserialized!");

    log::warn!("test config parse: {:#?}", config["fontdir"]);
    log::warn!("test config parse: {:#?}", config["generic"]);
    log::warn!("test config parse: {:#?}", config["fallback"]);
    log::warn!("test config parse: {:#?}", config["font_file_map"]);

    // iterate through alias
    log::warn!("test config parse: {:#?}", config["generic"]["alias"]);

    let fontdir: String = config["fontdir"].to_string();

    let mut generic = Vec::<GenericFontFamilyOHOS>::new();
    if let Some(generic_obj) = config["generic"].as_array() {
        for family_obj in generic_obj.iter() {
            let family = family_obj["family"]
                .as_str()
                .expect("Unexpected value of generic::family in fontconfig.json")
                .to_string();
            let alias = parse_alias(&family_obj["alias"]);
            // let adjust = parse_adjust_or_font_variations(&family_obj["adjust"]);
            // let font_variations = parse_adjust_or_font_variations(&family_obj["font-variations"]);
            let adjust = Vec::<[(String, i32); 2]>::new();
            let font_variations = Vec::<[(String, i32); 2]>::new();
            generic.push({
                GenericFontFamilyOHOS {
                    family,
                    alias,
                    adjust,
                    font_variations,
                }
            });
        }
    }

    let fallback = parse_fallback(&config["fallback"]);
    let mut font_file_map = parse_font_file_map(&config["font_file_map"]);

    Some(FontconfigOHOS {
        fontdir,
        generic,
        fallback,
        font_file_map,
    })
}

// TODO(ddesyatkin): Add propper tests for all parsing functions;
// 2 variants correct and incorrect input;

#[test]
fn test_generic_font_family_ohos_serialize() {
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
          "HarmonyOS-Sans-Medium": 700
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
          "weight": 300, "wght": 247
        },
        {
          "weight": 400, "wght": 400
        },
        {
          "weight": 500, "wght": 500
        },
        {
          "weight": 700, "wght": 706
        },
        {
          "weight": 900, "wght": 844
        }
      ]
    }"#;
    let v: serde_json::Value = serde_json::from_str(data)?;

    println!(
        "Please call {} at the number {}",
        v["family"], v["adjust"][0]
    );

    Ok(())
}
