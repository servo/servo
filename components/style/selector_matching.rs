/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use media_queries::{Device, MediaType};
use node::TElementAttributes;
use properties::{PropertyDeclaration, PropertyDeclarationBlock};
use restyle_hints::{ElementSnapshot, RestyleHint, DependencySet};
use selectors::Element;
use selectors::bloom::BloomFilter;
use selectors::matching::DeclarationBlock as GenericDeclarationBlock;
use selectors::matching::{Rule, SelectorMap};
use selectors::parser::PseudoElement;
use selectors::states::*;
use smallvec::VecLike;
use std::process;
use style_traits::viewport::ViewportConstraints;
use stylesheets::{CSSRuleIteratorExt, Origin, Stylesheet};
use url::Url;
use util::opts;
use util::resource_files::read_resource_file;
use viewport::{MaybeNew, ViewportRuleCascade};


pub type DeclarationBlock = GenericDeclarationBlock<Vec<PropertyDeclaration>>;


lazy_static! {
    pub static ref USER_OR_USER_AGENT_STYLESHEETS: Vec<Stylesheet> = {
        let mut stylesheets = vec!();
        // FIXME: presentational-hints.css should be at author origin with zero specificity.
        //        (Does it make a difference?)
        for &filename in &["user-agent.css", "servo.css", "presentational-hints.css"] {
            match read_resource_file(&[filename]) {
                Ok(res) => {
                    let ua_stylesheet = Stylesheet::from_bytes(
                        &res,
                        Url::parse(&format!("chrome:///{:?}", filename)).unwrap(),
                        None,
                        None,
                        Origin::UserAgent);
                    stylesheets.push(ua_stylesheet);
                }
                Err(..) => {
                    error!("Failed to load UA stylesheet {}!", filename);
                    process::exit(1);
                }
            }
        }
        for &(ref contents, ref url) in &opts::get().user_stylesheets {
            stylesheets.push(Stylesheet::from_bytes(
                &contents, url.clone(), None, None, Origin::User));
        }
        stylesheets
    };
}

lazy_static! {
    pub static ref QUIRKS_MODE_STYLESHEET: Stylesheet = {
        match read_resource_file(&["quirks-mode.css"]) {
            Ok(res) => {
                Stylesheet::from_bytes(
                    &res,
                    url!("chrome:///quirks-mode.css"),
                    None,
                    None,
                    Origin::UserAgent)
            },
            Err(..) => {
                error!("Stylist failed to load 'quirks-mode.css'!");
                process::exit(1);
            }
        }
    };
}

pub struct Stylist {
    // Device that the stylist is currently evaluating against.
    pub device: Device,

    // Viewport constraints based on the current device.
    viewport_constraints: Option<ViewportConstraints>,

    // If true, the quirks-mode stylesheet is applied.
    quirks_mode: bool,

    // If true, the device has changed, and the stylist needs to be updated.
    is_device_dirty: bool,

    // The current selector maps, after evaluating media
    // rules against the current device.
    element_map: PerPseudoElementSelectorMap,
    before_map: PerPseudoElementSelectorMap,
    after_map: PerPseudoElementSelectorMap,
    rules_source_order: usize,

    // Selector dependencies used to compute restyle hints.
    state_deps: DependencySet,
}

impl Stylist {
    #[inline]
    pub fn new(device: Device) -> Stylist {
        let stylist = Stylist {
            viewport_constraints: None,
            device: device,
            is_device_dirty: true,
            quirks_mode: false,

            element_map: PerPseudoElementSelectorMap::new(),
            before_map: PerPseudoElementSelectorMap::new(),
            after_map: PerPseudoElementSelectorMap::new(),
            rules_source_order: 0,
            state_deps: DependencySet::new(),
        };
        // FIXME: Add iso-8859-9.css when the document’s encoding is ISO-8859-8.
        stylist
    }

    pub fn update(&mut self, doc_stylesheets: &[&Stylesheet],
                      stylesheets_changed: bool) -> bool {
        if !(self.is_device_dirty || stylesheets_changed) {
            return false;
        }
        self.element_map = PerPseudoElementSelectorMap::new();
        self.before_map = PerPseudoElementSelectorMap::new();
        self.after_map = PerPseudoElementSelectorMap::new();
        self.rules_source_order = 0;
        self.state_deps.clear();

        for ref stylesheet in USER_OR_USER_AGENT_STYLESHEETS.iter() {
            self.add_stylesheet(&stylesheet);
        }

        if self.quirks_mode {
            self.add_stylesheet(&QUIRKS_MODE_STYLESHEET);
        }

        for ref stylesheet in doc_stylesheets.iter() {
            self.add_stylesheet(stylesheet);
        }

        self.is_device_dirty = false;
        true
    }

