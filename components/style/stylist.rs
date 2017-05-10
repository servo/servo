/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Selector matching.

#![deny(missing_docs)]

use {Atom, LocalName};
use bit_vec::BitVec;
use context::QuirksMode;
use data::ComputedStyle;
use dom::{AnimationRules, PresentationalHintsSynthetizer, TElement};
use error_reporting::RustLogReporter;
use font_metrics::FontMetricsProvider;
use keyframes::KeyframesAnimation;
use media_queries::Device;
use pdqsort::sort_by;
use properties::{self, CascadeFlags, ComputedValues};
#[cfg(feature = "servo")]
use properties::INHERIT_ALL;
use properties::PropertyDeclarationBlock;
use restyle_hints::{RestyleHint, DependencySet};
use rule_tree::{CascadeLevel, RuleTree, StrongRuleNode, StyleSource};
use selector_parser::{SelectorImpl, PseudoElement, SnapshotMap};
use selectors::Element;
use selectors::bloom::BloomFilter;
use selectors::matching::{AFFECTED_BY_STYLE_ATTRIBUTE, AFFECTED_BY_PRESENTATIONAL_HINTS};
use selectors::matching::{ElementSelectorFlags, StyleRelations, matches_selector};
use selectors::parser::{Combinator, Component, Selector, SelectorInner, SelectorIter};
use selectors::parser::{SelectorMethods, LocalName as LocalNameSelector};
use selectors::visitor::SelectorVisitor;
use shared_lock::{Locked, SharedRwLockReadGuard, StylesheetGuards};
use sink::Push;
use smallvec::VecLike;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::fmt;
use std::hash::Hash;
#[cfg(feature = "servo")]
use std::marker::PhantomData;
use style_traits::viewport::ViewportConstraints;
use stylearc::Arc;
use stylesheets::{CssRule, FontFaceRule, Origin, StyleRule, Stylesheet, UserAgentStylesheets};
#[cfg(feature = "servo")]
use stylesheets::NestedRulesResult;
use thread_state;
use viewport::{self, MaybeNew, ViewportRule};

pub use ::fnv::FnvHashMap;

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
    ///
    /// In both cases, the device is actually _owned_ by the Stylist, and it's
    /// only an `Arc` so we can implement `add_stylesheet` more idiomatically.
    pub device: Arc<Device>,

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
    ///
    /// FIXME(emilio): Not `pub`!
    pub rule_tree: RuleTree,

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
pub struct ExtraStyleData<'a> {
    /// A list of effective font-face rules and their origin.
    #[cfg(feature = "gecko")]
    pub font_faces: &'a mut Vec<(Arc<Locked<FontFaceRule>>, Origin)>,

    #[allow(missing_docs)]
    #[cfg(feature = "servo")]
    pub marker: PhantomData<&'a usize>,
}

#[cfg(feature = "gecko")]
impl<'a> ExtraStyleData<'a> {
    /// Clear the internal @font-face rule list.
    fn clear_font_faces(&mut self) {
        self.font_faces.clear();
    }

    /// Add the given @font-face rule.
    fn add_font_face(&mut self, rule: &Arc<Locked<FontFaceRule>>, origin: Origin) {
        self.font_faces.push((rule.clone(), origin));
    }
}

#[cfg(feature = "servo")]
impl<'a> ExtraStyleData<'a> {
    fn clear_font_faces(&mut self) {}
    fn add_font_face(&mut self, _: &Arc<Locked<FontFaceRule>>, _: Origin) {}
}

