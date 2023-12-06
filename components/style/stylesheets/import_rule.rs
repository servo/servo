/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The [`@import`][import] at-rule.
//!
//! [import]: https://drafts.csswg.org/css-cascade-3/#at-import

use crate::media_queries::MediaList;
use crate::parser::{Parse, ParserContext};
use crate::shared_lock::{
    DeepCloneParams, DeepCloneWithLock, SharedRwLock, SharedRwLockReadGuard, ToCssWithGuard,
};
use crate::str::CssStringWriter;
use crate::stylesheets::{
    layer_rule::LayerName, supports_rule::SupportsCondition, CssRule, CssRuleType,
    StylesheetInDocument,
};
use crate::values::CssUrl;
use cssparser::{Parser, SourceLocation};
use std::fmt::{self, Write};
use style_traits::{CssWriter, ToCss};
use to_shmem::{self, SharedMemoryBuilder, ToShmem};

/// A sheet that is held from an import rule.
#[cfg(feature = "gecko")]
#[derive(Debug)]
pub enum ImportSheet {
    /// A bonafide stylesheet.
    Sheet(crate::gecko::data::GeckoStyleSheet),

    /// An @import created while parsing off-main-thread, whose Gecko sheet has
    /// yet to be created and attached.
    Pending,

    /// An @import created with a false <supports-condition>, so will never be fetched.
    Refused,
}

#[cfg(feature = "gecko")]
impl ImportSheet {
    /// Creates a new ImportSheet from a GeckoStyleSheet.
    pub fn new(sheet: crate::gecko::data::GeckoStyleSheet) -> Self {
        ImportSheet::Sheet(sheet)
    }

    /// Creates a pending ImportSheet for a load that has not started yet.
    pub fn new_pending() -> Self {
        ImportSheet::Pending
    }

    /// Creates a refused ImportSheet for a load that will not happen.
    pub fn new_refused() -> Self {
        ImportSheet::Refused
    }

    /// Returns a reference to the GeckoStyleSheet in this ImportSheet, if it
    /// exists.
    pub fn as_sheet(&self) -> Option<&crate::gecko::data::GeckoStyleSheet> {
        match *self {
            ImportSheet::Sheet(ref s) => {
                debug_assert!(!s.hack_is_null());
                if s.hack_is_null() {
                    return None;
                }
                Some(s)
            },
            ImportSheet::Refused | ImportSheet::Pending => None,
        }
    }

    /// Returns the media list for this import rule.
    pub fn media<'a>(&'a self, guard: &'a SharedRwLockReadGuard) -> Option<&'a MediaList> {
        self.as_sheet().and_then(|s| s.media(guard))
    }

    /// Returns the rule list for this import rule.
    pub fn rules<'a>(&'a self, guard: &'a SharedRwLockReadGuard) -> &'a [CssRule] {
        match self.as_sheet() {
            Some(s) => s.rules(guard),
            None => &[],
        }
    }
}

#[cfg(feature = "gecko")]
impl DeepCloneWithLock for ImportSheet {
    fn deep_clone_with_lock(
        &self,
        _lock: &SharedRwLock,
        _guard: &SharedRwLockReadGuard,
        params: &DeepCloneParams,
    ) -> Self {
        use crate::gecko::data::GeckoStyleSheet;
        use crate::gecko_bindings::bindings;
        match *self {
            ImportSheet::Sheet(ref s) => {
                let clone = unsafe {
                    bindings::Gecko_StyleSheet_Clone(s.raw() as *const _, params.reference_sheet)
                };
                ImportSheet::Sheet(unsafe { GeckoStyleSheet::from_addrefed(clone) })
            },
            ImportSheet::Pending => ImportSheet::Pending,
            ImportSheet::Refused => ImportSheet::Refused,
        }
    }
}

/// A sheet that is held from an import rule.
#[cfg(feature = "servo")]
#[derive(Debug)]
pub enum ImportSheet {
    /// A bonafide stylesheet.
    Sheet(::servo_arc::Arc<crate::stylesheets::Stylesheet>),

    /// An @import created with a false <supports-condition>, so will never be fetched.
    Refused,
}

#[cfg(feature = "servo")]
impl ImportSheet {
    /// Creates a new ImportSheet from a stylesheet.
    pub fn new(sheet: ::servo_arc::Arc<crate::stylesheets::Stylesheet>) -> Self {
        ImportSheet::Sheet(sheet)
    }

    /// Creates a refused ImportSheet for a load that will not happen.
    pub fn new_refused() -> Self {
        ImportSheet::Refused
    }

    /// Returns a reference to the stylesheet in this ImportSheet, if it exists.
    pub fn as_sheet(&self) -> Option<&::servo_arc::Arc<crate::stylesheets::Stylesheet>> {
        match *self {
            ImportSheet::Sheet(ref s) => Some(s),
            ImportSheet::Refused => None,
        }
    }

    /// Returns the media list for this import rule.
    pub fn media<'a>(&'a self, guard: &'a SharedRwLockReadGuard) -> Option<&'a MediaList> {
        self.as_sheet().and_then(|s| s.media(guard))
    }

    /// Returns the rules for this import rule.
    pub fn rules<'a>(&'a self, guard: &'a SharedRwLockReadGuard) -> &'a [CssRule] {
        match self.as_sheet() {
            Some(s) => s.rules(guard),
            None => &[],
        }
    }
}

