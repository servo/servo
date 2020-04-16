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
use crate::values::animated::{Animate, Procedure, ToAnimatedValue, ToAnimatedZero};
use crate::values::distance::{ComputeSquaredDistance, SquaredDistance};
use crate::values::generics::{calc, NonNegative};
use crate::values::specified::length::FontBaseSize;
use crate::values::{specified, CSSFloat};
use crate::Zero;
use app_units::Au;
use malloc_size_of::{MallocSizeOf, MallocSizeOfOps};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::fmt::{self, Write};
use style_traits::values::specified::AllowedNumericType;
use style_traits::{CssWriter, ToCss};

#[doc(hidden)]
#[derive(Clone, Copy)]
#[repr(C)]
pub struct LengthVariant {
    tag: u8,
    length: Length,
}

#[doc(hidden)]
#[derive(Clone, Copy)]
#[repr(C)]
pub struct PercentageVariant {
    tag: u8,
    percentage: Percentage,
}

// NOTE(emilio): cbindgen only understands the #[cfg] on the top level
// definition.
#[doc(hidden)]
#[derive(Clone, Copy)]
#[repr(C)]
#[cfg(target_pointer_width = "32")]
pub struct CalcVariant {
    tag: u8,
    ptr: *mut CalcLengthPercentage,
}

#[doc(hidden)]
#[derive(Clone, Copy)]
#[repr(C)]
#[cfg(target_pointer_width = "64")]
pub struct CalcVariant {
    ptr: usize, // In little-endian byte order
}

// `CalcLengthPercentage` is `Send + Sync` as asserted below.
unsafe impl Send for CalcVariant {}
unsafe impl Sync for CalcVariant {}

