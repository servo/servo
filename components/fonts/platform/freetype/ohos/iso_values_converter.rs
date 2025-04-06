/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// Proper locale handling
use std::str::FromStr;

use codes_iso_639::part_3::LanguageCode;
use codes_iso_15924::ScriptCode;
use icu_locid::subtags::{Language, Script};
use itertools::enumerate;

#[allow(unused)]
pub fn lang_code_to_u32(lang_code: LanguageCode) -> u32 {
    let mut value: u32 = 0;
    let mut ascii_lang_code = vec![0u8; 3];
    for (index, character) in enumerate(lang_code.chars()) {
        ascii_lang_code[index] = character.to_ascii_lowercase() as u8;
    }
    value |= (ascii_lang_code[0] as u32) << 16;
    value |= (ascii_lang_code[1] as u32) << 8;
    value |= ascii_lang_code[2] as u32;
    value
}

pub fn convert_language(language_subtag: Language) -> u32 {
    let mut value: u32 = 0;
    let mut ascii_lang_code = vec![0u8; 3];
    for (index, character) in enumerate(language_subtag.as_str().chars()) {
        ascii_lang_code[index] = character.to_ascii_lowercase() as u8;
    }
    value |= (ascii_lang_code[0] as u32) << 16;
    value |= (ascii_lang_code[1] as u32) << 8;
    value |= ascii_lang_code[2] as u32;
    value
}

// pub fn convert_language(language_subtag: Language) -> u32 {
//     let lang_code: Result<LanguageCode, _> = language_subtag.as_str().parse();
//     match lang_code {
//         Err(e) => {
//             log::warn!(
//                 r#"
//                 Unexpected problem in lang_code conversion:
//                 language_subtag: {}
//                 language code err {}"#,
//                 language_subtag.as_str(),
//                 e
//             );
//             lang_code_to_u32(LanguageCode::Und)
//         },
//         Ok(lang) => lang_code_to_u32(lang),
//     }
// }

pub fn convert_script(script_subtag: Option<Script>) -> u16 {
    if let Some(script_subtag) = script_subtag {
        let script_code = ScriptCode::from_str(script_subtag.as_str());
        match script_code {
            Err(_e) => return ScriptCode::Zzzz.numeric_code() as u16,
            Ok(script) => return script.numeric_code() as u16,
        }
    }
    ScriptCode::Zzzz.numeric_code() as u16
}
