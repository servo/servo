/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Parsing of the stylesheet contents.

use {Namespace, Prefix};
use counter_style::{parse_counter_style_body, parse_counter_style_name};
use cssparser::{AtRuleParser, AtRuleType, Parser, QualifiedRuleParser, RuleListParser};
use cssparser::{CompactCowStr, SourceLocation, BasicParseError};
use error_reporting::ContextualParseError;
use font_face::parse_font_face_block;
use media_queries::{parse_media_query_list, MediaList};
use parser::{Parse, ParserContext, log_css_error};
use properties::parse_property_declaration_list;
use selector_parser::{SelectorImpl, SelectorParser};
use selectors::SelectorList;
use selectors::parser::SelectorParseError;
use shared_lock::{Locked, SharedRwLock};
use std::borrow::Cow;
use str::starts_with_ignore_ascii_case;
use style_traits::{StyleParseError, ParseError};
use stylearc::Arc;
use stylesheets::{CssRule, CssRules, CssRuleType, Origin, StylesheetLoader};
use stylesheets::{DocumentRule, KeyframesRule, MediaRule, NamespaceRule, PageRule};
use stylesheets::{StyleRule, SupportsRule, ViewportRule};
use stylesheets::document_rule::DocumentCondition;
use stylesheets::keyframes_rule::parse_keyframe_list;
use stylesheets::stylesheet::Namespaces;
use stylesheets::supports_rule::SupportsCondition;
use stylesheets::viewport_rule;
use values::CustomIdent;
use values::KeyframesName;
use values::specified::url::SpecifiedUrl;

/// The parser for the top-level rules in a stylesheet.
pub struct TopLevelRuleParser<'a> {
    /// The origin of the stylesheet we're parsing.
    pub stylesheet_origin: Origin,
    /// A reference to the lock we need to use to create rules.
    pub shared_lock: &'a SharedRwLock,
    /// A reference to a stylesheet loader if applicable, for `@import` rules.
    pub loader: Option<&'a StylesheetLoader>,
    /// The parser context. This initially won't contain any namespaces, but
    /// will be populated after parsing namespace rules, if any.
    pub context: ParserContext<'a>,
    /// The current state of the parser.
    pub state: State,
    /// The namespace map we use for parsing. Needs to start as `Some()`, and
    /// will be taken out after parsing namespace rules, and that reference will
    /// be moved to `ParserContext`.
    pub namespaces: Option<&'a mut Namespaces>,
}

impl<'b> TopLevelRuleParser<'b> {
    fn nested<'a: 'b>(&'a self) -> NestedRuleParser<'a, 'b> {
        NestedRuleParser {
            stylesheet_origin: self.stylesheet_origin,
            shared_lock: self.shared_lock,
            context: &self.context,
        }
    }

    /// Returns the associated parser context with this rule parser.
    pub fn context(&self) -> &ParserContext {
        &self.context
    }

    /// Returns the current state of the parser.
    pub fn state(&self) -> State {
        self.state
    }
}

