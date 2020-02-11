/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! This module contains conversion helpers between Servo and Gecko types
//! Ideally, it would be in geckolib itself, but coherence
//! forces us to keep the traits and implementations here
//!
//! FIXME(emilio): This file should generally just die.

#![allow(unsafe_code)]

use crate::gecko_bindings::structs::{self, Matrix4x4Components, nsresult};
use crate::stylesheets::RulesMutateError;
use crate::values::computed::transform::Matrix3D;
use crate::values::computed::TextAlign;

pub mod basic_shape {
    //! Conversions from and to CSS shape representations.
    use crate::gecko_bindings::structs::{
        StyleGeometryBox, StyleShapeSource, StyleShapeSourceType,
    };
    use crate::values::computed::basic_shape::{BasicShape, ClippingShape, FloatAreaShape};
    use crate::values::computed::motion::OffsetPath;
    use crate::values::generics::basic_shape::{ShapeGeometryBox, Path, ShapeBox, ShapeSource};
    use crate::values::specified::SVGPathData;

    impl StyleShapeSource {
        /// Convert StyleShapeSource to ShapeSource except URL and Image
        /// types.
        fn to_shape_source<ReferenceBox, ImageOrUrl>(
            &self,
        ) -> Option<ShapeSource<BasicShape, ReferenceBox, ImageOrUrl>>
        where
            ReferenceBox: From<StyleGeometryBox> + Default + PartialEq,
        {
            match self.mType {
                StyleShapeSourceType::None => Some(ShapeSource::None),
                StyleShapeSourceType::Box => Some(ShapeSource::Box(self.mReferenceBox.into())),
                StyleShapeSourceType::Shape => {
                    let other_shape = unsafe { &*self.__bindgen_anon_1.mBasicShape.as_ref().mPtr };
                    let shape = Box::new(other_shape.clone());
                    let reference_box = self.mReferenceBox.into();
                    Some(ShapeSource::Shape(shape, reference_box))
                },
                StyleShapeSourceType::Image => None,
                StyleShapeSourceType::Path => {
                    let path = self.to_svg_path().expect("expect an SVGPathData");
                    let fill = unsafe { &*self.__bindgen_anon_1.mSVGPath.as_ref().mPtr }.mFillRule;
                    Some(ShapeSource::Path(Path { fill, path }))
                },
            }
        }

        /// Generate a SVGPathData from StyleShapeSource if possible.
        fn to_svg_path(&self) -> Option<SVGPathData> {
            match self.mType {
                StyleShapeSourceType::Path => {
                    let gecko_path = unsafe { &*self.__bindgen_anon_1.mSVGPath.as_ref().mPtr };
                    Some(SVGPathData(gecko_path.mPath.clone()))
                },
                _ => None,
            }
        }
    }

    impl<'a> From<&'a StyleShapeSource> for ClippingShape {
        fn from(other: &'a StyleShapeSource) -> Self {
            match other.mType {
                StyleShapeSourceType::Image => unsafe {
                    use crate::values::generics::image::Image as GenericImage;

                    let shape_image = &*other.__bindgen_anon_1.mShapeImage.as_ref().mPtr;
                    match *shape_image {
                        GenericImage::Url(ref url) => ShapeSource::ImageOrUrl(url.0.clone()),
                        _ => panic!("ClippingShape doesn't support non-url images"),
                    }
                },
                _ => other
                    .to_shape_source()
                    .expect("Couldn't convert to StyleSource!"),
            }
        }
    }

    impl<'a> From<&'a StyleShapeSource> for FloatAreaShape {
        fn from(other: &'a StyleShapeSource) -> Self {
            match other.mType {
                StyleShapeSourceType::Image => unsafe {
                    let shape_image = &*other.__bindgen_anon_1.mShapeImage.as_ref().mPtr;
                    ShapeSource::ImageOrUrl(shape_image.clone())
                },
                _ => other
                    .to_shape_source()
                    .expect("Couldn't convert to StyleSource!"),
            }
        }
    }

    impl<'a> From<&'a StyleShapeSource> for OffsetPath {
        fn from(other: &'a StyleShapeSource) -> Self {
            use crate::values::generics::motion::GenericOffsetPath;
            match other.mType {
                StyleShapeSourceType::Path => GenericOffsetPath::Path(
                    other.to_svg_path().expect("Cannot convert to SVGPathData"),
                ),
                StyleShapeSourceType::None => OffsetPath::none(),
                StyleShapeSourceType::Shape |
                StyleShapeSourceType::Box |
                StyleShapeSourceType::Image => unreachable!("Unsupported offset-path type"),
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

    impl From<ShapeGeometryBox> for StyleGeometryBox {
        fn from(reference: ShapeGeometryBox) -> Self {
            use crate::gecko_bindings::structs::StyleGeometryBox::*;
            match reference {
                ShapeGeometryBox::ShapeBox(shape_box) => From::from(shape_box),
                ShapeGeometryBox::FillBox => FillBox,
                ShapeGeometryBox::StrokeBox => StrokeBox,
                ShapeGeometryBox::ViewBox => ViewBox,
                ShapeGeometryBox::ElementDependent => NoBox,
            }
        }
    }

    impl From<StyleGeometryBox> for ShapeGeometryBox {
        fn from(reference: StyleGeometryBox) -> Self {
            use crate::gecko_bindings::structs::StyleGeometryBox::*;
            match reference {
                ContentBox => ShapeGeometryBox::ShapeBox(ShapeBox::ContentBox),
                PaddingBox => ShapeGeometryBox::ShapeBox(ShapeBox::PaddingBox),
                BorderBox => ShapeGeometryBox::ShapeBox(ShapeBox::BorderBox),
                MarginBox => ShapeGeometryBox::ShapeBox(ShapeBox::MarginBox),
                FillBox => ShapeGeometryBox::FillBox,
                StrokeBox => ShapeGeometryBox::StrokeBox,
                ViewBox => ShapeGeometryBox::ViewBox,
                NoBox => ShapeGeometryBox::ElementDependent,
                NoClip | Text | MozAlmostPadding => unreachable!(),
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

impl<'a> From<&'a Matrix4x4Components> for Matrix3D {
    fn from(m: &'a Matrix4x4Components) -> Matrix3D {
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

impl From<Matrix3D> for Matrix4x4Components {
    fn from(matrix: Matrix3D) -> Self {
        [
            matrix.m11, matrix.m12, matrix.m13, matrix.m14, matrix.m21, matrix.m22, matrix.m23,
            matrix.m24, matrix.m31, matrix.m32, matrix.m33, matrix.m34, matrix.m41, matrix.m42,
            matrix.m43, matrix.m44,
        ]
    }
}
