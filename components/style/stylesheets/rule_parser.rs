/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Parsing of the stylesheet contents.

use crate::counter_style::{parse_counter_style_body, parse_counter_style_name_definition};
#[cfg(feature = "gecko")]
use crate::custom_properties::parse_name as parse_custom_property_name;
use crate::error_reporting::ContextualParseError;
use crate::font_face::parse_font_face_block;
use crate::media_queries::MediaList;
use crate::parser::{Parse, ParserContext};
use crate::properties::declaration_block::{
    parse_property_declaration_list, DeclarationParserState, PropertyDeclarationBlock,
};
use crate::properties_and_values::rule::{parse_property_block, PropertyRuleName};
use crate::selector_parser::{SelectorImpl, SelectorParser};
use crate::shared_lock::{Locked, SharedRwLock};
use crate::str::starts_with_ignore_ascii_case;
use crate::stylesheets::container_rule::{ContainerCondition, ContainerRule};
use crate::stylesheets::document_rule::DocumentCondition;
use crate::stylesheets::font_feature_values_rule::parse_family_name_list;
use crate::stylesheets::import_rule::{ImportLayer, ImportRule, ImportSupportsCondition};
use crate::stylesheets::keyframes_rule::parse_keyframe_list;
use crate::stylesheets::layer_rule::{LayerBlockRule, LayerName, LayerStatementRule};
use crate::stylesheets::supports_rule::SupportsCondition;
use crate::stylesheets::{
    AllowImportRules, CorsMode, CssRule, CssRuleType, CssRules, DocumentRule,
    FontFeatureValuesRule, FontPaletteValuesRule, KeyframesRule, MediaRule, NamespaceRule,
    PageRule, PageSelectors, RulesMutateError, StyleRule, StylesheetLoader, SupportsRule,
};
use crate::values::computed::font::FamilyName;
use crate::values::{CssUrl, CustomIdent, DashedIdent, KeyframesName};
#[cfg(feature = "gecko")]
use crate::Atom;
use crate::{Namespace, Prefix};
use cssparser::{
    AtRuleParser, BasicParseError, BasicParseErrorKind, CowRcStr, DeclarationParser, Parser,
    ParserState, QualifiedRuleParser, RuleBodyItemParser, RuleBodyParser, SourceLocation,
    SourcePosition,
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
pub struct TopLevelRuleParser<'a, 'i> {
    /// A reference to the lock we need to use to create rules.
    pub shared_lock: &'a SharedRwLock,
    /// A reference to a stylesheet loader if applicable, for `@import` rules.
    pub loader: Option<&'a dyn StylesheetLoader>,
    /// The top-level parser context.
    pub context: ParserContext<'a>,
    /// The current state of the parser.
    pub state: State,
    /// Whether we have tried to parse was invalid due to being in the wrong
    /// place (e.g. an @import rule was found while in the `Body` state). Reset
    /// to `false` when `take_had_hierarchy_error` is called.
    pub dom_error: Option<RulesMutateError>,
    /// The info we need insert a rule in a list.
    pub insert_rule_context: Option<InsertRuleContext<'a>>,
    /// Whether @import rules will be allowed.
    pub allow_import_rules: AllowImportRules,
    /// Parser state for declaration blocks in either nested rules or style rules.
    pub declaration_parser_state: DeclarationParserState<'i>,
    /// The rules we've parsed so far.
    pub rules: Vec<CssRule>,
}

impl<'a, 'i> TopLevelRuleParser<'a, 'i> {
    fn nested<'b>(&'b mut self) -> NestedRuleParser<'b, 'a, 'i> {
        NestedRuleParser {
            shared_lock: self.shared_lock,
            context: &mut self.context,
            declaration_parser_state: &mut self.declaration_parser_state,
            rules: &mut self.rules,
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
    /// A @keyframes rule, with its animation name and vendor prefix if exists.
    Keyframes(KeyframesName, Option<VendorPrefix>),
    /// A @page rule prelude, with its page name if it exists.
    Page(PageSelectors),
    /// A @property rule prelude.
    Property(PropertyRuleName),
    /// A @document rule, with its conditional.
    Document(DocumentCondition),
    /// A @import rule prelude.
    Import(
        CssUrl,
        Arc<Locked<MediaList>>,
        Option<ImportSupportsCondition>,
        ImportLayer,
    ),
    /// A @namespace rule prelude.
    Namespace(Option<Prefix>, Namespace),
    /// A @layer rule prelude.
    Layer(Vec<LayerName>),
}

impl<'a, 'i> AtRuleParser<'i> for TopLevelRuleParser<'a, 'i> {
    type Prelude = AtRulePrelude;
    type AtRule = SourcePosition;
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

                let (layer, supports) = ImportRule::parse_layer_and_supports(input, &mut self.context);

                let media = MediaList::parse(&self.context, input);
                let media = Arc::new(self.shared_lock.wrap(media));

                return Ok(AtRulePrelude::Import(url, media, supports, layer));
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
        AtRuleParser::parse_block(&mut self.nested(), prelude, start, input)?;
        self.state = State::Body;
        Ok(start.position())
    }

    #[inline]
    fn rule_without_block(
        &mut self,
        prelude: AtRulePrelude,
        start: &ParserState,
    ) -> Result<Self::AtRule, ()> {
        match prelude {
            AtRulePrelude::Import(url, media, supports, layer) => {
                let loader = self
                    .loader
                    .expect("Expected a stylesheet loader for @import");

                let import_rule = loader.request_stylesheet(
                    url,
                    start.source_location(),
                    &self.context,
                    &self.shared_lock,
                    media,
                    supports,
                    layer,
                );

                self.state = State::Imports;
                self.rules.push(CssRule::Import(import_rule))
            },
            AtRulePrelude::Namespace(prefix, url) => {
                let namespaces = self.context.namespaces.to_mut();
                let prefix = if let Some(prefix) = prefix {
                    namespaces.prefixes.insert(prefix.clone(), url.clone());
                    Some(prefix)
                } else {
                    namespaces.default = Some(url.clone());
                    None
                };

                self.state = State::Namespaces;
                self.rules.push(CssRule::Namespace(Arc::new(NamespaceRule {
                    prefix,
                    url,
                    source_location: start.source_location(),
                })));
            },
            AtRulePrelude::Layer(..) => {
                AtRuleParser::rule_without_block(&mut self.nested(), prelude, start)?;
                if self.state <= State::EarlyLayers {
                    self.state = State::EarlyLayers;
                } else {
                    self.state = State::Body;
                }
            },
            _ => AtRuleParser::rule_without_block(&mut self.nested(), prelude, start)?,
        };

        Ok(start.position())
    }
}

