/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::{Parser, ToCss};
use selectors::parser::SelectorList;
use style::selector_parser::{SelectorImpl, SelectorParser};
use style::stylesheets::{Origin, Namespaces};

fn parse_selector(input: &mut Parser) -> Result<SelectorList<SelectorImpl>, ()> {
    let mut ns = Namespaces::default();
    ns.prefixes.insert("svg".into(), (ns!(svg), ()));
    let parser = SelectorParser {
        stylesheet_origin: Origin::UserAgent,
        namespaces: &ns,
    };
    SelectorList::parse(&parser, input)
}

#[test]
fn test_selectors() {
    assert_roundtrip!(parse_selector, "div");
    assert_roundtrip!(parse_selector, "svg|circle");
    assert_roundtrip!(parse_selector, "p:before", "p::before");
    assert_roundtrip!(parse_selector, "[border = \"0\"]:-servo-nonzero-border ~ ::-servo-details-summary");
}
