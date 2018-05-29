/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Parsing of the stylesheet contents.

use {Namespace, Prefix};
use counter_style::{parse_counter_style_body, parse_counter_style_name_definition};
use cssparser::{AtRuleParser, AtRuleType, Parser, QualifiedRuleParser, RuleListParser};
use cssparser::{BasicParseError, BasicParseErrorKind, CowRcStr, SourceLocation};
use error_reporting::{ContextualParseError, ParseErrorReporter};
use font_face::parse_font_face_block;
use media_queries::{parse_media_query_list, MediaList};
use parser::{Parse, ParserContext, ParserErrorContext};
use properties::parse_property_declaration_list;
use selector_parser::{SelectorImpl, SelectorParser};
use selectors::SelectorList;
use servo_arc::Arc;
use shared_lock::{Locked, SharedRwLock};
use str::starts_with_ignore_ascii_case;
use style_traits::{ParseError, StyleParseErrorKind};
use stylesheets::{CssRule, CssRuleType, CssRules, Origin, RulesMutateError, StylesheetLoader};
use stylesheets::{DocumentRule, FontFeatureValuesRule, KeyframesRule, MediaRule};
use stylesheets::{NamespaceRule, PageRule, StyleRule, SupportsRule, ViewportRule};
use stylesheets::document_rule::DocumentCondition;
use stylesheets::font_feature_values_rule::parse_family_name_list;
use stylesheets::keyframes_rule::parse_keyframe_list;
use stylesheets::stylesheet::Namespaces;
use stylesheets::supports_rule::SupportsCondition;
use stylesheets::viewport_rule;
use values::{CssUrl, CustomIdent, KeyframesName};
use values::computed::font::FamilyName;

/// The information we need particularly to do CSSOM insertRule stuff.
pub struct InsertRuleContext<'a> {
    /// The rule list we're about to insert into.
    pub rule_list: &'a [CssRule],
    /// The index we're about to get inserted at.
    pub index: usize,
}

/// The parser for the top-level rules in a stylesheet.
pub struct TopLevelRuleParser<'a, R: 'a> {
    /// The origin of the stylesheet we're parsing.
    pub stylesheet_origin: Origin,
    /// A reference to the lock we need to use to create rules.
    pub shared_lock: &'a SharedRwLock,
    /// A reference to a stylesheet loader if applicable, for `@import` rules.
    pub loader: Option<&'a StylesheetLoader>,
    /// The top-level parser context.
    ///
    /// This won't contain any namespaces, and only nested parsers created with
    /// `ParserContext::new_with_rule_type` will.
    pub context: ParserContext<'a>,
    /// The context required for reporting parse errors.
    pub error_context: ParserErrorContext<'a, R>,
    /// The current state of the parser.
    pub state: State,
    /// Whether we have tried to parse was invalid due to being in the wrong
    /// place (e.g. an @import rule was found while in the `Body` state). Reset
    /// to `false` when `take_had_hierarchy_error` is called.
    pub dom_error: Option<RulesMutateError>,
    /// The namespace map we use for parsing. Needs to start as `Some()`, and
    /// will be taken out after parsing namespace rules, and that reference will
    /// be moved to `ParserContext`.
    pub namespaces: &'a mut Namespaces,
    /// The info we need insert a rule in a list.
    pub insert_rule_context: Option<InsertRuleContext<'a>>,
}

impl<'b, R> TopLevelRuleParser<'b, R> {
    fn nested<'a: 'b>(&'a self) -> NestedRuleParser<'a, 'b, R> {
        NestedRuleParser {
            stylesheet_origin: self.stylesheet_origin,
            shared_lock: self.shared_lock,
            context: &self.context,
            error_context: &self.error_context,
            namespaces: &self.namespaces,
        }
    }

    /// Returns the current state of the parser.
    pub fn state(&self) -> State {
        self.state
    }

    /// Checks whether we can parse a rule that would transition us to
    /// `new_state`.
    ///
    /// This is usually a simple branch, but we may need more bookkeeping if
    /// doing `insertRule` from CSSOM.
    fn check_state(&mut self, new_state: State) -> bool {
        if self.state > new_state {
            self.dom_error = Some(RulesMutateError::HierarchyRequest);
            return false;
        }

        let ctx = match self.insert_rule_context {
            Some(ref ctx) => ctx,
            None => return true,
        };

        let next_rule_state = match ctx.rule_list.get(ctx.index) {
            None => return true,
            Some(rule) => rule.rule_state(),
        };

        if new_state > next_rule_state {
            self.dom_error = Some(RulesMutateError::HierarchyRequest);
            return false;
        }

        // If there's anything that isn't a namespace rule (or import rule, but
        // we checked that already at the beginning), reject with a
        // StateError.
        if new_state == State::Namespaces &&
            ctx.rule_list[ctx.index..].iter().any(|r| !matches!(*r, CssRule::Namespace(..)))
        {
            self.dom_error = Some(RulesMutateError::InvalidState);
            return false;
        }

        true
    }
}

