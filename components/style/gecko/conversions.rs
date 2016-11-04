/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! This module contains conversion helpers between Servo and Gecko types
//! Ideally, it would be in geckolib itself, but coherence
//! forces us to keep the traits and implementations here

#![allow(unsafe_code)]

use app_units::Au;
use gecko::values::{convert_rgba_to_nscolor, StyleCoordHelpers};
use gecko_bindings::bindings::{Gecko_CreateGradient, Gecko_SetGradientImageValue, Gecko_SetUrlImageValue};
use gecko_bindings::bindings::{RawServoStyleSheet, RawServoDeclarationBlock, ServoComputedValues};
use gecko_bindings::structs::{nsStyleCoord_CalcValue, nsStyleImage};
use gecko_bindings::sugar::ns_style_coord::{CoordDataValue, CoordDataMut};
use gecko_bindings::sugar::ownership::{HasArcFFI, HasFFI};
use parking_lot::RwLock;
use properties::{ComputedValues, PropertyDeclarationBlock};
use stylesheets::Stylesheet;
use values::computed::{CalcLengthOrPercentage, Gradient, Image, LengthOrPercentage, LengthOrPercentageOrAuto};

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

impl nsStyleImage {
    pub fn set(&mut self, image: Image, with_url: bool, cacheable: &mut bool) {
        match image {
            Image::Gradient(gradient) => {
                self.set_gradient(gradient)
            },
            Image::Url(ref url, ref extra_data) if with_url => {
                unsafe {
                    Gecko_SetUrlImageValue(self,
                                           url.as_str().as_ptr(),
                                           url.as_str().len() as u32,
                                           extra_data.base.get(),
                                           extra_data.referrer.get(),
                                           extra_data.principal.get());
                }
                // We unfortunately must make any url() value uncacheable, since
                // the applicable declarations cache is not per document, but
                // global, and the imgRequestProxy objects we store in the style
                // structs don't like to be tracked by more than one document.
                *cacheable = false;
            },
            _ => (),
        }
    }