#[doc(hidden)]
#[derive(Clone, Copy)]
#[repr(C)]
pub struct TagVariant {
    tag: u8,
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
    pub const TAG_CALC: u8 = 0;
    #[doc(hidden)]
    pub const TAG_LENGTH: u8 = 1;
    #[doc(hidden)]
    pub const TAG_PERCENTAGE: u8 = 2;
    #[doc(hidden)]
    pub const TAG_MASK: u8 = 0b11;
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
enum Tag {
    Calc = LengthPercentageUnion::TAG_CALC,
    Length = LengthPercentageUnion::TAG_LENGTH,
    Percentage = LengthPercentageUnion::TAG_PERCENTAGE,
}

// All the members should be 64 bits, even in 32-bit builds.
#[allow(unused)]
unsafe fn static_assert() {
    fn assert_send_and_sync<T: Send + Sync>() {}
    std::mem::transmute::<u64, LengthVariant>(0u64);
    std::mem::transmute::<u64, PercentageVariant>(0u64);
    std::mem::transmute::<u64, CalcVariant>(0u64);
    std::mem::transmute::<u64, LengthPercentage>(0u64);
    assert_send_and_sync::<LengthVariant>();
    assert_send_and_sync::<PercentageVariant>();
    assert_send_and_sync::<CalcLengthPercentage>();
}

impl Drop for LengthPercentage {
    fn drop(&mut self) {
        if self.tag() == Tag::Calc {
            let _ = unsafe { Box::from_raw(self.calc_ptr()) };
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
#[derive(Clone, Debug, PartialEq, ToCss)]
enum Unpacked<'a> {
    Calc(&'a CalcLengthPercentage),
    Length(Length),
    Percentage(Percentage),
}

/// An unpacked `<length-percentage>` that mutably borrows the `calc()` variant.
enum UnpackedMut<'a> {
    Calc(&'a mut CalcLengthPercentage),
    Length(Length),
    Percentage(Percentage),
}

/// An unpacked `<length-percentage>` that owns the `calc()` variant, for
/// serialization purposes.
#[derive(Deserialize, PartialEq, Serialize)]
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

    /// 0%
    #[inline]
    pub fn zero_percent() -> Self {
        Self::new_percent(Percentage::zero())
    }

    fn to_calc_node(&self) -> Cow<CalcNode> {
        match self.unpack() {
            Unpacked::Length(l) => Cow::Owned(CalcNode::Leaf(CalcLengthPercentageLeaf::Length(l))),
            Unpacked::Percentage(p) => {
                Cow::Owned(CalcNode::Leaf(CalcLengthPercentageLeaf::Percentage(p)))
            },
            Unpacked::Calc(p) => Cow::Borrowed(&p.node),
        }
    }

    /// Constructs a length value.
    #[inline]
    pub fn new_length(length: Length) -> Self {
        let length = Self(LengthPercentageUnion {
            length: LengthVariant {
                tag: LengthPercentageUnion::TAG_LENGTH,
                length,
            },
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
            },
        });
        debug_assert_eq!(percent.tag(), Tag::Percentage);
        percent
    }

    /// Given a `LengthPercentage` value `v`, construct the value representing
    /// `calc(100% - v)`.
    pub fn hundred_percent_minus(v: Self, clamping_mode: AllowedNumericType) -> Self {
        // TODO: This could in theory take ownership of the calc node in `v` if
        // possible instead of cloning.
        let mut node = v.to_calc_node().into_owned();
        node.negate();

        let new_node = CalcNode::Sum(
            vec![
                CalcNode::Leaf(CalcLengthPercentageLeaf::Percentage(Percentage::hundred())),
                node,
            ]
            .into(),
        );

        Self::new_calc(new_node, clamping_mode)
    }

    /// Constructs a `calc()` value.
    #[inline]
    pub fn new_calc(mut node: CalcNode, clamping_mode: AllowedNumericType) -> Self {
        node.simplify_and_sort();

        match node {
            CalcNode::Leaf(l) => {
                return match l {
                    CalcLengthPercentageLeaf::Length(l) => {
                        Self::new_length(Length::new(clamping_mode.clamp(l.px())))
                    },
                    CalcLengthPercentageLeaf::Percentage(p) => {
                        Self::new_percent(Percentage(clamping_mode.clamp(p.0)))
                    },
                }
            },
            _ => Self::new_calc_unchecked(Box::new(CalcLengthPercentage {
                clamping_mode,
                node,
            })),
        }
    }

    /// Private version of new_calc() that constructs a calc() variant without
    /// checking.
    fn new_calc_unchecked(calc: Box<CalcLengthPercentage>) -> Self {
        let ptr = Box::into_raw(calc);

        #[cfg(target_pointer_width = "32")]
        let calc = CalcVariant {
            tag: LengthPercentageUnion::TAG_CALC,
            ptr,
        };

        #[cfg(target_pointer_width = "64")]
        let calc = CalcVariant {
            #[cfg(target_endian = "little")]
            ptr: ptr as usize,
            #[cfg(target_endian = "big")]
            ptr: (ptr as usize).swap_bytes(),
        };

        let calc = Self(LengthPercentageUnion { calc });
        debug_assert_eq!(calc.tag(), Tag::Calc);
        calc
    }

    #[inline]
    fn tag(&self) -> Tag {
        match unsafe { self.0.tag.tag & LengthPercentageUnion::TAG_MASK } {
            LengthPercentageUnion::TAG_CALC => Tag::Calc,
            LengthPercentageUnion::TAG_LENGTH => Tag::Length,
            LengthPercentageUnion::TAG_PERCENTAGE => Tag::Percentage,
            _ => unsafe { debug_unreachable!("Bogus tag?") },
        }
    }

    #[inline]
    fn unpack_mut<'a>(&'a mut self) -> UnpackedMut<'a> {
        unsafe {
            match self.tag() {
                Tag::Calc => UnpackedMut::Calc(&mut *self.calc_ptr()),
                Tag::Length => UnpackedMut::Length(self.0.length.length),
                Tag::Percentage => UnpackedMut::Percentage(self.0.percentage.percentage),
            }
        }
    }

