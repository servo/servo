/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Wrapper around Gecko's CSS error reporting mechanism.

#![allow(unsafe_code)]

use cssparser::{Parser, SourcePosition, ParseError as CssParseError, Token, BasicParseError};
use cssparser::CowRcStr;
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
    fn into_str(self) -> String {
        match self {
            ErrorString::Snippet(s) => s.as_ref().to_owned(),
            ErrorString::Ident(i) => escape_css_ident(&i),
            ErrorString::UnexpectedToken(t) => token_to_str(t),
        }
    }
}

// This is identical to the behaviour of cssparser::serialize_identifier, except that
// it uses numerical escapes for a larger set of characters.
fn escape_css_ident(ident: &str) -> String {
    // The relevant parts of the CSS grammar are:
    //   ident    ([-]?{nmstart}|[-][-]){nmchar}*
    //   nmstart  [_a-z]|{nonascii}|{escape}
    //   nmchar   [_a-z0-9-]|{nonascii}|{escape}
    //   nonascii [^\0-\177]
    //   escape   {unicode}|\\[^\n\r\f0-9a-f]
    //   unicode  \\[0-9a-f]{1,6}(\r\n|[ \n\r\t\f])?
    // from http://www.w3.org/TR/CSS21/syndata.html#tokenization but
    // modified for idents by
    // http://dev.w3.org/csswg/cssom/#serialize-an-identifier and
    // http://dev.w3.org/csswg/css-syntax/#would-start-an-identifier
    if ident.is_empty() {
        return ident.into()
    }

    let mut escaped = String::new();

    // A leading dash does not need to be escaped as long as it is not the
    // *only* character in the identifier.
    let mut iter = ident.chars().peekable();
    if iter.peek() == Some(&'-') {
        if ident.len() == 1 {
            return "\\-".into();
        }

        escaped.push('-');
        // Skip the first character.
        let _ = iter.next();
    }

    // Escape a digit at the start (including after a dash),
    // numerically.  If we didn't escape it numerically, it would get
    // interpreted as a numeric escape for the wrong character.
    if iter.peek().map_or(false, |&c| '0' <= c && c <= '9') {
        let ch = iter.next().unwrap();
        escaped.push_str(&format!("\\{:x} ", ch as u32));
    }

    while let Some(ch) = iter.next() {
        if ch == '\0' {
            escaped.push_str("\u{FFFD}");
        } else if ch < (0x20 as char) || (0x7f as char <= ch && ch < (0xA0 as char)) {
            // Escape U+0000 through U+001F and U+007F through U+009F numerically.
            escaped.push_str(&format!("\\{:x} ", ch as u32));
        } else {
            // Escape ASCII non-identifier printables as a backslash plus
            // the character.
            if (ch < (0x7F as char)) &&
                ch != '_' && ch != '-' &&
                (ch < '0' || '9' < ch) &&
                (ch < 'A' || 'Z' < ch) &&
                (ch < 'a' || 'z' < ch)
            {
                escaped.push('\\');
            }
            escaped.push(ch);
        }
    }

    escaped
}

// This is identical to the behaviour of cssparser::CssStringWriter, except that
// the characters between 0x7F and 0xA0 as numerically escaped as well.
fn escape_css_string(s: &str) -> String {
    let mut escaped = String::new();
    for ch in s.chars() {
        if ch < ' ' || (ch >= (0x7F as char) && ch < (0xA0 as char)) {
            escaped.push_str(&format!("\\{:x} ", ch as u32));
        } else {
            if ch == '"' || ch == '\'' || ch == '\\' {
                // Escape backslash and quote characters symbolically.
                // It's not technically necessary to escape the quote
                // character that isn't being used to delimit the string,
                // but we do it anyway because that makes testing simpler.
                escaped.push('\\');
            }
            escaped.push(ch);
        }
    }
    escaped
}

fn token_to_str<'a>(t: Token<'a>) -> String {
    match t {
        Token::Ident(i) => escape_css_ident(&i),
        Token::AtKeyword(kw) => format!("@{}", escape_css_ident(&kw)),
        Token::Hash(h) | Token::IDHash(h) => format!("#{}", escape_css_ident(&h)),
        Token::QuotedString(s) => format!("'{}'", escape_css_string(&s)),
        Token::UnquotedUrl(u) => format!("'{}'", escape_css_string(&u)),
        Token::Delim(d) => d.to_string(),
        Token::Number { int_value: Some(i), .. } => i.to_string(),
        Token::Number { value, .. } => value.to_string(),
        Token::Percentage { int_value: Some(i), .. } => i.to_string(),
        Token::Percentage { unit_value, .. } => unit_value.to_string(),
        Token::Dimension { int_value: Some(i), ref unit, .. } =>
            format!("{}{}", i.to_string(), escape_css_ident(&unit.to_string())),
        Token::Dimension { value, ref unit, .. } =>
            format!("{}{}", value.to_string(), escape_css_ident(&unit.to_string())),
        Token::WhiteSpace(_) => "whitespace".into(),
        Token::Comment(_) => "comment".into(),
        Token::Colon => ":".into(),
        Token::Semicolon => ";".into(),
        Token::Comma => ",".into(),
        Token::IncludeMatch => "~=".into(),
        Token::DashMatch => "|=".into(),
        Token::PrefixMatch => "^=".into(),
        Token::SuffixMatch => "$=".into(),
        Token::SubstringMatch => "*=".into(),
        Token::Column => "||".into(),
        Token::CDO => "<!--".into(),
        Token::CDC => "-->".into(),
        Token::Function(f) => format!("{}(", escape_css_ident(&f)),
        Token::ParenthesisBlock => "(".into(),
        Token::SquareBracketBlock => "[".into(),
        Token::CurlyBracketBlock => "{".into(),
        Token::BadUrl(url) => format!("url('{}", escape_css_string(&url)).into(),
        Token::BadString(s) => format!("'{}", escape_css_string(&s)).into(),
        Token::CloseParenthesis => "unmatched close parenthesis".into(),
        Token::CloseSquareBracket => "unmatched close square bracket".into(),
        Token::CloseCurlyBracket => "unmatched close curly bracket".into(),
    }
}

