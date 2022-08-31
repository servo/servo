/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Animated values.
//!
//! Some values, notably colors, cannot be interpolated directly with their
//! computed values and need yet another intermediate representation. This
//! module's raison d'être is to ultimately contain all these types.

use crate::properties::PropertyId;
use crate::values::computed::length::LengthPercentage;
use crate::values::computed::url::ComputedUrl;
use crate::values::computed::Angle as ComputedAngle;
use crate::values::computed::Image;
use crate::values::specified::SVGPathData;
use crate::values::CSSFloat;
use app_units::Au;
use smallvec::SmallVec;
use std::cmp;

pub mod color;
pub mod effects;
mod font;
mod grid;
mod svg;
pub mod transform;

/// The category a property falls into for ordering purposes.
///
/// https://drafts.csswg.org/web-animations/#calculating-computed-keyframes
#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
enum PropertyCategory {
    Custom,
    PhysicalLonghand,
    LogicalLonghand,
    Shorthand,
}

impl PropertyCategory {
    fn of(id: &PropertyId) -> Self {
        match *id {
            PropertyId::Shorthand(..) | PropertyId::ShorthandAlias(..) => {
                PropertyCategory::Shorthand
            },
            PropertyId::Longhand(id) | PropertyId::LonghandAlias(id, ..) => {
                if id.is_logical() {
                    PropertyCategory::LogicalLonghand
                } else {
                    PropertyCategory::PhysicalLonghand
                }
            },
            PropertyId::Custom(..) => PropertyCategory::Custom,
        }
    }
}

/// A comparator to sort PropertyIds such that physical longhands are sorted
/// before logical longhands and shorthands, shorthands with fewer components
/// are sorted before shorthands with more components, and otherwise shorthands
/// are sorted by IDL name as defined by [Web Animations][property-order].
///
/// Using this allows us to prioritize values specified by longhands (or smaller
/// shorthand subsets) when longhands and shorthands are both specified on the
/// one keyframe.
///
/// [property-order] https://drafts.csswg.org/web-animations/#calculating-computed-keyframes
pub fn compare_property_priority(a: &PropertyId, b: &PropertyId) -> cmp::Ordering {
    let a_category = PropertyCategory::of(a);
    let b_category = PropertyCategory::of(b);

    if a_category != b_category {
        return a_category.cmp(&b_category);
    }

    if a_category != PropertyCategory::Shorthand {
        return cmp::Ordering::Equal;
    }

    let a = a.as_shorthand().unwrap();
    let b = b.as_shorthand().unwrap();
    // Within shorthands, sort by the number of subproperties, then by IDL
    // name.
    let subprop_count_a = a.longhands().count();
    let subprop_count_b = b.longhands().count();
    subprop_count_a
        .cmp(&subprop_count_b)
        .then_with(|| a.idl_name_sort_order().cmp(&b.idl_name_sort_order()))
}

/// A helper function to animate two multiplicative factor.
pub fn animate_multiplicative_factor(
    this: CSSFloat,
    other: CSSFloat,
    procedure: Procedure,
) -> Result<CSSFloat, ()> {
    Ok((this - 1.).animate(&(other - 1.), procedure)? + 1.)
}

/// Animate from one value to another.
///
/// This trait is derivable with `#[derive(Animate)]`. The derived
/// implementation uses a `match` expression with identical patterns for both
/// `self` and `other`, calling `Animate::animate` on each fields of the values.
/// If a field is annotated with `#[animation(constant)]`, the two values should
/// be equal or an error is returned.
///
/// If a variant is annotated with `#[animation(error)]`, the corresponding
/// `match` arm returns an error.
///
/// Trait bounds for type parameter `Foo` can be opted out of with
/// `#[animation(no_bound(Foo))]` on the type definition, trait bounds for
/// fields can be opted into with `#[animation(field_bound)]` on the field.
pub trait Animate: Sized {
    /// Animate a value towards another one, given an animation procedure.
    fn animate(&self, other: &Self, procedure: Procedure) -> Result<Self, ()>;
}

/// An animation procedure.
///
/// <https://drafts.csswg.org/web-animations/#procedures-for-animating-properties>
#[allow(missing_docs)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Procedure {
    /// <https://drafts.csswg.org/web-animations/#animation-interpolation>
    Interpolate { progress: f64 },
    /// <https://drafts.csswg.org/web-animations/#animation-addition>
    Add,
    /// <https://drafts.csswg.org/web-animations/#animation-accumulation>
    Accumulate { count: u64 },
}

/// Conversion between computed values and intermediate values for animations.
///
/// Notably, colors are represented as four floats during animations.
///
/// This trait is derivable with `#[derive(ToAnimatedValue)]`.
pub trait ToAnimatedValue {
    /// The type of the animated value.
    type AnimatedValue;

    /// Converts this value to an animated value.
    fn to_animated_value(self) -> Self::AnimatedValue;

    /// Converts back an animated value into a computed value.
    fn from_animated_value(animated: Self::AnimatedValue) -> Self;
}