/// The current state of the parser.
#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone)]
pub enum State {
    /// We haven't started parsing rules.
    Start = 1,
    /// We're parsing `@import` rules.
    Imports = 2,
    /// We're parsing `@namespace` rules.
    Namespaces = 3,
    /// We're parsing the main body of the stylesheet.
    Body = 4,
    /// We've found an invalid state (as, a namespace rule after style rules),
    /// and the rest of the stylesheet should be ignored.
    Invalid = 5,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
/// Vendor prefix.
pub enum VendorPrefix {
    /// -moz prefix.
    Moz,
    /// -webkit prefix.
    WebKit,
}

/// A rule prelude for a given at-rule.
pub enum AtRulePrelude {
    /// A @font-face rule prelude.
    FontFace(SourceLocation),
    /// A @counter-style rule prelude, with its counter style name.
    CounterStyle(CustomIdent),
    /// A @media rule prelude, with its media queries.
    Media(Arc<Locked<MediaList>>, SourceLocation),
    /// An @supports rule, with its conditional
    Supports(SupportsCondition, SourceLocation),
    /// A @viewport rule prelude.
    Viewport,
    /// A @keyframes rule, with its animation name and vendor prefix if exists.
    Keyframes(KeyframesName, Option<VendorPrefix>, SourceLocation),
    /// A @page rule prelude.
    Page(SourceLocation),
    /// A @document rule, with its conditional.
    Document(DocumentCondition, SourceLocation),
}


#[cfg(feature = "gecko")]
fn register_namespace(ns: &Namespace) -> Result<i32, ()> {
    use gecko_bindings::bindings;
    let id = unsafe { bindings::Gecko_RegisterNamespace(ns.0.as_ptr()) };
    if id == -1 {
        Err(())
    } else {
        Ok(id)
    }
}

#[cfg(feature = "servo")]
fn register_namespace(_: &Namespace) -> Result<(), ()> {
    Ok(()) // servo doesn't use namespace ids
}

impl<'a, 'i> AtRuleParser<'i> for TopLevelRuleParser<'a> {
    type Prelude = AtRulePrelude;
    type AtRule = CssRule;
    type Error = SelectorParseError<'i, StyleParseError<'i>>;

    fn parse_prelude<'t>(
        &mut self,
        name: CompactCowStr<'i>,
        input: &mut Parser<'i, 't>
    ) -> Result<AtRuleType<AtRulePrelude, CssRule>, ParseError<'i>> {
        let location = get_location_with_offset(input.current_source_location(),
                                                self.context.line_number_offset);
        match_ignore_ascii_case! { &*name,
            "import" => {
                if self.state > State::Imports {
                    self.state = State::Invalid;
                    // "@import must be before any rule but @charset"
                    return Err(StyleParseError::UnexpectedImportRule.into())
                }

                self.state = State::Imports;
                let url_string = input.expect_url_or_string()?.into_owned();
                let specified_url = SpecifiedUrl::parse_from_string(url_string, &self.context)?;

                let media = parse_media_query_list(&self.context, input);
                let media = Arc::new(self.shared_lock.wrap(media));

                let loader =
                    self.loader.expect("Expected a stylesheet loader for @import");

                let import_rule = loader.request_stylesheet(
                    specified_url,
                    location,
                    &self.context,
                    &self.shared_lock,
                    media,
                );

                return Ok(AtRuleType::WithoutBlock(CssRule::Import(import_rule)))
            },
            "namespace" => {
                if self.state > State::Namespaces {
                    self.state = State::Invalid;
                    // "@namespace must be before any rule but @charset and @import"
                    return Err(StyleParseError::UnexpectedNamespaceRule.into())
                }
                self.state = State::Namespaces;

                let prefix_result = input.try(|input| input.expect_ident());
                let maybe_namespace = match input.expect_url_or_string() {
                    Ok(url_or_string) => url_or_string,
                    Err(BasicParseError::UnexpectedToken(t)) =>
                        return Err(StyleParseError::UnexpectedTokenWithinNamespace(t).into()),
                    Err(e) => return Err(e.into()),
                };
                let url = Namespace::from(Cow::from(maybe_namespace));

                let id = register_namespace(&url)
                    .map_err(|()| StyleParseError::UnspecifiedError)?;

                let mut namespaces = self.namespaces.as_mut().unwrap();

                let opt_prefix = if let Ok(prefix) = prefix_result {
                    let prefix = Prefix::from(Cow::from(prefix));
                    namespaces
                        .prefixes
                        .insert(prefix.clone(), (url.clone(), id));
                    Some(prefix)
                } else {
                    namespaces.default = Some((url.clone(), id));
                    None
                };

                return Ok(AtRuleType::WithoutBlock(CssRule::Namespace(Arc::new(
                    self.shared_lock.wrap(NamespaceRule {
                        prefix: opt_prefix,
                        url: url,
                        source_location: location,
                    })
                ))))
            },
            // @charset is removed by rust-cssparser if it’s the first rule in the stylesheet
            // anything left is invalid.
            "charset" => return Err(StyleParseError::UnexpectedCharsetRule.into()),
            _ => {}
        }
        // Don't allow starting with an invalid state
        if self.state > State::Body {
            self.state = State::Invalid;
            return Err(StyleParseError::UnspecifiedError.into());
        }
        self.state = State::Body;