    #[inline]
    fn unpack<'a>(&'a self) -> Unpacked<'a> {
        unsafe {
            match self.tag() {
                Tag::Calc => Unpacked::Calc(&*self.calc_ptr()),
                Tag::Length => Unpacked::Length(self.0.length.length),
                Tag::Percentage => Unpacked::Percentage(self.0.percentage.percentage),
            }
        }
    }

    #[inline]
    unsafe fn calc_ptr(&self) -> *mut CalcLengthPercentage {
        #[cfg(not(all(target_endian = "big", target_pointer_width = "64")))]
        {
            self.0.calc.ptr as *mut _
        }
        #[cfg(all(target_endian = "big", target_pointer_width = "64"))]
        {
            self.0.calc.ptr.swap_bytes() as *mut _
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
            Unpacked::Calc(..) => false,
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
            Unpacked::Percentage(..) | Unpacked::Calc(..) => true,
        }
    }

    /// Converts to a `<length>` if possible.
    pub fn to_length(&self) -> Option<Length> {
        match self.unpack() {
            Unpacked::Length(l) => Some(l),
            Unpacked::Percentage(..) | Unpacked::Calc(..) => {
                debug_assert!(self.has_percentage());
                return None;
            },
        }
    }

    /// Converts to a `<percentage>` if possible.
    #[inline]
    pub fn to_percentage(&self) -> Option<Percentage> {
        match self.unpack() {
            Unpacked::Percentage(p) => Some(p),
            Unpacked::Length(..) | Unpacked::Calc(..) => None,
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
    pub fn maybe_to_used_value(&self, container_len: Option<Length>) -> Option<Au> {
        self.maybe_percentage_relative_to(container_len)
            .map(Au::from)
    }

    /// If there are special rules for computing percentages in a value (e.g.
    /// the height property), they apply whenever a calc() expression contains
    /// percentages.
    pub fn maybe_percentage_relative_to(&self, container_len: Option<Length>) -> Option<Length> {
        if let Unpacked::Length(l) = self.unpack() {
            return Some(l);
        }
        Some(self.resolve(container_len?))
    }

    /// Returns the clamped non-negative values.
    #[inline]
    pub fn clamp_to_non_negative(mut self) -> Self {
        match self.unpack_mut() {
            UnpackedMut::Length(l) => Self::new_length(l.clamp_to_non_negative()),
            UnpackedMut::Percentage(p) => Self::new_percent(p.clamp_to_non_negative()),
            UnpackedMut::Calc(ref mut c) => {
                c.clamping_mode = AllowedNumericType::NonNegative;
                self
            },
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
            Unpacked::Calc(c) => Self::new_calc_unchecked(Box::new(c.clone())),
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
            specified::LengthPercentage::Percentage(value) => LengthPercentage::new_percent(value),
            specified::LengthPercentage::Calc(ref calc) => (**calc).to_computed_value(context),
        }
    }

    fn from_computed_value(computed: &LengthPercentage) -> Self {
        match computed.unpack() {
            Unpacked::Length(ref l) => {
                specified::LengthPercentage::Length(ToComputedValue::from_computed_value(l))
            },
            Unpacked::Percentage(p) => specified::LengthPercentage::Percentage(p),
            Unpacked::Calc(c) => {
                // We simplify before constructing the LengthPercentage if
                // needed, so this is always fine.
                specified::LengthPercentage::Calc(Box::new(
                    specified::CalcLengthPercentage::from_computed_value(c),
                ))
            },
        }
    }
}

impl ComputeSquaredDistance for LengthPercentage {
    #[inline]
    fn compute_squared_distance(&self, other: &Self) -> Result<SquaredDistance, ()> {
        // A somewhat arbitrary base, it doesn't really make sense to mix
        // lengths with percentages, but we can't do much better here, and this
        // ensures that the distance between length-only and percentage-only
        // lengths makes sense.
        let basis = Length::new(100.);
        self.resolve(basis)
            .compute_squared_distance(&other.resolve(basis))
    }
}

impl ToCss for LengthPercentage {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        self.unpack().to_css(dest)
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
        Ok(Self::from_serializable(Serializable::deserialize(
            deserializer,
        )?))
    }
}

