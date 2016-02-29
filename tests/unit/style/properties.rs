/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::sync::Arc;
use style::computed_values::display::T::inline_block;
use style::properties::{PropertyDeclaration, PropertyDeclarationBlock, DeclaredValue};
use style::values::specified::{Length, LengthOrPercentageOrAuto, LengthOrPercentage};

#[test]
fn property_declaration_block_should_serialize_correctly() {
    let mut normal = Vec::new();
    let mut important = Vec::new();

    let length = LengthOrPercentageOrAuto::Length(Length::from_px(70f32));
    let value = DeclaredValue::Value(length);
    normal.push(PropertyDeclaration::Width(value));

    let min_height = LengthOrPercentage::Length(Length::from_px(20f32));
    let value = DeclaredValue::Value(min_height);
    normal.push(PropertyDeclaration::MinHeight(value));

    let value = DeclaredValue::Value(inline_block);
    normal.push(PropertyDeclaration::Display(value));

    let height = LengthOrPercentageOrAuto::Length(Length::from_px(20f32));
    let value = DeclaredValue::Value(height);
    important.push(PropertyDeclaration::Height(value));

    normal.reverse();
    important.reverse();
    let block = PropertyDeclarationBlock {
        normal: Arc::new(normal),
        important: Arc::new(important)
    };

    let css_string = block.serialize();

    assert_eq!(
        css_string,
        "width: 70px; min-height: 20px; display: inline-block; height: 20px ! important;"
    );
}
