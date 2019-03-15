/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! This module contains conversion helpers between Servo and Gecko types
//! Ideally, it would be in geckolib itself, but coherence
//! forces us to keep the traits and implementations here
//!
//! FIXME(emilio): This file should generally just die.

#![allow(unsafe_code)]

use crate::gecko::values::GeckoStyleCoordConvertible;
use crate::gecko_bindings::bindings;
use crate::gecko_bindings::structs::RawGeckoGfxMatrix4x4;
use crate::gecko_bindings::structs::{self, nsStyleCoord_CalcValue};
use crate::gecko_bindings::structs::{nsStyleImage, nsresult, SheetType};
use crate::gecko_bindings::sugar::ns_style_coord::{CoordData, CoordDataMut, CoordDataValue};
use crate::stylesheets::{Origin, RulesMutateError};
use crate::values::computed::image::LineDirection;
use crate::values::computed::transform::Matrix3D;
use crate::values::computed::url::ComputedImageUrl;
use crate::values::computed::{Angle, Gradient, Image};
use crate::values::computed::{Integer, LengthPercentage};
use crate::values::computed::{LengthPercentageOrAuto, NonNegativeLengthPercentageOrAuto};
use crate::values::computed::{Percentage, TextAlign};
use crate::values::generics::box_::VerticalAlign;
use crate::values::generics::grid::{TrackListValue, TrackSize};
use crate::values::generics::image::{CompatMode, GradientItem, Image as GenericImage};
use crate::values::generics::rect::Rect;
use crate::values::generics::NonNegative;
use app_units::Au;
use std::f32::consts::PI;
use style_traits::values::specified::AllowedNumericType;

impl From<LengthPercentage> for nsStyleCoord_CalcValue {
    fn from(other: LengthPercentage) -> nsStyleCoord_CalcValue {
        let has_percentage = other.percentage.is_some();
        nsStyleCoord_CalcValue {
            mLength: other.unclamped_length().to_i32_au(),
            mPercent: other.percentage.map_or(0., |p| p.0),
            mHasPercent: has_percentage,
        }
    }
}

impl From<nsStyleCoord_CalcValue> for LengthPercentage {
    fn from(other: nsStyleCoord_CalcValue) -> LengthPercentage {
        let percentage = if other.mHasPercent {
            Some(Percentage(other.mPercent))
        } else {
            None
        };
        Self::with_clamping_mode(
            Au(other.mLength).into(),
            percentage,
            AllowedNumericType::All,
            /* was_calc = */ true,
        )
    }
}

impl LengthPercentageOrAuto {
    /// Convert this value in an appropriate `nsStyleCoord::CalcValue`.
    pub fn to_calc_value(&self) -> Option<nsStyleCoord_CalcValue> {
        match *self {
            LengthPercentageOrAuto::LengthPercentage(len) => Some(From::from(len)),
            LengthPercentageOrAuto::Auto => None,
        }
    }
}

impl From<nsStyleCoord_CalcValue> for LengthPercentageOrAuto {
    fn from(other: nsStyleCoord_CalcValue) -> LengthPercentageOrAuto {
        LengthPercentageOrAuto::LengthPercentage(LengthPercentage::from(other))
    }
}

// FIXME(emilio): A lot of these impl From should probably become explicit or
// disappear as we move more stuff to cbindgen.
impl From<nsStyleCoord_CalcValue> for NonNegativeLengthPercentageOrAuto {
    fn from(other: nsStyleCoord_CalcValue) -> Self {
        NonNegative(
            LengthPercentageOrAuto::LengthPercentage(LengthPercentage::with_clamping_mode(
                Au(other.mLength).into(),
                if other.mHasPercent {
                    Some(Percentage(other.mPercent))
                } else {
                    None
                },
                AllowedNumericType::NonNegative,
                /* was_calc = */ true,
            ))
        )
    }
}

impl From<Angle> for CoordDataValue {
    fn from(reference: Angle) -> Self {
        CoordDataValue::Degree(reference.degrees())
    }
}

fn line_direction(horizontal: LengthPercentage, vertical: LengthPercentage) -> LineDirection {
    use crate::values::computed::position::Position;
    use crate::values::specified::position::{X, Y};

    let horizontal_percentage = horizontal.as_percentage();
    let vertical_percentage = vertical.as_percentage();

    let horizontal_as_corner = horizontal_percentage.and_then(|percentage| {
        if percentage.0 == 0.0 {
            Some(X::Left)
        } else if percentage.0 == 1.0 {
            Some(X::Right)
        } else {
            None
        }
    });

    let vertical_as_corner = vertical_percentage.and_then(|percentage| {
        if percentage.0 == 0.0 {
            Some(Y::Top)
        } else if percentage.0 == 1.0 {
            Some(Y::Bottom)
        } else {
            None
        }
    });

    if let (Some(hc), Some(vc)) = (horizontal_as_corner, vertical_as_corner) {
        return LineDirection::Corner(hc, vc);
    }

    if let Some(hc) = horizontal_as_corner {
        if vertical_percentage == Some(Percentage(0.5)) {
            return LineDirection::Horizontal(hc);
        }
    }

    if let Some(vc) = vertical_as_corner {
        if horizontal_percentage == Some(Percentage(0.5)) {
            return LineDirection::Vertical(vc);
        }
    }

    LineDirection::MozPosition(
        Some(Position {
            horizontal,
            vertical,
        }),
        None,
    )
}