/// The current state of the parser.
#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub enum State {
    /// We haven't started parsing rules.
    Start = 1,
    /// We're parsing `@import` rules.
    Imports = 2,
    /// We're parsing `@namespace` rules.
    Namespaces = 3,
    /// We're parsing the main body of the stylesheet.
    Body = 4,
}

#[derive(Clone, Debug, MallocSizeOf)]
/// Vendor prefix.
pub enum VendorPrefix {
    /// -moz prefix.
    Moz,
    /// -webkit prefix.
    WebKit,
}

/// A rule prelude for at-rule with block.
pub enum AtRuleBlockPrelude {
    /// A @font-face rule prelude.
    FontFace(SourceLocation),
    /// A @font-feature-values rule prelude, with its FamilyName list.
    FontFeatureValues(Vec<FamilyName>, SourceLocation),
    /// A @counter-style rule prelude, with its counter style name.
    CounterStyle(CustomIdent, SourceLocation),
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

/// A rule prelude for at-rule without block.
pub enum AtRuleNonBlockPrelude {
    /// A @import rule prelude.
    Import(CssUrl, Arc<Locked<MediaList>>, SourceLocation),
    /// A @namespace rule prelude.
    Namespace(Option<Prefix>, Namespace, SourceLocation),
}

impl<'a, 'i, R: ParseErrorReporter> AtRuleParser<'i> for TopLevelRuleParser<'a, R> {
    type PreludeNoBlock = AtRuleNonBlockPrelude;
    type PreludeBlock = AtRuleBlockPrelude;
    type AtRule = CssRule;
    type Error = StyleParseErrorKind<'i>;

    fn parse_prelude<'t>(
        &mut self,
        name: CowRcStr<'i>,
        input: &mut Parser<'i, 't>,
    ) -> Result<AtRuleType<AtRuleNonBlockPrelude, AtRuleBlockPrelude>, ParseError<'i>> {
        let location = input.current_source_location();
        match_ignore_ascii_case! { &*name,
            "import" => {
                if !self.check_state(State::Imports) {
                    return Err(input.new_custom_error(StyleParseErrorKind::UnexpectedImportRule))
                }

                let url_string = input.expect_url_or_string()?.as_ref().to_owned();
                let url = CssUrl::parse_from_string(url_string, &self.context);

                let media = parse_media_query_list(&self.context, input,
                                                   self.error_context.error_reporter);
                let media = Arc::new(self.shared_lock.wrap(media));

                let prelude = AtRuleNonBlockPrelude::Import(url, media, location);
                return Ok(AtRuleType::WithoutBlock(prelude));
            },
            "namespace" => {
                if !self.check_state(State::Namespaces) {
                    return Err(input.new_custom_error(StyleParseErrorKind::UnexpectedNamespaceRule))
                }

                let prefix = input.try(|i| i.expect_ident_cloned())
                                  .map(|s| Prefix::from(s.as_ref())).ok();
                let maybe_namespace = match input.expect_url_or_string() {
                    Ok(url_or_string) => url_or_string,
                    Err(BasicParseError { kind: BasicParseErrorKind::UnexpectedToken(t), location }) => {
                        return Err(location.new_custom_error(StyleParseErrorKind::UnexpectedTokenWithinNamespace(t)))
                    }
                    Err(e) => return Err(e.into()),
                };
                let url = Namespace::from(maybe_namespace.as_ref());
                let prelude = AtRuleNonBlockPrelude::Namespace(prefix, url, location);
                return Ok(AtRuleType::WithoutBlock(prelude));
            },
            // @charset is removed by rust-cssparser if it’s the first rule in the stylesheet
            // anything left is invalid.
            "charset" => {
                self.dom_error = Some(RulesMutateError::HierarchyRequest);
                return Err(input.new_custom_error(StyleParseErrorKind::UnexpectedCharsetRule))
            }
            _ => {}
        }

        if !self.check_state(State::Body) {
            return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
        }

