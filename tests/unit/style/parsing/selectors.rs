/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use cssparser::{Parser, ParserInput, ToCss};
use selectors::parser::SelectorList;
use style::selector_parser::{SelectorImpl, SelectorParser};
use style::stylesheets::{Namespaces, Origin};
use style_traits::ParseError;

fn parse_selector<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<SelectorList<SelectorImpl>, ParseError<'i>> {
    let mut ns = Namespaces::default();
    ns.prefixes.insert("svg".into(), ns!(svg));
    let parser = SelectorParser {
        stylesheet_origin: Origin::UserAgent,
        namespaces: &ns,
        url_data: None,
    };
    SelectorList::parse(&parser, input)
}

#[test]
fn test_selectors() {
    assert_roundtrip!(parse_selector, "div");
    assert_roundtrip!(parse_selector, "svg|circle");
    assert_roundtrip!(parse_selector, "p:before", "p::before");
    assert_roundtrip!(
        parse_selector,
        "[border=\"0\"]:-servo-nonzero-border ~ ::-servo-details-summary"
    );
    assert_roundtrip!(parse_selector, "* > *");
    assert_roundtrip!(parse_selector, "*|* + *", "* + *");
}
