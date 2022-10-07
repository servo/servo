/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Parsing of the stylesheet contents.

use crate::counter_style::{parse_counter_style_body, parse_counter_style_name_definition};
use crate::error_reporting::ContextualParseError;
use crate::font_face::parse_font_face_block;
use crate::media_queries::MediaList;
use crate::parser::{Parse, ParserContext};
use crate::properties::parse_property_declaration_list;
use crate::selector_parser::{SelectorImpl, SelectorParser};
use crate::shared_lock::{Locked, SharedRwLock};
use crate::str::starts_with_ignore_ascii_case;
use crate::stylesheets::container_rule::{ContainerCondition, ContainerRule};
use crate::stylesheets::document_rule::DocumentCondition;
use crate::stylesheets::font_feature_values_rule::parse_family_name_list;
use crate::stylesheets::import_rule::ImportLayer;
use crate::stylesheets::keyframes_rule::parse_keyframe_list;
use crate::stylesheets::layer_rule::{LayerBlockRule, LayerName, LayerStatementRule};
use crate::stylesheets::stylesheet::Namespaces;
use crate::stylesheets::supports_rule::SupportsCondition;
use crate::stylesheets::{
    viewport_rule, AllowImportRules, CorsMode, CssRule, CssRuleType, CssRules, DocumentRule,
    FontFeatureValuesRule, FontPaletteValuesRule, KeyframesRule, MediaRule, NamespaceRule,
    PageRule, PageSelectors, RulesMutateError, StyleRule, StylesheetLoader, SupportsRule,
    ViewportRule,
};
use crate::values::computed::font::FamilyName;
use crate::values::{CssUrl, CustomIdent, DashedIdent, KeyframesName};
use crate::{Namespace, Prefix};
use cssparser::{
    AtRuleParser, BasicParseError, BasicParseErrorKind, CowRcStr, Parser, ParserState,
    QualifiedRuleParser, RuleListParser, SourcePosition,
};
use selectors::SelectorList;
use servo_arc::Arc;
use style_traits::{ParseError, StyleParseErrorKind};

/// The information we need particularly to do CSSOM insertRule stuff.
pub struct InsertRuleContext<'a> {
    /// The rule list we're about to insert into.
    pub rule_list: &'a [CssRule],
    /// The index we're about to get inserted at.
    pub index: usize,
}

impl<'a> InsertRuleContext<'a> {
    /// Returns the max rule state allowable for insertion at a given index in
    /// the rule list.
    pub fn max_rule_state_at_index(&self, index: usize) -> State {
        let rule = match self.rule_list.get(index) {
            Some(rule) => rule,
            None => return State::Body,
        };
        match rule {
            CssRule::Import(..) => State::Imports,
            CssRule::Namespace(..) => State::Namespaces,
            CssRule::LayerStatement(..) => {
                // If there are @import / @namespace after this layer, then
                // we're in the early-layers phase, otherwise we're in the body
                // and everything is fair game.
                let next_non_layer_statement_rule = self.rule_list[index + 1..]
                    .iter()
                    .find(|r| !matches!(*r, CssRule::LayerStatement(..)));
                if let Some(non_layer) = next_non_layer_statement_rule {
                    if matches!(*non_layer, CssRule::Import(..) | CssRule::Namespace(..)) {
                        return State::EarlyLayers;
                    }
                }
                State::Body
            },
            _ => State::Body,
        }
    }
}

/// The parser for the top-level rules in a stylesheet.
pub struct TopLevelRuleParser<'a> {
    /// A reference to the lock we need to use to create rules.
    pub shared_lock: &'a SharedRwLock,
    /// A reference to a stylesheet loader if applicable, for `@import` rules.
    pub loader: Option<&'a dyn StylesheetLoader>,
    /// The top-level parser context.
    ///
    /// This won't contain any namespaces, and only nested parsers created with
    /// `ParserContext::new_with_rule_type` will.
    pub context: ParserContext<'a>,
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
    /// Whether @import rules will be allowed.
    pub allow_import_rules: AllowImportRules,
}