impl Stylist {
    /// Construct a new `Stylist`, using a given `Device`.  If more members are
    /// added here, think about whether they should be reset in clear().
    #[inline]
    pub fn new(device: Device) -> Self {
        let mut stylist = Stylist {
            viewport_constraints: None,
            device: Arc::new(device),
            is_device_dirty: true,
            is_cleared: true,
            quirks_mode: QuirksMode::NoQuirks,

            element_map: PerPseudoElementSelectorMap::new(),
            pseudos_map: Default::default(),
            animations: Default::default(),
            precomputed_pseudo_element_decls: Default::default(),
            rules_source_order: 0,
            rule_tree: RuleTree::new(),
            dependencies: DependencySet::new(),
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
    pub fn rebuild<'a>(&mut self,
                       doc_stylesheets: &[Arc<Stylesheet>],
                       guards: &StylesheetGuards,
                       ua_stylesheets: Option<&UserAgentStylesheets>,
                       stylesheets_changed: bool,
                       author_style_disabled: bool,
                       extra_data: &mut ExtraStyleData<'a>) -> bool {
        debug_assert!(!self.is_cleared || self.is_device_dirty);

        self.is_cleared = false;

        if !(self.is_device_dirty || stylesheets_changed) {
            return false;
        }

        self.num_rebuilds += 1;

        let cascaded_rule = ViewportRule {
            declarations: viewport::Cascade::from_stylesheets(
                doc_stylesheets, guards.author, &self.device
            ).finish(),
        };

        self.viewport_constraints =
            ViewportConstraints::maybe_new(&self.device, &cascaded_rule, self.quirks_mode);

        if let Some(ref constraints) = self.viewport_constraints {
            Arc::get_mut(&mut self.device).unwrap()
                .account_for_viewport_rule(constraints);
        }

        SelectorImpl::each_eagerly_cascaded_pseudo_element(|pseudo| {
            self.pseudos_map.insert(pseudo, PerPseudoElementSelectorMap::new());
        });

        extra_data.clear_font_faces();

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
        let sheets_to_add = doc_stylesheets.iter().filter(|s| {
            !author_style_disabled || s.origin != Origin::Author
        });

        for ref stylesheet in sheets_to_add {
            self.add_stylesheet(stylesheet, guards.author, extra_data);
        }

        SelectorImpl::each_precomputed_pseudo_element(|pseudo| {
            if let Some(map) = self.pseudos_map.remove(&pseudo) {
                let declarations =
                    map.user_agent.get_universal_rules(
                        guards.ua_or_user, CascadeLevel::UANormal, CascadeLevel::UAImportant
                    );
                self.precomputed_pseudo_element_decls.insert(pseudo, declarations);
            }
        });

        self.is_device_dirty = false;
        true
    }

    /// clear the stylist and then rebuild it.  Chances are, you want to use
    /// either clear() or rebuild(), with the latter done lazily, instead.
    pub fn update<'a>(&mut self,
                      doc_stylesheets: &[Arc<Stylesheet>],
                      guards: &StylesheetGuards,
                      ua_stylesheets: Option<&UserAgentStylesheets>,
                      stylesheets_changed: bool,
                      author_style_disabled: bool,
                      extra_data: &mut ExtraStyleData<'a>) -> bool {
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

    fn add_stylesheet<'a>(&mut self, stylesheet: &Stylesheet, guard: &SharedRwLockReadGuard,
                          extra_data: &mut ExtraStyleData<'a>) {
        if stylesheet.disabled() || !stylesheet.is_effective_for_device(&self.device, guard) {
            return;
        }

        // Cheap `Arc` clone so that the closure below can borrow `&mut Stylist`.
        let device = self.device.clone();

        stylesheet.effective_rules(&device, guard, |rule| {
            match *rule {
                CssRule::Style(ref locked) => {
                    let style_rule = locked.read_with(&guard);
                    self.num_declarations += style_rule.block.read_with(&guard).len();
                    for selector in &style_rule.selectors.0 {
                        self.num_selectors += 1;
                        self.add_rule_to_map(guard, selector, locked, stylesheet);
                        self.dependencies.note_selector(selector);
                        self.note_for_revalidation(selector);
                    }
                    self.rules_source_order += 1;
                }
                CssRule::Import(ref import) => {
                    let import = import.read_with(guard);
                    self.add_stylesheet(&import.stylesheet, guard, extra_data)
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
                CssRule::FontFace(ref rule) => {
                    extra_data.add_font_face(&rule, stylesheet.origin);
                }
                // We don't care about any other rule.
                _ => {}
            }
        });
    }

    #[inline]
    fn add_rule_to_map(&mut self,
                       guard: &SharedRwLockReadGuard,
                       selector: &Selector<SelectorImpl>,
                       rule: &Arc<Locked<StyleRule>>,
                       stylesheet: &Stylesheet)
    {
        let map = if let Some(ref pseudo) = selector.pseudo_element {
            self.pseudos_map
                .entry(pseudo.clone())
                .or_insert_with(PerPseudoElementSelectorMap::new)
                .borrow_for_origin(&stylesheet.origin)
        } else {
            self.element_map.borrow_for_origin(&stylesheet.origin)
        };

        map.insert(Rule::new(guard,
                             selector.inner.clone(),
                             rule.clone(),
                             self.rules_source_order,
                             selector.specificity));
    }

    #[inline]
    fn note_for_revalidation(&mut self, selector: &Selector<SelectorImpl>) {
        if needs_revalidation(selector) {
            self.selectors_for_cache_revalidation.insert(selector.inner.clone());
        }
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
                self.rule_tree.insert_ordered_rules(
                    declarations.into_iter().map(|a| (a.source.clone(), a.level)))
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
                                                  parent: &Arc<ComputedValues>,
                                                  font_metrics: &FontMetricsProvider)
                                                  -> Option<ComputedStyle>
        where E: TElement +
                 fmt::Debug +
                 PresentationalHintsSynthetizer
    {
        let rule_node = match self.lazy_pseudo_rules(guards, element, pseudo) {
            Some(rule_node) => rule_node,
            None => return None
        };

        // Read the comment on `precomputed_values_for_pseudo` to see why it's
        // difficult to assert that display: contents nodes never arrive here
        // (tl;dr: It doesn't apply for replaced elements and such, but the
        // computed value is still "contents").
        let computed =
            properties::cascade(&self.device,
                                &rule_node,
                                guards,
                                Some(&**parent),
                                Some(&**parent),
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
                                pseudo: &PseudoElement)
                                -> Option<StrongRuleNode>
        where E: TElement + fmt::Debug + PresentationalHintsSynthetizer
    {
        debug_assert!(pseudo.is_lazy());
        if self.pseudos_map.get(pseudo).is_none() {
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


        let mut declarations = vec![];
        self.push_applicable_declarations(element,
                                          None,
                                          None,
                                          None,
                                          AnimationRules(None, None),
                                          Some(pseudo),
                                          guards,
                                          &mut declarations,
                                          &mut set_selector_flags);
        if declarations.is_empty() {
            return None
        }

        let rule_node = self.rule_tree.insert_ordered_rules(declarations.into_iter().map(|a| {
            (a.source, a.level)
        }));
        Some(rule_node)
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
    pub fn set_device(&mut self, mut device: Device, guard: &SharedRwLockReadGuard,
                      stylesheets: &[Arc<Stylesheet>]) {
        let cascaded_rule = ViewportRule {
            declarations: viewport::Cascade::from_stylesheets(stylesheets, guard, &device).finish(),
        };

        self.viewport_constraints =
            ViewportConstraints::maybe_new(&device, &cascaded_rule, self.quirks_mode);

        if let Some(ref constraints) = self.viewport_constraints {
            device.account_for_viewport_rule(constraints);
        }

        fn mq_eval_changed(guard: &SharedRwLockReadGuard, rules: &[CssRule],
                           before: &Device, after: &Device, quirks_mode: QuirksMode) -> bool {
            for rule in rules {
                let changed = rule.with_nested_rules_mq_and_doc_rule(guard,
                                                                     |result| {
                    let rules = match result {
                        NestedRulesResult::Rules(rules) => rules,
                        NestedRulesResult::RulesWithMediaQueries(rules, mq) => {
                            if mq.evaluate(before, quirks_mode) != mq.evaluate(after, quirks_mode) {
                                return true;
                            }
                            rules
                        },
                        NestedRulesResult::RulesWithDocument(rules, doc_rule) => {
                            if !doc_rule.condition.evaluate(before) {
                                return false;
                            }
                            rules
                        },
                    };
                    mq_eval_changed(guard, rules, before, after, quirks_mode)
                });
                if changed {
                    return true
                }
            }
            false
        }
        self.is_device_dirty |= stylesheets.iter().any(|stylesheet| {
            let mq = stylesheet.media.read_with(guard);
            if mq.evaluate(&self.device, self.quirks_mode) != mq.evaluate(&device, self.quirks_mode) {
                return true
            }

            mq_eval_changed(guard, &stylesheet.rules.read_with(guard).0, &self.device, &device, self.quirks_mode)
        });

        self.device = Arc::new(device);
    }

    /// Returns the viewport constraints that apply to this document because of
    /// a @viewport rule.
    pub fn viewport_constraints(&self) -> Option<&ViewportConstraints> {
        self.viewport_constraints.as_ref()
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
    /// The returned `StyleRelations` indicate hints about which kind of rules
    /// have matched.
    pub fn push_applicable_declarations<E, V, F>(
                                        &self,
                                        element: &E,
                                        parent_bf: Option<&BloomFilter>,
                                        style_attribute: Option<&Arc<Locked<PropertyDeclarationBlock>>>,
                                        smil_override: Option<&Arc<Locked<PropertyDeclarationBlock>>>,
                                        animation_rules: AnimationRules,
                                        pseudo_element: Option<&PseudoElement>,
                                        guards: &StylesheetGuards,
                                        applicable_declarations: &mut V,
                                        flags_setter: &mut F)
                                        -> StyleRelations
        where E: TElement +
                 fmt::Debug +
                 PresentationalHintsSynthetizer,
              V: Push<ApplicableDeclarationBlock> + VecLike<ApplicableDeclarationBlock>,
              F: FnMut(&E, ElementSelectorFlags),
    {
        debug_assert!(!self.is_device_dirty);
        // Gecko definitely has pseudo-elements with style attributes, like
        // ::-moz-color-swatch.
        debug_assert!(cfg!(feature = "gecko") ||
                      style_attribute.is_none() || pseudo_element.is_none(),
                      "Style attributes do not apply to pseudo-elements");
        debug_assert!(pseudo_element.as_ref().map_or(true, |p| !p.is_precomputed()));

        let map = match pseudo_element {
            Some(ref pseudo) => self.pseudos_map.get(pseudo).unwrap(),
            None => &self.element_map,
        };

        let mut relations = StyleRelations::empty();

        debug!("Determining if style is shareable: pseudo: {}", pseudo_element.is_some());
        // Step 1: Normal user-agent rules.
        map.user_agent.get_all_matching_rules(element,
                                              parent_bf,
                                              applicable_declarations,
                                              &mut relations,
                                              flags_setter,
                                              CascadeLevel::UANormal);
        debug!("UA normal: {:?}", relations);

        if pseudo_element.is_none() {
            // Step 2: Presentational hints.
            let length_before_preshints = applicable_declarations.len();
            element.synthesize_presentational_hints_for_legacy_attributes(applicable_declarations);
            if applicable_declarations.len() != length_before_preshints {
                if cfg!(debug_assertions) {
                    for declaration in &applicable_declarations[length_before_preshints..] {
                        assert_eq!(declaration.level, CascadeLevel::PresHints);
                    }
                }
                // Never share style for elements with preshints
                relations |= AFFECTED_BY_PRESENTATIONAL_HINTS;
            }
            debug!("preshints: {:?}", relations);
        }

        if element.matches_user_and_author_rules() {
            // Step 3: User and author normal rules.
            map.user.get_all_matching_rules(element,
                                            parent_bf,
                                            applicable_declarations,
                                            &mut relations,
                                            flags_setter,
                                            CascadeLevel::UserNormal);
            debug!("user normal: {:?}", relations);
            map.author.get_all_matching_rules(element,
                                              parent_bf,
                                              applicable_declarations,
                                              &mut relations,
                                              flags_setter,
                                              CascadeLevel::AuthorNormal);
            debug!("author normal: {:?}", relations);

            // Step 4: Normal style attributes.
            if let Some(sa) = style_attribute {
                if sa.read_with(guards.author).any_normal() {
                    relations |= AFFECTED_BY_STYLE_ATTRIBUTE;
                    Push::push(
                        applicable_declarations,
                        ApplicableDeclarationBlock::from_declarations(sa.clone(),
                                                                      CascadeLevel::StyleAttributeNormal));
                }
            }

            debug!("style attr: {:?}", relations);

            // Step 5: SMIL override.
            // Declarations from SVG SMIL animation elements.
            if let Some(so) = smil_override {
                Push::push(
                    applicable_declarations,
                    ApplicableDeclarationBlock::from_declarations(so.clone(),
                                                                  CascadeLevel::SMILOverride));
            }
            debug!("SMIL: {:?}", relations);

            // Step 6: Animations.
            // The animations sheet (CSS animations, script-generated animations,
            // and CSS transitions that are no longer tied to CSS markup)
            if let Some(anim) = animation_rules.0 {
                Push::push(
                    applicable_declarations,
                    ApplicableDeclarationBlock::from_declarations(anim,
                                                                  CascadeLevel::Animations));
            }
            debug!("animation: {:?}", relations);

            // Step 7: Author-supplied `!important` rules.
            map.author.get_all_matching_rules(element,
                                              parent_bf,
                                              applicable_declarations,
                                              &mut relations,
                                              flags_setter,
                                              CascadeLevel::AuthorImportant);

            debug!("author important: {:?}", relations);

            // Step 8: `!important` style attributes.
            if let Some(sa) = style_attribute {
                if sa.read_with(guards.author).any_important() {
                    relations |= AFFECTED_BY_STYLE_ATTRIBUTE;
                    Push::push(
                        applicable_declarations,
                        ApplicableDeclarationBlock::from_declarations(sa.clone(),
                                                                      CascadeLevel::StyleAttributeImportant));
                }
            }

            debug!("style attr important: {:?}", relations);

            // Step 9: User `!important` rules.
            map.user.get_all_matching_rules(element,
                                            parent_bf,
                                            applicable_declarations,
                                            &mut relations,
                                            flags_setter,
                                            CascadeLevel::UserImportant);

            debug!("user important: {:?}", relations);
        } else {
            debug!("skipping non-agent rules");
        }

        // Step 10: UA `!important` rules.
        map.user_agent.get_all_matching_rules(element,
                                              parent_bf,
                                              applicable_declarations,
                                              &mut relations,
                                              flags_setter,
                                              CascadeLevel::UAImportant);

        debug!("UA important: {:?}", relations);

        // Step 11: Transitions.
        // The transitions sheet (CSS transitions that are tied to CSS markup)
        if let Some(anim) = animation_rules.1 {
            Push::push(
                applicable_declarations,
                ApplicableDeclarationBlock::from_declarations(anim, CascadeLevel::Transitions));
        }
        debug!("transition: {:?}", relations);

        debug!("push_applicable_declarations: shareable: {:?}", relations);

        relations
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
        use selectors::matching::StyleRelations;
        use selectors::matching::matches_selector;

        // Note that, by the time we're revalidating, we're guaranteed that the
        // candidate and the entry have the same id, classes, and local name.
        // This means we're guaranteed to get the same rulehash buckets for all
        // the lookups, which means that the bitvecs are comparable. We verify
        // this in the caller by asserting that the bitvecs are same-length.
        let mut results = BitVec::new();
        self.selectors_for_cache_revalidation.lookup(*element, &mut |selector| {
            results.push(matches_selector(selector,
                                          element,
                                          Some(bloom),
                                          &mut StyleRelations::empty(),
                                          flags_setter));
            true
        });

        results
    }

    /// Given an element, and a snapshot table that represents a previous state
    /// of the tree, compute the appropriate restyle hint, that is, the kind of
    /// restyle we need to do.
    pub fn compute_restyle_hint<E>(&self,
                                   element: &E,
                                   snapshots: &SnapshotMap)
                                   -> RestyleHint
        where E: TElement,
    {
        self.dependencies.compute_hint(element, snapshots)
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

        let metrics = get_metrics_provider_for_product();
        Arc::new(properties::cascade(&self.device,
                                     &rule_node,
                                     guards,
                                     Some(parent_style),
                                     Some(parent_style),
                                     None,
                                     &RustLogReporter,
                                     &metrics,
                                     CascadeFlags::empty(),
                                     self.quirks_mode))
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
            Component::AttrExists(_) |
            Component::AttrEqual(_, _, _) |
            Component::AttrIncludes(_, _) |
            Component::AttrDashMatch(_, _) |
            Component::AttrPrefixMatch(_, _) |
            Component::AttrSubstringMatch(_, _) |
            Component::AttrSuffixMatch(_, _) |
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

/// Map element data to selector-providing objects for which the last simple
/// selector starts with them.
///
/// e.g.,
/// "p > img" would go into the set of selectors corresponding to the
/// element "img"
/// "a .foo .bar.baz" would go into the set of selectors corresponding to
/// the class "bar"
///
/// Because we match selectors right-to-left (i.e., moving up the tree
/// from an element), we need to compare the last simple selector in the
/// selector with the element.
///
/// So, if an element has ID "id1" and classes "foo" and "bar", then all
/// the rules it matches will have their last simple selector starting
/// either with "#id1" or with ".foo" or with ".bar".
///
/// Hence, the union of the rules keyed on each of element's classes, ID,
/// element name, etc. will contain the Selectors that actually match that
/// element.
///
/// TODO: Tune the initial capacity of the HashMap
#[derive(Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct SelectorMap<T: Clone + Borrow<SelectorInner<SelectorImpl>>> {
    /// A hash from an ID to rules which contain that ID selector.
    pub id_hash: FnvHashMap<Atom, Vec<T>>,
    /// A hash from a class name to rules which contain that class selector.
    pub class_hash: FnvHashMap<Atom, Vec<T>>,
    /// A hash from local name to rules which contain that local name selector.
    pub local_name_hash: FnvHashMap<LocalName, Vec<T>>,
    /// Rules that don't have ID, class, or element selectors.
    pub other: Vec<T>,
    /// The number of entries in this map.
    pub count: usize,
}

#[inline]
fn sort_by_key<T, F: Fn(&T) -> K, K: Ord>(v: &mut [T], f: F) {
    sort_by(v, |a, b| f(a).cmp(&f(b)))
}

impl<T> SelectorMap<T> where T: Clone + Borrow<SelectorInner<SelectorImpl>> {
    /// Trivially constructs an empty `SelectorMap`.
    pub fn new() -> Self {
        SelectorMap {
            id_hash: HashMap::default(),
            class_hash: HashMap::default(),
            local_name_hash: HashMap::default(),
            other: Vec::new(),
            count: 0,
        }
    }

    /// Returns whether there are any entries in the map.
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Returns the number of entries.
    pub fn len(&self) -> usize {
        self.count
    }
}

impl SelectorMap<Rule> {
    /// Append to `rule_list` all Rules in `self` that match element.
    ///
    /// Extract matching rules as per element's ID, classes, tag name, etc..
    /// Sort the Rules at the end to maintain cascading order.
    pub fn get_all_matching_rules<E, V, F>(&self,
                                           element: &E,
                                           parent_bf: Option<&BloomFilter>,
                                           matching_rules_list: &mut V,
                                           relations: &mut StyleRelations,
                                           flags_setter: &mut F,
                                           cascade_level: CascadeLevel)
        where E: Element<Impl=SelectorImpl>,
              V: VecLike<ApplicableDeclarationBlock>,
              F: FnMut(&E, ElementSelectorFlags),
    {
        if self.is_empty() {
            return
        }

        // At the end, we're going to sort the rules that we added, so remember where we began.
        let init_len = matching_rules_list.len();
        if let Some(id) = element.get_id() {
            SelectorMap::get_matching_rules_from_hash(element,
                                                      parent_bf,
                                                      &self.id_hash,
                                                      &id,
                                                      matching_rules_list,
                                                      relations,
                                                      flags_setter,
                                                      cascade_level)
        }

        element.each_class(|class| {
            SelectorMap::get_matching_rules_from_hash(element,
                                                      parent_bf,
                                                      &self.class_hash,
                                                      class,
                                                      matching_rules_list,
                                                      relations,
                                                      flags_setter,
                                                      cascade_level);
        });

        SelectorMap::get_matching_rules_from_hash(element,
                                                  parent_bf,
                                                  &self.local_name_hash,
                                                  element.get_local_name(),
                                                  matching_rules_list,
                                                  relations,
                                                  flags_setter,
                                                  cascade_level);

        SelectorMap::get_matching_rules(element,
                                        parent_bf,
                                        &self.other,
                                        matching_rules_list,
                                        relations,
                                        flags_setter,
                                        cascade_level);

        // Sort only the rules we just added.
        sort_by_key(&mut matching_rules_list[init_len..],
                    |block| (block.specificity, block.source_order));
    }

    /// Append to `rule_list` all universal Rules (rules with selector `*|*`) in
    /// `self` sorted by specificity and source order.
    pub fn get_universal_rules(&self,
                               guard: &SharedRwLockReadGuard,
                               cascade_level: CascadeLevel,
                               important_cascade_level: CascadeLevel)
                               -> Vec<ApplicableDeclarationBlock> {
        debug_assert!(!cascade_level.is_important());
        debug_assert!(important_cascade_level.is_important());
        if self.is_empty() {
            return vec![];
        }

        let mut matching_rules_list = vec![];

        // We need to insert important rules _after_ normal rules for this to be
        // correct, and also to not trigger rule tree assertions.
        let mut important = vec![];
        for rule in self.other.iter() {
            if rule.selector.complex.iter_raw().next().is_none() {
                let style_rule = rule.style_rule.read_with(guard);
                let block = style_rule.block.read_with(guard);
                if block.any_normal() {
                    matching_rules_list.push(
                        rule.to_applicable_declaration_block(cascade_level));
                }
                if block.any_important() {
                    important.push(
                        rule.to_applicable_declaration_block(important_cascade_level));
                }
            }
        }

        let normal_len = matching_rules_list.len();
        matching_rules_list.extend(important.into_iter());

        sort_by_key(&mut matching_rules_list[0..normal_len],
                    |block| (block.specificity, block.source_order));
        sort_by_key(&mut matching_rules_list[normal_len..],
                    |block| (block.specificity, block.source_order));

        matching_rules_list
    }

    fn get_matching_rules_from_hash<E, Str, BorrowedStr: ?Sized, Vector, F>(
        element: &E,
        parent_bf: Option<&BloomFilter>,
        hash: &FnvHashMap<Str, Vec<Rule>>,
        key: &BorrowedStr,
        matching_rules: &mut Vector,
        relations: &mut StyleRelations,
        flags_setter: &mut F,
        cascade_level: CascadeLevel)
        where E: Element<Impl=SelectorImpl>,
              Str: Borrow<BorrowedStr> + Eq + Hash,
              BorrowedStr: Eq + Hash,
              Vector: VecLike<ApplicableDeclarationBlock>,
              F: FnMut(&E, ElementSelectorFlags),
    {
        if let Some(rules) = hash.get(key) {
            SelectorMap::get_matching_rules(element,
                                            parent_bf,
                                            rules,
                                            matching_rules,
                                            relations,
                                            flags_setter,
                                            cascade_level)
        }
    }

    /// Adds rules in `rules` that match `element` to the `matching_rules` list.
    fn get_matching_rules<E, V, F>(element: &E,
                                   parent_bf: Option<&BloomFilter>,
                                   rules: &[Rule],
                                   matching_rules: &mut V,
                                   relations: &mut StyleRelations,
                                   flags_setter: &mut F,
                                   cascade_level: CascadeLevel)
        where E: Element<Impl=SelectorImpl>,
              V: VecLike<ApplicableDeclarationBlock>,
              F: FnMut(&E, ElementSelectorFlags),
    {
        for rule in rules.iter() {
            let any_declaration_for_importance = if cascade_level.is_important() {
                rule.any_important_declarations()
            } else {
                rule.any_normal_declarations()
            };
            if any_declaration_for_importance &&
               matches_selector(&rule.selector, element, parent_bf,
                                relations, flags_setter) {
                matching_rules.push(
                    rule.to_applicable_declaration_block(cascade_level));
            }
        }
    }
}

impl<T> SelectorMap<T> where T: Clone + Borrow<SelectorInner<SelectorImpl>> {
    /// Inserts into the correct hash, trying id, class, and localname.
    pub fn insert(&mut self, entry: T) {
        self.count += 1;

        if let Some(id_name) = get_id_name(entry.borrow()) {
            find_push(&mut self.id_hash, id_name, entry);
            return;
        }

        if let Some(class_name) = get_class_name(entry.borrow()) {
            find_push(&mut self.class_hash, class_name, entry);
            return;
        }

        if let Some(LocalNameSelector { name, lower_name }) = get_local_name(entry.borrow()) {
            // If the local name in the selector isn't lowercase, insert it into
            // the rule hash twice. This means that, during lookup, we can always
            // find the rules based on the local name of the element, regardless
            // of whether it's an html element in an html document (in which case
            // we match against lower_name) or not (in which case we match against
            // name).
            //
            // In the case of a non-html-element-in-html-document with a
            // lowercase localname and a non-lowercase selector, the rulehash
            // lookup may produce superfluous selectors, but the subsequent
            // selector matching work will filter them out.
            if name != lower_name {
                find_push(&mut self.local_name_hash, lower_name, entry.clone());
            }
            find_push(&mut self.local_name_hash, name, entry);

            return;
        }

        self.other.push(entry);
    }

    /// Looks up entries by id, class, local name, and other (in order).
    ///
    /// Each entry is passed to the callback, which returns true to continue
    /// iterating entries, or false to terminate the lookup.
    ///
    /// Returns false if the callback ever returns false.
    ///
    /// FIXME(bholley) This overlaps with SelectorMap<Rule>::get_all_matching_rules,
    /// but that function is extremely hot and I'd rather not rearrange it.
    #[inline]
    pub fn lookup<E, F>(&self, element: E, f: &mut F) -> bool
        where E: TElement,
              F: FnMut(&T) -> bool
    {
        // Id.
        if let Some(id) = element.get_id() {
            if let Some(v) = self.id_hash.get(&id) {
                for entry in v.iter() {
                    if !f(&entry) {
                        return false;
                    }
                }
            }
        }

        // Class.
        let mut done = false;
        element.each_class(|class| {
            if !done {
                if let Some(v) = self.class_hash.get(class) {
                    for entry in v.iter() {
                        if !f(&entry) {
                            done = true;
                            return;
                        }
                    }
                }
            }
        });
        if done {
            return false;
        }

        // Local name.
        if let Some(v) = self.local_name_hash.get(element.get_local_name()) {
            for entry in v.iter() {
                if !f(&entry) {
                    return false;
                }
            }
        }

        // Other.
        for entry in self.other.iter() {
            if !f(&entry) {
                return false;
            }
        }

        true
    }

    /// Performs a normal lookup, and also looks up entries for the passed-in
    /// id and classes.
    ///
    /// Each entry is passed to the callback, which returns true to continue
    /// iterating entries, or false to terminate the lookup.
    ///
    /// Returns false if the callback ever returns false.
    #[inline]
    pub fn lookup_with_additional<E, F>(&self,
                                        element: E,
                                        additional_id: Option<Atom>,
                                        additional_classes: &[Atom],
                                        f: &mut F)
                                        -> bool
        where E: TElement,
              F: FnMut(&T) -> bool
    {
        // Do the normal lookup.
        if !self.lookup(element, f) {
            return false;
        }

        // Check the additional id.
        if let Some(id) = additional_id {
            if let Some(v) = self.id_hash.get(&id) {
                for entry in v.iter() {
                    if !f(&entry) {
                        return false;
                    }
                }
            }
        }

        // Check the additional classes.
        for class in additional_classes {
            if let Some(v) = self.class_hash.get(class) {
                for entry in v.iter() {
                    if !f(&entry) {
                        return false;
                    }
                }
            }
        }

        true
    }
}

/// Retrieve the first ID name in the selector, or None otherwise.
pub fn get_id_name(selector: &SelectorInner<SelectorImpl>) -> Option<Atom> {
    for ss in selector.complex.iter() {
        // TODO(pradeep): Implement case-sensitivity based on the
        // document type and quirks mode.
        if let Component::ID(ref id) = *ss {
            return Some(id.clone());
        }
    }

    None
}

/// Retrieve the FIRST class name in the selector, or None otherwise.
pub fn get_class_name(selector: &SelectorInner<SelectorImpl>) -> Option<Atom> {
    for ss in selector.complex.iter() {
        // TODO(pradeep): Implement case-sensitivity based on the
        // document type and quirks mode.
        if let Component::Class(ref class) = *ss {
            return Some(class.clone());
        }
    }

    None
}

/// Retrieve the name if it is a type selector, or None otherwise.
pub fn get_local_name(selector: &SelectorInner<SelectorImpl>)
                      -> Option<LocalNameSelector<SelectorImpl>> {
    for ss in selector.complex.iter() {
        if let Component::LocalName(ref n) = *ss {
            return Some(LocalNameSelector {
                name: n.name.clone(),
                lower_name: n.lower_name.clone(),
            })
        }
    }

    None
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
    pub selector: SelectorInner<SelectorImpl>,
    /// The actual style rule.
    #[cfg_attr(feature = "servo", ignore_heap_size_of = "Arc")]
    pub style_rule: Arc<Locked<StyleRule>>,
    /// The source order this style rule appears in.
    pub source_order: usize,
    /// Bottom 30 bits: The specificity of the rule this selector represents.
    /// 31st bit: Whether the rule's declaration block has any important declarations.
    /// 32nd bit: Whether the rule's declaration block has any normal declarations.
    specificity_and_bits: u32,
}

impl Borrow<SelectorInner<SelectorImpl>> for Rule {
    fn borrow(&self) -> &SelectorInner<SelectorImpl> {
        &self.selector
    }
}

/// Masks for specificity_and_bits.
const SPECIFICITY_MASK: u32 = 0x3fffffff;
const ANY_IMPORTANT_DECLARATIONS_BIT: u32 = 1 << 30;
const ANY_NORMAL_DECLARATIONS_BIT: u32 = 1 << 31;

impl Rule {
    /// Returns the specificity of the rule.
    pub fn specificity(&self) -> u32 {
        self.specificity_and_bits & SPECIFICITY_MASK
    }

    fn any_important_declarations(&self) -> bool {
        (self.specificity_and_bits & ANY_IMPORTANT_DECLARATIONS_BIT) != 0
    }

    fn any_normal_declarations(&self) -> bool {
        (self.specificity_and_bits & ANY_NORMAL_DECLARATIONS_BIT) != 0
    }

    fn to_applicable_declaration_block(&self, level: CascadeLevel) -> ApplicableDeclarationBlock {
        ApplicableDeclarationBlock {
            source: StyleSource::Style(self.style_rule.clone()),
            level: level,
            source_order: self.source_order,
            specificity: self.specificity(),
        }
    }

    /// Creates a new Rule.
    pub fn new(guard: &SharedRwLockReadGuard,
               selector: SelectorInner<SelectorImpl>,
               style_rule: Arc<Locked<StyleRule>>,
               source_order: usize,
               specificity: u32)
               -> Self
    {
        let (any_important, any_normal) = {
            let block = style_rule.read_with(guard).block.read_with(guard);
            (block.any_important(), block.any_normal())
        };
        debug_assert!(specificity & (ANY_IMPORTANT_DECLARATIONS_BIT | ANY_NORMAL_DECLARATIONS_BIT) == 0);
        let mut specificity_and_bits = specificity;
        if any_important {
            specificity_and_bits |= ANY_IMPORTANT_DECLARATIONS_BIT;
        }
        if any_normal {
            specificity_and_bits |= ANY_NORMAL_DECLARATIONS_BIT;
        }

        Rule {
            selector: selector,
            style_rule: style_rule,
            source_order: source_order,
            specificity_and_bits: specificity_and_bits,
        }
    }
}

/// A property declaration together with its precedence among rules of equal
/// specificity so that we can sort them.
///
/// This represents the declarations in a given declaration block for a given
/// importance.
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[derive(Debug, Clone)]
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

#[inline]
fn find_push<Str: Eq + Hash, V>(map: &mut FnvHashMap<Str, Vec<V>>,
                                key: Str,
                                value: V) {
    map.entry(key).or_insert_with(Vec::new).push(value)
}
