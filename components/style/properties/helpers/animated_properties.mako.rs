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
use properties::longhands::font_weight::computed_value::T as FontWeight;
use properties::longhands::line_height::computed_value::T as LineHeight;
use properties::longhands::text_shadow::computed_value::T as TextShadowList;
use properties::longhands::text_shadow::computed_value::TextShadow;
use properties::longhands::box_shadow::computed_value::T as BoxShadowList;
use properties::longhands::box_shadow::single_value::computed_value::T as BoxShadow;
use properties::longhands::vertical_align::computed_value::T as VerticalAlign;
use properties::longhands::visibility::computed_value::T as Visibility;
use properties::longhands::z_index::computed_value::T as ZIndex;
use std::cmp;
use std::fmt;
use super::ComputedValues;
use values::CSSFloat;
use values::computed::{Angle, LengthOrPercentageOrAuto, LengthOrPercentageOrNone};
use values::computed::{BorderRadiusSize, LengthOrNone};
use values::computed::{CalcLengthOrPercentage, LengthOrPercentage};
use values::computed::position::Position;
use values::specified::Angle as SpecifiedAngle;



// NB: This needs to be here because it needs all the longhands generated
// beforehand.
#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
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

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
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

    pub fn update(&self, style: &mut ComputedValues, progress: f64) {
        match *self {
            % for prop in data.longhands:
                % if prop.animatable:
                    AnimatedProperty::${prop.camel_case}(ref from, ref to) => {
                        if let Ok(value) = from.interpolate(to, progress) {
                            style.mutate_${prop.style_struct.ident.strip("_")}().set_${prop.ident}(value);
                        }
                    }
                % endif
            % endfor
        }
    }

    pub fn from_transition_property(transition_property: &TransitionProperty,
                                    old_style: &ComputedValues,
                                    new_style: &ComputedValues)
                                    -> AnimatedProperty {
        match *transition_property {
            TransitionProperty::All => panic!("Can't use TransitionProperty::All here."),
            % for prop in data.longhands:
                % if prop.animatable:
                    TransitionProperty::${prop.camel_case} => {
                        AnimatedProperty::${prop.camel_case}(
                            old_style.get_${prop.style_struct.ident.strip("_")}().clone_${prop.ident}(),
                            new_style.get_${prop.style_struct.ident.strip("_")}().clone_${prop.ident}())
                    }
                % endif
            % endfor
        }
    }
}

/// A trait used to implement [interpolation][interpolated-types].
///
/// [interpolated-types]: https://drafts.csswg.org/css-transitions/#interpolated-types
pub trait Interpolate: Sized {
    fn interpolate(&self, other: &Self, time: f64) -> Result<Self, ()>;
}

/// https://drafts.csswg.org/css-transitions/#animtype-repeatable-list
pub trait RepeatableListInterpolate: Interpolate {}

impl<T: RepeatableListInterpolate> Interpolate for Vec<T> {
    fn interpolate(&self, other: &Self, time: f64) -> Result<Self, ()> {
        use num_integer::lcm;
        let len = lcm(self.len(), other.len());
        self.iter().cycle().zip(other.iter().cycle()).take(len).map(|(me, you)| {
            me.interpolate(you, time)
        }).collect()
    }
}
/// https://drafts.csswg.org/css-transitions/#animtype-number
impl Interpolate for Au {
    #[inline]
    fn interpolate(&self, other: &Self, time: f64) -> Result<Self, ()> {
        Ok(Au((self.0 as f64 + (other.0 as f64 - self.0 as f64) * time).round() as i32))
    }
}

impl <T> Interpolate for Option<T> where T: Interpolate {
    #[inline]
    fn interpolate(&self, other: &Option<T>, time: f64) -> Result<Option<T>, ()> {
        match (self, other) {
            (&Some(ref this), &Some(ref other)) => {
                Ok(this.interpolate(other, time).ok())
            }
            _ => Err(()),
        }
    }
}

/// https://drafts.csswg.org/css-transitions/#animtype-number
impl Interpolate for f32 {
    #[inline]
    fn interpolate(&self, other: &f32, time: f64) -> Result<Self, ()> {
        Ok(((*self as f64) + ((*other as f64) - (*self as f64)) * time) as f32)
    }
}

/// https://drafts.csswg.org/css-transitions/#animtype-number
impl Interpolate for f64 {
    #[inline]
    fn interpolate(&self, other: &f64, time: f64) -> Result<Self, ()> {
        Ok(*self + (*other - *self) * time)
    }
}

