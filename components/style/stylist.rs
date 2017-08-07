/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Selector matching.

use {Atom, LocalName, Namespace};
use applicable_declarations::{ApplicableDeclarationBlock, ApplicableDeclarationList};
use bit_vec::BitVec;
use context::{CascadeInputs, QuirksMode};
use dom::TElement;
use element_state::ElementState;
use font_metrics::FontMetricsProvider;
#[cfg(feature = "gecko")]
use gecko_bindings::structs::{nsIAtom, StyleRuleInclusion};
use invalidation::element::invalidation_map::InvalidationMap;
use invalidation::media_queries::{EffectiveMediaQueryResults, ToMediaListKey};
use media_queries::Device;
use properties::{self, CascadeFlags, ComputedValues};
use properties::{AnimationRules, PropertyDeclarationBlock};
#[cfg(feature = "servo")]
use properties::INHERIT_ALL;
use properties::IS_LINK;
use rule_tree::{CascadeLevel, RuleTree, StyleSource};
use selector_map::{SelectorMap, SelectorMapEntry};
use selector_parser::{SelectorImpl, PseudoElement};
use selectors::attr::NamespaceConstraint;
use selectors::bloom::BloomFilter;
use selectors::matching::{ElementSelectorFlags, matches_selector, MatchingContext, MatchingMode};
use selectors::matching::VisitedHandlingMode;
use selectors::parser::{AncestorHashes, Combinator, Component, Selector};
use selectors::parser::{SelectorIter, SelectorMethods};
use selectors::sink::Push;
use selectors::visitor::SelectorVisitor;
use servo_arc::{Arc, ArcBorrow};
use shared_lock::{Locked, SharedRwLockReadGuard, StylesheetGuards};
use smallvec::VecLike;
use std::fmt::Debug;
#[cfg(feature = "servo")]
use std::marker::PhantomData;
use style_traits::viewport::ViewportConstraints;
#[cfg(feature = "gecko")]
use stylesheets::{CounterStyleRule, FontFaceRule};
use stylesheets::{CssRule, StyleRule};
use stylesheets::{StylesheetInDocument, Origin, UserAgentStylesheets};
use stylesheets::keyframes_rule::KeyframesAnimation;
use stylesheets::viewport_rule::{self, MaybeNew, ViewportRule};
use thread_state;

pub use ::fnv::FnvHashMap;

/// This structure holds all the selectors and device characteristics
/// for a given document. The selectors are converted into `Rule`s
/// (defined in rust-selectors), and sorted into `SelectorMap`s keyed
/// off stylesheet origin and pseudo-element (see `CascadeData`).
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

    /// Effective media query results cached from the last rebuild.
    effective_media_query_results: EffectiveMediaQueryResults,

    /// If true, the quirks-mode stylesheet is applied.
    #[cfg_attr(feature = "servo", ignore_heap_size_of = "defined in selectors")]
    quirks_mode: QuirksMode,

    /// If true, the device has changed, and the stylist needs to be updated.
    is_device_dirty: bool,

    /// If true, the stylist is in a cleared state (e.g. just-constructed, or
    /// had clear() called on it with no following rebuild()).
    is_cleared: bool,

    /// Selector maps for all of the style sheets in the stylist, after
    /// evalutaing media rules against the current device, split out per
    /// cascade level.
    cascade_data: CascadeData,

    /// The rule tree, that stores the results of selector matching.
    rule_tree: RuleTree,

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
    rules_source_order: u32,

    /// The invalidation map for this document.
    invalidation_map: InvalidationMap,

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

    /// The ids that appear in the rightmost complex selector of selectors (and
    /// hence in our selector maps).  Used to determine when sharing styles is
    /// safe: we disallow style sharing for elements whose id matches this
    /// filter, and hence might be in one of our selector maps.
    ///
    /// FIXME(bz): This doesn't really need to be a counting Blooom filter.
    #[cfg_attr(feature = "servo", ignore_heap_size_of = "just an array")]
    mapped_ids: BloomFilter,

    /// Selectors that require explicit cache revalidation (i.e. which depend
    /// on state that is not otherwise visible to the cache, like attributes or
    /// tree-structural state like child index and pseudos).
    #[cfg_attr(feature = "servo", ignore_heap_size_of = "Arc")]
    selectors_for_cache_revalidation: SelectorMap<RevalidationSelectorAndHashes>,

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

