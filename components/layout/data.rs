/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use construct::ConstructionResult;
use incremental::RestyleDamage;
use style::servo::PrivateStyleData;

/// Data that layout associates with a node.
pub struct PrivateLayoutData {
    /// Data that the style system associates with a node. When the
    /// style system is being used standalone, this is all that hangs
    /// off the node. This must be first to permit the various
    /// transmuations between PrivateStyleData PrivateLayoutData.
    pub style_data: PrivateStyleData,

    /// Description of how to account for recent style changes.
    pub restyle_damage: RestyleDamage,

    /// The current results of flow construction for this node. This is either a flow or a
    /// `ConstructionItem`. See comments in `construct.rs` for more details.
    pub flow_construction_result: ConstructionResult,

    pub before_flow_construction_result: ConstructionResult,

    pub after_flow_construction_result: ConstructionResult,

    pub details_summary_flow_construction_result: ConstructionResult,

    pub details_content_flow_construction_result: ConstructionResult,

    /// Various flags.
    pub flags: LayoutDataFlags,
}

impl PrivateLayoutData {
    /// Creates new layout data.
    pub fn new() -> PrivateLayoutData {
        PrivateLayoutData {
            style_data: PrivateStyleData::new(),
            restyle_damage: RestyleDamage::empty(),
            flow_construction_result: ConstructionResult::None,
            before_flow_construction_result: ConstructionResult::None,
            after_flow_construction_result: ConstructionResult::None,
            details_summary_flow_construction_result: ConstructionResult::None,
            details_content_flow_construction_result: ConstructionResult::None,
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