/// https://drafts.csswg.org/css-transitions/#animtype-number
impl Interpolate for i32 {
    #[inline]
    fn interpolate(&self, other: &i32, time: f64) -> Result<Self, ()> {
        let a = *self as f64;
        let b = *other as f64;
        Ok((a + (b - a) * time).round() as i32)
    }
}

/// https://drafts.csswg.org/css-transitions/#animtype-number
impl Interpolate for Angle {
    #[inline]
    fn interpolate(&self, other: &Angle, time: f64) -> Result<Self, ()> {
        self.radians().interpolate(&other.radians(), time).map(Angle)
    }
}

/// https://drafts.csswg.org/css-transitions/#animtype-visibility
impl Interpolate for Visibility {
    #[inline]
    fn interpolate(&self, other: &Self, time: f64) -> Result<Self, ()> {
        match (*self, *other) {
            (Visibility::visible, _) | (_, Visibility::visible) => {
                Ok(if time >= 0.0 && time <= 1.0 {
                    Visibility::visible
                } else if time < 0.0 {
                    *self
                } else {
                    *other
                })
            }
            _ => Err(()),
        }
    }
}

/// https://drafts.csswg.org/css-transitions/#animtype-integer
impl Interpolate for ZIndex {
    #[inline]
    fn interpolate(&self, other: &Self, time: f64) -> Result<Self, ()> {
        match (*self, *other) {
            (ZIndex::Number(ref this),
             ZIndex::Number(ref other)) => {
                this.interpolate(other, time).map(ZIndex::Number)
            }
            _ => Err(()),
        }
    }
}

impl<T: Interpolate + Copy> Interpolate for Size2D<T> {
    #[inline]
    fn interpolate(&self, other: &Self, time: f64) -> Result<Self, ()> {
        let width = try!(self.width.interpolate(&other.width, time));
        let height = try!(self.height.interpolate(&other.height, time));

        Ok(Size2D::new(width, height))
    }
}

impl<T: Interpolate + Copy> Interpolate for Point2D<T> {
    #[inline]
    fn interpolate(&self, other: &Self, time: f64) -> Result<Self, ()> {
        let x = try!(self.x.interpolate(&other.x, time));
        let y = try!(self.y.interpolate(&other.y, time));

        Ok(Point2D::new(x, y))
    }
}

impl Interpolate for BorderRadiusSize {
    #[inline]
    fn interpolate(&self, other: &Self, time: f64) -> Result<Self, ()> {
        self.0.interpolate(&other.0, time).map(BorderRadiusSize)
    }
}

/// https://drafts.csswg.org/css-transitions/#animtype-length
impl Interpolate for VerticalAlign {
    #[inline]
    fn interpolate(&self, other: &Self, time: f64) -> Result<Self, ()> {
        match (*self, *other) {
            (VerticalAlign::LengthOrPercentage(LengthOrPercentage::Length(ref this)),
             VerticalAlign::LengthOrPercentage(LengthOrPercentage::Length(ref other))) => {
                this.interpolate(other, time).map(|value| {
                    VerticalAlign::LengthOrPercentage(LengthOrPercentage::Length(value))
                })
            }
            _ => Err(()),
        }
    }
}
impl Interpolate for BackgroundSize {
    #[inline]
    fn interpolate(&self, other: &Self, time: f64) -> Result<Self, ()> {
        self.0.interpolate(&other.0, time).map(BackgroundSize)
    }
}


/// https://drafts.csswg.org/css-transitions/#animtype-color
impl Interpolate for RGBA {
    #[inline]
    fn interpolate(&self, other: &RGBA, time: f64) -> Result<Self, ()> {
        Ok(RGBA {
            red: try!(self.red.interpolate(&other.red, time)),
            green: try!(self.green.interpolate(&other.green, time)),
            blue: try!(self.blue.interpolate(&other.blue, time)),
            alpha: try!(self.alpha.interpolate(&other.alpha, time)),
        })
    }
}

/// https://drafts.csswg.org/css-transitions/#animtype-color
impl Interpolate for CSSParserColor {
    #[inline]
    fn interpolate(&self, other: &Self, time: f64) -> Result<Self, ()> {
        match (*self, *other) {
            (CSSParserColor::RGBA(ref this), CSSParserColor::RGBA(ref other)) => {
                this.interpolate(other, time).map(CSSParserColor::RGBA)
            }
            _ => Err(()),
        }
    }
}

