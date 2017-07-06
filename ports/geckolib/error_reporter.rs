/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Wrapper around Gecko's CSS error reporting mechanism.

#![allow(unsafe_code)]

use cssparser::{Parser, SourcePosition};
use style::error_reporting::{ParseErrorReporter, ContextualParseError};
use style::gecko_bindings::bindings::{Gecko_CreateCSSErrorReporter, Gecko_DestroyCSSErrorReporter};
use style::gecko_bindings::bindings::Gecko_ReportUnexpectedCSSError;
use style::gecko_bindings::structs::{Loader, ServoStyleSheet, nsIURI};
use style::gecko_bindings::structs::ErrorReporter as GeckoErrorReporter;
use style::gecko_bindings::structs::URLExtraData as RawUrlExtraData;
use style::gecko_bindings::sugar::refptr::RefPtr;
use style::stylesheets::UrlExtraData;

/// Wrapper around an instance of Gecko's CSS error reporter.
pub struct ErrorReporter(*mut GeckoErrorReporter);

impl ErrorReporter {
    /// Create a new instance of the Gecko error reporter.
    pub fn new(sheet: *mut ServoStyleSheet,
               loader: *mut Loader,
               url: *mut RawUrlExtraData) -> ErrorReporter {
        unsafe {
            let url = RefPtr::from_ptr_ref(&url);
            ErrorReporter(Gecko_CreateCSSErrorReporter(sheet, loader, url.mBaseURI.raw::<nsIURI>()))
        }
    }
}

impl Drop for ErrorReporter {
    fn drop(&mut self) {
        unsafe {
            Gecko_DestroyCSSErrorReporter(self.0);
        }
    }
}

trait ErrorHelpers<'a> {
    fn to_gecko_message(&self) -> (&'static [u8], &'a str);
}

impl<'a> ErrorHelpers<'a> for ContextualParseError<'a> {
    fn to_gecko_message(&self) -> (&'static [u8], &'a str) {
        match *self {
            ContextualParseError::UnsupportedPropertyDeclaration(decl, _) =>
                (b"PEUnknownProperty\0", decl),
            ContextualParseError::UnsupportedFontFaceDescriptor(decl, _) =>
                (b"PEUnknwnFontDesc\0", decl),
            ContextualParseError::InvalidKeyframeRule(rule, _) =>
                (b"PEKeyframeBadName\0", rule),
            ContextualParseError::UnsupportedKeyframePropertyDeclaration(decl, _) =>
                (b"PEBadSelectorKeyframeRuleIgnored\0", decl),
            ContextualParseError::InvalidRule(rule, _) =>
                (b"PEDeclDropped\0", rule),
            ContextualParseError::UnsupportedRule(rule, _) =>
                (b"PEDeclDropped\0", rule),
            ContextualParseError::UnsupportedViewportDescriptorDeclaration(..) |
            ContextualParseError::UnsupportedCounterStyleDescriptorDeclaration(..) |
            ContextualParseError::InvalidCounterStyleWithoutSymbols(..) |
            ContextualParseError::InvalidCounterStyleNotEnoughSymbols(..) |
            ContextualParseError::InvalidCounterStyleWithoutAdditiveSymbols |
            ContextualParseError::InvalidCounterStyleExtendsWithSymbols |
            ContextualParseError::InvalidCounterStyleExtendsWithAdditiveSymbols =>
                (b"PEUnknownAtRule\0", ""),
        }
    }
}

impl ParseErrorReporter for ErrorReporter {
    fn report_error<'a>(&self,
                        input: &mut Parser,
                        position: SourcePosition,
                        error: ContextualParseError<'a>,
                        url: &UrlExtraData,
                        line_number_offset: u64) {
        let location = input.source_location(position);
        let line_number = location.line + line_number_offset as u32;

        let (name, param) = error.to_gecko_message();
        let source = "";
        unsafe {
            Gecko_ReportUnexpectedCSSError(self.0,
                                           name.as_ptr() as *const _,
                                           param.as_ptr() as *const _,
                                           param.len() as u32,
                                           source.as_ptr() as *const _,
                                           source.len() as u32,
                                           line_number as u32,
                                           location.column as u32,
                                           url.mBaseURI.raw::<nsIURI>());
        }
    }
}
