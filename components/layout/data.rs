/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use atomic_refcell::AtomicRefCell;
use construct::ConstructionResult;
use script_layout_interface::StyleData;

#[repr(C)]
pub struct StyleAndLayoutData {
    /// Data accessed by script_layout_interface. This must be first to allow
    /// casting between StyleAndLayoutData and StyleData.
    pub style_data: StyleData,
    /// The layout data associated with a node.
    pub layout_data: AtomicRefCell<LayoutData>,
}

impl StyleAndLayoutData {
    pub fn new() -> Self {
        Self {
            style_data: StyleData::new(),
            layout_data: AtomicRefCell::new(LayoutData::new()),
        }
    }
}


/// Data that layout associates with a node.
#[repr(C)]
pub struct LayoutData {
    /// The current results of flow construction for this node. This is either a
    /// flow or a `ConstructionItem`. See comments in `construct.rs` for more
    /// details.
    pub flow_construction_result: ConstructionResult,

    pub before_flow_construction_result: ConstructionResult,

    pub after_flow_construction_result: ConstructionResult,

    pub details_summary_flow_construction_result: ConstructionResult,

    pub details_content_flow_construction_result: ConstructionResult,

    /// Various flags.
    pub flags: LayoutDataFlags,
}

impl LayoutData {
    /// Creates new layout data.
    pub fn new() -> LayoutData {
        Self {
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
    pub flags LayoutDataFlags: u8 {
        #[doc = "Whether a flow has been newly constructed."]
        const HAS_NEWLY_CONSTRUCTED_FLOW = 0x01,
        #[doc = "Whether this node has been traversed by layout."]
        const HAS_BEEN_TRAVERSED = 0x02,
    }
}