impl nsStyleImage {
    /// Set a given Servo `Image` value into this `nsStyleImage`.
    pub fn set(&mut self, image: Image) {
        match image {
            GenericImage::Gradient(boxed_gradient) => self.set_gradient(*boxed_gradient),
            GenericImage::Url(ref url) => unsafe {
                bindings::Gecko_SetLayerImageImageValue(self, (url.0).0.url_value.get());
            },
            GenericImage::Rect(ref image_rect) => {
                unsafe {
                    bindings::Gecko_SetLayerImageImageValue(
                        self,
                        (image_rect.url.0).0.url_value.get(),
                    );
                    bindings::Gecko_InitializeImageCropRect(self);

                    // Set CropRect
                    let ref mut rect = *self.mCropRect.mPtr;
                    image_rect
                        .top
                        .to_gecko_style_coord(&mut rect.data_at_mut(0));
                    image_rect
                        .right
                        .to_gecko_style_coord(&mut rect.data_at_mut(1));
                    image_rect
                        .bottom
                        .to_gecko_style_coord(&mut rect.data_at_mut(2));
                    image_rect
                        .left
                        .to_gecko_style_coord(&mut rect.data_at_mut(3));
                }
            },
            GenericImage::Element(ref element) => unsafe {
                bindings::Gecko_SetImageElement(self, element.as_ptr());
            },
        }
    }

    // FIXME(emilio): This is really complex, we should use cbindgen for this.
    fn set_gradient(&mut self, gradient: Gradient) {
        use self::structs::nsStyleCoord;
        use self::structs::NS_STYLE_GRADIENT_SIZE_CLOSEST_CORNER as CLOSEST_CORNER;
        use self::structs::NS_STYLE_GRADIENT_SIZE_CLOSEST_SIDE as CLOSEST_SIDE;
        use self::structs::NS_STYLE_GRADIENT_SIZE_FARTHEST_CORNER as FARTHEST_CORNER;
        use self::structs::NS_STYLE_GRADIENT_SIZE_FARTHEST_SIDE as FARTHEST_SIDE;
        use crate::values::generics::image::{
            Circle, Ellipse, EndingShape, GradientKind, ShapeExtent,
        };
        use crate::values::specified::position::{X, Y};

        let stop_count = gradient.items.len();
        if stop_count >= ::std::u32::MAX as usize {
            warn!("stylo: Prevented overflow due to too many gradient stops");
            return;
        }

        let gecko_gradient = match gradient.kind {
            GradientKind::Linear(direction) => {
                let gecko_gradient = unsafe {
                    bindings::Gecko_CreateGradient(
                        structs::NS_STYLE_GRADIENT_SHAPE_LINEAR as u8,
                        structs::NS_STYLE_GRADIENT_SIZE_FARTHEST_CORNER as u8,
                        gradient.repeating,
                        gradient.compat_mode != CompatMode::Modern,
                        gradient.compat_mode == CompatMode::Moz,
                        stop_count as u32,
                    )
                };

                match direction {
                    LineDirection::Angle(angle) => {
                        // PI radians (180deg) is ignored because it is the default value.
                        if angle.radians() != PI {
                            unsafe {
                                (*gecko_gradient).mAngle.set(angle);
                            }
                        }
                    },
                    LineDirection::Horizontal(x) => {
                        let x = match x {
                            X::Left => 0.0,
                            X::Right => 1.0,
                        };

                        unsafe {
                            (*gecko_gradient)
                                .mBgPosX
                                .set_value(CoordDataValue::Percent(x));
                            (*gecko_gradient)
                                .mBgPosY
                                .set_value(CoordDataValue::Percent(0.5));
                        }
                    },
                    LineDirection::Vertical(y) => {
                        // Although bottom is the default value, we can not ignore
                        // it here, because the rendering code of Gecko relies on
                        // this to behave correctly for legacy mode.
                        let y = match y {
                            Y::Top => 0.0,
                            Y::Bottom => 1.0,
                        };
                        unsafe {
                            (*gecko_gradient)
                                .mBgPosX
                                .set_value(CoordDataValue::Percent(0.5));
                            (*gecko_gradient)
                                .mBgPosY
                                .set_value(CoordDataValue::Percent(y));
                        }
                    },
                    LineDirection::Corner(horiz, vert) => {
                        let percent_x = match horiz {
                            X::Left => 0.0,
                            X::Right => 1.0,
                        };
                        let percent_y = match vert {
                            Y::Top => 0.0,
                            Y::Bottom => 1.0,
                        };

                        unsafe {
                            (*gecko_gradient)
                                .mBgPosX
                                .set_value(CoordDataValue::Percent(percent_x));
                            (*gecko_gradient)
                                .mBgPosY
                                .set_value(CoordDataValue::Percent(percent_y));
                        }
                    },
                    #[cfg(feature = "gecko")]
                    LineDirection::MozPosition(position, angle) => unsafe {
                        if let Some(position) = position {
                            (*gecko_gradient).mBgPosX.set(position.horizontal);
                            (*gecko_gradient).mBgPosY.set(position.vertical);
                        }
                        if let Some(angle) = angle {
                            (*gecko_gradient).mAngle.set(angle);
                        }
                    },
                }
                gecko_gradient
            },
            GradientKind::Radial(shape, position, angle) => {
                let keyword_to_gecko_size = |keyword| match keyword {
                    ShapeExtent::ClosestSide => CLOSEST_SIDE,
                    ShapeExtent::FarthestSide => FARTHEST_SIDE,
                    ShapeExtent::ClosestCorner => CLOSEST_CORNER,
                    ShapeExtent::FarthestCorner => FARTHEST_CORNER,
                    ShapeExtent::Contain => CLOSEST_SIDE,
                    ShapeExtent::Cover => FARTHEST_CORNER,
                };
                let (gecko_shape, gecko_size) = match shape {
                    EndingShape::Circle(ref circle) => {
                        let size = match *circle {
                            Circle::Extent(extent) => keyword_to_gecko_size(extent),
                            _ => structs::NS_STYLE_GRADIENT_SIZE_EXPLICIT_SIZE,
                        };
                        (structs::NS_STYLE_GRADIENT_SHAPE_CIRCULAR as u8, size as u8)
                    },
                    EndingShape::Ellipse(ref ellipse) => {
                        let size = match *ellipse {
                            Ellipse::Extent(extent) => keyword_to_gecko_size(extent),
                            _ => structs::NS_STYLE_GRADIENT_SIZE_EXPLICIT_SIZE,
                        };
                        (
                            structs::NS_STYLE_GRADIENT_SHAPE_ELLIPTICAL as u8,
                            size as u8,
                        )
                    },
                };

                let gecko_gradient = unsafe {
                    bindings::Gecko_CreateGradient(
                        gecko_shape,
                        gecko_size,
                        gradient.repeating,
                        gradient.compat_mode == CompatMode::Moz,
                        gradient.compat_mode == CompatMode::Moz,
                        stop_count as u32,
                    )
                };

                // Clear mBgPos field and set mAngle if angle is set. Otherwise clear it.
                unsafe {
                    if let Some(angle) = angle {
                        (*gecko_gradient).mAngle.set(angle);
                    }
                }

                // Setting radius values depending shape
                match shape {
                    EndingShape::Circle(Circle::Radius(length)) => unsafe {
                        let au = length.to_i32_au();
                        (*gecko_gradient)
                            .mRadiusX
                            .set_value(CoordDataValue::Coord(au));
                        (*gecko_gradient)
                            .mRadiusY
                            .set_value(CoordDataValue::Coord(au));
                    },
                    EndingShape::Ellipse(Ellipse::Radii(x, y)) => unsafe {
                        (*gecko_gradient).mRadiusX.set(x);
                        (*gecko_gradient).mRadiusY.set(y);
                    },
                    _ => {},
                }
                unsafe {
                    (*gecko_gradient).mBgPosX.set(position.horizontal);
                    (*gecko_gradient).mBgPosY.set(position.vertical);
                }

                gecko_gradient
            },
        };

        for (index, item) in gradient.items.iter().enumerate() {
            // NB: stops are guaranteed to be none in the gecko side by
            // default.

            let gecko_stop = unsafe { &mut (*gecko_gradient).mStops[index] };
            let mut coord = nsStyleCoord::null();

            match *item {
                GradientItem::ColorStop(ref stop) => {
                    gecko_stop.mColor = stop.color.into();
                    gecko_stop.mIsInterpolationHint = false;
                    coord.set(stop.position);
                },
                GradientItem::InterpolationHint(hint) => {
                    gecko_stop.mIsInterpolationHint = true;
                    coord.set(Some(hint));
                },
            }

            gecko_stop.mLocation.move_from(coord);
        }

        unsafe {
            bindings::Gecko_SetGradientImageValue(self, gecko_gradient);
        }
    }