/// The leaves of a `<length-percentage>` calc expression.
#[derive(
    Clone,
    Debug,
    Deserialize,
    MallocSizeOf,
    PartialEq,
    Serialize,
    ToAnimatedZero,
    ToCss,
    ToResolvedValue,
)]
#[allow(missing_docs)]
#[repr(u8)]
pub enum CalcLengthPercentageLeaf {
    Length(Length),
    Percentage(Percentage),
}

impl CalcLengthPercentageLeaf {
    fn is_zero_length(&self) -> bool {
        match *self {
            Self::Length(ref l) => l.is_zero(),
            Self::Percentage(..) => false,
        }
    }
}

impl PartialOrd for CalcLengthPercentageLeaf {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        use self::CalcLengthPercentageLeaf::*;

        if std::mem::discriminant(self) != std::mem::discriminant(other) {
            return None;
        }

        match (self, other) {
            (&Length(ref one), &Length(ref other)) => one.partial_cmp(other),
            (&Percentage(ref one), &Percentage(ref other)) => one.partial_cmp(other),
            _ => {
                match *self {
                    Length(..) | Percentage(..) => {},
                }
                unsafe {
                    debug_unreachable!("Forgot a branch?");
                }
            },
        }
    }
}

impl calc::CalcNodeLeaf for CalcLengthPercentageLeaf {
    fn is_negative(&self) -> bool {
        match *self {
            Self::Length(ref l) => l.px() < 0.,
            Self::Percentage(ref p) => p.0 < 0.,
        }
    }

    fn try_sum_in_place(&mut self, other: &Self) -> Result<(), ()> {
        use self::CalcLengthPercentageLeaf::*;

        // 0px plus anything else is equal to the right hand side.
        if self.is_zero_length() {
            *self = other.clone();
            return Ok(());
        }

        if other.is_zero_length() {
            return Ok(());
        }

        match (self, other) {
            (&mut Length(ref mut one), &Length(ref other)) => {
                *one += *other;
            },
            (&mut Percentage(ref mut one), &Percentage(ref other)) => {
                one.0 += other.0;
            },
            _ => return Err(()),
        }

        Ok(())
    }

    fn mul_by(&mut self, scalar: f32) {
        match *self {
            Self::Length(ref mut l) => *l = *l * scalar,
            Self::Percentage(ref mut p) => p.0 *= scalar,
        }
    }

    fn simplify(&mut self) {}

    fn sort_key(&self) -> calc::SortKey {
        match *self {
            Self::Length(..) => calc::SortKey::Px,
            Self::Percentage(..) => calc::SortKey::Percentage,
        }
    }
}

/// The computed version of a calc() node for `<length-percentage>` values.
pub type CalcNode = calc::GenericCalcNode<CalcLengthPercentageLeaf>;

/// The representation of a calc() function with mixed lengths and percentages.
#[derive(
    Clone, Debug, Deserialize, MallocSizeOf, Serialize, ToAnimatedZero, ToResolvedValue, ToCss,
)]
#[repr(C)]
pub struct CalcLengthPercentage {
    #[animation(constant)]
    #[css(skip)]
    clamping_mode: AllowedNumericType,
    node: CalcNode,
}

impl CalcLengthPercentage {
    /// Resolves the percentage.
    #[inline]
    fn resolve(&self, basis: Length) -> Length {
        // unwrap() is fine because the conversion below is infallible.
        let px = self
            .node
            .resolve(|l| {
                Ok(match *l {
                    CalcLengthPercentageLeaf::Length(l) => l.px(),
                    CalcLengthPercentageLeaf::Percentage(ref p) => basis.px() * p.0,
                })
            })
            .unwrap();
        Length::new(self.clamping_mode.clamp(px))
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
        self.node == other.node
    }
}

