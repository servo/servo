/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Wrapper around Gecko's CSS error reporting mechanism.

#![allow(unsafe_code)]

use cssparser::{CowRcStr, serialize_identifier, ToCss};
use cssparser::{SourceLocation, ParseError as CssParseError, Token, BasicParseError};
use selectors::parser::SelectorParseError;
use std::ptr;
use style::error_reporting::{ParseErrorReporter, ContextualParseError};
use style::gecko_bindings::bindings::{Gecko_CreateCSSErrorReporter, Gecko_DestroyCSSErrorReporter};
use style::gecko_bindings::bindings::Gecko_ReportUnexpectedCSSError;
use style::gecko_bindings::structs::{Loader, ServoStyleSheet, nsIURI};
use style::gecko_bindings::structs::ErrorReporter as GeckoErrorReporter;
use style::gecko_bindings::structs::URLExtraData as RawUrlExtraData;
use style::gecko_bindings::sugar::refptr::RefPtr;
use style::stylesheets::UrlExtraData;
use style_traits::{ParseError, StyleParseError, PropertyDeclarationParseError, ValueParseError};

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

enum ErrorString<'a> {
    Snippet(CowRcStr<'a>),
    Ident(CowRcStr<'a>),
    UnexpectedToken(Token<'a>),
}

impl<'a> ErrorString<'a> {
    fn into_str(self) -> CowRcStr<'a> {
        match self {
            ErrorString::Snippet(s) => s,
            ErrorString::UnexpectedToken(t) => t.to_css_string().into(),
            ErrorString::Ident(i) => {
                let mut s = String::new();
                serialize_identifier(&i, &mut s).unwrap();
                s.into()
            }
        }
    }
}

enum Action {
    Nothing,
    Skip,
    Drop,
}

trait ErrorHelpers<'a> {
    fn error_data(self) -> (CowRcStr<'a>, ParseError<'a>);
    fn error_params(self) -> ErrorParams<'a>;
    fn to_gecko_message(&self) -> (Option<&'static [u8]>, &'static [u8], Action);
}

fn extract_error_param<'a>(err: ParseError<'a>) -> Option<ErrorString<'a>> {
    Some(match err {
        CssParseError::Basic(BasicParseError::UnexpectedToken(t)) => {
            ErrorString::UnexpectedToken(t)
        }

        CssParseError::Basic(BasicParseError::AtRuleInvalid(i)) |
        CssParseError::Custom(SelectorParseError::Custom(
            StyleParseError::UnsupportedAtRule(i)
        )) => {
            let mut s = String::from("@");
            serialize_identifier(&i, &mut s).unwrap();
            ErrorString::Snippet(s.into())
        }

        CssParseError::Custom(SelectorParseError::Custom(
            StyleParseError::PropertyDeclaration(
                PropertyDeclarationParseError::InvalidValue(property, None)
            )
        )) => {
            ErrorString::Snippet(property)
        }

        CssParseError::Custom(SelectorParseError::UnexpectedIdent(ident)) => {
            ErrorString::Ident(ident)
        }

        CssParseError::Custom(SelectorParseError::Custom(
            StyleParseError::PropertyDeclaration(
                PropertyDeclarationParseError::UnknownProperty(property)
            )
        )) => {
            ErrorString::Ident(property)
        }

        CssParseError::Custom(SelectorParseError::Custom(
            StyleParseError::UnexpectedTokenWithinNamespace(token)
        )) => {
            ErrorString::UnexpectedToken(token)
        }

        _ => return None,
    })
}

fn extract_value_error_param<'a>(err: ValueParseError<'a>) -> ErrorString<'a> {
    match err {
        ValueParseError::InvalidColor(t) => ErrorString::UnexpectedToken(t),
    }
}

struct ErrorParams<'a> {
    prefix_param: Option<ErrorString<'a>>,
    main_param: Option<ErrorString<'a>>,
}

