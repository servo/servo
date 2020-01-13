/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! `<length-percentage>` computed values, and related ones.
//!
//! The over-all design is a tagged pointer, with the lower bits of the pointer
//! being non-zero if it is a non-calc value.
//!
//! It is expected to take 64 bits both in x86 and x86-64. This is implemented
//! as a `union`, with 4 different variants:
//!
//!  * The length and percentage variants have a { tag, f32 } (effectively)
//!    layout. The tag has to overlap with the lower 2 bits of the calc variant.
//!
//!  * The `calc()` variant is a { tag, pointer } in x86 (so same as the
//!    others), or just a { pointer } in x86-64 (so that the two bits of the tag
//!    can be obtained from the lower bits of the pointer).
//!
//!  * There's a `tag` variant just to make clear when only the tag is intended
//!    to be read. Note that the tag needs to be masked always by `TAG_MASK`, to
//!    deal with the pointer variant in x86-64.
//!
//! The assertions in the constructor methods ensure that the tag getter matches
//! our expectations.

use super::{Context, Length, Percentage, ToComputedValue};
use crate::values::animated::{ToAnimatedValue, ToAnimatedZero};
use crate::values::distance::{ComputeSquaredDistance, SquaredDistance};
use crate::values::generics::NonNegative;
use crate::values::specified::length::FontBaseSize;
use crate::values::{specified, CSSFloat};
use crate::Zero;
use app_units::Au;
use malloc_size_of::{MallocSizeOf, MallocSizeOfOps};
use serde::{Serialize, Deserialize};
use std::fmt::{self, Write};
use style_traits::values::specified::AllowedNumericType;
use style_traits::{CssWriter, ToCss};

#[doc(hidden)]
#[derive(Copy, Clone)]
#[repr(C)]
pub struct LengthVariant {
    tag: u32,
    length: Length,
}

#[doc(hidden)]
#[derive(Copy, Clone)]
#[repr(C)]
pub struct PercentageVariant {
    tag: u32,
    percentage: Percentage,
}

// NOTE(emilio): cbindgen only understands the #[cfg] on the top level
// definition.
#[doc(hidden)]
#[derive(Copy, Clone)]
#[repr(C)]
#[cfg(target_pointer_width = "32")]
pub struct CalcVariant {
    tag: u32,
    ptr: *mut CalcLengthPercentage,
}

#[doc(hidden)]
#[derive(Copy, Clone)]
#[repr(C)]
#[cfg(target_pointer_width = "64")]
pub struct CalcVariant {
    ptr: *mut CalcLengthPercentage,
}

#[doc(hidden)]
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TagVariant {
    tag: u32,
}

/// A `<length-percentage>` value. This can be either a `<length>`, a
/// `<percentage>`, or a combination of both via `calc()`.
///
/// cbindgen:private-default-tagged-enum-constructor=false
/// cbindgen:derive-mut-casts=true
///
/// https://drafts.csswg.org/css-values-4/#typedef-length-percentage
///
/// The tag is stored in the lower two bits.
///
/// We need to use a struct instead of the union directly because unions with
/// Drop implementations are unstable, looks like.
///
/// Also we need the union and the variants to be `pub` (even though the member
/// is private) so that cbindgen generates it. They're not part of the public
/// API otherwise.
#[repr(transparent)]
pub struct LengthPercentage(LengthPercentageUnion);

#[doc(hidden)]
#[repr(C)]
pub union LengthPercentageUnion {
    length: LengthVariant,
    percentage: PercentageVariant,
    calc: CalcVariant,
    tag: TagVariant,
}

impl LengthPercentageUnion {
    #[doc(hidden)] // Need to be public so that cbindgen generates it.
    pub const TAG_CALC: u32 = 0;
    #[doc(hidden)]
    pub const TAG_LENGTH: u32 = 1;
    #[doc(hidden)]
    pub const TAG_PERCENTAGE: u32 = 2;
    #[doc(hidden)]
    pub const TAG_MASK: u32 = 0b11;
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(u32)]
enum Tag {
    Calc = LengthPercentageUnion::TAG_CALC,
    Length = LengthPercentageUnion::TAG_LENGTH,
    Percentage = LengthPercentageUnion::TAG_PERCENTAGE,
}