impl<'b> TopLevelRuleParser<'b> {
    fn nested<'a: 'b>(&'a self) -> NestedRuleParser<'a, 'b> {
        NestedRuleParser {
            shared_lock: self.shared_lock,
            context: &self.context,
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

        let max_rule_state = ctx.max_rule_state_at_index(ctx.index);
        if new_state > max_rule_state {
            self.dom_error = Some(RulesMutateError::HierarchyRequest);
            return false;
        }

        // If there's anything that isn't a namespace rule (or import rule, but
        // we checked that already at the beginning), reject with a
        // StateError.
        if new_state == State::Namespaces &&
            ctx.rule_list[ctx.index..]
                .iter()
                .any(|r| !matches!(*r, CssRule::Namespace(..)))
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
    /// We're parsing early `@layer` statement rules.
    EarlyLayers = 2,
    /// We're parsing `@import` and early `@layer` statement rules.
    Imports = 3,
    /// We're parsing `@namespace` rules.
    Namespaces = 4,
    /// We're parsing the main body of the stylesheet.
    Body = 5,
}

#[derive(Clone, Debug, MallocSizeOf, ToShmem)]
/// Vendor prefix.
pub enum VendorPrefix {
    /// -moz prefix.
    Moz,
    /// -webkit prefix.
    WebKit,
}

/// A rule prelude for at-rule with block.
pub enum AtRulePrelude {
    /// A @font-face rule prelude.
    FontFace,
    /// A @font-feature-values rule prelude, with its FamilyName list.
    FontFeatureValues(Vec<FamilyName>),
    /// A @font-palette-values rule prelude, with its identifier.
    FontPaletteValues(DashedIdent),
    /// A @counter-style rule prelude, with its counter style name.
    CounterStyle(CustomIdent),
    /// A @media rule prelude, with its media queries.
    Media(Arc<Locked<MediaList>>),
    /// A @container rule prelude.
    Container(Arc<ContainerCondition>),
    /// An @supports rule, with its conditional
    Supports(SupportsCondition),
    /// A @viewport rule prelude.
    Viewport,
    /// A @keyframes rule, with its animation name and vendor prefix if exists.
    Keyframes(KeyframesName, Option<VendorPrefix>),
    /// A @page rule prelude, with its page name if it exists.
    Page(PageSelectors),
    /// A @document rule, with its conditional.
    Document(DocumentCondition),
    /// A @import rule prelude.
    Import(CssUrl, Arc<Locked<MediaList>>, Option<ImportLayer>),
    /// A @namespace rule prelude.
    Namespace(Option<Prefix>, Namespace),
    /// A @layer rule prelude.
    Layer(Vec<LayerName>),
}

impl<'a, 'i> AtRuleParser<'i> for TopLevelRuleParser<'a> {
    type Prelude = AtRulePrelude;
    type AtRule = (SourcePosition, CssRule);
    type Error = StyleParseErrorKind<'i>;

    fn parse_prelude<'t>(
        &mut self,
        name: CowRcStr<'i>,
        input: &mut Parser<'i, 't>,
    ) -> Result<AtRulePrelude, ParseError<'i>> {
        match_ignore_ascii_case! { &*name,
            "import" => {
                if !self.check_state(State::Imports) {
                    return Err(input.new_custom_error(StyleParseErrorKind::UnexpectedImportRule))
                }

                if let AllowImportRules::No = self.allow_import_rules {
                    return Err(input.new_custom_error(StyleParseErrorKind::DisallowedImportRule))
                }

                // FIXME(emilio): We should always be able to have a loader
                // around! See bug 1533783.
                if self.loader.is_none() {
                    error!("Saw @import rule, but no way to trigger the load");
                    return Err(input.new_custom_error(StyleParseErrorKind::UnexpectedImportRule))
                }

                let url_string = input.expect_url_or_string()?.as_ref().to_owned();
                let url = CssUrl::parse_from_string(url_string, &self.context, CorsMode::None);

                #[cfg(feature = "gecko")]
                let layers_enabled = static_prefs::pref!("layout.css.cascade-layers.enabled");
                #[cfg(feature = "servo")]
                let layers_enabled = false;

                let layer = if !layers_enabled {
                    None
                } else if input.try_parse(|input| input.expect_ident_matching("layer")).is_ok() {
                    Some(ImportLayer {
                        name: None,
                    })
                } else {
                    input.try_parse(|input| {
                        input.expect_function_matching("layer")?;
                        input.parse_nested_block(|input| {
                            LayerName::parse(&self.context, input)
                        }).map(|name| ImportLayer {
                            name: Some(name),
                        })
                    }).ok()
                };

                let media = MediaList::parse(&self.context, input);
                let media = Arc::new(self.shared_lock.wrap(media));

                return Ok(AtRulePrelude::Import(url, media, layer));
            },
            "namespace" => {
                if !self.check_state(State::Namespaces) {
                    return Err(input.new_custom_error(StyleParseErrorKind::UnexpectedNamespaceRule))
                }

                let prefix = input.try_parse(|i| i.expect_ident_cloned())
                                  .map(|s| Prefix::from(s.as_ref())).ok();
                let maybe_namespace = match input.expect_url_or_string() {
                    Ok(url_or_string) => url_or_string,
                    Err(BasicParseError { kind: BasicParseErrorKind::UnexpectedToken(t), location }) => {
                        return Err(location.new_custom_error(StyleParseErrorKind::UnexpectedTokenWithinNamespace(t)))
                    }
                    Err(e) => return Err(e.into()),
                };
                let url = Namespace::from(maybe_namespace.as_ref());
                return Ok(AtRulePrelude::Namespace(prefix, url));
            },
            // @charset is removed by rust-cssparser if it’s the first rule in the stylesheet
            // anything left is invalid.
            "charset" => {
                self.dom_error = Some(RulesMutateError::HierarchyRequest);
                return Err(input.new_custom_error(StyleParseErrorKind::UnexpectedCharsetRule))
            },
            "layer" => {
                let state_to_check = if self.state <= State::EarlyLayers {
                    // The real state depends on whether there's a block or not.
                    // We don't know that yet, but the parse_block check deals
                    // with that.
                    State::EarlyLayers
                } else {
                    State::Body
                };
                if !self.check_state(state_to_check) {
                    return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError));
                }
            },
            _ => {
                // All other rules have blocks, so we do this check early in
                // parse_block instead.
            }
        }

        AtRuleParser::parse_prelude(&mut self.nested(), name, input)
    }

    #[inline]
    fn parse_block<'t>(
        &mut self,
        prelude: AtRulePrelude,
        start: &ParserState,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self::AtRule, ParseError<'i>> {
        if !self.check_state(State::Body) {
            return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError));
        }
        let rule = AtRuleParser::parse_block(&mut self.nested(), prelude, start, input)?;
        self.state = State::Body;
        Ok((start.position(), rule))
    }

    #[inline]
    fn rule_without_block(
        &mut self,
        prelude: AtRulePrelude,
        start: &ParserState,
    ) -> Result<Self::AtRule, ()> {
        let rule = match prelude {
            AtRulePrelude::Import(url, media, layer) => {
                let loader = self
                    .loader
                    .expect("Expected a stylesheet loader for @import");

                let import_rule = loader.request_stylesheet(
                    url,
                    start.source_location(),
                    &self.context,
                    &self.shared_lock,
                    media,
                    layer,
                );

                self.state = State::Imports;
                CssRule::Import(import_rule)
            },
            AtRulePrelude::Namespace(prefix, url) => {
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
                    source_location: start.source_location(),
                })))
            },
            AtRulePrelude::Layer(ref names) => {
                if names.is_empty() {
                    return Err(());
                }
                if self.state <= State::EarlyLayers {
                    self.state = State::EarlyLayers;
                } else {
                    self.state = State::Body;
                }
                AtRuleParser::rule_without_block(&mut self.nested(), prelude, start)
                    .expect("All validity checks on the nested parser should be done before changing self.state")
            },
            _ => AtRuleParser::rule_without_block(&mut self.nested(), prelude, start)?,
        };

        Ok((start.position(), rule))
    }
}

