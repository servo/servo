/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Wrapper around Gecko's CSS error reporting mechanism.

#![allow(unsafe_code)]

use cssparser::{Parser, SourcePosition, ParseError as CssParseError, Token, BasicParseError};
use cssparser::CompactCowStr;
use selectors::parser::SelectorParseError;
use style::error_reporting::{ParseErrorReporter, ContextualParseError};
use style::gecko_bindings::bindings::{Gecko_CreateCSSErrorReporter, Gecko_DestroyCSSErrorReporter};
use style::gecko_bindings::bindings::Gecko_ReportUnexpectedCSSError;
use style::gecko_bindings::structs::{Loader, ServoStyleSheet, nsIURI};
use style::gecko_bindings::structs::ErrorReporter as GeckoErrorReporter;
use style::gecko_bindings::structs::URLExtraData as RawUrlExtraData;
use style::gecko_bindings::sugar::refptr::RefPtr;
use style::stylesheets::UrlExtraData;
use style_traits::{ParseError, StyleParseError, PropertyDeclarationParseError};

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
    Snippet(CompactCowStr<'a>),
    Ident(CompactCowStr<'a>),
    UnexpectedToken(Token<'a>),
}

impl<'a> ErrorString<'a> {
    fn into_str(self) -> String {
        match self {
            ErrorString::Snippet(s) => s.into_owned(),
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

trait ErrorHelpers<'a> {
    fn error_data(self) -> (CompactCowStr<'a>, ParseError<'a>);
    fn error_param(self) -> ErrorString<'a>;
    fn to_gecko_message(&self) -> &'static [u8];
}

impl<'a> ErrorHelpers<'a> for ContextualParseError<'a> {
    fn error_data(self) -> (CompactCowStr<'a>, ParseError<'a>) {
        match self {
            ContextualParseError::UnsupportedPropertyDeclaration(s, err) |
            ContextualParseError::UnsupportedFontFaceDescriptor(s, err) |
            ContextualParseError::InvalidKeyframeRule(s, err) |
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

    fn error_param(self) -> ErrorString<'a> {
        match self.error_data() {
            (_, CssParseError::Basic(BasicParseError::UnexpectedToken(t))) =>
                ErrorString::UnexpectedToken(t),

            (_, CssParseError::Basic(BasicParseError::AtRuleInvalid(i))) =>
                ErrorString::Snippet(format!("@{}", escape_css_ident(&i)).into()),

            (_, CssParseError::Custom(SelectorParseError::Custom(
                StyleParseError::PropertyDeclaration(
                    PropertyDeclarationParseError::InvalidValue(property))))) =>
                ErrorString::Snippet(property.into()),

            (_, CssParseError::Custom(SelectorParseError::UnexpectedIdent(ident))) =>
                ErrorString::Ident(ident),

            (_, CssParseError::Custom(SelectorParseError::ExpectedNamespace(namespace))) =>
                ErrorString::Ident(namespace),

            (_, CssParseError::Custom(SelectorParseError::Custom(
                StyleParseError::UnknownProperty(property)))) =>
                ErrorString::Ident(property),

            (_, CssParseError::Custom(SelectorParseError::Custom(
                StyleParseError::UnexpectedTokenWithinNamespace(token)))) =>
                ErrorString::UnexpectedToken(token),

            (s, _)  => ErrorString::Snippet(s)
        }
    }

    fn to_gecko_message(&self) -> &'static [u8] {
        match *self {
            ContextualParseError::UnsupportedPropertyDeclaration(
                _, CssParseError::Basic(BasicParseError::UnexpectedToken(_))) |
            ContextualParseError::UnsupportedPropertyDeclaration(
                _, CssParseError::Basic(BasicParseError::AtRuleInvalid(_))) =>
                b"PEParseDeclarationDeclExpected\0",
            ContextualParseError::UnsupportedPropertyDeclaration(
                _, CssParseError::Custom(SelectorParseError::Custom(
                    StyleParseError::PropertyDeclaration(
                        PropertyDeclarationParseError::InvalidValue(_))))) =>
                b"PEValueParsingError\0",
            ContextualParseError::UnsupportedPropertyDeclaration(..) =>
                b"PEUnknownProperty\0",
            ContextualParseError::UnsupportedFontFaceDescriptor(..) =>
                b"PEUnknwnFontDesc\0",
            ContextualParseError::InvalidKeyframeRule(..) =>
                b"PEKeyframeBadName\0",
            ContextualParseError::UnsupportedKeyframePropertyDeclaration(..) =>
                b"PEBadSelectorKeyframeRuleIgnored\0",
            ContextualParseError::InvalidRule(
                _, CssParseError::Custom(SelectorParseError::ExpectedNamespace(_))) =>
                b"PEUnknownNamespacePrefix\0",
            ContextualParseError::InvalidRule(
                _, CssParseError::Custom(SelectorParseError::Custom(
                StyleParseError::UnexpectedTokenWithinNamespace(_)))) =>
                b"PEAtNSUnexpected\0",
            ContextualParseError::InvalidRule(..) =>
                b"PEBadSelectorRSIgnored\0",
            ContextualParseError::UnsupportedRule(..) =>
                b"PEDeclDropped\0",
            ContextualParseError::UnsupportedViewportDescriptorDeclaration(..) |
            ContextualParseError::UnsupportedCounterStyleDescriptorDeclaration(..) |
            ContextualParseError::InvalidCounterStyleWithoutSymbols(..) |
            ContextualParseError::InvalidCounterStyleNotEnoughSymbols(..) |
            ContextualParseError::InvalidCounterStyleWithoutAdditiveSymbols |
            ContextualParseError::InvalidCounterStyleExtendsWithSymbols |
            ContextualParseError::InvalidCounterStyleExtendsWithAdditiveSymbols =>
                b"PEUnknownAtRule\0",
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

        let name = error.to_gecko_message();
        let param = error.error_param().into_str();
        // The CSS source text is unused and will be removed in bug 1381188.
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