impl Stylist {
    /// Construct a new `Stylist`, using given `Device` and `QuirksMode`.
    /// If more members are added here, think about whether they should
    /// be reset in clear().
    #[inline]
    pub fn new(device: Device, quirks_mode: QuirksMode) -> Self {
        Stylist {
            viewport_constraints: None,
            device: device,
            is_device_dirty: true,
            is_cleared: true,
            quirks_mode: quirks_mode,
            effective_media_query_results: EffectiveMediaQueryResults::new(),

            cascade_data: CascadeData::new(),
            animations: Default::default(),
            precomputed_pseudo_element_decls: Default::default(),
            rules_source_order: 0,
            rule_tree: RuleTree::new(),
            invalidation_map: InvalidationMap::new(),
            attribute_dependencies: BloomFilter::new(),
            style_attribute_dependency: false,
            state_dependencies: ElementState::empty(),
            mapped_ids: BloomFilter::new(),
            selectors_for_cache_revalidation: SelectorMap::new(),
            num_selectors: 0,
            num_declarations: 0,
            num_rebuilds: 0,
        }

        // FIXME: Add iso-8859-9.css when the document’s encoding is ISO-8859-8.
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

    /// Returns the number of revalidation_selectors.
    pub fn num_revalidation_selectors(&self) -> usize {
        self.selectors_for_cache_revalidation.len()
    }

    /// Gets a reference to the invalidation map.
    pub fn invalidation_map(&self) -> &InvalidationMap {
        &self.invalidation_map
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

        self.effective_media_query_results.clear();
        self.viewport_constraints = None;
        // preserve current device
        self.is_device_dirty = true;
        // preserve current quirks_mode value
        self.cascade_data.clear();
        self.animations.clear(); // Or set to Default::default()?
        self.precomputed_pseudo_element_decls = Default::default();
        self.rules_source_order = 0;
        // We want to keep rule_tree around across stylist rebuilds.
        self.invalidation_map.clear();
        self.attribute_dependencies.clear();
        self.style_attribute_dependency = false;
        self.state_dependencies = ElementState::empty();
        self.mapped_ids.clear();
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
    pub fn rebuild<'a, 'b, I, S>(
        &mut self,
        doc_stylesheets: I,
        guards: &StylesheetGuards,
        ua_stylesheets: Option<&UserAgentStylesheets>,
        stylesheets_changed: bool,
        author_style_disabled: bool,
        extra_data: &mut ExtraStyleData<'a>
    ) -> bool
    where
        I: Iterator<Item = &'b S> + Clone,
        S: StylesheetInDocument + ToMediaListKey + 'static,
    {
        debug_assert!(!self.is_cleared || self.is_device_dirty);

        self.is_cleared = false;

        if !(self.is_device_dirty || stylesheets_changed) {
            return false;
        }

        self.num_rebuilds += 1;

        self.viewport_constraints = None;

        if viewport_rule::enabled() {
            // TODO(emilio): This doesn't look so efficient.
            //
            // Presumably when we properly implement this we can at least have a
            // bit on the stylesheet that says whether it contains viewport
            // rules to skip it entirely?
            //
            // Processing it with the rest of rules seems tricky since it
            // overrides the viewport size which may change the evaluation of
            // media queries (or may not? how are viewport units in media
            // queries defined?)
            let cascaded_rule = ViewportRule {
                declarations: viewport_rule::Cascade::from_stylesheets(
                    doc_stylesheets.clone(), guards.author, &self.device
                ).finish()
            };

            self.viewport_constraints =
                ViewportConstraints::maybe_new(&self.device,
                                               &cascaded_rule,
                                               self.quirks_mode)
        }

        if let Some(ref constraints) = self.viewport_constraints {
            self.device.account_for_viewport_rule(constraints);
        }

        extra_data.clear();

        if let Some(ua_stylesheets) = ua_stylesheets {
            for stylesheet in &ua_stylesheets.user_or_user_agent_stylesheets {
                self.add_stylesheet(stylesheet, guards.ua_or_user, extra_data);
            }

            if self.quirks_mode != QuirksMode::NoQuirks {
                self.add_stylesheet(&ua_stylesheets.quirks_mode_stylesheet,
                                    guards.ua_or_user, extra_data);
            }
        }

        // Only use author stylesheets if author styles are enabled.
        let sheets_to_add = doc_stylesheets.filter(|s| {
            !author_style_disabled || s.origin(guards.author) != Origin::Author
        });

        for stylesheet in sheets_to_add {
            self.add_stylesheet(stylesheet, guards.author, extra_data);
        }

        SelectorImpl::each_precomputed_pseudo_element(|pseudo| {
            if let Some(map) = self.cascade_data.user_agent.pseudos_map.remove(&pseudo) {
                let declarations = map.get_universal_rules(CascadeLevel::UANormal);
                self.precomputed_pseudo_element_decls.insert(pseudo, declarations);
            }
        });

        self.is_device_dirty = false;
        true
    }

    /// clear the stylist and then rebuild it.  Chances are, you want to use
    /// either clear() or rebuild(), with the latter done lazily, instead.
    pub fn update<'a, 'b, I, S>(
        &mut self,
        doc_stylesheets: I,
        guards: &StylesheetGuards,
        ua_stylesheets: Option<&UserAgentStylesheets>,
        stylesheets_changed: bool,
        author_style_disabled: bool,
        extra_data: &mut ExtraStyleData<'a>
    ) -> bool
    where
        I: Iterator<Item = &'b S> + Clone,
        S: StylesheetInDocument + ToMediaListKey + 'static,
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

    fn add_stylesheet<'a, S>(
        &mut self,
        stylesheet: &S,
        guard: &SharedRwLockReadGuard,
        _extra_data: &mut ExtraStyleData<'a>
    )
    where
        S: StylesheetInDocument + ToMediaListKey + 'static,
    {
        if !stylesheet.enabled() ||
           !stylesheet.is_effective_for_device(&self.device, guard) {
            return;
        }

        self.effective_media_query_results.saw_effective(stylesheet);

        let origin = stylesheet.origin(guard);

        let origin_cascade_data =
            self.cascade_data.borrow_mut_for_origin(&origin);

        for rule in stylesheet.effective_rules(&self.device, guard) {
            match *rule {
                CssRule::Style(ref locked) => {
                    let style_rule = locked.read_with(&guard);
                    self.num_declarations += style_rule.block.read_with(&guard).len();
                    for selector in &style_rule.selectors.0 {
                        self.num_selectors += 1;

                        let hashes =
                            AncestorHashes::new(&selector, self.quirks_mode);

                        origin_cascade_data
                            .borrow_mut_for_pseudo_or_insert(selector.pseudo_element())
                            .insert(
                                Rule::new(selector.clone(),
                                          hashes.clone(),
                                          locked.clone(),
                                          self.rules_source_order),
                                self.quirks_mode);

                        self.invalidation_map.note_selector(selector, self.quirks_mode);
                        let mut visitor = StylistSelectorVisitor {
                            needs_revalidation: false,
                            passed_rightmost_selector: false,
                            attribute_dependencies: &mut self.attribute_dependencies,
                            style_attribute_dependency: &mut self.style_attribute_dependency,
                            state_dependencies: &mut self.state_dependencies,
                            mapped_ids: &mut self.mapped_ids,
                        };

                        selector.visit(&mut visitor);

                        if visitor.needs_revalidation {
                            self.selectors_for_cache_revalidation.insert(
                                RevalidationSelectorAndHashes::new(selector.clone(), hashes),
                                self.quirks_mode);
                        }
                    }
                    self.rules_source_order += 1;
                }
                CssRule::Import(ref lock) => {
                    let import_rule = lock.read_with(guard);
                    self.effective_media_query_results.saw_effective(import_rule);

                    // NOTE: effective_rules visits the inner stylesheet if
                    // appropriate.
                }
                CssRule::Media(ref lock) => {
                    let media_rule = lock.read_with(guard);
                    self.effective_media_query_results.saw_effective(media_rule);
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
                    _extra_data.add_font_face(&rule, origin);
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
        if self.is_cleared || self.is_device_dirty {
            // We can't tell what attributes are in our style rules until
            // we rebuild.
            true
        } else if *local_name == local_name!("style") {
            self.style_attribute_dependency
        } else {
            self.attribute_dependencies.might_contain_hash(local_name.get_hash())
        }
    }

    /// Returns whether the given ElementState bit might be relied upon by a
    /// selector of some rule in the stylist.
    pub fn might_have_state_dependency(&self, state: ElementState) -> bool {
        if self.is_cleared || self.is_device_dirty {
            // If self.is_cleared is true, we can't tell what states our style
            // rules rely on until we rebuild.
            true
        } else {
            self.state_dependencies.intersects(state)
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
                                         parent: Option<&ComputedValues>,
                                         cascade_flags: CascadeFlags,
                                         font_metrics: &FontMetricsProvider)
                                         -> Arc<ComputedValues> {
        debug_assert!(pseudo.is_precomputed());

        let rule_node = match self.precomputed_pseudo_element_decls.get(pseudo) {
            Some(declarations) => {
                self.rule_tree.insert_ordered_rules_with_important(
                    declarations.into_iter().map(|a| (a.source.clone(), a.level())),
                    guards
                )
            }
            None => self.rule_tree.root().clone(),
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
        properties::cascade(&self.device,
                            Some(pseudo),
                            &rule_node,
                            guards,
                            parent,
                            parent,
                            parent,
                            None,
                            None,
                            font_metrics,
                            cascade_flags,
                            self.quirks_mode)
    }

    /// Returns the style for an anonymous box of the given type.
    #[cfg(feature = "servo")]
    pub fn style_for_anonymous(&self,
                               guards: &StylesheetGuards,
                               pseudo: &PseudoElement,
                               parent_style: &ComputedValues)
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
                                                  is_probe: bool,
                                                  font_metrics: &FontMetricsProvider)
                                                  -> Option<Arc<ComputedValues>>
        where E: TElement,
    {
        let cascade_inputs =
            self.lazy_pseudo_rules(guards, element, pseudo, is_probe, rule_inclusion);
        self.compute_pseudo_element_style_with_inputs(&cascade_inputs,
                                                      pseudo,
                                                      guards,
                                                      parent_style,
                                                      font_metrics)
    }

    /// Computes a pseudo-element style lazily using the given CascadeInputs.
    /// This can be used for truly lazy pseudo-elements or to avoid redoing
    /// selector matching for eager pseudo-elements when we need to recompute
    /// their style with a new parent style.
    pub fn compute_pseudo_element_style_with_inputs(&self,
                                                    inputs: &CascadeInputs,
                                                    pseudo: &PseudoElement,
                                                    guards: &StylesheetGuards,
                                                    parent_style: &ComputedValues,
                                                    font_metrics: &FontMetricsProvider)
                                                    -> Option<Arc<ComputedValues>>
    {
        // We may have only visited rules in cases when we are actually
        // resolving, not probing, pseudo-element style.
        if inputs.rules.is_none() && inputs.visited_rules.is_none() {
            return None
        }

        // FIXME(emilio): The lack of layout_parent_style here could be
        // worrying, but we're probably dropping the display fixup for
        // pseudos other than before and after, so it's probably ok.
        //
        // (Though the flags don't indicate so!)
        Some(self.compute_style_with_inputs(inputs,
                                            Some(pseudo),
                                            guards,
                                            parent_style,
                                            parent_style,
                                            parent_style,
                                            font_metrics,
                                            CascadeFlags::empty()))
    }

    /// Computes a style using the given CascadeInputs.  This can be used to
    /// compute a style any time we know what rules apply and just need to use
    /// the given parent styles.
    ///
    /// parent_style is the style to inherit from for properties affected by
    /// first-line ancestors.
    ///
    /// parent_style_ignoring_first_line is the style to inherit from for
    /// properties not affected by first-line ancestors.
    ///
    /// layout_parent_style is the style used for some property fixups.  It's
    /// the style of the nearest ancestor with a layout box.
    ///
    /// is_link should be true if we're computing style for a link; that affects
    /// how :visited handling is done.
    pub fn compute_style_with_inputs(&self,
                                     inputs: &CascadeInputs,
                                     pseudo: Option<&PseudoElement>,
                                     guards: &StylesheetGuards,
                                     parent_style: &ComputedValues,
                                     parent_style_ignoring_first_line: &ComputedValues,
                                     layout_parent_style: &ComputedValues,
                                     font_metrics: &FontMetricsProvider,
                                     cascade_flags: CascadeFlags)
                                     -> Arc<ComputedValues>
    {
        // We need to compute visited values if we have visited rules or if our
        // parent has visited values.
        let visited_values = if inputs.visited_rules.is_some() || parent_style.get_visited_style().is_some() {
            // Slightly annoying: we know that inputs has either rules or
            // visited rules, but we can't do inputs.rules() up front because
            // maybe it just has visited rules, so can't unwrap_or.
            let rule_node = match inputs.visited_rules.as_ref() {
                Some(rules) => rules,
                None => inputs.rules.as_ref().unwrap(),
            };
            let inherited_style;
            let inherited_style_ignoring_first_line;
            let layout_parent_style_for_visited;
            if cascade_flags.contains(IS_LINK) {
                // We just want to use our parent style as our parent.
                inherited_style = parent_style;
                inherited_style_ignoring_first_line = parent_style_ignoring_first_line;
                layout_parent_style_for_visited = layout_parent_style;
            } else {
                // We want to use the visited bits (if any) from our parent
                // style as our parent.
                inherited_style =
                    parent_style.get_visited_style().unwrap_or(parent_style);
                inherited_style_ignoring_first_line =
                    parent_style_ignoring_first_line.get_visited_style().unwrap_or(parent_style_ignoring_first_line);
                layout_parent_style_for_visited =
                    layout_parent_style.get_visited_style().unwrap_or(layout_parent_style);
            }

            let computed =
                properties::cascade(&self.device,
                                    pseudo,
                                    rule_node,
                                    guards,
                                    Some(inherited_style),
                                    Some(inherited_style_ignoring_first_line),
                                    Some(layout_parent_style_for_visited),
                                    None,
                                    None,
                                    font_metrics,
                                    cascade_flags,
                                    self.quirks_mode);

            Some(computed)
        } else {
            None
        };

        // We may not have non-visited rules, if we only had visited ones.  In
        // that case we want to use the root rulenode for our non-visited rules.
        let rules = inputs.rules.as_ref().unwrap_or(self.rule_tree.root());

        // Read the comment on `precomputed_values_for_pseudo` to see why it's
        // difficult to assert that display: contents nodes never arrive here
        // (tl;dr: It doesn't apply for replaced elements and such, but the
        // computed value is still "contents").
        properties::cascade(&self.device,
                            pseudo,
                            rules,
                            guards,
                            Some(parent_style),
                            Some(parent_style_ignoring_first_line),
                            Some(layout_parent_style),
                            visited_values,
                            None,
                            font_metrics,
                            cascade_flags,
                            self.quirks_mode)
    }

    /// Computes the cascade inputs for a lazily-cascaded pseudo-element.
    ///
    /// See the documentation on lazy pseudo-elements in
    /// docs/components/style.md
    pub fn lazy_pseudo_rules<E>(&self,
                                guards: &StylesheetGuards,
                                element: &E,
                                pseudo: &PseudoElement,
                                is_probe: bool,
                                rule_inclusion: RuleInclusion)
                                -> CascadeInputs
        where E: TElement
    {
        let pseudo = pseudo.canonical();
        debug_assert!(pseudo.is_lazy());

        if !self.cascade_data.has_rules_for_pseudo(&pseudo) {
            return CascadeInputs::default()
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

        let mut inputs = CascadeInputs::default();
        let mut declarations = ApplicableDeclarationList::new();
        let mut matching_context =
            MatchingContext::new(MatchingMode::ForStatelessPseudoElement,
                                 None,
                                 self.quirks_mode);

        self.push_applicable_declarations(
            element,
            Some(&pseudo),
            None,
            None,
            AnimationRules(None, None),
            rule_inclusion,
            &mut declarations,
            &mut matching_context,
            &mut set_selector_flags
        );

        if !declarations.is_empty() {
            let rule_node =
                self.rule_tree.compute_rule_node(&mut declarations, guards);
            debug_assert!(rule_node != *self.rule_tree.root());
            inputs.rules = Some(rule_node);
        }

        if is_probe && inputs.rules.is_none() {
            // When probing, don't compute visited styles if we have no
            // unvisited styles.
            return inputs;
        }

        if matching_context.relevant_link_found {
            let mut declarations = ApplicableDeclarationList::new();
            let mut matching_context =
                MatchingContext::new_for_visited(MatchingMode::ForStatelessPseudoElement,
                                                 None,
                                                 VisitedHandlingMode::RelevantLinkVisited,
                                                 self.quirks_mode);
            self.push_applicable_declarations(element,
                                              Some(&pseudo),
                                              None,
                                              None,
                                              AnimationRules(None, None),
                                              rule_inclusion,
                                              &mut declarations,
                                              &mut matching_context,
                                              &mut set_selector_flags);
            if !declarations.is_empty() {
                let rule_node =
                    self.rule_tree.insert_ordered_rules_with_important(
                        declarations.drain().map(|a| a.order_and_level()),
                        guards);
                if rule_node != *self.rule_tree.root() {
                    inputs.visited_rules = Some(rule_node);
                }
            }
        }

        inputs
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
                      stylesheets: &[Arc<::stylesheets::Stylesheet>]) {
        let cascaded_rule = ViewportRule {
            declarations: viewport_rule::Cascade::from_stylesheets(
                stylesheets.iter().map(|s| &**s),
                guard,
                &device
            ).finish(),
        };

        self.viewport_constraints =
            ViewportConstraints::maybe_new(&device, &cascaded_rule, self.quirks_mode);

        if let Some(ref constraints) = self.viewport_constraints {
            device.account_for_viewport_rule(constraints);
        }

        self.device = device;
        let features_changed = self.media_features_change_changed_style(
            stylesheets.iter().map(|s| &**s),
            guard
        );
        self.is_device_dirty |= features_changed;
    }

    /// Returns whether, given a media feature change, any previously-applicable
    /// style has become non-applicable, or vice-versa.
    pub fn media_features_change_changed_style<'a, I, S>(
        &self,
        stylesheets: I,
        guard: &SharedRwLockReadGuard,
    ) -> bool
    where
        I: Iterator<Item = &'a S>,
        S: StylesheetInDocument + ToMediaListKey + 'static,
    {
        use invalidation::media_queries::PotentiallyEffectiveMediaRules;

        debug!("Stylist::media_features_change_changed_style");

        for stylesheet in stylesheets {
            let effective_now =
                stylesheet.is_effective_for_device(&self.device, guard);

            let effective_then =
                self.effective_media_query_results.was_effective(stylesheet);

            if effective_now != effective_then {
                debug!(" > Stylesheet changed -> {}, {}",
                       effective_then, effective_now);
                return true
            }

            if !effective_now {
                continue;
            }

            let mut iter =
                stylesheet.iter_rules::<PotentiallyEffectiveMediaRules>(
                    &self.device,
                    guard
                );

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
                    CssRule::Document(..) |
                    CssRule::FontFeatureValues(..) => {
                        // Not affected by device changes.
                        continue;
                    }
                    CssRule::Import(ref lock) => {
                        let import_rule = lock.read_with(guard);
                        let effective_now =
                            import_rule.stylesheet
                                .is_effective_for_device(&self.device, guard);
                        let effective_then =
                            self.effective_media_query_results.was_effective(import_rule);
                        if effective_now != effective_then {
                            debug!(" > @import rule changed {} -> {}",
                                   effective_then, effective_now);
                            return true;
                        }

                        if !effective_now {
                            iter.skip_children();
                        }
                    }
                    CssRule::Media(ref lock) => {
                        let media_rule = lock.read_with(guard);
                        let mq = media_rule.media_queries.read_with(guard);
                        let effective_now =
                            mq.evaluate(&self.device, self.quirks_mode);
                        let effective_then =
                            self.effective_media_query_results.was_effective(media_rule);
                        if effective_now != effective_then {
                            debug!(" > @media rule changed {} -> {}",
                                   effective_then, effective_now);
                            return true;
                        }

                        if !effective_now {
                            iter.skip_children();
                        }
                    }
                }
            }
        }

        return false;
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

    /// Returns the applicable CSS declarations for the given element by
    /// treating us as an XBL stylesheet-only stylist.
    pub fn push_applicable_declarations_as_xbl_only_stylist<E, V>(&self,
                                                                  element: &E,
                                                                  pseudo_element: Option<&PseudoElement>,
                                                                  applicable_declarations: &mut V)
        where E: TElement,
              V: Push<ApplicableDeclarationBlock> + VecLike<ApplicableDeclarationBlock>,
    {
        let mut matching_context =
            MatchingContext::new(MatchingMode::Normal, None, self.quirks_mode);
        let mut dummy_flag_setter = |_: &E, _: ElementSelectorFlags| {};

        let rule_hash_target = element.rule_hash_target();

        // nsXBLPrototypeResources::LoadResources() loads Chrome XBL style
        // sheets under eAuthorSheetFeatures level.
        if let Some(map) = self.cascade_data.author.borrow_for_pseudo(pseudo_element) {
            map.get_all_matching_rules(element,
                                       &rule_hash_target,
                                       applicable_declarations,
                                       &mut matching_context,
                                       self.quirks_mode,
                                       &mut dummy_flag_setter,
                                       CascadeLevel::XBL);
        }
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
                                        style_attribute: Option<ArcBorrow<Locked<PropertyDeclarationBlock>>>,
                                        smil_override: Option<ArcBorrow<Locked<PropertyDeclarationBlock>>>,
                                        animation_rules: AnimationRules,
                                        rule_inclusion: RuleInclusion,
                                        applicable_declarations: &mut V,
                                        context: &mut MatchingContext,
                                        flags_setter: &mut F)
        where E: TElement,
              V: Push<ApplicableDeclarationBlock> + VecLike<ApplicableDeclarationBlock> + Debug,
              F: FnMut(&E, ElementSelectorFlags),
    {
        debug_assert!(!self.is_device_dirty);
        // Gecko definitely has pseudo-elements with style attributes, like
        // ::-moz-color-swatch.
        debug_assert!(cfg!(feature = "gecko") ||
                      style_attribute.is_none() || pseudo_element.is_none(),
                      "Style attributes do not apply to pseudo-elements");
        debug_assert!(pseudo_element.map_or(true, |p| !p.is_precomputed()));

        let rule_hash_target = element.rule_hash_target();

        debug!("Determining if style is shareable: pseudo: {}",
               pseudo_element.is_some());

        let only_default_rules = rule_inclusion == RuleInclusion::DefaultOnly;

        // Step 1: Normal user-agent rules.
        if let Some(map) = self.cascade_data.user_agent.borrow_for_pseudo(pseudo_element) {
            map.get_all_matching_rules(element,
                                       &rule_hash_target,
                                       applicable_declarations,
                                       context,
                                       self.quirks_mode,
                                       flags_setter,
                                       CascadeLevel::UANormal);
        }

        if pseudo_element.is_none() && !only_default_rules {
            // Step 2: Presentational hints.
            let length_before_preshints = applicable_declarations.len();
            element.synthesize_presentational_hints_for_legacy_attributes(
                context.visited_handling,
                applicable_declarations
            );
            if applicable_declarations.len() != length_before_preshints {
                if cfg!(debug_assertions) {
                    for declaration in &applicable_declarations[length_before_preshints..] {
                        assert_eq!(declaration.level(), CascadeLevel::PresHints);
                    }
                }
            }
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
            if let Some(map) = self.cascade_data.user.borrow_for_pseudo(pseudo_element) {
                map.get_all_matching_rules(element,
                                           &rule_hash_target,
                                           applicable_declarations,
                                           context,
                                           self.quirks_mode,
                                           flags_setter,
                                           CascadeLevel::UserNormal);
            }
        } else {
            debug!("skipping user rules");
        }

        // Step 3b: XBL rules.
        let cut_off_inheritance =
            element.get_declarations_from_xbl_bindings(pseudo_element,
                                                       applicable_declarations);

        if rule_hash_target.matches_user_and_author_rules() && !only_default_rules {
            // Gecko skips author normal rules if cutting off inheritance.
            // See nsStyleSet::FileRules().
            if !cut_off_inheritance {
                // Step 3c: Author normal rules.
                if let Some(map) = self.cascade_data.author.borrow_for_pseudo(pseudo_element) {
                    map.get_all_matching_rules(element,
                                               &rule_hash_target,
                                               applicable_declarations,
                                               context,
                                               self.quirks_mode,
                                               flags_setter,
                                               CascadeLevel::AuthorNormal);
                }
            } else {
                debug!("skipping author normal rules due to cut off inheritance");
            }
        } else {
            debug!("skipping author normal rules");
        }

        if !only_default_rules {
            // Step 4: Normal style attributes.
            if let Some(sa) = style_attribute {
                Push::push(
                    applicable_declarations,
                    ApplicableDeclarationBlock::from_declarations(sa.clone_arc(),
                                                                  CascadeLevel::StyleAttributeNormal));
            }

            // Step 5: SMIL override.
            // Declarations from SVG SMIL animation elements.
            if let Some(so) = smil_override {
                Push::push(
                    applicable_declarations,
                    ApplicableDeclarationBlock::from_declarations(so.clone_arc(),
                                                                  CascadeLevel::SMILOverride));
            }

            // Step 6: Animations.
            // The animations sheet (CSS animations, script-generated animations,
            // and CSS transitions that are no longer tied to CSS markup)
            if let Some(anim) = animation_rules.0 {
                Push::push(
                    applicable_declarations,
                    ApplicableDeclarationBlock::from_declarations(anim.clone(),
                                                                  CascadeLevel::Animations));
            }
        } else {
            debug!("skipping style attr and SMIL & animation rules");
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
        } else {
            debug!("skipping transition rules");
        }
    }

    /// Given an id, returns whether there might be any rules for that id in any
    /// of our rule maps.
    #[inline]
    pub fn may_have_rules_for_id(&self, id: &Atom) -> bool {
        self.mapped_ids.might_contain_hash(id.get_hash())
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

    /// Computes the match results of a given element against the set of
    /// revalidation selectors.
    pub fn match_revalidation_selectors<E, F>(&self,
                                              element: &E,
                                              bloom: Option<&BloomFilter>,
                                              flags_setter: &mut F)
                                              -> BitVec
        where E: TElement,
              F: FnMut(&E, ElementSelectorFlags),
    {
        // NB: `MatchingMode` doesn't really matter, given we don't share style
        // between pseudos.
        let mut matching_context =
            MatchingContext::new(MatchingMode::Normal, bloom, self.quirks_mode);

        // Note that, by the time we're revalidating, we're guaranteed that the
        // candidate and the entry have the same id, classes, and local name.
        // This means we're guaranteed to get the same rulehash buckets for all
        // the lookups, which means that the bitvecs are comparable. We verify
        // this in the caller by asserting that the bitvecs are same-length.
        let mut results = BitVec::new();
        self.selectors_for_cache_revalidation.lookup(
            *element, self.quirks_mode, &mut |selector_and_hashes| {
                results.push(matches_selector(&selector_and_hashes.selector,
                                              selector_and_hashes.selector_offset,
                                              Some(&selector_and_hashes.hashes),
                                              element,
                                              &mut matching_context,
                                              flags_setter));
                true
            }
        );

        results
    }

    /// Computes styles for a given declaration with parent_style.
    pub fn compute_for_declarations(&self,
                                    guards: &StylesheetGuards,
                                    parent_style: &ComputedValues,
                                    declarations: Arc<Locked<PropertyDeclarationBlock>>)
                                    -> Arc<ComputedValues> {
        use font_metrics::get_metrics_provider_for_product;

        let v = vec![
            ApplicableDeclarationBlock::from_declarations(declarations.clone(),
                                                          CascadeLevel::StyleAttributeNormal)
        ];
        let rule_node =
            self.rule_tree.insert_ordered_rules(v.into_iter().map(|a| a.order_and_level()));

        // This currently ignores visited styles.  It appears to be used for
        // font styles in <canvas> via Servo_StyleSet_ResolveForDeclarations.
        // It is unclear if visited styles are meaningful for this case.
        let metrics = get_metrics_provider_for_product();
        // FIXME(emilio): the pseudo bit looks quite dubious!
        properties::cascade(&self.device,
                            /* pseudo = */ None,
                            &rule_node,
                            guards,
                            Some(parent_style),
                            Some(parent_style),
                            Some(parent_style),
                            None,
                            None,
                            &metrics,
                            CascadeFlags::empty(),
                            self.quirks_mode)
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

/// SelectorMapEntry implementation for use in our revalidation selector map.
#[derive(Clone, Debug)]
struct RevalidationSelectorAndHashes {
    selector: Selector<SelectorImpl>,
    selector_offset: usize,
    hashes: AncestorHashes,
}

impl RevalidationSelectorAndHashes {
    fn new(selector: Selector<SelectorImpl>, hashes: AncestorHashes) -> Self {
        let selector_offset = {
            // We basically want to check whether the first combinator is a
            // pseudo-element combinator.  If it is, we want to use the offset
            // one past it.  Otherwise, our offset is 0.
            let mut index = 0;
            let mut iter = selector.iter();

            // First skip over the first ComplexSelector.
            //
            // We can't check what sort of what combinator we have until we do
            // that.
            for _ in &mut iter {
                index += 1; // Simple selector
            }

            match iter.next_sequence() {
                Some(Combinator::PseudoElement) => index + 1, // +1 for the combinator
                _ => 0
            }
        };

        RevalidationSelectorAndHashes { selector, selector_offset, hashes, }
    }
}

impl SelectorMapEntry for RevalidationSelectorAndHashes {
    fn selector(&self) -> SelectorIter<SelectorImpl> {
        self.selector.iter_from(self.selector_offset)
    }
}

/// A selector visitor implementation that collects all the state the Stylist
/// cares about a selector.
struct StylistSelectorVisitor<'a> {
    /// Whether the selector needs revalidation for the style sharing cache.
    needs_revalidation: bool,
    /// Whether we've past the rightmost compound selector, not counting
    /// pseudo-elements.
    passed_rightmost_selector: bool,
    /// The filter with all the id's getting referenced from rightmost
    /// selectors.
    mapped_ids: &'a mut BloomFilter,
    /// The filter with the local names of attributes there are selectors for.
    attribute_dependencies: &'a mut BloomFilter,
    /// Whether there's any attribute selector for the [style] attribute.
    style_attribute_dependency: &'a mut bool,
    /// All the states selectors in the page reference.
    state_dependencies: &'a mut ElementState,
}

fn component_needs_revalidation(
    c: &Component<SelectorImpl>,
    passed_rightmost_selector: bool,
) -> bool {
    match *c {
        Component::ID(_) => {
            // TODO(emilio): This could also check that the ID is not already in
            // the rule hash. In that case, we could avoid making this a
            // revalidation selector too.
            //
            // See https://bugzilla.mozilla.org/show_bug.cgi?id=1369611
            passed_rightmost_selector
        }
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
            true
        },
        Component::NonTSPseudoClass(ref p) => {
            p.needs_cache_revalidation()
        },
        _ => {
            false
        }
    }
}

impl<'a> SelectorVisitor for StylistSelectorVisitor<'a> {
    type Impl = SelectorImpl;

    fn visit_complex_selector(
        &mut self,
        _: SelectorIter<SelectorImpl>,
        combinator: Option<Combinator>
    ) -> bool {
        self.needs_revalidation =
            self.needs_revalidation || combinator.map_or(false, |c| c.is_sibling());

        // NOTE(emilio): This works properly right now because we can't store
        // complex selectors in nested selectors, otherwise we may need to
        // rethink this.
        //
        // Also, note that this call happens before we visit any of the simple
        // selectors in the next ComplexSelector, so we can use this to skip
        // looking at them.
        self.passed_rightmost_selector =
            self.passed_rightmost_selector ||
            !matches!(combinator, None | Some(Combinator::PseudoElement));

        true
    }

    fn visit_attribute_selector(
        &mut self,
        _ns: &NamespaceConstraint<&Namespace>,
        name: &LocalName,
        lower_name: &LocalName
    ) -> bool {
        if *lower_name == local_name!("style") {
            *self.style_attribute_dependency = true;
        } else {
            self.attribute_dependencies.insert_hash(name.get_hash());
            self.attribute_dependencies.insert_hash(lower_name.get_hash());
        }
        true
    }

    fn visit_simple_selector(&mut self, s: &Component<SelectorImpl>) -> bool {
        self.needs_revalidation =
            self.needs_revalidation ||
            component_needs_revalidation(s, self.passed_rightmost_selector);

        match *s {
            Component::NonTSPseudoClass(ref p) => {
                self.state_dependencies.insert(p.state_flag());
            }
            Component::ID(ref id) if !self.passed_rightmost_selector => {
                // We want to stop storing mapped ids as soon as we've moved off
                // the rightmost ComplexSelector that is not a pseudo-element.
                //
                // That can be detected by a visit_complex_selector call with a
                // combinator other than None and PseudoElement.
                //
                // Importantly, this call happens before we visit any of the
                // simple selectors in that ComplexSelector.
                //
                // NOTE(emilio): See the comment regarding on when this may
                // break in visit_complex_selector.
                self.mapped_ids.insert_hash(id.get_hash());
            }
            _ => {},
        }

        true
    }
}

/// Data resulting from performing the CSS cascade.
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[derive(Debug)]
struct CascadeData {
    /// Rules from user agent stylesheets
    user_agent: PerOriginCascadeData,
    /// Rules from author stylesheets
    author: PerOriginCascadeData,
    /// Rules from user stylesheets
    user: PerOriginCascadeData,
}

impl CascadeData {
    fn new() -> Self {
        CascadeData {
            user_agent: PerOriginCascadeData::new(),
            author: PerOriginCascadeData::new(),
            user: PerOriginCascadeData::new(),
        }
    }

    #[inline]
    fn borrow_mut_for_origin(&mut self, origin: &Origin) -> &mut PerOriginCascadeData {
        match *origin {
            Origin::UserAgent => &mut self.user_agent,
            Origin::Author => &mut self.author,
            Origin::User => &mut self.user,
        }
    }

    fn clear(&mut self) {
        self.user_agent.clear();
        self.author.clear();
        self.user.clear();
    }

    fn has_rules_for_pseudo(&self, pseudo: &PseudoElement) -> bool {
        self.iter_origins().any(|d| d.has_rules_for_pseudo(pseudo))
    }

    fn iter_origins(&self) -> CascadeDataIter {
        CascadeDataIter {
            cascade_data: &self,
            cur: 0,
        }
    }
}

struct CascadeDataIter<'a> {
    cascade_data: &'a CascadeData,
    cur: usize,
}

impl<'a> Iterator for CascadeDataIter<'a> {
    type Item = &'a PerOriginCascadeData;

    fn next(&mut self) -> Option<&'a PerOriginCascadeData> {
        let result = match self.cur {
            0 => &self.cascade_data.user_agent,
            1 => &self.cascade_data.author,
            2 => &self.cascade_data.user,
            _ => return None,
        };
        self.cur += 1;
        Some(result)
    }
}

/// Data resulting from performing the CSS cascade that is specific to a given
/// origin.
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[derive(Debug)]
struct PerOriginCascadeData {
    /// Rules from stylesheets at this `CascadeData`'s origin.
    element_map: SelectorMap<Rule>,

    /// Rules from stylesheets at this `CascadeData`'s origin that correspond
    /// to a given pseudo-element.
    pseudos_map: FnvHashMap<PseudoElement, SelectorMap<Rule>>,
}

impl PerOriginCascadeData {
    fn new() -> Self {
        let mut data = PerOriginCascadeData {
            element_map: SelectorMap::new(),
            pseudos_map: Default::default(),
        };
        SelectorImpl::each_eagerly_cascaded_pseudo_element(|pseudo| {
            data.pseudos_map.insert(pseudo, SelectorMap::new());
        });
        data
    }

    #[inline]
    fn borrow_for_pseudo(&self, pseudo: Option<&PseudoElement>) -> Option<&SelectorMap<Rule>> {
        match pseudo {
            Some(pseudo) => self.pseudos_map.get(&pseudo.canonical()),
            None => Some(&self.element_map),
        }
    }

    #[inline]
    fn borrow_mut_for_pseudo_or_insert(&mut self, pseudo: Option<&PseudoElement>) -> &mut SelectorMap<Rule> {
        match pseudo {
            Some(pseudo) => {
                self.pseudos_map
                    .entry(pseudo.canonical())
                    .or_insert_with(SelectorMap::new)
            }
            None => &mut self.element_map,
        }
    }

    fn clear(&mut self) {
        self.element_map = SelectorMap::new();
        self.pseudos_map = Default::default();
        SelectorImpl::each_eagerly_cascaded_pseudo_element(|pseudo| {
            self.pseudos_map.insert(pseudo, SelectorMap::new());
        });
    }

    fn has_rules_for_pseudo(&self, pseudo: &PseudoElement) -> bool {
        // FIXME(emilio): We should probably make the pseudos map be an
        // enumerated array.
        self.pseudos_map.contains_key(pseudo)
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
    /// The ancestor hashes associated with the selector.
    #[cfg_attr(feature = "servo", ignore_heap_size_of = "No heap data")]
    pub hashes: AncestorHashes,
    /// The source order this style rule appears in. Note that we only use
    /// three bytes to store this value in ApplicableDeclarationsBlock, so
    /// we could repurpose that storage here if we needed to.
    pub source_order: u32,
    /// The actual style rule.
    #[cfg_attr(feature = "servo", ignore_heap_size_of = "Arc")]
    pub style_rule: Arc<Locked<StyleRule>>,
}

impl SelectorMapEntry for Rule {
    fn selector(&self) -> SelectorIter<SelectorImpl> {
        self.selector.iter()
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
        let source = StyleSource::Style(self.style_rule.clone());
        ApplicableDeclarationBlock::new(source,
                                        self.source_order,
                                        level,
                                        self.specificity())
    }

    /// Creates a new Rule.
    pub fn new(selector: Selector<SelectorImpl>,
               hashes: AncestorHashes,
               style_rule: Arc<Locked<StyleRule>>,
               source_order: u32)
               -> Self
    {
        Rule {
            selector: selector,
            hashes: hashes,
            style_rule: style_rule,
            source_order: source_order,
        }
    }
}

/// A function to be able to test the revalidation stuff.
pub fn needs_revalidation_for_testing(s: &Selector<SelectorImpl>) -> bool {
    let mut attribute_dependencies = BloomFilter::new();
    let mut mapped_ids = BloomFilter::new();
    let mut style_attribute_dependency = false;
    let mut state_dependencies = ElementState::empty();
    let mut visitor = StylistSelectorVisitor {
        needs_revalidation: false,
        passed_rightmost_selector: false,
        attribute_dependencies: &mut attribute_dependencies,
        style_attribute_dependency: &mut style_attribute_dependency,
        state_dependencies: &mut state_dependencies,
        mapped_ids: &mut mapped_ids,
    };
    s.visit(&mut visitor);
    visitor.needs_revalidation
}
