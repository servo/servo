/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Generic types for counters-related CSS values.

#[cfg(feature = "servo")]
use crate::computed_values::list_style_type::T as ListStyleType;
#[cfg(feature = "gecko")]
use crate::values::generics::CounterStyle;
use crate::values::specified::Attr;
use crate::values::CustomIdent;
use std::fmt::{self, Write};
use std::ops::Deref;
use style_traits::{CssWriter, ToCss};

/// A name / value pair for counters.
#[derive(
    Clone,
    Debug,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToComputedValue,
    ToResolvedValue,
    ToShmem,
)]
#[repr(C)]
pub struct GenericCounterPair<Integer> {
    /// The name of the counter.
    pub name: CustomIdent,
    /// The value of the counter / increment / etc.
    pub value: Integer,
    /// If true, then this represents `reversed(name)`.
    /// NOTE: It can only be true on `counter-reset` values.
    pub is_reversed: bool,
}
pub use self::GenericCounterPair as CounterPair;

impl<Integer> ToCss for CounterPair<Integer>
where
    Integer: ToCss + PartialEq<i32>,
{
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        if self.is_reversed {
            dest.write_str("reversed(")?;
        }
        self.name.to_css(dest)?;
        if self.is_reversed {
            dest.write_char(')')?;
            if self.value == i32::min_value() {
                return Ok(());
            }
        }
        dest.write_char(' ')?;
        self.value.to_css(dest)
    }
}

/// A generic value for the `counter-increment` property.
#[derive(
    Clone,
    Debug,
    Default,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
#[repr(transparent)]
pub struct GenericCounterIncrement<I>(#[css(field_bound)] pub GenericCounters<I>);
pub use self::GenericCounterIncrement as CounterIncrement;

impl<I> CounterIncrement<I> {
    /// Returns a new value for `counter-increment`.
    #[inline]
    pub fn new(counters: Vec<CounterPair<I>>) -> Self {
        CounterIncrement(Counters(counters.into()))
    }
}

impl<I> Deref for CounterIncrement<I> {
    type Target = [CounterPair<I>];

    #[inline]
    fn deref(&self) -> &Self::Target {
        &(self.0).0
    }
}

/// A generic value for the `counter-set` property.
#[derive(
    Clone,
    Debug,
    Default,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
#[repr(transparent)]
pub struct GenericCounterSet<I>(#[css(field_bound)] pub GenericCounters<I>);
pub use self::GenericCounterSet as CounterSet;

impl<I> CounterSet<I> {
    /// Returns a new value for `counter-set`.
    #[inline]
    pub fn new(counters: Vec<CounterPair<I>>) -> Self {
        CounterSet(Counters(counters.into()))
    }
}

impl<I> Deref for CounterSet<I> {
    type Target = [CounterPair<I>];

    #[inline]
    fn deref(&self) -> &Self::Target {
        &(self.0).0
    }
}

/// A generic value for the `counter-reset` property.
#[derive(
    Clone,
    Debug,
    Default,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
#[repr(transparent)]
pub struct GenericCounterReset<I>(#[css(field_bound)] pub GenericCounters<I>);
pub use self::GenericCounterReset as CounterReset;

impl<I> CounterReset<I> {
    /// Returns a new value for `counter-reset`.
    #[inline]
    pub fn new(counters: Vec<CounterPair<I>>) -> Self {
        CounterReset(Counters(counters.into()))
    }
}

impl<I> Deref for CounterReset<I> {
    type Target = [CounterPair<I>];

    #[inline]
    fn deref(&self) -> &Self::Target {
        &(self.0).0
    }
}

/// A generic value for lists of counters.
///
/// Keyword `none` is represented by an empty vector.
#[derive(
    Clone,
    Debug,
    Default,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
#[repr(transparent)]
pub struct GenericCounters<I>(
    #[css(field_bound)]
    #[css(iterable, if_empty = "none")]
    crate::OwnedSlice<GenericCounterPair<I>>,
);
pub use self::GenericCounters as Counters;

#[cfg(feature = "servo")]
type CounterStyleType = ListStyleType;

#[cfg(feature = "gecko")]
type CounterStyleType = CounterStyle;

#[cfg(feature = "servo")]
#[inline]
fn is_decimal(counter_type: &CounterStyleType) -> bool {
    *counter_type == ListStyleType::Decimal
}

#[cfg(feature = "gecko")]
#[inline]
fn is_decimal(counter_type: &CounterStyleType) -> bool {
    *counter_type == CounterStyle::decimal()
}

/// The specified value for the `content` property.
///
/// https://drafts.csswg.org/css-content/#propdef-content
#[derive(
    Clone, Debug, Eq, MallocSizeOf, PartialEq, SpecifiedValueInfo, ToComputedValue, ToCss, ToShmem,
)]
#[repr(u8)]
pub enum GenericContent<Image> {
    /// `normal` reserved keyword.
    Normal,
    /// `none` reserved keyword.
    None,
    /// Content items.
    Items(#[css(iterable)] crate::OwnedSlice<GenericContentItem<Image>>),
}

pub use self::GenericContent as Content;

impl<Image> Content<Image> {
    /// Whether `self` represents list of items.
    #[inline]
    pub fn is_items(&self) -> bool {
        matches!(*self, Self::Items(..))
    }

    /// Set `content` property to `normal`.
    #[inline]
    pub fn normal() -> Self {
        Content::Normal
    }
}

/// Items for the `content` property.
#[derive(
    Clone,
    Debug,
    Eq,
    MallocSizeOf,
    PartialEq,
    ToComputedValue,
    SpecifiedValueInfo,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
#[repr(u8)]
pub enum GenericContentItem<I> {
    /// Literal string content.
    String(crate::OwnedStr),
    /// `counter(name, style)`.
    #[css(comma, function)]
    Counter(CustomIdent, #[css(skip_if = "is_decimal")] CounterStyleType),
    /// `counters(name, separator, style)`.
    #[css(comma, function)]
    Counters(
        CustomIdent,
        crate::OwnedStr,
        #[css(skip_if = "is_decimal")] CounterStyleType,
    ),
    /// `open-quote`.
    OpenQuote,
    /// `close-quote`.
    CloseQuote,
    /// `no-open-quote`.
    NoOpenQuote,
    /// `no-close-quote`.
    NoCloseQuote,
    /// `-moz-alt-content`.
    #[cfg(feature = "gecko")]
    MozAltContent,
    /// `-moz-label-content`.
    /// This is needed to make `accesskey` work for XUL labels. It's basically
    /// attr(value) otherwise.
    #[cfg(feature = "gecko")]
    MozLabelContent,
    /// `attr([namespace? `|`]? ident)`
    Attr(Attr),
    /// image-set(url) | url(url)
    Image(I),
}

pub use self::GenericContentItem as ContentItem;