impl<'a, 'i> QualifiedRuleParser<'i> for TopLevelRuleParser<'a> {
    type Prelude = SelectorList<SelectorImpl>;
    type QualifiedRule = (SourcePosition, CssRule);
    type Error = StyleParseErrorKind<'i>;

    #[inline]
    fn parse_prelude<'t>(
        &mut self,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self::Prelude, ParseError<'i>> {
        if !self.check_state(State::Body) {
            return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError));
        }

        QualifiedRuleParser::parse_prelude(&mut self.nested(), input)
    }

    #[inline]
    fn parse_block<'t>(
        &mut self,
        prelude: Self::Prelude,
        start: &ParserState,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self::QualifiedRule, ParseError<'i>> {
        let rule = QualifiedRuleParser::parse_block(&mut self.nested(), prelude, start, input)?;
        self.state = State::Body;
        Ok((start.position(), rule))
    }
}

#[derive(Clone)] // shallow, relatively cheap .clone
struct NestedRuleParser<'a, 'b: 'a> {
    shared_lock: &'a SharedRwLock,
    context: &'a ParserContext<'b>,
    namespaces: &'a Namespaces,
}

impl<'a, 'b> NestedRuleParser<'a, 'b> {
    fn parse_nested_rules(
        &mut self,
        input: &mut Parser,
        rule_type: CssRuleType,
    ) -> Arc<Locked<CssRules>> {
        let context = ParserContext::new_with_rule_type(self.context, rule_type, self.namespaces);

        let nested_parser = NestedRuleParser {
            shared_lock: self.shared_lock,
            context: &context,
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
                    self.context.log_css_error(location, error);
                },
            }
        }
        CssRules::new(rules, self.shared_lock)
    }
}

