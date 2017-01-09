/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Selector matching.

#![deny(missing_docs)]

use {Atom, LocalName};
use data::ComputedStyle;
use dom::{PresentationalHintsSynthetizer, TElement};
use error_reporting::StdoutErrorReporter;
use keyframes::KeyframesAnimation;
use media_queries::Device;
use parking_lot::RwLock;
use properties::{self, CascadeFlags, ComputedValues, INHERIT_ALL, Importance};
use properties::{PropertyDeclaration, PropertyDeclarationBlock};
use quickersort::sort_by;
use restyle_hints::{RestyleHint, DependencySet};
use rule_tree::{RuleTree, StrongRuleNode, StyleSource};
use selector_parser::{ElementExt, SelectorImpl, PseudoElement, Snapshot};
use selectors::Element;
use selectors::bloom::BloomFilter;
use selectors::matching::{AFFECTED_BY_STYLE_ATTRIBUTE, AFFECTED_BY_PRESENTATIONAL_HINTS};
use selectors::matching::{MatchingReason, StyleRelations, matches_complex_selector};
use selectors::parser::{Selector, SimpleSelector, LocalName as LocalNameSelector, ComplexSelector};
use sink::Push;
use smallvec::VecLike;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::fmt;
use std::hash::Hash;
use std::slice;
use std::sync::Arc;
use style_traits::viewport::ViewportConstraints;
use stylesheets::{CssRule, Origin, StyleRule, Stylesheet, UserAgentStylesheets};
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
    pub device: Arc<Device>,

    /// Viewport constraints based on the current device.
    viewport_constraints: Option<ViewportConstraints>,

    /// If true, the quirks-mode stylesheet is applied.
    quirks_mode: bool,

    /// If true, the device has changed, and the stylist needs to be updated.
    is_device_dirty: bool,

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
    state_deps: DependencySet,

    /// Selectors in the page affecting siblings
    #[cfg_attr(feature = "servo", ignore_heap_size_of = "Arc")]
    sibling_affecting_selectors: Vec<Selector<SelectorImpl>>,

    /// Selectors in the page matching elements with non-common style-affecting
    /// attributes.
    #[cfg_attr(feature = "servo", ignore_heap_size_of = "Arc")]
    non_common_style_affecting_attributes_selectors: Vec<Selector<SelectorImpl>>,
}

impl Stylist {
    /// Construct a new `Stylist`, using a given `Device`.
    #[inline]
    pub fn new(device: Device) -> Self {
        let mut stylist = Stylist {
            viewport_constraints: None,
            device: Arc::new(device),
            is_device_dirty: true,
            quirks_mode: false,

            element_map: PerPseudoElementSelectorMap::new(),
            pseudos_map: Default::default(),
            animations: Default::default(),
            precomputed_pseudo_element_decls: Default::default(),
            rules_source_order: 0,
            rule_tree: RuleTree::new(),
            state_deps: DependencySet::new(),

            // XXX remember resetting them!
            sibling_affecting_selectors: vec![],
            non_common_style_affecting_attributes_selectors: vec![]
        };

        SelectorImpl::each_eagerly_cascaded_pseudo_element(|pseudo| {
            stylist.pseudos_map.insert(pseudo, PerPseudoElementSelectorMap::new());
        });

        // FIXME: Add iso-8859-9.css when the document’s encoding is ISO-8859-8.

        stylist
    }

