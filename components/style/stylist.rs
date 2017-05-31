/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Selector matching.

use {Atom, LocalName, Namespace};
use bit_vec::BitVec;
use context::{QuirksMode, SharedStyleContext};
use data::ComputedStyle;
use dom::TElement;
use element_state::ElementState;
use error_reporting::RustLogReporter;
use font_metrics::FontMetricsProvider;
#[cfg(feature = "gecko")]
use gecko_bindings::structs::{nsIAtom, StyleRuleInclusion};
use keyframes::KeyframesAnimation;
use media_queries::Device;
use properties::{self, CascadeFlags, ComputedValues};
use properties::{AnimationRules, PropertyDeclarationBlock};
#[cfg(feature = "servo")]
use properties::INHERIT_ALL;
use restyle_hints::{HintComputationContext, DependencySet, RestyleHint};
use rule_tree::{CascadeLevel, RuleTree, StrongRuleNode, StyleSource};
use selector_map::{SelectorMap, SelectorMapEntry};
use selector_parser::{SelectorImpl, PseudoElement};
use selectors::attr::NamespaceConstraint;
use selectors::bloom::BloomFilter;
use selectors::matching::{ElementSelectorFlags, matches_selector, MatchingContext, MatchingMode};
use selectors::matching::AFFECTED_BY_PRESENTATIONAL_HINTS;
use selectors::parser::{Combinator, Component, Selector, SelectorInner, SelectorIter, SelectorMethods};
use selectors::visitor::SelectorVisitor;
use shared_lock::{Locked, SharedRwLockReadGuard, StylesheetGuards};
use sink::Push;
use smallvec::{SmallVec, VecLike};
#[cfg(feature = "servo")]
use std::marker::PhantomData;
use style_traits::viewport::ViewportConstraints;
use stylearc::Arc;
#[cfg(feature = "gecko")]
use stylesheets::{CounterStyleRule, FontFaceRule};
use stylesheets::{CssRule, DocumentRule, ImportRule, MediaRule, StyleRule, SupportsRule};
use stylesheets::{Stylesheet, Origin, UserAgentStylesheets};
use stylesheets::NestedRuleIterationCondition;
use thread_state;
use viewport::{self, MaybeNew, ViewportRule};

pub use ::fnv::FnvHashMap;

/// List of applicable declaration. This is a transient structure that shuttles
/// declarations between selector matching and inserting into the rule tree, and
/// therefore we want to avoid heap-allocation where possible.
///
/// In measurements on wikipedia, we pretty much never have more than 8 applicable
/// declarations, so we could consider making this 8 entries instead of 16.
/// However, it may depend a lot on workload, and stack space is cheap.
pub type ApplicableDeclarationList = SmallVec<[ApplicableDeclarationBlock; 16]>;

/// This structure holds all the selectors and device characteristics
/// for a given document. The selectors are converted into `Rule`s
/// (defined in rust-selectors), and introduced in a `SelectorMap`
/// depending on the pseudo-element (see `PerPseudoElementSelectorMap`),
/// and stylesheet origin (see the fields of `PerPseudoElementSelectorMap`).
///
/// This structure is effectively created once per pipeline, in the
/// LayoutThread corresponding to that pipeline.
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct Stylist {
    /// Device that the stylist is currently evaluating against.
    ///
    /// This field deserves a bigger comment due to the different use that Gecko
    /// and Servo give to it (that we should eventually unify).
    ///
    /// With Gecko, the device is never changed. Gecko manually tracks whether
    /// the device data should be reconstructed, and "resets" the state of the
    /// device.
    ///
    /// On Servo, on the other hand, the device is a really cheap representation
    /// that is recreated each time some constraint changes and calling
    /// `set_device`.
    device: Device,

    /// Viewport constraints based on the current device.
    viewport_constraints: Option<ViewportConstraints>,

    /// If true, the quirks-mode stylesheet is applied.
    quirks_mode: QuirksMode,

    /// If true, the device has changed, and the stylist needs to be updated.
    is_device_dirty: bool,

    /// If true, the stylist is in a cleared state (e.g. just-constructed, or
    /// had clear() called on it with no following rebuild()).
    is_cleared: bool,

    /// The current selector maps, after evaluating media
    /// rules against the current device.
    element_map: PerPseudoElementSelectorMap,

    /// The rule tree, that stores the results of selector matching.
    rule_tree: RuleTree,

    /// The selector maps corresponding to a given pseudo-element
    /// (depending on the implementation)
    pseudos_map: FnvHashMap<PseudoElement, PerPseudoElementSelectorMap>,

    /// A map with all the animations indexed by name.
    animations: FnvHashMap<Atom, KeyframesAnimation>,

    /// Applicable declarations for a given non-eagerly cascaded pseudo-element.
    /// These are eagerly computed once, and then used to resolve the new
    /// computed values on the fly on layout.
    ///
    /// FIXME(emilio): Use the rule tree!
    precomputed_pseudo_element_decls: FnvHashMap<PseudoElement, Vec<ApplicableDeclarationBlock>>,

    /// A monotonically increasing counter to represent the order on which a
    /// style rule appears in a stylesheet, needed to sort them by source order.
    rules_source_order: usize,

    /// Selector dependencies used to compute restyle hints.
    dependencies: DependencySet,

    /// The attribute local names that appear in attribute selectors.  Used
    /// to avoid taking element snapshots when an irrelevant attribute changes.
    /// (We don't bother storing the namespace, since namespaced attributes
    /// are rare.)
    ///
    /// FIXME(heycam): This doesn't really need to be a counting Bloom filter.
    #[cfg_attr(feature = "servo", ignore_heap_size_of = "just an array")]
    attribute_dependencies: BloomFilter,

    /// Whether `"style"` appears in an attribute selector.  This is not common,
    /// and by tracking this explicitly, we can avoid taking an element snapshot
    /// in the common case of style=""` changing due to modifying
    /// `element.style`.  (We could track this in `attribute_dependencies`, like
    /// all other attributes, but we should probably not risk incorrectly
    /// returning `true` for `"style"` just due to a hash collision.)
    style_attribute_dependency: bool,

    /// The element state bits that are relied on by selectors.  Like
    /// `attribute_dependencies`, this is used to avoid taking element snapshots
    /// when an irrelevant element state bit changes.
    state_dependencies: ElementState,

    /// Selectors that require explicit cache revalidation (i.e. which depend
    /// on state that is not otherwise visible to the cache, like attributes or
    /// tree-structural state like child index and pseudos).
    #[cfg_attr(feature = "servo", ignore_heap_size_of = "Arc")]
    selectors_for_cache_revalidation: SelectorMap<SelectorInner<SelectorImpl>>,

    /// The total number of selectors.
    num_selectors: usize,

    /// The total number of declarations.
    num_declarations: usize,

    /// The total number of times the stylist has been rebuilt.
    num_rebuilds: usize,
}

