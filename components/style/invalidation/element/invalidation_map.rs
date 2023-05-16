/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Code for invalidations due to state or attribute changes.

use crate::context::QuirksMode;
use crate::element_state::{DocumentState, ElementState};
use crate::selector_map::{
    MaybeCaseInsensitiveHashMap, PrecomputedHashMap, SelectorMap, SelectorMapEntry,
};
use crate::selector_parser::SelectorImpl;
use crate::{Atom, LocalName, Namespace};
use fallible::FallibleVec;
use hashglobe::FailedAllocationError;
use selectors::attr::NamespaceConstraint;
use selectors::parser::{Combinator, Component};
use selectors::parser::{Selector, SelectorIter};
use selectors::visitor::SelectorVisitor;
use smallvec::SmallVec;

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
#[derive(Clone, Debug, MallocSizeOf)]
pub struct Dependency {
    /// The dependency selector.
    #[cfg_attr(
        feature = "gecko",
        ignore_malloc_size_of = "CssRules have primary refs, we measure there"
    )]
    #[cfg_attr(feature = "servo", ignore_malloc_size_of = "Arc")]
    pub selector: Selector<SelectorImpl>,

    /// The offset into the selector that we should match on.
    pub selector_offset: usize,

    /// The parent dependency for an ancestor selector. For example, consider
    /// the following:
    ///
    ///     .foo .bar:where(.baz span) .qux
    ///         ^               ^     ^
    ///         A               B     C
    ///
    ///  We'd generate:
    ///
    ///    * One dependency for .qux (offset: 0, parent: None)
    ///    * One dependency for .baz pointing to B with parent being a
    ///      dependency pointing to C.
    ///    * One dependency from .bar pointing to C (parent: None)
    ///    * One dependency from .foo pointing to A (parent: None)
    ///
    pub parent: Option<Box<Dependency>>,
}

/// The kind of elements down the tree this dependency may affect.
#[derive(Debug, Eq, PartialEq)]
pub enum DependencyInvalidationKind {
    /// This dependency may affect the element that changed itself.
    Element,
    /// This dependency affects the style of the element itself, and also the
    /// style of its descendants.
    ///
    /// TODO(emilio): Each time this feels more of a hack for eager pseudos...
    ElementAndDescendants,
    /// This dependency may affect descendants down the tree.
    Descendants,
    /// This dependency may affect siblings to the right of the element that
    /// changed.
    Siblings,
    /// This dependency may affect slotted elements of the element that changed.
    SlottedElements,
    /// This dependency may affect parts of the element that changed.
    Parts,
}

impl Dependency {
    /// Creates a dummy dependency to invalidate the whole selector.
    ///
    /// This is necessary because document state invalidation wants to
    /// invalidate all elements in the document.
    ///
    /// The offset is such as that Invalidation::new(self) returns a zero
    /// offset. That is, it points to a virtual "combinator" outside of the
    /// selector, so calling combinator() on such a dependency will panic.
    pub fn for_full_selector_invalidation(selector: Selector<SelectorImpl>) -> Self {
        Self {
            selector_offset: selector.len() + 1,
            selector,
            parent: None,
        }
    }

    /// Returns the combinator to the right of the partial selector this
    /// dependency represents.
    ///
    /// TODO(emilio): Consider storing inline if it helps cache locality?
    pub fn combinator(&self) -> Option<Combinator> {
        if self.selector_offset == 0 {
            return None;
        }

        Some(
            self.selector
                .combinator_at_match_order(self.selector_offset - 1),
        )
    }

    /// The kind of invalidation that this would generate.
    pub fn invalidation_kind(&self) -> DependencyInvalidationKind {
        match self.combinator() {
            None => DependencyInvalidationKind::Element,
            Some(Combinator::Child) | Some(Combinator::Descendant) => {
                DependencyInvalidationKind::Descendants
            },
            Some(Combinator::LaterSibling) | Some(Combinator::NextSibling) => {
                DependencyInvalidationKind::Siblings
            },
            // TODO(emilio): We could look at the selector itself to see if it's
            // an eager pseudo, and return only Descendants here if not.
            Some(Combinator::PseudoElement) => DependencyInvalidationKind::ElementAndDescendants,
            Some(Combinator::SlotAssignment) => DependencyInvalidationKind::SlottedElements,
            Some(Combinator::Part) => DependencyInvalidationKind::Parts,
        }
    }
}

