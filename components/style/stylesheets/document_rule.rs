/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! [@document rules](https://www.w3.org/TR/2012/WD-css3-conditional-20120911/#at-document)
//! initially in CSS Conditional Rules Module Level 3, @document has been postponed to the level 4.
//! We implement the prefixed `@-moz-document`.

use crate::media_queries::Device;
use crate::parser::{Parse, ParserContext};
use crate::shared_lock::{DeepCloneParams, DeepCloneWithLock, Locked};
use crate::shared_lock::{SharedRwLock, SharedRwLockReadGuard, ToCssWithGuard};
use crate::str::CssStringWriter;
use crate::stylesheets::CssRules;
use crate::values::CssUrl;
use cssparser::{Parser, SourceLocation};
#[cfg(feature = "gecko")]
use malloc_size_of::{MallocSizeOfOps, MallocUnconditionalShallowSizeOf};
use servo_arc::Arc;
use std::fmt::{self, Write};
use style_traits::{CssWriter, ParseError, StyleParseErrorKind, ToCss};

#[derive(Debug, ToShmem)]
/// A @-moz-document rule
pub struct DocumentRule {
    /// The parsed condition
    pub condition: DocumentCondition,
    /// Child rules
    pub rules: Arc<Locked<CssRules>>,
    /// The line and column of the rule's source code.
    pub source_location: SourceLocation,
}

impl DocumentRule {
    /// Measure heap usage.
    #[cfg(feature = "gecko")]
    pub fn size_of(&self, guard: &SharedRwLockReadGuard, ops: &mut MallocSizeOfOps) -> usize {
        // Measurement of other fields may be added later.
        self.rules.unconditional_shallow_size_of(ops) +
            self.rules.read_with(guard).size_of(guard, ops)
    }
}

impl ToCssWithGuard for DocumentRule {
    fn to_css(&self, guard: &SharedRwLockReadGuard, dest: &mut CssStringWriter) -> fmt::Result {
        dest.write_str("@-moz-document ")?;
        self.condition.to_css(&mut CssWriter::new(dest))?;
        dest.write_str(" {")?;
        for rule in self.rules.read_with(guard).0.iter() {
            dest.write_str(" ")?;
            rule.to_css(guard, dest)?;
        }
        dest.write_str(" }")
    }
}

impl DeepCloneWithLock for DocumentRule {
    /// Deep clones this DocumentRule.
    fn deep_clone_with_lock(
        &self,
        lock: &SharedRwLock,
        guard: &SharedRwLockReadGuard,
        params: &DeepCloneParams,
    ) -> Self {
        let rules = self.rules.read_with(guard);
        DocumentRule {
            condition: self.condition.clone(),
            rules: Arc::new(lock.wrap(rules.deep_clone_with_lock(lock, guard, params))),
            source_location: self.source_location.clone(),
        }
    }
}

/// The kind of media document that the rule will match.
#[derive(Clone, Copy, Debug, Parse, PartialEq, ToCss, ToShmem)]
#[allow(missing_docs)]
pub enum MediaDocumentKind {
    All,
    Plugin,
    Image,
    Video,
}

/// A matching function for a `@document` rule's condition.
#[derive(Clone, Debug, ToCss, ToShmem)]
pub enum DocumentMatchingFunction {
    /// Exact URL matching function. It evaluates to true whenever the
    /// URL of the document being styled is exactly the URL given.
    Url(CssUrl),
    /// URL prefix matching function. It evaluates to true whenever the
    /// URL of the document being styled has the argument to the
    /// function as an initial substring (which is true when the two
    /// strings are equal). When the argument is the empty string,
    /// it evaluates to true for all documents.
    #[css(function)]
    UrlPrefix(String),
    /// Domain matching function. It evaluates to true whenever the URL
    /// of the document being styled has a host subcomponent and that
    /// host subcomponent is exactly the argument to the ‘domain()’
    /// function or a final substring of the host component is a
    /// period (U+002E) immediately followed by the argument to the
    /// ‘domain()’ function.
    #[css(function)]
    Domain(String),
    /// Regular expression matching function. It evaluates to true
    /// whenever the regular expression matches the entirety of the URL
    /// of the document being styled.
    #[css(function)]
    Regexp(String),
    /// Matching function for a media document.
    #[css(function)]
    MediaDocument(MediaDocumentKind),
}

macro_rules! parse_quoted_or_unquoted_string {
    ($input:ident, $url_matching_function:expr) => {
        $input.parse_nested_block(|input| {
            let start = input.position();
            input
                .parse_entirely(|input| {
                    let string = input.expect_string()?;
                    Ok($url_matching_function(string.as_ref().to_owned()))
                })
                .or_else(|_: ParseError| {
                    while let Ok(_) = input.next() {}
                    Ok($url_matching_function(input.slice_from(start).to_string()))
                })
        })
    };
}

impl DocumentMatchingFunction {
    /// Parse a URL matching function for a`@document` rule's condition.
    pub fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        if let Ok(url) = input.try(|input| CssUrl::parse(context, input)) {
            return Ok(DocumentMatchingFunction::Url(url));
        }

        let location = input.current_source_location();
        let function = input.expect_function()?.clone();
        match_ignore_ascii_case! { &function,
            "url-prefix" => {
                parse_quoted_or_unquoted_string!(input, DocumentMatchingFunction::UrlPrefix)
            }
            "domain" => {
                parse_quoted_or_unquoted_string!(input, DocumentMatchingFunction::Domain)
            }
            "regexp" => {
                input.parse_nested_block(|input| {
                    Ok(DocumentMatchingFunction::Regexp(
                        input.expect_string()?.as_ref().to_owned(),
                    ))
                })
            }
            "media-document" => {
                input.parse_nested_block(|input| {
                    let kind = MediaDocumentKind::parse(input)?;
                    Ok(DocumentMatchingFunction::MediaDocument(kind))
                })
            }
            _ => {
                Err(location.new_custom_error(
                    StyleParseErrorKind::UnexpectedFunction(function.clone())
                ))
            }
        }
    }

    #[cfg(feature = "gecko")]
    /// Evaluate a URL matching function.
    pub fn evaluate(&self, device: &Device) -> bool {
        use crate::gecko_bindings::bindings::Gecko_DocumentRule_UseForPresentation;
        use crate::gecko_bindings::structs::DocumentMatchingFunction as GeckoDocumentMatchingFunction;
        use nsstring::nsCStr;

        let func = match *self {
            DocumentMatchingFunction::Url(_) => GeckoDocumentMatchingFunction::URL,
            DocumentMatchingFunction::UrlPrefix(_) => GeckoDocumentMatchingFunction::URLPrefix,
            DocumentMatchingFunction::Domain(_) => GeckoDocumentMatchingFunction::Domain,
            DocumentMatchingFunction::Regexp(_) => GeckoDocumentMatchingFunction::RegExp,
            DocumentMatchingFunction::MediaDocument(_) => {
                GeckoDocumentMatchingFunction::MediaDocument
            },
        };

        let pattern = nsCStr::from(match *self {
            DocumentMatchingFunction::Url(ref url) => url.as_str(),
            DocumentMatchingFunction::UrlPrefix(ref pat) |
            DocumentMatchingFunction::Domain(ref pat) |
            DocumentMatchingFunction::Regexp(ref pat) => pat,
            DocumentMatchingFunction::MediaDocument(kind) => match kind {
                MediaDocumentKind::All => "all",
                MediaDocumentKind::Image => "image",
                MediaDocumentKind::Plugin => "plugin",
                MediaDocumentKind::Video => "video",
            },
        });
        unsafe { Gecko_DocumentRule_UseForPresentation(device.document(), &*pattern, func) }
    }

    #[cfg(not(feature = "gecko"))]
    /// Evaluate a URL matching function.
    pub fn evaluate(&self, _: &Device) -> bool {
        false
    }
}

