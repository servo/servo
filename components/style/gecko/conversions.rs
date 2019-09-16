/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! This module contains conversion helpers between Servo and Gecko types
//! Ideally, it would be in geckolib itself, but coherence
//! forces us to keep the traits and implementations here
//!
//! FIXME(emilio): This file should generally just die.

#![allow(unsafe_code)]

use crate::gecko_bindings::bindings;
use crate::gecko_bindings::structs::{self, Matrix4x4Components};
use crate::gecko_bindings::structs::{nsStyleImage, nsresult};
use crate::stylesheets::RulesMutateError;
use crate::values::computed::transform::Matrix3D;
use crate::values::computed::url::ComputedImageUrl;
use crate::values::computed::{Gradient, Image, TextAlign};
use crate::values::generics::image::GenericImage;
use crate::values::generics::rect::Rect;

impl nsStyleImage {
    /// Set a given Servo `Image` value into this `nsStyleImage`.
    pub fn set(&mut self, image: Image) {
        match image {
            GenericImage::Gradient(boxed_gradient) => self.set_gradient(boxed_gradient),
            GenericImage::Url(ref url) => unsafe {
                bindings::Gecko_SetLayerImageImageValue(self, url);
            },
            GenericImage::Rect(ref image_rect) => {
                unsafe {
                    bindings::Gecko_SetLayerImageImageValue(self, &image_rect.url);
                    bindings::Gecko_InitializeImageCropRect(self);

                    // Set CropRect
                    let ref mut rect = *self.mCropRect.mPtr;
                    *rect = Rect(
                        image_rect.top,
                        image_rect.right,
                        image_rect.bottom,
                        image_rect.left,
                    );
                }
            }
            GenericImage::Element(ref element) => unsafe {
                bindings::Gecko_SetImageElement(self, element.as_ptr());
            },
        }
    }

    fn set_gradient(&mut self, gradient: Box<Gradient>) {
        unsafe {
            bindings::Gecko_SetGradientImageValue(self, Box::into_raw(gradient));
        }
    }

    /// Converts into Image.
    pub unsafe fn into_image(self: &nsStyleImage) -> Option<Image> {
        use crate::gecko_bindings::structs::nsStyleImageType;
        use crate::values::computed::MozImageRect;

        match self.mType {
            nsStyleImageType::eStyleImageType_Null => None,
            nsStyleImageType::eStyleImageType_Image => {
                let url = self.get_image_url();
                if self.mCropRect.mPtr.is_null() {
                    Some(GenericImage::Url(url))
                } else {
                    let rect = &*self.mCropRect.mPtr;
                    Some(GenericImage::Rect(Box::new(MozImageRect {
                        url,
                        top: rect.0,
                        right: rect.1,
                        bottom: rect.2,
                        left: rect.3,
                    })))
                }
            }
            nsStyleImageType::eStyleImageType_Gradient => {
                let gradient: &Gradient = &**self.__bindgen_anon_1.mGradient.as_ref();
                Some(GenericImage::Gradient(Box::new(gradient.clone())))
            }
            nsStyleImageType::eStyleImageType_Element => {
                use crate::gecko_string_cache::Atom;
                let atom = bindings::Gecko_GetImageElement(self);
                Some(GenericImage::Element(Atom::from_raw(atom)))
            }
        }
    }

    unsafe fn get_image_url(&self) -> ComputedImageUrl {
        let image_request = bindings::Gecko_GetImageRequest(self)
            .as_ref()
            .expect("Null image request?");
        ComputedImageUrl::from_image_request(image_request)
    }
}

pub mod basic_shape {
    //! Conversions from and to CSS shape representations.
    use crate::gecko_bindings::structs::{
        StyleGeometryBox, StyleShapeSource, StyleShapeSourceType,
    };
    use crate::values::computed::basic_shape::{BasicShape, ClippingShape, FloatAreaShape};
    use crate::values::computed::motion::OffsetPath;
    use crate::values::generics::basic_shape::{GeometryBox, Path, ShapeBox, ShapeSource};
    use crate::values::specified::SVGPathData;

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
                    let shape = Box::new(other_shape.clone());
                    let reference_box = if self.mReferenceBox == StyleGeometryBox::NoBox {
                        None
                    } else {
                        Some(self.mReferenceBox.into())
                    };
                    Some(ShapeSource::Shape(shape, reference_box))
                }
                StyleShapeSourceType::Image => None,
                StyleShapeSourceType::Path => {
                    let path = self.to_svg_path().expect("expect an SVGPathData");
                    let fill = unsafe { &*self.__bindgen_anon_1.mSVGPath.as_ref().mPtr }.mFillRule;
                    Some(ShapeSource::Path(Path { fill, path }))
                }
            }
        }

        /// Generate a SVGPathData from StyleShapeSource if possible.
        fn to_svg_path(&self) -> Option<SVGPathData> {
            match self.mType {
                StyleShapeSourceType::Path => {
                    let gecko_path = unsafe { &*self.__bindgen_anon_1.mSVGPath.as_ref().mPtr };
                    Some(SVGPathData(gecko_path.mPath.clone()))
                }
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
                    let image = shape_image.into_image().expect("Cannot convert to Image");
                    match image {
                        GenericImage::Url(url) => ShapeSource::ImageOrUrl(url.0),
                        _ => panic!("ClippingShape doesn't support non-url images"),
                    }
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
            use crate::values::generics::motion::GenericOffsetPath;
            match other.mType {
                StyleShapeSourceType::Path => GenericOffsetPath::Path(
                    other.to_svg_path().expect("Cannot convert to SVGPathData"),
                ),
                StyleShapeSourceType::None => OffsetPath::none(),
                StyleShapeSourceType::Shape
                | StyleShapeSourceType::Box
                | StyleShapeSourceType::Image => unreachable!("Unsupported offset-path type"),
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
