/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::marker::PhantomData;
use std::sync::{Arc, Mutex};

use atomic_refcell::{AtomicRefCell, AtomicRefMut};
use msg::constellation_msg::{BrowsingContextId, PipelineId};
use net_traits::image::base::Image as NetImage;
use script_layout_interface::wrapper_traits::{LayoutDataTrait, LayoutNode, ThreadSafeLayoutNode};
use script_layout_interface::{HTMLCanvasDataSource, StyleData};
use servo_arc::Arc as ServoArc;
use style::properties::ComputedValues;

use crate::cell::ArcRefCell;
use crate::context::LayoutContext;
use crate::dom_traversal::WhichPseudoElement;
use crate::flexbox::FlexLevelBox;
use crate::flow::inline::InlineLevelBox;
use crate::flow::BlockLevelBox;
use crate::geom::PhysicalSize;
use crate::replaced::{CanvasInfo, CanvasSource};

/// The data that is stored in each DOM node that is used by layout.
#[derive(Default)]
pub struct DOMLayoutData {
    pub(super) self_box: ArcRefCell<Option<LayoutBox>>,
    pub(super) pseudo_before_box: ArcRefCell<Option<LayoutBox>>,
    pub(super) pseudo_after_box: ArcRefCell<Option<LayoutBox>>,
}

/// A box that is stored in one of the `DOMLayoutData` slots.
pub(super) enum LayoutBox {
    DisplayContents,
    BlockLevel(ArcRefCell<BlockLevelBox>),
    InlineLevel(ArcRefCell<InlineLevelBox>),
    FlexLevel(ArcRefCell<FlexLevelBox>),
}

// The implementation of this trait allows the data to be stored in the DOM.
impl LayoutDataTrait for DOMLayoutData {}

pub struct StyleAndLayoutData<'dom> {
    pub style_data: &'dom StyleData,
    pub(super) layout_data: &'dom AtomicRefCell<DOMLayoutData>,
}

pub struct BoxSlot<'dom> {
    pub(crate) slot: Option<ArcRefCell<Option<LayoutBox>>>,
    pub(crate) marker: PhantomData<&'dom ()>,
}

/// A mutable reference to a `LayoutBox` stored in a DOM element.
impl BoxSlot<'_> {
    pub(crate) fn new(slot: ArcRefCell<Option<LayoutBox>>) -> Self {
        *slot.borrow_mut() = None;
        let slot = Some(slot);
        Self {
            slot,
            marker: PhantomData,
        }
    }

    pub(crate) fn dummy() -> Self {
        let slot = None;
        Self {
            slot,
            marker: PhantomData,
        }
    }

    pub(crate) fn set(mut self, box_: LayoutBox) {
        if let Some(slot) = &mut self.slot {
            *slot.borrow_mut() = Some(box_);
        }
    }
}

impl Drop for BoxSlot<'_> {
    fn drop(&mut self) {
        if !std::thread::panicking() {
            if let Some(slot) = &mut self.slot {
                assert!(slot.borrow().is_some(), "failed to set a layout box");
            }
        }
    }
}

pub(crate) trait NodeExt<'dom>: 'dom + LayoutNode<'dom> {
    /// Returns the image if it’s loaded, and its size in image pixels
    /// adjusted for `image_density`.
    fn as_image(self) -> Option<(Option<Arc<NetImage>>, PhysicalSize<f64>)>;
    fn as_canvas(self) -> Option<(CanvasInfo, PhysicalSize<f64>)>;
    fn as_iframe(self) -> Option<(PipelineId, BrowsingContextId)>;
    fn style(self, context: &LayoutContext) -> ServoArc<ComputedValues>;

    fn get_style_and_layout_data(self) -> Option<StyleAndLayoutData<'dom>>;
    fn layout_data_mut(self) -> AtomicRefMut<'dom, DOMLayoutData>;
    fn element_box_slot(&self) -> BoxSlot<'dom>;
    fn pseudo_element_box_slot(&self, which: WhichPseudoElement) -> BoxSlot<'dom>;
    fn unset_pseudo_element_box(self, which: WhichPseudoElement);

    /// Remove boxes for the element itself, and its `:before` and `:after` if any.
    fn unset_all_boxes(self);
}

