/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use app_units::Au;
use cssparser::{Color as CSSParserColor, Parser, RGBA, ToCss};
use euclid::{Point2D, Size2D};
use properties::PropertyDeclaration;
use properties::longhands;
use properties::longhands::background_position::computed_value::T as BackgroundPosition;
use properties::longhands::background_size::computed_value::T as BackgroundSize;
use properties::longhands::border_spacing::computed_value::T as BorderSpacing;
use properties::longhands::clip::computed_value::ClipRect;
use properties::longhands::font_weight::computed_value::T as FontWeight;
use properties::longhands::line_height::computed_value::T as LineHeight;
use properties::longhands::text_shadow::computed_value::T as TextShadowList;
use properties::longhands::text_shadow::computed_value::TextShadow;
use properties::longhands::transform::computed_value::ComputedMatrix;
use properties::longhands::transform::computed_value::ComputedOperation as TransformOperation;
use properties::longhands::transform::computed_value::T as TransformList;
use properties::longhands::vertical_align::computed_value::T as VerticalAlign;
use properties::longhands::visibility::computed_value::T as Visibility;
use properties::longhands::z_index::computed_value::T as ZIndex;
use properties::style_struct_traits::*;
use std::cmp::Ordering;
use std::fmt;
use std::iter::repeat;
use super::ComputedValues;
use values::computed::{Angle, LengthOrPercentageOrAuto, LengthOrPercentageOrNone};
use values::computed::{BorderRadiusSize, LengthOrNone};
use values::computed::{CalcLengthOrPercentage, LengthOrPercentage};

// NB: This needs to be here because it needs all the longhands generated
// beforehand.
#[derive(Copy, Clone, Debug, PartialEq, HeapSizeOf)]
pub enum TransitionProperty {
    All,
    % for prop in data.longhands:
        % if prop.animatable:
            ${prop.camel_case},
        % endif
    % endfor
}

impl TransitionProperty {
    /// Iterates over each property that is not `All`.
    pub fn each<F: FnMut(TransitionProperty) -> ()>(mut cb: F) {
        % for prop in data.longhands:
            % if prop.animatable:
                cb(TransitionProperty::${prop.camel_case});
            % endif
        % endfor
    }

    pub fn parse(input: &mut Parser) -> Result<Self, ()> {
        match_ignore_ascii_case! { try!(input.expect_ident()),
            "all" => Ok(TransitionProperty::All),
            % for prop in data.longhands:
                % if prop.animatable:
                    "${prop.name}" => Ok(TransitionProperty::${prop.camel_case}),
                % endif
            % endfor
            _ => Err(())
        }
    }

    pub fn from_declaration(declaration: &PropertyDeclaration) -> Option<Self> {
        match *declaration {
            % for prop in data.longhands:
                % if prop.animatable:
                    PropertyDeclaration::${prop.camel_case}(..)
                        => Some(TransitionProperty::${prop.camel_case}),
                % endif
            % endfor
            _ => None,
        }
    }
}

impl ToCss for TransitionProperty {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match *self {
            TransitionProperty::All => dest.write_str("all"),
            % for prop in data.longhands:
                % if prop.animatable:
                    TransitionProperty::${prop.camel_case} => dest.write_str("${prop.name}"),
                % endif
            % endfor
        }
    }
}

#[derive(Clone, Debug, PartialEq, HeapSizeOf)]
pub enum AnimatedProperty {
    % for prop in data.longhands:
        % if prop.animatable:
            ${prop.camel_case}(longhands::${prop.ident}::computed_value::T,
                               longhands::${prop.ident}::computed_value::T),
        % endif
    % endfor
}

impl AnimatedProperty {
    pub fn does_animate(&self) -> bool {
        match *self {
            % for prop in data.longhands:
                % if prop.animatable:
                    AnimatedProperty::${prop.camel_case}(ref from, ref to) => from != to,
                % endif
            % endfor
        }
    }

    pub fn update<C: ComputedValues>(&self, style: &mut C, progress: f64) {
        match *self {
            % for prop in data.longhands:
                % if prop.animatable:
                    AnimatedProperty::${prop.camel_case}(ref from, ref to) => {
                        if let Some(value) = from.interpolate(to, progress) {
                            style.mutate_${prop.style_struct.ident.strip("_")}().set_${prop.ident}(value);
                        }
                    }
                % endif
            % endfor
        }
    }