/// A `@document` rule's condition.
///
/// <https://www.w3.org/TR/2012/WD-css3-conditional-20120911/#at-document>
///
/// The `@document` rule's condition is written as a comma-separated list of
/// URL matching functions, and the condition evaluates to true whenever any
/// one of those functions evaluates to true.
#[css(comma)]
#[derive(Clone, Debug, ToCss, ToShmem)]
pub struct DocumentCondition(#[css(iterable)] Vec<DocumentMatchingFunction>);

impl DocumentCondition {
    /// Parse a document condition.
    pub fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        let conditions =
            input.parse_comma_separated(|input| DocumentMatchingFunction::parse(context, input))?;

        let condition = DocumentCondition(conditions);
        if !condition.allowed_in(context) {
            return Err(
                input.new_custom_error(StyleParseErrorKind::UnsupportedAtRule(
                    "-moz-document".into(),
                )),
            );
        }
        Ok(condition)
    }

    /// Evaluate a document condition.
    pub fn evaluate(&self, device: &Device) -> bool {
        self.0
            .iter()
            .any(|url_matching_function| url_matching_function.evaluate(device))
    }

    #[cfg(feature = "servo")]
    fn allowed_in(&self, _: &ParserContext) -> bool {
        false
    }

    #[cfg(feature = "gecko")]
    fn allowed_in(&self, context: &ParserContext) -> bool {
        use crate::gecko_bindings::structs;
        use crate::stylesheets::Origin;

        if context.stylesheet_origin != Origin::Author {
            return true;
        }

        if unsafe { structs::StaticPrefs_sVarCache_layout_css_moz_document_content_enabled } {
            return true;
        }

        if !unsafe {
            structs::StaticPrefs_sVarCache_layout_css_moz_document_url_prefix_hack_enabled
        } {
            return false;
        }

        // Allow a single url-prefix() for compatibility.
        //
        // See bug 1446470 and dependencies.
        if self.0.len() != 1 {
            return false;
        }

        // NOTE(emilio): This technically allows url-prefix("") too, but...
        match self.0[0] {
            DocumentMatchingFunction::UrlPrefix(ref prefix) => prefix.is_empty(),
            _ => false,
        }
    }
}
