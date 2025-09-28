/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use app_units::Au;
use euclid::Vector2D;
use euclid::default::Rect;
use layout_api::{AxesOverflow, ScrollContainerQueryFlags};
use script_bindings::codegen::GenericBindings::WindowBinding::ScrollBehavior;
use script_bindings::inheritance::Castable;
use script_bindings::root::DomRoot;
use style::values::computed::Overflow;
use webrender_api::units::{LayoutSize, LayoutVector2D};

use crate::dom::bindings::codegen::Bindings::ElementBinding::ScrollLogicalPosition;
use crate::dom::node::{Node, NodeTraits};
use crate::dom::types::{Document, Element};

pub(crate) struct ScrollingBox {
    target: ScrollingBoxSource,
    overflow: AxesOverflow,
    cached_content_size: Cell<Option<LayoutSize>>,
    cached_size: Cell<Option<LayoutSize>>,
}

/// Represents a scrolling box that can be either an element or the viewport
/// <https://drafts.csswg.org/cssom-view/#scrolling-box>
pub(crate) enum ScrollingBoxSource {
    Element(DomRoot<Element>),
    Viewport(DomRoot<Document>),
}

#[derive(Copy, Clone)]
pub(crate) enum ScrollingBoxAxis {
    X,
    Y,
}

impl ScrollingBox {
    pub(crate) fn new(target: ScrollingBoxSource, overflow: AxesOverflow) -> Self {
        Self {
            target,
            overflow,
            cached_content_size: Default::default(),
            cached_size: Default::default(),
        }
    }

    pub(crate) fn target(&self) -> &ScrollingBoxSource {
        &self.target
    }

    pub(crate) fn is_viewport(&self) -> bool {
        matches!(self.target, ScrollingBoxSource::Viewport(..))
    }

    pub(crate) fn scroll_position(&self) -> LayoutVector2D {
        match &self.target {
            ScrollingBoxSource::Element(element) => element
                .owner_window()
                .scroll_offset_query(element.upcast::<Node>()),
            ScrollingBoxSource::Viewport(document) => document.window().scroll_offset(),
        }
    }

    pub(crate) fn content_size(&self) -> LayoutSize {
        if let Some(content_size) = self.cached_content_size.get() {
            return content_size;
        }

        let (document, node_to_query) = match &self.target {
            ScrollingBoxSource::Element(element) => {
                (element.owner_document(), Some(element.upcast()))
            },
            ScrollingBoxSource::Viewport(document) => (document.clone(), None),
        };

        let content_size = document
            .window()
            .scrolling_area_query(node_to_query)
            .size
            .to_f32()
            .cast_unit();
        self.cached_content_size.set(Some(content_size));
        content_size
    }

    pub(crate) fn size(&self) -> LayoutSize {
        if let Some(size) = self.cached_size.get() {
            return size;
        }

        let size = match &self.target {
            ScrollingBoxSource::Element(element) => element.client_rect().size.to_f32().cast_unit(),
            ScrollingBoxSource::Viewport(document) => {
                document.window().viewport_details().size.cast_unit()
            },
        };
        self.cached_size.set(Some(size));
        size
    }

    pub(crate) fn parent(&self) -> Option<ScrollingBox> {
        match &self.target {
            ScrollingBoxSource::Element(element) => {
                element.scrolling_box(ScrollContainerQueryFlags::empty())
            },
            ScrollingBoxSource::Viewport(_) => None,
        }
    }

    pub(crate) fn node(&self) -> &Node {
        match &self.target {
            ScrollingBoxSource::Element(element) => element.upcast(),
            ScrollingBoxSource::Viewport(document) => document.upcast(),
        }
    }

    pub(crate) fn scroll_to(&self, position: LayoutVector2D, behavior: ScrollBehavior) {
        match &self.target {
            ScrollingBoxSource::Element(element) => {
                element
                    .owner_window()
                    .scroll_an_element(element, position.x, position.y, behavior);
            },
            ScrollingBoxSource::Viewport(document) => {
                document.window().scroll(position.x, position.y, behavior);
            },
        }
    }

    pub(crate) fn can_keyboard_scroll_in_axis(&self, axis: ScrollingBoxAxis) -> bool {
        let overflow = match axis {
            ScrollingBoxAxis::X => self.overflow.x,
            ScrollingBoxAxis::Y => self.overflow.y,
        };
        if overflow == Overflow::Hidden {
            return false;
        }
        match axis {
            ScrollingBoxAxis::X => self.content_size().width > self.size().width,
            ScrollingBoxAxis::Y => self.content_size().height > self.size().height,
        }
    }