/// If an error parameter is present in the given error, return it. Additionally return
/// a second parameter if it exists, for use in the prefix for the eventual error message.
fn extract_error_params<'a>(err: ParseError<'a>) -> Option<ErrorParams<'a>> {
    let (main, prefix) = match err {
        CssParseError::Custom(SelectorParseError::Custom(
            StyleParseError::PropertyDeclaration(
                PropertyDeclarationParseError::InvalidValue(property, Some(e))))) =>
            (Some(ErrorString::Snippet(property.into())), Some(extract_value_error_param(e))),

        CssParseError::Custom(SelectorParseError::UnexpectedTokenInAttributeSelector(t)) |
        CssParseError::Custom(SelectorParseError::BadValueInAttr(t)) |
        CssParseError::Custom(SelectorParseError::ExpectedBarInAttr(t)) |
        CssParseError::Custom(SelectorParseError::NoQualifiedNameInAttributeSelector(t)) |
        CssParseError::Custom(SelectorParseError::InvalidQualNameInAttr(t)) |
        CssParseError::Custom(SelectorParseError::ExplicitNamespaceUnexpectedToken(t)) |
        CssParseError::Custom(SelectorParseError::PseudoElementExpectedIdent(t)) |
        CssParseError::Custom(SelectorParseError::NoIdentForPseudo(t)) |
        CssParseError::Custom(SelectorParseError::ClassNeedsIdent(t)) |
        CssParseError::Custom(SelectorParseError::PseudoElementExpectedColon(t)) =>
            (None, Some(ErrorString::UnexpectedToken(t))),

        CssParseError::Custom(SelectorParseError::ExpectedNamespace(namespace)) =>
            (None, Some(ErrorString::Ident(namespace))),

        CssParseError::Custom(SelectorParseError::UnsupportedPseudoClassOrElement(p)) =>
            (None, Some(ErrorString::Ident(p))),

        CssParseError::Custom(SelectorParseError::EmptySelector) |
        CssParseError::Custom(SelectorParseError::DanglingCombinator) =>
            (None, None),

        CssParseError::Custom(SelectorParseError::EmptyNegation) =>
            (None, Some(ErrorString::Snippet(")".into()))),

        err => match extract_error_param(err) {
            Some(e) => (Some(e), None),
            None => return None,
        }
    };
    Some(ErrorParams {
        main_param: main,
        prefix_param: prefix,
    })
}