// All the members should be 64 bits, even in 32-bit builds.
#[allow(unused)]
unsafe fn static_assert() {
    std::mem::transmute::<u64, LengthVariant>(0u64);
    std::mem::transmute::<u64, PercentageVariant>(0u64);
    std::mem::transmute::<u64, CalcVariant>(0u64);
    std::mem::transmute::<u64, LengthPercentage>(0u64);
}

impl Drop for LengthPercentage {
    fn drop(&mut self) {
        if self.tag() == Tag::Calc {
            let _ = unsafe { Box::from_raw(self.0.calc.ptr) };
        }
    }
}

impl MallocSizeOf for LengthPercentage {
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        match self.unpack() {
            Unpacked::Length(..) | Unpacked::Percentage(..) => 0,
            Unpacked::Calc(c) => unsafe { ops.malloc_size_of(c) },
        }
    }
}

/// An unpacked `<length-percentage>` that borrows the `calc()` variant.
#[derive(Clone, Debug, PartialEq)]
enum Unpacked<'a> {
    Calc(&'a CalcLengthPercentage),
    Length(Length),
    Percentage(Percentage),
}

/// An unpacked `<length-percentage>` that owns the `calc()` variant, for
/// serialization purposes.
#[derive(Deserialize, Serialize, PartialEq)]
enum Serializable {
    Calc(CalcLengthPercentage),
    Length(Length),
    Percentage(Percentage),
}

impl LengthPercentage {
    /// 1px length value for SVG defaults
    #[inline]
    pub fn one() -> Self {
        Self::new_length(Length::new(1.))
    }

    /// Constructs a length value.
    #[inline]
    pub fn new_length(length: Length) -> Self {
        let length = Self(LengthPercentageUnion {
            length: LengthVariant {
                tag: LengthPercentageUnion::TAG_LENGTH,
                length,
            }
        });
        debug_assert_eq!(length.tag(), Tag::Length);
        length
    }

    /// Constructs a percentage value.
    #[inline]
    pub fn new_percent(percentage: Percentage) -> Self {
        let percent = Self(LengthPercentageUnion {
            percentage: PercentageVariant {
                tag: LengthPercentageUnion::TAG_PERCENTAGE,
                percentage,
            }
        });
        debug_assert_eq!(percent.tag(), Tag::Percentage);
        percent
    }

    /// Constructs a `calc()` value.
    #[inline]
    pub fn new_calc(l: Length, percentage: Option<Percentage>) -> Self {
        CalcLengthPercentage::new(l, percentage).to_length_percentge()
    }

    /// Private version of new_calc() that constructs a calc() variant without
    /// checking.
    fn new_calc_unchecked(calc: Box<CalcLengthPercentage>) -> Self {
        let ptr = Box::into_raw(calc);
        let calc = Self(LengthPercentageUnion {
            calc: CalcVariant {
                #[cfg(target_pointer_width = "32")]
                tag: LengthPercentageUnion::TAG_CALC,
                ptr,
            }
        });
        debug_assert_eq!(calc.tag(), Tag::Calc);
        calc
    }

    #[inline]
    fn tag(&self) -> Tag {
        match unsafe { self.0.tag.tag & LengthPercentageUnion::TAG_MASK } {
            LengthPercentageUnion::TAG_CALC => Tag::Calc,
            LengthPercentageUnion::TAG_LENGTH => Tag::Length,
            LengthPercentageUnion::TAG_PERCENTAGE => Tag::Percentage,
            _ => unreachable!("Bogus tag?"),
        }
    }

