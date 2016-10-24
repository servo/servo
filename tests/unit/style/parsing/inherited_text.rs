/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::Parser;
use media_queries::CSSErrorReporterTest;
use style::parser::ParserContext;
use style::stylesheets::Origin;
use url::Url;

#[test]
fn text_emphasis_style_longhand_should_parse_properly() {
    use style::properties::longhands::text_emphasis_style;
    use style::properties::longhands::text_emphasis_style::{ShapeKeyword, SpecifiedValue, KeywordValue};

    let none = parse_longhand!(text_emphasis_style, "none");
    assert_eq!(none, SpecifiedValue::None);

    let fill = parse_longhand!(text_emphasis_style, "open");
    let fill_struct = SpecifiedValue::Keyword(KeywordValue::Fill(false));
    assert_eq!(fill, fill_struct);

    let shape = parse_longhand!(text_emphasis_style, "triangle");
    let shape_struct = SpecifiedValue::Keyword(KeywordValue::Shape(ShapeKeyword::Triangle));
    assert_eq!(shape, shape_struct);

    let fill_shape = parse_longhand!(text_emphasis_style, "filled dot");
    let fill_shape_struct = SpecifiedValue::Keyword(KeywordValue::FillAndShape(true, ShapeKeyword::Dot));
    assert_eq!(fill_shape, fill_shape_struct);

    let shape_fill = parse_longhand!(text_emphasis_style, "dot filled");
    let shape_fill_struct = SpecifiedValue::Keyword(KeywordValue::FillAndShape(true, ShapeKeyword::Dot));
    assert_eq!(shape_fill, shape_fill_struct);

    let a_string = parse_longhand!(text_emphasis_style, "\"a\"");
    let a_string_struct = SpecifiedValue::String("a".to_string());
    assert_eq!(a_string, a_string_struct);

    let chinese_string = parse_longhand!(text_emphasis_style, "\"点\"");
    let chinese_string_struct = SpecifiedValue::String("点".to_string());
    assert_eq!(chinese_string, chinese_string_struct);

    let unicode_string = parse_longhand!(text_emphasis_style, "\"\\25B2\"");
    let unicode_string_struct = SpecifiedValue::String("▲".to_string());
    assert_eq!(unicode_string, unicode_string_struct);

    let devanagari_string = parse_longhand!(text_emphasis_style, "\"षि\"");
    let devanagari_string_struct = SpecifiedValue::String("षि".to_string());
    assert_eq!(devanagari_string, devanagari_string_struct);
}