impl<'a> ErrorHelpers<'a> for ContextualParseError<'a> {
    fn error_data(self) -> (CowRcStr<'a>, ParseError<'a>) {
        match self {
            ContextualParseError::UnsupportedPropertyDeclaration(s, err) |
            ContextualParseError::UnsupportedFontFaceDescriptor(s, err) |
            ContextualParseError::UnsupportedFontFeatureValuesDescriptor(s, err) |
            ContextualParseError::InvalidKeyframeRule(s, err) |
            ContextualParseError::InvalidFontFeatureValuesRule(s, err) |
            ContextualParseError::UnsupportedKeyframePropertyDeclaration(s, err) |
            ContextualParseError::InvalidRule(s, err) |
            ContextualParseError::UnsupportedRule(s, err) |
            ContextualParseError::UnsupportedViewportDescriptorDeclaration(s, err) |
            ContextualParseError::UnsupportedCounterStyleDescriptorDeclaration(s, err) =>
                (s.into(), err),
            ContextualParseError::InvalidCounterStyleWithoutSymbols(s) |
            ContextualParseError::InvalidCounterStyleNotEnoughSymbols(s) =>
                (s.into(), StyleParseError::UnspecifiedError.into()),
            ContextualParseError::InvalidCounterStyleWithoutAdditiveSymbols |
            ContextualParseError::InvalidCounterStyleExtendsWithSymbols |
            ContextualParseError::InvalidCounterStyleExtendsWithAdditiveSymbols =>
                ("".into(), StyleParseError::UnspecifiedError.into())
        }
    }

    fn error_params(self) -> ErrorParams<'a> {
        let (s, error) = self.error_data();
        extract_error_params(error).unwrap_or_else(|| ErrorParams {
            main_param: Some(ErrorString::Snippet(s)),
            prefix_param: None
        })
    }

    fn to_gecko_message(&self) -> (Option<&'static [u8]>, &'static [u8], Action) {
        let (msg, action): (&[u8], Action) = match *self {
            ContextualParseError::UnsupportedPropertyDeclaration(
                _, CssParseError::Basic(BasicParseError::UnexpectedToken(_))) |
            ContextualParseError::UnsupportedPropertyDeclaration(
                _, CssParseError::Basic(BasicParseError::AtRuleInvalid(_))) =>
                (b"PEParseDeclarationDeclExpected\0", Action::Skip),
            ContextualParseError::UnsupportedPropertyDeclaration(
                _, CssParseError::Custom(SelectorParseError::Custom(
                    StyleParseError::PropertyDeclaration(
                        PropertyDeclarationParseError::InvalidValue(_, ref err))))) => {
                let prefix = match *err {
                    Some(ValueParseError::InvalidColor(_)) => Some(&b"PEColorNotColor\0"[..]),
                    _ => None,
                };
                return (prefix, b"PEValueParsingError\0", Action::Drop);
            }
            ContextualParseError::UnsupportedPropertyDeclaration(..) =>
                (b"PEUnknownProperty\0", Action::Drop),
            ContextualParseError::UnsupportedFontFaceDescriptor(..) =>
                (b"PEUnknwnFontDesc\0", Action::Skip),
            ContextualParseError::InvalidKeyframeRule(..) =>
                (b"PEKeyframeBadName\0", Action::Nothing),
            ContextualParseError::UnsupportedKeyframePropertyDeclaration(..) =>
                (b"PEBadSelectorKeyframeRuleIgnored\0", Action::Nothing),
            ContextualParseError::InvalidRule(
                _, CssParseError::Custom(SelectorParseError::Custom(
                StyleParseError::UnexpectedTokenWithinNamespace(_)))) =>
                (b"PEAtNSUnexpected\0", Action::Nothing),
            ContextualParseError::InvalidRule(
                _, CssParseError::Basic(BasicParseError::AtRuleInvalid(_))) |
            ContextualParseError::InvalidRule(
                _, CssParseError::Custom(SelectorParseError::Custom(
                    StyleParseError::UnsupportedAtRule(_)))) =>
                (b"PEUnknownAtRule\0", Action::Nothing),
            ContextualParseError::InvalidRule(_, ref err) => {
                let prefix = match *err {
                    CssParseError::Custom(SelectorParseError::UnexpectedTokenInAttributeSelector(_)) =>
                        Some(&b"PEAttSelUnexpected\0"[..]),
                    CssParseError::Custom(SelectorParseError::ExpectedBarInAttr(_)) =>
                        Some(&b"PEAttSelNoBar\0"[..]),
                    CssParseError::Custom(SelectorParseError::BadValueInAttr(_)) =>
                        Some(&b"PEAttSelBadValue\0"[..]),
                    CssParseError::Custom(SelectorParseError::NoQualifiedNameInAttributeSelector(_)) =>
                        Some(&b"PEAttributeNameOrNamespaceExpected\0"[..]),
                    CssParseError::Custom(SelectorParseError::InvalidQualNameInAttr(_)) =>
                        Some(&b"PEAttributeNameExpected\0"[..]),
                    CssParseError::Custom(SelectorParseError::ExplicitNamespaceUnexpectedToken(_)) =>
                        Some(&b"PETypeSelNotType\0"[..]),
                    CssParseError::Custom(SelectorParseError::ExpectedNamespace(_)) =>
                       Some(&b"PEUnknownNamespacePrefix\0"[..]),
                    CssParseError::Custom(SelectorParseError::EmptySelector) =>
                        Some(&b"PESelectorGroupNoSelector\0"[..]),
                    CssParseError::Custom(SelectorParseError::DanglingCombinator) =>
                        Some(&b"PESelectorGroupExtraCombinator\0"[..]),
                    CssParseError::Custom(SelectorParseError::UnsupportedPseudoClassOrElement(_)) =>
                        Some(&b"PEPseudoSelUnknown\0"[..]),
                    CssParseError::Custom(SelectorParseError::PseudoElementExpectedColon(_)) =>
                        Some(&b"PEPseudoSelEndOrUserActionPC\0"[..]),
                    CssParseError::Custom(SelectorParseError::NoIdentForPseudo(_)) =>
                        Some(&b"PEPseudoClassArgNotIdent\0"[..]),
                    CssParseError::Custom(SelectorParseError::PseudoElementExpectedIdent(_)) =>
                        Some(&b"PEPseudoSelBadName\0"[..]),
                    CssParseError::Custom(SelectorParseError::ClassNeedsIdent(_)) =>
                        Some(&b"PEClassSelNotIdent\0"[..]),
                    CssParseError::Custom(SelectorParseError::EmptyNegation) =>
                        Some(&b"PENegationBadArg\0"[..]),
                    _ => None,
                };
                return (prefix, b"PEBadSelectorRSIgnored\0", Action::Nothing);
            }
            ContextualParseError::UnsupportedRule(..) =>
                (b"PEDeclDropped\0", Action::Nothing),
            ContextualParseError::UnsupportedViewportDescriptorDeclaration(..) |
            ContextualParseError::UnsupportedCounterStyleDescriptorDeclaration(..) |
            ContextualParseError::InvalidCounterStyleWithoutSymbols(..) |
            ContextualParseError::InvalidCounterStyleNotEnoughSymbols(..) |
            ContextualParseError::InvalidCounterStyleWithoutAdditiveSymbols |
            ContextualParseError::InvalidCounterStyleExtendsWithSymbols |
            ContextualParseError::InvalidCounterStyleExtendsWithAdditiveSymbols |
            ContextualParseError::UnsupportedFontFeatureValuesDescriptor(..) |
            ContextualParseError::InvalidFontFeatureValuesRule(..) =>
                (b"PEUnknownAtRule\0", Action::Skip),
        };
        (None, msg, action)
    }
}

