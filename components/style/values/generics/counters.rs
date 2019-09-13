/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Generic types for counters-related CSS values.

#[cfg(feature = "servo")]
use crate::computed_values::list_style_type::T as ListStyleType;
#[cfg(feature = "gecko")]
use crate::values::generics::CounterStyle;
#[cfg(feature = "gecko")]
use crate::values::specified::Attr;
use crate::values::CustomIdent;
use std::ops::Deref;

/// A name / value pair for counters.
#[derive(
    Clone,
    Debug,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
#[repr(C)]
pub struct GenericCounterPair<Integer> {
    /// The name of the counter.
    pub name: CustomIdent,
    /// The value of the counter / increment / etc.
    pub value: Integer,
}
pub use self::GenericCounterPair as CounterPair;

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
pub struct GenericCounterIncrement<I>(pub GenericCounters<I>);
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

/// A generic value for the `counter-set` and `counter-reset` properties.
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
pub struct GenericCounterSetOrReset<I>(pub GenericCounters<I>);
pub use self::GenericCounterSetOrReset as CounterSetOrReset;

impl<I> CounterSetOrReset<I> {
    /// Returns a new value for `counter-set` / `counter-reset`.
    #[inline]
    pub fn new(counters: Vec<CounterPair<I>>) -> Self {
        CounterSetOrReset(Counters(counters.into()))
    }
}

impl<I> Deref for CounterSetOrReset<I> {
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
    #[css(iterable, if_empty = "none")] crate::OwnedSlice<GenericCounterPair<I>>,
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
    Clone,
    Debug,
    Eq,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
pub enum Content<ImageUrl> {
    /// `normal` reserved keyword.
    Normal,
    /// `none` reserved keyword.
    None,
    /// `-moz-alt-content`.
    #[cfg(feature = "gecko")]
    MozAltContent,
    /// Content items.
    Items(#[css(iterable)] Box<[ContentItem<ImageUrl>]>),
}

impl<ImageUrl> Content<ImageUrl> {
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
    SpecifiedValueInfo,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
pub enum ContentItem<ImageUrl> {
    /// Literal string content.
    String(Box<str>),
    /// `counter(name, style)`.
    #[css(comma, function)]
    Counter(CustomIdent, #[css(skip_if = "is_decimal")] CounterStyleType),
    /// `counters(name, separator, style)`.
    #[css(comma, function)]
    Counters(
        CustomIdent,
        Box<str>,
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
    /// `attr([namespace? `|`]? ident)`
    #[cfg(feature = "gecko")]
    Attr(Attr),
    /// `url(url)`
    Url(ImageUrl),
}
