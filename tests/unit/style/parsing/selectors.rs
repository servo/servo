/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use cssparser::{Parser, ParserInput, ToCss};
use selectors::parser::{ParseRelative, SelectorList};
use style::selector_parser::{SelectorImpl, SelectorParser};
use style::stylesheets::{Namespaces, Origin};
use style_traits::ParseError;
use url::Url;

fn parse_selector<'i>(
    input: &mut Parser<'i, '_>,
) -> Result<SelectorList<SelectorImpl>, ParseError<'i>> {
    let mut ns = Namespaces::default();
    ns.prefixes
        .insert("svg".into(), style::Namespace::new(ns!(svg)));
    let dummy_url_data = Url::parse("about:blank").unwrap().into();
    let parser = SelectorParser {
        stylesheet_origin: Origin::UserAgent,
        namespaces: &ns,
        url_data: &dummy_url_data,
        for_supports_rule: false,
    };
    SelectorList::parse(&parser, input, ParseRelative::No)
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