fn container_queries_enabled() -> bool {
    #[cfg(feature = "gecko")]
    return static_prefs::pref!("layout.css.container-queries.enabled");
    #[cfg(feature = "servo")]
    return servo_config::prefs::pref_map()
        .get("layout.container-queries.enabled")
        .as_bool()
        .unwrap_or(false);
}

impl<'a, 'b, 'i> AtRuleParser<'i> for NestedRuleParser<'a, 'b> {
    type Prelude = AtRulePrelude;
    type AtRule = CssRule;
    type Error = StyleParseErrorKind<'i>;

    fn parse_prelude<'t>(
        &mut self,
        name: CowRcStr<'i>,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self::Prelude, ParseError<'i>> {
        Ok(match_ignore_ascii_case! { &*name,
            "media" => {
                let media_queries = MediaList::parse(self.context, input);
                let arc = Arc::new(self.shared_lock.wrap(media_queries));
                AtRulePrelude::Media(arc)
            },
            "supports" => {
                let cond = SupportsCondition::parse(input)?;
                AtRulePrelude::Supports(cond)
            },
            "font-face" => {
                AtRulePrelude::FontFace
            },
            "container" if container_queries_enabled() => {
                let condition = Arc::new(ContainerCondition::parse(self.context, input)?);
                AtRulePrelude::Container(condition)
            },
            "layer" => {
                let names = input.try_parse(|input| {
                    input.parse_comma_separated(|input| {
                        LayerName::parse(self.context, input)
                    })
                }).unwrap_or_default();
                AtRulePrelude::Layer(names)
            },
            "font-feature-values" if cfg!(feature = "gecko") => {
                let family_names = parse_family_name_list(self.context, input)?;
                AtRulePrelude::FontFeatureValues(family_names)
            },
            "font-palette-values" if static_prefs::pref!("layout.css.font-palette.enabled") => {
                let name = DashedIdent::parse(self.context, input)?;
                AtRulePrelude::FontPaletteValues(name)
            },
            "counter-style" if cfg!(feature = "gecko") => {
                let name = parse_counter_style_name_definition(input)?;
                AtRulePrelude::CounterStyle(name)
            },
            "viewport" if viewport_rule::enabled() => {
                AtRulePrelude::Viewport
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
                AtRulePrelude::Keyframes(name, prefix)
            },
            #[cfg(feature = "gecko")]
            "page" => {
                AtRulePrelude::Page(if static_prefs::pref!("layout.css.named-pages.enabled") {
                    input.try_parse(|i| PageSelectors::parse(self.context, i)).unwrap_or_default()
                } else {
                    PageSelectors::default()
                })
            },
            "-moz-document" if cfg!(feature = "gecko") => {
                let cond = DocumentCondition::parse(self.context, input)?;
                AtRulePrelude::Document(cond)
            },
            _ => return Err(input.new_custom_error(StyleParseErrorKind::UnsupportedAtRule(name.clone())))
        })
    }