impl<'a, 'i> QualifiedRuleParser<'i> for TopLevelRuleParser<'a, 'i> {
    type Prelude = SelectorList<SelectorImpl>;
    type QualifiedRule = SourcePosition;
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
        QualifiedRuleParser::parse_block(&mut self.nested(), prelude, start, input)?;
        self.state = State::Body;
        Ok(start.position())
    }
}

struct NestedRuleParser<'a, 'b: 'a, 'i> {
    shared_lock: &'a SharedRwLock,
    context: &'a mut ParserContext<'b>,
    declaration_parser_state: &'a mut DeclarationParserState<'i>,
    rules: &'a mut Vec<CssRule>,
}

struct NestedParseResult {
    rules: Vec<CssRule>,
    declarations: PropertyDeclarationBlock,
}

impl NestedParseResult {
    fn into_rules(
        mut self,
        shared_lock: &SharedRwLock,
        source_location: SourceLocation,
    ) -> Arc<Locked<CssRules>> {
        lazy_static! {
            static ref AMPERSAND: SelectorList<SelectorImpl> = {
                let list = SelectorList::ampersand();
                list.0
                    .iter()
                    .for_each(|selector| selector.mark_as_intentionally_leaked());
                list
            };
        };

        if !self.declarations.is_empty() {
            self.rules.insert(
                0,
                CssRule::Style(Arc::new(shared_lock.wrap(StyleRule {
                    selectors: AMPERSAND.clone(),
                    block: Arc::new(shared_lock.wrap(self.declarations)),
                    rules: None,
                    source_location,
                }))),
            )
        }

        CssRules::new(self.rules, shared_lock)
    }
}

