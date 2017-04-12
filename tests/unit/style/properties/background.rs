/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::Parser;
use media_queries::CSSErrorReporterTest;
use style::parser::ParserContext;
use style::properties::longhands::background_size;
use style::stylesheets::{CssRuleType, Origin};

#[test]
fn background_size_should_reject_negative_values() {
    let url = ::servo_url::ServoUrl::parse("http://localhost").unwrap();
    let reporter = CSSErrorReporterTest;
    let context = ParserContext::new(Origin::Author, &url, &reporter, Some(CssRuleType::Style));

    let parse_result = background_size::parse(&context, &mut Parser::new("-40% -40%"));

    assert_eq!(parse_result.is_err(), true);
}
