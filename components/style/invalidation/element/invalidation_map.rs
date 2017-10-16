/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Code for invalidations due to state or attribute changes.

use {Atom, LocalName, Namespace};
use context::QuirksMode;
use element_state::ElementState;
use fallible::FallibleVec;
use hashglobe::FailedAllocationError;
use selector_map::{MaybeCaseInsensitiveHashMap, SelectorMap, SelectorMapEntry};
use selector_parser::SelectorImpl;
use selectors::attr::NamespaceConstraint;
use selectors::parser::{Combinator, Component};
use selectors::parser::{Selector, SelectorIter, SelectorMethods};
use selectors::visitor::SelectorVisitor;
use smallvec::SmallVec;

#[cfg(feature = "gecko")]
/// Gets the element state relevant to the given `:dir` pseudo-class selector.
pub fn dir_selector_to_state(s: &[u16]) -> ElementState {
    use element_state::{IN_LTR_STATE, IN_RTL_STATE};

    // Jump through some hoops to deal with our Box<[u16]> thing.
    const LTR: [u16; 4] = [b'l' as u16, b't' as u16, b'r' as u16, 0];
    const RTL: [u16; 4] = [b'r' as u16, b't' as u16, b'l' as u16, 0];

    if LTR == *s {
        IN_LTR_STATE
    } else if RTL == *s {
        IN_RTL_STATE
    } else {
        // :dir(something-random) is a valid selector, but shouldn't
        // match anything.
        ElementState::empty()
    }
}

/// Mapping between (partial) CompoundSelectors (and the combinator to their
/// right) and the states and attributes they depend on.
///
/// In general, for all selectors in all applicable stylesheets of the form:
///
/// |a _ b _ c _ d _ e|
///
/// Where:
///   * |b| and |d| are simple selectors that depend on state (like :hover) or
///     attributes (like [attr...], .foo, or #foo).
///   * |a|, |c|, and |e| are arbitrary simple selectors that do not depend on
///     state or attributes.
///
/// We generate a Dependency for both |a _ b:X _| and |a _ b:X _ c _ d:Y _|,
/// even though those selectors may not appear on their own in any stylesheet.
/// This allows us to quickly scan through the dependency sites of all style
/// rules and determine the maximum effect that a given state or attribute
/// change may have on the style of elements in the document.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "gecko", derive(MallocSizeOf))]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct Dependency {
    /// The dependency selector.
    #[cfg_attr(feature = "gecko",
               ignore_malloc_size_of = "CssRules have primary refs, we measure there")]
    #[cfg_attr(feature = "servo", ignore_heap_size_of = "Arc")]
    pub selector: Selector<SelectorImpl>,

    /// The offset into the selector that we should match on.
    pub selector_offset: usize,
}

impl Dependency {
    /// Returns the combinator to the right of the partial selector this
    /// dependency represents.
    ///
    /// TODO(emilio): Consider storing inline if it helps cache locality?
    pub fn combinator(&self) -> Option<Combinator> {
        if self.selector_offset == 0 {
            return None;
        }

        Some(self.selector.combinator_at_match_order(self.selector_offset - 1))
    }

    /// Whether this dependency affects the style of the element.
    ///
    /// NOTE(emilio): pseudo-elements need to be here to account for eager
    /// pseudos, since they just grab the style from the originating element.
    ///
    /// TODO(emilio): We could look at the selector itself to see if it's an
    /// eager pseudo, and return false here if not.
    pub fn affects_self(&self) -> bool {
        matches!(self.combinator(), None | Some(Combinator::PseudoElement))
    }

    /// Whether this dependency may affect style of any of our descendants.
    pub fn affects_descendants(&self) -> bool {
        matches!(self.combinator(), Some(Combinator::PseudoElement) |
                                    Some(Combinator::Child) |
                                    Some(Combinator::Descendant))
    }

    /// Whether this dependency may affect style of any of our later siblings.
    pub fn affects_later_siblings(&self) -> bool {
        matches!(self.combinator(), Some(Combinator::NextSibling) |
                                    Some(Combinator::LaterSibling))
    }
}

impl SelectorMapEntry for Dependency {
    fn selector(&self) -> SelectorIter<SelectorImpl> {
        self.selector.iter_from(self.selector_offset)
    }
}

