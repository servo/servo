/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use app_units::Au;
use cssparser::RGBA;
use style::properties::animated_properties::{Animatable, IntermediateRGBA};
use style::properties::longhands::transform::computed_value::ComputedOperation as TransformOperation;
use style::properties::longhands::transform::computed_value::T as TransformList;

fn interpolate_rgba(from: RGBA, to: RGBA, progress: f64) -> RGBA {
    let from: IntermediateRGBA = from.into();
    let to: IntermediateRGBA = to.into();
    from.interpolate(&to, progress).unwrap().into()
}

// Color
#[test]
fn test_rgba_color_interepolation_preserves_transparent() {
    assert_eq!(interpolate_rgba(RGBA::transparent(),
                                RGBA::transparent(), 0.5),
               RGBA::transparent());
}

#[test]
fn test_rgba_color_interepolation_alpha() {
    assert_eq!(interpolate_rgba(RGBA::new(200, 0, 0, 100),
                                RGBA::new(0, 200, 0, 200), 0.5),
               RGBA::new(67, 133, 0, 150));
}

#[test]
fn test_rgba_color_interepolation_out_of_range_1() {
    // Some cubic-bezier functions produce values that are out of range [0, 1].
    // Unclamped cases.
    assert_eq!(interpolate_rgba(RGBA::from_floats(0.3, 0.0, 0.0, 0.4),
                                RGBA::from_floats(0.0, 1.0, 0.0, 0.6), -0.5),
               RGBA::new(154, 0, 0, 77));
}

#[test]
fn test_rgba_color_interepolation_out_of_range_2() {
    assert_eq!(interpolate_rgba(RGBA::from_floats(1.0, 0.0, 0.0, 0.6),
                                RGBA::from_floats(0.0, 0.3, 0.0, 0.4), 1.5),
               RGBA::new(0, 154, 0, 77));
}

#[test]
fn test_rgba_color_interepolation_out_of_range_clamped_1() {
    assert_eq!(interpolate_rgba(RGBA::from_floats(1.0, 0.0, 0.0, 0.8),
                                RGBA::from_floats(0.0, 1.0, 0.0, 0.2), -0.5),
               RGBA::from_floats(1.0, 0.0, 0.0, 1.0));
}

#[test]
fn test_rgba_color_interepolation_out_of_range_clamped_2() {
    assert_eq!(interpolate_rgba(RGBA::from_floats(1.0, 0.0, 0.0, 0.8),
                                RGBA::from_floats(0.0, 1.0, 0.0, 0.2), 1.5),
               RGBA::from_floats(0.0, 0.0, 0.0, 0.0));
}

// Transform
#[test]
fn test_transform_interpolation_on_translate() {
    use style::values::computed::{CalcLengthOrPercentage, LengthOrPercentage};

    let from = TransformList(Some(vec![
        TransformOperation::Translate(LengthOrPercentage::Length(Au(0)),
                                      LengthOrPercentage::Length(Au(100)),
                                      Au(25))]));
    let to = TransformList(Some(vec![
        TransformOperation::Translate(LengthOrPercentage::Length(Au(100)),
                                      LengthOrPercentage::Length(Au(0)),
                                      Au(75))]));
    assert_eq!(from.interpolate(&to, 0.5).unwrap(),
               TransformList(Some(vec![
                   TransformOperation::Translate(LengthOrPercentage::Length(Au(50)),
                                                 LengthOrPercentage::Length(Au(50)),
                                                 Au(50))])));

    let from = TransformList(Some(vec![
        TransformOperation::Translate(LengthOrPercentage::Percentage(0.5),
                                      LengthOrPercentage::Percentage(1.0),
                                      Au(25))]));
    let to = TransformList(Some(vec![
        TransformOperation::Translate(LengthOrPercentage::Length(Au(100)),
                                      LengthOrPercentage::Length(Au(50)),
                                      Au(75))]));
    assert_eq!(from.interpolate(&to, 0.5).unwrap(),
               TransformList(Some(vec![
                   TransformOperation::Translate(LengthOrPercentage::Calc(
                                                     // calc(50px + 25%)
                                                     CalcLengthOrPercentage::new(Au(50),
                                                                                 Some(0.25))),
                                                 LengthOrPercentage::Calc(
                                                     // calc(25px + 50%)
                                                     CalcLengthOrPercentage::new(Au(25),
                                                                                 Some(0.5))),
                                                 Au(50))])));
}

#[test]
fn test_transform_interpolation_on_scale() {
    let from = TransformList(Some(vec![TransformOperation::Scale(1.0, 2.0, 1.0)]));
    let to = TransformList(Some(vec![TransformOperation::Scale(2.0, 4.0, 2.0)]));
    assert_eq!(from.interpolate(&to, 0.5).unwrap(),
               TransformList(Some(vec![TransformOperation::Scale(1.5, 3.0, 1.5)])));
}

#[test]
fn test_transform_interpolation_on_rotate() {
    use style::values::computed::Angle;

    let from = TransformList(Some(vec![TransformOperation::Rotate(0.0, 0.0, 1.0,
                                                                  Angle::from_radians(0.0))]));
    let to = TransformList(Some(vec![TransformOperation::Rotate(0.0, 0.0, 1.0,
                                                                Angle::from_radians(100.0))]));
    assert_eq!(from.interpolate(&to, 0.5).unwrap(),
               TransformList(Some(vec![TransformOperation::Rotate(0.0, 0.0, 1.0,
                                                                  Angle::from_radians(50.0))])));
}

#[test]
fn test_transform_interpolation_on_skew() {
    use style::values::computed::Angle;

    let from = TransformList(Some(vec![TransformOperation::Skew(Angle::from_radians(0.0),
                                                                Angle::from_radians(100.0))]));
    let to = TransformList(Some(vec![TransformOperation::Skew(Angle::from_radians(100.0),
                                                              Angle::from_radians(0.0))]));
    assert_eq!(from.interpolate(&to, 0.5).unwrap(),
               TransformList(Some(vec![TransformOperation::Skew(Angle::from_radians(50.0),
                                                                Angle::from_radians(50.0))])));
}

#[test]
fn test_transform_interpolation_on_mismatched_lists() {
    use style::values::computed::{Angle, LengthOrPercentage, Percentage};

    let from = TransformList(Some(vec![TransformOperation::Rotate(0.0, 0.0, 1.0,
                                                                  Angle::from_radians(100.0))]));
    let to = TransformList(Some(vec![
        TransformOperation::Translate(LengthOrPercentage::Length(Au(100)),
                                      LengthOrPercentage::Length(Au(0)),
                                      Au(0))]));
    assert_eq!(from.interpolate(&to, 0.5).unwrap(),
               TransformList(Some(vec![TransformOperation::InterpolateMatrix {
                   from_list: from.clone(),
                   to_list: to.clone(),
                   progress: Percentage(0.5)
               }])));
}
