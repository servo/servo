/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Resolved values. These are almost always computed values, but in some cases
//! there are used values.

use crate::properties::ComputedValues;
use crate::ArcSlice;
use cssparser;
use servo_arc::Arc;
use smallvec::SmallVec;

mod color;
mod counters;

use crate::values::computed;

/// Information needed to resolve a given value.
pub struct Context<'a> {
    /// The style we're resolving for. This is useful to resolve currentColor.
    pub style: &'a ComputedValues,
    // TODO(emilio): Add layout box information, and maybe property-specific
    // information?
}

/// A trait to represent the conversion between resolved and resolved values.
///
/// This trait is derivable with `#[derive(ToResolvedValue)]`.
///
/// The deriving code assumes that if the type isn't generic, then the trait can
/// be implemented as simple move. This means that a manual implementation with
/// `ResolvedValue = Self` is bogus if it returns anything else than a clone.
pub trait ToResolvedValue {
    /// The resolved value type we're going to be converted to.
    type ResolvedValue;

    /// Convert a resolved value to a resolved value.
    fn to_resolved_value(self, context: &Context) -> Self::ResolvedValue;

    /// Convert a resolved value to resolved value form.
    fn from_resolved_value(resolved: Self::ResolvedValue) -> Self;
}

macro_rules! trivial_to_resolved_value {
    ($ty:ty) => {
        impl $crate::values::resolved::ToResolvedValue for $ty {
            type ResolvedValue = Self;

            #[inline]
            fn to_resolved_value(self, _: &Context) -> Self {
                self
            }

            #[inline]
            fn from_resolved_value(resolved: Self::ResolvedValue) -> Self {
                resolved
            }
        }
    };
}

trivial_to_resolved_value!(());
trivial_to_resolved_value!(bool);
trivial_to_resolved_value!(f32);
trivial_to_resolved_value!(i32);
trivial_to_resolved_value!(u8);
trivial_to_resolved_value!(i8);
trivial_to_resolved_value!(u16);
trivial_to_resolved_value!(u32);
trivial_to_resolved_value!(usize);
trivial_to_resolved_value!(String);
trivial_to_resolved_value!(Box<str>);
trivial_to_resolved_value!(crate::OwnedStr);
trivial_to_resolved_value!(cssparser::RGBA);
trivial_to_resolved_value!(crate::Atom);
trivial_to_resolved_value!(app_units::Au);
trivial_to_resolved_value!(computed::url::ComputedUrl);
#[cfg(feature = "gecko")]
trivial_to_resolved_value!(computed::url::ComputedImageUrl);
#[cfg(feature = "servo")]
trivial_to_resolved_value!(html5ever::Prefix);
trivial_to_resolved_value!(computed::LengthPercentage);
trivial_to_resolved_value!(style_traits::values::specified::AllowedNumericType);

impl<A, B> ToResolvedValue for (A, B)
where
    A: ToResolvedValue,
    B: ToResolvedValue,
{
    type ResolvedValue = (
        <A as ToResolvedValue>::ResolvedValue,
        <B as ToResolvedValue>::ResolvedValue,
    );

    #[inline]
    fn to_resolved_value(self, context: &Context) -> Self::ResolvedValue {
        (
            self.0.to_resolved_value(context),
            self.1.to_resolved_value(context),
        )
    }

    #[inline]
    fn from_resolved_value(resolved: Self::ResolvedValue) -> Self {
        (
            A::from_resolved_value(resolved.0),
            B::from_resolved_value(resolved.1),
        )
    }
}

impl<T> ToResolvedValue for Option<T>
where
    T: ToResolvedValue,
{
    type ResolvedValue = Option<<T as ToResolvedValue>::ResolvedValue>;

    #[inline]
    fn to_resolved_value(self, context: &Context) -> Self::ResolvedValue {
        self.map(|item| item.to_resolved_value(context))
    }

    #[inline]
    fn from_resolved_value(resolved: Self::ResolvedValue) -> Self {
        resolved.map(T::from_resolved_value)
    }
}