/// This struct holds data which user of Stylist may want to extract
/// from stylesheets which can be done at the same time as updating.
#[cfg(feature = "gecko")]
pub struct ExtraStyleData<'a> {
    /// A list of effective font-face rules and their origin.
    pub font_faces: &'a mut Vec<(Arc<Locked<FontFaceRule>>, Origin)>,
    /// A map of effective counter-style rules.
    pub counter_styles: &'a mut FnvHashMap<Atom, Arc<Locked<CounterStyleRule>>>,
}

#[cfg(feature = "gecko")]
impl<'a> ExtraStyleData<'a> {
    /// Clear the internal data.
    fn clear(&mut self) {
        self.font_faces.clear();
        self.counter_styles.clear();
    }

    /// Add the given @font-face rule.
    fn add_font_face(&mut self, rule: &Arc<Locked<FontFaceRule>>, origin: Origin) {
        self.font_faces.push((rule.clone(), origin));
    }

    /// Add the given @counter-style rule.
    fn add_counter_style(&mut self, guard: &SharedRwLockReadGuard,
                         rule: &Arc<Locked<CounterStyleRule>>) {
        let name = rule.read_with(guard).mName.raw::<nsIAtom>().into();
        self.counter_styles.insert(name, rule.clone());
    }
}

#[allow(missing_docs)]
#[cfg(feature = "servo")]
pub struct ExtraStyleData<'a> {
    pub marker: PhantomData<&'a usize>,
}

#[cfg(feature = "servo")]
impl<'a> ExtraStyleData<'a> {
    fn clear(&mut self) {}
}

/// What cascade levels to include when styling elements.
#[derive(Copy, Clone, PartialEq)]
pub enum RuleInclusion {
    /// Include rules for style sheets at all cascade levels.  This is the
    /// normal rule inclusion mode.
    All,
    /// Only include rules from UA and user level sheets.  Used to implement
    /// `getDefaultComputedStyle`.
    DefaultOnly,
}

#[cfg(feature = "gecko")]
impl From<StyleRuleInclusion> for RuleInclusion {
    fn from(value: StyleRuleInclusion) -> Self {
        match value {
            StyleRuleInclusion::All => RuleInclusion::All,
            StyleRuleInclusion::DefaultOnly => RuleInclusion::DefaultOnly,
        }
    }
}

/// A filter that filters over effective rules, but allowing all potentially
/// effective `@media` rules.
pub struct PotentiallyEffectiveMediaRules;

impl NestedRuleIterationCondition for PotentiallyEffectiveMediaRules {
    fn process_import(
        _: &SharedRwLockReadGuard,
        _: &Device,
        _: QuirksMode,
        _: &ImportRule)
        -> bool
    {
        true
    }

    fn process_media(
        _: &SharedRwLockReadGuard,
        _: &Device,
        _: QuirksMode,
        _: &MediaRule)
        -> bool
    {
        true
    }

    /// Whether we should process the nested rules in a given `@-moz-document` rule.
    fn process_document(
        guard: &SharedRwLockReadGuard,
        device: &Device,
        quirks_mode: QuirksMode,
        rule: &DocumentRule)
        -> bool
    {
        use stylesheets::EffectiveRules;
        EffectiveRules::process_document(guard, device, quirks_mode, rule)
    }

    /// Whether we should process the nested rules in a given `@supports` rule.
    fn process_supports(
        guard: &SharedRwLockReadGuard,
        device: &Device,
        quirks_mode: QuirksMode,
        rule: &SupportsRule)
        -> bool
    {
        use stylesheets::EffectiveRules;
        EffectiveRules::process_supports(guard, device, quirks_mode, rule)
    }
}


impl Stylist {
    /// Construct a new `Stylist`, using given `Device` and `QuirksMode`.
    /// If more members are added here, think about whether they should
    /// be reset in clear().
    #[inline]
    pub fn new(device: Device, quirks_mode: QuirksMode) -> Self {
        let mut stylist = Stylist {
            viewport_constraints: None,
            device: device,
            is_device_dirty: true,
            is_cleared: true,
            quirks_mode: quirks_mode,

            element_map: PerPseudoElementSelectorMap::new(),
            pseudos_map: Default::default(),
            animations: Default::default(),
            precomputed_pseudo_element_decls: Default::default(),
            rules_source_order: 0,
            rule_tree: RuleTree::new(),
            dependencies: DependencySet::new(),
            attribute_dependencies: BloomFilter::new(),
            style_attribute_dependency: false,
            state_dependencies: ElementState::empty(),
            selectors_for_cache_revalidation: SelectorMap::new(),
            num_selectors: 0,
            num_declarations: 0,
            num_rebuilds: 0,
        };

        SelectorImpl::each_eagerly_cascaded_pseudo_element(|pseudo| {
            stylist.pseudos_map.insert(pseudo, PerPseudoElementSelectorMap::new());
        });

        // FIXME: Add iso-8859-9.css when the document’s encoding is ISO-8859-8.

        stylist
    }

    /// Returns the number of selectors.
    pub fn num_selectors(&self) -> usize {
        self.num_selectors
    }

    /// Returns the number of declarations.
    pub fn num_declarations(&self) -> usize {
        self.num_declarations
    }

    /// Returns the number of times the stylist has been rebuilt.
    pub fn num_rebuilds(&self) -> usize {
        self.num_rebuilds
    }

    /// Returns the number of dependencies in the DependencySet.
    pub fn num_dependencies(&self) -> usize {
        self.dependencies.len()
    }

    /// Returns the number of revalidation_selectors.
    pub fn num_revalidation_selectors(&self) -> usize {
        self.selectors_for_cache_revalidation.len()
    }

