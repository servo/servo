/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Selector matching.

use dom::PresentationalHintsSynthetizer;
use element_state::*;
use error_reporting::StdoutErrorReporter;
use keyframes::KeyframesAnimation;
use media_queries::{Device, MediaType};
use properties::{self, PropertyDeclaration, PropertyDeclarationBlock, ComputedValues, Importance};
use quickersort::sort_by;
use restyle_hints::{RestyleHint, DependencySet};
use selector_impl::{ElementExt, TheSelectorImpl, PseudoElement};
use selectors::Element;
use selectors::bloom::BloomFilter;
use selectors::matching::{AFFECTED_BY_STYLE_ATTRIBUTE, AFFECTED_BY_PRESENTATIONAL_HINTS};
use selectors::matching::{StyleRelations, matches_complex_selector};
use selectors::parser::{Selector, SimpleSelector, LocalName, ComplexSelector};
use sink::Push;
use smallvec::VecLike;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::fmt;
use std::hash::BuildHasherDefault;
use std::hash::Hash;
use std::slice;
use std::sync::Arc;
use string_cache::Atom;
use style_traits::viewport::ViewportConstraints;
use stylesheets::{CSSRule, CSSRuleIteratorExt, Origin, Stylesheet};
use viewport::{MaybeNew, ViewportRuleCascade};

pub type FnvHashMap<K, V> = HashMap<K, V, BuildHasherDefault<::fnv::FnvHasher>>;

/// This structure holds all the selectors and device characteristics
/// for a given document. The selectors are converted into `Rule`s
/// (defined in rust-selectors), and introduced in a `SelectorMap`
/// depending on the pseudo-element (see `PerPseudoElementSelectorMap`),
/// stylesheet origin (see `PerOriginSelectorMap`), and priority
/// (see the `normal` and `important` fields in `PerOriginSelectorMap`).
///
/// This structure is effectively created once per pipeline, in the
/// LayoutThread corresponding to that pipeline.
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct Stylist {
    /// Device that the stylist is currently evaluating against.
    pub device: Device,

    /// Viewport constraints based on the current device.
    viewport_constraints: Option<ViewportConstraints>,

    /// If true, the quirks-mode stylesheet is applied.
    quirks_mode: bool,

    /// If true, the device has changed, and the stylist needs to be updated.
    is_device_dirty: bool,

    /// The current selector maps, after evaluating media
    /// rules against the current device.
    element_map: PerPseudoElementSelectorMap,

    /// The selector maps corresponding to a given pseudo-element
    /// (depending on the implementation)
    pseudos_map: FnvHashMap<PseudoElement, PerPseudoElementSelectorMap>,

    /// A map with all the animations indexed by name.
    animations: FnvHashMap<Atom, KeyframesAnimation>,

    /// Applicable declarations for a given non-eagerly cascaded pseudo-element.
    /// These are eagerly computed once, and then used to resolve the new
    /// computed values on the fly on layout.
    precomputed_pseudo_element_decls: FnvHashMap<PseudoElement, Vec<DeclarationBlock>>,

    rules_source_order: usize,

    /// Selector dependencies used to compute restyle hints.
    state_deps: DependencySet,

    /// Selectors in the page affecting siblings
    sibling_affecting_selectors: Vec<Selector<TheSelectorImpl>>,

    /// Selectors in the page matching elements with non-common style-affecting
    /// attributes.
    non_common_style_affecting_attributes_selectors: Vec<Selector<TheSelectorImpl>>,
}

impl Stylist {
    #[inline]
    pub fn new(device: Device) -> Self {
        let mut stylist = Stylist {
            viewport_constraints: None,
            device: device,
            is_device_dirty: true,
            quirks_mode: false,

            element_map: PerPseudoElementSelectorMap::new(),
            pseudos_map: Default::default(),
            animations: Default::default(),
            precomputed_pseudo_element_decls: Default::default(),
            rules_source_order: 0,
            state_deps: DependencySet::new(),

            // XXX remember resetting them!
            sibling_affecting_selectors: vec![],
            non_common_style_affecting_attributes_selectors: vec![]
        };

        TheSelectorImpl::each_eagerly_cascaded_pseudo_element(|pseudo| {
            stylist.pseudos_map.insert(pseudo, PerPseudoElementSelectorMap::new());
        });

        // FIXME: Add iso-8859-9.css when the document’s encoding is ISO-8859-8.

        stylist
    }