/// https://drafts.csswg.org/css-transitions/#animtype-lpcalc
impl Interpolate for CalcLengthOrPercentage {
    #[inline]
    fn interpolate(&self, other: &Self, time: f64) -> Result<Self, ()> {
        fn interpolate_half<T>(this: Option<T>,
                               other: Option<T>,
                               time: f64)
                               -> Result<Option<T>, ()>
            where T: Default + Interpolate
        {
            match (this, other) {
                (None, None) => Ok(None),
                (this, other) => {
                    let this = this.unwrap_or(T::default());
                    let other = other.unwrap_or(T::default());
                    this.interpolate(&other, time).map(Some)
                }
            }
        }

        Ok(CalcLengthOrPercentage {
            length: try!(interpolate_half(self.length, other.length, time)),
            percentage: try!(interpolate_half(self.percentage, other.percentage, time)),
        })
    }
}

/// https://drafts.csswg.org/css-transitions/#animtype-lpcalc
impl Interpolate for LengthOrPercentage {
    #[inline]
    fn interpolate(&self, other: &Self, time: f64) -> Result<Self, ()> {
        match (*self, *other) {
            (LengthOrPercentage::Length(ref this),
             LengthOrPercentage::Length(ref other)) => {
                this.interpolate(other, time).map(LengthOrPercentage::Length)
            }
            (LengthOrPercentage::Percentage(ref this),
             LengthOrPercentage::Percentage(ref other)) => {
                this.interpolate(other, time).map(LengthOrPercentage::Percentage)
            }
            (this, other) => {
                let this: CalcLengthOrPercentage = From::from(this);
                let other: CalcLengthOrPercentage = From::from(other);
                this.interpolate(&other, time)
                    .map(LengthOrPercentage::Calc)
            }
        }
    }
}

/// https://drafts.csswg.org/css-transitions/#animtype-lpcalc
impl Interpolate for LengthOrPercentageOrAuto {
    #[inline]
    fn interpolate(&self, other: &Self, time: f64) -> Result<Self, ()> {
        match (*self, *other) {
            (LengthOrPercentageOrAuto::Length(ref this),
             LengthOrPercentageOrAuto::Length(ref other)) => {
                this.interpolate(other, time).map(LengthOrPercentageOrAuto::Length)
            }
            (LengthOrPercentageOrAuto::Percentage(ref this),
             LengthOrPercentageOrAuto::Percentage(ref other)) => {
                this.interpolate(other, time).map(LengthOrPercentageOrAuto::Percentage)
            }
            (LengthOrPercentageOrAuto::Auto, LengthOrPercentageOrAuto::Auto) => {
                Ok(LengthOrPercentageOrAuto::Auto)
            }
            (this, other) => {
                let this: Option<CalcLengthOrPercentage> = From::from(this);
                let other: Option<CalcLengthOrPercentage> = From::from(other);
                match this.interpolate(&other, time) {
                    Ok(Some(result)) => Ok(LengthOrPercentageOrAuto::Calc(result)),
                    _ => Err(()),
                }
            }
        }
    }
}

/// https://drafts.csswg.org/css-transitions/#animtype-lpcalc
impl Interpolate for LengthOrPercentageOrNone {
    #[inline]
    fn interpolate(&self, other: &Self, time: f64) -> Result<Self, ()> {
        match (*self, *other) {
            (LengthOrPercentageOrNone::Length(ref this),
             LengthOrPercentageOrNone::Length(ref other)) => {
                this.interpolate(other, time).map(LengthOrPercentageOrNone::Length)
            }
            (LengthOrPercentageOrNone::Percentage(ref this),
             LengthOrPercentageOrNone::Percentage(ref other)) => {
                this.interpolate(other, time).map(LengthOrPercentageOrNone::Percentage)
            }
            (LengthOrPercentageOrNone::None, LengthOrPercentageOrNone::None) => {
                Ok(LengthOrPercentageOrNone::None)
            }
            _ => Err(())
        }
    }
}