impl specified::CalcLengthPercentage {
    /// Compute the value, zooming any absolute units by the zoom function.
    fn to_computed_value_with_zoom<F>(
        &self,
        context: &Context,
        zoom_fn: F,
        base_size: FontBaseSize,
    ) -> LengthPercentage
    where
        F: Fn(Length) -> Length,
    {
        use crate::values::specified::calc::Leaf;
        use crate::values::specified::length::NoCalcLength;

        let node = self.node.map_leaves(|leaf| match *leaf {
            Leaf::Percentage(p) => CalcLengthPercentageLeaf::Percentage(Percentage(p)),
            Leaf::Length(l) => CalcLengthPercentageLeaf::Length(match l {
                NoCalcLength::Absolute(ref abs) => zoom_fn(abs.to_computed_value(context)),
                NoCalcLength::FontRelative(ref fr) => fr.to_computed_value(context, base_size),
                other => other.to_computed_value(context),
            }),
            Leaf::Number(..) | Leaf::Angle(..) | Leaf::Time(..) => {
                unreachable!("Shouldn't have parsed")
            },
        });

        LengthPercentage::new_calc(node, self.clamping_mode)
    }

    /// Compute font-size or line-height taking into account text-zoom if necessary.
    pub fn to_computed_value_zoomed(
        &self,
        context: &Context,
        base_size: FontBaseSize,
    ) -> LengthPercentage {
        self.to_computed_value_with_zoom(
            context,
            |abs| context.maybe_zoom_text(abs.into()),
            base_size,
        )
    }

    /// Compute the value into pixel length as CSSFloat without context,
    /// so it returns Err(()) if there is any non-absolute unit.
    pub fn to_computed_pixel_length_without_context(&self) -> Result<CSSFloat, ()> {
        use crate::values::specified::calc::Leaf;
        use crate::values::specified::length::NoCalcLength;

        // Simplification should've turned this into an absolute length,
        // otherwise it wouldn't have been able to.
        match self.node {
            calc::CalcNode::Leaf(Leaf::Length(NoCalcLength::Absolute(ref l))) => Ok(l.to_px()),
            _ => Err(()),
        }
    }

    /// Compute the calc using the current font-size (and without text-zoom).
    pub fn to_computed_value(&self, context: &Context) -> LengthPercentage {
        self.to_computed_value_with_zoom(context, |abs| abs, FontBaseSize::CurrentStyle)
    }

    #[inline]
    fn from_computed_value(computed: &CalcLengthPercentage) -> Self {
        use crate::values::specified::calc::Leaf;
        use crate::values::specified::length::NoCalcLength;

        specified::CalcLengthPercentage {
            clamping_mode: computed.clamping_mode,
            node: computed.node.map_leaves(|l| match l {
                CalcLengthPercentageLeaf::Length(ref l) => {
                    Leaf::Length(NoCalcLength::from_px(l.px()))
                },
                CalcLengthPercentageLeaf::Percentage(ref p) => Leaf::Percentage(p.0),
            }),
        }
    }
}

/// https://drafts.csswg.org/css-transitions/#animtype-lpcalc
/// https://drafts.csswg.org/css-values-4/#combine-math
/// https://drafts.csswg.org/css-values-4/#combine-mixed
impl Animate for LengthPercentage {
    #[inline]
    fn animate(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        Ok(match (self.unpack(), other.unpack()) {
            (Unpacked::Length(one), Unpacked::Length(other)) => {
                Self::new_length(one.animate(&other, procedure)?)
            },
            (Unpacked::Percentage(one), Unpacked::Percentage(other)) => {
                Self::new_percent(one.animate(&other, procedure)?)
            },
            _ => {
                let mut one = self.to_calc_node().into_owned();
                let mut other = other.to_calc_node().into_owned();
                let (l, r) = procedure.weights();

                one.mul_by(l as f32);
                other.mul_by(r as f32);

                Self::new_calc(
                    CalcNode::Sum(vec![one, other].into()),
                    AllowedNumericType::All,
                )
            },
        })
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
