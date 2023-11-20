/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Computed types for box properties.

use crate::values::animated::{Animate, Procedure};
use crate::values::computed::length::{LengthPercentage, NonNegativeLength};
use crate::values::computed::{Context, Integer, ToComputedValue};
use crate::values::generics::box_::{
    GenericContainIntrinsicSize, GenericLineClamp, GenericPerspective, GenericVerticalAlign,
};
use crate::values::specified::box_ as specified;

pub use crate::values::specified::box_::{
    Appearance, BreakBetween, BreakWithin, Clear as SpecifiedClear, Contain, ContainerName,
    ContainerType, ContentVisibility, Display, Float as SpecifiedFloat, Overflow, OverflowAnchor,
    OverflowClipBox, OverscrollBehavior, ScrollSnapAlign, ScrollSnapAxis, ScrollSnapStop,
    ScrollSnapStrictness, ScrollSnapType, ScrollbarGutter, TouchAction, WillChange,
};

/// A computed value for the `vertical-align` property.
pub type VerticalAlign = GenericVerticalAlign<LengthPercentage>;

/// A computed value for the `contain-intrinsic-size` property.
pub type ContainIntrinsicSize = GenericContainIntrinsicSize<NonNegativeLength>;

/// A computed value for the `line-clamp` property.
pub type LineClamp = GenericLineClamp<Integer>;

impl Animate for LineClamp {
    #[inline]
    fn animate(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        if self.is_none() != other.is_none() {
            return Err(());
        }
        if self.is_none() {
            return Ok(Self::none());
        }
        Ok(Self(self.0.animate(&other.0, procedure)?.max(1)))
    }
}

/// A computed value for the `perspective` property.
pub type Perspective = GenericPerspective<NonNegativeLength>;

#[allow(missing_docs)]
#[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    FromPrimitive,
    Hash,
    MallocSizeOf,
    Parse,
    PartialEq,
    SpecifiedValueInfo,
    ToCss,
    ToResolvedValue,
)]
#[repr(u8)]
/// A computed value for the `float` property.
pub enum Float {
    Left,
    Right,
    None,
}

impl Float {
    /// Returns true if `self` is not `None`.
    pub fn is_floating(self) -> bool {
        self != Self::None
    }
}

impl ToComputedValue for SpecifiedFloat {
    type ComputedValue = Float;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        let ltr = context.style().writing_mode.is_bidi_ltr();
        // https://drafts.csswg.org/css-logical-props/#float-clear
        match *self {
            SpecifiedFloat::InlineStart => {
                context
                    .rule_cache_conditions
                    .borrow_mut()
                    .set_writing_mode_dependency(context.builder.writing_mode);
                if ltr {
                    Float::Left
                } else {
                    Float::Right
                }
            },
            SpecifiedFloat::InlineEnd => {
                context
                    .rule_cache_conditions
                    .borrow_mut()
                    .set_writing_mode_dependency(context.builder.writing_mode);
                if ltr {
                    Float::Right
                } else {
                    Float::Left
                }
            },
            SpecifiedFloat::Left => Float::Left,
            SpecifiedFloat::Right => Float::Right,
            SpecifiedFloat::None => Float::None,
        }
    }

    #[inline]
    fn from_computed_value(computed: &Self::ComputedValue) -> SpecifiedFloat {
        match *computed {
            Float::Left => SpecifiedFloat::Left,
            Float::Right => SpecifiedFloat::Right,
            Float::None => SpecifiedFloat::None,
        }
    }
}

#[allow(missing_docs)]
#[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    FromPrimitive,
    Hash,
    MallocSizeOf,
    Parse,
    PartialEq,
    SpecifiedValueInfo,
    ToCss,
    ToResolvedValue,
)]
/// A computed value for the `clear` property.
#[repr(u8)]
pub enum Clear {
    None,
    Left,
    Right,
    Both,
}

impl ToComputedValue for SpecifiedClear {
    type ComputedValue = Clear;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        let ltr = context.style().writing_mode.is_bidi_ltr();
        // https://drafts.csswg.org/css-logical-props/#float-clear
        match *self {
            SpecifiedClear::InlineStart => {
                context
                    .rule_cache_conditions
                    .borrow_mut()
                    .set_writing_mode_dependency(context.builder.writing_mode);
                if ltr {
                    Clear::Left
                } else {
                    Clear::Right
                }
            },
            SpecifiedClear::InlineEnd => {
                context
                    .rule_cache_conditions
                    .borrow_mut()
                    .set_writing_mode_dependency(context.builder.writing_mode);
                if ltr {
                    Clear::Right
                } else {
                    Clear::Left
                }
            },
            SpecifiedClear::None => Clear::None,
            SpecifiedClear::Left => Clear::Left,
            SpecifiedClear::Right => Clear::Right,
            SpecifiedClear::Both => Clear::Both,
        }
    }

    #[inline]
    fn from_computed_value(computed: &Self::ComputedValue) -> SpecifiedClear {
        match *computed {
            Clear::None => SpecifiedClear::None,
            Clear::Left => SpecifiedClear::Left,
            Clear::Right => SpecifiedClear::Right,
            Clear::Both => SpecifiedClear::Both,
        }
    }
}

/// A computed value for the `resize` property.
#[allow(missing_docs)]
#[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
#[derive(Clone, Copy, Debug, Eq, Hash, MallocSizeOf, Parse, PartialEq, ToCss, ToResolvedValue)]
#[repr(u8)]
pub enum Resize {
    None,
    Both,
    Horizontal,
    Vertical,
}

impl ToComputedValue for specified::Resize {
    type ComputedValue = Resize;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> Resize {
        let is_vertical = context.style().writing_mode.is_vertical();
        match self {
            specified::Resize::Inline => {
                context
                    .rule_cache_conditions
                    .borrow_mut()
                    .set_writing_mode_dependency(context.builder.writing_mode);
                if is_vertical {
                    Resize::Vertical
                } else {
                    Resize::Horizontal
                }
            },
            specified::Resize::Block => {
                context
                    .rule_cache_conditions
                    .borrow_mut()
                    .set_writing_mode_dependency(context.builder.writing_mode);
                if is_vertical {
                    Resize::Horizontal
                } else {
                    Resize::Vertical
                }
            },
            specified::Resize::None => Resize::None,
            specified::Resize::Both => Resize::Both,
            specified::Resize::Horizontal => Resize::Horizontal,
            specified::Resize::Vertical => Resize::Vertical,
        }
    }

    #[inline]
    fn from_computed_value(computed: &Resize) -> specified::Resize {
        match computed {
            Resize::None => specified::Resize::None,
            Resize::Both => specified::Resize::Both,
            Resize::Horizontal => specified::Resize::Horizontal,
            Resize::Vertical => specified::Resize::Vertical,
        }
    }
}