    fn parse_block<'t>(
        &mut self,
        prelude: AtRulePrelude,
        start: &ParserState,
        input: &mut Parser<'i, 't>,
    ) -> Result<CssRule, ParseError<'i>> {
        match prelude {
            AtRulePrelude::FontFace => {
                let context = ParserContext::new_with_rule_type(
                    self.context,
                    CssRuleType::FontFace,
                    self.namespaces,
                );

                Ok(CssRule::FontFace(Arc::new(self.shared_lock.wrap(
                    parse_font_face_block(&context, input, start.source_location()).into(),
                ))))
            },
            AtRulePrelude::FontFeatureValues(family_names) => {
                let context = ParserContext::new_with_rule_type(
                    self.context,
                    CssRuleType::FontFeatureValues,
                    self.namespaces,
                );

                Ok(CssRule::FontFeatureValues(Arc::new(self.shared_lock.wrap(
                    FontFeatureValuesRule::parse(
                        &context,
                        input,
                        family_names,
                        start.source_location(),
                    ),
                ))))
            },
            AtRulePrelude::FontPaletteValues(name) => {
                let context = ParserContext::new_with_rule_type(
                    self.context,
                    CssRuleType::FontPaletteValues,
                    self.namespaces,
                );

                Ok(CssRule::FontPaletteValues(Arc::new(self.shared_lock.wrap(
                    FontPaletteValuesRule::parse(
                        &context,
                        input,
                        name,
                        start.source_location(),
                    ),
                ))))
            },
            AtRulePrelude::CounterStyle(name) => {
                let context = ParserContext::new_with_rule_type(
                    self.context,
                    CssRuleType::CounterStyle,
                    self.namespaces,
                );

                Ok(CssRule::CounterStyle(Arc::new(
                    self.shared_lock.wrap(
                        parse_counter_style_body(name, &context, input, start.source_location())?
                            .into(),
                    ),
                )))
            },
            AtRulePrelude::Media(media_queries) => {
                Ok(CssRule::Media(Arc::new(self.shared_lock.wrap(MediaRule {
                    media_queries,
                    rules: self.parse_nested_rules(input, CssRuleType::Media),
                    source_location: start.source_location(),
                }))))
            },
            AtRulePrelude::Supports(condition) => {
                let eval_context = ParserContext::new_with_rule_type(
                    self.context,
                    CssRuleType::Style,
                    self.namespaces,
                );

                let enabled = condition.eval(&eval_context, self.namespaces);
                Ok(CssRule::Supports(Arc::new(self.shared_lock.wrap(
                    SupportsRule {
                        condition,
                        rules: self.parse_nested_rules(input, CssRuleType::Supports),
                        enabled,
                        source_location: start.source_location(),
                    },
                ))))
            },
            AtRulePrelude::Viewport => {
                let context = ParserContext::new_with_rule_type(
                    self.context,
                    CssRuleType::Viewport,
                    self.namespaces,
                );

                Ok(CssRule::Viewport(Arc::new(
                    self.shared_lock.wrap(ViewportRule::parse(&context, input)?),
                )))
            },
            AtRulePrelude::Keyframes(name, vendor_prefix) => {
                let context = ParserContext::new_with_rule_type(
                    self.context,
                    CssRuleType::Keyframes,
                    self.namespaces,
                );

                Ok(CssRule::Keyframes(Arc::new(self.shared_lock.wrap(
                    KeyframesRule {
                        name,
                        keyframes: parse_keyframe_list(&context, input, self.shared_lock),
                        vendor_prefix,
                        source_location: start.source_location(),
                    },
                ))))
            },
            AtRulePrelude::Page(selectors) => {
                let context = ParserContext::new_with_rule_type(
                    self.context,
                    CssRuleType::Page,
                    self.namespaces,
                );

                let declarations = parse_property_declaration_list(&context, input, None);
                Ok(CssRule::Page(Arc::new(self.shared_lock.wrap(PageRule {
                    selectors,
                    block: Arc::new(self.shared_lock.wrap(declarations)),
                    source_location: start.source_location(),
                }))))
            },
            AtRulePrelude::Document(condition) => {
                if !cfg!(feature = "gecko") {
                    unreachable!()
                }
                Ok(CssRule::Document(Arc::new(self.shared_lock.wrap(
                    DocumentRule {
                        condition,
                        rules: self.parse_nested_rules(input, CssRuleType::Document),
                        source_location: start.source_location(),
                    },
                ))))
            },
            AtRulePrelude::Container(condition) => Ok(CssRule::Container(Arc::new(
                self.shared_lock.wrap(ContainerRule {
                    condition,
                    rules: self.parse_nested_rules(input, CssRuleType::Container),
                    source_location: start.source_location(),
                }),
            ))),
            AtRulePrelude::Layer(names) => {
                let name = match names.len() {
                    0 | 1 => names.into_iter().next(),
                    _ => return Err(input.new_error(BasicParseErrorKind::AtRuleBodyInvalid)),
                };
                Ok(CssRule::LayerBlock(Arc::new(self.shared_lock.wrap(
                    LayerBlockRule {
                        name,
                        rules: self.parse_nested_rules(input, CssRuleType::LayerBlock),
                        source_location: start.source_location(),
                    },
                ))))
            },
            AtRulePrelude::Import(..) | AtRulePrelude::Namespace(..) => {
                // These rules don't have blocks.
                Err(input.new_unexpected_token_error(cssparser::Token::CurlyBracketBlock))
            },
        }
    }

    #[inline]
    fn rule_without_block(
        &mut self,
        prelude: AtRulePrelude,
        start: &ParserState,
    ) -> Result<Self::AtRule, ()> {
        Ok(match prelude {
            AtRulePrelude::Layer(names) => {
                if names.is_empty() {
                    return Err(());
                }
                CssRule::LayerStatement(Arc::new(self.shared_lock.wrap(LayerStatementRule {
                    names,
                    source_location: start.source_location(),
                })))
            },
            _ => return Err(()),
        })
    }
}

