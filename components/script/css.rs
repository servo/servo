/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Helpers for CSS value parsing.

use style::context::QuirksMode;
use style::error_reporting::ParseErrorReporter;
use style::parser::ParserContext;
use style::stylesheets::{CssRuleType, Origin, UrlExtraData};
use style_traits::ParsingMode;

use crate::dom::document::Document;

/// Creates a `ParserContext` from the given document.
///
/// Automatically configures quirks mode and error reporter from the document.
pub(crate) fn parser_context_for_document<'a>(
    document: &'a Document,
    rule_type: CssRuleType,
    parsing_mode: ParsingMode,
    url_data: &'a UrlExtraData,
) -> ParserContext<'a> {
    let quirks_mode = document.quirks_mode();
    let error_reporter = document.window().css_error_reporter();

    ParserContext::new(
        Origin::Author,
        url_data,
        Some(rule_type),
        parsing_mode,
        quirks_mode,
        /* namespaces = */ Default::default(),
        Some(error_reporter),
        None,
    )
}

/// Like [`parser_context_for_document`], but with a custom error reporter.
pub(crate) fn parser_context_for_document_with_reporter<'a>(
    document: &'a Document,
    rule_type: CssRuleType,
    parsing_mode: ParsingMode,
    url_data: &'a UrlExtraData,
    error_reporter: &'a dyn ParseErrorReporter,
) -> ParserContext<'a> {
    let quirks_mode = document.quirks_mode();

    ParserContext::new(
        Origin::Author,
        url_data,
        Some(rule_type),
        parsing_mode,
        quirks_mode,
        /* namespaces = */ Default::default(),
        Some(error_reporter),
        None,
    )
}

/// Creates a `ParserContext` without a document, using no quirks mode
/// and no error reporter.
pub(crate) fn parser_context_for_anonymous_content<'a>(
    rule_type: CssRuleType,
    parsing_mode: ParsingMode,
    url_data: &'a UrlExtraData,
) -> ParserContext<'a> {
    ParserContext::new(
        Origin::Author,
        url_data,
        Some(rule_type),
        parsing_mode,
        QuirksMode::NoQuirks,
        /* namespaces = */ Default::default(),
        None,
        None,
    )
}