    #[inline]
    fn unpack<'a>(&'a self) -> Unpacked<'a> {
        unsafe {
            match self.tag() {
                Tag::Calc => Unpacked::Calc(&*self.0.calc.ptr),
                Tag::Length => Unpacked::Length(self.0.length.length),
                Tag::Percentage => Unpacked::Percentage(self.0.percentage.percentage),
            }
        }
    }

    #[inline]
    fn to_serializable(&self) -> Serializable {
        match self.unpack() {
            Unpacked::Calc(c) => Serializable::Calc(c.clone()),
            Unpacked::Length(l) => Serializable::Length(l),
            Unpacked::Percentage(p) => Serializable::Percentage(p),
        }
    }

    #[inline]
    fn from_serializable(s: Serializable) -> Self {
        match s {
            Serializable::Calc(c) => Self::new_calc_unchecked(Box::new(c)),
            Serializable::Length(l) => Self::new_length(l),
            Serializable::Percentage(p) => Self::new_percent(p),
        }
    }

    /// Returns true if the computed value is absolute 0 or 0%.
    #[inline]
    pub fn is_definitely_zero(&self) -> bool {
        match self.unpack() {
            Unpacked::Length(l) => l.px() == 0.0,
            Unpacked::Percentage(p) => p.0 == 0.0,
            Unpacked::Calc(ref c) => c.is_definitely_zero(),
        }
    }

    /// Returns the `<length>` component of this `calc()`, unclamped.
    #[inline]
    pub fn unclamped_length(&self) -> Length {
        match self.unpack() {
            Unpacked::Length(l) => l,
            Unpacked::Percentage(..) => Zero::zero(),
            Unpacked::Calc(c) => c.unclamped_length(),
        }
    }

    /// Returns this `calc()` as a `<length>`.
    ///
    /// Panics in debug mode if a percentage is present in the expression.
    #[inline]
    fn length(&self) -> Length {
        debug_assert!(!self.has_percentage());
        self.length_component()
    }

    /// Returns the `<length>` component of this `calc()`, clamped.
    #[inline]
    pub fn length_component(&self) -> Length {
        match self.unpack() {
            Unpacked::Length(l) => l,
            Unpacked::Percentage(..) => Zero::zero(),
            Unpacked::Calc(c) => c.length_component(),
        }
    }

    /// Returns the `<percentage>` component of this `calc()`, unclamped, as a
    /// float.
    ///
    /// FIXME: This are very different semantics from length(), we should
    /// probably rename this.
    #[inline]
    pub fn percentage(&self) -> CSSFloat {
        match self.unpack() {
            Unpacked::Length(..) => 0.,
            Unpacked::Percentage(p) => p.0,
            Unpacked::Calc(c) => c.percentage.0,
        }
    }

    /// Returns the `<length>` component of this `calc()`, clamped.
    #[inline]
    pub fn as_percentage(&self) -> Option<Percentage> {
        match self.unpack() {
            Unpacked::Length(..) => None,
            Unpacked::Percentage(p) => Some(p),
            Unpacked::Calc(ref c) => c.as_percentage(),
        }
    }

    /// Resolves the percentage.
    #[inline]
    pub fn resolve(&self, basis: Length) -> Length {
        match self.unpack() {
            Unpacked::Length(l) => l,
            Unpacked::Percentage(p) => Length::new(basis.px() * p.0),
            Unpacked::Calc(ref c) => c.resolve(basis),
        }
    }

    /// Resolves the percentage. Just an alias of resolve().
    #[inline]
    pub fn percentage_relative_to(&self, basis: Length) -> Length {
        self.resolve(basis)
    }

    /// Return whether there's any percentage in this value.
    #[inline]
    pub fn has_percentage(&self) -> bool {
        match self.unpack() {
            Unpacked::Length(..) => false,
            Unpacked::Percentage(..) => true,
            Unpacked::Calc(ref c) => c.has_percentage,
        }
    }

    /// Return the specified percentage if any.
    #[inline]
    pub fn specified_percentage(&self) -> Option<Percentage> {
        match self.unpack() {
            Unpacked::Length(..) => None,
            Unpacked::Percentage(p) => Some(p),
            Unpacked::Calc(ref c) => c.specified_percentage(),
        }
    }

    /// Returns the used value.
    #[inline]
    pub fn to_used_value(&self, containing_length: Au) -> Au {
        Au::from(self.to_pixel_length(containing_length))
    }

    /// Returns the used value as CSSPixelLength.
    #[inline]
    pub fn to_pixel_length(&self, containing_length: Au) -> Length {
        self.resolve(containing_length.into())
    }

    /// Convert the computed value into used value.
    #[inline]
    fn maybe_to_used_value(&self, container_len: Option<Length>) -> Option<Au> {
        self.maybe_percentage_relative_to(container_len).map(Au::from)
    }

    /// If there are special rules for computing percentages in a value (e.g.
    /// the height property), they apply whenever a calc() expression contains
    /// percentages.
    pub fn maybe_percentage_relative_to(&self, container_len: Option<Length>) -> Option<Length> {
        if self.has_percentage() {
            return Some(self.resolve(container_len?));
        }
        Some(self.length())
    }

    /// Returns the clamped non-negative values.
    #[inline]
    pub fn clamp_to_non_negative(&self) -> Self {
        match self.unpack() {
            Unpacked::Length(l) => Self::new_length(l.clamp_to_non_negative()),
            Unpacked::Percentage(p) => Self::new_percent(p.clamp_to_non_negative()),
            Unpacked::Calc(c) => c.clamp_to_non_negative().to_length_percentge(),
        }
    }
}