    /// Update the stylist for the given document stylesheets, and optionally
    /// with a set of user agent stylesheets.
    ///
    /// This method resets all the style data each time the stylesheets change
    /// (which is indicated by the `stylesheets_changed` parameter), or the
    /// device is dirty, which means we need to re-evaluate media queries.
    pub fn update(&mut self,
                  doc_stylesheets: &[Arc<Stylesheet>],
                  ua_stylesheets: Option<&UserAgentStylesheets>,
                  stylesheets_changed: bool) -> bool {
        if !(self.is_device_dirty || stylesheets_changed) {
            return false;
        }

        self.element_map = PerPseudoElementSelectorMap::new();
        self.pseudos_map = Default::default();
        self.animations = Default::default();
        SelectorImpl::each_eagerly_cascaded_pseudo_element(|pseudo| {
            self.pseudos_map.insert(pseudo, PerPseudoElementSelectorMap::new());
        });

        self.precomputed_pseudo_element_decls = Default::default();
        self.rules_source_order = 0;
        self.state_deps.clear();

        self.sibling_affecting_selectors.clear();
        self.non_common_style_affecting_attributes_selectors.clear();

        if let Some(ua_stylesheets) = ua_stylesheets {
            for stylesheet in &ua_stylesheets.user_or_user_agent_stylesheets {
                self.add_stylesheet(&stylesheet);
            }

            if self.quirks_mode {
                self.add_stylesheet(&ua_stylesheets.quirks_mode_stylesheet);
            }
        }

        for ref stylesheet in doc_stylesheets.iter() {
            self.add_stylesheet(stylesheet);
        }

        debug!("Stylist stats:");
        debug!(" - Got {} sibling-affecting selectors",
               self.sibling_affecting_selectors.len());
        debug!(" - Got {} non-common-style-attribute-affecting selectors",
               self.non_common_style_affecting_attributes_selectors.len());
        debug!(" - Got {} deps for style-hint calculation",
               self.state_deps.len());

        SelectorImpl::each_precomputed_pseudo_element(|pseudo| {
            // TODO: Consider not doing this and just getting the rules on the
            // fly. It should be a bit slower, but we'd take rid of the
            // extra field, and avoid this precomputation entirely.
            if let Some(map) = self.pseudos_map.remove(&pseudo) {
                let mut declarations = vec![];
                map.user_agent.get_universal_rules(&mut declarations);
                self.precomputed_pseudo_element_decls.insert(pseudo, declarations);
            }
        });

        self.is_device_dirty = false;
        true
    }

    fn add_stylesheet(&mut self, stylesheet: &Stylesheet) {
        if stylesheet.disabled() || !stylesheet.is_effective_for_device(&self.device) {
            return;
        }

        // Cheap `Arc` clone so that the closure below can borrow `&mut Stylist`.
        let device = self.device.clone();

        stylesheet.effective_rules(&device, |rule| {
            match *rule {
                CssRule::Style(ref style_rule) => {
                    let guard = style_rule.read();
                    for selector in &guard.selectors.0 {
                        let map = if let Some(ref pseudo) = selector.pseudo_element {
                            self.pseudos_map
                                .entry(pseudo.clone())
                                .or_insert_with(PerPseudoElementSelectorMap::new)
                                .borrow_for_origin(&stylesheet.origin)
                        } else {
                            self.element_map.borrow_for_origin(&stylesheet.origin)
                        };

                        map.insert(Rule {
                            selector: selector.complex_selector.clone(),
                            style_rule: style_rule.clone(),
                            specificity: selector.specificity,
                            source_order: self.rules_source_order,
                        });
                    }
                    self.rules_source_order += 1;

                    for selector in &guard.selectors.0 {
                        self.state_deps.note_selector(&selector.complex_selector);
                        if selector.affects_siblings() {
                            self.sibling_affecting_selectors.push(selector.clone());
                        }

                        if selector.matches_non_common_style_affecting_attribute() {
                            self.non_common_style_affecting_attributes_selectors.push(selector.clone());
                        }
                    }
                }
                CssRule::Import(ref import) => {
                    let import = import.read();
                    self.add_stylesheet(&import.stylesheet)
                }
                CssRule::Keyframes(ref keyframes_rule) => {
                    let keyframes_rule = keyframes_rule.read();
                    debug!("Found valid keyframes rule: {:?}", *keyframes_rule);
                    if let Some(animation) = KeyframesAnimation::from_keyframes(&keyframes_rule.keyframes) {
                        debug!("Found valid keyframe animation: {:?}", animation);
                        self.animations.insert(keyframes_rule.name.clone(),
                                               animation);
                    } else {
                        // If there's a valid keyframes rule, even if it doesn't
                        // produce an animation, should shadow other animations
                        // with the same name.
                        self.animations.remove(&keyframes_rule.name);
                    }
                }
                // We don't care about any other rule.
                _ => {}
            }
        });
    }