    // NB: Transition properties need clone
    pub fn from_transition_property<C: ComputedValues>(transition_property: &TransitionProperty,
                                                       old_style: &C,
                                                       new_style: &C) -> AnimatedProperty {
        // TODO: Generalise this for GeckoLib, adding clone_xxx to the
        // appropiate longhands.
        let old_style = old_style.as_servo();
        let new_style = new_style.as_servo();
        match *transition_property {
            TransitionProperty::All => panic!("Can't use TransitionProperty::All here."),
            % for prop in data.longhands:
                % if prop.animatable:
                    TransitionProperty::${prop.camel_case} => {
                        AnimatedProperty::${prop.camel_case}(
                            old_style.get_${prop.style_struct.ident.strip("_")}().${prop.ident}.clone(),
                            new_style.get_${prop.style_struct.ident.strip("_")}().${prop.ident}.clone())
                    }
                % endif
            % endfor
        }
    }
}

pub trait Interpolate: Sized {
    fn interpolate(&self, other: &Self, time: f64) -> Option<Self>;
}

impl Interpolate for Au {
    #[inline]
    fn interpolate(&self, other: &Au, time: f64) -> Option<Au> {
        Some(Au((self.0 as f64 + (other.0 as f64 - self.0 as f64) * time).round() as i32))
    }
}

impl <T> Interpolate for Option<T> where T: Interpolate {
    #[inline]
    fn interpolate(&self, other: &Option<T>, time: f64) -> Option<Option<T>> {
        match (self, other) {
            (&Some(ref this), &Some(ref other)) => {
                this.interpolate(other, time).and_then(|value| {
                    Some(Some(value))
                })
            }
            (_, _) => None
        }
    }
}

impl Interpolate for f32 {
    #[inline]
    fn interpolate(&self, other: &f32, time: f64) -> Option<f32> {
        Some(((*self as f64) + ((*other as f64) - (*self as f64)) * time) as f32)
    }
}

impl Interpolate for f64 {
    #[inline]
    fn interpolate(&self, other: &f64, time: f64) -> Option<f64> {
        Some(*self + (*other - *self) * time)
    }
}

impl Interpolate for i32 {
    #[inline]
    fn interpolate(&self, other: &i32, time: f64) -> Option<i32> {
        let a = *self as f64;
        let b = *other as f64;
        Some((a + (b - a) * time).round() as i32)
    }
}

impl Interpolate for Angle {
    #[inline]
    fn interpolate(&self, other: &Angle, time: f64) -> Option<Angle> {
        self.radians().interpolate(&other.radians(), time).map(Angle)
    }
}

impl Interpolate for Visibility {
    #[inline]
    fn interpolate(&self, other: &Visibility, time: f64)
                   -> Option<Visibility> {
        match (*self, *other) {
            (Visibility::visible, _) | (_, Visibility::visible) => {
                if time >= 0.0 && time <= 1.0 {
                    Some(Visibility::visible)
                } else if time < 0.0 {
                    Some(*self)
                } else {
                    Some(*other)
                }
            }
            (_, _) => None,
        }
    }
}

impl Interpolate for ZIndex {
    #[inline]
    fn interpolate(&self, other: &ZIndex, time: f64)
                   -> Option<ZIndex> {
        match (*self, *other) {
            (ZIndex::Number(ref this),
             ZIndex::Number(ref other)) => {
                this.interpolate(other, time).and_then(|value| {
                    Some(ZIndex::Number(value))
                })
            }
            (_, _) => None,
        }
    }
}

impl<T: Interpolate + Clone> Interpolate for Size2D<T> {
    #[inline]
    fn interpolate(&self, other: &Self, time: f64) -> Option<Self> {
        let width = match self.width.interpolate(&other.width, time) {
            Some(width) => width,
            None => return None,
        };

        let height = match self.height.interpolate(&other.height, time) {
            Some(height) => height,
            None => return None,
        };
        Some(Size2D::new(width, height))
    }
}

impl<T: Interpolate + Clone> Interpolate for Point2D<T> {
    #[inline]
    fn interpolate(&self, other: &Self, time: f64) -> Option<Self> {
        let x = match self.x.interpolate(&other.x, time) {
            Some(x) => x,
            None => return None,
        };

        let y = match self.y.interpolate(&other.y, time) {
            Some(y) => y,
            None => return None,
        };

        Some(Point2D::new(x, y))
    }
}

