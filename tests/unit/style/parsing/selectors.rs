/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::{Parser, ToCss};
use selectors::parser::{Selector, ParserContext, parse_selector_list};
use style::selector_impl::TheSelectorImpl;

fn parse(input: &mut Parser) -> Result<Selector<TheSelectorImpl>, ()> {
    let mut context = ParserContext::new();
    context.in_user_agent_stylesheet = true;
    context.namespace_prefixes.insert("svg".into(), ns!(svg));
    parse_selector_list(&context, input).map(|mut vec| vec.pop().unwrap())
}

#[test]
fn test_selectors() {
    assert_roundtrip!(parse, "div");
    assert_roundtrip!(parse, "svg|circle");
    assert_roundtrip!(parse, "p:before", "p::before");
    assert_roundtrip!(parse, "[border = \"0\"]:-servo-nonzero-border ~ ::-servo-details-summary");
}
