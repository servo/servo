/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Common [values][values] used in CSS.
//!
//! [values]: https://drafts.csswg.org/css-values/

#![deny(missing_docs)]

use crate::parser::{Parse, ParserContext};
use crate::values::distance::{ComputeSquaredDistance, SquaredDistance};
use crate::Atom;
pub use cssparser::{serialize_identifier, serialize_name, CowRcStr, Parser};
pub use cssparser::{SourceLocation, Token, RGBA};
use precomputed_hash::PrecomputedHash;
use selectors::parser::SelectorParseErrorKind;
use std::fmt::{self, Debug, Write};
use std::hash;
use style_traits::{CssWriter, ParseError, StyleParseErrorKind, ToCss};
use to_shmem::impl_trivial_to_shmem;

#[cfg(feature = "gecko")]
pub use crate::gecko::url::CssUrl;
#[cfg(feature = "servo")]
pub use crate::servo::url::CssUrl;

pub mod animated;
pub mod computed;
pub mod distance;
pub mod generics;
pub mod resolved;
pub mod specified;

/// A CSS float value.
pub type CSSFloat = f32;

/// Normalizes a float value to zero after a set of operations that might turn
/// it into NaN.
#[inline]
pub fn normalize(v: CSSFloat) -> CSSFloat {
    if v.is_nan() {
        0.0
    } else {
        v
    }
}

/// A CSS integer value.
pub type CSSInteger = i32;

define_keyword_type!(None_, "none");
define_keyword_type!(Auto, "auto");

/// Serialize an identifier which is represented as an atom.
#[cfg(feature = "gecko")]
pub fn serialize_atom_identifier<W>(ident: &Atom, dest: &mut W) -> fmt::Result
where
    W: Write,
{
    ident.with_str(|s| serialize_identifier(s, dest))
}

/// Serialize an identifier which is represented as an atom.
#[cfg(feature = "servo")]
pub fn serialize_atom_identifier<Static, W>(
    ident: &::string_cache::Atom<Static>,
    dest: &mut W,
) -> fmt::Result
where
    Static: string_cache::StaticAtomSet,
    W: Write,
{
    serialize_identifier(&ident, dest)
}

/// Serialize a name which is represented as an Atom.
#[cfg(feature = "gecko")]
pub fn serialize_atom_name<W>(ident: &Atom, dest: &mut W) -> fmt::Result
where
    W: Write,
{
    ident.with_str(|s| serialize_name(s, dest))
}

/// Serialize a name which is represented as an Atom.
#[cfg(feature = "servo")]
pub fn serialize_atom_name<Static, W>(
    ident: &::string_cache::Atom<Static>,
    dest: &mut W,
) -> fmt::Result
where
    Static: string_cache::StaticAtomSet,
    W: Write,
{
    serialize_name(&ident, dest)
}

/// A CSS string stored as an `Atom`.
#[repr(transparent)]
#[derive(
    Clone,
    Debug,
    Default,
    Deref,
    Eq,
    Hash,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToComputedValue,
    ToResolvedValue,
    ToShmem,
)]
pub struct AtomString(pub Atom);

#[cfg(feature = "servo")]
impl AsRef<str> for AtomString {
    fn as_ref(&self) -> &str {
        &*self.0
    }
}

impl cssparser::ToCss for AtomString {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
    where
        W: Write,
    {
        #[cfg(feature = "servo")]
        {
            cssparser::CssStringWriter::new(dest).write_str(self.as_ref())
        }
        #[cfg(feature = "gecko")]
        {
            self.0
                .with_str(|s| cssparser::CssStringWriter::new(dest).write_str(s))
        }
    }
}

impl PrecomputedHash for AtomString {
    #[inline]
    fn precomputed_hash(&self) -> u32 {
        self.0.precomputed_hash()
    }
}

impl<'a> From<&'a str> for AtomString {
    #[inline]
    fn from(string: &str) -> Self {
        Self(Atom::from(string))
    }
}

