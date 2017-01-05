/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Per-node data used in style calculation.

#![deny(missing_docs)]

use dom::TElement;
use properties::ComputedValues;
use properties::longhands::display::computed_value as display;
use restyle_hints::{RESTYLE_LATER_SIBLINGS, RestyleHint};
use rule_tree::StrongRuleNode;
use selector_parser::{PseudoElement, RestyleDamage, Snapshot};
use std::collections::HashMap;
use std::fmt;
use std::hash::BuildHasherDefault;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use stylist::Stylist;
use thread_state;

/// The structure that represents the result of style computation. This is
/// effectively a tuple of rules and computed values, that is, the rule node,
/// and the result of computing that rule node's rules, the `ComputedValues`.
#[derive(Clone)]
pub struct ComputedStyle {
    /// The rule node representing the ordered list of rules matched for this
    /// node.
    pub rules: StrongRuleNode,

    /// The computed values for each property obtained by cascading the
    /// matched rules.
    pub values: Arc<ComputedValues>,
}

impl ComputedStyle {
    /// Trivially construct a new `ComputedStyle`.
    pub fn new(rules: StrongRuleNode, values: Arc<ComputedValues>) -> Self {
        ComputedStyle {
            rules: rules,
            values: values,
        }
    }
}

// We manually implement Debug for ComputedStyle so tht we can avoid the verbose
// stringification of ComputedValues for normal logging.
impl fmt::Debug for ComputedStyle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ComputedStyle {{ rules: {:?}, values: {{..}} }}", self.rules)
    }
}

type PseudoStylesInner = HashMap<PseudoElement, ComputedStyle,
                                 BuildHasherDefault<::fnv::FnvHasher>>;

/// A set of styles for a given element's pseudo-elements.
///
/// This is a map from pseudo-element to `ComputedStyle`.
///
/// TODO(emilio): This should probably be a small array by default instead of a
/// full-blown `HashMap`.
#[derive(Clone, Debug)]
pub struct PseudoStyles(PseudoStylesInner);

impl PseudoStyles {
    /// Construct an empty set of `PseudoStyles`.
    pub fn empty() -> Self {
        PseudoStyles(HashMap::with_hasher(Default::default()))
    }
}

impl Deref for PseudoStyles {
    type Target = PseudoStylesInner;
    fn deref(&self) -> &Self::Target { &self.0 }
}

impl DerefMut for PseudoStyles {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

/// The styles associated with a node, including the styles for any
/// pseudo-elements.
#[derive(Clone, Debug)]
pub struct ElementStyles {
    /// The element's style.
    pub primary: ComputedStyle,
    /// The map of styles for the element's pseudos.
    pub pseudos: PseudoStyles,
}

impl ElementStyles {
    /// Trivially construct a new `ElementStyles`.
    pub fn new(primary: ComputedStyle) -> Self {
        ElementStyles {
            primary: primary,
            pseudos: PseudoStyles::empty(),
        }
    }

    /// Whether this element `display` value is `none`.
    pub fn is_display_none(&self) -> bool {
        self.primary.values.get_box().clone_display() == display::T::none
    }
}

/// Enum to describe the different requirements that a restyle hint may impose
/// on its descendants.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DescendantRestyleHint {
    /// This hint does not require any descendants to be restyled.
    Empty,
    /// This hint requires direct children to be restyled.
    Children,
    /// This hint requires all descendants to be restyled.
    Descendants,
}

impl DescendantRestyleHint {
    /// Propagates this descendant behavior to a child element.
    fn propagate(self) -> Self {
        use self::DescendantRestyleHint::*;
        if self == Descendants {
            Descendants
        } else {
            Empty
        }
    }

    fn union(self, other: Self) -> Self {
        use self::DescendantRestyleHint::*;
        if self == Descendants || other == Descendants {
            Descendants
        } else if self == Children || other == Children {
            Children
        } else {
            Empty
        }
    }
}

/// Restyle hint for storing on ElementData. We use a separate representation
/// to provide more type safety while propagating restyle hints down the tree.
#[derive(Clone, Debug)]
pub struct StoredRestyleHint {
    /// Whether this element should be restyled during the traversal.
    pub restyle_self: bool,
    /// Whether the descendants of this element need to be restyled.
    pub descendants: DescendantRestyleHint,
}

impl StoredRestyleHint {
    /// Propagates this restyle hint to a child element.
    pub fn propagate(&self) -> Self {
        StoredRestyleHint {
            restyle_self: self.descendants != DescendantRestyleHint::Empty,
            descendants: self.descendants.propagate(),
        }
    }