        // "Freeze" the namespace map (no more namespace rules can be parsed
        // after this point), and stick it in the context.
        if self.namespaces.is_some() {
            let namespaces = &*self.namespaces.take().unwrap();
            self.context.namespaces = Some(namespaces);
        }
        AtRuleParser::parse_prelude(&mut self.nested(), name, input)
    }

    #[inline]
    fn parse_block<'t>(&mut self, prelude: AtRulePrelude, input: &mut Parser<'i, 't>)
                       -> Result<CssRule, ParseError<'i>> {
        AtRuleParser::parse_block(&mut self.nested(), prelude, input)
    }
}

pub struct QualifiedRuleParserPrelude {
    selectors: SelectorList<SelectorImpl>,
    source_location: SourceLocation,
}

impl<'a, 'i> QualifiedRuleParser<'i> for TopLevelRuleParser<'a> {
    type Prelude = QualifiedRuleParserPrelude;
    type QualifiedRule = CssRule;
    type Error = SelectorParseError<'i, StyleParseError<'i>>;

    #[inline]
    fn parse_prelude<'t>(&mut self, input: &mut Parser<'i, 't>)
                         -> Result<QualifiedRuleParserPrelude, ParseError<'i>> {
        self.state = State::Body;

        // "Freeze" the namespace map (no more namespace rules can be parsed
        // after this point), and stick it in the context.
        if self.namespaces.is_some() {
            let namespaces = &*self.namespaces.take().unwrap();
            self.context.namespaces = Some(namespaces);
        }

        QualifiedRuleParser::parse_prelude(&mut self.nested(), input)
    }

    #[inline]
    fn parse_block<'t>(
        &mut self,
        prelude: QualifiedRuleParserPrelude,
        input: &mut Parser<'i, 't>
    ) -> Result<CssRule, ParseError<'i>> {
        QualifiedRuleParser::parse_block(&mut self.nested(), prelude, input)
    }
}

#[derive(Clone)]  // shallow, relatively cheap .clone
struct NestedRuleParser<'a, 'b: 'a> {
    stylesheet_origin: Origin,
    shared_lock: &'a SharedRwLock,
    context: &'a ParserContext<'b>,
}

impl<'a, 'b> NestedRuleParser<'a, 'b> {
    fn parse_nested_rules(
        &mut self,
        input: &mut Parser,
        rule_type: CssRuleType
    ) -> Arc<Locked<CssRules>> {
        let context = ParserContext::new_with_rule_type(self.context, Some(rule_type));

        let nested_parser = NestedRuleParser {
            stylesheet_origin: self.stylesheet_origin,
            shared_lock: self.shared_lock,
            context: &context,
        };

        let mut iter = RuleListParser::new_for_nested_rule(input, nested_parser);
        let mut rules = Vec::new();
        while let Some(result) = iter.next() {
            match result {
                Ok(rule) => rules.push(rule),
                Err(err) => {
                    let pos = err.span.start;
                    let error = ContextualParseError::UnsupportedRule(
                        iter.input.slice(err.span), err.error);
                    log_css_error(iter.input, pos, error, self.context);
                }
            }
        }
        CssRules::new(rules, self.shared_lock)
    }
}

impl<'a, 'b, 'i> AtRuleParser<'i> for NestedRuleParser<'a, 'b> {
    type Prelude = AtRulePrelude;
    type AtRule = CssRule;
    type Error = SelectorParseError<'i, StyleParseError<'i>>;

