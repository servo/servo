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
    use style::properties::longhands::text_emphasis_style::{ShapeKeyword, SpecifiedValue, Keyword};

    let none = parse_longhand!(text_emphasis_style, "none");
    assert_eq!(none, SpecifiedValue(None));

    let fill = parse_longhand!(text_emphasis_style, "open");
    let fill_struct = SpecifiedValue(Some(Keyword {
        fill: Some(false),
        shape: None
    }));
    assert_eq!(fill, fill_struct);

    let shape = parse_longhand!(text_emphasis_style, "triangle");
    let shape_struct = SpecifiedValue(Some(Keyword {
        fill: None,
        shape: Some(ShapeKeyword::Triangle)
    }));
    assert_eq!(shape, shape_struct);

    let fill_shape = parse_longhand!(text_emphasis_style, "filled dot");
    let fill_shape_struct = SpecifiedValue(Some(Keyword {
        fill: Some(true),
        shape: Some(ShapeKeyword::Dot)
    }));
    assert_eq!(fill_shape, fill_shape_struct);

    let shape_fill = parse_longhand!(text_emphasis_style, "dot filled");
    let shape_fill_struct = SpecifiedValue(Some(Keyword {
        fill: Some(true),
        shape: Some(ShapeKeyword::Dot)
    }));
    assert_eq!(shape_fill, shape_fill_struct);
}