/// https://drafts.csswg.org/css-transitions/#animtype-number
/// https://drafts.csswg.org/css-transitions/#animtype-length
impl Interpolate for LineHeight {
    #[inline]
    fn interpolate(&self, other: &Self, time: f64) -> Result<Self, ()> {
        match (*self, *other) {
            (LineHeight::Length(ref this),
             LineHeight::Length(ref other)) => {
                this.interpolate(other, time).map(LineHeight::Length)
            }
            (LineHeight::Number(ref this),
             LineHeight::Number(ref other)) => {
                this.interpolate(other, time).map(LineHeight::Number)
            }
            (LineHeight::Normal, LineHeight::Normal) => {
                Ok(LineHeight::Normal)
            }
            _ => Err(()),
        }
    }
}

/// http://dev.w3.org/csswg/css-transitions/#animtype-font-weight
impl Interpolate for FontWeight {
    #[inline]
    fn interpolate(&self, other: &Self, time: f64) -> Result<Self, ()> {
        let a = (*self as u32) as f64;
        let b = (*other as u32) as f64;
        let weight = a + (b - a) * time;
        Ok(if weight < 150. {
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

/// https://drafts.csswg.org/css-transitions/#animtype-simple-list
impl Interpolate for Position {
    #[inline]
    fn interpolate(&self, other: &Self, time: f64) -> Result<Self, ()> {
        Ok(Position {
            horizontal: try!(self.horizontal.interpolate(&other.horizontal, time)),
            vertical: try!(self.vertical.interpolate(&other.vertical, time)),
        })
    }
}

impl RepeatableListInterpolate for Position {}

impl Interpolate for BackgroundPosition {
    #[inline]
    fn interpolate(&self, other: &Self, time: f64) -> Result<Self, ()> {
        Ok(BackgroundPosition(try!(self.0.interpolate(&other.0, time))))
    }
}

/// https://drafts.csswg.org/css-transitions/#animtype-shadow-list
impl Interpolate for TextShadow {
    #[inline]
    fn interpolate(&self, other: &Self, time: f64) -> Result<Self, ()> {
        Ok(TextShadow {
            offset_x: try!(self.offset_x.interpolate(&other.offset_x, time)),
            offset_y: try!(self.offset_y.interpolate(&other.offset_y, time)),
            blur_radius: try!(self.blur_radius.interpolate(&other.blur_radius, time)),
            color: try!(self.color.interpolate(&other.color, time)),
        })
    }
}

/// https://drafts.csswg.org/css-transitions/#animtype-shadow-list
impl Interpolate for TextShadowList {
    #[inline]
    fn interpolate(&self, other: &Self, time: f64) -> Result<Self, ()> {
        let zero = TextShadow {
            offset_x: Au(0),
            offset_y: Au(0),
            blur_radius: Au(0),
            color: CSSParserColor::RGBA(RGBA {
                red: 0.0, green: 0.0, blue: 0.0, alpha: 0.0
            })
        };

        let max_len = cmp::max(self.0.len(), other.0.len());
        let mut result = Vec::with_capacity(max_len);

        for i in 0..max_len {
            let shadow = match (self.0.get(i), other.0.get(i)) {
                (Some(shadow), Some(other))
                    => try!(shadow.interpolate(other, time)),
                (Some(shadow), None) => {
                    shadow.interpolate(&zero, time).unwrap()
                }
                (None, Some(shadow)) => {
                    zero.interpolate(&shadow, time).unwrap()
                }
                (None, None) => unreachable!(),
            };
            result.push(shadow);
        }

        Ok(TextShadowList(result))
    }
}


impl Interpolate for BoxShadowList {
    #[inline]
    fn interpolate(&self, other: &Self, time: f64) -> Result<Self, ()> {
        // The inset value must change
        let mut zero = BoxShadow {
            offset_x: Au(0),
            offset_y: Au(0),
            spread_radius: Au(0),
            blur_radius: Au(0),
            color: CSSParserColor::RGBA(RGBA {
                red: 0.0, green: 0.0, blue: 0.0, alpha: 0.0
            }),
            inset: false,
        };

        let max_len = cmp::max(self.0.len(), other.0.len());
        let mut result = Vec::with_capacity(max_len);

        for i in 0..max_len {
            let shadow = match (self.0.get(i), other.0.get(i)) {
                (Some(shadow), Some(other))
                    => try!(shadow.interpolate(other, time)),
                (Some(shadow), None) => {
                    zero.inset = shadow.inset;
                    shadow.interpolate(&zero, time).unwrap()
                }
                (None, Some(shadow)) => {
                    zero.inset = shadow.inset;
                    zero.interpolate(&shadow, time).unwrap()
                }
                (None, None) => unreachable!(),
            };
            result.push(shadow);
        }

        Ok(BoxShadowList(result))
    }
}

/// https://drafts.csswg.org/css-transitions/#animtype-shadow-list
impl Interpolate for BoxShadow {
    #[inline]
    fn interpolate(&self, other: &Self, time: f64) -> Result<Self, ()> {
        if self.inset != other.inset {
            return Err(());
        }

        let x = try!(self.offset_x.interpolate(&other.offset_x, time));
        let y = try!(self.offset_y.interpolate(&other.offset_y, time));
        let color = try!(self.color.interpolate(&other.color, time));
        let spread = try!(self.spread_radius.interpolate(&other.spread_radius, time));
        let blur = try!(self.blur_radius.interpolate(&other.blur_radius, time));

        Ok(BoxShadow {
            offset_x: x,
            offset_y: y,
            blur_radius: blur,
            spread_radius: spread,
            color: color,
            inset: self.inset,
        })
    }
}

impl Interpolate for LengthOrNone {
    fn interpolate(&self, other: &Self, time: f64) -> Result<Self, ()> {
        match (*self, *other) {
            (LengthOrNone::Length(ref len), LengthOrNone::Length(ref other)) =>
                len.interpolate(&other, time).map(LengthOrNone::Length),
            _ => Err(()),
        }
    }
}

% if product == "servo":
    use properties::longhands::transform::computed_value::ComputedMatrix;
    use properties::longhands::transform::computed_value::ComputedOperation as TransformOperation;
    use properties::longhands::transform::computed_value::T as TransformList;

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
                        let decomposed_from = MatrixDecomposed2D::from(from);
                        let decomposed_to = MatrixDecomposed2D::from(from);
                        let interpolated = decomposed_from.interpolate(&decomposed_to, time).unwrap();
                        let recomposed_matrix = ComputedMatrix::from(interpolated);
                        result.push(TransformOperation::Matrix(recomposed_matrix));
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
                     &TransformOperation::Rotate(tx, ty, tz, ta)) => {
                        let norm_f = ((fx * fx) + (fy * fy) + (fz * fz)).sqrt();
                        let norm_t = ((tx * tx) + (ty * ty) + (tz * tz)).sqrt();
                        let (fx, fy, fz) = (fx / norm_f, fy / norm_f, fz / norm_f);
                        let (tx, ty, tz) = (tx / norm_t, ty / norm_t, tz / norm_t);
                        if fx == tx && fy == ty && fz == tz {
                            let ia = fa.interpolate(&ta, time).unwrap();
                            result.push(TransformOperation::Rotate(fx, fy, fz, ia));
                        } else {
                            let matrix_f = rotate_to_matrix(fx, fy, fz, fa);
                            let matrix_t = rotate_to_matrix(tx, ty, tz, ta);

                            let decomposed_f = MatrixDecomposed2D::from(matrix_f);
                            let decomposed_t = MatrixDecomposed2D::from(matrix_t);
                            let interpolated = decomposed_f.interpolate(&decomposed_t, time).unwrap();
                            let recomposed_matrix = ComputedMatrix::from(interpolated);

                            result.push(TransformOperation::Matrix(recomposed_matrix));
                        }
                    }
                    (&TransformOperation::Perspective(fd),
                     &TransformOperation::Perspective(_td)) => {
                        let mut fd_matrix = ComputedMatrix::identity();
                        let mut td_matrix = ComputedMatrix::identity();
                        fd_matrix.m43 = -1. / fd.0 as f32;
                        td_matrix.m43 = -1. / _td.0 as f32;
                        let decomposed_fd = MatrixDecomposed2D::from(fd_matrix);
                        let decomposed_td = MatrixDecomposed2D::from(td_matrix);
                        let interpolated = decomposed_fd.interpolate(&decomposed_td, time).unwrap();
                        let recomposed_matrix = ComputedMatrix::from(interpolated);
                        result.push(TransformOperation::Matrix(recomposed_matrix));
                    }
                    _ => {
                        // This should be unreachable due to the can_interpolate_list() call.
                        unreachable!();
                    }
                }
            }
        } else {
            // TODO(gw): Implement matrix decomposition and interpolation
            // TODO(canaltinova): Do I need to interpolate each item of the list?
            result.extend_from_slice(from_list);
        }

        TransformList(Some(result))
    }

    fn rotate_to_matrix(x: f32, y: f32, z: f32, a: SpecifiedAngle) -> ComputedMatrix {
        let rad = a.radians();
        let sc = (rad / 2.0).sin() * (rad / 2.0).cos();
        let sq = 1.0 / 2.0 * (1.0 - (rad).cos());

        ComputedMatrix {
            m11: 1.0 - 2.0 * (y * y + z * z) * sq, m12: 2.0 * (x * y * sq - z * sc),
            m13: 2.0 * (x * z * sq + y * sc), m14: 0.0, m21: 2.0 * (x * y * sq + z * sc),
            m22: 1.0 - 2.0 * (x * x + z * z) * sq, m23: 2.0 * (y * z * sq - x * sc), m24: 0.0,
            m31: 2.0 * (x * z * sq - y * sc), m32: 2.0 * (y * z * sq + x * sc),
            m33: 1.0 - 2.0 * (x * x + y * y) * sq, m34: 0.0, m41: 0.0, m42: 0.0, m43: 0.0, m44: 1.0
        }
    }

    #[derive(Clone, Copy, Debug)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub struct DecomposedMatrix {
        pub m11: CSSFloat, pub m12: CSSFloat,
        pub m21: CSSFloat, pub m22: CSSFloat,
    }

    #[derive(Clone, Copy, Debug)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub struct Translate2D(f32, f32);

    #[derive(Clone, Copy, Debug)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub struct Scale2D(f32, f32);

    #[derive(Clone, Copy, Debug)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub struct MatrixDecomposed2D {
        pub translate: Translate2D,
        pub scale: Scale2D,
        pub angle: f32,
        pub matrix: DecomposedMatrix,
    }

    impl Interpolate for DecomposedMatrix {
        fn interpolate(&self, other: &Self, time: f64) -> Result<Self, ()> {
            Ok(DecomposedMatrix {
                m11: try!(self.m11.interpolate(&other.m11, time)),
                m12: try!(self.m12.interpolate(&other.m12, time)),
                m21: try!(self.m21.interpolate(&other.m21, time)),
                m22: try!(self.m22.interpolate(&other.m22, time)),
            })
        }
    }

    impl Interpolate for Translate2D {
        fn interpolate(&self, other: &Self, time: f64) -> Result<Self, ()> {
            Ok(Translate2D(
                try!(self.0.interpolate(&other.0, time)),
                try!(self.1.interpolate(&other.1, time))
            ))
        }
    }

    impl Interpolate for Scale2D {
        fn interpolate(&self, other: &Self, time: f64) -> Result<Self, ()> {
            Ok(Scale2D(
                try!(self.0.interpolate(&other.0, time)),
                try!(self.1.interpolate(&other.1, time))
            ))
        }
    }

    impl Interpolate for MatrixDecomposed2D {
        /// https://drafts.csswg.org/css-transforms/#interpolation-of-decomposed-2d-matrix-values
        fn interpolate(&self, other: &Self, time: f64) -> Result<Self, ()> {
            // If x-axis of one is flipped, and y-axis of the other,
            // convert to an unflipped rotation.
            let mut scale = self.scale;
            let mut angle = self.angle;
            let mut other_angle = other.angle;
            if (scale.0 < 0.0 && other.scale.1 < 0.0) || (scale.1 < 0.0 && other.scale.0 < 0.0) {
                scale.0 = -scale.0;
                scale.1 = -scale.1;
                angle += if angle < 0.0 {180.} else {-180.};
            }

            // Don't rotate the long way around.
            if angle == 0.0 {
                angle = 360.
            }
            if other_angle == 0.0 {
                other_angle = 360.
            }

            if (angle - other_angle).abs() > 180. {
                if angle > other_angle {
                    angle -= 360.
                }
                else{
                    other_angle -= 360.
                }
            }

            // Interpolate all values.
            let translate = try!(self.translate.interpolate(&other.translate, time));
            let scale = try!(scale.interpolate(&other.scale, time));
            let angle = try!(angle.interpolate(&other_angle, time));
            let matrix = try!(self.matrix.interpolate(&other.matrix, time));

            Ok(MatrixDecomposed2D {
                translate: translate,
                scale: scale,
                angle: angle,
                matrix: matrix,
            })
        }
    }

    impl From<ComputedMatrix> for MatrixDecomposed2D {
        /// Decompose a matrix.
        /// https://drafts.csswg.org/css-transforms/#decomposing-a-2d-matrix
        fn from(matrix: ComputedMatrix) -> MatrixDecomposed2D {
            let mut row0x = matrix.m11;
            let mut row0y = matrix.m12;
            let mut row1x = matrix.m21;
            let mut row1y = matrix.m22;

            let translate = Translate2D(matrix.m41, matrix.m42);
            let mut scale = Scale2D((row0x * row0x + row0y * row0y).sqrt(),
                                    (row1x * row1x + row1y * row1y).sqrt());

            // If determinant is negative, one axis was flipped.
            let determinant = row0x * row1y - row0y * row1x;
            if determinant < 0. {
                if row0x < row1y {
                    scale.0 = -scale.0;
                } else {
                    scale.1 = -scale.1;
                }
            }

            // Renormalize matrix to remove scale.
            if scale.0 != 0.0 {
                row0x *= 1. / scale.0;
                row0y *= 1. / scale.0;
            }
            if scale.1 != 0.0 {
                row1x *= 1. / scale.1;
                row1y *= 1. / scale.1;
            }

            // Compute rotation and renormalize matrix.
            let mut angle = row0y.atan2(row0x);
            if angle != 0.0 {
                let sn = -row0y;
                let cs = row0x;
                let m11 = row0x;
                let m12 = row0y;
                let m21 = row1x;
                let m22 = row1y;
                row0x = cs * m11 + sn * m21;
                row0y = cs * m12 + sn * m22;
                row1x = -sn * m11 + cs * m21;
                row1y = -sn * m12 + cs * m22;
            }

            let m = DecomposedMatrix {
                m11: row0x, m12: row0y,
                m21: row1x, m22: row1y,
            };

            // Convert into degrees because our rotation functions expect it.
            angle = angle.to_degrees();
            MatrixDecomposed2D {
                translate: translate,
                scale: scale,
                angle: angle,
                matrix: m,
            }
        }
    }

    impl From<MatrixDecomposed2D> for ComputedMatrix {
        /// Recompose a matrix.
        /// https://drafts.csswg.org/css-transforms/#recomposing-to-a-2d-matrix
        fn from(decomposed: MatrixDecomposed2D) -> ComputedMatrix {
            let mut computed_matrix = ComputedMatrix::identity();
            computed_matrix.m11 = decomposed.matrix.m11;
            computed_matrix.m12 = decomposed.matrix.m12;
            computed_matrix.m21 = decomposed.matrix.m21;
            computed_matrix.m22 = decomposed.matrix.m22;

            // Translate matrix.
            computed_matrix.m41 = decomposed.translate.0 * decomposed.matrix.m11 +
                                  decomposed.translate.1 * decomposed.matrix.m21;
            computed_matrix.m42 = decomposed.translate.0 * decomposed.matrix.m11 +
                                  decomposed.translate.1 * decomposed.matrix.m21;

            // Rotate matrix.
            let angle = decomposed.angle.to_radians();
            let cos_angle = angle.cos();
            let sin_angle = angle.sin();

            let mut rotate_matrix = ComputedMatrix::identity();
            rotate_matrix.m11 = cos_angle;
            rotate_matrix.m12 = sin_angle;
            rotate_matrix.m21 = -sin_angle;
            rotate_matrix.m22 = cos_angle;

            let matrix_clone = computed_matrix;
            // Multiplication of computed_matrix and rotate_matrix
            % for i in range(1, 5):
                % for j in range(1, 5):
                    computed_matrix.m${i}${j} = (matrix_clone.m${i}1 * rotate_matrix.m1${j}) +
                                                (matrix_clone.m${i}2 * rotate_matrix.m2${j}) +
                                                (matrix_clone.m${i}3 * rotate_matrix.m3${j}) +
                                                (matrix_clone.m${i}4 * rotate_matrix.m4${j});
                % endfor
            % endfor

            // Scale matrix.
            computed_matrix.m11 *= decomposed.scale.0;
            computed_matrix.m12 *= decomposed.scale.0;
            computed_matrix.m21 *= decomposed.scale.1;
            computed_matrix.m22 *= decomposed.scale.1;
            computed_matrix
        }
    }

    /// https://drafts.csswg.org/css-transforms/#interpolation-of-transforms
    impl Interpolate for TransformList {
        #[inline]
        fn interpolate(&self, other: &TransformList, time: f64) -> Result<Self, ()> {
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

            Ok(result)
        }
    }
% endif