    fn add_stylesheet(&mut self, stylesheet: &Stylesheet) {
        let device = &self.device;
        if !stylesheet.is_effective_for_device(device) {
            return;
        }
        let (mut element_map, mut before_map, mut after_map) = match stylesheet.origin {
            Origin::UserAgent => (
                &mut self.element_map.user_agent,
                &mut self.before_map.user_agent,
                &mut self.after_map.user_agent,
            ),
            Origin::Author => (
                &mut self.element_map.author,
                &mut self.before_map.author,
                &mut self.after_map.author,
            ),
            Origin::User => (
                &mut self.element_map.user,
                &mut self.before_map.user,
                &mut self.after_map.user,
            ),
        };
        let mut rules_source_order = self.rules_source_order;

        // Take apart the StyleRule into individual Rules and insert
        // them into the SelectorMap of that priority.
        macro_rules! append(
            ($style_rule: ident, $priority: ident) => {
                if $style_rule.declarations.$priority.len() > 0 {
                    for selector in &$style_rule.selectors {
                        let map = match selector.pseudo_element {
                            None => &mut element_map,
                            Some(PseudoElement::Before) => &mut before_map,
                            Some(PseudoElement::After) => &mut after_map,
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

        for style_rule in stylesheet.effective_rules(&self.device).style() {
            append!(style_rule, normal);
            append!(style_rule, important);
            rules_source_order += 1;
            for selector in &style_rule.selectors {
                self.state_deps.note_selector(selector.compound_selectors.clone());
            }
        }
        self.rules_source_order = rules_source_order;
    }

    pub fn compute_restyle_hint<E>(&self, element: &E,
                                   snapshot: &ElementSnapshot,
                                   // NB: We need to pass current_state as an argument because
                                   // selectors::Element doesn't provide access to ElementState
                                   // directly, and computing it from the ElementState would be
                                   // more expensive than getting it directly from the caller.
                                   current_state: ElementState)
                                   -> RestyleHint
                                   where E: Element + Clone {
        self.state_deps.compute_hint(element, snapshot, current_state)
    }

    pub fn set_device(&mut self, mut device: Device, stylesheets: &[&Stylesheet]) {
        let cascaded_rule = stylesheets.iter()
            .flat_map(|s| s.effective_rules(&self.device).viewport())
            .cascade();

        self.viewport_constraints = ViewportConstraints::maybe_new(self.device.viewport_size, &cascaded_rule);
        if let Some(ref constraints) = self.viewport_constraints {
            device = Device::new(MediaType::Screen, constraints.size);
        }
        let is_device_dirty = self.is_device_dirty || stylesheets.iter()
            .flat_map(|stylesheet| stylesheet.rules().media())
            .any(|media_rule| media_rule.evaluate(&self.device) != media_rule.evaluate(&device));

        self.device = device;
        self.is_device_dirty |= is_device_dirty;
    }

    pub fn viewport_constraints(&self) -> &Option<ViewportConstraints> {
        &self.viewport_constraints
    }

    pub fn set_quirks_mode(&mut self, enabled: bool) {
        self.quirks_mode = enabled;
    }

    /// Returns the applicable CSS declarations for the given element. This corresponds to
    /// `ElementRuleCollector` in WebKit.
    ///
    /// The returned boolean indicates whether the style is *shareable*; that is, whether the
    /// matched selectors are simple enough to allow the matching logic to be reduced to the logic
    /// in `css::matching::PrivateMatchMethods::candidate_element_allows_for_style_sharing`.
    pub fn push_applicable_declarations<E, V>(
                                        &self,
                                        element: &E,
                                        parent_bf: Option<&BloomFilter>,
                                        style_attribute: Option<&PropertyDeclarationBlock>,
                                        pseudo_element: Option<PseudoElement>,
                                        applicable_declarations: &mut V)
                                        -> bool
                                        where E: Element + TElementAttributes,
                                              V: VecLike<DeclarationBlock> {
        assert!(!self.is_device_dirty);
        assert!(style_attribute.is_none() || pseudo_element.is_none(),
                "Style attributes do not apply to pseudo-elements");

        let map = match pseudo_element {
            None => &self.element_map,
            Some(PseudoElement::Before) => &self.before_map,
            Some(PseudoElement::After) => &self.after_map,
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
            applicable_declarations.push(
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
            applicable_declarations.push(
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

    pub fn is_device_dirty(&self) -> bool {
        self.is_device_dirty
    }
}

struct PerOriginSelectorMap {
    normal: SelectorMap<Vec<PropertyDeclaration>>,
    important: SelectorMap<Vec<PropertyDeclaration>>,
}

impl PerOriginSelectorMap {
    #[inline]
    fn new() -> PerOriginSelectorMap {
        PerOriginSelectorMap {
            normal: SelectorMap::new(),
            important: SelectorMap::new(),
        }
    }
}

struct PerPseudoElementSelectorMap {
    user_agent: PerOriginSelectorMap,
    author: PerOriginSelectorMap,
    user: PerOriginSelectorMap,
}

impl PerPseudoElementSelectorMap {
    #[inline]
    fn new() -> PerPseudoElementSelectorMap {
        PerPseudoElementSelectorMap {
            user_agent: PerOriginSelectorMap::new(),
            author: PerOriginSelectorMap::new(),
            user: PerOriginSelectorMap::new(),
        }
    }
}