    /// <https://drafts.csswg.org/cssom-view/#determine-the-scroll-into-view-position>
    pub(crate) fn determine_scroll_into_view_position(
        &self,
        block: ScrollLogicalPosition,
        inline: ScrollLogicalPosition,
        target_rect: Rect<Au>,
    ) -> LayoutVector2D {
        let device_pixel_ratio = self.node().owner_window().device_pixel_ratio().get();
        let to_pixel = |value: Au| value.to_nearest_pixel(device_pixel_ratio);

        // Step 1 should be handled by the caller, and provided as |target_rect|.
        // > Let target bounding border box be the box represented by the return value
        // > of invoking Element’s getBoundingClientRect(), if target is an Element,
        // > or Range’s getBoundingClientRect(), if target is a Range.
        let target_top_left = target_rect.origin.map(to_pixel).to_untyped();
        let target_bottom_right = target_rect.max().map(to_pixel);

        // The rest of the steps diverge from the specification here, but essentially try
        // to follow it using our own geometry types.
        //
        // TODO: This makes the code below wrong for the purposes of writing modes.
        let (adjusted_element_top_left, adjusted_element_bottom_right) = match self.target() {
            ScrollingBoxSource::Viewport(_) => (target_top_left, target_bottom_right),
            ScrollingBoxSource::Element(scrolling_element) => {
                let scrolling_padding_rect_top_left = scrolling_element
                    .upcast::<Node>()
                    .padding_box()
                    .unwrap_or_default()
                    .origin
                    .map(to_pixel);
                (
                    target_top_left - scrolling_padding_rect_top_left.to_vector(),
                    target_bottom_right - scrolling_padding_rect_top_left.to_vector(),
                )
            },
        };

        let size = self.size();
        let current_scroll_position = self.scroll_position();
        Vector2D::new(
            Self::calculate_scroll_position_one_axis(
                inline,
                adjusted_element_top_left.x,
                adjusted_element_bottom_right.x,
                size.width,
                current_scroll_position.x,
            ),
            Self::calculate_scroll_position_one_axis(
                block,
                adjusted_element_top_left.y,
                adjusted_element_bottom_right.y,
                size.height,
                current_scroll_position.y,
            ),
        )
    }

    /// Step 10 from <https://drafts.csswg.org/cssom-view/#determine-the-scroll-into-view-position>:
    fn calculate_scroll_position_one_axis(
        alignment: ScrollLogicalPosition,
        element_start: f32,
        element_end: f32,
        container_size: f32,
        current_scroll_offset: f32,
    ) -> f32 {
        let element_size = element_end - element_start;
        current_scroll_offset +
            match alignment {
                // Step 1 & 5: If inline is "start", then align element start edge with scrolling box start edge.
                ScrollLogicalPosition::Start => element_start,
                // Step 2 & 6: If inline is "end", then align element end edge with
                // scrolling box end edge.
                ScrollLogicalPosition::End => element_end - container_size,
                // Step 3 & 7: If inline is "center", then align the center of target bounding
                // border box with the center of scrolling box in scrolling box’s inline base direction.
                ScrollLogicalPosition::Center => {
                    element_start + (element_size - container_size) / 2.0
                },
                // Step 4 & 8: If inline is "nearest",
                ScrollLogicalPosition::Nearest => {
                    let viewport_start = current_scroll_offset;
                    let viewport_end = current_scroll_offset + container_size;

                    // Step 4.2 & 8.2: If element start edge is outside scrolling box start edge and element
                    // size is less than scrolling box size or If element end edge is outside
                    // scrolling box end edge and element size is greater than scrolling box size:
                    // Align element start edge with scrolling box start edge.
                    if (element_start < viewport_start && element_size <= container_size) ||
                        (element_end > viewport_end && element_size >= container_size)
                    {
                        element_start
                    }
                    // Step 4.3 & 8.3: If element end edge is outside scrolling box start edge and element
                    // size is greater than scrolling box size or If element start edge is outside
                    // scrolling box end edge and element size is less than scrolling box size:
                    // Align element end edge with scrolling box end edge.
                    else if (element_end > viewport_end && element_size < container_size) ||
                        (element_start < viewport_start && element_size > container_size)
                    {
                        element_end - container_size
                    }
                    // Step 4.1 & 8.1: If element start edge and element end edge are both outside scrolling
                    // box start edge and scrolling box end edge or an invalid situation: Do nothing.
                    else {
                        current_scroll_offset
                    }
                },
            }
    }
}