    /// Creates an empty `StoredRestyleHint`.
    pub fn empty() -> Self {
        StoredRestyleHint {
            restyle_self: false,
            descendants: DescendantRestyleHint::Empty,
        }
    }

    /// Creates a restyle hint that forces the whole subtree to be restyled,
    /// including the element.
    pub fn subtree() -> Self {
        StoredRestyleHint {
            restyle_self: true,
            descendants: DescendantRestyleHint::Descendants,
        }
    }

    /// Whether the restyle hint is empty (nothing requires to be restyled).
    pub fn is_empty(&self) -> bool {
        !self.restyle_self && self.descendants == DescendantRestyleHint::Empty
    }

    /// Insert another restyle hint, effectively resulting in the union of both.
    pub fn insert(&mut self, other: &Self) {
        self.restyle_self = self.restyle_self || other.restyle_self;
        self.descendants = self.descendants.union(other.descendants);
    }
}

impl Default for StoredRestyleHint {
    fn default() -> Self {
        StoredRestyleHint {
            restyle_self: false,
            descendants: DescendantRestyleHint::Empty,
        }
    }
}

impl From<RestyleHint> for StoredRestyleHint {
    fn from(hint: RestyleHint) -> Self {
        use restyle_hints::*;
        use self::DescendantRestyleHint::*;
        debug_assert!(!hint.contains(RESTYLE_LATER_SIBLINGS), "Caller should apply sibling hints");
        StoredRestyleHint {
            restyle_self: hint.contains(RESTYLE_SELF),
            descendants: if hint.contains(RESTYLE_DESCENDANTS) { Descendants } else { Empty },
        }
    }
}

static NO_SNAPSHOT: Option<Snapshot> = None;

/// We really want to store an Option<Snapshot> here, but we can't drop Gecko
/// Snapshots off-main-thread. So we make a convenient little wrapper to provide
/// the semantics of Option<Snapshot>, while deferring the actual drop.
#[derive(Debug)]
pub struct SnapshotOption {
    snapshot: Option<Snapshot>,
    destroyed: bool,
}

impl SnapshotOption {
    /// An empty snapshot.
    pub fn empty() -> Self {
        SnapshotOption {
            snapshot: None,
            destroyed: false,
        }
    }

    /// Destroy this snapshot.
    pub fn destroy(&mut self) {
        self.destroyed = true;
        debug_assert!(self.is_none());
    }

    /// Ensure a snapshot is available and return a mutable reference to it.
    pub fn ensure<F: FnOnce() -> Snapshot>(&mut self, create: F) -> &mut Snapshot {
        debug_assert!(thread_state::get().is_layout());
        if self.is_none() {
            self.snapshot = Some(create());
            self.destroyed = false;
        }

        self.snapshot.as_mut().unwrap()
    }
}

impl Deref for SnapshotOption {
    type Target = Option<Snapshot>;
    fn deref(&self) -> &Option<Snapshot> {
        if self.destroyed {
            &NO_SNAPSHOT
        } else {
            &self.snapshot
        }
    }
}

/// Transient data used by the restyle algorithm. This structure is instantiated
/// either before or during restyle traversal, and is cleared at the end of node
/// processing.
#[derive(Debug)]
pub struct RestyleData {
    /// The restyle hint, which indicates whether selectors need to be rematched
    /// for this element, its children, and its descendants.
    pub hint: StoredRestyleHint,

    /// Whether we need to recascade.
    /// FIXME(bholley): This should eventually become more fine-grained.
    pub recascade: bool,

    /// The restyle damage, indicating what kind of layout changes are required
    /// afte restyling.
    pub damage: RestyleDamage,

    /// An optional snapshot of the original state and attributes of the element,
    /// from which we may compute additional restyle hints at traversal time.
    pub snapshot: SnapshotOption,
}

impl Default for RestyleData {
    fn default() -> Self {
        RestyleData {
            hint: StoredRestyleHint::default(),
            recascade: false,
            damage: RestyleDamage::empty(),
            snapshot: SnapshotOption::empty(),
        }
    }
}

