/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use parsing::parse;
use style::properties::longhands::{font_feature_settings, font_weight};
use style::properties::longhands::font_feature_settings::SpecifiedValue;
use style::values::generics::{FontSettings, FontSettingTag, FontSettingTagInt};
use style_traits::ToCss;

#[test]
fn font_feature_settings_should_parse_properly() {
    use byteorder::{ReadBytesExt, BigEndian};
    use std::io::Cursor;

    let normal = parse_longhand!(font_feature_settings, "normal");
    let normal_computed = SpecifiedValue::Value(FontSettings::Normal);
    assert_eq!(normal, normal_computed);

    let mut a_d_bytes = Cursor::new(b"abcd");
    let mut e_h_bytes = Cursor::new(b"efgh");

    let abcd = a_d_bytes.read_u32::<BigEndian>().unwrap();
    let efgh = e_h_bytes.read_u32::<BigEndian>().unwrap();

    let on = parse_longhand!(font_feature_settings, "\"abcd\" on");
    let on_computed = SpecifiedValue::Value(FontSettings::Tag(vec![
        FontSettingTag { tag: abcd, value: FontSettingTagInt(1) }
    ]));
    assert_eq!(on, on_computed);

    let off = parse_longhand!(font_feature_settings, "\"abcd\" off");
    let off_computed = SpecifiedValue::Value(FontSettings::Tag(vec![
        FontSettingTag { tag: abcd, value: FontSettingTagInt(0) }
    ]));
    assert_eq!(off, off_computed);

    let no_value = parse_longhand!(font_feature_settings, "\"abcd\"");
    let no_value_computed = SpecifiedValue::Value(FontSettings::Tag(vec![
        FontSettingTag { tag: abcd, value: FontSettingTagInt(1) }
    ]));
    assert_eq!(no_value, no_value_computed);

    let pos_integer = parse_longhand!(font_feature_settings, "\"abcd\" 100");
    let pos_integer_computed = SpecifiedValue::Value(FontSettings::Tag(vec![
        FontSettingTag { tag: abcd, value: FontSettingTagInt(100) }
    ]));
    assert_eq!(pos_integer, pos_integer_computed);

    let multiple = parse_longhand!(font_feature_settings, "\"abcd\" off, \"efgh\"");
    let multiple_computed = SpecifiedValue::Value(FontSettings::Tag(vec![
        FontSettingTag { tag: abcd, value: FontSettingTagInt(0) },
        FontSettingTag { tag: efgh, value: FontSettingTagInt(1) }
    ]));
    assert_eq!(multiple, multiple_computed);
}

#[test]
fn font_feature_settings_should_throw_on_bad_input() {
    assert!(parse(font_feature_settings::parse, "").is_err());
    assert!(parse(font_feature_settings::parse, "\"abcd\" -1").is_err());
    assert!(parse(font_feature_settings::parse, "\"abc\"").is_err());
    assert!(parse(font_feature_settings::parse, "\"abcó\"").is_err());
}

#[test]
fn font_feature_settings_to_css() {
    assert_roundtrip_with_context!(font_feature_settings::parse, "normal");
    assert_roundtrip_with_context!(font_feature_settings::parse, "\"abcd\"");
    assert_roundtrip_with_context!(font_feature_settings::parse, "\"abcd\" on", "\"abcd\"");
    assert_roundtrip_with_context!(font_feature_settings::parse, "\"abcd\" off");
    assert_roundtrip_with_context!(font_feature_settings::parse, "\"abcd\" 4");
    assert_roundtrip_with_context!(font_feature_settings::parse, "\"abcd\", \"efgh\"");
}

#[test]
fn font_language_override_should_parse_properly() {
    use style::properties::longhands::font_language_override::{self, SpecifiedValue};

    let normal = parse_longhand!(font_language_override, "normal");
    assert_eq!(normal, SpecifiedValue::Normal);

    let empty_str = parse_longhand!(font_language_override, "\"\"");
    assert_eq!(empty_str, SpecifiedValue::Override("".to_string()));

    let normal_str = parse_longhand!(font_language_override, "\"normal\"");
    assert_eq!(normal_str, SpecifiedValue::Override("normal".to_string()));

    let turkic = parse_longhand!(font_language_override, "\"TRK\"");
    assert_eq!(turkic, SpecifiedValue::Override("TRK".to_string()));

    let danish = parse_longhand!(font_language_override, "\"DAN\"");
    assert_eq!(danish, SpecifiedValue::Override("DAN".to_string()));
}

#[test]
fn font_weight_keyword_should_preserve_keyword() {
    use style::properties::longhands::font_weight::SpecifiedValue;

    let result = parse(font_weight::parse, "normal").unwrap();
    assert_eq!(result, SpecifiedValue::Normal);

    let result = parse(font_weight::parse, "bold").unwrap();
    assert_eq!(result, SpecifiedValue::Bold);
}

#[test]
#[should_panic]
fn font_language_override_should_fail_on_empty_str() {
    use style::properties::longhands::font_language_override;

    parse_longhand!(font_language_override, "");
}