    /// Clear the stylist's state, effectively resetting it to more or less
    /// the state Stylist::new creates.
    ///
    /// We preserve the state of the following members:
    ///   device: Someone might have set this on us.
    ///   quirks_mode: Again, someone might have set this on us.
    ///   num_rebuilds: clear() followed by rebuild() should just increment this
    ///
    /// We don't just use struct update syntax with Stylist::new(self.device)
    /// beause for some of our members we can clear them instead of creating new
    /// objects.  This does cause unfortunate code duplication with
    /// Stylist::new.
    pub fn clear(&mut self) {
        if self.is_cleared {
            return
        }

        self.is_cleared = true;

        self.viewport_constraints = None;
        // preserve current device
        self.is_device_dirty = true;
        // preserve current quirks_mode value
        self.element_map = PerPseudoElementSelectorMap::new();
        self.pseudos_map = Default::default();
        self.animations.clear(); // Or set to Default::default()?
        self.precomputed_pseudo_element_decls = Default::default();
        self.rules_source_order = 0;
        // We want to keep rule_tree around across stylist rebuilds.
        self.dependencies.clear();
        self.attribute_dependencies.clear();
        self.style_attribute_dependency = false;
        self.state_dependencies = ElementState::empty();
        self.selectors_for_cache_revalidation = SelectorMap::new();
        self.num_selectors = 0;
        self.num_declarations = 0;
        // preserve num_rebuilds value, since it should stay across
        // clear()/rebuild() cycles.
    }