impl SelectorMapEntry for Dependency {
    fn selector(&self) -> SelectorIter<SelectorImpl> {
        self.selector.iter_from(self.selector_offset)
    }
}

/// The same, but for state selectors, which can track more exactly what state
/// do they track.
#[derive(Clone, Debug, MallocSizeOf)]
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

/// The same, but for document state selectors.
#[derive(Clone, Debug, MallocSizeOf)]
pub struct DocumentStateDependency {
    /// We track `Dependency` even though we don't need to track an offset,
    /// since when it changes it changes for the whole document anyway.
    #[cfg_attr(
        feature = "gecko",
        ignore_malloc_size_of = "CssRules have primary refs, we measure there"
    )]
    #[cfg_attr(feature = "servo", ignore_malloc_size_of = "Arc")]
    pub dependency: Dependency,
    /// The state this dependency is affected by.
    pub state: DocumentState,
}

/// A map where we store invalidations.
///
/// This is slightly different to a SelectorMap, in the sense of that the same
/// selector may appear multiple times.
///
/// In particular, we want to lookup as few things as possible to get the fewer
/// selectors the better, so this looks up by id, class, or looks at the list of
/// state/other attribute affecting selectors.
#[derive(Clone, Debug, MallocSizeOf)]
pub struct InvalidationMap {
    /// A map from a given class name to all the selectors with that class
    /// selector.
    pub class_to_selector: MaybeCaseInsensitiveHashMap<Atom, SmallVec<[Dependency; 1]>>,
    /// A map from a given id to all the selectors with that ID in the
    /// stylesheets currently applying to the document.
    pub id_to_selector: MaybeCaseInsensitiveHashMap<Atom, SmallVec<[Dependency; 1]>>,
    /// A map of all the state dependencies.
    pub state_affecting_selectors: SelectorMap<StateDependency>,
    /// A list of document state dependencies in the rules we represent.
    pub document_state_selectors: Vec<DocumentStateDependency>,
    /// A map of other attribute affecting selectors.
    pub other_attribute_affecting_selectors:
        PrecomputedHashMap<LocalName, SmallVec<[Dependency; 1]>>,
}

impl InvalidationMap {
    /// Creates an empty `InvalidationMap`.
    pub fn new() -> Self {
        Self {
            class_to_selector: MaybeCaseInsensitiveHashMap::new(),
            id_to_selector: MaybeCaseInsensitiveHashMap::new(),
            state_affecting_selectors: SelectorMap::new(),
            document_state_selectors: Vec::new(),
            other_attribute_affecting_selectors: PrecomputedHashMap::default(),
        }
    }

    /// Returns the number of dependencies stored in the invalidation map.
    pub fn len(&self) -> usize {
        self.state_affecting_selectors.len() +
            self.document_state_selectors.len() +
            self.other_attribute_affecting_selectors
                .iter()
                .fold(0, |accum, (_, ref v)| accum + v.len()) +
            self.id_to_selector
                .iter()
                .fold(0, |accum, (_, ref v)| accum + v.len()) +
            self.class_to_selector
                .iter()
                .fold(0, |accum, (_, ref v)| accum + v.len())
    }

    /// Clears this map, leaving it empty.
    pub fn clear(&mut self) {
        self.class_to_selector.clear();
        self.id_to_selector.clear();
        self.state_affecting_selectors.clear();
        self.document_state_selectors.clear();
        self.other_attribute_affecting_selectors.clear();
    }