impl PartialEq for LengthPercentage {
    fn eq(&self, other: &Self) -> bool {
        self.unpack() == other.unpack()
    }
}

impl fmt::Debug for LengthPercentage {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        self.unpack().fmt(formatter)
    }
}

impl ToAnimatedZero for LengthPercentage {
    fn to_animated_zero(&self) -> Result<Self, ()> {
        Ok(match self.unpack() {
            Unpacked::Length(l) => Self::new_length(l.to_animated_zero()?),
            Unpacked::Percentage(p) => Self::new_percent(p.to_animated_zero()?),
            Unpacked::Calc(c) => Self::new_calc_unchecked(Box::new(c.to_animated_zero()?)),
        })
    }
}

impl Clone for LengthPercentage {
    fn clone(&self) -> Self {
        match self.unpack() {
            Unpacked::Length(l) => Self::new_length(l),
            Unpacked::Percentage(p) => Self::new_percent(p),
            Unpacked::Calc(c) => Self::new_calc_unchecked(Box::new(c.clone()))
        }
    }
}

impl ToComputedValue for specified::LengthPercentage {
    type ComputedValue = LengthPercentage;

    fn to_computed_value(&self, context: &Context) -> LengthPercentage {
        match *self {
            specified::LengthPercentage::Length(ref value) => {
                LengthPercentage::new_length(value.to_computed_value(context))
            },
            specified::LengthPercentage::Percentage(value) => {
                LengthPercentage::new_percent(value)
            },
            specified::LengthPercentage::Calc(ref calc) => {
                (**calc).to_computed_value(context).to_length_percentge()
            },
        }
    }

    fn from_computed_value(computed: &LengthPercentage) -> Self {
        match computed.unpack() {
            Unpacked::Length(ref l) => {
                specified::LengthPercentage::Length(ToComputedValue::from_computed_value(l))
            }
            Unpacked::Percentage(p) => {
                specified::LengthPercentage::Percentage(p)
            }
            Unpacked::Calc(c) => {
                if let Some(p) = c.as_percentage() {
                    return specified::LengthPercentage::Percentage(p)
                }
                if !c.has_percentage {
                    return specified::LengthPercentage::Length(ToComputedValue::from_computed_value(&c.length_component()))
                }
                specified::LengthPercentage::Calc(Box::new(specified::CalcLengthPercentage::from_computed_value(c)))
            }
        }
    }
}

impl ComputeSquaredDistance for LengthPercentage {
    #[inline]
    fn compute_squared_distance(&self, other: &Self) -> Result<SquaredDistance, ()> {
        // FIXME(nox): This looks incorrect to me, to add a distance between lengths
        // with a distance between percentages.
        Ok(self
            .unclamped_length()
            .compute_squared_distance(&other.unclamped_length())? +
            self.percentage()
                .compute_squared_distance(&other.percentage())?)
    }
}

impl ToCss for LengthPercentage {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        specified::LengthPercentage::from_computed_value(self).to_css(dest)
    }
}

impl Zero for LengthPercentage {
    fn zero() -> Self {
        LengthPercentage::new_length(Length::zero())
    }

    #[inline]
    fn is_zero(&self) -> bool {
        self.is_definitely_zero()
    }
}

impl Serialize for LengthPercentage {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_serializable().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for LengthPercentage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Self::from_serializable(Serializable::deserialize(deserializer)?))
    }
}

/// The representation of a calc() function.
#[derive(
    Clone, Debug, Deserialize, MallocSizeOf, Serialize, ToAnimatedZero, ToResolvedValue,
)]
#[repr(C)]
pub struct CalcLengthPercentage {
    length: Length,

    percentage: Percentage,

    #[animation(constant)]
    clamping_mode: AllowedNumericType,

    /// Whether we specified a percentage or not.
    #[animation(constant)]
    pub has_percentage: bool,
}