    /// Converts into Image.
    pub unsafe fn into_image(self: &nsStyleImage) -> Option<Image> {
        use crate::gecko_bindings::structs::nsStyleImageType;
        use crate::values::computed::{MozImageRect, NumberOrPercentage};

        match self.mType {
            nsStyleImageType::eStyleImageType_Null => None,
            nsStyleImageType::eStyleImageType_Image => {
                let url = self.get_image_url();
                if self.mCropRect.mPtr.is_null() {
                    Some(GenericImage::Url(url))
                } else {
                    let ref rect = *self.mCropRect.mPtr;
                    match (
                        NumberOrPercentage::from_gecko_style_coord(&rect.data_at(0)),
                        NumberOrPercentage::from_gecko_style_coord(&rect.data_at(1)),
                        NumberOrPercentage::from_gecko_style_coord(&rect.data_at(2)),
                        NumberOrPercentage::from_gecko_style_coord(&rect.data_at(3)),
                    ) {
                        (Some(top), Some(right), Some(bottom), Some(left)) => {
                            Some(GenericImage::Rect(Box::new(MozImageRect {
                                url,
                                top,
                                right,
                                bottom,
                                left,
                            })))
                        },
                        _ => {
                            debug_assert!(
                                false,
                                "mCropRect could not convert to NumberOrPercentage"
                            );
                            None
                        },
                    }
                }
            },
            nsStyleImageType::eStyleImageType_Gradient => {
                Some(GenericImage::Gradient(self.get_gradient()))
            },
            nsStyleImageType::eStyleImageType_Element => {
                use crate::gecko_string_cache::Atom;
                let atom = bindings::Gecko_GetImageElement(self);
                Some(GenericImage::Element(Atom::from_raw(atom)))
            },
            _ => panic!("Unexpected image type"),
        }
    }