/// Returns a value similar to `self` that represents zero.
///
/// This trait is derivable with `#[derive(ToAnimatedValue)]`. If a field is
/// annotated with `#[animation(constant)]`, a clone of its value will be used
/// instead of calling `ToAnimatedZero::to_animated_zero` on it.
///
/// If a variant is annotated with `#[animation(error)]`, the corresponding
/// `match` arm is not generated.
///
/// Trait bounds for type parameter `Foo` can be opted out of with
/// `#[animation(no_bound(Foo))]` on the type definition.
pub trait ToAnimatedZero: Sized {
    /// Returns a value that, when added with an underlying value, will produce the underlying
    /// value. This is used for SMIL animation's "by-animation" where SMIL first interpolates from
    /// the zero value to the 'by' value, and then adds the result to the underlying value.
    ///
    /// This is not the necessarily the same as the initial value of a property. For example, the
    /// initial value of 'stroke-width' is 1, but the zero value is 0, since adding 1 to the
    /// underlying value will not produce the underlying value.
    fn to_animated_zero(&self) -> Result<Self, ()>;
}

impl Procedure {
    /// Returns this procedure as a pair of weights.
    ///
    /// This is useful for animations that don't animate differently
    /// depending on the used procedure.
    #[inline]
    pub fn weights(self) -> (f64, f64) {
        match self {
            Procedure::Interpolate { progress } => (1. - progress, progress),
            Procedure::Add => (1., 1.),
            Procedure::Accumulate { count } => (count as f64, 1.),
        }
    }
}

/// <https://drafts.csswg.org/css-transitions/#animtype-number>
impl Animate for i32 {
    #[inline]
    fn animate(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        Ok(((*self as f64).animate(&(*other as f64), procedure)? + 0.5).floor() as i32)
    }
}

/// <https://drafts.csswg.org/css-transitions/#animtype-number>
impl Animate for f32 {
    #[inline]
    fn animate(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        use std::f32;

        let ret = (*self as f64).animate(&(*other as f64), procedure)?;
        Ok(ret.min(f32::MAX as f64).max(f32::MIN as f64) as f32)
    }
}

/// <https://drafts.csswg.org/css-transitions/#animtype-number>
impl Animate for f64 {
    #[inline]
    fn animate(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        use std::f64;

        let (self_weight, other_weight) = procedure.weights();

        let ret = *self * self_weight + *other * other_weight;
        Ok(ret.min(f64::MAX).max(f64::MIN))
    }
}

impl<T> Animate for Option<T>
where
    T: Animate,
{
    #[inline]
    fn animate(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        match (self.as_ref(), other.as_ref()) {
            (Some(ref this), Some(ref other)) => Ok(Some(this.animate(other, procedure)?)),
            (None, None) => Ok(None),
            _ => Err(()),
        }
    }
}

impl Animate for Au {
    #[inline]
    fn animate(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        Ok(Au::new(self.0.animate(&other.0, procedure)?))
    }
}

impl<T: Animate> Animate for Box<T> {
    #[inline]
    fn animate(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        Ok(Box::new((**self).animate(&other, procedure)?))
    }
}

impl<T> ToAnimatedValue for Option<T>
where
    T: ToAnimatedValue,
{
    type AnimatedValue = Option<<T as ToAnimatedValue>::AnimatedValue>;

    #[inline]
    fn to_animated_value(self) -> Self::AnimatedValue {
        self.map(T::to_animated_value)
    }

    #[inline]
    fn from_animated_value(animated: Self::AnimatedValue) -> Self {
        animated.map(T::from_animated_value)
    }
}

impl<T> ToAnimatedValue for Vec<T>
where
    T: ToAnimatedValue,
{
    type AnimatedValue = Vec<<T as ToAnimatedValue>::AnimatedValue>;

    #[inline]
    fn to_animated_value(self) -> Self::AnimatedValue {
        self.into_iter().map(T::to_animated_value).collect()
    }

    #[inline]
    fn from_animated_value(animated: Self::AnimatedValue) -> Self {
        animated.into_iter().map(T::from_animated_value).collect()
    }
}

impl<T> ToAnimatedValue for Box<T>
where
    T: ToAnimatedValue,
{
    type AnimatedValue = Box<<T as ToAnimatedValue>::AnimatedValue>;

    #[inline]
    fn to_animated_value(self) -> Self::AnimatedValue {
        Box::new((*self).to_animated_value())
    }

    #[inline]
    fn from_animated_value(animated: Self::AnimatedValue) -> Self {
        Box::new(T::from_animated_value(*animated))
    }
}

impl<T> ToAnimatedValue for Box<[T]>
where
    T: ToAnimatedValue,
{
    type AnimatedValue = Box<[<T as ToAnimatedValue>::AnimatedValue]>;

    #[inline]
    fn to_animated_value(self) -> Self::AnimatedValue {
        self.into_vec()
            .into_iter()
            .map(T::to_animated_value)
            .collect::<Vec<_>>()
            .into_boxed_slice()
    }

    #[inline]
    fn from_animated_value(animated: Self::AnimatedValue) -> Self {
        animated
            .into_vec()
            .into_iter()
            .map(T::from_animated_value)
            .collect::<Vec<_>>()
            .into_boxed_slice()
    }
}

