/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Wrapper around Gecko's CSS error reporting mechanism.

#![allow(unsafe_code)]

use cssparser::{CowRcStr, serialize_identifier, ToCss};
use cssparser::{SourceLocation, ParseError, ParseErrorKind, Token, BasicParseErrorKind};
use selectors::parser::SelectorParseErrorKind;
use std::ffi::CStr;
use std::ptr;
use style::error_reporting::{ParseErrorReporter, ContextualParseError};
use style::gecko_bindings::bindings::{Gecko_CreateCSSErrorReporter, Gecko_DestroyCSSErrorReporter};
use style::gecko_bindings::bindings::Gecko_ReportUnexpectedCSSError;
use style::gecko_bindings::structs::{Loader, StyleSheet as DomStyleSheet, nsIURI};
use style::gecko_bindings::structs::ErrorReporter as GeckoErrorReporter;
use style::gecko_bindings::structs::URLExtraData as RawUrlExtraData;
use style::stylesheets::UrlExtraData;
use style_traits::{StyleParseErrorKind, ValueParseErrorKind};

pub type ErrorKind<'i> = ParseErrorKind<'i, StyleParseErrorKind<'i>>;

/// Wrapper around an instance of Gecko's CSS error reporter.
pub struct ErrorReporter(*mut GeckoErrorReporter);

impl ErrorReporter {
    /// Create a new instance of the Gecko error reporter.
    pub fn new(
        sheet: *mut DomStyleSheet,
        loader: *mut Loader,
        extra_data: *mut RawUrlExtraData,
    ) -> Self {
        unsafe {
            let url = extra_data.as_ref()
                .map(|d| d.mBaseURI.raw::<nsIURI>())
                .unwrap_or(ptr::null_mut());
            ErrorReporter(Gecko_CreateCSSErrorReporter(sheet, loader, url))
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

#[derive(Debug)]
enum Action {
    Nothing,
    Skip,
    Drop,
}

trait ErrorHelpers<'a> {
    fn error_data(self) -> (CowRcStr<'a>, ErrorKind<'a>);
    fn error_params(self) -> ErrorParams<'a>;
    fn to_gecko_message(&self) -> (Option<&'static CStr>, &'static CStr, Action);
}

fn extract_error_param<'a>(err: ErrorKind<'a>) -> Option<ErrorString<'a>> {
    Some(match err {
        ParseErrorKind::Basic(BasicParseErrorKind::UnexpectedToken(t)) => {
            ErrorString::UnexpectedToken(t)
        }

        ParseErrorKind::Basic(BasicParseErrorKind::AtRuleInvalid(i)) |
        ParseErrorKind::Custom(StyleParseErrorKind::UnsupportedAtRule(i)) => {
            let mut s = String::from("@");
            serialize_identifier(&i, &mut s).unwrap();
            ErrorString::Snippet(s.into())
        }

        ParseErrorKind::Custom(StyleParseErrorKind::OtherInvalidValue(property)) => {
            ErrorString::Snippet(property)
        }

        ParseErrorKind::Custom(
            StyleParseErrorKind::SelectorError(
                SelectorParseErrorKind::UnexpectedIdent(ident)
            )
        ) => {
            ErrorString::Ident(ident)
        }

        ParseErrorKind::Custom(StyleParseErrorKind::UnknownProperty(property)) => {
            ErrorString::Ident(property)
        }

        ParseErrorKind::Custom(
            StyleParseErrorKind::UnexpectedTokenWithinNamespace(token)
        ) => {
            ErrorString::UnexpectedToken(token)
        }

        _ => return None,
    })
}

struct ErrorParams<'a> {
    prefix_param: Option<ErrorString<'a>>,
    main_param: Option<ErrorString<'a>>,
}

