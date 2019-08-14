/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The `Fragment` type, which represents the leaves of the layout tree.

use crate::context::LayoutContext;
use crate::display_list::items::OpaqueNode;
use crate::ServoArc;
use app_units::Au;
use script_layout_interface::wrapper_traits::{PseudoElementType, ThreadSafeLayoutNode};
use serde::ser::{Serialize, SerializeStruct, Serializer};
use style::logical_geometry::{LogicalMargin, LogicalRect};
use style::properties::ComputedValues;
use style::selector_parser::RestyleDamage;
use style::servo::restyle_damage::ServoRestyleDamage;

#[derive(Clone)]
pub struct Fragment {
    pub node: OpaqueNode,
    pub style: ServoArc<ComputedValues>,
    pub border_box: LogicalRect<Au>,
    pub border_padding: LogicalMargin<Au>,
    pub margin: LogicalMargin<Au>,
    pub specific: SpecificFragmentInfo,
    pub restyle_damage: RestyleDamage,
    pub pseudo: PseudoElementType,
}

impl Serialize for Fragment {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut serializer = serializer.serialize_struct("fragment", 3)?;
        serializer.serialize_field("border_box", &self.border_box)?;
        serializer.serialize_field("margin", &self.margin)?;
        serializer.end()
    }
}

#[derive(Clone)]
pub enum SpecificFragmentInfo {
    Generic,
}

impl SpecificFragmentInfo {
    fn restyle_damage(&self) -> RestyleDamage {
        RestyleDamage::empty()
    }
}

impl Fragment {
    /// Constructs a new `Fragment` instance.
    pub fn new<N: ThreadSafeLayoutNode>(
        node: &N,
        specific: SpecificFragmentInfo,
        ctx: &LayoutContext,
    ) -> Fragment {
        let shared_context = ctx.shared_context();
        let style = node.style(shared_context);
        let writing_mode = style.writing_mode;

        let mut restyle_damage = RestyleDamage::rebuild_and_reflow();
        restyle_damage.remove(ServoRestyleDamage::RECONSTRUCT_FLOW);

        Fragment {
            node: node.opaque(),
            style: style,
            restyle_damage: restyle_damage,
            border_box: LogicalRect::zero(writing_mode),
            border_padding: LogicalMargin::zero(writing_mode),
            margin: LogicalMargin::zero(writing_mode),
            specific: specific,
            pseudo: node.get_pseudo_element_type(),
        }
    }

    pub fn restyle_damage(&self) -> RestyleDamage {
        self.restyle_damage | self.specific.restyle_damage()
    }

    pub fn contains_node(&self, node_address: OpaqueNode) -> bool {
        node_address == self.node
    }

    /// Returns the sum of the inline-sizes of all the borders of this fragment. Note that this
    /// can be expensive to compute, so if possible use the `border_padding` field instead.
    #[inline]
    pub fn border_width(&self) -> LogicalMargin<Au> {
        self.style().logical_border_width()
    }

    #[inline(always)]
    pub fn style(&self) -> &ComputedValues {
        &*self.style
    }

    pub fn is_primary_fragment(&self) -> bool {
        true
    }
}