        AtRuleParser::parse_prelude(&mut self.nested(), name, input)
    }

    #[inline]
    fn parse_block<'t>(
        &mut self,
        prelude: AtRuleBlockPrelude,
        input: &mut Parser<'i, 't>,
    ) -> Result<CssRule, ParseError<'i>> {
        AtRuleParser::parse_block(&mut self.nested(), prelude, input).map(|rule| {
            self.state = State::Body;
            rule
        })
    }

    #[inline]
    fn rule_without_block(&mut self, prelude: AtRuleNonBlockPrelude) -> CssRule {
        match prelude {
            AtRuleNonBlockPrelude::Import(url, media, location) => {
                let loader = self.loader
                    .expect("Expected a stylesheet loader for @import");

                let import_rule = loader.request_stylesheet(
                    url,
                    location,
                    &self.context,
                    &self.shared_lock,
                    media,
                );

                self.state = State::Imports;
                CssRule::Import(import_rule)
            },
            AtRuleNonBlockPrelude::Namespace(prefix, url, source_location) => {
                let prefix = if let Some(prefix) = prefix {
                    self.namespaces.prefixes.insert(prefix.clone(), url.clone());
                    Some(prefix)
                } else {
                    self.namespaces.default = Some(url.clone());
                    None
                };

                self.state = State::Namespaces;
                CssRule::Namespace(Arc::new(self.shared_lock.wrap(NamespaceRule {
                    prefix,
                    url,
                    source_location,
                })))
            },
        }
    }
}

pub struct QualifiedRuleParserPrelude {
    selectors: SelectorList<SelectorImpl>,
    source_location: SourceLocation,
}

impl<'a, 'i, R: ParseErrorReporter> QualifiedRuleParser<'i> for TopLevelRuleParser<'a, R> {
    type Prelude = QualifiedRuleParserPrelude;
    type QualifiedRule = CssRule;
    type Error = StyleParseErrorKind<'i>;

    #[inline]
    fn parse_prelude<'t>(
        &mut self,
        input: &mut Parser<'i, 't>,
    ) -> Result<QualifiedRuleParserPrelude, ParseError<'i>> {
        if !self.check_state(State::Body) {
            return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
        }

        QualifiedRuleParser::parse_prelude(&mut self.nested(), input)
    }

    #[inline]
    fn parse_block<'t>(
        &mut self,
        prelude: QualifiedRuleParserPrelude,
        input: &mut Parser<'i, 't>,
    ) -> Result<CssRule, ParseError<'i>> {
        QualifiedRuleParser::parse_block(&mut self.nested(), prelude, input).map(|result| {
            self.state = State::Body;
            result
        })
    }
}

#[derive(Clone)] // shallow, relatively cheap .clone
struct NestedRuleParser<'a, 'b: 'a, R: 'b> {
    stylesheet_origin: Origin,
    shared_lock: &'a SharedRwLock,
    context: &'a ParserContext<'b>,
    error_context: &'a ParserErrorContext<'b, R>,
    namespaces: &'a Namespaces,
}

impl<'a, 'b, R: ParseErrorReporter> NestedRuleParser<'a, 'b, R> {
    fn parse_nested_rules(
        &mut self,
        input: &mut Parser,
        rule_type: CssRuleType,
    ) -> Arc<Locked<CssRules>> {
        let context = ParserContext::new_with_rule_type(self.context, rule_type, self.namespaces);

        let nested_parser = NestedRuleParser {
            stylesheet_origin: self.stylesheet_origin,
            shared_lock: self.shared_lock,
            context: &context,
            error_context: &self.error_context,
            namespaces: self.namespaces,
        };

        let mut iter = RuleListParser::new_for_nested_rule(input, nested_parser);
        let mut rules = Vec::new();
        while let Some(result) = iter.next() {
            match result {
                Ok(rule) => rules.push(rule),
                Err((error, slice)) => {
                    let location = error.location;
                    let error = ContextualParseError::InvalidRule(slice, error);
                    self.context
                        .log_css_error(self.error_context, location, error);
                },
            }
        }
        CssRules::new(rules, self.shared_lock)
    }
}

impl<'a, 'b, 'i, R: ParseErrorReporter> AtRuleParser<'i> for NestedRuleParser<'a, 'b, R> {
    type PreludeNoBlock = AtRuleNonBlockPrelude;
    type PreludeBlock = AtRuleBlockPrelude;
    type AtRule = CssRule;
    type Error = StyleParseErrorKind<'i>;