    /// rebuild the stylist for the given document stylesheets, and optionally
    /// with a set of user agent stylesheets.
    ///
    /// This method resets all the style data each time the stylesheets change
    /// (which is indicated by the `stylesheets_changed` parameter), or the
    /// device is dirty, which means we need to re-evaluate media queries.
    pub fn rebuild<'a, 'b, I>(&mut self,
                              doc_stylesheets: I,
                              guards: &StylesheetGuards,
                              ua_stylesheets: Option<&UserAgentStylesheets>,
                              stylesheets_changed: bool,
                              author_style_disabled: bool,
                              extra_data: &mut ExtraStyleData<'a>) -> bool
        where I: Iterator<Item = &'b Arc<Stylesheet>> + Clone,
    {
        debug_assert!(!self.is_cleared || self.is_device_dirty);

        self.is_cleared = false;

        if !(self.is_device_dirty || stylesheets_changed) {
            return false;
        }

        self.num_rebuilds += 1;

        let cascaded_rule = ViewportRule {
            declarations: viewport::Cascade::from_stylesheets(
                doc_stylesheets.clone(), guards.author, &self.device
            ).finish(),
        };

        self.viewport_constraints =
            ViewportConstraints::maybe_new(&self.device, &cascaded_rule, self.quirks_mode);

        if let Some(ref constraints) = self.viewport_constraints {
            self.device.account_for_viewport_rule(constraints);
        }

        SelectorImpl::each_eagerly_cascaded_pseudo_element(|pseudo| {
            self.pseudos_map.insert(pseudo, PerPseudoElementSelectorMap::new());
        });

        extra_data.clear();

        if let Some(ua_stylesheets) = ua_stylesheets {
            for stylesheet in &ua_stylesheets.user_or_user_agent_stylesheets {
                self.add_stylesheet(&stylesheet, guards.ua_or_user, extra_data);
            }

            if self.quirks_mode != QuirksMode::NoQuirks {
                self.add_stylesheet(&ua_stylesheets.quirks_mode_stylesheet,
                                    guards.ua_or_user, extra_data);
            }
        }

        // Only use author stylesheets if author styles are enabled.
        let sheets_to_add = doc_stylesheets.filter(|s| {
            !author_style_disabled || s.origin != Origin::Author
        });

        for ref stylesheet in sheets_to_add {
            self.add_stylesheet(stylesheet, guards.author, extra_data);
        }

        SelectorImpl::each_precomputed_pseudo_element(|pseudo| {
            if let Some(map) = self.pseudos_map.remove(&pseudo) {
                let declarations = map.user_agent.get_universal_rules(CascadeLevel::UANormal);
                self.precomputed_pseudo_element_decls.insert(pseudo, declarations);
            }
        });

        self.is_device_dirty = false;
        true
    }

    /// clear the stylist and then rebuild it.  Chances are, you want to use
    /// either clear() or rebuild(), with the latter done lazily, instead.
    pub fn update<'a, 'b, I>(&mut self,
                             doc_stylesheets: I,
                             guards: &StylesheetGuards,
                             ua_stylesheets: Option<&UserAgentStylesheets>,
                             stylesheets_changed: bool,
                             author_style_disabled: bool,
                             extra_data: &mut ExtraStyleData<'a>) -> bool
        where I: Iterator<Item = &'b Arc<Stylesheet>> + Clone,
    {
        debug_assert!(!self.is_cleared || self.is_device_dirty);

        // We have to do a dirtiness check before clearing, because if
        // we're not actually dirty we need to no-op here.
        if !(self.is_device_dirty || stylesheets_changed) {
            return false;
        }
        self.clear();
        self.rebuild(doc_stylesheets, guards, ua_stylesheets, stylesheets_changed,
                     author_style_disabled, extra_data)
    }

    fn add_stylesheet<'a>(&mut self,
                          stylesheet: &Stylesheet,
                          guard: &SharedRwLockReadGuard,
                          _extra_data: &mut ExtraStyleData<'a>) {
        if stylesheet.disabled() || !stylesheet.is_effective_for_device(&self.device, guard) {
            return;
        }

        for rule in stylesheet.effective_rules(&self.device, guard) {
            match *rule {
                CssRule::Style(ref locked) => {
                    let style_rule = locked.read_with(&guard);
                    self.num_declarations += style_rule.block.read_with(&guard).len();
                    for selector in &style_rule.selectors.0 {
                        self.num_selectors += 1;

                        let map = if let Some(pseudo) = selector.pseudo_element() {
                            self.pseudos_map
                                .entry(pseudo.canonical())
                                .or_insert_with(PerPseudoElementSelectorMap::new)
                                .borrow_for_origin(&stylesheet.origin)
                        } else {
                            self.element_map.borrow_for_origin(&stylesheet.origin)
                        };

                        map.insert(Rule::new(selector.clone(),
                                             locked.clone(),
                                             self.rules_source_order));

                        self.dependencies.note_selector(selector);
                        if needs_revalidation(selector) {
                            self.selectors_for_cache_revalidation.insert(selector.inner.clone());
                        }
                        selector.visit(&mut AttributeAndStateDependencyVisitor {
                            attribute_dependencies: &mut self.attribute_dependencies,
                            style_attribute_dependency: &mut self.style_attribute_dependency,
                            state_dependencies: &mut self.state_dependencies,
                        });
                    }
                    self.rules_source_order += 1;
                }
                CssRule::Import(..) => {
                    // effective_rules visits the inner stylesheet if
                    // appropriate.
                }
                CssRule::Keyframes(ref keyframes_rule) => {
                    let keyframes_rule = keyframes_rule.read_with(guard);
                    debug!("Found valid keyframes rule: {:?}", *keyframes_rule);

                    // Don't let a prefixed keyframes animation override a non-prefixed one.
                    let needs_insertion = keyframes_rule.vendor_prefix.is_none() ||
                        self.animations.get(keyframes_rule.name.as_atom()).map_or(true, |rule|
                            rule.vendor_prefix.is_some());
                    if needs_insertion {
                        let animation = KeyframesAnimation::from_keyframes(
                            &keyframes_rule.keyframes, keyframes_rule.vendor_prefix.clone(), guard);
                        debug!("Found valid keyframe animation: {:?}", animation);
                        self.animations.insert(keyframes_rule.name.as_atom().clone(), animation);
                    }
                }
                #[cfg(feature = "gecko")]
                CssRule::FontFace(ref rule) => {
                    _extra_data.add_font_face(&rule, stylesheet.origin);
                }
                #[cfg(feature = "gecko")]
                CssRule::CounterStyle(ref rule) => {
                    _extra_data.add_counter_style(guard, &rule);
                }
                // We don't care about any other rule.
                _ => {}
            }
        }
    }

    /// Returns whether the given attribute might appear in an attribute
    /// selector of some rule in the stylist.
    pub fn might_have_attribute_dependency(&self,
                                           local_name: &LocalName)
                                           -> bool {
        if *local_name == local_name!("style") {
            self.style_attribute_dependency
        } else {
            self.attribute_dependencies.might_contain(local_name)
        }
    }

    /// Returns whether the given ElementState bit is relied upon by a selector
    /// of some rule in the stylist.
    pub fn has_state_dependency(&self, state: ElementState) -> bool {
        self.state_dependencies.intersects(state)
    }

    /// Computes the style for a given "precomputed" pseudo-element, taking the
    /// universal rules and applying them.
    ///
    /// If `inherit_all` is true, then all properties are inherited from the
    /// parent; otherwise, non-inherited properties are reset to their initial
    /// values. The flow constructor uses this flag when constructing anonymous
    /// flows.
    pub fn precomputed_values_for_pseudo(&self,
                                         guards: &StylesheetGuards,
                                         pseudo: &PseudoElement,
                                         parent: Option<&Arc<ComputedValues>>,
                                         cascade_flags: CascadeFlags,
                                         font_metrics: &FontMetricsProvider)
                                         -> ComputedStyle {
        debug_assert!(pseudo.is_precomputed());

        let rule_node = match self.precomputed_pseudo_element_decls.get(pseudo) {
            Some(declarations) => {
                // FIXME(emilio): When we've taken rid of the cascade we can just
                // use into_iter.
                self.rule_tree.insert_ordered_rules_with_important(
                    declarations.into_iter().map(|a| (a.source.clone(), a.level)),
                    guards)
            }
            None => self.rule_tree.root(),
        };

        // NOTE(emilio): We skip calculating the proper layout parent style
        // here.
        //
        // It'd be fine to assert that this isn't called with a parent style
        // where display contents is in effect, but in practice this is hard to
        // do for stuff like :-moz-fieldset-content with a
        // <fieldset style="display: contents">. That is, the computed value of
        // display for the fieldset is "contents", even though it's not the used
        // value, so we don't need to adjust in a different way anyway.
        //
        // In practice, I don't think any anonymous content can be a direct
        // descendant of a display: contents element where display: contents is
        // the actual used value, and the computed value of it would need
        // blockification.
        let computed =
            properties::cascade(&self.device,
                                &rule_node,
                                guards,
                                parent.map(|p| &**p),
                                parent.map(|p| &**p),
                                None,
                                None,
                                &RustLogReporter,
                                font_metrics,
                                cascade_flags,
                                self.quirks_mode);
        ComputedStyle::new(rule_node, Arc::new(computed))
    }

    /// Returns the style for an anonymous box of the given type.
    #[cfg(feature = "servo")]
    pub fn style_for_anonymous(&self,
                               guards: &StylesheetGuards,
                               pseudo: &PseudoElement,
                               parent_style: &Arc<ComputedValues>)
                               -> Arc<ComputedValues> {
        use font_metrics::ServoMetricsProvider;

        // For most (but not all) pseudo-elements, we inherit all values from the parent.
        let inherit_all = match *pseudo {
            PseudoElement::ServoText |
            PseudoElement::ServoInputText => false,
            PseudoElement::ServoAnonymousBlock |
            PseudoElement::ServoAnonymousTable |
            PseudoElement::ServoAnonymousTableCell |
            PseudoElement::ServoAnonymousTableRow |
            PseudoElement::ServoAnonymousTableWrapper |
            PseudoElement::ServoTableWrapper |
            PseudoElement::ServoInlineBlockWrapper |
            PseudoElement::ServoInlineAbsolute => true,
            PseudoElement::Before |
            PseudoElement::After |
            PseudoElement::Selection |
            PseudoElement::DetailsSummary |
            PseudoElement::DetailsContent => {
                unreachable!("That pseudo doesn't represent an anonymous box!")
            }
        };
        let mut cascade_flags = CascadeFlags::empty();
        if inherit_all {
            cascade_flags.insert(INHERIT_ALL);
        }
        self.precomputed_values_for_pseudo(guards, &pseudo, Some(parent_style), cascade_flags,
                                           &ServoMetricsProvider)
            .values.unwrap()
    }

    /// Computes a pseudo-element style lazily during layout.
    ///
    /// This can only be done for a certain set of pseudo-elements, like
    /// :selection.
    ///
    /// Check the documentation on lazy pseudo-elements in
    /// docs/components/style.md
    pub fn lazily_compute_pseudo_element_style<E>(&self,
                                                  guards: &StylesheetGuards,
                                                  element: &E,
                                                  pseudo: &PseudoElement,
                                                  rule_inclusion: RuleInclusion,
                                                  parent_style: &ComputedValues,
                                                  font_metrics: &FontMetricsProvider)
                                                  -> Option<ComputedStyle>
        where E: TElement,
    {
        let rule_node =
            match self.lazy_pseudo_rules(guards, element, pseudo, rule_inclusion) {
                Some(rule_node) => rule_node,
                None => return None
            };

        // Read the comment on `precomputed_values_for_pseudo` to see why it's
        // difficult to assert that display: contents nodes never arrive here
        // (tl;dr: It doesn't apply for replaced elements and such, but the
        // computed value is still "contents").
        // Bug 1364242: We need to add visited support for lazy pseudos
        let computed =
            properties::cascade(&self.device,
                                &rule_node,
                                guards,
                                Some(parent_style),
                                Some(parent_style),
                                None,
                                None,
                                &RustLogReporter,
                                font_metrics,
                                CascadeFlags::empty(),
                                self.quirks_mode);

        Some(ComputedStyle::new(rule_node, Arc::new(computed)))
    }

    /// Computes the rule node for a lazily-cascaded pseudo-element.
    ///
    /// See the documentation on lazy pseudo-elements in
    /// docs/components/style.md
    pub fn lazy_pseudo_rules<E>(&self,
                                guards: &StylesheetGuards,
                                element: &E,
                                pseudo: &PseudoElement,
                                rule_inclusion: RuleInclusion)
                                -> Option<StrongRuleNode>
        where E: TElement
    {
        let pseudo = pseudo.canonical();
        debug_assert!(pseudo.is_lazy());
        if self.pseudos_map.get(&pseudo).is_none() {
            return None
        }

        // Apply the selector flags. We should be in sequential mode
        // already, so we can directly apply the parent flags.
        let mut set_selector_flags = |element: &E, flags: ElementSelectorFlags| {
            if cfg!(feature = "servo") {
                // Servo calls this function from the worker, but only for internal
                // pseudos, so we should never generate selector flags here.
                unreachable!("internal pseudo generated slow selector flags?");
            }

            // No need to bother setting the selector flags when we're computing
            // default styles.
            if rule_inclusion == RuleInclusion::DefaultOnly {
                return;
            }

            // Gecko calls this from sequential mode, so we can directly apply
            // the flags.
            debug_assert!(thread_state::get() == thread_state::LAYOUT);
            let self_flags = flags.for_self();
            if !self_flags.is_empty() {
                unsafe { element.set_selector_flags(self_flags); }
            }
            let parent_flags = flags.for_parent();
            if !parent_flags.is_empty() {
                if let Some(p) = element.parent_element() {
                    unsafe { p.set_selector_flags(parent_flags); }
                }
            }
        };

        // Bug 1364242: We need to add visited support for lazy pseudos
        let mut declarations = ApplicableDeclarationList::new();
        let mut matching_context =
            MatchingContext::new(MatchingMode::ForStatelessPseudoElement, None);
        self.push_applicable_declarations(element,
                                          Some(&pseudo),
                                          None,
                                          None,
                                          AnimationRules(None, None),
                                          rule_inclusion,
                                          &mut declarations,
                                          &mut matching_context,
                                          &mut set_selector_flags);
        if declarations.is_empty() {
            return None
        }

        let rule_node =
            self.rule_tree.insert_ordered_rules_with_important(
                declarations.into_iter().map(|a| (a.source, a.level)),
                guards);
        if rule_node == self.rule_tree.root() {
            None
        } else {
            Some(rule_node)
        }
    }

    /// Set a given device, which may change the styles that apply to the
    /// document.
    ///
    /// This means that we may need to rebuild style data even if the
    /// stylesheets haven't changed.
    ///
    /// Also, the device that arrives here may need to take the viewport rules
    /// into account.
    ///
    /// TODO(emilio): Probably should be unified with `update`, right now I
    /// don't think we take into account dynamic updates to viewport rules.
    ///
    /// Probably worth to make the stylist own a single `Device`, and have a
    /// `update_device` function?
    ///
    /// feature = "servo" because gecko only has one device, and manually tracks
    /// when the device is dirty.
    ///
    /// FIXME(emilio): The semantics of the device for Servo and Gecko are
    /// different enough we may want to unify them.
    #[cfg(feature = "servo")]
    pub fn set_device(&mut self,
                      mut device: Device,
                      guard: &SharedRwLockReadGuard,
                      stylesheets: &[Arc<Stylesheet>]) {
        let cascaded_rule = ViewportRule {
            declarations: viewport::Cascade::from_stylesheets(stylesheets.iter(), guard, &device).finish(),
        };

        self.viewport_constraints =
            ViewportConstraints::maybe_new(&device, &cascaded_rule, self.quirks_mode);

        if let Some(ref constraints) = self.viewport_constraints {
            device.account_for_viewport_rule(constraints);
        }

        self.is_device_dirty |= stylesheets.iter().any(|stylesheet| {
            let mq = stylesheet.media.read_with(guard);
            if mq.evaluate(&self.device, self.quirks_mode) != mq.evaluate(&device, self.quirks_mode) {
                return true
            }

            let mut iter =
                stylesheet.iter_rules::<PotentiallyEffectiveMediaRules>(
                    &self.device,
                    guard);

            while let Some(rule) = iter.next() {
                match *rule {
                    CssRule::Style(..) |
                    CssRule::Namespace(..) |
                    CssRule::FontFace(..) |
                    CssRule::CounterStyle(..) |
                    CssRule::Supports(..) |
                    CssRule::Keyframes(..) |
                    CssRule::Page(..) |
                    CssRule::Viewport(..) |
                    CssRule::Document(..) => {
                        // Not affected by device changes.
                        continue;
                    }
                    CssRule::Import(ref lock) => {
                        let import_rule = lock.read_with(guard);
                        let mq = import_rule.stylesheet.media.read_with(guard);
                        let effective_now = mq.evaluate(&self.device, self.quirks_mode);
                        if effective_now != mq.evaluate(&device, self.quirks_mode) {
                            return true;
                        }

                        if !effective_now {
                            iter.skip_children();
                        }
                    }
                    CssRule::Media(ref lock) => {
                        let media_rule = lock.read_with(guard);
                        let mq = media_rule.media_queries.read_with(guard);
                        let effective_now = mq.evaluate(&self.device, self.quirks_mode);
                        if effective_now != mq.evaluate(&device, self.quirks_mode) {
                            return true;
                        }

                        if !effective_now {
                            iter.skip_children();
                        }
                    }
                }
            }

            return false;
        });

        self.device = device;
    }

    /// Returns the viewport constraints that apply to this document because of
    /// a @viewport rule.
    pub fn viewport_constraints(&self) -> Option<&ViewportConstraints> {
        self.viewport_constraints.as_ref()
    }

    /// Returns the Quirks Mode of the document.
    pub fn quirks_mode(&self) -> QuirksMode {
        self.quirks_mode
    }

    /// Sets the quirks mode of the document.
    pub fn set_quirks_mode(&mut self, quirks_mode: QuirksMode) {
        // FIXME(emilio): We don't seem to change the quirks mode dynamically
        // during multiple layout passes, but this is totally bogus, in the
        // sense that it's updated asynchronously.
        //
        // This should probably be an argument to `update`, and use the quirks
        // mode info in the `SharedLayoutContext`.
        self.quirks_mode = quirks_mode;
    }

    /// Returns the applicable CSS declarations for the given element.
    ///
    /// This corresponds to `ElementRuleCollector` in WebKit.
    ///
    /// The `StyleRelations` recorded in `MatchingContext` indicate hints about
    /// which kind of rules have matched.
    pub fn push_applicable_declarations<E, V, F>(
                                        &self,
                                        element: &E,
                                        pseudo_element: Option<&PseudoElement>,
                                        style_attribute: Option<&Arc<Locked<PropertyDeclarationBlock>>>,
                                        smil_override: Option<&Arc<Locked<PropertyDeclarationBlock>>>,
                                        animation_rules: AnimationRules,
                                        rule_inclusion: RuleInclusion,
                                        applicable_declarations: &mut V,
                                        context: &mut MatchingContext,
                                        flags_setter: &mut F)
        where E: TElement,
              V: Push<ApplicableDeclarationBlock> + VecLike<ApplicableDeclarationBlock> + ::std::fmt::Debug,
              F: FnMut(&E, ElementSelectorFlags),
    {
        debug_assert!(!self.is_device_dirty);
        // Gecko definitely has pseudo-elements with style attributes, like
        // ::-moz-color-swatch.
        debug_assert!(cfg!(feature = "gecko") ||
                      style_attribute.is_none() || pseudo_element.is_none(),
                      "Style attributes do not apply to pseudo-elements");
        debug_assert!(pseudo_element.map_or(true, |p| !p.is_precomputed()));

        let map = match pseudo_element {
            Some(pseudo) => self.pseudos_map.get(pseudo).unwrap(),
            None => &self.element_map,
        };

        let is_implemented_pseudo =
            element.implemented_pseudo_element().is_some();

        // NB: This causes use to rule has pseudo selectors based on the
        // properties of the originating element (which is fine, given the
        // find_first_from_right usage).
        let rule_hash_target = if is_implemented_pseudo {
            element.closest_non_native_anonymous_ancestor().unwrap()
        } else {
            *element
        };

        debug!("Determining if style is shareable: pseudo: {}",
               pseudo_element.is_some());

        let only_default_rules = rule_inclusion == RuleInclusion::DefaultOnly;

        // Step 1: Normal user-agent rules.
        map.user_agent.get_all_matching_rules(element,
                                              &rule_hash_target,
                                              applicable_declarations,
                                              context,
                                              flags_setter,
                                              CascadeLevel::UANormal);
        debug!("UA normal: {:?}", context.relations);

        if pseudo_element.is_none() && !only_default_rules {
            // Step 2: Presentational hints.
            let length_before_preshints = applicable_declarations.len();
            element.synthesize_presentational_hints_for_legacy_attributes(applicable_declarations);
            if applicable_declarations.len() != length_before_preshints {
                if cfg!(debug_assertions) {
                    for declaration in &applicable_declarations[length_before_preshints..] {
                        assert_eq!(declaration.level, CascadeLevel::PresHints);
                    }
                }
                // Note the existence of presentational attributes so that the
                // style sharing cache can avoid re-querying them if they don't
                // exist.
                context.relations |= AFFECTED_BY_PRESENTATIONAL_HINTS;
            }
            debug!("preshints: {:?}", context.relations);
        }

        // NB: the following condition, although it may look somewhat
        // inaccurate, would be equivalent to something like:
        //
        //     element.matches_user_and_author_rules() ||
        //     (is_implemented_pseudo &&
        //      rule_hash_target.matches_user_and_author_rules())
        //
        // Which may be more what you would probably expect.
        if rule_hash_target.matches_user_and_author_rules() {
            // Step 3a: User normal rules.
            map.user.get_all_matching_rules(element,
                                            &rule_hash_target,
                                            applicable_declarations,
                                            context,
                                            flags_setter,
                                            CascadeLevel::UserNormal);
            debug!("user normal: {:?}", context.relations);
        } else {
            debug!("skipping user rules");
        }

        if rule_hash_target.matches_user_and_author_rules() && !only_default_rules {
            // Step 3b: Author normal rules.
            map.author.get_all_matching_rules(element,
                                              &rule_hash_target,
                                              applicable_declarations,
                                              context,
                                              flags_setter,
                                              CascadeLevel::AuthorNormal);
            debug!("author normal: {:?}", context.relations);

            // Step 4: Normal style attributes.
            if let Some(sa) = style_attribute {
                Push::push(
                    applicable_declarations,
                    ApplicableDeclarationBlock::from_declarations(sa.clone(),
                                                                  CascadeLevel::StyleAttributeNormal));
            }

            debug!("style attr: {:?}", context.relations);

            // Step 5: SMIL override.
            // Declarations from SVG SMIL animation elements.
            if let Some(so) = smil_override {
                Push::push(
                    applicable_declarations,
                    ApplicableDeclarationBlock::from_declarations(so.clone(),
                                                                  CascadeLevel::SMILOverride));
            }
            debug!("SMIL: {:?}", context.relations);

            // Step 6: Animations.
            // The animations sheet (CSS animations, script-generated animations,
            // and CSS transitions that are no longer tied to CSS markup)
            if let Some(anim) = animation_rules.0 {
                Push::push(
                    applicable_declarations,
                    ApplicableDeclarationBlock::from_declarations(anim.clone(),
                                                                  CascadeLevel::Animations));
            }
            debug!("animation: {:?}", context.relations);
        } else {
            debug!("skipping non-agent rules");
        }

        //
        // Steps 7-10 correspond to !important rules, and are handled during
        // rule tree insertion.
        //

        if !only_default_rules {
            // Step 11: Transitions.
            // The transitions sheet (CSS transitions that are tied to CSS markup)
            if let Some(anim) = animation_rules.1 {
                Push::push(
                    applicable_declarations,
                    ApplicableDeclarationBlock::from_declarations(anim.clone(),
                                                                  CascadeLevel::Transitions));
            }
            debug!("transition: {:?}", context.relations);
        } else {
            debug!("skipping transition rules");
        }

        debug!("push_applicable_declarations: shareable: {:?}", context.relations);
    }

    /// Return whether the device is dirty, that is, whether the screen size or
    /// media type have changed (for now).
    #[inline]
    pub fn is_device_dirty(&self) -> bool {
        self.is_device_dirty
    }

    /// Returns the map of registered `@keyframes` animations.
    #[inline]
    pub fn animations(&self) -> &FnvHashMap<Atom, KeyframesAnimation> {
        &self.animations
    }

    /// Returns the rule root node.
    #[inline]
    pub fn rule_tree_root(&self) -> StrongRuleNode {
        self.rule_tree.root()
    }

    /// Computes the match results of a given element against the set of
    /// revalidation selectors.
    pub fn match_revalidation_selectors<E, F>(&self,
                                              element: &E,
                                              bloom: &BloomFilter,
                                              flags_setter: &mut F)
                                              -> BitVec
        where E: TElement,
              F: FnMut(&E, ElementSelectorFlags),
    {
        // NB: `MatchingMode` doesn't really matter, given we don't share style
        // between pseudos.
        let mut matching_context =
            MatchingContext::new(MatchingMode::Normal, Some(bloom));

        // Note that, by the time we're revalidating, we're guaranteed that the
        // candidate and the entry have the same id, classes, and local name.
        // This means we're guaranteed to get the same rulehash buckets for all
        // the lookups, which means that the bitvecs are comparable. We verify
        // this in the caller by asserting that the bitvecs are same-length.
        let mut results = BitVec::new();
        self.selectors_for_cache_revalidation.lookup(*element, &mut |selector| {
            results.push(matches_selector(selector,
                                          element,
                                          &mut matching_context,
                                          flags_setter));
            true
        });

        results
    }

    /// Given an element, and a snapshot table that represents a previous state
    /// of the tree, compute the appropriate restyle hint, that is, the kind of
    /// restyle we need to do.
    pub fn compute_restyle_hint<'a, E>(&self,
                                       element: &E,
                                       shared_context: &SharedStyleContext,
                                       context: HintComputationContext<'a, E>)
                                       -> RestyleHint
        where E: TElement,
    {
        self.dependencies.compute_hint(element, shared_context, context)
    }

    /// Computes styles for a given declaration with parent_style.
    pub fn compute_for_declarations(&self,
                                    guards: &StylesheetGuards,
                                    parent_style: &Arc<ComputedValues>,
                                    declarations: Arc<Locked<PropertyDeclarationBlock>>)
                                    -> Arc<ComputedValues> {
        use font_metrics::get_metrics_provider_for_product;

        let v = vec![
            ApplicableDeclarationBlock::from_declarations(declarations.clone(),
                                                          CascadeLevel::StyleAttributeNormal)
        ];
        let rule_node =
            self.rule_tree.insert_ordered_rules(v.into_iter().map(|a| (a.source, a.level)));

        // This currently ignores visited styles.  It appears to be used for
        // font styles in <canvas> via Servo_StyleSet_ResolveForDeclarations.
        // It is unclear if visited styles are meaningful for this case.
        let metrics = get_metrics_provider_for_product();
        Arc::new(properties::cascade(&self.device,
                                     &rule_node,
                                     guards,
                                     Some(parent_style),
                                     Some(parent_style),
                                     None,
                                     None,
                                     &RustLogReporter,
                                     &metrics,
                                     CascadeFlags::empty(),
                                     self.quirks_mode))
    }

    /// Accessor for a shared reference to the device.
    pub fn device(&self) -> &Device {
        &self.device
    }

    /// Accessor for a mutable reference to the device.
    pub fn device_mut(&mut self) -> &mut Device {
        &mut self.device
    }

    /// Accessor for a shared reference to the rule tree.
    pub fn rule_tree(&self) -> &RuleTree {
        &self.rule_tree
    }
}

impl Drop for Stylist {
    fn drop(&mut self) {
        // This is the last chance to GC the rule tree.  If we have dropped all
        // strong rule node references before the Stylist is dropped, then this
        // will cause the rule tree to be destroyed correctly.  If we haven't
        // dropped all strong rule node references before now, then we will
        // leak them, since there will be no way to call gc() on the rule tree
        // after this point.
        //
        // TODO(emilio): We can at least assert all the elements in the free
        // list are indeed free.
        unsafe { self.rule_tree.gc(); }
    }
}

/// Visitor to collect names that appear in attribute selectors and any
/// dependencies on ElementState bits.
struct AttributeAndStateDependencyVisitor<'a> {
    attribute_dependencies: &'a mut BloomFilter,
    style_attribute_dependency: &'a mut bool,
    state_dependencies: &'a mut ElementState,
}