impl<'dom, LayoutNodeType> NodeExt<'dom> for LayoutNodeType
where
    LayoutNodeType: 'dom + LayoutNode<'dom>,
{
    fn as_image(self) -> Option<(Option<Arc<NetImage>>, PhysicalSize<f64>)> {
        let node = self.to_threadsafe();
        let (resource, metadata) = node.image_data()?;
        let (width, height) = resource
            .as_ref()
            .map(|image| (image.width, image.height))
            .or_else(|| metadata.map(|metadata| (metadata.width, metadata.height)))
            .unwrap_or((0, 0));
        let (mut width, mut height) = (width as f64, height as f64);
        if let Some(density) = node.image_density().filter(|density| *density != 1.) {
            width = width / density;
            height = height / density;
        }
        Some((resource, PhysicalSize::new(width, height)))
    }

    fn as_canvas(self) -> Option<(CanvasInfo, PhysicalSize<f64>)> {
        let node = self.to_threadsafe();
        let canvas_data = node.canvas_data()?;
        let source = match canvas_data.source {
            HTMLCanvasDataSource::WebGL(texture_id) => CanvasSource::WebGL(texture_id),
            HTMLCanvasDataSource::Image(ipc_sender) => {
                CanvasSource::Image(ipc_sender.map(|renderer| Arc::new(Mutex::new(renderer))))
            },
            HTMLCanvasDataSource::WebGPU(image_key) => CanvasSource::WebGPU(image_key),
        };
        Some((
            CanvasInfo {
                source,
                canvas_id: canvas_data.canvas_id,
            },
            PhysicalSize::new(canvas_data.width.into(), canvas_data.height.into()),
        ))
    }

    fn as_iframe(self) -> Option<(PipelineId, BrowsingContextId)> {
        let node = self.to_threadsafe();
        match (node.iframe_pipeline_id(), node.iframe_browsing_context_id()) {
            (Some(pipeline_id), Some(browsing_context_id)) => {
                Some((pipeline_id, browsing_context_id))
            },
            _ => None,
        }
    }

    fn style(self, context: &LayoutContext) -> ServoArc<ComputedValues> {
        self.to_threadsafe().style(context.shared_context())
    }

    fn layout_data_mut(self) -> AtomicRefMut<'dom, DOMLayoutData> {
        self.get_style_and_layout_data()
            .map(|d| d.layout_data.borrow_mut())
            .unwrap()
    }

    fn element_box_slot(&self) -> BoxSlot<'dom> {
        BoxSlot::new(self.layout_data_mut().self_box.clone())
    }

    fn pseudo_element_box_slot(&self, which: WhichPseudoElement) -> BoxSlot<'dom> {
        let data = self.layout_data_mut();
        let cell = match which {
            WhichPseudoElement::Before => &data.pseudo_before_box,
            WhichPseudoElement::After => &data.pseudo_after_box,
        };
        BoxSlot::new(cell.clone())
    }

    fn unset_pseudo_element_box(self, which: WhichPseudoElement) {
        let data = self.layout_data_mut();
        let cell = match which {
            WhichPseudoElement::Before => &data.pseudo_before_box,
            WhichPseudoElement::After => &data.pseudo_after_box,
        };
        *cell.borrow_mut() = None;
    }

    fn unset_all_boxes(self) {
        let data = self.layout_data_mut();
        *data.self_box.borrow_mut() = None;
        *data.pseudo_before_box.borrow_mut() = None;
        *data.pseudo_after_box.borrow_mut() = None;
        // Stylo already takes care of removing all layout data
        // for DOM descendants of elements with `display: none`.
    }

    fn get_style_and_layout_data(self) -> Option<StyleAndLayoutData<'dom>> {
        self.get_style_and_opaque_layout_data()
            .map(|data| StyleAndLayoutData {
                style_data: &data.style_data,
                layout_data: data.generic_data.downcast_ref().unwrap(),
            })
    }
}
