/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

pub use cssparser::Parser;
pub use media_queries::CSSErrorReporterTest;
pub use servo_url::ServoUrl;
pub use style::parser::ParserContext;
pub use style::properties::{DeclaredValue, PropertyDeclaration, PropertyDeclarationBlock, Importance, PropertyId};
pub use style::properties::parse_property_declaration_list;
pub use style::properties::property_bit_field::PropertyBitField;
pub use style::stylesheets::Origin;
pub use style::values::specified::{Length, NoCalcLength};
pub use style::values::specified::LengthOrPercentageOrAuto;

#[test]
fn property_declaration_list_should_parse_and_deduplicate_correctly() {
    let px_10 = DeclaredValue::Value(LengthOrPercentageOrAuto::Length(NoCalcLength::from_px(10f32)));
    let px_20 = DeclaredValue::Value(LengthOrPercentageOrAuto::Length(NoCalcLength::from_px(20f32)));

    let url = ServoUrl::parse("http://localhost").unwrap();
    let context = ParserContext::new(Origin::Author, &url, Box::new(CSSErrorReporterTest));

    let mut parser = Parser::new("width: 20px !important; width: 10px; height: 20px; height: 10px;");

    let block = parse_property_declaration_list(&context, &mut parser);

    assert_eq!(
        block,
        PropertyDeclarationBlock {
            declarations: vec![
                (PropertyDeclaration::Width(px_20),
                 Importance::Important),

                (PropertyDeclaration::Height(px_10),
                 Importance::Normal),
            ],
            important_count: 1,
        }
    );
}

#[test]
fn property_declaration_parse_seen_before_should_set_flag() {
    let mut results = vec![];
    let mut possibly_duplicated = false;
    let mut seen_properties = PropertyBitField::new();
    let url = ServoUrl::parse("http://localhost").unwrap();
    let context = ParserContext::new(Origin::Author, &url, Box::new(CSSErrorReporterTest));

    let mut parser = Parser::new("initial");
    let id = PropertyId::parse("width".into()).unwrap();
    seen_properties.set_width();

    PropertyDeclaration::parse(id, &context, &mut parser, &mut results, &mut seen_properties,
                               &mut possibly_duplicated, false);

    assert_eq!(true, possibly_duplicated);
}

#[test]
fn property_declaration_parse_custom_should_set_flag() {
    let mut results = vec![];
    let mut possibly_duplicated = false;
    let mut seen_properties = PropertyBitField::new();
    let url = ServoUrl::parse("http://localhost").unwrap();
    let context = ParserContext::new(Origin::Author, &url, Box::new(CSSErrorReporterTest));

    let mut parser = Parser::new("initial");
    let id = PropertyId::parse("--custom-property".into()).unwrap();

    PropertyDeclaration::parse(id, &context, &mut parser, &mut results, &mut seen_properties,
                               &mut possibly_duplicated, false);

    assert_eq!(true, possibly_duplicated);
}

#[test]
fn property_declaration_parse_shorthand_wide_should_mark_subproperties() {
    let mut results = vec![];
    let mut possibly_duplicated = false;
    let mut seen_properties = PropertyBitField::new();
    let url = ServoUrl::parse("http://localhost").unwrap();
    let context = ParserContext::new(Origin::Author, &url, Box::new(CSSErrorReporterTest));

    let mut parser = Parser::new("initial");
    let id = PropertyId::parse("border-color".into()).unwrap();

    PropertyDeclaration::parse(id, &context, &mut parser, &mut results, &mut seen_properties,
                               &mut possibly_duplicated, false);

    assert_eq!(true, seen_properties.get_border_top_color());
    assert_eq!(true, seen_properties.get_border_right_color());
    assert_eq!(true, seen_properties.get_border_bottom_color());
    assert_eq!(true, seen_properties.get_border_left_color());
}

#[test]
fn property_declaration_parse_shorthand_should_mark_subproperties() {
    let mut results = vec![];
    let mut possibly_duplicated = false;
    let mut seen_properties = PropertyBitField::new();
    let url = ServoUrl::parse("http://localhost").unwrap();
    let context = ParserContext::new(Origin::Author, &url, Box::new(CSSErrorReporterTest));

    let mut parser = Parser::new("#000");
    let id = PropertyId::parse("border-color".into()).unwrap();

    PropertyDeclaration::parse(id, &context, &mut parser, &mut results, &mut seen_properties,
                               &mut possibly_duplicated, false);

    assert_eq!(true, seen_properties.get_border_top_color());
    assert_eq!(true, seen_properties.get_border_right_color());
    assert_eq!(true, seen_properties.get_border_bottom_color());
    assert_eq!(true, seen_properties.get_border_left_color());
}
