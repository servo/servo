/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! This module contains conversion helpers between Servo and Gecko types
//! Ideally, it would be in geckolib itself, but coherence
//! forces us to keep the traits and implementations here

#![allow(unsafe_code)]

use app_units::Au;
use gecko::values::{convert_rgba_to_nscolor, GeckoStyleCoordConvertible};
use gecko_bindings::bindings::{Gecko_CreateGradient, Gecko_SetGradientImageValue, Gecko_SetUrlImageValue};
use gecko_bindings::bindings::{Gecko_InitializeImageCropRect, Gecko_SetImageElement};
use gecko_bindings::structs::{nsCSSUnit, nsStyleCoord_CalcValue, nsStyleImage};
use gecko_bindings::structs::{nsresult, SheetType};
use gecko_bindings::sugar::ns_style_coord::{CoordDataValue, CoordDataMut};
use stylesheets::{Origin, RulesMutateError};
use values::computed::{Angle, CalcLengthOrPercentage, Gradient, GradientItem, Image};
use values::computed::{LengthOrPercentage, LengthOrPercentageOrAuto};

impl From<CalcLengthOrPercentage> for nsStyleCoord_CalcValue {
    fn from(other: CalcLengthOrPercentage) -> nsStyleCoord_CalcValue {
        let has_percentage = other.percentage.is_some();
        nsStyleCoord_CalcValue {
            mLength: other.length.0,
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
            length: Au(other.mLength),
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
    /// Convert this value in an appropriate `nsStyleCoord::CalcValue`.
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

impl From<Angle> for CoordDataValue {
    fn from(reference: Angle) -> Self {
        match reference {
            Angle::Degree(val) => CoordDataValue::Degree(val),
            Angle::Gradian(val) => CoordDataValue::Grad(val),
            Angle::Radian(val) => CoordDataValue::Radian(val),
            Angle::Turn(val) => CoordDataValue::Turn(val),
        }
    }
}

impl Angle {
    /// Converts Angle struct into (value, unit) pair.
    pub fn to_gecko_values(&self) -> (f32, nsCSSUnit) {
        match *self {
            Angle::Degree(val) => (val, nsCSSUnit::eCSSUnit_Degree),
            Angle::Gradian(val) => (val, nsCSSUnit::eCSSUnit_Grad),
            Angle::Radian(val) => (val, nsCSSUnit::eCSSUnit_Radian),
            Angle::Turn(val) => (val, nsCSSUnit::eCSSUnit_Turn),
        }
    }

    /// Converts gecko (value, unit) pair into Angle struct
    pub fn from_gecko_values(value: f32, unit: nsCSSUnit) -> Angle {
        match unit {
            nsCSSUnit::eCSSUnit_Degree => Angle::Degree(value),
            nsCSSUnit::eCSSUnit_Grad => Angle::Gradian(value),
            nsCSSUnit::eCSSUnit_Radian => Angle::Radian(value),
            nsCSSUnit::eCSSUnit_Turn => Angle::Turn(value),
            _ => panic!("Unexpected unit {:?} for angle", unit),
        }
    }
}

impl nsStyleImage {
    /// Set a given Servo `Image` value into this `nsStyleImage`.
    pub fn set(&mut self, image: Image, cacheable: &mut bool) {
        match image {
            Image::Gradient(gradient) => {
                self.set_gradient(gradient)
            },
            Image::Url(ref url) => {
                unsafe {
                    Gecko_SetUrlImageValue(self, url.for_ffi());
                    // We unfortunately must make any url() value uncacheable, since
                    // the applicable declarations cache is not per document, but
                    // global, and the imgRequestProxy objects we store in the style
                    // structs don't like to be tracked by more than one document.
                    //
                    // FIXME(emilio): With the scoped TLS thing this is no longer
                    // true, remove this line in a follow-up!
                    *cacheable = false;
                }
            },
            Image::ImageRect(ref image_rect) => {
                unsafe {
                    Gecko_SetUrlImageValue(self, image_rect.url.for_ffi());
                    Gecko_InitializeImageCropRect(self);

                    // We unfortunately must make any url() value uncacheable, since
                    // the applicable declarations cache is not per document, but
                    // global, and the imgRequestProxy objects we store in the style
                    // structs don't like to be tracked by more than one document.
                    //
                    // FIXME(emilio): With the scoped TLS thing this is no longer
                    // true, remove this line in a follow-up!
                    *cacheable = false;

                    // Set CropRect
                    let ref mut rect = *self.mCropRect.mPtr;
                    image_rect.top.to_gecko_style_coord(&mut rect.data_at_mut(0));
                    image_rect.right.to_gecko_style_coord(&mut rect.data_at_mut(1));
                    image_rect.bottom.to_gecko_style_coord(&mut rect.data_at_mut(2));
                    image_rect.left.to_gecko_style_coord(&mut rect.data_at_mut(3));
                }
            }
            Image::Element(ref element) => {
                unsafe {
                    Gecko_SetImageElement(self, element.as_ptr());
                }
            }
        }
    }

    fn set_gradient(&mut self, gradient: Gradient) {
        use cssparser::Color as CSSColor;
        use gecko_bindings::structs::{NS_STYLE_GRADIENT_SHAPE_CIRCULAR, NS_STYLE_GRADIENT_SHAPE_ELLIPTICAL};
        use gecko_bindings::structs::{NS_STYLE_GRADIENT_SHAPE_LINEAR, NS_STYLE_GRADIENT_SIZE_CLOSEST_CORNER};
        use gecko_bindings::structs::{NS_STYLE_GRADIENT_SIZE_CLOSEST_SIDE, NS_STYLE_GRADIENT_SIZE_EXPLICIT_SIZE};
        use gecko_bindings::structs::{NS_STYLE_GRADIENT_SIZE_FARTHEST_CORNER, NS_STYLE_GRADIENT_SIZE_FARTHEST_SIDE};
        use gecko_bindings::structs::nsStyleCoord;
        use values::computed::{AngleOrCorner, GradientKind, GradientShape, LengthOrKeyword};
        use values::computed::LengthOrPercentageOrKeyword;
        use values::specified::{HorizontalDirection, SizeKeyword, VerticalDirection};

        let stop_count = gradient.items.len();
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
                let keyword_to_gecko_size = |keyword| {
                    match keyword {
                        SizeKeyword::ClosestSide => NS_STYLE_GRADIENT_SIZE_CLOSEST_SIDE,
                        SizeKeyword::FarthestSide => NS_STYLE_GRADIENT_SIZE_FARTHEST_SIDE,
                        SizeKeyword::ClosestCorner => NS_STYLE_GRADIENT_SIZE_CLOSEST_CORNER,
                        SizeKeyword::FarthestCorner => NS_STYLE_GRADIENT_SIZE_FARTHEST_CORNER,
                        SizeKeyword::Contain => NS_STYLE_GRADIENT_SIZE_CLOSEST_SIDE,
                        SizeKeyword::Cover => NS_STYLE_GRADIENT_SIZE_FARTHEST_CORNER,
                    }
                };
                let (gecko_shape, gecko_size) = match shape {
                    GradientShape::Circle(ref length) => {
                        let size = match *length {
                            LengthOrKeyword::Keyword(keyword) => {
                                keyword_to_gecko_size(keyword)
                            },
                            _ => NS_STYLE_GRADIENT_SIZE_EXPLICIT_SIZE,
                        };
                        (NS_STYLE_GRADIENT_SHAPE_CIRCULAR as u8, size as u8)
                    },
                    GradientShape::Ellipse(ref length) => {
                        let size = match *length {
                            LengthOrPercentageOrKeyword::Keyword(keyword) => {
                                keyword_to_gecko_size(keyword)
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
                    (*gecko_gradient).mBgPosX.set(position.horizontal.0);
                    (*gecko_gradient).mBgPosY.set(position.vertical.0);
                }

                gecko_gradient
            },
        };

        for (index, item) in gradient.items.iter().enumerate() {
            // NB: stops are guaranteed to be none in the gecko side by
            // default.

            let mut gecko_stop = unsafe {
                &mut (*gecko_gradient).mStops[index]
            };
            let mut coord = nsStyleCoord::null();

            match *item {
                GradientItem::ColorStop(ref stop) => {
                    gecko_stop.mColor = match stop.color {
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
                    gecko_stop.mIsInterpolationHint = false;
                    coord.set(stop.position);
                },
                GradientItem::InterpolationHint(hint) => {
                    gecko_stop.mIsInterpolationHint = true;
                    coord.set(Some(hint));
                }
            }

            gecko_stop.mLocation.move_from(coord);
        }

        unsafe {
            Gecko_SetGradientImageValue(self, gecko_gradient);
        }
    }
}

pub mod basic_shape {
    //! Conversions from and to CSS shape representations.

    use gecko::values::GeckoStyleCoordConvertible;
    use gecko_bindings::structs;
    use gecko_bindings::structs::{StyleBasicShape, StyleBasicShapeType, StyleFillRule};
    use gecko_bindings::structs::{nsStyleCoord, nsStyleCorners};
    use gecko_bindings::structs::StyleGeometryBox;
    use gecko_bindings::sugar::ns_style_coord::{CoordDataMut, CoordDataValue};
    use std::borrow::Borrow;
    use values::computed::{BorderRadiusSize, LengthOrPercentage};
    use values::computed::basic_shape::*;
    use values::computed::position;
    use values::generics::BorderRadiusSize as GenericBorderRadiusSize;
    use values::generics::basic_shape::FillRule;
    use values::generics::position::{HorizontalPosition, VerticalPosition};

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
                GenericBorderRadiusSize::new(
                    LengthOrPercentage::from_gecko_style_coord(&other.data_at(index))
                        .expect("<border-radius> should be a length, percentage, or calc value"),
                    LengthOrPercentage::from_gecko_style_coord(&other.data_at(index + 1))
                        .expect("<border-radius> should be a length, percentage, or calc value"))
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
        /// Set this `BorderRadius` into a given `nsStyleCoord`.
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
                mXPosition: other.horizontal.0.into(),
                mYPosition: other.vertical.0.into()
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
                horizontal: HorizontalPosition(other.mXPosition.into()),
                vertical: VerticalPosition(other.mYPosition.into()),
            }
        }
    }

    impl From<ShapeBox> for StyleGeometryBox {
        fn from(reference: ShapeBox) -> Self {
            use gecko_bindings::structs::StyleGeometryBox::*;
            match reference {
                ShapeBox::ContentBox => ContentBox,
                ShapeBox::PaddingBox => PaddingBox,
                ShapeBox::BorderBox => BorderBox,
                ShapeBox::MarginBox => MarginBox,
            }
        }
    }

    impl From<GeometryBox> for StyleGeometryBox {
        fn from(reference: GeometryBox) -> Self {
            use gecko_bindings::structs::StyleGeometryBox::*;
            match reference {
                GeometryBox::ShapeBox(shape_box) => From::from(shape_box),
                GeometryBox::FillBox => FillBox,
                GeometryBox::StrokeBox => StrokeBox,
                GeometryBox::ViewBox => ViewBox,
            }
        }
    }

    // Will panic on NoBox
    // Ideally these would be implemented on Option<T>,
    // but coherence doesn't like that and TryFrom isn't stable
    impl From<StyleGeometryBox> for GeometryBox {
        fn from(reference: StyleGeometryBox) -> Self {
            use gecko_bindings::structs::StyleGeometryBox::*;
            match reference {
                ContentBox => GeometryBox::ShapeBox(ShapeBox::ContentBox),
                PaddingBox => GeometryBox::ShapeBox(ShapeBox::PaddingBox),
                BorderBox => GeometryBox::ShapeBox(ShapeBox::BorderBox),
                MarginBox => GeometryBox::ShapeBox(ShapeBox::MarginBox),
                FillBox => GeometryBox::FillBox,
                StrokeBox => GeometryBox::StrokeBox,
                ViewBox => GeometryBox::ViewBox,
                other => panic!("Unexpected StyleGeometryBox::{:?} while converting to GeometryBox", other),
            }
        }
    }
}

impl From<RulesMutateError> for nsresult {
    fn from(other: RulesMutateError) -> Self {
        match other {
            RulesMutateError::Syntax => nsresult::NS_ERROR_DOM_SYNTAX_ERR,
            RulesMutateError::IndexSize => nsresult::NS_ERROR_DOM_INDEX_SIZE_ERR,
            RulesMutateError::HierarchyRequest => nsresult::NS_ERROR_DOM_HIERARCHY_REQUEST_ERR,
            RulesMutateError::InvalidState => nsresult::NS_ERROR_DOM_INVALID_STATE_ERR,
        }
    }
}

impl From<Origin> for SheetType {
    fn from(other: Origin) -> Self {
        match other {
            Origin::UserAgent => SheetType::Agent,
            Origin::Author => SheetType::Doc,
            Origin::User => SheetType::User,
        }
    }
}
