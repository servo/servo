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
use std::mem;
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
///
/// TODO(emilio): Tell bholley to document this more accurately. I can try (and
/// the fields are certainly mostly self-explanatory), but it's better if he
/// does, to avoid any misconception.
#[derive(Debug)]
#[allow(missing_docs)]
pub struct RestyleData {
    pub styles: ElementStyles,
    pub hint: StoredRestyleHint,
    pub recascade: bool,
    pub damage: RestyleDamage,
    pub snapshot: SnapshotOption,
}

impl RestyleData {
    fn new(styles: ElementStyles) -> Self {
        RestyleData {
            styles: styles,
            hint: StoredRestyleHint::default(),
            recascade: false,
            damage: RestyleDamage::empty(),
            snapshot: SnapshotOption::empty(),
        }
    }

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

    /// Return if the element style's are up to date.
    pub fn has_current_styles(&self) -> bool {
        !(self.hint.restyle_self || self.recascade || self.snapshot.is_some())
    }

    /// Returns the element styles.
    pub fn styles(&self) -> &ElementStyles {
        &self.styles
    }

    /// Returns a mutable reference to the element styles.
    pub fn styles_mut(&mut self) -> &mut ElementStyles {
        &mut self.styles
    }

    fn finish_styling(&mut self, styles: ElementStyles, damage: RestyleDamage) {
        debug_assert!(!self.has_current_styles());
        debug_assert!(self.snapshot.is_none(), "Traversal should have expanded snapshots");
        self.styles = styles;
        self.damage |= damage;
        // The hint and recascade bits get cleared by the traversal code. This
        // is a bit confusing, and we should simplify it when we separate matching
        // from cascading.
    }
}

/// Style system data associated with a node.
///
/// In Gecko, this hangs directly off a node, but is dropped when the frame takes
/// ownership of the computed style data.
///
/// In Servo, this is embedded inside of layout data, which itself hangs directly
/// off the node. Servo does not currently implement ownership transfer of the
/// computed style data to the frame.
///
/// In both cases, it is wrapped inside an AtomicRefCell to ensure thread
/// safety.
#[derive(Debug)]
pub enum ElementData {
    /// This is the first styling for this element.
    Initial(Option<ElementStyles>),
    /// This element has been restyled already, and all the relevant data is
    /// inside the `RestyleData`.
    Restyle(RestyleData),
    /// This element has already been restyled, and only keeps its styles
    /// around.
    Persistent(ElementStyles),
}

impl ElementData {
    /// Trivially construct an ElementData.
    pub fn new(existing: Option<ElementStyles>) -> Self {
        if let Some(s) = existing {
            ElementData::Persistent(s)
        } else {
            ElementData::Initial(None)
        }
    }

    /// Return whether this data is from an initial restyle.
    pub fn is_initial(&self) -> bool {
        match *self {
            ElementData::Initial(_) => true,
            _ => false,
        }
    }

    /// Return whether this data is from an element that hasn't been restyled.
    pub fn is_unstyled_initial(&self) -> bool {
        match *self {
            ElementData::Initial(None) => true,
            _ => false,
        }
    }

    /// Return whether this data is from an element whose first restyle has just
    /// been done.
    pub fn is_styled_initial(&self) -> bool {
        match *self {
            ElementData::Initial(Some(_)) => true,
            _ => false,
        }
    }

    /// Returns true if this element is being restyled and has been styled
    /// before.
    pub fn is_restyle(&self) -> bool {
        match *self {
            ElementData::Restyle(_) => true,
            _ => false,
        }
    }

    /// Returns the `RestyleData` if it exists.
    pub fn as_restyle(&self) -> Option<&RestyleData> {
        match *self {
            ElementData::Restyle(ref x) => Some(x),
            _ => None,
        }
    }

    /// Returns a mutable reference to the RestyleData, if it exists.
    pub fn as_restyle_mut(&mut self) -> Option<&mut RestyleData> {
        match *self {
            ElementData::Restyle(ref mut x) => Some(x),
            _ => None,
        }
    }

    /// Returns whether this element's style is persistent.
    pub fn is_persistent(&self) -> bool {
        match *self {
            ElementData::Persistent(_) => true,
            _ => false,
        }
    }