    unsafe fn get_image_url(&self) -> ComputedImageUrl {
        let image_request = bindings::Gecko_GetImageRequest(self)
            .as_ref()
            .expect("Null image request?");
        ComputedImageUrl::from_image_request(image_request)
    }

    unsafe fn get_gradient(self: &nsStyleImage) -> Box<Gradient> {
        use self::structs::NS_STYLE_GRADIENT_SIZE_CLOSEST_CORNER as CLOSEST_CORNER;
        use self::structs::NS_STYLE_GRADIENT_SIZE_CLOSEST_SIDE as CLOSEST_SIDE;
        use self::structs::NS_STYLE_GRADIENT_SIZE_FARTHEST_CORNER as FARTHEST_CORNER;
        use self::structs::NS_STYLE_GRADIENT_SIZE_FARTHEST_SIDE as FARTHEST_SIDE;
        use crate::values::computed::image::LineDirection;
        use crate::values::computed::position::Position;
        use crate::values::computed::Length;
        use crate::values::generics::image::{Circle, ColorStop, CompatMode, Ellipse};
        use crate::values::generics::image::{EndingShape, GradientKind, ShapeExtent};

        let gecko_gradient = bindings::Gecko_GetGradientImageValue(self)
            .as_ref()
            .unwrap();
        let angle = Angle::from_gecko_style_coord(&gecko_gradient.mAngle);
        let horizontal_style = LengthPercentage::from_gecko_style_coord(&gecko_gradient.mBgPosX);
        let vertical_style = LengthPercentage::from_gecko_style_coord(&gecko_gradient.mBgPosY);

        let kind = match gecko_gradient.mShape as u32 {
            structs::NS_STYLE_GRADIENT_SHAPE_LINEAR => {
                let line_direction = match (angle, horizontal_style, vertical_style) {
                    (Some(a), None, None) => LineDirection::Angle(a),
                    (None, Some(horizontal), Some(vertical)) => {
                        line_direction(horizontal, vertical)
                    },
                    (Some(_), Some(horizontal), Some(vertical)) => LineDirection::MozPosition(
                        Some(Position {
                            horizontal,
                            vertical,
                        }),
                        angle,
                    ),
                    _ => {
                        debug_assert!(
                            horizontal_style.is_none() && vertical_style.is_none(),
                            "Unexpected linear gradient direction"
                        );
                        LineDirection::MozPosition(None, None)
                    },
                };
                GradientKind::Linear(line_direction)
            },
            _ => {
                let gecko_size_to_keyword = |gecko_size| {
                    match gecko_size {
                        CLOSEST_SIDE => ShapeExtent::ClosestSide,
                        FARTHEST_SIDE => ShapeExtent::FarthestSide,
                        CLOSEST_CORNER => ShapeExtent::ClosestCorner,
                        FARTHEST_CORNER => ShapeExtent::FarthestCorner,
                        // FIXME: We should support ShapeExtent::Contain and ShapeExtent::Cover.
                        // But we can't choose those yet since Gecko does not support both values.
                        // https://bugzilla.mozilla.org/show_bug.cgi?id=1217664
                        _ => panic!("Found unexpected gecko_size"),
                    }
                };

                let shape = match gecko_gradient.mShape as u32 {
                    structs::NS_STYLE_GRADIENT_SHAPE_CIRCULAR => {
                        let circle = match gecko_gradient.mSize as u32 {
                            structs::NS_STYLE_GRADIENT_SIZE_EXPLICIT_SIZE => {
                                let radius =
                                    Length::from_gecko_style_coord(&gecko_gradient.mRadiusX)
                                        .expect("mRadiusX could not convert to Length");
                                debug_assert_eq!(
                                    radius,
                                    Length::from_gecko_style_coord(&gecko_gradient.mRadiusY)
                                        .unwrap()
                                );
                                Circle::Radius(radius)
                            },
                            size => Circle::Extent(gecko_size_to_keyword(size)),
                        };
                        EndingShape::Circle(circle)
                    },
                    structs::NS_STYLE_GRADIENT_SHAPE_ELLIPTICAL => {
                        let length_percentage_keyword = match gecko_gradient.mSize as u32 {
                            structs::NS_STYLE_GRADIENT_SIZE_EXPLICIT_SIZE => match (
                                LengthPercentage::from_gecko_style_coord(
                                    &gecko_gradient.mRadiusX,
                                ),
                                LengthPercentage::from_gecko_style_coord(
                                    &gecko_gradient.mRadiusY,
                                ),
                            ) {
                                (Some(x), Some(y)) => Ellipse::Radii(x, y),
                                _ => {
                                    debug_assert!(false,
                                                      "mRadiusX, mRadiusY could not convert to LengthPercentage");
                                    Ellipse::Radii(
                                        LengthPercentage::zero(),
                                        LengthPercentage::zero(),
                                    )
                                },
                            },
                            size => Ellipse::Extent(gecko_size_to_keyword(size)),
                        };
                        EndingShape::Ellipse(length_percentage_keyword)
                    },
                    _ => panic!("Found unexpected mShape"),
                };

                let position = match (horizontal_style, vertical_style) {
                    (Some(horizontal), Some(vertical)) => Position {
                        horizontal,
                        vertical,
                    },
                    _ => {
                        debug_assert!(
                            false,
                            "mRadiusX, mRadiusY could not convert to LengthPercentage"
                        );
                        Position {
                            horizontal: LengthPercentage::zero(),
                            vertical: LengthPercentage::zero(),
                        }
                    },
                };

                GradientKind::Radial(shape, position, angle)
            },
        };

        let items = gecko_gradient
            .mStops
            .iter()
            .map(|ref stop| {
                if stop.mIsInterpolationHint {
                    GradientItem::InterpolationHint(
                        LengthPercentage::from_gecko_style_coord(&stop.mLocation)
                            .expect("mLocation could not convert to LengthPercentage"),
                    )
                } else {
                    GradientItem::ColorStop(ColorStop {
                        color: stop.mColor.into(),
                        position: LengthPercentage::from_gecko_style_coord(&stop.mLocation),
                    })
                }
            })
            .collect();

        let compat_mode = if gecko_gradient.mMozLegacySyntax {
            CompatMode::Moz
        } else if gecko_gradient.mLegacySyntax {
            CompatMode::WebKit
        } else {
            CompatMode::Modern
        };

        Box::new(Gradient {
            items,
            repeating: gecko_gradient.mRepeating,
            kind,
            compat_mode,
        })
    }
}