impl<T> ToAnimatedValue for crate::OwnedSlice<T>
where
    T: ToAnimatedValue,
{
    type AnimatedValue = crate::OwnedSlice<<T as ToAnimatedValue>::AnimatedValue>;

    #[inline]
    fn to_animated_value(self) -> Self::AnimatedValue {
        self.into_box().to_animated_value().into()
    }

    #[inline]
    fn from_animated_value(animated: Self::AnimatedValue) -> Self {
        Self::from(Box::from_animated_value(animated.into_box()))
    }
}

impl<T> ToAnimatedValue for SmallVec<[T; 1]>
where
    T: ToAnimatedValue,
{
    type AnimatedValue = SmallVec<[T::AnimatedValue; 1]>;

    #[inline]
    fn to_animated_value(self) -> Self::AnimatedValue {
        self.into_iter().map(T::to_animated_value).collect()
    }

    #[inline]
    fn from_animated_value(animated: Self::AnimatedValue) -> Self {
        animated.into_iter().map(T::from_animated_value).collect()
    }
}

macro_rules! trivial_to_animated_value {
    ($ty:ty) => {
        impl $crate::values::animated::ToAnimatedValue for $ty {
            type AnimatedValue = Self;

            #[inline]
            fn to_animated_value(self) -> Self {
                self
            }

            #[inline]
            fn from_animated_value(animated: Self::AnimatedValue) -> Self {
                animated
            }
        }
    };
}

trivial_to_animated_value!(Au);
trivial_to_animated_value!(LengthPercentage);
trivial_to_animated_value!(ComputedAngle);
trivial_to_animated_value!(ComputedUrl);
trivial_to_animated_value!(bool);
trivial_to_animated_value!(f32);
trivial_to_animated_value!(i32);
// Note: This implementation is for ToAnimatedValue of ShapeSource.
//
// SVGPathData uses Box<[T]>. If we want to derive ToAnimatedValue for all the
// types, we have to do "impl ToAnimatedValue for Box<[T]>" first.
// However, the general version of "impl ToAnimatedValue for Box<[T]>" needs to
// clone |T| and convert it into |T::AnimatedValue|. However, for SVGPathData
// that is unnecessary--moving |T| is sufficient. So here, we implement this
// trait manually.
trivial_to_animated_value!(SVGPathData);
// FIXME: Bug 1514342, Image is not animatable, but we still need to implement
// this to avoid adding this derive to generic::Image and all its arms. We can
// drop this after landing Bug 1514342.
trivial_to_animated_value!(Image);

impl ToAnimatedZero for Au {
    #[inline]
    fn to_animated_zero(&self) -> Result<Self, ()> {
        Ok(Au(0))
    }
}

impl ToAnimatedZero for f32 {
    #[inline]
    fn to_animated_zero(&self) -> Result<Self, ()> {
        Ok(0.)
    }
}

impl ToAnimatedZero for f64 {
    #[inline]
    fn to_animated_zero(&self) -> Result<Self, ()> {
        Ok(0.)
    }
}

impl ToAnimatedZero for i32 {
    #[inline]
    fn to_animated_zero(&self) -> Result<Self, ()> {
        Ok(0)
    }
}

impl<T> ToAnimatedZero for Box<T>
where
    T: ToAnimatedZero,
{
    #[inline]
    fn to_animated_zero(&self) -> Result<Self, ()> {
        Ok(Box::new((**self).to_animated_zero()?))
    }
}

impl<T> ToAnimatedZero for Option<T>
where
    T: ToAnimatedZero,
{
    #[inline]
    fn to_animated_zero(&self) -> Result<Self, ()> {
        match *self {
            Some(ref value) => Ok(Some(value.to_animated_zero()?)),
            None => Ok(None),
        }
    }
}

impl<T> ToAnimatedZero for Vec<T>
where
    T: ToAnimatedZero,
{
    #[inline]
    fn to_animated_zero(&self) -> Result<Self, ()> {
        self.iter().map(|v| v.to_animated_zero()).collect()
    }
}

impl<T> ToAnimatedZero for Box<[T]>
where
    T: ToAnimatedZero,
{
    #[inline]
    fn to_animated_zero(&self) -> Result<Self, ()> {
        self.iter().map(|v| v.to_animated_zero()).collect()
    }
}

impl<T> ToAnimatedZero for crate::OwnedSlice<T>
where
    T: ToAnimatedZero,
{
    #[inline]
    fn to_animated_zero(&self) -> Result<Self, ()> {
        self.iter().map(|v| v.to_animated_zero()).collect()
    }
}

impl<T> ToAnimatedZero for crate::ArcSlice<T>
where
    T: ToAnimatedZero,
{
    #[inline]
    fn to_animated_zero(&self) -> Result<Self, ()> {
        let v = self
            .iter()
            .map(|v| v.to_animated_zero())
            .collect::<Result<Vec<_>, _>>()?;
        Ok(crate::ArcSlice::from_iter(v.into_iter()))
    }
}