/// A generic CSS `<ident>` stored as an `Atom`.
#[cfg(feature = "servo")]
#[repr(transparent)]
#[derive(Deref)]
pub struct GenericAtomIdent<Set>(pub string_cache::Atom<Set>)
where
    Set: string_cache::StaticAtomSet;

/// A generic CSS `<ident>` stored as an `Atom`, for the default atom set.
#[cfg(feature = "servo")]
pub type AtomIdent = GenericAtomIdent<servo_atoms::AtomStaticSet>;

#[cfg(feature = "servo")]
impl<Set: string_cache::StaticAtomSet> style_traits::SpecifiedValueInfo for GenericAtomIdent<Set> {}

#[cfg(feature = "servo")]
impl<Set: string_cache::StaticAtomSet> Default for GenericAtomIdent<Set> {
    fn default() -> Self {
        Self(string_cache::Atom::default())
    }
}

#[cfg(feature = "servo")]
impl<Set: string_cache::StaticAtomSet> std::fmt::Debug for GenericAtomIdent<Set> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(feature = "servo")]
impl<Set: string_cache::StaticAtomSet> std::hash::Hash for GenericAtomIdent<Set> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}

#[cfg(feature = "servo")]
impl<Set: string_cache::StaticAtomSet> Eq for GenericAtomIdent<Set> {}

#[cfg(feature = "servo")]
impl<Set: string_cache::StaticAtomSet> PartialEq for GenericAtomIdent<Set> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

#[cfg(feature = "servo")]
impl<Set: string_cache::StaticAtomSet> Clone for GenericAtomIdent<Set> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

#[cfg(feature = "servo")]
impl<Set: string_cache::StaticAtomSet> to_shmem::ToShmem for GenericAtomIdent<Set> {
    fn to_shmem(&self, builder: &mut to_shmem::SharedMemoryBuilder) -> to_shmem::Result<Self> {
        use std::mem::ManuallyDrop;

        let atom = self.0.to_shmem(builder)?;
        Ok(ManuallyDrop::new(Self(ManuallyDrop::into_inner(atom))))
    }
}

#[cfg(feature = "servo")]
impl<Set: string_cache::StaticAtomSet> malloc_size_of::MallocSizeOf for GenericAtomIdent<Set> {
    fn size_of(&self, ops: &mut malloc_size_of::MallocSizeOfOps) -> usize {
        self.0.size_of(ops)
    }
}

#[cfg(feature = "servo")]
impl<Set: string_cache::StaticAtomSet> cssparser::ToCss for GenericAtomIdent<Set> {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
    where
        W: Write,
    {
        serialize_atom_identifier(&self.0, dest)
    }
}

#[cfg(feature = "servo")]
impl<Set: string_cache::StaticAtomSet> PrecomputedHash for GenericAtomIdent<Set> {
    #[inline]
    fn precomputed_hash(&self) -> u32 {
        self.0.precomputed_hash()
    }
}

#[cfg(feature = "servo")]
impl<'a, Set: string_cache::StaticAtomSet> From<&'a str> for GenericAtomIdent<Set> {
    #[inline]
    fn from(string: &str) -> Self {
        Self(string_cache::Atom::from(string))
    }
}

#[cfg(feature = "servo")]
impl<Set: string_cache::StaticAtomSet> std::borrow::Borrow<string_cache::Atom<Set>>
    for GenericAtomIdent<Set>
{
    #[inline]
    fn borrow(&self) -> &string_cache::Atom<Set> {
        &self.0
    }
}

#[cfg(feature = "servo")]
impl<Set: string_cache::StaticAtomSet> GenericAtomIdent<Set> {
    /// Constructs a new GenericAtomIdent.
    #[inline]
    pub fn new(atom: string_cache::Atom<Set>) -> Self {
        Self(atom)
    }