pub mod basic_shape {
    //! Conversions from and to CSS shape representations.

    use crate::gecko::values::GeckoStyleCoordConvertible;
    use crate::gecko_bindings::structs;
    use crate::gecko_bindings::structs::{nsStyleCoord, nsStyleCorners};
    use crate::gecko_bindings::structs::{StyleBasicShape, StyleBasicShapeType};
    use crate::gecko_bindings::structs::{
        StyleGeometryBox, StyleShapeSource, StyleShapeSourceType,
    };
    use crate::gecko_bindings::sugar::ns_style_coord::{CoordDataMut, CoordDataValue};
    use crate::gecko_bindings::sugar::refptr::RefPtr;
    use crate::values::computed::basic_shape::{
        BasicShape, ClippingShape, FloatAreaShape, ShapeRadius,
    };
    use crate::values::computed::border::{BorderCornerRadius, BorderRadius};
    use crate::values::computed::length::LengthPercentage;
    use crate::values::computed::motion::OffsetPath;
    use crate::values::computed::position;
    use crate::values::computed::url::ComputedUrl;
    use crate::values::generics::basic_shape::{
        BasicShape as GenericBasicShape, InsetRect, Polygon,
    };
    use crate::values::generics::basic_shape::{Circle, Ellipse, Path, PolygonCoord};
    use crate::values::generics::basic_shape::{GeometryBox, ShapeBox, ShapeSource};
    use crate::values::generics::border::BorderRadius as GenericBorderRadius;
    use crate::values::generics::rect::Rect;
    use crate::values::generics::NonNegative;
    use crate::values::specified::SVGPathData;
    use std::borrow::Borrow;

    impl StyleShapeSource {
        /// Convert StyleShapeSource to ShapeSource except URL and Image
        /// types.
        fn into_shape_source<ReferenceBox, ImageOrUrl>(
            &self,
        ) -> Option<ShapeSource<BasicShape, ReferenceBox, ImageOrUrl>>
        where
            ReferenceBox: From<StyleGeometryBox>,
        {
            match self.mType {
                StyleShapeSourceType::None => Some(ShapeSource::None),
                StyleShapeSourceType::Box => Some(ShapeSource::Box(self.mReferenceBox.into())),
                StyleShapeSourceType::Shape => {
                    let other_shape = unsafe { &*self.__bindgen_anon_1.mBasicShape.as_ref().mPtr };
                    let shape = other_shape.into();
                    let reference_box = if self.mReferenceBox == StyleGeometryBox::NoBox {
                        None
                    } else {
                        Some(self.mReferenceBox.into())
                    };
                    Some(ShapeSource::Shape(shape, reference_box))
                },
                StyleShapeSourceType::URL | StyleShapeSourceType::Image => None,
                StyleShapeSourceType::Path => {
                    let path = self.to_svg_path().expect("expect an SVGPathData");
                    let fill = unsafe { &*self.__bindgen_anon_1.mSVGPath.as_ref().mPtr }.mFillRule;
                    Some(ShapeSource::Path(Path { fill, path }))
                },
            }
        }

        /// Generate a SVGPathData from StyleShapeSource if possible.
        fn to_svg_path(&self) -> Option<SVGPathData> {
            use crate::values::specified::svg_path::PathCommand;
            match self.mType {
                StyleShapeSourceType::Path => {
                    let gecko_path = unsafe { &*self.__bindgen_anon_1.mSVGPath.as_ref().mPtr };
                    let result: Vec<PathCommand> = gecko_path.mPath.iter().cloned().collect();
                    Some(SVGPathData::new(result.into_boxed_slice()))
                },
                _ => None,
            }
        }
    }

