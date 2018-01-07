/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! [@document rules](https://www.w3.org/TR/2012/WD-css3-conditional-20120911/#at-document)
//! initially in CSS Conditional Rules Module Level 3, @document has been postponed to the level 4.
//! We implement the prefixed `@-moz-document`.

use cssparser::{Parser, Token, SourceLocation};
#[cfg(feature = "gecko")]
use malloc_size_of::{MallocSizeOfOps, MallocUnconditionalShallowSizeOf};
use media_queries::Device;
use parser::{Parse, ParserContext};
use servo_arc::Arc;
use shared_lock::{DeepCloneParams, DeepCloneWithLock, Locked, SharedRwLock, SharedRwLockReadGuard, ToCssWithGuard};
use std::fmt;
use style_traits::{ToCss, ParseError, StyleParseErrorKind};
use stylesheets::CssRules;
use values::specified::url::SpecifiedUrl;

#[derive(Debug)]
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
    fn to_css<W>(&self, guard: &SharedRwLockReadGuard, dest: &mut W) -> fmt::Result
    where W: fmt::Write {
        dest.write_str("@-moz-document ")?;
        self.condition.to_css(dest)?;
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

/// A URL matching function for a `@document` rule's condition.
#[derive(Clone, Debug)]
pub enum UrlMatchingFunction {
    /// Exact URL matching function. It evaluates to true whenever the
    /// URL of the document being styled is exactly the URL given.
    Url(SpecifiedUrl),
    /// URL prefix matching function. It evaluates to true whenever the
    /// URL of the document being styled has the argument to the
    /// function as an initial substring (which is true when the two
    /// strings are equal). When the argument is the empty string,
    /// it evaluates to true for all documents.
    UrlPrefix(String),
    /// Domain matching function. It evaluates to true whenever the URL
    /// of the document being styled has a host subcomponent and that
    /// host subcomponent is exactly the argument to the ‘domain()’
    /// function or a final substring of the host component is a
    /// period (U+002E) immediately followed by the argument to the
    /// ‘domain()’ function.
    Domain(String),
    /// Regular expression matching function. It evaluates to true
    /// whenever the regular expression matches the entirety of the URL
    /// of the document being styled.
    RegExp(String),
}

macro_rules! parse_quoted_or_unquoted_string {
    ($input:ident, $url_matching_function:expr) => {
        $input.parse_nested_block(|input| {
            let start = input.position();
            input.parse_entirely(|input| {
                let location = input.current_source_location();
                match *input.next()? {
                    Token::QuotedString(ref value) => {
                        Ok($url_matching_function(value.as_ref().to_owned()))
                    },
                    ref t => Err(location.new_unexpected_token_error(t.clone())),
                }
            }).or_else(|_: ParseError| {
                while let Ok(_) = input.next() {}
                Ok($url_matching_function(input.slice_from(start).to_string()))
            })
        })
    }
}

impl UrlMatchingFunction {
    /// Parse a URL matching function for a`@document` rule's condition.
    pub fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>)
        -> Result<UrlMatchingFunction, ParseError<'i>> {
        if input.try(|input| input.expect_function_matching("url-prefix")).is_ok() {
            parse_quoted_or_unquoted_string!(input, UrlMatchingFunction::UrlPrefix)
        } else if input.try(|input| input.expect_function_matching("domain")).is_ok() {
            parse_quoted_or_unquoted_string!(input, UrlMatchingFunction::Domain)
        } else if input.try(|input| input.expect_function_matching("regexp")).is_ok() {
            input.parse_nested_block(|input| {
                Ok(UrlMatchingFunction::RegExp(input.expect_string()?.as_ref().to_owned()))
            })
        } else if let Ok(url) = input.try(|input| SpecifiedUrl::parse(context, input)) {
            Ok(UrlMatchingFunction::Url(url))
        } else {
            Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
        }
    }

    #[cfg(feature = "gecko")]
    /// Evaluate a URL matching function.
    pub fn evaluate(&self, device: &Device) -> bool {
        use gecko_bindings::bindings::Gecko_DocumentRule_UseForPresentation;
        use gecko_bindings::structs::URLMatchingFunction as GeckoUrlMatchingFunction;
        use nsstring::nsCStr;

        let func = match *self {
            UrlMatchingFunction::Url(_) => GeckoUrlMatchingFunction::eURL,
            UrlMatchingFunction::UrlPrefix(_) => GeckoUrlMatchingFunction::eURLPrefix,
            UrlMatchingFunction::Domain(_) => GeckoUrlMatchingFunction::eDomain,
            UrlMatchingFunction::RegExp(_) => GeckoUrlMatchingFunction::eRegExp,
        };

        let pattern = nsCStr::from(match *self {
            UrlMatchingFunction::Url(ref url) => url.as_str(),
            UrlMatchingFunction::UrlPrefix(ref pat) |
            UrlMatchingFunction::Domain(ref pat) |
            UrlMatchingFunction::RegExp(ref pat) => pat,
        });
        unsafe {
            Gecko_DocumentRule_UseForPresentation(device.pres_context(), &*pattern, func)
        }
    }

    #[cfg(not(feature = "gecko"))]
    /// Evaluate a URL matching function.
    pub fn evaluate(&self, _: &Device) -> bool {
        false
    }
}

impl ToCss for UrlMatchingFunction {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
        where W: fmt::Write {
        match *self {
            UrlMatchingFunction::Url(ref url) => {
                url.to_css(dest)
            },
            UrlMatchingFunction::UrlPrefix(ref url_prefix) => {
                dest.write_str("url-prefix(")?;
                url_prefix.to_css(dest)?;
                dest.write_str(")")
            },
            UrlMatchingFunction::Domain(ref domain) => {
                dest.write_str("domain(")?;
                domain.to_css(dest)?;
                dest.write_str(")")
            },
            UrlMatchingFunction::RegExp(ref regex) => {
                dest.write_str("regexp(")?;
                regex.to_css(dest)?;
                dest.write_str(")")
            },
        }
    }
}

/// A `@document` rule's condition.
///
/// <https://www.w3.org/TR/2012/WD-css3-conditional-20120911/#at-document>
///
/// The `@document` rule's condition is written as a comma-separated list of
/// URL matching functions, and the condition evaluates to true whenever any
/// one of those functions evaluates to true.
#[derive(Clone, Debug)]
pub struct DocumentCondition(Vec<UrlMatchingFunction>);

impl DocumentCondition {
    /// Parse a document condition.
    pub fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>)
        -> Result<Self, ParseError<'i>> {
        input.parse_comma_separated(|input| UrlMatchingFunction::parse(context, input))
             .map(DocumentCondition)
    }

    /// Evaluate a document condition.
    pub fn evaluate(&self, device: &Device) -> bool {
        self.0.iter().any(|ref url_matching_function|
            url_matching_function.evaluate(device)
        )
    }
}

impl ToCss for DocumentCondition {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
        where W: fmt::Write {
        let mut iter = self.0.iter();
        let first = iter.next()
            .expect("Empty DocumentCondition, should contain at least one URL matching function");
        first.to_css(dest)?;
        for url_matching_function in iter {
            dest.write_str(", ")?;
            url_matching_function.to_css(dest)?;
        }
        Ok(())
    }
}