    /// Cast an atom ref to an AtomIdent ref.
    #[inline]
    pub fn cast<'a>(atom: &'a string_cache::Atom<Set>) -> &'a Self {
        let ptr = atom as *const _ as *const Self;
        // safety: repr(transparent)
        unsafe { &*ptr }
    }
}

/// A CSS `<ident>` stored as an `Atom`.
#[cfg(feature = "gecko")]
#[repr(transparent)]
#[derive(
    Clone, Debug, Default, Deref, Eq, Hash, MallocSizeOf, PartialEq, SpecifiedValueInfo, ToShmem,
)]
pub struct AtomIdent(pub Atom);

#[cfg(feature = "gecko")]
impl cssparser::ToCss for AtomIdent {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
    where
        W: Write,
    {
        serialize_atom_identifier(&self.0, dest)
    }
}

#[cfg(feature = "gecko")]
impl PrecomputedHash for AtomIdent {
    #[inline]
    fn precomputed_hash(&self) -> u32 {
        self.0.precomputed_hash()
    }
}

#[cfg(feature = "gecko")]
impl<'a> From<&'a str> for AtomIdent {
    #[inline]
    fn from(string: &str) -> Self {
        Self(Atom::from(string))
    }
}

#[cfg(feature = "gecko")]
impl AtomIdent {
    /// Constructs a new AtomIdent.
    #[inline]
    pub fn new(atom: Atom) -> Self {
        Self(atom)
    }

    /// Like `Atom::with` but for `AtomIdent`.
    pub unsafe fn with<F, R>(ptr: *const crate::gecko_bindings::structs::nsAtom, callback: F) -> R
    where
        F: FnOnce(&Self) -> R,
    {
        Atom::with(ptr, |atom: &Atom| {
            // safety: repr(transparent)
            let atom = atom as *const Atom as *const AtomIdent;
            callback(&*atom)
        })
    }
}

#[cfg(feature = "gecko")]
impl std::borrow::Borrow<crate::gecko_string_cache::WeakAtom> for AtomIdent {
    #[inline]
    fn borrow(&self) -> &crate::gecko_string_cache::WeakAtom {
        self.0.borrow()
    }
}

/// Serialize a normalized value into percentage.
pub fn serialize_percentage<W>(value: CSSFloat, dest: &mut CssWriter<W>) -> fmt::Result
where
    W: Write,
{
    (value * 100.).to_css(dest)?;
    dest.write_str("%")
}

/// Convenience void type to disable some properties and values through types.
#[cfg_attr(feature = "servo", derive(Deserialize, MallocSizeOf, Serialize))]
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    SpecifiedValueInfo,
    ToAnimatedValue,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
)]
pub enum Impossible {}

// FIXME(nox): This should be derived but the derive code cannot cope
// with uninhabited enums.
impl ComputeSquaredDistance for Impossible {
    #[inline]
    fn compute_squared_distance(&self, _other: &Self) -> Result<SquaredDistance, ()> {
        match *self {}
    }
}

impl_trivial_to_shmem!(Impossible);

impl Parse for Impossible {
    fn parse<'i, 't>(
        _context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
    }
}

/// A struct representing one of two kinds of values.
#[derive(
    Animate,
    Clone,
    ComputeSquaredDistance,
    Copy,
    MallocSizeOf,
    PartialEq,
    Parse,
    SpecifiedValueInfo,
    ToAnimatedValue,
    ToAnimatedZero,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
pub enum Either<A, B> {
    /// The first value.
    First(A),
    /// The second kind of value.
    Second(B),
}

impl<A: Debug, B: Debug> Debug for Either<A, B> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Either::First(ref v) => v.fmt(f),
            Either::Second(ref v) => v.fmt(f),
        }
    }
}

/// <https://drafts.csswg.org/css-values-4/#custom-idents>
#[derive(
    Clone,
    Debug,
    Eq,
    Hash,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToComputedValue,
    ToResolvedValue,
    ToShmem,
)]
#[repr(C)]
pub struct CustomIdent(pub Atom);

