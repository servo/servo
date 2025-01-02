/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use atomic_refcell::AtomicRefCell;
use bitflags::bitflags;
use script_layout_interface::wrapper_traits::LayoutDataTrait;

use crate::construct::ConstructionResult;

/// Data that layout associates with a node.
#[derive(Clone, Default)]
pub struct InnerLayoutData {
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

bitflags! {
    #[derive(Clone, Copy, Default)]
    pub struct LayoutDataFlags: u8 {
        /// Whether a flow has been newly constructed.
        const HAS_NEWLY_CONSTRUCTED_FLOW = 0x01;
        /// Whether this node has been traversed by layout.
        const HAS_BEEN_TRAVERSED = 0x02;
    }
}

/// A wrapper for [`InnerLayoutData`]. This is necessary to give the entire data
/// structure interior mutability, as we will need to mutate the layout data of
/// non-mutable DOM nodes.
#[derive(Clone, Default)]
pub struct LayoutData(pub AtomicRefCell<InnerLayoutData>);

// The implementation of this trait allows the data to be stored in the DOM.
impl LayoutDataTrait for LayoutData {}