    fn parse_prelude<'t>(
        &mut self,
        name: CompactCowStr<'i>,
        input: &mut Parser<'i, 't>
    ) -> Result<AtRuleType<AtRulePrelude, CssRule>, ParseError<'i>> {
        let location =
            get_location_with_offset(
                input.current_source_location(),
                self.context.line_number_offset
            );

        match_ignore_ascii_case! { &*name,
            "media" => {
                let media_queries = parse_media_query_list(self.context, input);
                let arc = Arc::new(self.shared_lock.wrap(media_queries));
                Ok(AtRuleType::WithBlock(AtRulePrelude::Media(arc, location)))
            },
            "supports" => {
                let cond = SupportsCondition::parse(input)?;
                Ok(AtRuleType::WithBlock(AtRulePrelude::Supports(cond, location)))
            },
            "font-face" => {
                Ok(AtRuleType::WithBlock(AtRulePrelude::FontFace(location)))
            },
            "counter-style" => {
                if !cfg!(feature = "gecko") {
                    // Support for this rule is not fully implemented in Servo yet.
                    return Err(StyleParseError::UnsupportedAtRule(name.clone()).into())
                }
                let name = parse_counter_style_name(input)?;
                // ASCII-case-insensitive matches for "decimal" and "disc".
                // The name is already lower-cased by `parse_counter_style_name`
                // so we can use == here.
                if name.0 == atom!("decimal") || name.0 == atom!("disc") {
                    return Err(StyleParseError::UnspecifiedError.into())
                }
                Ok(AtRuleType::WithBlock(AtRulePrelude::CounterStyle(name)))
            },
            "viewport" => {
                if viewport_rule::enabled() {
                    Ok(AtRuleType::WithBlock(AtRulePrelude::Viewport))
                } else {
                    Err(StyleParseError::UnsupportedAtRule(name.clone()).into())
                }
            },
            "keyframes" | "-webkit-keyframes" | "-moz-keyframes" => {
                let prefix = if starts_with_ignore_ascii_case(&*name, "-webkit-") {
                    Some(VendorPrefix::WebKit)
                } else if starts_with_ignore_ascii_case(&*name, "-moz-") {
                    Some(VendorPrefix::Moz)
                } else {
                    None
                };
                if cfg!(feature = "servo") &&
                   prefix.as_ref().map_or(false, |p| matches!(*p, VendorPrefix::Moz)) {
                    // Servo should not support @-moz-keyframes.
                    return Err(StyleParseError::UnsupportedAtRule(name.clone()).into())
                }
                let name = KeyframesName::parse(self.context, input)?;

                Ok(AtRuleType::WithBlock(AtRulePrelude::Keyframes(name, prefix, location)))
            },
            "page" => {
                if cfg!(feature = "gecko") {
                    Ok(AtRuleType::WithBlock(AtRulePrelude::Page(location)))
                } else {
                    Err(StyleParseError::UnsupportedAtRule(name.clone()).into())
                }
            },
            "-moz-document" => {
                if cfg!(feature = "gecko") {
                    let cond = DocumentCondition::parse(self.context, input)?;
                    Ok(AtRuleType::WithBlock(AtRulePrelude::Document(cond, location)))
                } else {
                    Err(StyleParseError::UnsupportedAtRule(name.clone()).into())
                }
            },
            _ => Err(StyleParseError::UnsupportedAtRule(name.clone()).into())
        }
    }

    fn parse_block<'t>(
        &mut self,
        prelude: AtRulePrelude,
        input: &mut Parser<'i, 't>
    ) -> Result<CssRule, ParseError<'i>> {
        match prelude {
            AtRulePrelude::FontFace(location) => {
                let context = ParserContext::new_with_rule_type(self.context, Some(CssRuleType::FontFace));
                Ok(CssRule::FontFace(Arc::new(self.shared_lock.wrap(
                   parse_font_face_block(&context, input, location).into()))))
            }
            AtRulePrelude::CounterStyle(name) => {
                let context = ParserContext::new_with_rule_type(self.context, Some(CssRuleType::CounterStyle));
                Ok(CssRule::CounterStyle(Arc::new(self.shared_lock.wrap(
                   parse_counter_style_body(name, &context, input)?.into()))))
            }
            AtRulePrelude::Media(media_queries, location) => {
                Ok(CssRule::Media(Arc::new(self.shared_lock.wrap(MediaRule {
                    media_queries: media_queries,
                    rules: self.parse_nested_rules(input, CssRuleType::Media),
                    source_location: location,
                }))))
            }
            AtRulePrelude::Supports(cond, location) => {
                let enabled = cond.eval(self.context);
                Ok(CssRule::Supports(Arc::new(self.shared_lock.wrap(SupportsRule {
                    condition: cond,
                    rules: self.parse_nested_rules(input, CssRuleType::Supports),
                    enabled: enabled,
                    source_location: location,
                }))))
            }
            AtRulePrelude::Viewport => {
                let context = ParserContext::new_with_rule_type(self.context, Some(CssRuleType::Viewport));
                Ok(CssRule::Viewport(Arc::new(self.shared_lock.wrap(
                   ViewportRule::parse(&context, input)?))))
            }
            AtRulePrelude::Keyframes(name, prefix, location) => {
                let context = ParserContext::new_with_rule_type(self.context, Some(CssRuleType::Keyframes));
                Ok(CssRule::Keyframes(Arc::new(self.shared_lock.wrap(KeyframesRule {
                    name: name,
                    keyframes: parse_keyframe_list(&context, input, self.shared_lock),
                    vendor_prefix: prefix,
                    source_location: location,
                }))))
            }
            AtRulePrelude::Page(location) => {
                let context = ParserContext::new_with_rule_type(self.context, Some(CssRuleType::Page));
                let declarations = parse_property_declaration_list(&context, input);
                Ok(CssRule::Page(Arc::new(self.shared_lock.wrap(PageRule {
                    block: Arc::new(self.shared_lock.wrap(declarations)),
                    source_location: location,
                }))))
            }
            AtRulePrelude::Document(cond, location) => {
                if cfg!(feature = "gecko") {
                    Ok(CssRule::Document(Arc::new(self.shared_lock.wrap(DocumentRule {
                        condition: cond,
                        rules: self.parse_nested_rules(input, CssRuleType::Document),
                        source_location: location,
                    }))))
                } else {
                    unreachable!()
                }
            }
        }
    }
}