#[cfg(feature = "servo")]
impl DeepCloneWithLock for ImportSheet {
    fn deep_clone_with_lock(
        &self,
        _lock: &SharedRwLock,
        _guard: &SharedRwLockReadGuard,
        _params: &DeepCloneParams,
    ) -> Self {
        match *self {
            ImportSheet::Sheet(ref s) => {
                use servo_arc::Arc;
                ImportSheet::Sheet(Arc::new((&**s).clone()))
            },
            ImportSheet::Refused => ImportSheet::Refused,
        }
    }
}

/// The layer specified in an import rule (can be none, anonymous, or named).
#[derive(Debug, Clone)]
pub enum ImportLayer {
    /// No layer specified
    None,

    /// Anonymous layer (`layer`)
    Anonymous,

    /// Named layer (`layer(name)`)
    Named(LayerName),
}

/// The supports condition in an import rule.
#[derive(Debug, Clone)]
pub struct ImportSupportsCondition {
    /// The supports condition.
    pub condition: SupportsCondition,

    /// If the import is enabled, from the result of the import condition.
    pub enabled: bool,
}

impl ToCss for ImportLayer {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        match *self {
            ImportLayer::None => Ok(()),
            ImportLayer::Anonymous => dest.write_str("layer"),
            ImportLayer::Named(ref name) => {
                dest.write_str("layer(")?;
                name.to_css(dest)?;
                dest.write_char(')')
            },
        }
    }
}

/// The [`@import`][import] at-rule.
///
/// [import]: https://drafts.csswg.org/css-cascade-3/#at-import
#[derive(Debug)]
pub struct ImportRule {
    /// The `<url>` this `@import` rule is loading.
    pub url: CssUrl,

    /// The stylesheet is always present. However, in the case of gecko async
    /// parsing, we don't actually have a Gecko sheet at first, and so the
    /// ImportSheet just has stub behavior until it appears.
    pub stylesheet: ImportSheet,

    /// A <supports-condition> for the rule.
    pub supports: Option<ImportSupportsCondition>,

    /// A `layer()` function name.
    pub layer: ImportLayer,

    /// The line and column of the rule's source code.
    pub source_location: SourceLocation,
}

impl ImportRule {
    /// Parses the layer() / layer / supports() part of the import header, as per
    /// https://drafts.csswg.org/css-cascade-5/#at-import:
    ///
    ///     [ layer | layer(<layer-name>) ]?
    ///     [ supports([ <supports-condition> | <declaration> ]) ]?
    ///
    /// We do this here so that the import preloader can look at this without having to parse the
    /// whole import rule or parse the media query list or what not.
    pub fn parse_layer_and_supports<'i, 't>(
        input: &mut Parser<'i, 't>,
        context: &mut ParserContext,
    ) -> (ImportLayer, Option<ImportSupportsCondition>) {
        let layer = if input
            .try_parse(|input| input.expect_ident_matching("layer"))
            .is_ok()
        {
            ImportLayer::Anonymous
        } else {
            input
                .try_parse(|input| {
                    input.expect_function_matching("layer")?;
                    input
                        .parse_nested_block(|input| LayerName::parse(context, input))
                        .map(|name| ImportLayer::Named(name))
                })
                .ok()
                .unwrap_or(ImportLayer::None)
        };

        #[cfg(feature = "gecko")]
        let supports_enabled = static_prefs::pref!("layout.css.import-supports.enabled");
        #[cfg(feature = "servo")]
        let supports_enabled = false;

        let supports = if !supports_enabled {
            None
        } else {
            input
                .try_parse(SupportsCondition::parse_for_import)
                .map(|condition| {
                    let enabled = context
                        .nest_for_rule(CssRuleType::Style, |context| condition.eval(context));
                    ImportSupportsCondition { condition, enabled }
                })
                .ok()
        };

        (layer, supports)
    }
}

impl ToShmem for ImportRule {
    fn to_shmem(&self, _builder: &mut SharedMemoryBuilder) -> to_shmem::Result<Self> {
        Err(String::from(
            "ToShmem failed for ImportRule: cannot handle imported style sheets",
        ))
    }
}

impl DeepCloneWithLock for ImportRule {
    fn deep_clone_with_lock(
        &self,
        lock: &SharedRwLock,
        guard: &SharedRwLockReadGuard,
        params: &DeepCloneParams,
    ) -> Self {
        ImportRule {
            url: self.url.clone(),
            stylesheet: self.stylesheet.deep_clone_with_lock(lock, guard, params),
            supports: self.supports.clone(),
            layer: self.layer.clone(),
            source_location: self.source_location.clone(),
        }
    }
}

impl ToCssWithGuard for ImportRule {
    fn to_css(&self, guard: &SharedRwLockReadGuard, dest: &mut CssStringWriter) -> fmt::Result {
        dest.write_str("@import ")?;
        self.url.to_css(&mut CssWriter::new(dest))?;

        if !matches!(self.layer, ImportLayer::None) {
            dest.write_char(' ')?;
            self.layer.to_css(&mut CssWriter::new(dest))?;
        }

        if let Some(ref supports) = self.supports {
            dest.write_str(" supports(")?;
            supports.condition.to_css(&mut CssWriter::new(dest))?;
            dest.write_char(')')?;
        }

        if let Some(media) = self.stylesheet.media(guard) {
            if !media.is_empty() {
                dest.write_char(' ')?;
                media.to_css(&mut CssWriter::new(dest))?;
            }
        }

        dest.write_char(';')
    }
}
