/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! This module contains conversion helpers between Servo and Gecko types
//! Ideally, it would be in geckolib itself, but coherence
//! forces us to keep the traits and implementations here

#![allow(unsafe_code)]

use app_units::Au;
use gecko_bindings::bindings::{RawServoStyleSheet, ServoComputedValues, RawServoDeclarationBlock};
use gecko_bindings::structs::nsStyleCoord_CalcValue;
use gecko_bindings::sugar::ownership::{HasArcFFI, HasFFI};
use parking_lot::RwLock;
use properties::{ComputedValues, PropertyDeclarationBlock};
use stylesheets::Stylesheet;
use values::computed::{CalcLengthOrPercentage, LengthOrPercentage, LengthOrPercentageOrAuto};

unsafe impl HasFFI for Stylesheet {
    type FFIType = RawServoStyleSheet;
}
unsafe impl HasArcFFI for Stylesheet {}
unsafe impl HasFFI for ComputedValues {
    type FFIType = ServoComputedValues;
}
unsafe impl HasArcFFI for ComputedValues {}

unsafe impl HasFFI for RwLock<PropertyDeclarationBlock> {
    type FFIType = RawServoDeclarationBlock;
}
unsafe impl HasArcFFI for RwLock<PropertyDeclarationBlock> {}

impl From<CalcLengthOrPercentage> for nsStyleCoord_CalcValue {
    fn from(other: CalcLengthOrPercentage) -> nsStyleCoord_CalcValue {
        let has_percentage = other.percentage.is_some();
        nsStyleCoord_CalcValue {
            mLength: other.length.map_or(0, |l| l.0),
            mPercent: other.percentage.unwrap_or(0.0),
            mHasPercent: has_percentage,
        }
    }
}

impl From<nsStyleCoord_CalcValue> for CalcLengthOrPercentage {
    fn from(other: nsStyleCoord_CalcValue) -> CalcLengthOrPercentage {
        let percentage = if other.mHasPercent {
            Some(other.mPercent)
        } else {
            None
        };
        CalcLengthOrPercentage {
            length: Some(Au(other.mLength)),
            percentage: percentage,
        }
    }
}

impl From<LengthOrPercentage> for nsStyleCoord_CalcValue {
    fn from(other: LengthOrPercentage) -> nsStyleCoord_CalcValue {
        match other {
            LengthOrPercentage::Length(au) => {
                nsStyleCoord_CalcValue {
                    mLength: au.0,
                    mPercent: 0.0,
                    mHasPercent: false,
                }
            },
            LengthOrPercentage::Percentage(pc) => {
                nsStyleCoord_CalcValue {
                    mLength: 0,
                    mPercent: pc,
                    mHasPercent: true,
                }
            },
            LengthOrPercentage::Calc(calc) => calc.into(),
        }
    }
}

impl LengthOrPercentageOrAuto {
    pub fn to_calc_value(&self) -> Option<nsStyleCoord_CalcValue> {
        match *self {
            LengthOrPercentageOrAuto::Length(au) => {
                Some(nsStyleCoord_CalcValue {
                    mLength: au.0,
                    mPercent: 0.0,
                    mHasPercent: false,
                })
            },
            LengthOrPercentageOrAuto::Percentage(pc) => {
                Some(nsStyleCoord_CalcValue {
                    mLength: 0,
                    mPercent: pc,
                    mHasPercent: true,
                })
            },
            LengthOrPercentageOrAuto::Calc(calc) => Some(calc.into()),
            LengthOrPercentageOrAuto::Auto => None,
        }
    }
}

impl From<nsStyleCoord_CalcValue> for LengthOrPercentage {
    fn from(other: nsStyleCoord_CalcValue) -> LengthOrPercentage {
        match (other.mHasPercent, other.mLength) {
            (false, _) => LengthOrPercentage::Length(Au(other.mLength)),
            (true, 0) => LengthOrPercentage::Percentage(other.mPercent),
            _ => LengthOrPercentage::Calc(other.into()),
        }
    }
}

pub mod basic_shape {
    use euclid::size::Size2D;
    use gecko::values::GeckoStyleCoordConvertible;
    use gecko_bindings::structs;
    use gecko_bindings::structs::{StyleBasicShape, StyleBasicShapeType, StyleFillRule};
    use gecko_bindings::structs::{nsStyleCoord, nsStyleCorners};
    use gecko_bindings::structs::StyleClipPathGeometryBox;
    use gecko_bindings::sugar::ns_style_coord::{CoordDataMut, CoordDataValue};
    use std::borrow::Borrow;
    use values::computed::{BorderRadiusSize, LengthOrPercentage};
    use values::computed::basic_shape::*;
    use values::computed::position;