impl CalcLengthPercentage {
    /// Returns a new `LengthPercentage`.
    #[inline]
    pub fn new(length: Length, percentage: Option<Percentage>) -> Self {
        Self::with_clamping_mode(length, percentage, AllowedNumericType::All)
    }

    /// Converts this to a `LengthPercentage`, simplifying if possible.
    #[inline]
    pub fn to_length_percentge(self) -> LengthPercentage {
        if !self.has_percentage {
            return LengthPercentage::new_length(self.length_component())
        }
        if self.length.is_zero() {
            return LengthPercentage::new_percent(Percentage(self.clamping_mode.clamp(self.percentage.0)));
        }
        LengthPercentage::new_calc_unchecked(Box::new(self))
    }

    fn specified_percentage(&self) -> Option<Percentage> {
        if self.has_percentage {
            Some(self.percentage)
        } else {
            None
        }
    }

    /// Returns a new `LengthPercentage` with a specific clamping mode.
    #[inline]
    fn with_clamping_mode(
        length: Length,
        percentage: Option<Percentage>,
        clamping_mode: AllowedNumericType,
    ) -> Self {
        Self {
            clamping_mode,
            length,
            percentage: percentage.unwrap_or_default(),
            has_percentage: percentage.is_some(),
        }
    }

    /// Returns the length component of this `calc()`, clamped.
    #[inline]
    pub fn length_component(&self) -> Length {
        Length::new(self.clamping_mode.clamp(self.length.px()))
    }

    /// Returns the percentage component if this could be represented as a
    /// non-calc percentage.
    fn as_percentage(&self) -> Option<Percentage> {
        if !self.has_percentage || self.length.px() != 0. {
            return None;
        }

        Some(Percentage(self.clamping_mode.clamp(self.percentage.0)))
    }

    /// Resolves the percentage.
    #[inline]
    pub fn resolve(&self, basis: Length) -> Length {
        let length = self.length.px() + basis.px() * self.percentage.0;
        Length::new(self.clamping_mode.clamp(length))
    }

    /// Resolves the percentage.
    #[inline]
    pub fn percentage_relative_to(&self, basis: Length) -> Length {
        self.resolve(basis)
    }

    /// Returns the length, without clamping.
    #[inline]
    pub fn unclamped_length(&self) -> Length {
        self.length
    }

    /// Returns true if the computed value is absolute 0 or 0%.
    #[inline]
    fn is_definitely_zero(&self) -> bool {
        self.length.px() == 0.0 && self.percentage.0 == 0.0
    }

    /// Returns the clamped non-negative values.
    #[inline]
    fn clamp_to_non_negative(&self) -> Self {
        if self.has_percentage {
            // If we can eagerly clamp the percentage then just do that.
            if self.length.is_zero() {
                return Self::with_clamping_mode(
                    Length::zero(),
                    Some(self.percentage.clamp_to_non_negative()),
                    AllowedNumericType::NonNegative,
                );
            }
            return Self::with_clamping_mode(self.length, Some(self.percentage), AllowedNumericType::NonNegative);
        }

        Self::with_clamping_mode(
            self.length.clamp_to_non_negative(),
            None,
            AllowedNumericType::NonNegative,
        )
    }
}

// NOTE(emilio): We don't compare `clamping_mode` since we want to preserve the
// invariant that `from_computed_value(length).to_computed_value(..) == length`.
//
// Right now for e.g. a non-negative length, we set clamping_mode to `All`
// unconditionally for non-calc values, and to `NonNegative` for calc.
//
// If we determine that it's sound, from_computed_value() can generate an
// absolute length, which then would get `All` as the clamping mode.
//
// We may want to just eagerly-detect whether we can clamp in
// `LengthPercentage::new` and switch to `AllowedNumericType::NonNegative` then,
// maybe.
impl PartialEq for CalcLengthPercentage {
    fn eq(&self, other: &Self) -> bool {
        self.length == other.length &&
            self.percentage == other.percentage &&
            self.has_percentage == other.has_percentage
    }
}

