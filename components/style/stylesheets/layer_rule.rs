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

/// A `<layer-name>`: https://drafts.csswg.org/css-cascade-5/#typedef-layer-name
#[derive(Clone, Debug, ToShmem)]
pub struct LayerName(SmallVec<[AtomIdent; 1]>);

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
        /// The layer name, or `None` if anonymous.
        name: Option<LayerName>,
        /// The nested rules.
        rules: Arc<Locked<CssRules>>,
    },
    /// A statement `@layer <name>, <name>, <name>;`
    Statement {
        /// The list of layers to sort.
        names: SmallVec<[LayerName; 3]>,
    },
}

/// A [`@layer`][layer] urle.
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
            } => {
                if let Some(ref name) = *name {
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
                } => LayerRuleKind::Block {
                    name: name.clone(),
                    rules: Arc::new(
                        lock.wrap(
                            rules
                                .read_with(guard)
                                .deep_clone_with_lock(lock, guard, params),
                        ),
                    ),
                },
                LayerRuleKind::Statement { ref names } => LayerRuleKind::Statement {
                    names: names.clone(),
                },
            },
            source_location: self.source_location.clone(),
        }
    }
}