/// The same, but for state selectors, which can track more exactly what state
/// do they track.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "gecko", derive(MallocSizeOf))]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct StateDependency {
    /// The other dependency fields.
    pub dep: Dependency,
    /// The state this dependency is affected by.
    pub state: ElementState,
}

impl SelectorMapEntry for StateDependency {
    fn selector(&self) -> SelectorIter<SelectorImpl> {
        self.dep.selector()
    }
}

/// A map where we store invalidations.
///
/// This is slightly different to a SelectorMap, in the sense of that the same
/// selector may appear multiple times.
///
/// In particular, we want to lookup as few things as possible to get the fewer
/// selectors the better, so this looks up by id, class, or looks at the list of
/// state/other attribute affecting selectors.
#[derive(Debug)]
#[cfg_attr(feature = "gecko", derive(MallocSizeOf))]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct InvalidationMap {
    /// A map from a given class name to all the selectors with that class
    /// selector.
    pub class_to_selector: MaybeCaseInsensitiveHashMap<Atom, SmallVec<[Dependency; 1]>>,
    /// A map from a given id to all the selectors with that ID in the
    /// stylesheets currently applying to the document.
    pub id_to_selector: MaybeCaseInsensitiveHashMap<Atom, SmallVec<[Dependency; 1]>>,
    /// A map of all the state dependencies.
    pub state_affecting_selectors: SelectorMap<StateDependency>,
    /// A map of other attribute affecting selectors.
    pub other_attribute_affecting_selectors: SelectorMap<Dependency>,
    /// Whether there are attribute rules of the form `[class~="foo"]` that may
    /// match. In that case, we need to look at
    /// `other_attribute_affecting_selectors` too even if only the `class` has
    /// changed.
    pub has_class_attribute_selectors: bool,
    /// Whether there are attribute rules of the form `[id|="foo"]` that may
    /// match. In that case, we need to look at
    /// `other_attribute_affecting_selectors` too even if only the `id` has
    /// changed.
    pub has_id_attribute_selectors: bool,
}

impl InvalidationMap {
    /// Creates an empty `InvalidationMap`.
    pub fn new() -> Self {
        Self {
            class_to_selector: MaybeCaseInsensitiveHashMap::new(),
            id_to_selector: MaybeCaseInsensitiveHashMap::new(),
            state_affecting_selectors: SelectorMap::new(),
            other_attribute_affecting_selectors: SelectorMap::new(),
            has_class_attribute_selectors: false,
            has_id_attribute_selectors: false,
        }
    }

    /// Adds a selector to this `InvalidationMap`.  Returns Err(..) to
    /// signify OOM.
    pub fn note_selector(
        &mut self,
        selector: &Selector<SelectorImpl>,
        quirks_mode: QuirksMode
    ) -> Result<(), FailedAllocationError> {
        self.collect_invalidations_for(selector, quirks_mode)
    }

    /// Clears this map, leaving it empty.
    pub fn clear(&mut self) {
        self.class_to_selector.clear();
        self.id_to_selector.clear();
        self.state_affecting_selectors.clear();
        self.other_attribute_affecting_selectors.clear();
        self.has_id_attribute_selectors = false;
        self.has_class_attribute_selectors = false;
    }

    // Returns Err(..) to signify OOM.
    fn collect_invalidations_for(
        &mut self,
        selector: &Selector<SelectorImpl>,
        quirks_mode: QuirksMode
    ) -> Result<(), FailedAllocationError> {
        debug!("InvalidationMap::collect_invalidations_for({:?})", selector);

        let mut iter = selector.iter();
        let mut combinator;
        let mut index = 0;

        loop {
            let sequence_start = index;

            let mut compound_visitor = CompoundSelectorDependencyCollector {
                classes: SmallVec::new(),
                ids: SmallVec::new(),
                state: ElementState::empty(),
                other_attributes: false,
                has_id_attribute_selectors: false,
                has_class_attribute_selectors: false,
            };

            // Visit all the simple selectors in this sequence.
            //
            // Note that this works because we can't have combinators nested
            // inside simple selectors (i.e. in :not() or :-moz-any()).
            //
            // If we ever support that we'll need to visit nested complex
            // selectors as well, in order to mark them as affecting descendants
            // at least.
            for ss in &mut iter {
                ss.visit(&mut compound_visitor);
                index += 1; // Account for the simple selector.
            }

            self.has_id_attribute_selectors |= compound_visitor.has_id_attribute_selectors;
            self.has_class_attribute_selectors |= compound_visitor.has_class_attribute_selectors;

            for class in compound_visitor.classes {
                self.class_to_selector
                    .try_get_or_insert_with(class, quirks_mode, SmallVec::new)?
                    .try_push(Dependency {
                        selector: selector.clone(),
                        selector_offset: sequence_start,
                    })?;
            }

            for id in compound_visitor.ids {
                self.id_to_selector
                    .try_get_or_insert_with(id, quirks_mode, SmallVec::new)?
                    .try_push(Dependency {
                        selector: selector.clone(),
                        selector_offset: sequence_start,
                    })?;
            }

            if !compound_visitor.state.is_empty() {
                self.state_affecting_selectors
                    .insert(StateDependency {
                        dep: Dependency {
                            selector: selector.clone(),
                            selector_offset: sequence_start,
                        },
                        state: compound_visitor.state,
                    }, quirks_mode)?;
            }

            if compound_visitor.other_attributes {
                self.other_attribute_affecting_selectors
                    .insert(Dependency {
                        selector: selector.clone(),
                        selector_offset: sequence_start,
                    }, quirks_mode)?;
            }

            combinator = iter.next_sequence();
            if combinator.is_none() {
                break;
            }

            index += 1; // Account for the combinator.
        }

        Ok(())
    }