impl CustomIdent {
    /// Parse an already-tokenizer identifier
    pub fn from_ident<'i>(
        location: SourceLocation,
        ident: &CowRcStr<'i>,
        excluding: &[&str],
    ) -> Result<Self, ParseError<'i>> {
        let valid = match_ignore_ascii_case! { ident,
            "initial" | "inherit" | "unset" | "default" | "revert" => false,
            _ => true
        };
        if !valid {
            return Err(
                location.new_custom_error(SelectorParseErrorKind::UnexpectedIdent(ident.clone()))
            );
        }
        if excluding.iter().any(|s| ident.eq_ignore_ascii_case(s)) {
            Err(location.new_custom_error(StyleParseErrorKind::UnspecifiedError))
        } else {
            Ok(CustomIdent(Atom::from(ident.as_ref())))
        }
    }
}

impl ToCss for CustomIdent {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        serialize_atom_identifier(&self.0, dest)
    }
}

/// <https://drafts.csswg.org/css-animations/#typedef-keyframes-name>
#[derive(
    Clone, Debug, MallocSizeOf, SpecifiedValueInfo, ToComputedValue, ToResolvedValue, ToShmem,
)]
pub enum KeyframesName {
    /// <custom-ident>
    Ident(CustomIdent),
    /// <string>
    QuotedString(Atom),
}

impl KeyframesName {
    /// <https://drafts.csswg.org/css-animations/#dom-csskeyframesrule-name>
    pub fn from_ident(value: &str) -> Self {
        let location = SourceLocation { line: 0, column: 0 };
        let custom_ident = CustomIdent::from_ident(location, &value.into(), &["none"]).ok();
        match custom_ident {
            Some(ident) => KeyframesName::Ident(ident),
            None => KeyframesName::QuotedString(value.into()),
        }
    }

    /// Create a new KeyframesName from Atom.
    #[cfg(feature = "gecko")]
    pub fn from_atom(atom: Atom) -> Self {
        debug_assert_ne!(atom, atom!(""));

        // FIXME: We might want to preserve <string>, but currently Gecko
        // stores both of <custom-ident> and <string> into nsAtom, so
        // we can't tell it.
        KeyframesName::Ident(CustomIdent(atom))
    }

    /// The name as an Atom
    pub fn as_atom(&self) -> &Atom {
        match *self {
            KeyframesName::Ident(ref ident) => &ident.0,
            KeyframesName::QuotedString(ref atom) => atom,
        }
    }
}

impl Eq for KeyframesName {}

/// A trait that returns whether a given type is the `auto` value or not. So far
/// only needed for background-size serialization, which special-cases `auto`.
pub trait IsAuto {
    /// Returns whether the value is the `auto` value.
    fn is_auto(&self) -> bool;
}

impl PartialEq for KeyframesName {
    fn eq(&self, other: &Self) -> bool {
        self.as_atom() == other.as_atom()
    }
}

impl hash::Hash for KeyframesName {
    fn hash<H>(&self, state: &mut H)
    where
        H: hash::Hasher,
    {
        self.as_atom().hash(state)
    }
}

impl Parse for KeyframesName {
    fn parse<'i, 't>(
        _context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        let location = input.current_source_location();
        match *input.next()? {
            Token::Ident(ref s) => Ok(KeyframesName::Ident(CustomIdent::from_ident(
                location,
                s,
                &["none"],
            )?)),
            Token::QuotedString(ref s) => Ok(KeyframesName::QuotedString(Atom::from(s.as_ref()))),
            ref t => Err(location.new_unexpected_token_error(t.clone())),
        }
    }
}

impl ToCss for KeyframesName {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        match *self {
            KeyframesName::Ident(ref ident) => ident.to_css(dest),
            KeyframesName::QuotedString(ref atom) => atom.to_string().to_css(dest),
        }
    }
}