impl RestyleData {
    /// Expands the snapshot (if any) into a restyle hint. Returns true if later
    /// siblings must be restyled.
    pub fn expand_snapshot<E: TElement>(&mut self, element: E, stylist: &Stylist) -> bool {
        if self.snapshot.is_none() {
            return false;
        }

        // Compute the hint.
        let mut hint = stylist.compute_restyle_hint(&element,
                                                    self.snapshot.as_ref().unwrap());

        // If the hint includes a directive for later siblings, strip it out and
        // notify the caller to modify the base hint for future siblings.
        let later_siblings = hint.contains(RESTYLE_LATER_SIBLINGS);
        hint.remove(RESTYLE_LATER_SIBLINGS);

        // Insert the hint.
        self.hint.insert(&hint.into());

        // Destroy the snapshot.
        self.snapshot.destroy();

        later_siblings
    }

    /// Returns true if this RestyleData might invalidate the current style.
    pub fn has_invalidations(&self) -> bool {
        self.hint.restyle_self || self.recascade || self.snapshot.is_some()
    }
}

/// Style system data associated with an Element.
///
/// In Gecko, this hangs directly off the Element. Servo, this is embedded
/// inside of layout data, which itself hangs directly off the Element. In
/// both cases, it is wrapped inside an AtomicRefCell to ensure thread safety.
#[derive(Debug)]
pub struct ElementData {
    /// The computed styles for the element and its pseudo-elements.
    styles: Option<ElementStyles>,

    /// Restyle tracking. We separate this into a separate allocation so that
    /// we can drop it when no restyles are pending on the elemnt.
    restyle: Option<Box<RestyleData>>,
}

impl ElementData {
    /// Trivially construct an ElementData.
    pub fn new(existing: Option<ElementStyles>) -> Self {
        ElementData {
            styles: existing,
            restyle: None,
        }
    }

    /// Returns true if this element has a computed styled.
    pub fn has_styles(&self) -> bool {
        self.styles.is_some()
    }

    /// Returns true if this element's style is up-to-date and has no potential
    /// invalidation.
    pub fn has_current_styles(&self) -> bool {
        self.has_styles() &&
        self.restyle.as_ref().map_or(true, |r| !r.has_invalidations())
    }

    /// Gets the element styles, if any.
    pub fn get_styles(&self) -> Option<&ElementStyles> {
        self.styles.as_ref()
    }

    /// Gets the element styles. Panic if the element has never been styled.
    pub fn styles(&self) -> &ElementStyles {
        self.styles.as_ref().expect("Calling styles() on unstyled ElementData")
    }

    /// Gets a mutable reference to the element styles, if any.
    pub fn get_styles_mut(&mut self) -> Option<&mut ElementStyles> {
        self.styles.as_mut()
    }

    /// Gets a mutable reference to the element styles. Panic if the element has
    /// never been styled.
    pub fn styles_mut(&mut self) -> &mut ElementStyles {
        self.styles.as_mut().expect("Caling styles_mut() on unstyled ElementData")
    }

    /// Sets the computed element styles.
    pub fn set_styles(&mut self, styles: ElementStyles) {
        debug_assert!(self.get_restyle().map_or(true, |r| r.snapshot.is_none()),
                      "Traversal should have expanded snapshots");
        self.styles = Some(styles);
    }

    /// Returns true if the Element has a RestyleData.
    pub fn has_restyle(&self) -> bool {
        self.restyle.is_some()
    }

    /// Drops any RestyleData.
    pub fn clear_restyle(&mut self) {
        self.restyle = None;
    }

    /// Creates a RestyleData if one doesn't exist.
    ///
    /// Asserts that the Element has been styled.
    pub fn ensure_restyle(&mut self) -> &mut RestyleData {
        debug_assert!(self.styles.is_some(), "restyling unstyled element");
        if self.restyle.is_none() {
            self.restyle = Some(Box::new(RestyleData::default()));
        }
        self.restyle.as_mut().unwrap()
    }

    /// Gets a reference to the restyle data, if any.
    pub fn get_restyle(&self) -> Option<&RestyleData> {
        self.restyle.as_ref().map(|r| &**r)
    }

    /// Gets a reference to the restyle data. Panic if the element does not
    /// have restyle data.
    pub fn restyle(&self) -> &RestyleData {
        self.get_restyle().expect("Calling restyle without RestyleData")
    }

    /// Gets a mutable reference to the restyle data, if any.
    pub fn get_restyle_mut(&mut self) -> Option<&mut RestyleData> {
        self.restyle.as_mut().map(|r| &mut **r)
    }

    /// Gets a mutable reference to the restyle data. Panic if the element does
    /// not have restyle data.
    pub fn restyle_mut(&mut self) -> &mut RestyleData {
        self.get_restyle_mut().expect("Calling restyle_mut without RestyleData")
    }
}