    /// Adds a selector to this `InvalidationMap`.  Returns Err(..) to
    /// signify OOM.
    pub fn note_selector(
        &mut self,
        selector: &Selector<SelectorImpl>,
        quirks_mode: QuirksMode,
    ) -> Result<(), FailedAllocationError> {
        debug!("InvalidationMap::note_selector({:?})", selector);

        let mut document_state = DocumentState::empty();

        {
            let mut parent_stack = SmallVec::new();
            let mut alloc_error = None;
            let mut collector = SelectorDependencyCollector {
                map: self,
                document_state: &mut document_state,
                selector,
                parent_selectors: &mut parent_stack,
                quirks_mode,
                compound_state: PerCompoundState::new(0),
                alloc_error: &mut alloc_error,
            };

            let visit_result = collector.visit_whole_selector();
            debug_assert_eq!(!visit_result, alloc_error.is_some());
            if let Some(alloc_error) = alloc_error {
                return Err(alloc_error);
            }
        }

        if !document_state.is_empty() {
            let dep = DocumentStateDependency {
                state: document_state,
                dependency: Dependency::for_full_selector_invalidation(selector.clone()),
            };
            self.document_state_selectors.try_push(dep)?;
        }

        Ok(())
    }
}

struct PerCompoundState {
    /// The offset at which our compound starts.
    offset: usize,

    /// The state this compound selector is affected by.
    element_state: ElementState,
}

impl PerCompoundState {
    fn new(offset: usize) -> Self {
        Self {
            offset,
            element_state: ElementState::empty(),
        }
    }
}

/// A struct that collects invalidations for a given compound selector.
struct SelectorDependencyCollector<'a> {
    map: &'a mut InvalidationMap,

    /// The document this _complex_ selector is affected by.
    ///
    /// We don't need to track state per compound selector, since it's global
    /// state and it changes for everything.
    document_state: &'a mut DocumentState,

    /// The current selector and offset we're iterating.
    selector: &'a Selector<SelectorImpl>,

    /// The stack of parent selectors that we have, and at which offset of the
    /// sequence.
    ///
    /// This starts empty. It grows when we find nested :is and :where selector
    /// lists.
    parent_selectors: &'a mut SmallVec<[(Selector<SelectorImpl>, usize); 5]>,

    /// The quirks mode of the document where we're inserting dependencies.
    quirks_mode: QuirksMode,

    /// State relevant to a given compound selector.
    compound_state: PerCompoundState,

    /// The allocation error, if we OOM.
    alloc_error: &'a mut Option<FailedAllocationError>,
}

impl<'a> SelectorDependencyCollector<'a> {
    fn visit_whole_selector(&mut self) -> bool {
        let iter = self.selector.iter();
        self.visit_whole_selector_from(iter, 0)
    }

    fn visit_whole_selector_from(
        &mut self,
        mut iter: SelectorIter<SelectorImpl>,
        mut index: usize,
    ) -> bool {
        loop {
            // Reset the compound state.
            self.compound_state = PerCompoundState::new(index);

            // Visit all the simple selectors in this sequence.
            for ss in &mut iter {
                if !ss.visit(self) {
                    return false;
                }
                index += 1; // Account for the simple selector.
            }

            if !self.compound_state.element_state.is_empty() {
                let dependency = self.dependency();
                let result = self.map.state_affecting_selectors.insert(
                    StateDependency {
                        dep: dependency,
                        state: self.compound_state.element_state,
                    },
                    self.quirks_mode,
                );
                if let Err(alloc_error) = result {
                    *self.alloc_error = Some(alloc_error);
                    return false;
                }
            }

            let combinator = iter.next_sequence();
            if combinator.is_none() {
                return true;
            }
            index += 1; // account for the combinator
        }
    }

    fn add_attr_dependency(&mut self, name: LocalName) -> bool {
        let dependency = self.dependency();

        let map = &mut self.map.other_attribute_affecting_selectors;
        let entry = match map.try_entry(name) {
            Ok(entry) => entry,
            Err(err) => {
                *self.alloc_error = Some(err);
                return false;
            },
        };

        match entry.or_insert_with(SmallVec::new).try_push(dependency) {
            Ok(..) => true,
            Err(err) => {
                *self.alloc_error = Some(err);
                return false;
            },
        }
    }

