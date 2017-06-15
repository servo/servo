/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use parsing::parse;
use servo_atoms::Atom;
use style::properties::animated_properties::TransitionProperty;
use style::properties::longhands::transition_property;
use style::properties::shorthands::transition;
use style::values::CustomIdent;
use style_traits::ToCss;

#[test]
fn test_longhand_properties() {
    assert_roundtrip_with_context!(transition_property::parse, "margin-left");
    assert_roundtrip_with_context!(transition_property::parse, "background-color");
    assert_roundtrip_with_context!(transition_property::parse, "width");
    assert_roundtrip_with_context!(transition_property::parse, "transition-duration");
    assert_roundtrip_with_context!(transition_property::parse, "unsupported-property");
    assert_roundtrip_with_context!(transition_property::parse, "-other-unsupported-property");
    assert_roundtrip_with_context!(transition_property::parse, "--var");

    assert_eq!(
        parse_longhand!(transition_property, "margin-left, transition-delay, width, --var"),
        transition_property::SpecifiedValue(vec![
            TransitionProperty::MarginLeft,
            TransitionProperty::Unsupported(CustomIdent(Atom::from("transition-delay"))),
            TransitionProperty::Width,
            TransitionProperty::Unsupported(CustomIdent(Atom::from("--var"))),
        ])
    );

    assert!(parse(transition_property::parse, ".width").is_err());
    assert!(parse(transition_property::parse, "1width").is_err());
    assert!(parse(transition_property::parse, "- ").is_err());
}

#[test]
fn test_shorthand_properties() {
    assert_roundtrip_with_context!(transition_property::parse, "margin");
    assert_roundtrip_with_context!(transition_property::parse, "background");
    assert_roundtrip_with_context!(transition_property::parse, "border-bottom");

    assert_eq!(parse_longhand!(transition_property, "margin, background"),
               transition_property::SpecifiedValue(
                   vec![TransitionProperty::Margin,
                        TransitionProperty::Background]));
}

#[test]
fn test_keywords() {
    assert_roundtrip_with_context!(transition_property::parse, "all");
    assert_roundtrip_with_context!(transition_property::parse, "none");

    assert_eq!(parse_longhand!(transition_property, "all"),
               transition_property::SpecifiedValue(vec![TransitionProperty::All]));
    assert_eq!(parse_longhand!(transition_property, "width, all"),
               transition_property::SpecifiedValue(vec![TransitionProperty::Width,
                                                        TransitionProperty::All]));

    // Using CSS Wide Keyword or none in the list will get the syntax invalid.
    // Note: Only "none" alone is valid.
    assert!(parse(transition_property::parse, "none").is_ok());
    assert_eq!(parse_longhand!(transition_property, "none"),
               transition_property::SpecifiedValue(vec![]));
    assert!(parse(transition_property::parse, "inherit").is_err());
    assert!(parse(transition_property::parse, "initial").is_err());
    assert!(parse(transition_property::parse, "unset").is_err());
    assert!(parse(transition_property::parse, "width, none").is_err());
    assert!(parse(transition_property::parse, "width, initial").is_err());
    assert!(parse(transition_property::parse, "width, inherit").is_err());
    assert!(parse(transition_property::parse, "width, unset").is_err());
}

#[test]
fn test_transition_shorthand() {
    let result = parse(transition::parse_value, "2s margin-left, 4s background").unwrap();
    assert_eq!(result.transition_property,
               parse_longhand!(transition_property, "margin-left, background"));

    let result = parse(transition::parse_value, "2s margin, 4s all").unwrap();
    assert_eq!(result.transition_property,
               parse_longhand!(transition_property, "margin, all"));

    let result = parse(transition::parse_value, "2s width, 3s --var, 4s background").unwrap();
    assert_eq!(result.transition_property,
               parse_longhand!(transition_property, "width, --var, background"));

    let result = parse(transition::parse_value, "none").unwrap();
    assert_eq!(result.transition_property,
               parse_longhand!(transition_property, "none"));

    assert!(parse(transition::parse_value, "2s width, none").is_err());
    assert!(parse(transition::parse_value, "2s width, 2s initial").is_err());
    assert!(parse(transition::parse_value, "2s width, 3s unset").is_err());
    assert!(parse(transition::parse_value, "2s width, 4s inherit").is_err());
}