impl ParseErrorReporter for ErrorReporter {
    fn report_error(&self,
                    _url: &UrlExtraData,
                    location: SourceLocation,
                    error: ContextualParseError) {
        let (pre, name, action) = error.to_gecko_message();
        let suffix = match action {
            Action::Nothing => ptr::null(),
            Action::Skip => b"PEDeclSkipped\0".as_ptr(),
            Action::Drop => b"PEDeclDropped\0".as_ptr(),
        };
        let params = error.error_params();
        let param = params.main_param;
        let pre_param = params.prefix_param;
        let param = param.map(|p| p.into_str());
        let pre_param = pre_param.map(|p| p.into_str());
        let param_ptr = param.as_ref().map_or(ptr::null(), |p| p.as_ptr());
        let pre_param_ptr = pre_param.as_ref().map_or(ptr::null(), |p| p.as_ptr());
        // The CSS source text is unused and will be removed in bug 1381188.
        let source = "";
        unsafe {
            Gecko_ReportUnexpectedCSSError(self.0,
                                           name.as_ptr() as *const _,
                                           param_ptr as *const _,
                                           param.as_ref().map_or(0, |p| p.len()) as u32,
                                           pre.map_or(ptr::null(), |p| p.as_ptr()) as *const _,
                                           pre_param_ptr as *const _,
                                           pre_param.as_ref().map_or(0, |p| p.len()) as u32,
                                           suffix as *const _,
                                           source.as_ptr() as *const _,
                                           source.len() as u32,
                                           location.line,
                                           location.column);
        }
    }
}