/// If an error parameter is present in the given error, return it. Additionally return
/// a second parameter if it exists, for use in the prefix for the eventual error message.
fn extract_error_params<'a>(err: ErrorKind<'a>) -> Option<ErrorParams<'a>> {
    let (main, prefix) = match err {
        ParseErrorKind::Custom(StyleParseErrorKind::InvalidColor(property, token)) |
        ParseErrorKind::Custom(StyleParseErrorKind::InvalidFilter(property, token)) => {
            (Some(ErrorString::Snippet(property.into())), Some(ErrorString::UnexpectedToken(token)))
        }

        ParseErrorKind::Custom(
            StyleParseErrorKind::MediaQueryExpectedFeatureName(ident)
        ) => {
            (Some(ErrorString::Ident(ident)), None)
        }

        ParseErrorKind::Custom(
            StyleParseErrorKind::ExpectedIdentifier(token)
        ) |
        ParseErrorKind::Custom(
            StyleParseErrorKind::ValueError(ValueParseErrorKind::InvalidColor(token))
        ) => {
            (Some(ErrorString::UnexpectedToken(token)), None)
        }

        ParseErrorKind::Custom(StyleParseErrorKind::SelectorError(err)) => match err {
            SelectorParseErrorKind::UnexpectedTokenInAttributeSelector(t) |
            SelectorParseErrorKind::BadValueInAttr(t) |
            SelectorParseErrorKind::ExpectedBarInAttr(t) |
            SelectorParseErrorKind::NoQualifiedNameInAttributeSelector(t) |
            SelectorParseErrorKind::InvalidQualNameInAttr(t) |
            SelectorParseErrorKind::ExplicitNamespaceUnexpectedToken(t) |
            SelectorParseErrorKind::PseudoElementExpectedIdent(t) |
            SelectorParseErrorKind::NoIdentForPseudo(t) |
            SelectorParseErrorKind::ClassNeedsIdent(t) |
            SelectorParseErrorKind::PseudoElementExpectedColon(t) => {
                (None, Some(ErrorString::UnexpectedToken(t)))
            }
            SelectorParseErrorKind::ExpectedNamespace(namespace) => {
                (None, Some(ErrorString::Ident(namespace)))
            }
            SelectorParseErrorKind::UnsupportedPseudoClassOrElement(p) => {
                (None, Some(ErrorString::Ident(p)))
            }
            SelectorParseErrorKind::EmptySelector |
            SelectorParseErrorKind::DanglingCombinator => {
                (None, None)
            }
            SelectorParseErrorKind::EmptyNegation => {
                (None, Some(ErrorString::Snippet(")".into())))
            }
            err => match extract_error_param(ParseErrorKind::Custom(StyleParseErrorKind::SelectorError(err))) {
                Some(e) => (Some(e), None),
                None => return None,
            }
        },
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
    fn error_data(self) -> (CowRcStr<'a>, ErrorKind<'a>) {
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
            ContextualParseError::UnsupportedCounterStyleDescriptorDeclaration(s, err) |
            ContextualParseError::InvalidMediaRule(s, err) |
            ContextualParseError::UnsupportedValue(s, err) => {
                (s.into(), err.kind)
            }
            ContextualParseError::InvalidCounterStyleWithoutSymbols(s) |
            ContextualParseError::InvalidCounterStyleNotEnoughSymbols(s) => {
                (s.into(), ParseErrorKind::Custom(StyleParseErrorKind::UnspecifiedError.into()))
            }
            ContextualParseError::InvalidCounterStyleWithoutAdditiveSymbols |
            ContextualParseError::InvalidCounterStyleExtendsWithSymbols |
            ContextualParseError::InvalidCounterStyleExtendsWithAdditiveSymbols => {
                ("".into(), ParseErrorKind::Custom(StyleParseErrorKind::UnspecifiedError.into()))
            }
        }
    }

    fn error_params(self) -> ErrorParams<'a> {
        let (s, error) = self.error_data();
        extract_error_params(error).unwrap_or_else(|| ErrorParams {
            main_param: Some(ErrorString::Snippet(s)),
            prefix_param: None
        })
    }

    fn to_gecko_message(&self) -> (Option<&'static CStr>, &'static CStr, Action) {
        let (msg, action): (&CStr, Action) = match *self {
            ContextualParseError::UnsupportedPropertyDeclaration(
                _, ParseError { kind: ParseErrorKind::Basic(BasicParseErrorKind::UnexpectedToken(_)), .. }
            ) |
            ContextualParseError::UnsupportedPropertyDeclaration(
                _, ParseError { kind: ParseErrorKind::Basic(BasicParseErrorKind::AtRuleInvalid(_)), .. }
            ) => {
                (cstr!("PEParseDeclarationDeclExpected"), Action::Skip)
            }
            ContextualParseError::UnsupportedPropertyDeclaration(
                _, ParseError { kind: ParseErrorKind::Custom(ref err), .. }
            ) => {
                match *err {
                    StyleParseErrorKind::InvalidColor(_, _) => {
                        return (Some(cstr!("PEColorNotColor")),
                                cstr!("PEValueParsingError"), Action::Drop)
                    }
                    StyleParseErrorKind::InvalidFilter(_, _) => {
                        return (Some(cstr!("PEExpectedNoneOrURLOrFilterFunction")),
                                cstr!("PEValueParsingError"), Action::Drop)
                    }
                    StyleParseErrorKind::OtherInvalidValue(_) => {
                        (cstr!("PEValueParsingError"), Action::Drop)
                    }
                    _ => (cstr!("PEUnknownProperty"), Action::Drop)
                }
            }
            ContextualParseError::UnsupportedPropertyDeclaration(..) =>
                (cstr!("PEUnknownProperty"), Action::Drop),
            ContextualParseError::UnsupportedFontFaceDescriptor(..) =>
                (cstr!("PEUnknownFontDesc"), Action::Skip),
            ContextualParseError::InvalidKeyframeRule(..) =>
                (cstr!("PEKeyframeBadName"), Action::Nothing),
            ContextualParseError::UnsupportedKeyframePropertyDeclaration(..) =>
                (cstr!("PEBadSelectorKeyframeRuleIgnored"), Action::Nothing),
            ContextualParseError::InvalidRule(
                _, ParseError { kind: ParseErrorKind::Custom(
                    StyleParseErrorKind::UnexpectedTokenWithinNamespace(_)
                ), .. }
            ) => {
                (cstr!("PEAtNSUnexpected"), Action::Nothing)
            }
            ContextualParseError::InvalidRule(
                _, ParseError { kind: ParseErrorKind::Basic(BasicParseErrorKind::AtRuleInvalid(_)), .. }
            ) |
            ContextualParseError::InvalidRule(
                _, ParseError { kind: ParseErrorKind::Custom(
                    StyleParseErrorKind::UnsupportedAtRule(_)
                ), .. }
            ) => {
                (cstr!("PEUnknownAtRule"), Action::Nothing)
            }
            ContextualParseError::InvalidRule(_, ref err) => {
                let prefix = match err.kind {
                    ParseErrorKind::Custom(StyleParseErrorKind::SelectorError(ref err)) => match *err {
                        SelectorParseErrorKind::UnexpectedTokenInAttributeSelector(_) => {
                            Some(cstr!("PEAttSelUnexpected"))
                        }
                        SelectorParseErrorKind::ExpectedBarInAttr(_) => {
                            Some(cstr!("PEAttSelNoBar"))
                        }
                        SelectorParseErrorKind::BadValueInAttr(_) => {
                            Some(cstr!("PEAttSelBadValue"))
                        }
                        SelectorParseErrorKind::NoQualifiedNameInAttributeSelector(_) => {
                            Some(cstr!("PEAttributeNameOrNamespaceExpected"))
                        }
                        SelectorParseErrorKind::InvalidQualNameInAttr(_) => {
                            Some(cstr!("PEAttributeNameExpected"))
                        }
                        SelectorParseErrorKind::ExplicitNamespaceUnexpectedToken(_) => {
                            Some(cstr!("PETypeSelNotType"))
                        }
                        SelectorParseErrorKind::ExpectedNamespace(_) => {
                           Some(cstr!("PEUnknownNamespacePrefix"))
                        }
                        SelectorParseErrorKind::EmptySelector => {
                            Some(cstr!("PESelectorGroupNoSelector"))
                        }
                        SelectorParseErrorKind::DanglingCombinator => {
                            Some(cstr!("PESelectorGroupExtraCombinator"))
                        }
                        SelectorParseErrorKind::UnsupportedPseudoClassOrElement(_) => {
                            Some(cstr!("PEPseudoSelUnknown"))
                        }
                        SelectorParseErrorKind::PseudoElementExpectedColon(_) => {
                            Some(cstr!("PEPseudoSelEndOrUserActionPC"))
                        }
                        SelectorParseErrorKind::NoIdentForPseudo(_) => {
                            Some(cstr!("PEPseudoClassArgNotIdent"))
                        }
                        SelectorParseErrorKind::PseudoElementExpectedIdent(_) => {
                            Some(cstr!("PEPseudoSelBadName"))
                        }
                        SelectorParseErrorKind::ClassNeedsIdent(_) => {
                            Some(cstr!("PEClassSelNotIdent"))
                        }
                        SelectorParseErrorKind::EmptyNegation => {
                            Some(cstr!("PENegationBadArg"))
                        }
                        _ => None,
                    },
                    _ => None,
                };
                return (prefix, cstr!("PEBadSelectorRSIgnored"), Action::Nothing);
            }
            ContextualParseError::InvalidMediaRule(_, ref err) => {
                let err: &CStr = match err.kind {
                    ParseErrorKind::Custom(StyleParseErrorKind::ExpectedIdentifier(..)) => {
                        cstr!("PEGatherMediaNotIdent")
                    },
                    ParseErrorKind::Custom(StyleParseErrorKind::MediaQueryExpectedFeatureName(..)) => {
                        cstr!("PEMQExpectedFeatureName")
                    },
                    ParseErrorKind::Custom(StyleParseErrorKind::MediaQueryExpectedFeatureValue) => {
                        cstr!("PEMQExpectedFeatureValue")
                    },
                    ParseErrorKind::Custom(StyleParseErrorKind::RangedExpressionWithNoValue) => {
                        cstr!("PEMQNoMinMaxWithoutValue")
                    },
                    _ => {
                        cstr!("PEDeclDropped")
                    },
                };
                (err, Action::Nothing)
            }
            ContextualParseError::UnsupportedRule(..) =>
                (cstr!("PEDeclDropped"), Action::Nothing),
            ContextualParseError::UnsupportedViewportDescriptorDeclaration(..) |
            ContextualParseError::UnsupportedCounterStyleDescriptorDeclaration(..) |
            ContextualParseError::InvalidCounterStyleWithoutSymbols(..) |
            ContextualParseError::InvalidCounterStyleNotEnoughSymbols(..) |
            ContextualParseError::InvalidCounterStyleWithoutAdditiveSymbols |
            ContextualParseError::InvalidCounterStyleExtendsWithSymbols |
            ContextualParseError::InvalidCounterStyleExtendsWithAdditiveSymbols |
            ContextualParseError::UnsupportedFontFeatureValuesDescriptor(..) |
            ContextualParseError::InvalidFontFeatureValuesRule(..) =>
                (cstr!("PEUnknownAtRule"), Action::Skip),
            ContextualParseError::UnsupportedValue(_, ParseError { ref kind, .. }) => {
                match *kind {
                    ParseErrorKind::Custom(
                        StyleParseErrorKind::ValueError(
                            ValueParseErrorKind::InvalidColor(..)
                        )
                    ) => (cstr!("PEColorNotColor"), Action::Nothing),
                    _ => {
                        // Not the best error message, since we weren't parsing
                        // a declaration, just a value. But we don't produce
                        // UnsupportedValue errors other than InvalidColors
                        // currently.
                        debug_assert!(false, "should use a more specific error message");
                        (cstr!("PEDeclDropped"), Action::Nothing)
                    }
                }
            }
        };
        (None, msg, action)
    }
}

impl ErrorReporter {
    pub fn report(&self, location: SourceLocation, error: ContextualParseError) {
        let (pre, name, action) = error.to_gecko_message();
        let suffix = match action {
            Action::Nothing => ptr::null(),
            Action::Skip => cstr!("PEDeclSkipped").as_ptr(),
            Action::Drop => cstr!("PEDeclDropped").as_ptr(),
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

impl ParseErrorReporter for ErrorReporter {
    fn report_error(
        &self,
        _url: &UrlExtraData,
        location: SourceLocation,
        error: ContextualParseError
    ) {
        self.report(location, error)
    }
}