impl Interpolate for BorderRadiusSize {
    #[inline]
    fn interpolate(&self, other: &Self, time: f64) -> Option<Self> {
        self.0.interpolate(&other.0, time).map(BorderRadiusSize)
    }
}

impl Interpolate for VerticalAlign {
    #[inline]
    fn interpolate(&self, other: &VerticalAlign, time: f64)
                   -> Option<VerticalAlign> {
        match (*self, *other) {
            (VerticalAlign::LengthOrPercentage(LengthOrPercentage::Length(ref this)),
             VerticalAlign::LengthOrPercentage(LengthOrPercentage::Length(ref other))) => {
                this.interpolate(other, time).and_then(|value| {
                    Some(VerticalAlign::LengthOrPercentage(LengthOrPercentage::Length(value)))
                })
            }
            (_, _) => None,
        }
    }
}

impl Interpolate for BorderSpacing {
    #[inline]
    fn interpolate(&self, other: &BorderSpacing, time: f64)
                   -> Option<BorderSpacing> {
        self.horizontal.interpolate(&other.horizontal, time).and_then(|horizontal| {
            self.vertical.interpolate(&other.vertical, time).and_then(|vertical| {
                Some(BorderSpacing { horizontal: horizontal, vertical: vertical })
            })
        })
    }
}

impl Interpolate for RGBA {
    #[inline]
    fn interpolate(&self, other: &RGBA, time: f64) -> Option<RGBA> {
        match (self.red.interpolate(&other.red, time),
               self.green.interpolate(&other.green, time),
               self.blue.interpolate(&other.blue, time),
               self.alpha.interpolate(&other.alpha, time)) {
            (Some(red), Some(green), Some(blue), Some(alpha)) => {
                Some(RGBA { red: red, green: green, blue: blue, alpha: alpha })
            }
            (_, _, _, _) => None
        }
    }
}

impl Interpolate for CSSParserColor {
    #[inline]
    fn interpolate(&self, other: &Self, time: f64) -> Option<Self> {
        match (*self, *other) {
            (CSSParserColor::RGBA(ref this), CSSParserColor::RGBA(ref other)) => {
                this.interpolate(other, time).and_then(|value| {
                    Some(CSSParserColor::RGBA(value))
                })
            }
            (_, _) => None,
        }
    }
}

impl Interpolate for CalcLengthOrPercentage {
    #[inline]
    fn interpolate(&self, other: &CalcLengthOrPercentage, time: f64)
                   -> Option<CalcLengthOrPercentage> {
        Some(CalcLengthOrPercentage {
            length: self.length().interpolate(&other.length(), time),
            percentage: self.percentage().interpolate(&other.percentage(), time),
        })
    }
}

impl Interpolate for LengthOrPercentage {
    #[inline]
    fn interpolate(&self, other: &LengthOrPercentage, time: f64)
                   -> Option<LengthOrPercentage> {
        match (*self, *other) {
            (LengthOrPercentage::Length(ref this),
             LengthOrPercentage::Length(ref other)) => {
                this.interpolate(other, time).and_then(|value| {
                    Some(LengthOrPercentage::Length(value))
                })
            }
            (LengthOrPercentage::Percentage(ref this),
             LengthOrPercentage::Percentage(ref other)) => {
                this.interpolate(other, time).and_then(|value| {
                    Some(LengthOrPercentage::Percentage(value))
                })
            }
            (this, other) => {
                let this: CalcLengthOrPercentage = From::from(this);
                let other: CalcLengthOrPercentage = From::from(other);
                this.interpolate(&other, time).and_then(|value| {
                    Some(LengthOrPercentage::Calc(value))
                })
            }
        }
    }
}