impl<'a> SelectorVisitor for AttributeAndStateDependencyVisitor<'a> {
    type Impl = SelectorImpl;

    fn visit_attribute_selector(&mut self, _ns: &NamespaceConstraint<&Namespace>,
                                name: &LocalName, lower_name: &LocalName)
                                -> bool {
        #[cfg(feature = "servo")]
        let style_lower_name = local_name!("style");
        #[cfg(feature = "gecko")]
        let style_lower_name = atom!("style");

        if *lower_name == style_lower_name {
            *self.style_attribute_dependency = true;
        } else {
            self.attribute_dependencies.insert(&name);
            self.attribute_dependencies.insert(&lower_name);
        }
        true
    }

    fn visit_simple_selector(&mut self, s: &Component<SelectorImpl>) -> bool {
        if let Component::NonTSPseudoClass(ref p) = *s {
            self.state_dependencies.insert(p.state_flag());
        }
        true
    }
}

/// Visitor determine whether a selector requires cache revalidation.
///
/// Note that we just check simple selectors and eagerly return when the first
/// need for revalidation is found, so we don't need to store state on the
/// visitor.
///
/// Also, note that it's important to check the whole selector, due to cousins
/// sharing arbitrarily deep in the DOM, not just the rightmost part of it
/// (unfortunately, though).
///
/// With cousin sharing, we not only need to care about selectors in stuff like
/// foo:first-child, but also about selectors like p:first-child foo, since the
/// two parents may have shared style, and in that case we can test cousins
/// whose matching depends on the selector up in the chain.
///
/// TODO(emilio): We can optimize when matching only siblings to only match the
/// rightmost selector until a descendant combinator is found, I guess, and in
/// general when we're sharing at depth `n`, to the `n + 1` sequences of
/// descendant combinators.
///
/// I don't think that in presence of the bloom filter it's worth it, though.
struct RevalidationVisitor;