    // using Borrow so that we can have a non-moving .into()
    impl<T: Borrow<StyleBasicShape>> From<T> for BasicShape {
        fn from(other: T) -> Self {
            let other = other.borrow();
            match other.mType {
                StyleBasicShapeType::Inset => {
                    let t = LengthOrPercentage::from_gecko_style_coord(&other.mCoordinates[0]);
                    let r = LengthOrPercentage::from_gecko_style_coord(&other.mCoordinates[1]);
                    let b = LengthOrPercentage::from_gecko_style_coord(&other.mCoordinates[2]);
                    let l = LengthOrPercentage::from_gecko_style_coord(&other.mCoordinates[3]);
                    let round = (&other.mRadius).into();
                    BasicShape::Inset(InsetRect {
                        top: t.expect("inset() offset should be a length, percentage, or calc value"),
                        right: r.expect("inset() offset should be a length, percentage, or calc value"),
                        bottom: b.expect("inset() offset should be a length, percentage, or calc value"),
                        left: l.expect("inset() offset should be a length, percentage, or calc value"),
                        round: Some(round),
                    })
                }
                StyleBasicShapeType::Circle => {
                    BasicShape::Circle(Circle {
                        radius: (&other.mCoordinates[0]).into(),
                        position: (&other.mPosition).into()
                    })
                }
                StyleBasicShapeType::Ellipse => {
                    BasicShape::Ellipse(Ellipse {
                        semiaxis_x: (&other.mCoordinates[0]).into(),
                        semiaxis_y: (&other.mCoordinates[1]).into(),
                        position: (&other.mPosition).into()
                    })
                }
                StyleBasicShapeType::Polygon => {
                    let fill_rule = if other.mFillRule == StyleFillRule::Evenodd {
                        FillRule::EvenOdd
                    } else {
                        FillRule::NonZero
                    };
                    let mut coords = Vec::with_capacity(other.mCoordinates.len() / 2);
                    for i in 0..(other.mCoordinates.len() / 2) {
                        let x = 2 * i;
                        let y = x + 1;
                        coords.push((LengthOrPercentage::from_gecko_style_coord(&other.mCoordinates[x])
                                    .expect("polygon() coordinate should be a length, percentage, or calc value"),
                                LengthOrPercentage::from_gecko_style_coord(&other.mCoordinates[y])
                                    .expect("polygon() coordinate should be a length, percentage, or calc value")
                            ))
                    }
                    BasicShape::Polygon(Polygon {
                        fill: fill_rule,
                        coordinates: coords,
                    })
                }
            }
        }
    }

    impl<T: Borrow<nsStyleCorners>> From<T> for BorderRadius {
        fn from(other: T) -> Self {
            let other = other.borrow();
            let get_corner = |index| {
                BorderRadiusSize(Size2D::new(
                    LengthOrPercentage::from_gecko_style_coord(&other.data_at(index))
                        .expect("<border-radius> should be a length, percentage, or calc value"),
                    LengthOrPercentage::from_gecko_style_coord(&other.data_at(index + 1))
                        .expect("<border-radius> should be a length, percentage, or calc value")))
            };

            BorderRadius {
                top_left: get_corner(0),
                top_right: get_corner(2),
                bottom_right: get_corner(4),
                bottom_left: get_corner(6),
            }
        }
    }

    // Can't be a From impl since we need to set an existing
    // nsStyleCorners, not create a new one
    impl BorderRadius {
        pub fn set_corners(&self, other: &mut nsStyleCorners) {
            let mut set_corner = |field: &BorderRadiusSize, index| {
                field.0.width.to_gecko_style_coord(&mut other.data_at_mut(index));
                field.0.height.to_gecko_style_coord(&mut other.data_at_mut(index + 1));
            };
            set_corner(&self.top_left, 0);
            set_corner(&self.top_right, 2);
            set_corner(&self.bottom_right, 4);
            set_corner(&self.bottom_left, 6);
        }
    }

    /// We use None for a nonexistant radius, but Gecko uses (0 0 0 0 / 0 0 0 0)
    pub fn set_corners_from_radius(radius: Option<BorderRadius>, other: &mut nsStyleCorners) {
        if let Some(radius) = radius {
            radius.set_corners(other);
        } else {
            for i in 0..8 {
                other.data_at_mut(i).set_value(CoordDataValue::Coord(0));
            }
        }
    }

    // Can't be a From impl since we need to set an existing
    // Position, not create a new one
    impl From<position::Position> for structs::Position {
        fn from(other: position::Position) -> Self {
            structs::Position {
                mXPosition: other.horizontal.into(),
                mYPosition: other.vertical.into()
            }
        }
    }

    impl<T: Borrow<nsStyleCoord>> From<T> for ShapeRadius {
        fn from(other: T) -> Self {
            let other = other.borrow();
            ShapeRadius::from_gecko_style_coord(other)
                .expect("<shape-radius> should be a length, percentage, calc, or keyword value")
        }
    }

    impl<T: Borrow<structs::Position>> From<T> for position::Position {
        fn from(other: T) -> Self {
            let other = other.borrow();
            position::Position {
                horizontal: other.mXPosition.into(),
                vertical: other.mYPosition.into(),
            }
        }
    }

    impl From<GeometryBox> for StyleClipPathGeometryBox {
        fn from(reference: GeometryBox) -> Self {
            use gecko_bindings::structs::StyleClipPathGeometryBox::*;
            match reference {
                GeometryBox::ShapeBox(ShapeBox::Content) => Content,
                GeometryBox::ShapeBox(ShapeBox::Padding) => Padding,
                GeometryBox::ShapeBox(ShapeBox::Border) => Border,
                GeometryBox::ShapeBox(ShapeBox::Margin) => Margin,
                GeometryBox::Fill => Fill,
                GeometryBox::Stroke => Stroke,
                GeometryBox::View => View,
            }
        }
    }

    // Will panic on NoBox
    // Ideally these would be implemented on Option<T>,
    // but coherence doesn't like that and TryFrom isn't stable
    impl From<StyleClipPathGeometryBox> for GeometryBox {
        fn from(reference: StyleClipPathGeometryBox) -> Self {
            use gecko_bindings::structs::StyleClipPathGeometryBox::*;
            match reference {
                NoBox => panic!("Shouldn't convert NoBox to GeometryBox"),
                Content => GeometryBox::ShapeBox(ShapeBox::Content),
                Padding => GeometryBox::ShapeBox(ShapeBox::Padding),
                Border => GeometryBox::ShapeBox(ShapeBox::Border),
                Margin => GeometryBox::ShapeBox(ShapeBox::Margin),
                Fill => GeometryBox::Fill,
                Stroke => GeometryBox::Stroke,
                View => GeometryBox::View,
            }
        }
    }
}
