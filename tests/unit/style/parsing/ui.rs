/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::{Color, Parser, RGBA};
use media_queries::CSSErrorReporterTest;
use style::parser::ParserContext;
use style::stylesheets::Origin;
use style::values::{Auto, Either};

#[test]
fn test_caret_color() {
    use style::properties::longhands::caret_color;

    let auto = parse_longhand!(caret_color, "auto");
    assert_eq!(auto, caret_color::SpecifiedValue(Either::Second(Auto)));

    let blue_color = Color::RGBA(RGBA {
        red: 0.0,
        green: 0.0,
        blue: 1.0,
        alpha: 1.0,
    });

    let color = parse_longhand!(caret_color, "blue");
    assert_eq!(color,
               caret_color::SpecifiedValue(Either::First(blue_color)));
}