impl specified::CalcLengthPercentage {
    /// Compute the value, zooming any absolute units by the zoom function.
    fn to_computed_value_with_zoom<F>(
        &self,
        context: &Context,
        zoom_fn: F,
        base_size: FontBaseSize,
    ) -> CalcLengthPercentage
    where
        F: Fn(Length) -> Length,
    {
        use std::f32;
        use crate::values::specified::length::{ViewportPercentageLength, FontRelativeLength};

        let mut length = 0.;

        if let Some(absolute) = self.absolute {
            length += zoom_fn(absolute.to_computed_value(context)).px();
        }

        for val in &[
            self.vw.map(ViewportPercentageLength::Vw),
            self.vh.map(ViewportPercentageLength::Vh),
            self.vmin.map(ViewportPercentageLength::Vmin),
            self.vmax.map(ViewportPercentageLength::Vmax),
        ] {
            if let Some(val) = *val {
                let viewport_size = context.viewport_size_for_viewport_unit_resolution();
                length += val.to_computed_value(viewport_size).px();
            }
        }

        for val in &[
            self.ch.map(FontRelativeLength::Ch),
            self.em.map(FontRelativeLength::Em),
            self.ex.map(FontRelativeLength::Ex),
            self.rem.map(FontRelativeLength::Rem),
        ] {
            if let Some(val) = *val {
                length += val.to_computed_value(context, base_size).px();
            }
        }

        CalcLengthPercentage::with_clamping_mode(
            Length::new(length.min(f32::MAX).max(f32::MIN)),
            self.percentage,
            self.clamping_mode,
        )
    }

    /// Compute font-size or line-height taking into account text-zoom if necessary.
    pub fn to_computed_value_zoomed(
        &self,
        context: &Context,
        base_size: FontBaseSize,
    ) -> CalcLengthPercentage {
        self.to_computed_value_with_zoom(
            context,
            |abs| context.maybe_zoom_text(abs.into()),
            base_size,
        )
    }

    /// Compute the value into pixel length as CSSFloat without context,
    /// so it returns Err(()) if there is any non-absolute unit.
    pub fn to_computed_pixel_length_without_context(&self) -> Result<CSSFloat, ()> {
        if self.vw.is_some() ||
            self.vh.is_some() ||
            self.vmin.is_some() ||
            self.vmax.is_some() ||
            self.em.is_some() ||
            self.ex.is_some() ||
            self.ch.is_some() ||
            self.rem.is_some() ||
            self.percentage.is_some()
        {
            return Err(());
        }

        match self.absolute {
            Some(abs) => Ok(abs.to_px()),
            None => {
                debug_assert!(false, "Someone forgot to handle an unit here: {:?}", self);
                Err(())
            },
        }
    }

    /// Compute the calc using the current font-size (and without text-zoom).
    pub fn to_computed_value(&self, context: &Context) -> CalcLengthPercentage {
        self.to_computed_value_with_zoom(context, |abs| abs, FontBaseSize::CurrentStyle)
    }

    #[inline]
    fn from_computed_value(computed: &CalcLengthPercentage) -> Self {
        use crate::values::specified::length::AbsoluteLength;

        specified::CalcLengthPercentage {
            clamping_mode: computed.clamping_mode,
            absolute: Some(AbsoluteLength::from_computed_value(&computed.length)),
            percentage: computed.specified_percentage(),
            ..Default::default()
        }
    }
}

/// A wrapper of LengthPercentage, whose value must be >= 0.
pub type NonNegativeLengthPercentage = NonNegative<LengthPercentage>;

impl ToAnimatedValue for NonNegativeLengthPercentage {
    type AnimatedValue = LengthPercentage;

    #[inline]
    fn to_animated_value(self) -> Self::AnimatedValue {
        self.0
    }

    #[inline]
    fn from_animated_value(animated: Self::AnimatedValue) -> Self {
        NonNegative(animated.clamp_to_non_negative())
    }
}

impl NonNegativeLengthPercentage {
    /// Returns true if the computed value is absolute 0 or 0%.
    #[inline]
    pub fn is_definitely_zero(&self) -> bool {
        self.0.is_definitely_zero()
    }

    /// Returns the used value.
    #[inline]
    pub fn to_used_value(&self, containing_length: Au) -> Au {
        let resolved = self.0.to_used_value(containing_length);
        std::cmp::max(resolved, Au(0))
    }

    /// Convert the computed value into used value.
    #[inline]
    pub fn maybe_to_used_value(&self, containing_length: Option<Au>) -> Option<Au> {
        let resolved = self
            .0
            .maybe_to_used_value(containing_length.map(|v| v.into()))?;
        Some(std::cmp::max(resolved, Au(0)))
    }
}