enum Action {
    Nothing,
    Skip,
    Drop,
}

trait ErrorHelpers<'a> {
    fn error_data(self) -> (CowRcStr<'a>, ParseError<'a>);
    fn error_params(self) -> (ErrorString<'a>, Option<ErrorString<'a>>);
    fn to_gecko_message(&self) -> (Option<&'static [u8]>, &'static [u8], Action);
}

fn extract_error_param<'a>(err: ParseError<'a>) -> Option<ErrorString<'a>> {
    Some(match err {
        CssParseError::Basic(BasicParseError::UnexpectedToken(t)) =>
            ErrorString::UnexpectedToken(t),

        CssParseError::Basic(BasicParseError::AtRuleInvalid(i)) =>
            ErrorString::Snippet(format!("@{}", escape_css_ident(&i)).into()),

        CssParseError::Custom(SelectorParseError::Custom(
            StyleParseError::PropertyDeclaration(
                PropertyDeclarationParseError::InvalidValue(property, None)))) =>
            ErrorString::Snippet(property),

        CssParseError::Custom(SelectorParseError::UnexpectedIdent(ident)) =>
            ErrorString::Ident(ident),

        CssParseError::Custom(SelectorParseError::ExpectedNamespace(namespace)) =>
            ErrorString::Ident(namespace),

        CssParseError::Custom(SelectorParseError::Custom(
            StyleParseError::PropertyDeclaration(
                PropertyDeclarationParseError::UnknownProperty(property)))) =>
            ErrorString::Ident(property),

        CssParseError::Custom(SelectorParseError::Custom(
            StyleParseError::UnexpectedTokenWithinNamespace(token))) =>
            ErrorString::UnexpectedToken(token),

        _ => return None,
    })
}

fn extract_value_error_param<'a>(err: ValueParseError<'a>) -> ErrorString<'a> {
    match err {
        ValueParseError::InvalidColor(t) => ErrorString::UnexpectedToken(t),
    }
}

/// If an error parameter is present in the given error, return it. Additionally return
/// a second parameter if it exists, for use in the prefix for the eventual error message.
fn extract_error_params<'a>(err: ParseError<'a>) -> Option<(ErrorString<'a>, Option<ErrorString<'a>>)> {
    match err {
        CssParseError::Custom(SelectorParseError::Custom(
            StyleParseError::PropertyDeclaration(
                PropertyDeclarationParseError::InvalidValue(property, Some(e))))) =>
            Some((ErrorString::Snippet(property.into()), Some(extract_value_error_param(e)))),

        err => extract_error_param(err).map(|e| (e, None)),
    }
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

    fn error_params(self) -> (ErrorString<'a>, Option<ErrorString<'a>>) {
        let (s, error) = self.error_data();
        extract_error_params(error).unwrap_or((ErrorString::Snippet(s), None))
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
                _, CssParseError::Custom(SelectorParseError::ExpectedNamespace(_))) =>
                (b"PEUnknownNamespacePrefix\0", Action::Nothing),
            ContextualParseError::InvalidRule(
                _, CssParseError::Custom(SelectorParseError::Custom(
                StyleParseError::UnexpectedTokenWithinNamespace(_)))) =>
                (b"PEAtNSUnexpected\0", Action::Nothing),
            ContextualParseError::InvalidRule(..) =>
                (b"PEBadSelectorRSIgnored\0", Action::Nothing),
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
    fn report_error<'a>(&self,
                        input: &mut Parser,
                        position: SourcePosition,
                        error: ContextualParseError<'a>,
                        _url: &UrlExtraData,
                        line_number_offset: u64) {
        let location = input.source_location(position);
        let line_number = location.line + line_number_offset as u32;

        let (pre, name, action) = error.to_gecko_message();
        let suffix = match action {
            Action::Nothing => ptr::null(),
            Action::Skip => b"PEDeclSkipped\0".as_ptr(),
            Action::Drop => b"PEDeclDropped\0".as_ptr(),
        };
        let (param, pre_param) = error.error_params();
        let param = param.into_str();
        let pre_param = pre_param.map(|p| p.into_str());
        let pre_param_ptr = pre_param.as_ref().map_or(ptr::null(), |p| p.as_ptr());
        // The CSS source text is unused and will be removed in bug 1381188.
        let source = "";
        unsafe {
            Gecko_ReportUnexpectedCSSError(self.0,
                                           name.as_ptr() as *const _,
                                           param.as_ptr() as *const _,
                                           param.len() as u32,
                                           pre.map_or(ptr::null(), |p| p.as_ptr()) as *const _,
                                           pre_param_ptr as *const _,
                                           pre_param.as_ref().map_or(0, |p| p.len()) as u32,
                                           suffix as *const _,
                                           source.as_ptr() as *const _,
                                           source.len() as u32,
                                           line_number as u32,
                                           location.column as u32);
        }
    }
}