impl SelectorVisitor for RevalidationVisitor {
    type Impl = SelectorImpl;


    fn visit_complex_selector(&mut self,
                              _: SelectorIter<SelectorImpl>,
                              combinator: Option<Combinator>) -> bool {
        let is_sibling_combinator =
            combinator.map_or(false, |c| c.is_sibling());

        !is_sibling_combinator
    }


    /// Check whether sequence of simple selectors containing this simple
    /// selector to be explicitly matched against both the style sharing cache
    /// entry and the candidate.
    ///
    /// We use this for selectors that can have different matching behavior
    /// between siblings that are otherwise identical as far as the cache is
    /// concerned.
    fn visit_simple_selector(&mut self, s: &Component<SelectorImpl>) -> bool {
        match *s {
            Component::AttributeInNoNamespaceExists { .. } |
            Component::AttributeInNoNamespace { .. } |
            Component::AttributeOther(_) |
            Component::Empty |
            Component::FirstChild |
            Component::LastChild |
            Component::OnlyChild |
            Component::NthChild(..) |
            Component::NthLastChild(..) |
            Component::NthOfType(..) |
            Component::NthLastOfType(..) |
            Component::FirstOfType |
            Component::LastOfType |
            Component::OnlyOfType => {
                false
            },
            Component::NonTSPseudoClass(ref p) => {
                !p.needs_cache_revalidation()
            },
            _ => {
                true
            }
        }
    }
}