    impl<'a> From<&'a StyleShapeSource> for ClippingShape {
        fn from(other: &'a StyleShapeSource) -> Self {
            match other.mType {
                StyleShapeSourceType::URL => unsafe {
                    let shape_image = &*other.__bindgen_anon_1.mShapeImage.as_ref().mPtr;
                    let other_url =
                        RefPtr::new(*shape_image.__bindgen_anon_1.mURLValue.as_ref() as *mut _);
                    let url = ComputedUrl::from_url_value(other_url);
                    ShapeSource::ImageOrUrl(url)
                },
                StyleShapeSourceType::Image => {
                    unreachable!("ClippingShape doesn't support Image!");
                },
                _ => other
                    .into_shape_source()
                    .expect("Couldn't convert to StyleSource!"),
            }
        }
    }

    impl<'a> From<&'a StyleShapeSource> for FloatAreaShape {
        fn from(other: &'a StyleShapeSource) -> Self {
            match other.mType {
                StyleShapeSourceType::URL => {
                    unreachable!("FloatAreaShape doesn't support URL!");
                },
                StyleShapeSourceType::Image => unsafe {
                    let shape_image = &*other.__bindgen_anon_1.mShapeImage.as_ref().mPtr;
                    let image = shape_image.into_image().expect("Cannot convert to Image");
                    ShapeSource::ImageOrUrl(image)
                },
                _ => other
                    .into_shape_source()
                    .expect("Couldn't convert to StyleSource!"),
            }
        }
    }

    impl<'a> From<&'a StyleShapeSource> for OffsetPath {
        fn from(other: &'a StyleShapeSource) -> Self {
            match other.mType {
                StyleShapeSourceType::Path => {
                    OffsetPath::Path(other.to_svg_path().expect("Cannot convert to SVGPathData"))
                },
                StyleShapeSourceType::None => OffsetPath::none(),
                StyleShapeSourceType::Shape |
                StyleShapeSourceType::Box |
                StyleShapeSourceType::URL |
                StyleShapeSourceType::Image => unreachable!("Unsupported offset-path type"),
            }
        }
    }

    impl<'a> From<&'a StyleBasicShape> for BasicShape {
        fn from(other: &'a StyleBasicShape) -> Self {
            match other.mType {
                StyleBasicShapeType::Inset => {
                    let t = LengthPercentage::from_gecko_style_coord(&other.mCoordinates[0]);
                    let r = LengthPercentage::from_gecko_style_coord(&other.mCoordinates[1]);
                    let b = LengthPercentage::from_gecko_style_coord(&other.mCoordinates[2]);
                    let l = LengthPercentage::from_gecko_style_coord(&other.mCoordinates[3]);
                    let round: BorderRadius = (&other.mRadius).into();
                    let round = if round.all_zero() { None } else { Some(round) };
                    let rect = Rect::new(
                        t.expect("inset() offset should be a length, percentage, or calc value"),
                        r.expect("inset() offset should be a length, percentage, or calc value"),
                        b.expect("inset() offset should be a length, percentage, or calc value"),
                        l.expect("inset() offset should be a length, percentage, or calc value"),
                    );
                    GenericBasicShape::Inset(InsetRect { rect, round })
                },
                StyleBasicShapeType::Circle => GenericBasicShape::Circle(Circle {
                    radius: (&other.mCoordinates[0]).into(),
                    position: (&other.mPosition).into(),
                }),
                StyleBasicShapeType::Ellipse => GenericBasicShape::Ellipse(Ellipse {
                    semiaxis_x: (&other.mCoordinates[0]).into(),
                    semiaxis_y: (&other.mCoordinates[1]).into(),
                    position: (&other.mPosition).into(),
                }),
                StyleBasicShapeType::Polygon => {
                    let mut coords = Vec::with_capacity(other.mCoordinates.len() / 2);
                    for i in 0..(other.mCoordinates.len() / 2) {
                        let x = 2 * i;
                        let y = x + 1;
                        coords.push(PolygonCoord(
                            LengthPercentage::from_gecko_style_coord(&other.mCoordinates[x])
                                .expect(
                                    "polygon() coordinate should be a length, percentage, \
                                     or calc value",
                                ),
                            LengthPercentage::from_gecko_style_coord(&other.mCoordinates[y])
                                .expect(
                                    "polygon() coordinate should be a length, percentage, \
                                     or calc value",
                                ),
                        ))
                    }
                    GenericBasicShape::Polygon(Polygon {
                        fill: other.mFillRule,
                        coordinates: coords,
                    })
                },
            }
        }
    }

    impl<'a> From<&'a nsStyleCorners> for BorderRadius {
        fn from(other: &'a nsStyleCorners) -> Self {
            let get_corner = |index| {
                BorderCornerRadius::new(
                    NonNegative(
                        LengthPercentage::from_gecko_style_coord(&other.data_at(index)).expect(
                            "<border-radius> should be a length, percentage, or calc value",
                        ),
                    ),
                    NonNegative(
                        LengthPercentage::from_gecko_style_coord(&other.data_at(index + 1))
                            .expect(
                                "<border-radius> should be a length, percentage, or calc value",
                            ),
                    ),
                )
            };

            GenericBorderRadius {
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
            let mut set_corner = |field: &BorderCornerRadius, index| {
                field
                    .0
                    .width()
                    .to_gecko_style_coord(&mut other.data_at_mut(index));
                field
                    .0
                    .height()
                    .to_gecko_style_coord(&mut other.data_at_mut(index + 1));
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
                mYPosition: other.vertical.into(),
            }
        }
    }