    pub fn update(&mut self, doc_stylesheets: &[Arc<Stylesheet>], stylesheets_changed: bool) -> bool {
        if !(self.is_device_dirty || stylesheets_changed) {
            return false;
        }

        self.element_map = PerPseudoElementSelectorMap::new();
        self.pseudos_map = Default::default();
        self.animations = Default::default();
        TheSelectorImpl::each_eagerly_cascaded_pseudo_element(|pseudo| {
            self.pseudos_map.insert(pseudo, PerPseudoElementSelectorMap::new());
        });

        self.precomputed_pseudo_element_decls = Default::default();
        self.rules_source_order = 0;
        self.state_deps.clear();

        self.sibling_affecting_selectors.clear();
        self.non_common_style_affecting_attributes_selectors.clear();

        for ref stylesheet in TheSelectorImpl::get_user_or_user_agent_stylesheets().iter() {
            self.add_stylesheet(&stylesheet);
        }

        if self.quirks_mode {
            if let Some(s) = TheSelectorImpl::get_quirks_mode_stylesheet() {
                self.add_stylesheet(s);
            }
        }

        for ref stylesheet in doc_stylesheets.iter() {
            self.add_stylesheet(stylesheet);
        }

        self.is_device_dirty = false;
        true
    }

    fn add_stylesheet(&mut self, stylesheet: &Stylesheet) {
        if !stylesheet.is_effective_for_device(&self.device) {
            return;
        }
        let mut rules_source_order = self.rules_source_order;

        // Take apart the StyleRule into individual Rules and insert
        // them into the SelectorMap of that priority.
        macro_rules! append(
            ($style_rule: ident, $priority: ident, $importance: expr, $count: ident) => {
                if $style_rule.declarations.$count > 0 {
                    for selector in &$style_rule.selectors {
                        let map = if let Some(ref pseudo) = selector.pseudo_element {
                            self.pseudos_map
                                .entry(pseudo.clone())
                                .or_insert_with(PerPseudoElementSelectorMap::new)
                                .borrow_for_origin(&stylesheet.origin)
                        } else {
                            self.element_map.borrow_for_origin(&stylesheet.origin)
                        };

                        map.$priority.insert(Rule {
                            selector: selector.complex_selector.clone(),
                            declarations: DeclarationBlock {
                                specificity: selector.specificity,
                                mixed_declarations: $style_rule.declarations.declarations.clone(),
                                importance: $importance,
                                source_order: rules_source_order,
                            },
                        });
                    }
                }
            };
        );

        for rule in stylesheet.effective_rules(&self.device) {
            match *rule {
                CSSRule::Style(ref style_rule) => {
                    append!(style_rule, normal, Importance::Normal, normal_count);
                    append!(style_rule, important, Importance::Important, important_count);
                    rules_source_order += 1;

                    for selector in &style_rule.selectors {
                        self.state_deps.note_selector(&selector.complex_selector);
                        if selector.affects_siblings() {
                            self.sibling_affecting_selectors.push(selector.clone());
                        }

                        if selector.matches_non_common_style_affecting_attribute() {
                            self.non_common_style_affecting_attributes_selectors.push(selector.clone());
                        }
                    }

                    self.rules_source_order = rules_source_order;
                }
                CSSRule::Keyframes(ref keyframes_rule) => {
                    debug!("Found valid keyframes rule: {:?}", keyframes_rule);
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
        }

        debug!("Stylist stats:");
        debug!(" - Got {} sibling-affecting selectors",
               self.sibling_affecting_selectors.len());
        debug!(" - Got {} non-common-style-attribute-affecting selectors",
               self.non_common_style_affecting_attributes_selectors.len());
        debug!(" - Got {} deps for style-hint calculation",
               self.state_deps.len());

        TheSelectorImpl::each_precomputed_pseudo_element(|pseudo| {
            // TODO: Consider not doing this and just getting the rules on the
            // fly. It should be a bit slower, but we'd take rid of the
            // extra field, and avoid this precomputation entirely.
            if let Some(map) = self.pseudos_map.remove(&pseudo) {
                let mut declarations = vec![];

                map.user_agent.normal.get_universal_rules(&mut declarations);
                map.user_agent.important.get_universal_rules(&mut declarations);

                self.precomputed_pseudo_element_decls.insert(pseudo, declarations);
            }
        })
    }

    /// Computes the style for a given "precomputed" pseudo-element, taking the
    /// universal rules and applying them.
    pub fn precomputed_values_for_pseudo(&self,
                                         pseudo: &PseudoElement,
                                         parent: Option<&Arc<ComputedValues>>)
                                         -> Option<Arc<ComputedValues>> {
        debug_assert!(TheSelectorImpl::pseudo_element_cascade_type(pseudo).is_precomputed());
        if let Some(declarations) = self.precomputed_pseudo_element_decls.get(pseudo) {
            let (computed, _) =
                properties::cascade(self.device.au_viewport_size(),
                                    &declarations, false,
                                    parent.map(|p| &**p),
                                    None,
                                    None,
                                    Box::new(StdoutErrorReporter));
            Some(Arc::new(computed))
        } else {
            parent.map(|p| p.clone())
        }
    }

    pub fn lazily_compute_pseudo_element_style<E>(&self,
                                                  element: &E,
                                                  pseudo: &PseudoElement,
                                                  parent: &Arc<ComputedValues>)
                                                  -> Option<Arc<ComputedValues>>
        where E: Element<Impl=TheSelectorImpl> +
              fmt::Debug +
              PresentationalHintsSynthetizer
    {
        debug_assert!(TheSelectorImpl::pseudo_element_cascade_type(pseudo).is_lazy());
        if self.pseudos_map.get(pseudo).is_none() {
            return None;
        }

        let mut declarations = vec![];

        // NB: This being cached could be worth it, maybe allow an optional
        // ApplicableDeclarationsCache?.
        self.push_applicable_declarations(element,
                                          None,
                                          None,
                                          Some(pseudo),
                                          &mut declarations);

        let (computed, _) =
            properties::cascade(self.device.au_viewport_size(),
                                &declarations, false,
                                Some(&**parent), None, None,
                                Box::new(StdoutErrorReporter));


        Some(Arc::new(computed))
    }

    pub fn set_device(&mut self, mut device: Device, stylesheets: &[Arc<Stylesheet>]) {
        let cascaded_rule = stylesheets.iter()
            .flat_map(|s| s.effective_rules(&self.device).viewport())
            .cascade();

        self.viewport_constraints = ViewportConstraints::maybe_new(device.viewport_size, &cascaded_rule);
        if let Some(ref constraints) = self.viewport_constraints {
            device = Device::new(MediaType::Screen, constraints.size);
        }

        self.is_device_dirty |= stylesheets.iter().any(|stylesheet| {
                stylesheet.rules().media().any(|media_rule|
                    media_rule.evaluate(&self.device) != media_rule.evaluate(&device))
        });

        self.device = device;
    }

    pub fn viewport_constraints(&self) -> &Option<ViewportConstraints> {
        &self.viewport_constraints
    }

    pub fn set_quirks_mode(&mut self, enabled: bool) {
        self.quirks_mode = enabled;
    }

    /// Returns the applicable CSS declarations for the given element.
    /// This corresponds to `ElementRuleCollector` in WebKit.
    ///
    /// The returned boolean indicates whether the style is *shareable*;
    /// that is, whether the matched selectors are simple enough to allow the
    /// matching logic to be reduced to the logic in
    /// `css::matching::PrivateMatchMethods::candidate_element_allows_for_style_sharing`.
    pub fn push_applicable_declarations<E, V>(
                                        &self,
                                        element: &E,
                                        parent_bf: Option<&BloomFilter>,
                                        style_attribute: Option<&PropertyDeclarationBlock>,
                                        pseudo_element: Option<&PseudoElement>,
                                        applicable_declarations: &mut V) -> StyleRelations
        where E: Element<Impl=TheSelectorImpl> +
                 fmt::Debug +
                 PresentationalHintsSynthetizer,
              V: Push<DeclarationBlock> + VecLike<DeclarationBlock>
    {
        assert!(!self.is_device_dirty);
        assert!(style_attribute.is_none() || pseudo_element.is_none(),
                "Style attributes do not apply to pseudo-elements");
        debug_assert!(pseudo_element.is_none() ||
                      !TheSelectorImpl::pseudo_element_cascade_type(pseudo_element.as_ref().unwrap())
                        .is_precomputed());

        let map = match pseudo_element {
            Some(ref pseudo) => self.pseudos_map.get(pseudo).unwrap(),
            None => &self.element_map,
        };

        let mut relations = StyleRelations::empty();

        debug!("Determining if style is shareable: pseudo: {}", pseudo_element.is_some());
        // Step 1: Normal user-agent rules.
        map.user_agent.normal.get_all_matching_rules(element,
                                                     parent_bf,
                                                     applicable_declarations,
                                                     &mut relations);
        debug!("UA normal: {:?}", relations);

        // Step 2: Presentational hints.
        let length = applicable_declarations.len();
        element.synthesize_presentational_hints_for_legacy_attributes(applicable_declarations);
        if applicable_declarations.len() != length {
            // Never share style for elements with preshints
            relations |= AFFECTED_BY_PRESENTATIONAL_HINTS;
        }
        debug!("preshints: {:?}", relations);

        // Step 3: User and author normal rules.
        map.user.normal.get_all_matching_rules(element,
                                               parent_bf,
                                               applicable_declarations,
                                               &mut relations);
        debug!("user normal: {:?}", relations);
        map.author.normal.get_all_matching_rules(element,
                                                 parent_bf,
                                                 applicable_declarations,
                                                 &mut relations);
        debug!("author normal: {:?}", relations);

        // Step 4: Normal style attributes.
        if let Some(ref sa)  = style_attribute {
            if sa.normal_count > 0 {
                relations |= AFFECTED_BY_STYLE_ATTRIBUTE;
                Push::push(
                    applicable_declarations,
                    DeclarationBlock::from_declarations(
                        sa.declarations.clone(),
                        Importance::Normal));
            }
        }

        debug!("style attr: {:?}", relations);

        // Step 5: Author-supplied `!important` rules.
        map.author.important.get_all_matching_rules(element,
                                                    parent_bf,
                                                    applicable_declarations,
                                                    &mut relations);

        debug!("author important: {:?}", relations);

        // Step 6: `!important` style attributes.
        if let Some(ref sa) = style_attribute {
            if sa.important_count > 0 {
                relations |= AFFECTED_BY_STYLE_ATTRIBUTE;
                Push::push(
                    applicable_declarations,
                    DeclarationBlock::from_declarations(
                        sa.declarations.clone(),
                        Importance::Important));
            }
        }

        debug!("style attr important: {:?}", relations);

        // Step 7: User and UA `!important` rules.
        map.user.important.get_all_matching_rules(element,
                                                  parent_bf,
                                                  applicable_declarations,
                                                  &mut relations);

        debug!("user important: {:?}", relations);

        map.user_agent.important.get_all_matching_rules(element,
                                                        parent_bf,
                                                        applicable_declarations,
                                                        &mut relations);

        debug!("UA important: {:?}", relations);

        debug!("push_applicable_declarations: shareable: {:?}", relations);

        relations
    }

    #[inline]
    pub fn is_device_dirty(&self) -> bool {
        self.is_device_dirty
    }

    #[inline]
    pub fn animations(&self) -> &FnvHashMap<Atom, KeyframesAnimation> {
        &self.animations
    }

    pub fn match_same_not_common_style_affecting_attributes_rules<E>(&self,
                                                                     element: &E,
                                                                     candidate: &E) -> bool
    where E: ElementExt
    {
        use selectors::matching::StyleRelations;
        use selectors::matching::matches_complex_selector;
        // XXX we can probably do better, the candidate should already know what
        // rules it matches.
        //
        // XXX Could the bloom filter help here? Should be available.
        for ref selector in self.non_common_style_affecting_attributes_selectors.iter() {
            let element_matches = matches_complex_selector(&selector.complex_selector,
                                                           element,
                                                           None,
                                                           &mut StyleRelations::empty());
            let candidate_matches = matches_complex_selector(&selector.complex_selector,
                                                             candidate,
                                                             None,
                                                             &mut StyleRelations::empty());

            if element_matches != candidate_matches {
                return false;
            }
        }

        true
    }

    pub fn match_same_sibling_affecting_rules<E>(&self,
                                                 element: &E,
                                                 candidate: &E) -> bool
    where E: ElementExt
    {
        use selectors::matching::StyleRelations;
        use selectors::matching::matches_complex_selector;
        // XXX we can probably do better, the candidate should already know what
        // rules it matches.
        //
        // XXX The bloom filter would help here, and should be available.
        for ref selector in self.sibling_affecting_selectors.iter() {
            let element_matches = matches_complex_selector(&selector.complex_selector,
                                                           element,
                                                           None,
                                                           &mut StyleRelations::empty());

            let candidate_matches = matches_complex_selector(&selector.complex_selector,
                                                             candidate,
                                                             None,
                                                             &mut StyleRelations::empty());

            if element_matches != candidate_matches {
                debug!("match_same_sibling_affecting_rules: Failure due to {:?}",
                       selector.complex_selector);
                return false;
            }
        }

        true
    }

    pub fn compute_restyle_hint<E>(&self, element: &E,
                                   snapshot: &E::Snapshot,
                                   // NB: We need to pass current_state as an argument because
                                   // selectors::Element doesn't provide access to ElementState
                                   // directly, and computing it from the ElementState would be
                                   // more expensive than getting it directly from the caller.
                                   current_state: ElementState)
                                   -> RestyleHint
        where E: ElementExt + Clone
    {
        self.state_deps.compute_hint(element, snapshot, current_state)
    }
}


/// Map that contains the CSS rules for a given origin.
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
struct PerOriginSelectorMap {
    /// Rules that contains at least one property declaration with
    /// normal importance.
    normal: SelectorMap,
    /// Rules that contains at least one property declaration with
    /// !important.
    important: SelectorMap,
}

impl PerOriginSelectorMap {
    #[inline]
    fn new() -> Self {
        PerOriginSelectorMap {
            normal: SelectorMap::new(),
            important: SelectorMap::new(),
        }
    }
}

/// Map that contains the CSS rules for a specific PseudoElement
/// (or lack of PseudoElement).
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
struct PerPseudoElementSelectorMap {
    /// Rules from user agent stylesheets
    user_agent: PerOriginSelectorMap,
    /// Rules from author stylesheets
    author: PerOriginSelectorMap,
    /// Rules from user stylesheets
    user: PerOriginSelectorMap,
}

impl PerPseudoElementSelectorMap {
    #[inline]
    fn new() -> Self {
        PerPseudoElementSelectorMap {
            user_agent: PerOriginSelectorMap::new(),
            author: PerOriginSelectorMap::new(),
            user: PerOriginSelectorMap::new(),
        }
    }

    #[inline]
    fn borrow_for_origin(&mut self, origin: &Origin) -> &mut PerOriginSelectorMap {
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
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct SelectorMap {
    // TODO: Tune the initial capacity of the HashMap
    pub id_hash: FnvHashMap<Atom, Vec<Rule>>,
    pub class_hash: FnvHashMap<Atom, Vec<Rule>>,
    pub local_name_hash: FnvHashMap<Atom, Vec<Rule>>,
    /// Same as local_name_hash, but keys are lower-cased.
    /// For HTML elements in HTML documents.
    pub lower_local_name_hash: FnvHashMap<Atom, Vec<Rule>>,
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
                                        relations: &mut StyleRelations)
        where E: Element<Impl=TheSelectorImpl>,
              V: VecLike<DeclarationBlock>
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
                                                      relations)
        }

        element.each_class(|class| {
            SelectorMap::get_matching_rules_from_hash(element,
                                                      parent_bf,
                                                      &self.class_hash,
                                                      class,
                                                      matching_rules_list,
                                                      relations);
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
                                                  relations);

        SelectorMap::get_matching_rules(element,
                                        parent_bf,
                                        &self.other_rules,
                                        matching_rules_list,
                                        relations);

        // Sort only the rules we just added.
        sort_by_key(&mut matching_rules_list[init_len..],
                    |rule| (rule.specificity, rule.source_order));
    }

    /// Append to `rule_list` all universal Rules (rules with selector `*|*`) in
    /// `self` sorted by specifity and source order.
    pub fn get_universal_rules<V>(&self,
                                  matching_rules_list: &mut V)
        where V: VecLike<DeclarationBlock>
    {
        if self.empty {
            return
        }

        let init_len = matching_rules_list.len();

        for rule in self.other_rules.iter() {
            if rule.selector.compound_selector.is_empty() &&
               rule.selector.next.is_none() {
                matching_rules_list.push(rule.declarations.clone());
            }
        }

        sort_by_key(&mut matching_rules_list[init_len..],
                    |rule| (rule.specificity, rule.source_order));
    }

    fn get_matching_rules_from_hash<E, Str, BorrowedStr: ?Sized, Vector>(
        element: &E,
        parent_bf: Option<&BloomFilter>,
        hash: &FnvHashMap<Str, Vec<Rule>>,
        key: &BorrowedStr,
        matching_rules: &mut Vector,
        relations: &mut StyleRelations)
        where E: Element<Impl=TheSelectorImpl>,
              Str: Borrow<BorrowedStr> + Eq + Hash,
              BorrowedStr: Eq + Hash,
              Vector: VecLike<DeclarationBlock>
    {
        if let Some(rules) = hash.get(key) {
            SelectorMap::get_matching_rules(element,
                                            parent_bf,
                                            rules,
                                            matching_rules,
                                            relations)
        }
    }

    /// Adds rules in `rules` that match `element` to the `matching_rules` list.
    fn get_matching_rules<E, V>(element: &E,
                                parent_bf: Option<&BloomFilter>,
                                rules: &[Rule],
                                matching_rules: &mut V,
                                relations: &mut StyleRelations)
        where E: Element<Impl=TheSelectorImpl>,
              V: VecLike<DeclarationBlock>
    {
        for rule in rules.iter() {
            if matches_complex_selector(&*rule.selector,
                                         element, parent_bf, relations) {
                matching_rules.push(rule.declarations.clone());
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

        if let Some(LocalName { name, lower_name }) = SelectorMap::get_local_name(&rule) {
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
    pub fn get_local_name(rule: &Rule) -> Option<LocalName<TheSelectorImpl>> {
        for ss in &rule.selector.compound_selector {
            if let SimpleSelector::LocalName(ref n) = *ss {
                return Some(LocalName {
                    name: n.name.clone(),
                    lower_name: n.lower_name.clone(),
                })
            }
        }

        None
    }
}

#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[derive(Clone)]
pub struct Rule {
    // This is an Arc because Rule will essentially be cloned for every element
    // that it matches. Selector contains an owned vector (through
    // ComplexSelector) and we want to avoid the allocation.
    pub selector: Arc<ComplexSelector<TheSelectorImpl>>,
    pub declarations: DeclarationBlock,
}

/// A property declaration together with its precedence among rules of equal specificity so that
/// we can sort them.
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[derive(Debug, Clone)]
pub struct DeclarationBlock {
    /// Contains declarations of either importance, but only those of self.importance are relevant.
    /// Use DeclarationBlock::iter
    pub mixed_declarations: Arc<Vec<(PropertyDeclaration, Importance)>>,
    pub importance: Importance,
    pub source_order: usize,
    pub specificity: u32,
}

impl DeclarationBlock {
    #[inline]
    pub fn from_declarations(declarations: Arc<Vec<(PropertyDeclaration, Importance)>>,
                             importance: Importance)
                             -> Self {
        DeclarationBlock {
            mixed_declarations: declarations,
            importance: importance,
            source_order: 0,
            specificity: 0,
        }
    }

    pub fn iter(&self) -> DeclarationBlockIter {
        DeclarationBlockIter {
            iter: self.mixed_declarations.iter(),
            importance: self.importance,
        }
    }
}

pub struct DeclarationBlockIter<'a> {
    iter: slice::Iter<'a, (PropertyDeclaration, Importance)>,
    importance: Importance,
}

impl<'a> Iterator for DeclarationBlockIter<'a> {
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

impl<'a> DoubleEndedIterator for DeclarationBlockIter<'a> {
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

fn find_push<Str: Eq + Hash>(map: &mut FnvHashMap<Str, Vec<Rule>>, key: Str, value: Rule) {
    map.entry(key).or_insert_with(Vec::new).push(value)
}