impl<T> ToResolvedValue for SmallVec<[T; 1]>
where
    T: ToResolvedValue,
{
    type ResolvedValue = SmallVec<[<T as ToResolvedValue>::ResolvedValue; 1]>;

    #[inline]
    fn to_resolved_value(self, context: &Context) -> Self::ResolvedValue {
        self.into_iter()
            .map(|item| item.to_resolved_value(context))
            .collect()
    }

    #[inline]
    fn from_resolved_value(resolved: Self::ResolvedValue) -> Self {
        resolved.into_iter().map(T::from_resolved_value).collect()
    }
}

impl<T> ToResolvedValue for Vec<T>
where
    T: ToResolvedValue,
{
    type ResolvedValue = Vec<<T as ToResolvedValue>::ResolvedValue>;

    #[inline]
    fn to_resolved_value(self, context: &Context) -> Self::ResolvedValue {
        self.into_iter()
            .map(|item| item.to_resolved_value(context))
            .collect()
    }

    #[inline]
    fn from_resolved_value(resolved: Self::ResolvedValue) -> Self {
        resolved.into_iter().map(T::from_resolved_value).collect()
    }
}

impl<T> ToResolvedValue for Box<T>
where
    T: ToResolvedValue,
{
    type ResolvedValue = Box<<T as ToResolvedValue>::ResolvedValue>;

    #[inline]
    fn to_resolved_value(self, context: &Context) -> Self::ResolvedValue {
        Box::new(T::to_resolved_value(*self, context))
    }

    #[inline]
    fn from_resolved_value(resolved: Self::ResolvedValue) -> Self {
        Box::new(T::from_resolved_value(*resolved))
    }
}

impl<T> ToResolvedValue for Box<[T]>
where
    T: ToResolvedValue,
{
    type ResolvedValue = Box<[<T as ToResolvedValue>::ResolvedValue]>;

    #[inline]
    fn to_resolved_value(self, context: &Context) -> Self::ResolvedValue {
        Vec::from(self)
            .to_resolved_value(context)
            .into_boxed_slice()
    }

    #[inline]
    fn from_resolved_value(resolved: Self::ResolvedValue) -> Self {
        Vec::from_resolved_value(Vec::from(resolved)).into_boxed_slice()
    }
}

impl<T> ToResolvedValue for crate::OwnedSlice<T>
where
    T: ToResolvedValue,
{
    type ResolvedValue = crate::OwnedSlice<<T as ToResolvedValue>::ResolvedValue>;

    #[inline]
    fn to_resolved_value(self, context: &Context) -> Self::ResolvedValue {
        self.into_box().to_resolved_value(context).into()
    }

    #[inline]
    fn from_resolved_value(resolved: Self::ResolvedValue) -> Self {
        Self::from(Box::from_resolved_value(resolved.into_box()))
    }
}

// NOTE(emilio): This is implementable more generically, but it's unlikely what
// you want there, as it forces you to have an extra allocation.
//
// We could do that if needed, ideally with specialization for the case where
// ResolvedValue = T. But we don't need it for now.
impl<T> ToResolvedValue for Arc<T>
where
    T: ToResolvedValue<ResolvedValue = T>,
{
    type ResolvedValue = Self;

    #[inline]
    fn to_resolved_value(self, _: &Context) -> Self {
        self
    }

    #[inline]
    fn from_resolved_value(resolved: Self) -> Self {
        resolved
    }
}

// Same caveat as above applies.
impl<T> ToResolvedValue for ArcSlice<T>
where
    T: ToResolvedValue<ResolvedValue = T>,
{
    type ResolvedValue = Self;

    #[inline]
    fn to_resolved_value(self, _: &Context) -> Self {
        self
    }

    #[inline]
    fn from_resolved_value(resolved: Self) -> Self {
        resolved
    }
}