    /// Computes the style for a given "precomputed" pseudo-element, taking the
    /// universal rules and applying them.
    ///
    /// If `inherit_all` is true, then all properties are inherited from the
    /// parent; otherwise, non-inherited properties are reset to their initial
    /// values. The flow constructor uses this flag when constructing anonymous
    /// flows.
    pub fn precomputed_values_for_pseudo(&self,
                                         pseudo: &PseudoElement,
                                         parent: Option<&Arc<ComputedValues>>,
                                         default: &Arc<ComputedValues>,
                                         inherit_all: bool)
                                         -> Option<ComputedStyle> {
        debug_assert!(SelectorImpl::pseudo_element_cascade_type(pseudo).is_precomputed());
        if let Some(declarations) = self.precomputed_pseudo_element_decls.get(pseudo) {
            // FIXME(emilio): When we've taken rid of the cascade we can just
            // use into_iter.
            let rule_node =
                self.rule_tree.insert_ordered_rules(
                    declarations.into_iter().map(|a| (a.source.clone(), a.importance)));

            let mut flags = CascadeFlags::empty();
            if inherit_all {
                flags.insert(INHERIT_ALL)
            }

            let computed =
                properties::cascade(self.device.au_viewport_size(),
                                    &rule_node,
                                    parent.map(|p| &**p),
                                    default,
                                    None,
                                    Box::new(StdoutErrorReporter),
                                    flags);
            Some(ComputedStyle::new(rule_node, Arc::new(computed)))
        } else {
            parent.map(|p| ComputedStyle::new(self.rule_tree.root(), p.clone()))
        }
    }

    /// Returns the style for an anonymous box of the given type.
    #[cfg(feature = "servo")]
    pub fn style_for_anonymous_box(&self,
                                   pseudo: &PseudoElement,
                                   parent_style: &Arc<ComputedValues>,
                                   default_style: &Arc<ComputedValues>)
                                   -> Arc<ComputedValues> {
        // For most (but not all) pseudo-elements, we inherit all values from the parent.
        let inherit_all = match *pseudo {
            PseudoElement::ServoInputText => false,
            PseudoElement::ServoAnonymousBlock |
            PseudoElement::ServoAnonymousTable |
            PseudoElement::ServoAnonymousTableCell |
            PseudoElement::ServoAnonymousTableRow |
            PseudoElement::ServoAnonymousTableWrapper |
            PseudoElement::ServoTableWrapper => true,
            PseudoElement::Before |
            PseudoElement::After |
            PseudoElement::Selection |
            PseudoElement::DetailsSummary |
            PseudoElement::DetailsContent => {
                unreachable!("That pseudo doesn't represent an anonymous box!")
            }
        };
        self.precomputed_values_for_pseudo(&pseudo, Some(parent_style), default_style, inherit_all)
            .expect("style_for_anonymous_box(): No precomputed values for that pseudo!")
            .values
    }