#[inline(never)]
fn check_for_useless_selector(
    input: &mut Parser,
    context: &ParserContext,
    selectors: &SelectorList<SelectorImpl>,
) {
    use cssparser::ToCss;

    'selector_loop: for selector in selectors.0.iter() {
        let mut current = selector.iter();
        loop {
            let mut found_host = false;
            let mut found_non_host = false;
            for component in &mut current {
                if component.is_host() {
                    found_host = true;
                } else {
                    found_non_host = true;
                }
                if found_host && found_non_host {
                    let location = input.current_source_location();
                    context.log_css_error(
                        location,
                        ContextualParseError::NeverMatchingHostSelector(selector.to_css_string()),
                    );
                    continue 'selector_loop;
                }
            }
            if current.next_sequence().is_none() {
                break;
            }
        }
    }
}

impl<'a, 'b, 'i> QualifiedRuleParser<'i> for NestedRuleParser<'a, 'b> {
    type Prelude = SelectorList<SelectorImpl>;
    type QualifiedRule = CssRule;
    type Error = StyleParseErrorKind<'i>;

    fn parse_prelude<'t>(
        &mut self,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self::Prelude, ParseError<'i>> {
        let selector_parser = SelectorParser {
            stylesheet_origin: self.context.stylesheet_origin,
            namespaces: self.namespaces,
            url_data: self.context.url_data,
            for_supports_rule: false,
        };
        let selectors = SelectorList::parse(&selector_parser, input)?;
        if self.context.error_reporting_enabled() {
            check_for_useless_selector(input, &self.context, &selectors);
        }
        Ok(selectors)
    }

    fn parse_block<'t>(
        &mut self,
        selectors: Self::Prelude,
        start: &ParserState,
        input: &mut Parser<'i, 't>,
    ) -> Result<CssRule, ParseError<'i>> {
        let context =
            ParserContext::new_with_rule_type(self.context, CssRuleType::Style, self.namespaces);

        let declarations = parse_property_declaration_list(&context, input, Some(&selectors));
        let block = Arc::new(self.shared_lock.wrap(declarations));
        Ok(CssRule::Style(Arc::new(self.shared_lock.wrap(StyleRule {
            selectors,
            block,
            source_location: start.source_location(),
        }))))
    }
}
