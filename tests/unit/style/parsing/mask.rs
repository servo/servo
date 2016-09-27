/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::Parser;
use media_queries::CSSErrorReporterTest;
use style::parser::ParserContext;
use style::properties::longhands::{mask_clip, mask_composite, mask_image, mask_mode};
use style::properties::longhands::{mask_origin, mask_position, mask_repeat, mask_size};
use style::properties::shorthands::mask;
use style::stylesheets::Origin;
use url::Url;

macro_rules! parse_longhand {
    ($name:ident, $s:expr) => {{
        let url = Url::parse("http://localhost").unwrap();
        let context = ParserContext::new(Origin::Author, &url, Box::new(CSSErrorReporterTest));
        $name::parse(&context, &mut Parser::new($s)).unwrap()
    }};
}

#[test]
fn test_mask_shorthand() {
    let url = Url::parse("http://localhost").unwrap();
    let context = ParserContext::new(Origin::Author, &url, Box::new(CSSErrorReporterTest));
    let mut parser = Parser::new("url(\"http://servo/test.png\") luminance 7px 4px / 70px 50px \
                                 repeat-x padding-box border-box subtract");
    let result = mask::parse_value(&context, &mut parser).unwrap();

    assert_eq!(result.mask_image.unwrap(), parse_longhand!(mask_image, "url(\"http://servo/test.png\")"));
    assert_eq!(result.mask_mode.unwrap(), parse_longhand!(mask_mode, "luminance"));
    assert_eq!(result.mask_position.unwrap(), parse_longhand!(mask_position, "7px 4px"));
    assert_eq!(result.mask_size.unwrap(), parse_longhand!(mask_size, "70px 50px"));
    assert_eq!(result.mask_repeat.unwrap(), parse_longhand!(mask_repeat, "repeat-x"));
    assert_eq!(result.mask_origin.unwrap(), parse_longhand!(mask_origin, "padding-box"));
    assert_eq!(result.mask_clip.unwrap(), parse_longhand!(mask_clip, "border-box"));
    assert_eq!(result.mask_composite.unwrap(), parse_longhand!(mask_composite, "subtract"));
}