    /// Computes a pseudo-element style lazily during layout.
    ///
    /// This can only be done for a certain set of pseudo-elements, like
    /// :selection.
    ///
    /// Check the documentation on lazy pseudo-elements in
    /// docs/components/style.md
    pub fn lazily_compute_pseudo_element_style<E>(&self,
                                                  element: &E,
                                                  pseudo: &PseudoElement,
                                                  parent: &Arc<ComputedValues>,
                                                  default: &Arc<ComputedValues>)
                                                  -> Option<ComputedStyle>
        where E: ElementExt +
                 fmt::Debug +
                 PresentationalHintsSynthetizer
    {
        debug_assert!(SelectorImpl::pseudo_element_cascade_type(pseudo).is_lazy());
        if self.pseudos_map.get(pseudo).is_none() {
            return None;
        }

        let mut declarations = vec![];

        self.push_applicable_declarations(element,
                                          None,
                                          None,
                                          Some(pseudo),
                                          &mut declarations,
                                          MatchingReason::ForStyling);

        let rule_node =
            self.rule_tree.insert_ordered_rules(
                declarations.into_iter().map(|a| (a.source, a.importance)));

        let computed =
            properties::cascade(self.device.au_viewport_size(),
                                &rule_node,
                                Some(&**parent),
                                default,
                                None,
                                Box::new(StdoutErrorReporter),
                                CascadeFlags::empty());

        Some(ComputedStyle::new(rule_node, Arc::new(computed)))
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
    pub fn set_device(&mut self, mut device: Device, stylesheets: &[Arc<Stylesheet>]) {
        let cascaded_rule = ViewportRule {
            declarations: viewport::Cascade::from_stylesheets(stylesheets, &device).finish(),
        };

        self.viewport_constraints =
            ViewportConstraints::maybe_new(&device, &cascaded_rule);

        if let Some(ref constraints) = self.viewport_constraints {
            device.account_for_viewport_rule(constraints);
        }

        fn mq_eval_changed(rules: &[CssRule], before: &Device, after: &Device) -> bool {
            for rule in rules {
                if rule.with_nested_rules_and_mq(|rules, mq| {
                    if let Some(mq) = mq {
                        if mq.evaluate(before) != mq.evaluate(after) {
                            return true
                        }
                    }
                    mq_eval_changed(rules, before, after)
                }) {
                    return true
                }
            }
            false
        }
        self.is_device_dirty |= stylesheets.iter().any(|stylesheet| {
            mq_eval_changed(&stylesheet.rules.read().0, &self.device, &device)
        });

        self.device = Arc::new(device);
    }

    /// Returns the viewport constraints that apply to this document because of
    /// a @viewport rule.
    pub fn viewport_constraints(&self) -> Option<&ViewportConstraints> {
        self.viewport_constraints.as_ref()
    }

    /// Sets the quirks mode of the document.
    pub fn set_quirks_mode(&mut self, enabled: bool) {
        // FIXME(emilio): We don't seem to change the quirks mode dynamically
        // during multiple layout passes, but this is totally bogus, in the
        // sense that it's updated asynchronously.
        //
        // This should probably be an argument to `update`, and use the quirks
        // mode info in the `SharedLayoutContext`.
        self.quirks_mode = enabled;
    }

    /// Returns the applicable CSS declarations for the given element.
    ///
    /// This corresponds to `ElementRuleCollector` in WebKit.
    ///
    /// The returned `StyleRelations` indicate hints about which kind of rules
    /// have matched.
    pub fn push_applicable_declarations<E, V>(
                                        &self,
                                        element: &E,
                                        parent_bf: Option<&BloomFilter>,
                                        style_attribute: Option<&Arc<RwLock<PropertyDeclarationBlock>>>,
                                        pseudo_element: Option<&PseudoElement>,
                                        applicable_declarations: &mut V,
                                        reason: MatchingReason) -> StyleRelations
        where E: ElementExt +
                 fmt::Debug +
                 PresentationalHintsSynthetizer,
              V: Push<ApplicableDeclarationBlock> + VecLike<ApplicableDeclarationBlock>
    {
        debug_assert!(!self.is_device_dirty);
        debug_assert!(style_attribute.is_none() || pseudo_element.is_none(),
                      "Style attributes do not apply to pseudo-elements");
        debug_assert!(pseudo_element.is_none() ||
                      !SelectorImpl::pseudo_element_cascade_type(pseudo_element.as_ref().unwrap())
                        .is_precomputed());

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
                                              reason,
                                              Importance::Normal);
        debug!("UA normal: {:?}", relations);

        // Step 2: Presentational hints.
        let length = applicable_declarations.len();
        element.synthesize_presentational_hints_for_legacy_attributes(applicable_declarations);
        if applicable_declarations.len() != length {
            // Never share style for elements with preshints
            relations |= AFFECTED_BY_PRESENTATIONAL_HINTS;
        }
        debug!("preshints: {:?}", relations);

        if element.matches_user_and_author_rules() {
            // Step 3: User and author normal rules.
            map.user.get_all_matching_rules(element,
                                            parent_bf,
                                            applicable_declarations,
                                            &mut relations,
                                            reason,
                                            Importance::Normal);
            debug!("user normal: {:?}", relations);
            map.author.get_all_matching_rules(element,
                                              parent_bf,
                                              applicable_declarations,
                                              &mut relations,
                                              reason,
                                              Importance::Normal);
            debug!("author normal: {:?}", relations);

            // Step 4: Normal style attributes.
            if let Some(sa) = style_attribute {
                if sa.read().any_normal() {
                    relations |= AFFECTED_BY_STYLE_ATTRIBUTE;
                    Push::push(
                        applicable_declarations,
                        ApplicableDeclarationBlock::from_declarations(sa.clone(), Importance::Normal));
                }
            }

            debug!("style attr: {:?}", relations);

            // Step 5: Author-supplied `!important` rules.
            map.author.get_all_matching_rules(element,
                                              parent_bf,
                                              applicable_declarations,
                                              &mut relations,
                                              reason,
                                              Importance::Important);

            debug!("author important: {:?}", relations);

            // Step 6: `!important` style attributes.
            if let Some(sa) = style_attribute {
                if sa.read().any_important() {
                    relations |= AFFECTED_BY_STYLE_ATTRIBUTE;
                    Push::push(
                        applicable_declarations,
                        ApplicableDeclarationBlock::from_declarations(sa.clone(), Importance::Important));
                }
            }

            debug!("style attr important: {:?}", relations);

            // Step 7: User `!important` rules.
            map.user.get_all_matching_rules(element,
                                            parent_bf,
                                            applicable_declarations,
                                            &mut relations,
                                            reason,
                                            Importance::Important);

            debug!("user important: {:?}", relations);
        } else {
            debug!("skipping non-agent rules");
        }

        // Step 8: UA `!important` rules.
        map.user_agent.get_all_matching_rules(element,
                                              parent_bf,
                                              applicable_declarations,
                                              &mut relations,
                                              reason,
                                              Importance::Important);

        debug!("UA important: {:?}", relations);

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

    /// Whether two elements match the same not-common style-affecting attribute
    /// rules.
    ///
    /// This is used to test elements and candidates in the style-sharing
    /// candidate cache.
    pub fn match_same_not_common_style_affecting_attributes_rules<E>(&self,
                                                                     element: &E,
                                                                     candidate: &E) -> bool
        where E: ElementExt,
    {
        use selectors::matching::StyleRelations;
        use selectors::matching::matches_complex_selector;
        // TODO(emilio): we can probably do better, the candidate should already
        // know what rules it matches. Also, we should only match until we find
        // a descendant combinator, the rest should be ok, since the parent is
        // the same.
        //
        // TODO(emilio): Use the bloom filter, since they contain the element's
        // ancestor chain and it's correct for the candidate too.
        for ref selector in self.non_common_style_affecting_attributes_selectors.iter() {
            let element_matches =
                matches_complex_selector(&selector.complex_selector, element,
                                         None, &mut StyleRelations::empty(),
                                         MatchingReason::Other);
            let candidate_matches =
                matches_complex_selector(&selector.complex_selector, candidate,
                                         None, &mut StyleRelations::empty(),
                                         MatchingReason::Other);

            if element_matches != candidate_matches {
                return false;
            }
        }

        true
    }

    /// Returns the rule root node.
    #[inline]
    pub fn rule_tree_root(&self) -> StrongRuleNode {
        self.rule_tree.root()
    }

    /// Returns whether two elements match the same sibling-affecting rules.
    ///
    /// This is also for the style sharing candidate cache.
    pub fn match_same_sibling_affecting_rules<E>(&self,
                                                 element: &E,
                                                 candidate: &E) -> bool
        where E: ElementExt,
    {
        use selectors::matching::StyleRelations;
        use selectors::matching::matches_complex_selector;
        // TODO(emilio): we can probably do better, the candidate should already
        // know what rules it matches.
        //
        // TODO(emilio): Use the bloom filter, since they contain the element's
        // ancestor chain and it's correct for the candidate too.
        for ref selector in self.sibling_affecting_selectors.iter() {
            let element_matches =
                matches_complex_selector(&selector.complex_selector, element,
                                         None, &mut StyleRelations::empty(),
                                         MatchingReason::Other);

            let candidate_matches =
                matches_complex_selector(&selector.complex_selector, candidate,
                                         None, &mut StyleRelations::empty(),
                                         MatchingReason::Other);

            if element_matches != candidate_matches {
                debug!("match_same_sibling_affecting_rules: Failure due to {:?}",
                       selector.complex_selector);
                return false;
            }
        }

        true
    }

    /// Given an element, and a snapshot that represents a previous state of the
    /// element, compute the appropriate restyle hint, that is, the kind of
    /// restyle we need to do.
    pub fn compute_restyle_hint<E>(&self,
                                   element: &E,
                                   snapshot: &Snapshot)
                                   -> RestyleHint
        where E: TElement,
    {
        self.state_deps.compute_hint(element, snapshot)
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


/// Map that contains the CSS rules for a specific PseudoElement
/// (or lack of PseudoElement).
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
struct PerPseudoElementSelectorMap {
    /// Rules from user agent stylesheets
    user_agent: SelectorMap,
    /// Rules from author stylesheets
    author: SelectorMap,
    /// Rules from user stylesheets
    user: SelectorMap,
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
    fn borrow_for_origin(&mut self, origin: &Origin) -> &mut SelectorMap {
        match *origin {
            Origin::UserAgent => &mut self.user_agent,
            Origin::Author => &mut self.author,
            Origin::User => &mut self.user,
        }
    }
}

/// Map element data to Rules whose last simple selector starts with them.
///
/// e.g.,
/// "p > img" would go into the set of Rules corresponding to the
/// element "img"
/// "a .foo .bar.baz" would go into the set of Rules corresponding to
/// the class "bar"
///
/// Because we match Rules right-to-left (i.e., moving up the tree
/// from an element), we need to compare the last simple selector in the
/// Rule with the element.
///
/// So, if an element has ID "id1" and classes "foo" and "bar", then all
/// the rules it matches will have their last simple selector starting
/// either with "#id1" or with ".foo" or with ".bar".
///
/// Hence, the union of the rules keyed on each of element's classes, ID,
/// element name, etc. will contain the Rules that actually match that
/// element.
///
/// TODO: Tune the initial capacity of the HashMap
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct SelectorMap {
    /// A hash from an ID to rules which contain that ID selector.
    pub id_hash: FnvHashMap<Atom, Vec<Rule>>,
    /// A hash from a class name to rules which contain that class selector.
    pub class_hash: FnvHashMap<Atom, Vec<Rule>>,
    /// A hash from local name to rules which contain that local name selector.
    pub local_name_hash: FnvHashMap<LocalName, Vec<Rule>>,
    /// Same as local_name_hash, but keys are lower-cased.
    /// For HTML elements in HTML documents.
    pub lower_local_name_hash: FnvHashMap<LocalName, Vec<Rule>>,
    /// Rules that don't have ID, class, or element selectors.
    pub other_rules: Vec<Rule>,
    /// Whether this hash is empty.
    pub empty: bool,
}

#[inline]
fn sort_by_key<T, F: Fn(&T) -> K, K: Ord>(v: &mut [T], f: F) {
    sort_by(v, &|a, b| f(a).cmp(&f(b)))
}

impl SelectorMap {
    /// Trivially constructs an empty `SelectorMap`.
    pub fn new() -> Self {
        SelectorMap {
            id_hash: HashMap::default(),
            class_hash: HashMap::default(),
            local_name_hash: HashMap::default(),
            lower_local_name_hash: HashMap::default(),
            other_rules: Vec::new(),
            empty: true,
        }
    }

    /// Append to `rule_list` all Rules in `self` that match element.
    ///
    /// Extract matching rules as per element's ID, classes, tag name, etc..
    /// Sort the Rules at the end to maintain cascading order.
    pub fn get_all_matching_rules<E, V>(&self,
                                        element: &E,
                                        parent_bf: Option<&BloomFilter>,
                                        matching_rules_list: &mut V,
                                        relations: &mut StyleRelations,
                                        reason: MatchingReason,
                                        importance: Importance)
        where E: Element<Impl=SelectorImpl>,
              V: VecLike<ApplicableDeclarationBlock>
    {
        if self.empty {
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
                                                      reason,
                                                      importance)
        }

        element.each_class(|class| {
            SelectorMap::get_matching_rules_from_hash(element,
                                                      parent_bf,
                                                      &self.class_hash,
                                                      class,
                                                      matching_rules_list,
                                                      relations,
                                                      reason,
                                                      importance);
        });

        let local_name_hash = if element.is_html_element_in_html_document() {
            &self.lower_local_name_hash
        } else {
            &self.local_name_hash
        };
        SelectorMap::get_matching_rules_from_hash(element,
                                                  parent_bf,
                                                  local_name_hash,
                                                  element.get_local_name(),
                                                  matching_rules_list,
                                                  relations,
                                                  reason,
                                                  importance);

        SelectorMap::get_matching_rules(element,
                                        parent_bf,
                                        &self.other_rules,
                                        matching_rules_list,
                                        relations,
                                        reason,
                                        importance);

        // Sort only the rules we just added.
        sort_by_key(&mut matching_rules_list[init_len..],
                    |block| (block.specificity, block.source_order));
    }

    /// Append to `rule_list` all universal Rules (rules with selector `*|*`) in
    /// `self` sorted by specifity and source order.
    pub fn get_universal_rules<V>(&self,
                                  matching_rules_list: &mut V)
        where V: VecLike<ApplicableDeclarationBlock>
    {
        if self.empty {
            return
        }

        let init_len = matching_rules_list.len();

        for rule in self.other_rules.iter() {
            if rule.selector.compound_selector.is_empty() &&
               rule.selector.next.is_none() {
                let guard = rule.style_rule.read();
                let block = guard.block.read();
                if block.any_normal() {
                    matching_rules_list.push(
                        rule.to_applicable_declaration_block(Importance::Normal));
                }
                if block.any_important() {
                    matching_rules_list.push(
                        rule.to_applicable_declaration_block(Importance::Important));
                }
            }
        }

        sort_by_key(&mut matching_rules_list[init_len..],
                    |block| (block.specificity, block.source_order));
    }

    fn get_matching_rules_from_hash<E, Str, BorrowedStr: ?Sized, Vector>(
        element: &E,
        parent_bf: Option<&BloomFilter>,
        hash: &FnvHashMap<Str, Vec<Rule>>,
        key: &BorrowedStr,
        matching_rules: &mut Vector,
        relations: &mut StyleRelations,
        reason: MatchingReason,
        importance: Importance)
        where E: Element<Impl=SelectorImpl>,
              Str: Borrow<BorrowedStr> + Eq + Hash,
              BorrowedStr: Eq + Hash,
              Vector: VecLike<ApplicableDeclarationBlock>
    {
        if let Some(rules) = hash.get(key) {
            SelectorMap::get_matching_rules(element,
                                            parent_bf,
                                            rules,
                                            matching_rules,
                                            relations,
                                            reason,
                                            importance)
        }
    }

    /// Adds rules in `rules` that match `element` to the `matching_rules` list.
    fn get_matching_rules<E, V>(element: &E,
                                parent_bf: Option<&BloomFilter>,
                                rules: &[Rule],
                                matching_rules: &mut V,
                                relations: &mut StyleRelations,
                                reason: MatchingReason,
                                importance: Importance)
        where E: Element<Impl=SelectorImpl>,
              V: VecLike<ApplicableDeclarationBlock>
    {
        for rule in rules.iter() {
            let guard = rule.style_rule.read();
            let block = guard.block.read();
            let any_declaration_for_importance = if importance.important() {
                block.any_important()
            } else {
                block.any_normal()
            };
            if any_declaration_for_importance &&
               matches_complex_selector(&*rule.selector, element, parent_bf,
                                        relations, reason) {
                matching_rules.push(
                    rule.to_applicable_declaration_block(importance));
            }
        }
    }

    /// Insert rule into the correct hash.
    /// Order in which to try: id_hash, class_hash, local_name_hash, other_rules.
    pub fn insert(&mut self, rule: Rule) {
        self.empty = false;

        if let Some(id_name) = SelectorMap::get_id_name(&rule) {
            find_push(&mut self.id_hash, id_name, rule);
            return;
        }

        if let Some(class_name) = SelectorMap::get_class_name(&rule) {
            find_push(&mut self.class_hash, class_name, rule);
            return;
        }

        if let Some(LocalNameSelector { name, lower_name }) = SelectorMap::get_local_name(&rule) {
            find_push(&mut self.local_name_hash, name, rule.clone());
            find_push(&mut self.lower_local_name_hash, lower_name, rule);
            return;
        }

        self.other_rules.push(rule);
    }

    /// Retrieve the first ID name in Rule, or None otherwise.
    pub fn get_id_name(rule: &Rule) -> Option<Atom> {
        for ss in &rule.selector.compound_selector {
            // TODO(pradeep): Implement case-sensitivity based on the
            // document type and quirks mode.
            if let SimpleSelector::ID(ref id) = *ss {
                return Some(id.clone());
            }
        }

        None
    }

    /// Retrieve the FIRST class name in Rule, or None otherwise.
    pub fn get_class_name(rule: &Rule) -> Option<Atom> {
        for ss in &rule.selector.compound_selector {
            // TODO(pradeep): Implement case-sensitivity based on the
            // document type and quirks mode.
            if let SimpleSelector::Class(ref class) = *ss {
                return Some(class.clone());
            }
        }

        None
    }

    /// Retrieve the name if it is a type selector, or None otherwise.
    pub fn get_local_name(rule: &Rule) -> Option<LocalNameSelector<SelectorImpl>> {
        for ss in &rule.selector.compound_selector {
            if let SimpleSelector::LocalName(ref n) = *ss {
                return Some(LocalNameSelector {
                    name: n.name.clone(),
                    lower_name: n.lower_name.clone(),
                })
            }
        }

        None
    }
}

/// A rule, that wraps a style rule, but represents a single selector of the
/// rule.
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[derive(Clone, Debug)]
pub struct Rule {
    /// The selector this struct represents.
    /// This is an Arc because Rule will essentially be cloned for every element
    /// that it matches. Selector contains an owned vector (through
    /// ComplexSelector) and we want to avoid the allocation.
    ///
    /// FIXME(emilio): We should be able to get rid of it and just use the style
    /// rule? This predates the time where the rule was in `selectors`, and the
    /// style rule was a generic parameter to it. It's not trivial though, due
    /// to the specificity.
    #[cfg_attr(feature = "servo", ignore_heap_size_of = "Arc")]
    pub selector: Arc<ComplexSelector<SelectorImpl>>,
    /// The actual style rule.
    #[cfg_attr(feature = "servo", ignore_heap_size_of = "Arc")]
    pub style_rule: Arc<RwLock<StyleRule>>,
    /// The source order this style rule appears in.
    pub source_order: usize,
    /// The specificity of the rule this selector represents.
    pub specificity: u32,
}

impl Rule {
    fn to_applicable_declaration_block(&self, importance: Importance) -> ApplicableDeclarationBlock {
        ApplicableDeclarationBlock {
            source: StyleSource::Style(self.style_rule.clone()),
            importance: importance,
            source_order: self.source_order,
            specificity: self.specificity,
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
    /// The importance of this declaration block.
    pub importance: Importance,
    /// The source order of this block.
    pub source_order: usize,
    /// The specificity of the selector this block is represented by.
    pub specificity: u32,
}

impl ApplicableDeclarationBlock {
    /// Constructs an applicable declaration block from a given property
    /// declaration block and importance.
    #[inline]
    pub fn from_declarations(declarations: Arc<RwLock<PropertyDeclarationBlock>>,
                             importance: Importance)
                             -> Self {
        ApplicableDeclarationBlock {
            source: StyleSource::Declarations(declarations),
            source_order: 0,
            specificity: 0,
            importance: importance,
        }
    }
}

/// An iterator over the declarations that a given block represent, which is
/// effectively a filter by importance.
pub struct ApplicableDeclarationBlockIter<'a> {
    iter: slice::Iter<'a, (PropertyDeclaration, Importance)>,
    importance: Importance,
}

impl<'a> Iterator for ApplicableDeclarationBlockIter<'a> {
    type Item = &'a PropertyDeclaration;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(&(ref declaration, importance)) = self.iter.next() {
            if importance == self.importance {
                return Some(declaration)
            }
        }
        None
    }
}

impl<'a> DoubleEndedIterator for ApplicableDeclarationBlockIter<'a> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        while let Some(&(ref declaration, importance)) = self.iter.next_back() {
            if importance == self.importance {
                return Some(declaration)
            }
        }
        None
    }
}

#[inline]
fn find_push<Str: Eq + Hash>(map: &mut FnvHashMap<Str, Vec<Rule>>, key: Str,
                             value: Rule) {
    map.entry(key).or_insert_with(Vec::new).push(value)
}