impl<'a, 'b, 'i> QualifiedRuleParser<'i> for NestedRuleParser<'a, 'b> {
    type Prelude = QualifiedRuleParserPrelude;
    type QualifiedRule = CssRule;
    type Error = SelectorParseError<'i, StyleParseError<'i>>;

    fn parse_prelude<'t>(&mut self, input: &mut Parser<'i, 't>)
                         -> Result<QualifiedRuleParserPrelude, ParseError<'i>> {
        let selector_parser = SelectorParser {
            stylesheet_origin: self.stylesheet_origin,
            namespaces: self.context.namespaces.unwrap(),
        };

        let location = get_location_with_offset(input.current_source_location(),
                                                self.context.line_number_offset);
        let selectors = SelectorList::parse(&selector_parser, input)?;

        Ok(QualifiedRuleParserPrelude {
            selectors: selectors,
            source_location: location,
        })
    }

    fn parse_block<'t>(
        &mut self,
        prelude: QualifiedRuleParserPrelude,
        input: &mut Parser<'i, 't>
    ) -> Result<CssRule, ParseError<'i>> {
        let context = ParserContext::new_with_rule_type(self.context, Some(CssRuleType::Style));
        let declarations = parse_property_declaration_list(&context, input);
        Ok(CssRule::Style(Arc::new(self.shared_lock.wrap(StyleRule {
            selectors: prelude.selectors,
            block: Arc::new(self.shared_lock.wrap(declarations)),
            source_location: prelude.source_location,
        }))))
    }
}

/// Calculates the location of a rule's source given an offset.
fn get_location_with_offset(
    location: SourceLocation,
    offset: u64
) -> SourceLocation {
    SourceLocation {
        line: location.line + offset as u32,
        column: location.column,
    }
}
