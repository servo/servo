/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use parsing::parse;
use servo_atoms::Atom;
use style::parser::Parse;
use style::properties::longhands::animation_iteration_count::single_value::computed_value::T as AnimationIterationCount;
use style::properties::longhands::animation_name;
use style::values::{KeyframesName, CustomIdent};
use style_traits::ToCss;

#[test]
fn test_animation_name() {
    use self::animation_name::single_value::SpecifiedValue as SingleValue;
    let other_name = Atom::from("other-name");
    assert_eq!(parse_longhand!(animation_name, "none"),
               animation_name::SpecifiedValue(vec![SingleValue(None)]));
    assert_eq!(parse_longhand!(animation_name, "other-name, none, 'other-name', \"other-name\""),
               animation_name::SpecifiedValue(
                   vec![SingleValue(Some(KeyframesName::Ident(CustomIdent(other_name.clone())))),
                        SingleValue(None),
                        SingleValue(Some(KeyframesName::QuotedString(other_name.clone()))),
                        SingleValue(Some(KeyframesName::QuotedString(other_name.clone())))]));
}

#[test]
fn test_animation_iteration() {
    assert_roundtrip_with_context!(AnimationIterationCount::parse, "0", "0");
    assert_roundtrip_with_context!(AnimationIterationCount::parse, "0.1", "0.1");
    assert_roundtrip_with_context!(AnimationIterationCount::parse, "infinite", "infinite");

    // Negative numbers are invalid
    assert!(parse(AnimationIterationCount::parse, "-1").is_err());
}
