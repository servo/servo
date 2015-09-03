/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use url::Url;

use selectors::Element;
use selectors::bloom::BloomFilter;
use selectors::matching::DeclarationBlock as GenericDeclarationBlock;
use selectors::matching::{SelectorMap, Rule};
use selectors::parser::PseudoElement;
use smallvec::VecLike;
use std::process;
use util::opts;
use util::resource_files::read_resource_file;

use legacy::PresentationalHintSynthesis;
use media_queries::Device;
use node::TElementAttributes;
use properties::{PropertyDeclaration, PropertyDeclarationBlock};
use style_traits::viewport::ViewportConstraints;
use stylesheets::{Stylesheet, CSSRuleIteratorExt, Origin};
use viewport::{MaybeNew, ViewportRuleCascade};


pub type DeclarationBlock = GenericDeclarationBlock<Vec<PropertyDeclaration>>;


pub struct Stylist {
    // List of stylesheets (including all media rules)
    stylesheets: Vec<Stylesheet>,

    // Device that the stylist is currently evaluating against.
    pub device: Device,

    // If true, a stylesheet has been added or the device has
    // changed, and the stylist needs to be updated.
    is_dirty: bool,

    // The current selector maps, after evaluating media
    // rules against the current device.
    element_map: PerPseudoElementSelectorMap,
    before_map: PerPseudoElementSelectorMap,
    after_map: PerPseudoElementSelectorMap,
    rules_source_order: usize,
}

impl Stylist {
    #[inline]
    pub fn new(device: Device) -> Stylist {
        let mut stylist = Stylist {
            stylesheets: vec!(),
            device: device,
            is_dirty: true,

            element_map: PerPseudoElementSelectorMap::new(),
            before_map: PerPseudoElementSelectorMap::new(),
            after_map: PerPseudoElementSelectorMap::new(),
            rules_source_order: 0,
        };
        // FIXME: Add iso-8859-9.css when the document’s encoding is ISO-8859-8.
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
                    stylist.add_stylesheet(ua_stylesheet);
                }
                Err(..) => {
                    error!("Stylist::new() failed at loading {}!", filename);
                    process::exit(1);
                }
            }
        }
        for &(ref contents, ref url) in &opts::get().user_stylesheets {
            stylist.add_stylesheet(Stylesheet::from_bytes(
                &contents, url.clone(), None, None, Origin::User));
        }
        stylist
    }

    #[inline]
    pub fn stylesheets(&self) -> &[Stylesheet] {
        &self.stylesheets
    }

    pub fn constrain_viewport(&self) -> Option<ViewportConstraints> {
        let cascaded_rule = self.stylesheets.iter()
            .flat_map(|s| s.effective_rules(&self.device).viewport())
            .cascade();

        ViewportConstraints::maybe_new(self.device.viewport_size, &cascaded_rule)
    }

    pub fn update(&mut self) -> bool {
        if self.is_dirty {
            self.element_map = PerPseudoElementSelectorMap::new();
            self.before_map = PerPseudoElementSelectorMap::new();
            self.after_map = PerPseudoElementSelectorMap::new();
            self.rules_source_order = 0;

            for stylesheet in &self.stylesheets {
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
                }
                self.rules_source_order = rules_source_order;
            }

            self.is_dirty = false;
            return true;
        }

        false
    }

    pub fn set_device(&mut self, device: Device) {
        let is_dirty = self.is_dirty || self.stylesheets.iter()
            .flat_map(|stylesheet| stylesheet.rules().media())
            .any(|media_rule| media_rule.evaluate(&self.device) != media_rule.evaluate(&device));

        self.device = device;
        self.is_dirty |= is_dirty;
    }

    pub fn add_quirks_mode_stylesheet(&mut self) {
        match read_resource_file(&["quirks-mode.css"]) {
            Ok(res) => {
            self.add_stylesheet(Stylesheet::from_bytes(
                &res,
                Url::parse("chrome:///quirks-mode.css").unwrap(),
                None,
                None,
                Origin::UserAgent));
            }
            Err(..) => {
                error!("Stylist::add_quirks_mode_stylesheet() failed at loading 'quirks-mode.css'!");
                process::exit(1);
            }
        }
    }

    pub fn add_stylesheet(&mut self, stylesheet: Stylesheet) {
        self.stylesheets.push(stylesheet);
        self.is_dirty = true;
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
        assert!(!self.is_dirty);
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
        self.synthesize_presentational_hints_for_legacy_attributes(element,
                                                                   applicable_declarations,
                                                                   &mut shareable);

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

    pub fn is_dirty(&self) -> bool {
        self.is_dirty
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