    fn set_gradient(&mut self, gradient: Gradient) {
        use cssparser::Color as CSSColor;
        use gecko_bindings::structs::{NS_STYLE_GRADIENT_SHAPE_CIRCULAR, NS_STYLE_GRADIENT_SHAPE_ELLIPTICAL};
        use gecko_bindings::structs::{NS_STYLE_GRADIENT_SHAPE_LINEAR, NS_STYLE_GRADIENT_SIZE_CLOSEST_CORNER};
        use gecko_bindings::structs::{NS_STYLE_GRADIENT_SIZE_CLOSEST_SIDE, NS_STYLE_GRADIENT_SIZE_EXPLICIT_SIZE};
        use gecko_bindings::structs::{NS_STYLE_GRADIENT_SIZE_FARTHEST_CORNER, NS_STYLE_GRADIENT_SIZE_FARTHEST_SIDE};
        use gecko_bindings::structs::nsStyleCoord;
        use values::computed::{GradientKind, GradientShape, LengthOrKeyword};
        use values::computed::LengthOrPercentageOrKeyword;
        use values::specified::{AngleOrCorner, HorizontalDirection};
        use values::specified::{SizeKeyword, VerticalDirection};

        let stop_count = gradient.stops.len();
        if stop_count >= ::std::u32::MAX as usize {
            warn!("stylo: Prevented overflow due to too many gradient stops");
            return;
        }

        let gecko_gradient = match gradient.gradient_kind {
            GradientKind::Linear(angle_or_corner) => {
                let gecko_gradient = unsafe {
                    Gecko_CreateGradient(NS_STYLE_GRADIENT_SHAPE_LINEAR as u8,
                                         NS_STYLE_GRADIENT_SIZE_FARTHEST_CORNER as u8,
                                         gradient.repeating,
                                         /* legacy_syntax = */ false,
                                         stop_count as u32)
                };

                match angle_or_corner {
                    AngleOrCorner::Angle(angle) => {
                        unsafe {
                            (*gecko_gradient).mAngle.set(angle);
                            (*gecko_gradient).mBgPosX.set_value(CoordDataValue::None);
                            (*gecko_gradient).mBgPosY.set_value(CoordDataValue::None);
                        }
                    },
                    AngleOrCorner::Corner(horiz, vert) => {
                        let percent_x = match horiz {
                            HorizontalDirection::Left => 0.0,
                            HorizontalDirection::Right => 1.0,
                        };
                        let percent_y = match vert {
                            VerticalDirection::Top => 0.0,
                            VerticalDirection::Bottom => 1.0,
                        };

                        unsafe {
                            (*gecko_gradient).mAngle.set_value(CoordDataValue::None);
                            (*gecko_gradient).mBgPosX
                                             .set_value(CoordDataValue::Percent(percent_x));
                            (*gecko_gradient).mBgPosY
                                             .set_value(CoordDataValue::Percent(percent_y));
                        }
                    }
                }
                gecko_gradient
            },
            GradientKind::Radial(shape, position) => {
                let (gecko_shape, gecko_size) = match shape {
                    GradientShape::Circle(ref length) => {
                        let size = match *length {
                            LengthOrKeyword::Keyword(keyword) => {
                                match keyword {
                                    SizeKeyword::ClosestSide => NS_STYLE_GRADIENT_SIZE_CLOSEST_SIDE,
                                    SizeKeyword::FarthestSide => NS_STYLE_GRADIENT_SIZE_FARTHEST_SIDE,
                                    SizeKeyword::ClosestCorner => NS_STYLE_GRADIENT_SIZE_CLOSEST_CORNER,
                                    SizeKeyword::FarthestCorner => NS_STYLE_GRADIENT_SIZE_FARTHEST_CORNER,
                                }
                            },
                            _ => NS_STYLE_GRADIENT_SIZE_EXPLICIT_SIZE,
                        };
                        (NS_STYLE_GRADIENT_SHAPE_CIRCULAR as u8, size as u8)
                    },
                    GradientShape::Ellipse(ref length) => {
                        let size = match *length {
                            LengthOrPercentageOrKeyword::Keyword(keyword) => {
                                match keyword {
                                    SizeKeyword::ClosestSide => NS_STYLE_GRADIENT_SIZE_CLOSEST_SIDE,
                                    SizeKeyword::FarthestSide => NS_STYLE_GRADIENT_SIZE_FARTHEST_SIDE,
                                    SizeKeyword::ClosestCorner => NS_STYLE_GRADIENT_SIZE_CLOSEST_CORNER,
                                    SizeKeyword::FarthestCorner => NS_STYLE_GRADIENT_SIZE_FARTHEST_CORNER,
                                }
                            },
                            _ => NS_STYLE_GRADIENT_SIZE_EXPLICIT_SIZE,
                        };
                        (NS_STYLE_GRADIENT_SHAPE_ELLIPTICAL as u8, size as u8)
                    }
                };

                let gecko_gradient = unsafe {
                    Gecko_CreateGradient(gecko_shape,
                                         gecko_size,
                                         gradient.repeating,
                                         /* legacy_syntax = */ false,
                                         stop_count as u32)
                };

                // Clear mAngle and mBgPos fields
                unsafe {
                    (*gecko_gradient).mAngle.set_value(CoordDataValue::None);
                    (*gecko_gradient).mBgPosX.set_value(CoordDataValue::None);
                    (*gecko_gradient).mBgPosY.set_value(CoordDataValue::None);
                }

                // Setting radius values depending shape
                match shape {
                    GradientShape::Circle(length) => {
                        if let LengthOrKeyword::Length(len) = length {
                            unsafe {
                                (*gecko_gradient).mRadiusX.set_value(CoordDataValue::Coord(len.0));
                                (*gecko_gradient).mRadiusY.set_value(CoordDataValue::Coord(len.0));
                            }
                        }
                    },
                    GradientShape::Ellipse(length) => {
                        if let LengthOrPercentageOrKeyword::LengthOrPercentage(first_len, second_len) = length {
                            unsafe {
                                (*gecko_gradient).mRadiusX.set(first_len);
                                (*gecko_gradient).mRadiusY.set(second_len);
                            }
                        }
                    },
                }
                unsafe {
                    (*gecko_gradient).mBgPosX.set(position.horizontal);
                    (*gecko_gradient).mBgPosY.set(position.vertical);
                }

                gecko_gradient
            },
        };

        let mut coord: nsStyleCoord = nsStyleCoord::null();
        for (index, stop) in gradient.stops.iter().enumerate() {
            // NB: stops are guaranteed to be none in the gecko side by
            // default.
            coord.set(stop.position);
            let color = match stop.color {
                CSSColor::CurrentColor => {
                    // TODO(emilio): gecko just stores an nscolor,
                    // and it doesn't seem to support currentColor
                    // as value in a gradient.
                    //
                    // Double-check it and either remove
                    // currentColor for servo or see how gecko
                    // handles this.
                    0
                },
                CSSColor::RGBA(ref rgba) => convert_rgba_to_nscolor(rgba),
            };

            let mut stop = unsafe {
                &mut (*gecko_gradient).mStops[index]
            };

            stop.mColor = color;
            stop.mIsInterpolationHint = false;
            stop.mLocation.copy_from(&coord);
        }

        unsafe {
            Gecko_SetGradientImageValue(self, gecko_gradient);
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