/// Returns true if the given selector needs cache revalidation.
pub fn needs_revalidation(selector: &Selector<SelectorImpl>) -> bool {
    let mut visitor = RevalidationVisitor;
    !selector.visit(&mut visitor)
}

/// Map that contains the CSS rules for a specific PseudoElement
/// (or lack of PseudoElement).
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[derive(Debug)]
struct PerPseudoElementSelectorMap {
    /// Rules from user agent stylesheets
    user_agent: SelectorMap<Rule>,
    /// Rules from author stylesheets
    author: SelectorMap<Rule>,
    /// Rules from user stylesheets
    user: SelectorMap<Rule>,
}

impl PerPseudoElementSelectorMap {
    #[inline]
    fn new() -> Self {
        PerPseudoElementSelectorMap {
            user_agent: SelectorMap::new(),
            author: SelectorMap::new(),
            user: SelectorMap::new(),
        }
    }

    #[inline]
    fn borrow_for_origin(&mut self, origin: &Origin) -> &mut SelectorMap<Rule> {
        match *origin {
            Origin::UserAgent => &mut self.user_agent,
            Origin::Author => &mut self.author,
            Origin::User => &mut self.user,
        }
    }
}

/// A rule, that wraps a style rule, but represents a single selector of the
/// rule.
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[derive(Clone, Debug)]
pub struct Rule {
    /// The selector this struct represents. We store this and the
    /// any_{important,normal} booleans inline in the Rule to avoid
    /// pointer-chasing when gathering applicable declarations, which
    /// can ruin performance when there are a lot of rules.
    #[cfg_attr(feature = "servo", ignore_heap_size_of = "Arc")]
    pub selector: Selector<SelectorImpl>,
    /// The actual style rule.
    #[cfg_attr(feature = "servo", ignore_heap_size_of = "Arc")]
    pub style_rule: Arc<Locked<StyleRule>>,
    /// The source order this style rule appears in.
    pub source_order: usize,
}