impl Interpolate for LengthOrPercentageOrAuto {
    #[inline]
    fn interpolate(&self, other: &LengthOrPercentageOrAuto, time: f64)
                   -> Option<LengthOrPercentageOrAuto> {
        match (*self, *other) {
            (LengthOrPercentageOrAuto::Length(ref this),
             LengthOrPercentageOrAuto::Length(ref other)) => {
                this.interpolate(other, time).and_then(|value| {
                    Some(LengthOrPercentageOrAuto::Length(value))
                })
            }
            (LengthOrPercentageOrAuto::Percentage(ref this),
             LengthOrPercentageOrAuto::Percentage(ref other)) => {
                this.interpolate(other, time).and_then(|value| {
                    Some(LengthOrPercentageOrAuto::Percentage(value))
                })
            }
            (LengthOrPercentageOrAuto::Auto, LengthOrPercentageOrAuto::Auto) => {
                Some(LengthOrPercentageOrAuto::Auto)
            }
            (this, other) => {
                let this: Option<CalcLengthOrPercentage> = From::from(this);
                let other: Option<CalcLengthOrPercentage> = From::from(other);
                this.interpolate(&other, time).unwrap_or(None).and_then(|value| {
                    Some(LengthOrPercentageOrAuto::Calc(value))
                })
            }
        }
    }
}

impl Interpolate for LengthOrPercentageOrNone {
    #[inline]
    fn interpolate(&self, other: &LengthOrPercentageOrNone, time: f64)
                   -> Option<LengthOrPercentageOrNone> {
        match (*self, *other) {
            (LengthOrPercentageOrNone::Length(ref this),
             LengthOrPercentageOrNone::Length(ref other)) => {
                this.interpolate(other, time).and_then(|value| {
                    Some(LengthOrPercentageOrNone::Length(value))
                })
            }
            (LengthOrPercentageOrNone::Percentage(ref this),
             LengthOrPercentageOrNone::Percentage(ref other)) => {
                this.interpolate(other, time).and_then(|value| {
                    Some(LengthOrPercentageOrNone::Percentage(value))
                })
            }
            (LengthOrPercentageOrNone::None, LengthOrPercentageOrNone::None) => {
                Some(LengthOrPercentageOrNone::None)
            }
            (_, _) => None,
        }
    }
}

impl Interpolate for LineHeight {
    #[inline]
    fn interpolate(&self, other: &LineHeight, time: f64)
                   -> Option<LineHeight> {
        match (*self, *other) {
            (LineHeight::Length(ref this),
             LineHeight::Length(ref other)) => {
                this.interpolate(other, time).and_then(|value| {
                    Some(LineHeight::Length(value))
                })
            }
            (LineHeight::Number(ref this),
             LineHeight::Number(ref other)) => {
                this.interpolate(other, time).and_then(|value| {
                    Some(LineHeight::Number(value))
                })
            }
            (LineHeight::Normal, LineHeight::Normal) => {
                Some(LineHeight::Normal)
            }
            (_, _) => None,
        }
    }
}

/// http://dev.w3.org/csswg/css-transitions/#animtype-font-weight
impl Interpolate for FontWeight {
    #[inline]
    fn interpolate(&self, other: &FontWeight, time: f64)
                   -> Option<FontWeight> {
        let a = (*self as u32) as f64;
        let b = (*other as u32) as f64;
        let weight = a + (b - a) * time;
        Some(if weight < 150. {
            FontWeight::Weight100
        } else if weight < 250. {
            FontWeight::Weight200
        } else if weight < 350. {
            FontWeight::Weight300
        } else if weight < 450. {
            FontWeight::Weight400
        } else if weight < 550. {
            FontWeight::Weight500
        } else if weight < 650. {
            FontWeight::Weight600
        } else if weight < 750. {
            FontWeight::Weight700
        } else if weight < 850. {
            FontWeight::Weight800
        } else {
            FontWeight::Weight900
        })
    }
}

impl Interpolate for ClipRect {
    #[inline]
    fn interpolate(&self, other: &ClipRect, time: f64)
                   -> Option<ClipRect> {
        match (self.top.interpolate(&other.top, time),
               self.right.interpolate(&other.right, time),
               self.bottom.interpolate(&other.bottom, time),
               self.left.interpolate(&other.left, time)) {
            (Some(top), Some(right), Some(bottom), Some(left)) => {
                Some(ClipRect { top: top, right: right, bottom: bottom, left: left })
            },
            (_, _, _, _) => None,
        }
    }
}

impl Interpolate for BackgroundPosition {
    #[inline]
    fn interpolate(&self, other: &BackgroundPosition, time: f64)
                   -> Option<BackgroundPosition> {
        match (self.horizontal.interpolate(&other.horizontal, time),
               self.vertical.interpolate(&other.vertical, time)) {
            (Some(horizontal), Some(vertical)) => {
                Some(BackgroundPosition { horizontal: horizontal, vertical: vertical })
            },
            (_, _) => None,
        }
    }
}