    /// Allows mutation of this InvalidationMap.
    pub fn begin_mutation(&mut self) {
        self.class_to_selector.begin_mutation();
        self.id_to_selector.begin_mutation();
        self.state_affecting_selectors.begin_mutation();
        self.other_attribute_affecting_selectors.begin_mutation();
    }

    /// Disallows mutation of this InvalidationMap.
    pub fn end_mutation(&mut self) {
        self.class_to_selector.end_mutation();
        self.id_to_selector.end_mutation();
        self.state_affecting_selectors.end_mutation();
        self.other_attribute_affecting_selectors.end_mutation();
    }
}

/// A struct that collects invalidations for a given compound selector.
struct CompoundSelectorDependencyCollector {
    /// The state this compound selector is affected by.
    state: ElementState,

    /// The classes this compound selector is affected by.
    ///
    /// NB: This will be often a single class, but could be multiple in
    /// presence of :not, :-moz-any, .foo.bar.baz, etc.
    classes: SmallVec<[Atom; 5]>,

    /// The IDs this compound selector is affected by.
    ///
    /// NB: This will be almost always a single id, but could be multiple in
    /// presence of :not, :-moz-any, #foo#bar, etc.
    ids: SmallVec<[Atom; 5]>,

    /// Whether it affects other attribute-dependent selectors that aren't ID or
    /// class selectors (NB: We still set this to true in presence of [class] or
    /// [id] attribute selectors).
    other_attributes: bool,

    /// Whether there were attribute selectors with the id attribute.
    has_id_attribute_selectors: bool,

    /// Whether there were attribute selectors with the class attribute.
    has_class_attribute_selectors: bool,
}

impl SelectorVisitor for CompoundSelectorDependencyCollector {
    type Impl = SelectorImpl;

    fn visit_simple_selector(&mut self, s: &Component<SelectorImpl>) -> bool {
        #[cfg(feature = "gecko")]
        use selector_parser::NonTSPseudoClass;

        match *s {
            Component::ID(ref id) => {
                self.ids.push(id.clone());
            }
            Component::Class(ref class) => {
                self.classes.push(class.clone());
            }
            Component::NonTSPseudoClass(ref pc) => {
                self.other_attributes |= pc.is_attr_based();
                self.state |= match *pc {
                    #[cfg(feature = "gecko")]
                    NonTSPseudoClass::Dir(ref s) => {
                        dir_selector_to_state(s)
                    }
                    _ => pc.state_flag(),
                };
            }
            _ => {}
        }

        true
    }

    fn visit_attribute_selector(
        &mut self,
        constraint: &NamespaceConstraint<&Namespace>,
        _local_name: &LocalName,
        local_name_lower: &LocalName,
    ) -> bool {
        self.other_attributes = true;
        let may_match_in_no_namespace = match *constraint {
            NamespaceConstraint::Any => true,
            NamespaceConstraint::Specific(ref ns) => ns.is_empty(),
        };

        if may_match_in_no_namespace {
            self.has_id_attribute_selectors |= *local_name_lower == local_name!("id");
            self.has_class_attribute_selectors |= *local_name_lower == local_name!("class");
        }

        true
    }
}
