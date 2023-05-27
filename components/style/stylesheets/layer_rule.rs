/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! A [`@layer`][layer] urle.
//!
//! [layer]: https://drafts.csswg.org/css-cascade-5/#layering

use crate::parser::{Parse, ParserContext};
use crate::shared_lock::{DeepCloneParams, DeepCloneWithLock, Locked};
use crate::shared_lock::{SharedRwLock, SharedRwLockReadGuard, ToCssWithGuard};
use crate::values::AtomIdent;

use super::CssRules;

use cssparser::{Parser, SourceLocation, ToCss as CssParserToCss, Token};
use servo_arc::Arc;
use smallvec::SmallVec;
use std::fmt::{self, Write};
use style_traits::{CssWriter, ParseError, ToCss};

/// The order of a given layer. We encode in a 32-bit integer as follows:
///
///  * 0 is reserved for the initial (top-level) layer.
///  * Top 7 bits are for top level layer order.
///  * The 25 remaining bits are split in 5 chunks of 5 bits each, for each
///    nesting level.
///
/// This scheme this gives up to 127 layers in the top level, and up to 31
/// children layers in nested levels, with a max of 6 nesting levels over all.
///
/// This seemingly complicated scheme is to avoid fixing up layer orders after
/// the cascade data rebuild.
///
/// An alternative approach that would allow improving those limits would be to
/// make layers have a sequential identifier, and sort layer order after the
/// fact. But that complicates incremental cascade data rebuild.
#[derive(Clone, Copy, Debug, Eq, MallocSizeOf, PartialEq, PartialOrd, Ord)]
pub struct LayerOrder(u32);

impl LayerOrder {
    const FIRST_LEVEL_BITS: usize = 7;
    const CHILD_BITS: usize = 5;
    const FIRST_LEVEL_MASK: u32 = 0b11111110_00000000_00000000_00000000;
    const CHILD_MASK: u32 = 0b00011111;

    /// Get the raw value.
    pub fn raw(self) -> u32 {
        self.0
    }

    /// The top level layer (implicit) is zero.
    #[inline]
    pub const fn top_level() -> Self {
        Self(0)
    }

    /// The first layer order.
    #[inline]
    pub const fn first() -> Self {
        Self(1 << (32 - Self::FIRST_LEVEL_BITS))
    }

    fn child_bit_offset(self) -> usize {
        if self.0 & (Self::CHILD_MASK << 5) != 0 {
            return 0; // We're at the last or next-to-last level.
        }
        if self.0 & (Self::CHILD_MASK << 10) != 0 {
            return 5;
        }
        if self.0 & (Self::CHILD_MASK << 15) != 0 {
            return 10;
        }
        if self.0 & (Self::CHILD_MASK << 20) != 0 {
            return 15;
        }
        if self.0 != 0 {
            return 20;
        }
        return 25;
    }

    fn sibling_bit_mask_max_and_offset(self) -> (u32, u32, u32) {
        debug_assert_ne!(self.0, 0, "Top layer should have no siblings");
        for offset in &[0, 5, 10, 15, 20] {
            let mask = Self::CHILD_MASK << *offset;
            if self.0 & mask != 0 {
                return (mask, (1 << Self::CHILD_BITS) - 1, *offset);
            }
        }
        return (Self::FIRST_LEVEL_MASK, (1 << Self::FIRST_LEVEL_BITS) - 1, 25);
    }

    /// Generate the layer order for our first child.
    pub fn for_child(self) -> Self {
        Self(self.0 | (1 << self.child_bit_offset()))
    }

    /// Generate the layer order for our next sibling. Might return the same
    /// order when our limits overflow.
    pub fn for_next_sibling(self) -> Self {
        let (mask, max_index, offset) = self.sibling_bit_mask_max_and_offset();
        let self_index = (self.0 & mask) >> offset;
        let next_index = if self_index == max_index {
            self_index
        } else {
            self_index + 1
        };
        Self((self.0 & !mask) | (next_index << offset))
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

/// The kind of layer rule this is.
#[derive(Debug, ToShmem)]
pub enum LayerRuleKind {
    /// A block `@layer <name>? { ... }`
    Block {
        /// The layer name.
        name: LayerName,
        /// The nested rules.
        rules: Arc<Locked<CssRules>>,
        /// Whether the layer name is synthesized (and thus shouldn't be
        /// serialized).
        is_anonymous: bool,
    },
    /// A statement `@layer <name>, <name>, <name>;`
    Statement {
        /// The list of layers to sort.
        names: Vec<LayerName>,
    },
}

/// A [`@layer`][layer] rule.
///
/// [layer]: https://drafts.csswg.org/css-cascade-5/#layering
#[derive(Debug, ToShmem)]
pub struct LayerRule {
    /// The kind of layer rule we are.
    pub kind: LayerRuleKind,
    /// The source position where this media rule was found.
    pub source_location: SourceLocation,
}

impl ToCssWithGuard for LayerRule {
    fn to_css(
        &self,
        guard: &SharedRwLockReadGuard,
        dest: &mut crate::str::CssStringWriter,
    ) -> fmt::Result {
        dest.write_str("@layer ")?;
        match self.kind {
            LayerRuleKind::Block {
                ref name,
                ref rules,
                ref is_anonymous,
            } => {
                if !*is_anonymous {
                    name.to_css(&mut CssWriter::new(dest))?;
                    dest.write_char(' ')?;
                }
                rules.read_with(guard).to_css_block(guard, dest)
            },
            LayerRuleKind::Statement { ref names } => {
                let mut writer = CssWriter::new(dest);
                let mut first = true;
                for name in &**names {
                    if !first {
                        writer.write_str(", ")?;
                    }
                    first = false;
                    name.to_css(&mut writer)?;
                }
                dest.write_char(';')
            },
        }
    }
}

impl DeepCloneWithLock for LayerRule {
    fn deep_clone_with_lock(
        &self,
        lock: &SharedRwLock,
        guard: &SharedRwLockReadGuard,
        params: &DeepCloneParams,
    ) -> Self {
        Self {
            kind: match self.kind {
                LayerRuleKind::Block {
                    ref name,
                    ref rules,
                    ref is_anonymous,
                } => LayerRuleKind::Block {
                    name: if *is_anonymous {
                        LayerName::new_anonymous()
                    } else {
                        name.clone()
                    },
                    rules: Arc::new(
                        lock.wrap(
                            rules
                                .read_with(guard)
                                .deep_clone_with_lock(lock, guard, params),
                        ),
                    ),
                    is_anonymous: *is_anonymous,
                },
                LayerRuleKind::Statement { ref names } => LayerRuleKind::Statement {
                    names: names.clone(),
                },
            },
            source_location: self.source_location.clone(),
        }
    }
}