impl Interpolate for BackgroundSize {
    fn interpolate(&self, other: &Self, time: f64) -> Option<Self> {
        use properties::longhands::background_size::computed_value::ExplicitSize;
        match (self, other) {
            (&BackgroundSize::Explicit(ref me), &BackgroundSize::Explicit(ref other))
                => match (me.width.interpolate(&other.width, time),
                          me.height.interpolate(&other.height, time)) {
                       (Some(width), Some(height))
                           => Some(BackgroundSize::Explicit(
                               ExplicitSize { width: width, height: height })),
                        _ => None,
                   },
            _ => None
        }
    }
}

impl Interpolate for TextShadow {
    #[inline]
    fn interpolate(&self, other: &TextShadow, time: f64)
                   -> Option<TextShadow> {
        match (self.offset_x.interpolate(&other.offset_x, time),
               self.offset_y.interpolate(&other.offset_y, time),
               self.blur_radius.interpolate(&other.blur_radius, time),
               self.color.interpolate(&other.color, time)) {
            (Some(offset_x), Some(offset_y), Some(blur_radius), Some(color)) => {
                Some(TextShadow { offset_x: offset_x, offset_y: offset_y, blur_radius: blur_radius, color: color })
            },
            (_, _, _, _) => None,
        }
    }
}

impl Interpolate for TextShadowList {
    #[inline]
    fn interpolate(&self, other: &TextShadowList, time: f64)
                   -> Option<TextShadowList> {
        let zero = TextShadow {
            offset_x: Au(0),
            offset_y: Au(0),
            blur_radius: Au(0),
            color: CSSParserColor::RGBA(RGBA {
                red: 0.0, green: 0.0, blue: 0.0, alpha: 0.0
            })
        };

        let interpolate_each = |(a, b): (&TextShadow, &TextShadow)| {
            a.interpolate(b, time).unwrap()
        };

        Some(TextShadowList(match self.0.len().cmp(&other.0.len()) {
            Ordering::Less => other.0.iter().chain(repeat(&zero)).zip(other.0.iter()).map(interpolate_each).collect(),
            _ => self.0.iter().zip(other.0.iter().chain(repeat(&zero))).map(interpolate_each).collect(),
        }))
    }
}

/// Check if it's possible to do a direct numerical interpolation
/// between these two transform lists.
/// http://dev.w3.org/csswg/css-transforms/#transform-transform-animation
fn can_interpolate_list(from_list: &[TransformOperation],
                        to_list: &[TransformOperation]) -> bool {
    // Lists must be equal length
    if from_list.len() != to_list.len() {
        return false;
    }

    // Each transform operation must match primitive type in other list
    for (from, to) in from_list.iter().zip(to_list) {
        match (from, to) {
            (&TransformOperation::Matrix(..), &TransformOperation::Matrix(..)) |
            (&TransformOperation::Skew(..), &TransformOperation::Skew(..)) |
            (&TransformOperation::Translate(..), &TransformOperation::Translate(..)) |
            (&TransformOperation::Scale(..), &TransformOperation::Scale(..)) |
            (&TransformOperation::Rotate(..), &TransformOperation::Rotate(..)) |
            (&TransformOperation::Perspective(..), &TransformOperation::Perspective(..)) => {}
            _ => {
                return false;
            }
        }
    }

    true
}