    impl<'a> From<&'a nsStyleCoord> for ShapeRadius {
        fn from(other: &'a nsStyleCoord) -> Self {
            let other = other.borrow();
            ShapeRadius::from_gecko_style_coord(other)
                .expect("<shape-radius> should be a length, percentage, calc, or keyword value")
        }
    }

    impl<'a> From<&'a structs::Position> for position::Position {
        fn from(other: &'a structs::Position) -> Self {
            position::Position {
                horizontal: other.mXPosition.into(),
                vertical: other.mYPosition.into(),
            }
        }
    }

    impl From<ShapeBox> for StyleGeometryBox {
        fn from(reference: ShapeBox) -> Self {
            use crate::gecko_bindings::structs::StyleGeometryBox::*;
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
            use crate::gecko_bindings::structs::StyleGeometryBox::*;
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
            use crate::gecko_bindings::structs::StyleGeometryBox::*;
            match reference {
                ContentBox => GeometryBox::ShapeBox(ShapeBox::ContentBox),
                PaddingBox => GeometryBox::ShapeBox(ShapeBox::PaddingBox),
                BorderBox => GeometryBox::ShapeBox(ShapeBox::BorderBox),
                MarginBox => GeometryBox::ShapeBox(ShapeBox::MarginBox),
                FillBox => GeometryBox::FillBox,
                StrokeBox => GeometryBox::StrokeBox,
                ViewBox => GeometryBox::ViewBox,
                _ => panic!("Unexpected StyleGeometryBox while converting to GeometryBox"),
            }
        }
    }

