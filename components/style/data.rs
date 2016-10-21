/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Per-node data used in style calculation.

use properties::ComputedValues;
use selector_impl::PseudoElement;
use std::collections::HashMap;
use std::hash::BuildHasherDefault;
use std::mem;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

type PseudoStylesInner = HashMap<PseudoElement, Arc<ComputedValues>,
                                 BuildHasherDefault<::fnv::FnvHasher>>;
#[derive(Clone, Debug)]
pub struct PseudoStyles(PseudoStylesInner);

impl PseudoStyles {
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
pub struct NodeStyles {
    /// The results of CSS styling for this node.
    pub primary: Arc<ComputedValues>,

    /// The results of CSS styling for each pseudo-element (if any).
    pub pseudos: PseudoStyles,
}

impl NodeStyles {
    pub fn new(primary: Arc<ComputedValues>) -> Self {
        NodeStyles {
            primary: primary,
            pseudos: PseudoStyles::empty(),
        }
    }
}

#[derive(Debug)]
enum NodeDataStyles {
    /// The field has not been initialized.
    Uninitialized,

    /// The field holds the previous style of the node. If this is None, the
    /// node has not been previously styled.
    ///
    /// This is the input to the styling algorithm. It would ideally be
    /// immutable, but for now we need to mutate it a bit before styling to
    /// handle animations.
    ///
    /// Note that since NodeStyles contains an Arc, the null pointer
    /// optimization prevents the Option<> here from consuming an extra word.
    Previous(Option<NodeStyles>),

    /// The field holds the current, up-to-date style.
    ///
    /// This is the output of the styling algorithm.
    Current(NodeStyles),
}

impl NodeDataStyles {
    fn is_previous(&self) -> bool {
        use self::NodeDataStyles::*;
        match *self {
            Previous(_) => true,
            _ => false,
        }
    }
}

/// Transient data used by the restyle algorithm. This structure is instantiated
/// either before or during restyle traversal, and is cleared at the end of node
/// processing.
#[derive(Debug)]
pub struct RestyleData {
    // FIXME(bholley): Start adding the fields from the algorithm doc.
    pub _dummy: u64,
}

impl RestyleData {
    fn new() -> Self {
        RestyleData {
            _dummy: 42,
        }
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
pub struct NodeData {
    styles: NodeDataStyles,
    pub restyle_data: Option<RestyleData>,
}

impl NodeData {
    pub fn new() -> Self {
        NodeData {
            styles: NodeDataStyles::Uninitialized,
            restyle_data: None,
        }
    }

    pub fn has_current_styles(&self) -> bool {
        match self.styles {
            NodeDataStyles::Current(_) => true,
            _ => false,
        }
    }

    pub fn get_current_styles(&self) -> Option<&NodeStyles> {
        match self.styles {
            NodeDataStyles::Current(ref s) => Some(s),
            _ => None,
        }
    }

    pub fn current_styles(&self) -> &NodeStyles {
        self.get_current_styles().expect("Calling current_styles before or during styling")
    }

    // Servo does lazy pseudo computation in layout and needs mutable access
    // to the current styles
    #[cfg(not(feature = "gecko"))]
    pub fn current_pseudos_mut(&mut self) -> &mut PseudoStyles {
        match self.styles {
            NodeDataStyles::Current(ref mut s) => &mut s.pseudos,
            _ => panic!("Calling current_pseudos_mut before or during styling"),
        }
    }

    pub fn previous_styles(&self) -> Option<&NodeStyles> {
        match self.styles {
            NodeDataStyles::Previous(ref s) => s.as_ref(),
            _ => panic!("Calling previous_styles without having gathered it"),
        }
    }

    pub fn previous_styles_mut(&mut self) -> Option<&mut NodeStyles> {
        match self.styles {
            NodeDataStyles::Previous(ref mut s) => s.as_mut(),
            _ => panic!("Calling previous_styles without having gathered it"),
        }
    }

    pub fn gather_previous_styles<F>(&mut self, f: F)
        where F: FnOnce() -> Option<NodeStyles>
    {
        use self::NodeDataStyles::*;
        self.styles = match mem::replace(&mut self.styles, Uninitialized) {
            Uninitialized => Previous(f()),
            Current(x) => Previous(Some(x)),
            Previous(x) => Previous(x),
        };
    }

    pub fn ensure_restyle_data(&mut self) {
        if self.restyle_data.is_none() {
            self.restyle_data = Some(RestyleData::new());
        }
    }

    pub fn style_text_node(&mut self, style: Arc<ComputedValues>) {
        self.styles = NodeDataStyles::Current(NodeStyles::new(style));
        self.restyle_data = None;
    }

    pub fn finish_styling(&mut self, styles: NodeStyles) {
        debug_assert!(self.styles.is_previous());
        self.styles = NodeDataStyles::Current(styles);
        self.restyle_data = None;
    }
}