    fn parse_prelude<'t>(
        &mut self,
        name: CowRcStr<'i>,
        input: &mut Parser<'i, 't>,
    ) -> Result<AtRuleType<AtRuleNonBlockPrelude, AtRuleBlockPrelude>, ParseError<'i>> {
        let location = input.current_source_location();

        match_ignore_ascii_case! { &*name,
            "media" => {
                let media_queries = parse_media_query_list(self.context, input,
                                                           self.error_context.error_reporter);
                let arc = Arc::new(self.shared_lock.wrap(media_queries));
                Ok(AtRuleType::WithBlock(AtRuleBlockPrelude::Media(arc, location)))
            },
            "supports" => {
                let cond = SupportsCondition::parse(input)?;
                Ok(AtRuleType::WithBlock(AtRuleBlockPrelude::Supports(cond, location)))
            },
            "font-face" => {
                Ok(AtRuleType::WithBlock(AtRuleBlockPrelude::FontFace(location)))
            },
            "font-feature-values" => {
                if !cfg!(feature = "gecko") {
                    // Support for this rule is not fully implemented in Servo yet.
                    return Err(input.new_custom_error(StyleParseErrorKind::UnsupportedAtRule(name.clone())))
                }
                let family_names = parse_family_name_list(self.context, input)?;
                Ok(AtRuleType::WithBlock(AtRuleBlockPrelude::FontFeatureValues(family_names, location)))
            },
            "counter-style" => {
                if !cfg!(feature = "gecko") {
                    // Support for this rule is not fully implemented in Servo yet.
                    return Err(input.new_custom_error(StyleParseErrorKind::UnsupportedAtRule(name.clone())))
                }
                let name = parse_counter_style_name_definition(input)?;
                Ok(AtRuleType::WithBlock(AtRuleBlockPrelude::CounterStyle(name, location)))
            },
            "viewport" => {
                if viewport_rule::enabled() {
                    Ok(AtRuleType::WithBlock(AtRuleBlockPrelude::Viewport))
                } else {
                    Err(input.new_custom_error(StyleParseErrorKind::UnsupportedAtRule(name.clone())))
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
                    return Err(input.new_custom_error(StyleParseErrorKind::UnsupportedAtRule(name.clone())))
                }
                let name = KeyframesName::parse(self.context, input)?;

                Ok(AtRuleType::WithBlock(AtRuleBlockPrelude::Keyframes(name, prefix, location)))
            },
            "page" => {
                if cfg!(feature = "gecko") {
                    Ok(AtRuleType::WithBlock(AtRuleBlockPrelude::Page(location)))
                } else {
                    Err(input.new_custom_error(StyleParseErrorKind::UnsupportedAtRule(name.clone())))
                }
            },
            "-moz-document" => {
                if !cfg!(feature = "gecko") {
                    return Err(input.new_custom_error(
                        StyleParseErrorKind::UnsupportedAtRule(name.clone())
                    ))
                }

                let cond = DocumentCondition::parse(self.context, input)?;
                Ok(AtRuleType::WithBlock(AtRuleBlockPrelude::Document(cond, location)))
            },
            _ => Err(input.new_custom_error(StyleParseErrorKind::UnsupportedAtRule(name.clone())))
        }
    }

    fn parse_block<'t>(
        &mut self,
        prelude: AtRuleBlockPrelude,
        input: &mut Parser<'i, 't>,
    ) -> Result<CssRule, ParseError<'i>> {
        match prelude {
            AtRuleBlockPrelude::FontFace(location) => {
                let context = ParserContext::new_with_rule_type(
                    self.context,
                    CssRuleType::FontFace,
                    self.namespaces,
                );

                Ok(CssRule::FontFace(Arc::new(self.shared_lock.wrap(
                    parse_font_face_block(&context, self.error_context, input, location).into(),
                ))))
            },
            AtRuleBlockPrelude::FontFeatureValues(family_names, location) => {
                let context = ParserContext::new_with_rule_type(
                    self.context,
                    CssRuleType::FontFeatureValues,
                    self.namespaces,
                );

                Ok(CssRule::FontFeatureValues(Arc::new(self.shared_lock.wrap(
                    FontFeatureValuesRule::parse(
                        &context,
                        self.error_context,
                        input,
                        family_names,
                        location,
                    ),
                ))))
            },
            AtRuleBlockPrelude::CounterStyle(name, location) => {
                let context = ParserContext::new_with_rule_type(
                    self.context,
                    CssRuleType::CounterStyle,
                    self.namespaces,
                );

                Ok(CssRule::CounterStyle(Arc::new(
                    self.shared_lock.wrap(
                        parse_counter_style_body(
                            name,
                            &context,
                            self.error_context,
                            input,
                            location,
                        )?.into(),
                    ),
                )))
            },
            AtRuleBlockPrelude::Media(media_queries, source_location) => {
                Ok(CssRule::Media(Arc::new(self.shared_lock.wrap(MediaRule {
                    media_queries,
                    rules: self.parse_nested_rules(input, CssRuleType::Media),
                    source_location,
                }))))
            },
            AtRuleBlockPrelude::Supports(condition, source_location) => {
                let eval_context = ParserContext::new_with_rule_type(
                    self.context,
                    CssRuleType::Style,
                    self.namespaces,
                );

                let enabled = condition.eval(&eval_context);
                Ok(CssRule::Supports(Arc::new(self.shared_lock.wrap(
                    SupportsRule {
                        condition,
                        rules: self.parse_nested_rules(input, CssRuleType::Supports),
                        enabled,
                        source_location,
                    },
                ))))
            },
            AtRuleBlockPrelude::Viewport => {
                let context = ParserContext::new_with_rule_type(
                    self.context,
                    CssRuleType::Viewport,
                    self.namespaces,
                );

                Ok(CssRule::Viewport(Arc::new(self.shared_lock.wrap(
                    ViewportRule::parse(&context, self.error_context, input)?,
                ))))
            },
            AtRuleBlockPrelude::Keyframes(name, vendor_prefix, source_location) => {
                let context = ParserContext::new_with_rule_type(
                    self.context,
                    CssRuleType::Keyframes,
                    self.namespaces,
                );

                Ok(CssRule::Keyframes(Arc::new(self.shared_lock.wrap(
                    KeyframesRule {
                        name,
                        keyframes: parse_keyframe_list(
                            &context,
                            self.error_context,
                            input,
                            self.shared_lock,
                        ),
                        vendor_prefix,
                        source_location,
                    },
                ))))
            },
            AtRuleBlockPrelude::Page(source_location) => {
                let context = ParserContext::new_with_rule_type(
                    self.context,
                    CssRuleType::Page,
                    self.namespaces,
                );

                let declarations =
                    parse_property_declaration_list(&context, self.error_context, input);
                Ok(CssRule::Page(Arc::new(self.shared_lock.wrap(PageRule {
                    block: Arc::new(self.shared_lock.wrap(declarations)),
                    source_location,
                }))))
            },
            AtRuleBlockPrelude::Document(condition, source_location) => {
                if !cfg!(feature = "gecko") {
                    unreachable!()
                }
                Ok(CssRule::Document(Arc::new(self.shared_lock.wrap(
                    DocumentRule {
                        condition,
                        rules: self.parse_nested_rules(input, CssRuleType::Document),
                        source_location,
                    },
                ))))
            },
        }
    }
}