    fn dependency(&self) -> Dependency {
        let mut parent = None;

        // TODO(emilio): Maybe we should refcount the parent dependencies, or
        // cache them or something.
        for &(ref selector, ref selector_offset) in self.parent_selectors.iter() {
            debug_assert_ne!(
                self.compound_state.offset, 0,
                "Shouldn't bother creating nested dependencies for the rightmost compound",
            );
            let new_parent = Dependency {
                selector: selector.clone(),
                selector_offset: *selector_offset,
                parent,
            };
            parent = Some(Box::new(new_parent));
        }

        Dependency {
            selector: self.selector.clone(),
            selector_offset: self.compound_state.offset,
            parent,
        }
    }
}

impl<'a> SelectorVisitor for SelectorDependencyCollector<'a> {
    type Impl = SelectorImpl;

    fn visit_selector_list(&mut self, list: &[Selector<SelectorImpl>]) -> bool {
        for selector in list {
            // Here we cheat a bit: We can visit the rightmost compound with
            // the "outer" visitor, and it'd be fine. This reduces the amount of
            // state and attribute invalidations, and we need to check the outer
            // selector to the left anyway to avoid over-invalidation, so it
            // avoids matching it twice uselessly.
            let mut iter = selector.iter();
            let mut index = 0;

            for ss in &mut iter {
                if !ss.visit(self) {
                    return false;
                }
                index += 1;
            }

            let combinator = iter.next_sequence();
            if combinator.is_none() {
                continue;
            }

            index += 1; // account for the combinator.

            self.parent_selectors
                .push((self.selector.clone(), self.compound_state.offset));
            let mut nested = SelectorDependencyCollector {
                map: &mut *self.map,
                document_state: &mut *self.document_state,
                selector,
                parent_selectors: &mut *self.parent_selectors,
                quirks_mode: self.quirks_mode,
                compound_state: PerCompoundState::new(index),
                alloc_error: &mut *self.alloc_error,
            };
            if !nested.visit_whole_selector_from(iter, index) {
                return false;
            }
            self.parent_selectors.pop();
        }
        true
    }

    fn visit_simple_selector(&mut self, s: &Component<SelectorImpl>) -> bool {
        use crate::selector_parser::NonTSPseudoClass;

        match *s {
            Component::ID(ref atom) | Component::Class(ref atom) => {
                let dependency = self.dependency();
                let map = match *s {
                    Component::ID(..) => &mut self.map.id_to_selector,
                    Component::Class(..) => &mut self.map.class_to_selector,
                    _ => unreachable!(),
                };
                let entry = match map.try_entry(atom.0.clone(), self.quirks_mode) {
                    Ok(entry) => entry,
                    Err(err) => {
                        *self.alloc_error = Some(err);
                        return false;
                    },
                };
                match entry.or_insert_with(SmallVec::new).try_push(dependency) {
                    Ok(..) => true,
                    Err(err) => {
                        *self.alloc_error = Some(err);
                        return false;
                    },
                }
            },
            Component::NonTSPseudoClass(ref pc) => {
                self.compound_state.element_state |= match *pc {
                    #[cfg(feature = "gecko")]
                    NonTSPseudoClass::Dir(ref dir) => dir.element_state(),
                    _ => pc.state_flag(),
                };
                *self.document_state |= pc.document_state_flag();

                let attr_name = match *pc {
                    #[cfg(feature = "gecko")]
                    NonTSPseudoClass::MozTableBorderNonzero => local_name!("border"),
                    #[cfg(feature = "gecko")]
                    NonTSPseudoClass::MozBrowserFrame => local_name!("mozbrowser"),
                    #[cfg(feature = "gecko")]
                    NonTSPseudoClass::MozSelectListBox => {
                        // This depends on two attributes.
                        return self.add_attr_dependency(local_name!("multiple")) &&
                            self.add_attr_dependency(local_name!("size"));
                    },
                    NonTSPseudoClass::Lang(..) => local_name!("lang"),
                    _ => return true,
                };

                self.add_attr_dependency(attr_name)
            },
            _ => true,
        }
    }

    fn visit_attribute_selector(
        &mut self,
        _: &NamespaceConstraint<&Namespace>,
        local_name: &LocalName,
        local_name_lower: &LocalName,
    ) -> bool {
        if !self.add_attr_dependency(local_name.clone()) {
            return false;
        }

        if local_name != local_name_lower && !self.add_attr_dependency(local_name_lower.clone()) {
            return false;
        }

        true
    }
}
