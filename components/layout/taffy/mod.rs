/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
mod layout;
mod stylo_taffy;
use std::fmt;

use app_units::Au;
use malloc_size_of_derive::MallocSizeOf;
use script::layout_dom::ServoLayoutNode;
use servo_arc::Arc;
use style::context::SharedStyleContext;
use style::properties::ComputedValues;
use stylo_taffy::TaffyStyloStyle;

use crate::PropagatedBoxTreeData;
use crate::cell::ArcRefCell;
use crate::construct_modern::{ModernContainerBuilder, ModernItemKind};
use crate::context::LayoutContext;
use crate::dom::LayoutBox;
use crate::dom_traversal::{NodeAndStyleInfo, NonReplacedContents};
use crate::formatting_contexts::IndependentFormattingContext;
use crate::fragment_tree::Fragment;
use crate::positioned::{AbsolutelyPositionedBox, PositioningContext};

#[derive(Debug, MallocSizeOf)]
pub(crate) struct TaffyContainer {
    children: Vec<ArcRefCell<TaffyItemBox>>,
    style: Arc<ComputedValues>,
}

impl TaffyContainer {
    pub fn construct(
        context: &LayoutContext,
        info: &NodeAndStyleInfo,
        contents: NonReplacedContents,
        propagated_data: PropagatedBoxTreeData,
    ) -> Self {
        let mut builder = ModernContainerBuilder::new(context, info, propagated_data);
        contents.traverse(context, info, &mut builder);
        let items = builder.finish();

        let children = items
            .into_iter()
            .map(|item| {
                let box_ = match item.kind {
                    ModernItemKind::InFlow(independent_formatting_context) => {
                        ArcRefCell::new(TaffyItemBox::new(TaffyItemBoxInner::InFlowBox(
                            independent_formatting_context,
                        )))
                    },
                    ModernItemKind::OutOfFlow(independent_formatting_context) => {
                        let abs_pos_box = ArcRefCell::new(AbsolutelyPositionedBox::new(
                            independent_formatting_context,
                        ));
                        ArcRefCell::new(TaffyItemBox::new(
                            TaffyItemBoxInner::OutOfFlowAbsolutelyPositionedBox(abs_pos_box),
                        ))
                    },
                    ModernItemKind::ReusedBox(layout_box) => match layout_box {
                        LayoutBox::TaffyItemBox(taffy_item_box) => taffy_item_box,
                        _ => unreachable!("Undamaged taffy level element should be associated with taffy level box"),
                    },
                };

                if let Some(box_slot) = item.box_slot {
                    box_slot.set(LayoutBox::TaffyItemBox(box_.clone()));
                }

                box_
            })
            .collect();

        Self {
            children,
            style: info.style.clone(),
        }
    }

    pub(crate) fn repair_style(&mut self, new_style: &Arc<ComputedValues>) {
        self.style = new_style.clone();
    }
}

#[derive(MallocSizeOf)]
pub(crate) struct TaffyItemBox {
    pub(crate) taffy_layout: taffy::Layout,
    pub(crate) child_fragments: Vec<Fragment>,
    pub(crate) positioning_context: PositioningContext,
    pub(crate) style: Arc<ComputedValues>,
    pub(crate) taffy_level_box: TaffyItemBoxInner,
}

#[derive(Debug, MallocSizeOf)]
pub(crate) enum TaffyItemBoxInner {
    InFlowBox(IndependentFormattingContext),
    OutOfFlowAbsolutelyPositionedBox(ArcRefCell<AbsolutelyPositionedBox>),
}

impl fmt::Debug for TaffyItemBox {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TaffyItemBox")
            .field("taffy_layout", &self.taffy_layout)
            .field("child_fragments", &self.child_fragments.len())
            .field("style", &self.style)
            .field("taffy_level_box", &self.taffy_level_box)
            .finish()
    }
}

impl TaffyItemBox {
    fn new(inner: TaffyItemBoxInner) -> Self {
        let style: Arc<ComputedValues> = match &inner {
            TaffyItemBoxInner::InFlowBox(item) => item.style().clone(),
            TaffyItemBoxInner::OutOfFlowAbsolutelyPositionedBox(absbox) => {
                (*absbox).borrow().context.style().clone()
            },
        };

        Self {
            taffy_layout: Default::default(),
            child_fragments: Vec::new(),
            positioning_context: PositioningContext::default(),
            style,
            taffy_level_box: inner,
        }
    }

    pub(crate) fn clear_fragment_layout_cache(&mut self) {
        self.taffy_layout = Default::default();
        self.positioning_context = PositioningContext::default();
        match self.taffy_level_box {
            TaffyItemBoxInner::InFlowBox(ref independent_formatting_context) => {
                independent_formatting_context
                    .base
                    .clear_fragment_layout_cache()
            },
            TaffyItemBoxInner::OutOfFlowAbsolutelyPositionedBox(ref positioned_box) => {
                positioned_box
                    .borrow()
                    .context
                    .base
                    .clear_fragment_layout_cache()
            },
        }
    }

    pub(crate) fn fragments(&self) -> Vec<Fragment> {
        match self.taffy_level_box {
            TaffyItemBoxInner::InFlowBox(ref independent_formatting_context) => {
                independent_formatting_context.base.fragments()
            },
            TaffyItemBoxInner::OutOfFlowAbsolutelyPositionedBox(ref positioned_box) => {
                positioned_box.borrow().context.base.fragments()
            },
        }
    }

    pub(crate) fn repair_style(
        &mut self,
        context: &SharedStyleContext,
        node: &ServoLayoutNode,
        new_style: &Arc<ComputedValues>,
    ) {
        self.style = new_style.clone();
        match &mut self.taffy_level_box {
            TaffyItemBoxInner::InFlowBox(independent_formatting_context) => {
                independent_formatting_context.repair_style(context, node, new_style)
            },
            TaffyItemBoxInner::OutOfFlowAbsolutelyPositionedBox(positioned_box) => positioned_box
                .borrow_mut()
                .context
                .repair_style(context, node, new_style),
        }
    }

    fn is_in_flow_replaced(&self) -> bool {
        match &self.taffy_level_box {
            TaffyItemBoxInner::InFlowBox(fc) => fc.is_replaced(),
            TaffyItemBoxInner::OutOfFlowAbsolutelyPositionedBox(_) => false,
        }
    }
}

/// Details from Taffy grid layout that will be stored
#[derive(Clone, Debug, MallocSizeOf)]
pub(crate) struct SpecificTaffyGridInfo {
    pub rows: SpecificTaffyGridTrackInfo,
    pub columns: SpecificTaffyGridTrackInfo,
}

impl SpecificTaffyGridInfo {
    fn from_detailed_grid_layout(grid_info: taffy::DetailedGridInfo) -> Self {
        Self {
            rows: SpecificTaffyGridTrackInfo {
                sizes: grid_info
                    .rows
                    .sizes
                    .iter()
                    .map(|size| Au::from_f32_px(*size))
                    .collect(),
            },
            columns: SpecificTaffyGridTrackInfo {
                sizes: grid_info
                    .columns
                    .sizes
                    .iter()
                    .map(|size| Au::from_f32_px(*size))
                    .collect(),
            },
        }
    }
}

#[derive(Clone, Debug, MallocSizeOf)]
pub(crate) struct SpecificTaffyGridTrackInfo {
    pub sizes: Box<[Au]>,
}