impl<'a, 'b, 'i, R: ParseErrorReporter> QualifiedRuleParser<'i> for NestedRuleParser<'a, 'b, R> {
    type Prelude = QualifiedRuleParserPrelude;
    type QualifiedRule = CssRule;
    type Error = StyleParseErrorKind<'i>;

    fn parse_prelude<'t>(
        &mut self,
        input: &mut Parser<'i, 't>,
    ) -> Result<QualifiedRuleParserPrelude, ParseError<'i>> {
        let selector_parser = SelectorParser {
            stylesheet_origin: self.stylesheet_origin,
            namespaces: self.namespaces,
            url_data: Some(self.context.url_data),
        };

        let source_location = input.current_source_location();
        let selectors = SelectorList::parse(&selector_parser, input)?;

        Ok(QualifiedRuleParserPrelude { selectors, source_location, })
    }

    fn parse_block<'t>(
        &mut self,
        prelude: QualifiedRuleParserPrelude,
        input: &mut Parser<'i, 't>,
    ) -> Result<CssRule, ParseError<'i>> {
        let context =
            ParserContext::new_with_rule_type(self.context, CssRuleType::Style, self.namespaces);

        let declarations = parse_property_declaration_list(&context, self.error_context, input);
        Ok(CssRule::Style(Arc::new(self.shared_lock.wrap(StyleRule {
            selectors: prelude.selectors,
            block: Arc::new(self.shared_lock.wrap(declarations)),
            source_location: prelude.source_location,
        }))))
    }
}
