/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use atomic_refcell::AtomicRefCell;
use bitflags::bitflags;
use script_layout_interface::wrapper_traits::LayoutDataTrait;
use script_layout_interface::StyleData;

use crate::construct::ConstructionResult;

pub struct StyleAndLayoutData<'dom> {
    /// The style data associated with a node.
    pub style_data: &'dom StyleData,
    /// The layout data associated with a node.
    pub layout_data: &'dom AtomicRefCell<LayoutData>,
}

/// Data that layout associates with a node.
#[derive(Clone)]
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

impl LayoutDataTrait for LayoutData {}

impl Default for LayoutData {
    /// Creates new layout data.
    fn default() -> LayoutData {
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
    #[derive(Clone, Copy)]
    pub struct LayoutDataFlags: u8 {
        /// Whether a flow has been newly constructed.
        const HAS_NEWLY_CONSTRUCTED_FLOW = 0x01;
        /// Whether this node has been traversed by layout.
        const HAS_BEEN_TRAVERSED = 0x02;
    }
}
