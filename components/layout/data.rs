/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use construct::{ConstructionItem, ConstructionResult};
use incremental::RestyleDamage;
use msg::constellation_msg::ConstellationChan;
use parallel::DomParallelInfo;
use script::dom::node::SharedLayoutData;
use std::sync::Arc;
use style::properties::ComputedValues;

/// Data that layout associates with a node.
pub struct PrivateLayoutData {
    /// The results of CSS styling for this node's `before` pseudo-element, if any.
    pub before_style: Option<Arc<ComputedValues>>,

    /// The results of CSS styling for this node's `after` pseudo-element, if any.
    pub after_style: Option<Arc<ComputedValues>>,

    /// Description of how to account for recent style changes.
    pub restyle_damage: RestyleDamage,

    /// The current results of flow construction for this node. This is either a flow or a
    /// `ConstructionItem`. See comments in `construct.rs` for more details.
    pub flow_construction_result: ConstructionResult,

    pub before_flow_construction_result: ConstructionResult,

    pub after_flow_construction_result: ConstructionResult,

    /// Information needed during parallel traversals.
    pub parallel: DomParallelInfo,

    /// Various flags.
    pub flags: LayoutDataFlags,
}

impl PrivateLayoutData {
    /// Creates new layout data.
    pub fn new() -> PrivateLayoutData {
        PrivateLayoutData {
            before_style: None,
            after_style: None,
            restyle_damage: RestyleDamage::empty(),
            flow_construction_result: ConstructionResult::None,
            before_flow_construction_result: ConstructionResult::None,
            after_flow_construction_result: ConstructionResult::None,
            parallel: DomParallelInfo::new(),
            flags: LayoutDataFlags::empty(),
        }
    }
}

bitflags! {
    flags LayoutDataFlags: u8 {
        #[doc = "Whether a flow has been newly constructed."]
        const HAS_NEWLY_CONSTRUCTED_FLOW = 0x01
    }
}

pub struct LayoutDataWrapper {
    pub shared_data: SharedLayoutData,
    pub data: Box<PrivateLayoutData>,
}

impl LayoutDataWrapper {
    pub fn remove_compositor_layers(&self, constellation_chan: ConstellationChan) {
        match self.data.flow_construction_result {
            ConstructionResult::None => {}
            ConstructionResult::Flow(ref flow_ref, _) => {
                flow_ref.remove_compositor_layers(constellation_chan);
            }
            ConstructionResult::ConstructionItem(ref construction_item) => {
                match construction_item {
                    &ConstructionItem::InlineFragments(ref inline_fragments) => {
                        for fragment in &inline_fragments.fragments.fragments {
                            fragment.remove_compositor_layers(constellation_chan.clone());
                        }
                    }
                    &ConstructionItem::Whitespace(..) => {}
                    &ConstructionItem::TableColumnFragment(ref fragment) => {
                        fragment.remove_compositor_layers(constellation_chan.clone());
                    }
                }
            }
        }
    }
}

#[allow(dead_code, unsafe_code)]
fn static_assertion(x: Option<LayoutDataWrapper>) {
    unsafe {
        let _: Option<::script::dom::node::LayoutData> =
            ::std::intrinsics::transmute(x);
    }
}