    /// Sets an element up for restyle, returning None for an unstyled element.
    pub fn restyle(&mut self) -> Option<&mut RestyleData> {
        if self.is_unstyled_initial() {
            return None;
        }

        // If the caller never consumed the initial style, make sure that the
        // change hint represents the delta from zero, rather than a delta from
        // a previous style that was never observed. Ideally this shouldn't
        // happen, but we handle it for robustness' sake.
        let damage_override = if self.is_styled_initial() {
            RestyleDamage::rebuild_and_reflow()
        } else {
            RestyleDamage::empty()
        };

        if !self.is_restyle() {
            // Play some tricks to reshape the enum without cloning ElementStyles.
            let old = mem::replace(self, ElementData::new(None));
            let styles = match old {
                ElementData::Initial(Some(s)) => s,
                ElementData::Persistent(s) => s,
                _ => unreachable!()
            };
            *self = ElementData::Restyle(RestyleData::new(styles));
        }

        let restyle = self.as_restyle_mut().unwrap();
        restyle.damage |= damage_override;
        Some(restyle)
    }

    /// Converts Initial and Restyle to Persistent. No-op for Persistent.
    pub fn persist(&mut self) {
        if self.is_persistent() {
            return;
        }

        // Play some tricks to reshape the enum without cloning ElementStyles.
        let old = mem::replace(self, ElementData::new(None));
        let styles = match old {
            ElementData::Initial(i) => i.unwrap(),
            ElementData::Restyle(r) => r.styles,
            ElementData::Persistent(_) => unreachable!(),
        };
        *self = ElementData::Persistent(styles);
    }

    /// Return the restyle damage (if any).
    pub fn damage(&self) -> RestyleDamage {
        use self::ElementData::*;
        match *self {
            Initial(ref s) => {
                debug_assert!(s.is_some());
                RestyleDamage::rebuild_and_reflow()
            },
            Restyle(ref r) => {
                debug_assert!(r.has_current_styles());
                r.damage
            },
            Persistent(_) => RestyleDamage::empty(),
        }
    }

    /// A version of the above, with the assertions replaced with warnings to
    /// be more robust in corner-cases. This will go away soon.
    #[cfg(feature = "gecko")]
    pub fn damage_sloppy(&self) -> RestyleDamage {
        use self::ElementData::*;
        match *self {
            Initial(ref s) => {
                if s.is_none() {
                    error!("Accessing damage on unstyled element");
                }
                RestyleDamage::rebuild_and_reflow()
            },
            Restyle(ref r) => {
                if !r.has_current_styles() {
                    error!("Accessing damage on dirty element");
                }
                r.damage
            },
            Persistent(_) => RestyleDamage::empty(),
        }
    }

    /// Returns true if this element's style is up-to-date and has no potential
    /// invalidation.
    pub fn has_current_styles(&self) -> bool {
        use self::ElementData::*;
        match *self {
            Initial(ref x) => x.is_some(),
            Restyle(ref x) => x.has_current_styles(),
            Persistent(_) => true,
        }
    }

    /// Get the element styles, if any.
    pub fn get_styles(&self) -> Option<&ElementStyles> {
        use self::ElementData::*;
        match *self {
            Initial(ref x) => x.as_ref(),
            Restyle(ref x) => Some(x.styles()),
            Persistent(ref x) => Some(x),
        }
    }

    /// Get the element styles. Panic if the element has never been styled.
    pub fn styles(&self) -> &ElementStyles {
        self.get_styles().expect("Calling styles() on unstyled ElementData")
    }

    /// Get a mutable reference to the element styles, if any.
    pub fn get_styles_mut(&mut self) -> Option<&mut ElementStyles> {
        use self::ElementData::*;
        match *self {
            Initial(ref mut x) => x.as_mut(),
            Restyle(ref mut x) => Some(x.styles_mut()),
            Persistent(ref mut x) => Some(x),
        }
    }

    /// Get a mutable reference to the element styles. Panic if the element has
    /// never been styled.
    pub fn styles_mut(&mut self) -> &mut ElementStyles {
        self.get_styles_mut().expect("Calling styles_mut() on unstyled ElementData")
    }

    /// Finishes the styling of the element, effectively setting the style in
    /// the data.
    pub fn finish_styling(&mut self, styles: ElementStyles, damage: RestyleDamage) {
        use self::ElementData::*;
        match *self {
            Initial(ref mut x) => {
                debug_assert!(x.is_none());
                debug_assert!(damage == RestyleDamage::rebuild_and_reflow());
                *x = Some(styles);
            },
            Restyle(ref mut x) => x.finish_styling(styles, damage),
            Persistent(_) => panic!("Calling finish_styling on Persistent ElementData"),
        };
    }
}