/// Interpolate two transform lists.
/// http://dev.w3.org/csswg/css-transforms/#interpolation-of-transforms
fn interpolate_transform_list(from_list: &[TransformOperation],
                              to_list: &[TransformOperation],
                              time: f64) -> TransformList {
    let mut result = vec![];

    if can_interpolate_list(from_list, to_list) {
        for (from, to) in from_list.iter().zip(to_list) {
            match (from, to) {
                (&TransformOperation::Matrix(from),
                 &TransformOperation::Matrix(_to)) => {
                    // TODO(gw): Implement matrix decomposition and interpolation
                    result.push(TransformOperation::Matrix(from));
                }
                (&TransformOperation::Skew(fx, fy),
                 &TransformOperation::Skew(tx, ty)) => {
                    let ix = fx.interpolate(&tx, time).unwrap();
                    let iy = fy.interpolate(&ty, time).unwrap();
                    result.push(TransformOperation::Skew(ix, iy));
                }
                (&TransformOperation::Translate(fx, fy, fz),
                 &TransformOperation::Translate(tx, ty, tz)) => {
                    let ix = fx.interpolate(&tx, time).unwrap();
                    let iy = fy.interpolate(&ty, time).unwrap();
                    let iz = fz.interpolate(&tz, time).unwrap();
                    result.push(TransformOperation::Translate(ix, iy, iz));
                }
                (&TransformOperation::Scale(fx, fy, fz),
                 &TransformOperation::Scale(tx, ty, tz)) => {
                    let ix = fx.interpolate(&tx, time).unwrap();
                    let iy = fy.interpolate(&ty, time).unwrap();
                    let iz = fz.interpolate(&tz, time).unwrap();
                    result.push(TransformOperation::Scale(ix, iy, iz));
                }
                (&TransformOperation::Rotate(fx, fy, fz, fa),
                 &TransformOperation::Rotate(_tx, _ty, _tz, _ta)) => {
                    // TODO(gw): Implement matrix decomposition and interpolation
                    result.push(TransformOperation::Rotate(fx, fy, fz, fa));
                }
                (&TransformOperation::Perspective(fd),
                 &TransformOperation::Perspective(_td)) => {
                    // TODO(gw): Implement matrix decomposition and interpolation
                    result.push(TransformOperation::Perspective(fd));
                }
                _ => {
                    // This should be unreachable due to the can_interpolate_list() call.
                    unreachable!();
                }
            }
        }
    } else {
        // TODO(gw): Implement matrix decomposition and interpolation
        result.extend_from_slice(from_list);
    }

    TransformList(Some(result))
}

/// Build an equivalent 'identity transform function list' based
/// on an existing transform list.
/// http://dev.w3.org/csswg/css-transforms/#none-transform-animation
fn build_identity_transform_list(list: &[TransformOperation]) -> Vec<TransformOperation> {
    let mut result = vec!();

    for operation in list {
        match *operation {
            TransformOperation::Matrix(..) => {
                let identity = ComputedMatrix::identity();
                result.push(TransformOperation::Matrix(identity));
            }
            TransformOperation::Skew(..) => {
                result.push(TransformOperation::Skew(Angle(0.0), Angle(0.0)));
            }
            TransformOperation::Translate(..) => {
                result.push(TransformOperation::Translate(LengthOrPercentage::zero(),
                                                          LengthOrPercentage::zero(),
                                                          Au(0)));
            }
            TransformOperation::Scale(..) => {
                result.push(TransformOperation::Scale(1.0, 1.0, 1.0));
            }
            TransformOperation::Rotate(..) => {
                result.push(TransformOperation::Rotate(0.0, 0.0, 1.0, Angle(0.0)));
            }
            TransformOperation::Perspective(..) => {
                // http://dev.w3.org/csswg/css-transforms/#identity-transform-function
                let identity = ComputedMatrix::identity();
                result.push(TransformOperation::Matrix(identity));
            }
        }
    }

    result
}

impl Interpolate for LengthOrNone {
    fn interpolate(&self, other: &Self, time: f64) -> Option<Self> {
        match (*self, *other) {
            (LengthOrNone::Length(ref len), LengthOrNone::Length(ref other)) =>
                len.interpolate(&other, time).map(LengthOrNone::Length),
            _ => None,
        }
    }
}

impl Interpolate for TransformList {
    #[inline]
    fn interpolate(&self, other: &TransformList, time: f64) -> Option<TransformList> {
        // http://dev.w3.org/csswg/css-transforms/#interpolation-of-transforms
        let result = match (&self.0, &other.0) {
            (&Some(ref from_list), &Some(ref to_list)) => {
                // Two lists of transforms
                interpolate_transform_list(from_list, &to_list, time)
            }
            (&Some(ref from_list), &None) => {
                // http://dev.w3.org/csswg/css-transforms/#none-transform-animation
                let to_list = build_identity_transform_list(from_list);
                interpolate_transform_list(from_list, &to_list, time)
            }
            (&None, &Some(ref to_list)) => {
                // http://dev.w3.org/csswg/css-transforms/#none-transform-animation
                let from_list = build_identity_transform_list(to_list);
                interpolate_transform_list(&from_list, to_list, time)
            }
            _ => {
                // http://dev.w3.org/csswg/css-transforms/#none-none-animation
                TransformList(None)
            }
        };

        Some(result)
    }
}
