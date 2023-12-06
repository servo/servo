/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! A [`@layer`][layer] rule.
//!
//! [layer]: https://drafts.csswg.org/css-cascade-5/#layering

use crate::parser::{Parse, ParserContext};
use crate::shared_lock::{DeepCloneParams, DeepCloneWithLock, Locked};
use crate::shared_lock::{SharedRwLock, SharedRwLockReadGuard, ToCssWithGuard};
use crate::values::AtomIdent;

use super::CssRules;

use cssparser::{Parser, SourceLocation, Token};
use servo_arc::Arc;
use smallvec::SmallVec;
use std::fmt::{self, Write};
use style_traits::{CssWriter, ParseError, ToCss};

/// The order of a given layer. We use 16 bits so that we can pack LayerOrder
/// and CascadeLevel in a single 32-bit struct. If we need more bits we can go
/// back to packing CascadeLevel in a single byte as we did before.
#[derive(Clone, Copy, Debug, Eq, Hash, MallocSizeOf, PartialEq, PartialOrd, Ord)]
pub struct LayerOrder(u16);

impl LayerOrder {
    /// The order of the root layer.
    pub const fn root() -> Self {
        Self(std::u16::MAX - 1)
    }

    /// The order of the style attribute layer.
    pub const fn style_attribute() -> Self {
        Self(std::u16::MAX)
    }

    /// Returns whether this layer is for the style attribute, which behaves
    /// differently in terms of !important, see
    /// https://github.com/w3c/csswg-drafts/issues/6872
    ///
    /// (This is a bit silly, mind-you, but it's needed so that revert-layer
    /// behaves correctly).
    #[inline]
    pub fn is_style_attribute_layer(&self) -> bool {
        *self == Self::style_attribute()
    }

    /// The first cascade layer order.
    pub const fn first() -> Self {
        Self(0)
    }

    /// Increment the cascade layer order.
    #[inline]
    pub fn inc(&mut self) {
        if self.0 != std::u16::MAX - 1 {
            self.0 += 1;
        }
    }
}

/// A `<layer-name>`: https://drafts.csswg.org/css-cascade-5/#typedef-layer-name
#[derive(Clone, Debug, Eq, Hash, MallocSizeOf, PartialEq, ToShmem)]
pub struct LayerName(pub SmallVec<[AtomIdent; 1]>);

impl LayerName {
    /// Returns an empty layer name (which isn't a valid final state, so caller
    /// is responsible to fill up the name before use).
    pub fn new_empty() -> Self {
        Self(Default::default())
    }

    /// Returns a synthesized name for an anonymous layer.
    pub fn new_anonymous() -> Self {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static NEXT_ANONYMOUS_LAYER_NAME: AtomicUsize = AtomicUsize::new(0);

        let mut name = SmallVec::new();
        let next_id = NEXT_ANONYMOUS_LAYER_NAME.fetch_add(1, Ordering::Relaxed);
        // The parens don't _technically_ prevent conflicts with authors, as
        // authors could write escaped parens as part of the identifier, I
        // think, but highly reduces the possibility.
        name.push(AtomIdent::from(&*format!("-moz-anon-layer({})", next_id)));

        LayerName(name)
    }

    /// Returns the names of the layers. That is, for a layer like `foo.bar`,
    /// it'd return [foo, bar].
    pub fn layer_names(&self) -> &[AtomIdent] {
        &self.0
    }
}

impl Parse for LayerName {
    fn parse<'i, 't>(
        _: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        let mut result = SmallVec::new();
        result.push(AtomIdent::from(&**input.expect_ident()?));
        loop {
            let next_name = input.try_parse(|input| -> Result<AtomIdent, ParseError<'i>> {
                match input.next_including_whitespace()? {
                    Token::Delim('.') => {},
                    other => {
                        let t = other.clone();
                        return Err(input.new_unexpected_token_error(t));
                    },
                }

                let name = match input.next_including_whitespace()? {
                    Token::Ident(ref ident) => ident,
                    other => {
                        let t = other.clone();
                        return Err(input.new_unexpected_token_error(t));
                    },
                };

                Ok(AtomIdent::from(&**name))
            });

            match next_name {
                Ok(name) => result.push(name),
                Err(..) => break,
            }
        }
        Ok(LayerName(result))
    }
}

impl ToCss for LayerName {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        let mut first = true;
        for name in self.0.iter() {
            if !first {
                dest.write_char('.')?;
            }
            first = false;
            name.to_css(dest)?;
        }
        Ok(())
    }
}

#[derive(Debug, ToShmem)]
/// A block `@layer <name>? { ... }`
/// https://drafts.csswg.org/css-cascade-5/#layer-block
pub struct LayerBlockRule {
    /// The layer name, or `None` if anonymous.
    pub name: Option<LayerName>,
    /// The nested rules.
    pub rules: Arc<Locked<CssRules>>,
    /// The source position where this rule was found.
    pub source_location: SourceLocation,
}

impl ToCssWithGuard for LayerBlockRule {
    fn to_css(
        &self,
        guard: &SharedRwLockReadGuard,
        dest: &mut crate::str::CssStringWriter,
    ) -> fmt::Result {
        dest.write_str("@layer")?;
        if let Some(ref name) = self.name {
            dest.write_char(' ')?;
            name.to_css(&mut CssWriter::new(dest))?;
        }
        self.rules.read_with(guard).to_css_block(guard, dest)
    }
}

impl DeepCloneWithLock for LayerBlockRule {
    fn deep_clone_with_lock(
        &self,
        lock: &SharedRwLock,
        guard: &SharedRwLockReadGuard,
        params: &DeepCloneParams,
    ) -> Self {
        Self {
            name: self.name.clone(),
            rules: Arc::new(
                lock.wrap(
                    self.rules
                        .read_with(guard)
                        .deep_clone_with_lock(lock, guard, params),
                ),
            ),
            source_location: self.source_location.clone(),
        }
    }
}

/// A statement `@layer <name>, <name>, <name>;`
///
/// https://drafts.csswg.org/css-cascade-5/#layer-empty
#[derive(Clone, Debug, ToShmem)]
pub struct LayerStatementRule {
    /// The list of layers to sort.
    pub names: Vec<LayerName>,
    /// The source position where this rule was found.
    pub source_location: SourceLocation,
}

impl ToCssWithGuard for LayerStatementRule {
    fn to_css(
        &self,
        _: &SharedRwLockReadGuard,
        dest: &mut crate::str::CssStringWriter,
    ) -> fmt::Result {
        let mut writer = CssWriter::new(dest);
        writer.write_str("@layer ")?;
        let mut first = true;
        for name in &*self.names {
            if !first {
                writer.write_str(", ")?;
            }
            first = false;
            name.to_css(&mut writer)?;
        }
        writer.write_char(';')
    }
}
