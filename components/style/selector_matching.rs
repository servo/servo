/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Selector matching.

use dom::PresentationalHintsSynthetizer;
use element_state::*;
use error_reporting::StdoutErrorReporter;
use keyframes::KeyframesAnimation;
use media_queries::{Device, MediaType};
use properties::{self, PropertyDeclaration, PropertyDeclarationBlock, ComputedValues};
use restyle_hints::{RestyleHint, DependencySet};
use selector_impl::{ElementExt, SelectorImplExt, TheSelectorImpl, PseudoElement, AttrString};
use selectors::Element;
use selectors::bloom::BloomFilter;
use selectors::matching::DeclarationBlock as GenericDeclarationBlock;
use selectors::matching::{Rule, SelectorMap};
use sink::Push;
use smallvec::VecLike;
use std::collections::HashMap;
use std::hash::BuildHasherDefault;
use std::sync::Arc;
use string_cache::Atom;
use style_traits::viewport::ViewportConstraints;
use stylesheets::{CSSRule, CSSRuleIteratorExt, Origin, Stylesheet};
use viewport::{MaybeNew, ViewportRuleCascade};


pub type DeclarationBlock = GenericDeclarationBlock<Vec<PropertyDeclaration>>;

/// This structure holds all the selectors and device characteristics
/// for a given document. The selectors are converted into `Rule`s
/// (defined in rust-selectors), and introduced in a `SelectorMap`
/// depending on the pseudo-element (see `PerPseudoElementSelectorMap`),
/// stylesheet origin (see `PerOriginSelectorMap`), and priority
/// (see the `normal` and `important` fields in `PerOriginSelectorMap`).
///
/// This structure is effectively created once per pipeline, in the
/// LayoutThread corresponding to that pipeline.
///
/// The stylist is parameterized on `SelectorImplExt`, a trait that extends
/// `selectors::parser::SelectorImpl`, and that allows to customise what
/// pseudo-classes and pseudo-elements are parsed. This is actually either
/// `ServoSelectorImpl`, the implementation used by Servo's layout system in
/// regular builds, or `GeckoSelectorImpl`, the implementation used in the
/// geckolib port.
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
    pseudos_map: HashMap<PseudoElement,
                         PerPseudoElementSelectorMap,
                         BuildHasherDefault<::fnv::FnvHasher>>,

    /// A map with all the animations indexed by name.
    animations: HashMap<Atom, KeyframesAnimation>,

    /// Applicable declarations for a given non-eagerly cascaded pseudo-element.
    /// These are eagerly computed once, and then used to resolve the new
    /// computed values on the fly on layout.
    precomputed_pseudo_element_decls: HashMap<PseudoElement,
                                              Vec<DeclarationBlock>,
                                              BuildHasherDefault<::fnv::FnvHasher>>,

    rules_source_order: usize,

    /// Selector dependencies used to compute restyle hints.
    state_deps: DependencySet,
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
            pseudos_map: HashMap::with_hasher(Default::default()),
            animations: HashMap::with_hasher(Default::default()),
            precomputed_pseudo_element_decls: HashMap::with_hasher(Default::default()),
            rules_source_order: 0,
            state_deps: DependencySet::new(),
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
        self.pseudos_map = HashMap::with_hasher(Default::default());
        self.animations = HashMap::with_hasher(Default::default());
        TheSelectorImpl::each_eagerly_cascaded_pseudo_element(|pseudo| {
            self.pseudos_map.insert(pseudo, PerPseudoElementSelectorMap::new());
        });

        self.precomputed_pseudo_element_decls = HashMap::with_hasher(Default::default());
        self.rules_source_order = 0;
        self.state_deps.clear();

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
            ($style_rule: ident, $priority: ident) => {
                if !$style_rule.declarations.$priority.is_empty() {
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
                            selector: selector.compound_selectors.clone(),
                            declarations: DeclarationBlock {
                                specificity: selector.specificity,
                                declarations: $style_rule.declarations.$priority.clone(),
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
                    append!(style_rule, normal);
                    append!(style_rule, important);
                    rules_source_order += 1;
                    for selector in &style_rule.selectors {
                        self.state_deps.note_selector(selector.compound_selectors.clone());
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
                                                  where E: Element<Impl=TheSelectorImpl, AttrString=AttrString> +
                                                        PresentationalHintsSynthetizer {
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
                                Some(&**parent), None,
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
                                        applicable_declarations: &mut V)
                                        -> bool
                                        where E: Element<Impl=TheSelectorImpl, AttrString=AttrString> +
                                                 PresentationalHintsSynthetizer,
                                              V: Push<DeclarationBlock> + VecLike<DeclarationBlock> {
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

        let mut shareable = true;

        // Step 1: Normal user-agent rules.
        map.user_agent.normal.get_all_matching_rules(element,
                                                     parent_bf,
                                                     applicable_declarations,
                                                     &mut shareable);

        // Step 2: Presentational hints.
        let length = applicable_declarations.len();
        element.synthesize_presentational_hints_for_legacy_attributes(applicable_declarations);
        if applicable_declarations.len() != length {
            // Never share style for elements with preshints
            shareable = false;
        }

        // Step 3: User and author normal rules.
        map.user.normal.get_all_matching_rules(element,
                                               parent_bf,
                                               applicable_declarations,
                                               &mut shareable);
        map.author.normal.get_all_matching_rules(element,
                                                 parent_bf,
                                                 applicable_declarations,
                                                 &mut shareable);

        // Step 4: Normal style attributes.
        style_attribute.map(|sa| {
            shareable = false;
            Push::push(
                applicable_declarations,
                GenericDeclarationBlock::from_declarations(sa.normal.clone()))
        });

        // Step 5: Author-supplied `!important` rules.
        map.author.important.get_all_matching_rules(element,
                                                    parent_bf,
                                                    applicable_declarations,
                                                    &mut shareable);

        // Step 6: `!important` style attributes.
        style_attribute.map(|sa| {
            shareable = false;
            Push::push(
                applicable_declarations,
                GenericDeclarationBlock::from_declarations(sa.important.clone()))
        });

        // Step 7: User and UA `!important` rules.
        map.user.important.get_all_matching_rules(element,
                                                  parent_bf,
                                                  applicable_declarations,
                                                  &mut shareable);
        map.user_agent.important.get_all_matching_rules(element,
                                                        parent_bf,
                                                        applicable_declarations,
                                                        &mut shareable);

        shareable
    }

    #[inline]
    pub fn is_device_dirty(&self) -> bool {
        self.is_device_dirty
    }

    #[inline]
    pub fn animations(&self) -> &HashMap<Atom, KeyframesAnimation> {
        &self.animations
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
    /// Rules that contains at least one property declararion with
    /// normal importance.
    normal: SelectorMap<Vec<PropertyDeclaration>, TheSelectorImpl>,
    /// Rules that contains at least one property declararion with
    /// !important.
    important: SelectorMap<Vec<PropertyDeclaration>, TheSelectorImpl>,
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