    impl From<StyleGeometryBox> for ShapeBox {
        fn from(reference: StyleGeometryBox) -> Self {
            use crate::gecko_bindings::structs::StyleGeometryBox::*;
            match reference {
                ContentBox => ShapeBox::ContentBox,
                PaddingBox => ShapeBox::PaddingBox,
                BorderBox => ShapeBox::BorderBox,
                MarginBox => ShapeBox::MarginBox,
                _ => panic!("Unexpected StyleGeometryBox while converting to ShapeBox"),
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

impl TrackSize<LengthPercentage> {
    /// Return TrackSize from given two nsStyleCoord
    pub fn from_gecko_style_coords<T: CoordData>(gecko_min: &T, gecko_max: &T) -> Self {
        use crate::gecko_bindings::structs::root::nsStyleUnit;
        use crate::values::computed::length::LengthPercentage;
        use crate::values::generics::grid::{TrackBreadth, TrackSize};

        if gecko_min.unit() == nsStyleUnit::eStyleUnit_None {
            debug_assert!(
                gecko_max.unit() == nsStyleUnit::eStyleUnit_Coord ||
                    gecko_max.unit() == nsStyleUnit::eStyleUnit_Percent ||
                    gecko_max.unit() == nsStyleUnit::eStyleUnit_Calc
            );
            return TrackSize::FitContent(
                LengthPercentage::from_gecko_style_coord(gecko_max)
                    .expect("gecko_max could not convert to LengthPercentage"),
            );
        }

        let min = TrackBreadth::from_gecko_style_coord(gecko_min)
            .expect("gecko_min could not convert to TrackBreadth");
        let max = TrackBreadth::from_gecko_style_coord(gecko_max)
            .expect("gecko_max could not convert to TrackBreadth");
        if min == max {
            TrackSize::Breadth(max)
        } else {
            TrackSize::Minmax(min, max)
        }
    }

    /// Save TrackSize to given gecko fields.
    pub fn to_gecko_style_coords<T: CoordDataMut>(&self, gecko_min: &mut T, gecko_max: &mut T) {
        use crate::values::generics::grid::TrackSize;

        match *self {
            TrackSize::FitContent(ref lop) => {
                // Gecko sets min value to None and max value to the actual value in fit-content
                // https://searchfox.org/mozilla-central/rev/c05d9d61188d32b8/layout/style/nsRuleNode.cpp#7910
                gecko_min.set_value(CoordDataValue::None);
                lop.to_gecko_style_coord(gecko_max);
            },
            TrackSize::Breadth(ref breadth) => {
                // Set the value to both fields if there's one breadth value
                // https://searchfox.org/mozilla-central/rev/c05d9d61188d32b8/layout/style/nsRuleNode.cpp#7919
                breadth.to_gecko_style_coord(gecko_min);
                breadth.to_gecko_style_coord(gecko_max);
            },
            TrackSize::Minmax(ref min, ref max) => {
                min.to_gecko_style_coord(gecko_min);
                max.to_gecko_style_coord(gecko_max);
            },
        }
    }
}

impl TrackListValue<LengthPercentage, Integer> {
    /// Return TrackSize from given two nsStyleCoord
    pub fn from_gecko_style_coords<T: CoordData>(gecko_min: &T, gecko_max: &T) -> Self {
        TrackListValue::TrackSize(TrackSize::from_gecko_style_coords(gecko_min, gecko_max))
    }

    /// Save TrackSize to given gecko fields.
    pub fn to_gecko_style_coords<T: CoordDataMut>(&self, gecko_min: &mut T, gecko_max: &mut T) {
        use crate::values::generics::grid::TrackListValue;

        match *self {
            TrackListValue::TrackSize(ref size) => size.to_gecko_style_coords(gecko_min, gecko_max),
            _ => unreachable!("Should only transform from track-size computed values"),
        }
    }
}

impl<T> Rect<T>
where
    T: GeckoStyleCoordConvertible,
{
    /// Convert this generic Rect to given Gecko fields.
    pub fn to_gecko_rect(&self, sides: &mut crate::gecko_bindings::structs::nsStyleSides) {
        self.0.to_gecko_style_coord(&mut sides.data_at_mut(0));
        self.1.to_gecko_style_coord(&mut sides.data_at_mut(1));
        self.2.to_gecko_style_coord(&mut sides.data_at_mut(2));
        self.3.to_gecko_style_coord(&mut sides.data_at_mut(3));
    }

    /// Convert from given Gecko data to generic Rect.
    pub fn from_gecko_rect(
        sides: &crate::gecko_bindings::structs::nsStyleSides,
    ) -> Option<crate::values::generics::rect::Rect<T>> {
        use crate::values::generics::rect::Rect;

        Some(Rect::new(
            T::from_gecko_style_coord(&sides.data_at(0)).expect("coord[0] cound not convert"),
            T::from_gecko_style_coord(&sides.data_at(1)).expect("coord[1] cound not convert"),
            T::from_gecko_style_coord(&sides.data_at(2)).expect("coord[2] cound not convert"),
            T::from_gecko_style_coord(&sides.data_at(3)).expect("coord[3] cound not convert"),
        ))
    }
}

impl<L> VerticalAlign<L> {
    /// Converts an enumerated value coming from Gecko to a `VerticalAlign<L>`.
    pub fn from_gecko_keyword(value: u32) -> Self {
        match value {
            structs::NS_STYLE_VERTICAL_ALIGN_BASELINE => VerticalAlign::Baseline,
            structs::NS_STYLE_VERTICAL_ALIGN_SUB => VerticalAlign::Sub,
            structs::NS_STYLE_VERTICAL_ALIGN_SUPER => VerticalAlign::Super,
            structs::NS_STYLE_VERTICAL_ALIGN_TOP => VerticalAlign::Top,
            structs::NS_STYLE_VERTICAL_ALIGN_TEXT_TOP => VerticalAlign::TextTop,
            structs::NS_STYLE_VERTICAL_ALIGN_MIDDLE => VerticalAlign::Middle,
            structs::NS_STYLE_VERTICAL_ALIGN_BOTTOM => VerticalAlign::Bottom,
            structs::NS_STYLE_VERTICAL_ALIGN_TEXT_BOTTOM => VerticalAlign::TextBottom,
            structs::NS_STYLE_VERTICAL_ALIGN_MIDDLE_WITH_BASELINE => {
                VerticalAlign::MozMiddleWithBaseline
            },
            _ => panic!("unexpected enumerated value for vertical-align"),
        }
    }
}

impl TextAlign {
    /// Obtain a specified value from a Gecko keyword value
    ///
    /// Intended for use with presentation attributes, not style structs
    pub fn from_gecko_keyword(kw: u32) -> Self {
        match kw {
            structs::NS_STYLE_TEXT_ALIGN_LEFT => TextAlign::Left,
            structs::NS_STYLE_TEXT_ALIGN_RIGHT => TextAlign::Right,
            structs::NS_STYLE_TEXT_ALIGN_CENTER => TextAlign::Center,
            structs::NS_STYLE_TEXT_ALIGN_JUSTIFY => TextAlign::Justify,
            structs::NS_STYLE_TEXT_ALIGN_MOZ_LEFT => TextAlign::MozLeft,
            structs::NS_STYLE_TEXT_ALIGN_MOZ_RIGHT => TextAlign::MozRight,
            structs::NS_STYLE_TEXT_ALIGN_MOZ_CENTER => TextAlign::MozCenter,
            structs::NS_STYLE_TEXT_ALIGN_CHAR => TextAlign::Char,
            structs::NS_STYLE_TEXT_ALIGN_END => TextAlign::End,
            _ => panic!("Found unexpected value in style struct for text-align property"),
        }
    }
}

/// Convert to String from given chars pointer.
pub unsafe fn string_from_chars_pointer(p: *const u16) -> String {
    use std::slice;
    let mut length = 0;
    let mut iter = p;
    while *iter != 0 {
        length += 1;
        iter = iter.offset(1);
    }
    let char_vec = slice::from_raw_parts(p, length as usize);
    String::from_utf16_lossy(char_vec)
}

impl<'a> From<&'a RawGeckoGfxMatrix4x4> for Matrix3D {
    fn from(m: &'a RawGeckoGfxMatrix4x4) -> Matrix3D {
        Matrix3D {
            m11: m[0],
            m12: m[1],
            m13: m[2],
            m14: m[3],
            m21: m[4],
            m22: m[5],
            m23: m[6],
            m24: m[7],
            m31: m[8],
            m32: m[9],
            m33: m[10],
            m34: m[11],
            m41: m[12],
            m42: m[13],
            m43: m[14],
            m44: m[15],
        }
    }
}

impl From<Matrix3D> for RawGeckoGfxMatrix4x4 {
    fn from(matrix: Matrix3D) -> RawGeckoGfxMatrix4x4 {
        [
            matrix.m11, matrix.m12, matrix.m13, matrix.m14, matrix.m21, matrix.m22, matrix.m23,
            matrix.m24, matrix.m31, matrix.m32, matrix.m33, matrix.m34, matrix.m41, matrix.m42,
            matrix.m43, matrix.m44,
        ]
    }
}
