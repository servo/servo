/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// For lazy_static
#![allow(unsafe_code)]

use dom::TElement;
use element_state::*;
use error_reporting::{ParseErrorReporter, StdoutErrorReporter};
use media_queries::{Device, MediaType};
use properties::{PropertyDeclaration, PropertyDeclarationBlock};
use restyle_hints::{ElementSnapshot, RestyleHint, DependencySet};
use selector_impl::{SelectorImplExt, ServoSelectorImpl};
use selectors::Element;
use selectors::bloom::BloomFilter;
use selectors::matching::DeclarationBlock as GenericDeclarationBlock;
use selectors::matching::{Rule, SelectorMap};
use selectors::parser::SelectorImpl;
use smallvec::VecLike;
use std::collections::HashMap;
use std::hash::BuildHasherDefault;
use std::process;
use std::sync::Arc;
use style_traits::viewport::ViewportConstraints;
use stylesheets::{CSSRuleIteratorExt, Origin, Stylesheet};
use url::Url;
use util::opts;
use util::resource_files::read_resource_file;
use viewport::{MaybeNew, ViewportRuleCascade};


pub type DeclarationBlock = GenericDeclarationBlock<Vec<PropertyDeclaration>>;

lazy_static! {
    pub static ref USER_OR_USER_AGENT_STYLESHEETS: Vec<Stylesheet<ServoSelectorImpl>> = {
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
                        Origin::UserAgent,
                        box StdoutErrorReporter);
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
                &contents, url.clone(), None, None, Origin::User, box StdoutErrorReporter));
        }
        stylesheets
    };
}

lazy_static! {
    pub static ref QUIRKS_MODE_STYLESHEET: Stylesheet<ServoSelectorImpl> = {
        match read_resource_file(&["quirks-mode.css"]) {
            Ok(res) => {
                Stylesheet::from_bytes(
                    &res,
                    url!("chrome:///quirks-mode.css"),
                    None,
                    None,
                    Origin::UserAgent,
                    box StdoutErrorReporter)
            },
            Err(..) => {
                error!("Stylist failed to load 'quirks-mode.css'!");
                process::exit(1);
            }
        }
    };
}

pub struct Stylist<Impl: SelectorImplExt> {
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
    element_map: PerPseudoElementSelectorMap<Impl>,
    pseudos_map: HashMap<Impl::PseudoElement, PerPseudoElementSelectorMap<Impl>, BuildHasherDefault<::fnv::FnvHasher>>,
    rules_source_order: usize,

    // Selector dependencies used to compute restyle hints.
    state_deps: DependencySet<Impl>,
}

impl<Impl: SelectorImplExt> Stylist<Impl> {
    #[inline]
    pub fn new(device: Device) -> Stylist<Impl> {
        let mut stylist = Stylist {
            viewport_constraints: None,
            device: device,
            is_device_dirty: true,
            quirks_mode: false,

            element_map: PerPseudoElementSelectorMap::new(),
            pseudos_map: HashMap::with_hasher(Default::default()),
            rules_source_order: 0,
            state_deps: DependencySet::new(),
        };

        Impl::each_eagerly_cascaded_pseudo_element(|pseudo| {
            stylist.pseudos_map.insert(pseudo, PerPseudoElementSelectorMap::new());
        });

        // FIXME: Add iso-8859-9.css when the document’s encoding is ISO-8859-8.

        stylist
    }

    pub fn update(&mut self, doc_stylesheets: &[Arc<Stylesheet<Impl>>],
                  stylesheets_changed: bool) -> bool
                  where Impl: 'static {
        if !(self.is_device_dirty || stylesheets_changed) {
            return false;
        }
        self.element_map = PerPseudoElementSelectorMap::new();
        self.pseudos_map = HashMap::with_hasher(Default::default());
        self.rules_source_order = 0;
        self.state_deps.clear();

        for ref stylesheet in Impl::get_user_or_user_agent_stylesheets().iter() {
            self.add_stylesheet(&stylesheet);
        }

        if self.quirks_mode {
            if let Some(s) = Impl::get_quirks_mode_stylesheet() {
                self.add_stylesheet(s);
            }
        }

        for ref stylesheet in doc_stylesheets.iter() {
            self.add_stylesheet(stylesheet);
        }

        self.is_device_dirty = false;
        true
    }

    fn add_stylesheet(&mut self, stylesheet: &Stylesheet<Impl>) {
        let device = &self.device;
        if !stylesheet.is_effective_for_device(device) {
            return;
        }

        let mut rules_source_order = self.rules_source_order;

        // Take apart the StyleRule into individual Rules and insert
        // them into the SelectorMap of that priority.
        macro_rules! append(
            ($style_rule: ident, $priority: ident) => {
                if $style_rule.declarations.$priority.len() > 0 {
                    for selector in &$style_rule.selectors {
                        let map = if let Some(ref pseudo) = selector.pseudo_element {
                            self.pseudos_map.entry(pseudo.clone())
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
                                   where E: Element<Impl=Impl> + Clone {
        self.state_deps.compute_hint(element, snapshot, current_state)
    }

    pub fn set_device(&mut self, mut device: Device, stylesheets: &[Arc<Stylesheet<Impl>>]) {
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
                                        pseudo_element: Option<Impl::PseudoElement>,
                                        applicable_declarations: &mut V)
                                        -> bool
                                        where E: Element<Impl=Impl> + TElement,
                                              V: VecLike<DeclarationBlock> {
        assert!(!self.is_device_dirty);
        assert!(style_attribute.is_none() || pseudo_element.is_none(),
                "Style attributes do not apply to pseudo-elements");

        let map = match pseudo_element {
            Some(ref pseudo) => match self.pseudos_map.get(pseudo) {
                Some(map) => map,
                // TODO(ecoal95): get non eagerly-cascaded pseudo-element rules here.
                // Actually assume there are no rules applicable.
                None => return true,
            },
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

struct PerOriginSelectorMap<Impl: SelectorImpl> {
    normal: SelectorMap<Vec<PropertyDeclaration>, Impl>,
    important: SelectorMap<Vec<PropertyDeclaration>, Impl>,
}

impl<Impl: SelectorImpl> PerOriginSelectorMap<Impl> {
    #[inline]
    fn new() -> PerOriginSelectorMap<Impl> {
        PerOriginSelectorMap {
            normal: SelectorMap::new(),
            important: SelectorMap::new(),
        }
    }
}

struct PerPseudoElementSelectorMap<Impl: SelectorImpl> {
    user_agent: PerOriginSelectorMap<Impl>,
    author: PerOriginSelectorMap<Impl>,
    user: PerOriginSelectorMap<Impl>,
}

impl<Impl: SelectorImpl> PerPseudoElementSelectorMap<Impl> {
    #[inline]
    fn new() -> PerPseudoElementSelectorMap<Impl> {
        PerPseudoElementSelectorMap {
            user_agent: PerOriginSelectorMap::new(),
            author: PerOriginSelectorMap::new(),
            user: PerOriginSelectorMap::new(),
        }
    }

    #[inline]
    fn borrow_for_origin(&mut self, origin: &Origin) -> &mut PerOriginSelectorMap<Impl> {
        match *origin {
            Origin::UserAgent => &mut self.user_agent,
            Origin::Author => &mut self.author,
            Origin::User => &mut self.user,
        }
    }
}