impl SelectorMapEntry for Rule {
    fn selector(&self) -> &SelectorInner<SelectorImpl> {
        &self.selector.inner
    }
}

impl Rule {
    /// Returns the specificity of the rule.
    pub fn specificity(&self) -> u32 {
        self.selector.specificity()
    }

    /// Turns this rule into an `ApplicableDeclarationBlock` for the given
    /// cascade level.
    pub fn to_applicable_declaration_block(&self,
                                           level: CascadeLevel)
                                           -> ApplicableDeclarationBlock {
        ApplicableDeclarationBlock {
            source: StyleSource::Style(self.style_rule.clone()),
            level: level,
            source_order: self.source_order,
            specificity: self.specificity(),
        }
    }

    /// Creates a new Rule.
    pub fn new(selector: Selector<SelectorImpl>,
               style_rule: Arc<Locked<StyleRule>>,
               source_order: usize)
               -> Self
    {
        Rule {
            selector: selector,
            style_rule: style_rule,
            source_order: source_order,
        }
    }
}

/// A property declaration together with its precedence among rules of equal
/// specificity so that we can sort them.
///
/// This represents the declarations in a given declaration block for a given
/// importance.
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[derive(Debug, Clone, PartialEq)]
pub struct ApplicableDeclarationBlock {
    /// The style source, either a style rule, or a property declaration block.
    #[cfg_attr(feature = "servo", ignore_heap_size_of = "Arc")]
    pub source: StyleSource,
    /// The cascade level this applicable declaration block is in.
    pub level: CascadeLevel,
    /// The source order of this block.
    pub source_order: usize,
    /// The specificity of the selector this block is represented by.
    pub specificity: u32,
}

impl ApplicableDeclarationBlock {
    /// Constructs an applicable declaration block from a given property
    /// declaration block and importance.
    #[inline]
    pub fn from_declarations(declarations: Arc<Locked<PropertyDeclarationBlock>>,
                             level: CascadeLevel)
                             -> Self {
        ApplicableDeclarationBlock {
            source: StyleSource::Declarations(declarations),
            level: level,
            source_order: 0,
            specificity: 0,
        }
    }
}