impl<'a, 'b, 'i> NestedRuleParser<'a, 'b, 'i> {
    /// When nesting is disabled, we prevent parsing at rules and qualified rules inside style
    /// rules.
    fn allow_at_and_qualified_rules(&self) -> bool {
        if !self.context.rule_types.contains(CssRuleType::Style) {
            return true;
        }
        #[cfg(feature = "gecko")]
        return static_prefs::pref!("layout.css.nesting.enabled");
        #[cfg(feature = "servo")]
        return false;
    }

    fn nest_for_rule<R>(&mut self, rule_type: CssRuleType, cb: impl FnOnce(&mut Self) -> R) -> R {
        let old_rule_types = self.context.rule_types;
        self.context.rule_types.insert(rule_type);
        let r = cb(self);
        self.context.rule_types = old_rule_types;
        r
    }

    fn parse_nested(
        &mut self,
        input: &mut Parser<'i, '_>,
        rule_type: CssRuleType,
        selectors: Option<&SelectorList<SelectorImpl>>,
    ) -> NestedParseResult {
        self.nest_for_rule(rule_type, |parser| {
            let parse_declarations = parser.parse_declarations();
            let mut old_declaration_state = std::mem::take(parser.declaration_parser_state);
            let mut rules = std::mem::take(parser.rules);
            let mut iter = RuleBodyParser::new(input, parser);
            while let Some(result) = iter.next() {
                match result {
                    Ok(()) => {},
                    Err((error, slice)) => {
                        if parse_declarations {
                            iter.parser.declaration_parser_state.did_error(
                                iter.parser.context,
                                error,
                                slice,
                            );
                        } else {
                            let location = error.location;
                            let error = ContextualParseError::InvalidRule(slice, error);
                            iter.parser.context.log_css_error(location, error);
                        }
                    },
                }
            }
            let declarations = if parse_declarations {
                parser
                    .declaration_parser_state
                    .report_errors_if_needed(parser.context, selectors);
                parser.declaration_parser_state.take_declarations()
            } else {
                PropertyDeclarationBlock::default()
            };
            debug_assert!(
                !parser.declaration_parser_state.has_parsed_declarations(),
                "Parsed but didn't consume declarations"
            );
            std::mem::swap(parser.declaration_parser_state, &mut old_declaration_state);
            std::mem::swap(parser.rules, &mut rules);
            NestedParseResult {
                rules,
                declarations,
            }
        })
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

impl<'a, 'b, 'i> AtRuleParser<'i> for NestedRuleParser<'a, 'b, 'i> {
    type Prelude = AtRulePrelude;
    type AtRule = ();
    type Error = StyleParseErrorKind<'i>;

    fn parse_prelude<'t>(
        &mut self,
        name: CowRcStr<'i>,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self::Prelude, ParseError<'i>> {
        if !self.allow_at_and_qualified_rules() {
            return Err(input.new_error(BasicParseErrorKind::AtRuleInvalid(name)));
        }
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
            #[cfg(feature = "gecko")]
            "font-palette-values" if static_prefs::pref!("layout.css.font-palette.enabled") => {
                let name = DashedIdent::parse(self.context, input)?;
                AtRulePrelude::FontPaletteValues(name)
            },
            "counter-style" if cfg!(feature = "gecko") => {
                let name = parse_counter_style_name_definition(input)?;
                AtRulePrelude::CounterStyle(name)
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
                    return Err(input.new_error(BasicParseErrorKind::AtRuleInvalid(name.clone())))
                }
                let name = KeyframesName::parse(self.context, input)?;
                AtRulePrelude::Keyframes(name, prefix)
            },
            "page" if cfg!(feature = "gecko") => {
                AtRulePrelude::Page(
                    input.try_parse(|i| PageSelectors::parse(self.context, i)).unwrap_or_default()
                )
            },
            #[cfg(feature = "gecko")]
            "property" if static_prefs::pref!("layout.css.properties-and-values.enabled") => {
                let name = input.expect_ident_cloned()?;
                let name = parse_custom_property_name(&name).map_err(|_| {
                    input.new_custom_error(StyleParseErrorKind::UnexpectedIdent(name.clone()))
                })?;
                AtRulePrelude::Property(PropertyRuleName(Arc::new(Atom::from(name))))
            },
            "-moz-document" if cfg!(feature = "gecko") => {
                let cond = DocumentCondition::parse(self.context, input)?;
                AtRulePrelude::Document(cond)
            },
            _ => return Err(input.new_error(BasicParseErrorKind::AtRuleInvalid(name.clone())))
        })
    }

    fn parse_block<'t>(
        &mut self,
        prelude: AtRulePrelude,
        start: &ParserState,
        input: &mut Parser<'i, 't>,
    ) -> Result<(), ParseError<'i>> {
        let rule = match prelude {
            AtRulePrelude::FontFace => self.nest_for_rule(CssRuleType::FontFace, |p| {
                CssRule::FontFace(Arc::new(p.shared_lock.wrap(
                    parse_font_face_block(&p.context, input, start.source_location()).into(),
                )))
            }),
            AtRulePrelude::FontFeatureValues(family_names) => {
                self.nest_for_rule(CssRuleType::FontFeatureValues, |p| {
                    CssRule::FontFeatureValues(Arc::new(FontFeatureValuesRule::parse(
                        &p.context,
                        input,
                        family_names,
                        start.source_location(),
                    )))
                })
            },
            AtRulePrelude::FontPaletteValues(name) => {
                self.nest_for_rule(CssRuleType::FontPaletteValues, |p| {
                    CssRule::FontPaletteValues(Arc::new(FontPaletteValuesRule::parse(
                        &p.context,
                        input,
                        name,
                        start.source_location(),
                    )))
                })
            },
            AtRulePrelude::CounterStyle(name) => {
                let body = self.nest_for_rule(CssRuleType::CounterStyle, |p| {
                    parse_counter_style_body(name, &p.context, input, start.source_location())
                })?;
                CssRule::CounterStyle(Arc::new(self.shared_lock.wrap(body)))
            },
            AtRulePrelude::Media(media_queries) => {
                let source_location = start.source_location();
                CssRule::Media(Arc::new(MediaRule {
                    media_queries,
                    rules: self
                        .parse_nested(input, CssRuleType::Media, None)
                        .into_rules(self.shared_lock, source_location),
                    source_location,
                }))
            },
            AtRulePrelude::Supports(condition) => {
                let enabled =
                    self.nest_for_rule(CssRuleType::Style, |p| condition.eval(&p.context));
                let source_location = start.source_location();
                CssRule::Supports(Arc::new(SupportsRule {
                    condition,
                    rules: self
                        .parse_nested(input, CssRuleType::Supports, None)
                        .into_rules(self.shared_lock, source_location),
                    enabled,
                    source_location,
                }))
            },
            AtRulePrelude::Keyframes(name, vendor_prefix) => {
                self.nest_for_rule(CssRuleType::Keyframe, |p| {
                    CssRule::Keyframes(Arc::new(p.shared_lock.wrap(KeyframesRule {
                        name,
                        keyframes: parse_keyframe_list(&mut p.context, input, p.shared_lock),
                        vendor_prefix,
                        source_location: start.source_location(),
                    })))
                })
            },
            AtRulePrelude::Page(selectors) => {
                let declarations = self.nest_for_rule(CssRuleType::Page, |p| {
                    // TODO: Support nesting in @page rules?
                    parse_property_declaration_list(&p.context, input, None)
                });
                CssRule::Page(Arc::new(self.shared_lock.wrap(PageRule {
                    selectors,
                    block: Arc::new(self.shared_lock.wrap(declarations)),
                    source_location: start.source_location(),
                })))
            },
            AtRulePrelude::Property(name) => self.nest_for_rule(CssRuleType::Property, |p| {
                CssRule::Property(Arc::new(parse_property_block(
                    &p.context,
                    input,
                    name,
                    start.source_location(),
                )))
            }),
            AtRulePrelude::Document(condition) => {
                if !cfg!(feature = "gecko") {
                    unreachable!()
                }
                let source_location = start.source_location();
                CssRule::Document(Arc::new(DocumentRule {
                    condition,
                    rules: self
                        .parse_nested(input, CssRuleType::Document, None)
                        .into_rules(self.shared_lock, source_location),
                    source_location,
                }))
            },
            AtRulePrelude::Container(condition) => {
                let source_location = start.source_location();
                CssRule::Container(Arc::new(ContainerRule {
                    condition,
                    rules: self
                        .parse_nested(input, CssRuleType::Container, None)
                        .into_rules(self.shared_lock, source_location),
                    source_location,
                }))
            },
            AtRulePrelude::Layer(names) => {
                let name = match names.len() {
                    0 | 1 => names.into_iter().next(),
                    _ => return Err(input.new_error(BasicParseErrorKind::AtRuleBodyInvalid)),
                };
                let source_location = start.source_location();
                CssRule::LayerBlock(Arc::new(LayerBlockRule {
                    name,
                    rules: self
                        .parse_nested(input, CssRuleType::LayerBlock, None)
                        .into_rules(self.shared_lock, source_location),
                    source_location,
                }))
            },
            AtRulePrelude::Import(..) | AtRulePrelude::Namespace(..) => {
                // These rules don't have blocks.
                return Err(input.new_unexpected_token_error(cssparser::Token::CurlyBracketBlock));
            },
        };
        self.rules.push(rule);
        Ok(())
    }

    #[inline]
    fn rule_without_block(
        &mut self,
        prelude: AtRulePrelude,
        start: &ParserState,
    ) -> Result<(), ()> {
        let rule = match prelude {
            AtRulePrelude::Layer(names) => {
                if names.is_empty() {
                    return Err(());
                }
                CssRule::LayerStatement(Arc::new(LayerStatementRule {
                    names,
                    source_location: start.source_location(),
                }))
            },
            _ => return Err(()),
        };
        self.rules.push(rule);
        Ok(())
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

impl<'a, 'b, 'i> QualifiedRuleParser<'i> for NestedRuleParser<'a, 'b, 'i> {
    type Prelude = SelectorList<SelectorImpl>;
    type QualifiedRule = ();
    type Error = StyleParseErrorKind<'i>;

    fn parse_prelude<'t>(
        &mut self,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self::Prelude, ParseError<'i>> {
        let selector_parser = SelectorParser {
            stylesheet_origin: self.context.stylesheet_origin,
            namespaces: &self.context.namespaces,
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
    ) -> Result<(), ParseError<'i>> {
        let result = self.parse_nested(input, CssRuleType::Style, Some(&selectors));
        let block = Arc::new(self.shared_lock.wrap(result.declarations));
        self.rules
            .push(CssRule::Style(Arc::new(self.shared_lock.wrap(StyleRule {
                selectors,
                block,
                rules: if result.rules.is_empty() {
                    None
                } else {
                    Some(CssRules::new(result.rules, self.shared_lock))
                },
                source_location: start.source_location(),
            }))));
        Ok(())
    }
}

impl<'a, 'b, 'i> DeclarationParser<'i> for NestedRuleParser<'a, 'b, 'i> {
    type Declaration = ();
    type Error = StyleParseErrorKind<'i>;
    fn parse_value<'t>(
        &mut self,
        name: CowRcStr<'i>,
        input: &mut Parser<'i, 't>,
    ) -> Result<(), ParseError<'i>> {
        self.declaration_parser_state
            .parse_value(self.context, name, input)
    }
}

impl<'a, 'b, 'i> RuleBodyItemParser<'i, (), StyleParseErrorKind<'i>>
    for NestedRuleParser<'a, 'b, 'i>
{
    fn parse_qualified(&self) -> bool {
        self.allow_at_and_qualified_rules()
    }

    /// If nesting is disabled, we can't get there for a non-style-rule. If it's enabled, we parse
    /// raw declarations there.
    fn parse_declarations(&self) -> bool {
        self.context.rule_types.contains(CssRuleType::Style)
    }
}
