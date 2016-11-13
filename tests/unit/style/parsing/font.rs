/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::Parser;
use media_queries::CSSErrorReporterTest;
use style::parser::ParserContext;
use style::properties::longhands::font_feature_settings;
use style::properties::longhands::font_feature_settings::computed_value;
use style::properties::longhands::font_feature_settings::computed_value::FeatureTagValue;
use style::stylesheets::Origin;
use style_traits::ToCss;
use url::Url;

#[test]
fn font_feature_settings_should_parse_properly() {
    let normal = parse_longhand!(font_feature_settings, "normal");
    let normal_computed = computed_value::T::Normal;
    assert_eq!(normal, normal_computed);

    let on = parse_longhand!(font_feature_settings, "\"abcd\" on");
    let on_computed = computed_value::T::Tag(vec![
        FeatureTagValue { tag: String::from("abcd"), value: 1 }
    ]);
    assert_eq!(on, on_computed);

    let off = parse_longhand!(font_feature_settings, "\"abcd\" off");
    let off_computed = computed_value::T::Tag(vec![
        FeatureTagValue { tag: String::from("abcd"), value: 0 }
    ]);
    assert_eq!(off, off_computed);

    let no_value = parse_longhand!(font_feature_settings, "\"abcd\"");
    let no_value_computed = computed_value::T::Tag(vec![
        FeatureTagValue { tag: String::from("abcd"), value: 1 }
    ]);
    assert_eq!(no_value, no_value_computed);

    let pos_integer = parse_longhand!(font_feature_settings, "\"abcd\" 100");
    let pos_integer_computed = computed_value::T::Tag(vec![
        FeatureTagValue { tag: String::from("abcd"), value: 100 }
    ]);
    assert_eq!(pos_integer, pos_integer_computed);

    let multiple = parse_longhand!(font_feature_settings, "\"abcd\" off, \"efgh\"");
    let multiple_computed = computed_value::T::Tag(vec![
        FeatureTagValue { tag: String::from("abcd"), value: 0 },
        FeatureTagValue { tag: String::from("efgh"), value: 1 }
    ]);
    assert_eq!(multiple, multiple_computed);
}

#[test]
fn font_feature_settings_should_throw_on_bad_input() {
    let url = Url::parse("http://localhost").unwrap();
    let context = ParserContext::new(Origin::Author, &url, Box::new(CSSErrorReporterTest));

    let mut empty = Parser::new("");
    assert!(font_feature_settings::parse(&context, &mut empty).is_err());

    let mut negative = Parser::new("\"abcd\" -1");
    assert!(font_feature_settings::parse(&context, &mut negative).is_err());

    let mut short_tag = Parser::new("\"abc\"");
    assert!(font_feature_settings::parse(&context, &mut short_tag).is_err());

    let mut illegal_tag = Parser::new("\"abcó\"");
    assert!(font_feature_settings::parse(&context, &mut illegal_tag).is_err());
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
#[should_panic]
fn font_language_override_should_fail_on_empty_str() {
    use style::properties::longhands::font_language_override;

    parse_longhand!(font_language_override, "");
}
