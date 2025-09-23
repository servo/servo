/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use layout_api::{AxesOverflow, ScrollContainerQueryFlags};
use script_bindings::codegen::GenericBindings::WindowBinding::ScrollBehavior;
use script_bindings::inheritance::Castable;
use script_bindings::root::DomRoot;
use style::values::computed::Overflow;
use webrender_api::units::{LayoutSize, LayoutVector2D};

use crate::dom::node::{Node, NodeTraits};
use crate::dom::types::{Document, Element};

pub(crate) struct ScrollingBox {
    target: ScrollingBoxSource,
    cached_content_size: Cell<Option<LayoutSize>>,
    cached_size: Cell<Option<LayoutSize>>,
}

/// Represents a scrolling box that can be either an element or the viewport
/// <https://drafts.csswg.org/cssom-view/#scrolling-box>
pub(crate) enum ScrollingBoxSource {
    Element(DomRoot<Element>, AxesOverflow),
    Viewport(DomRoot<Document>),
}

#[derive(Copy, Clone)]
pub(crate) enum ScrollingBoxAxis {
    X,
    Y,
}

impl ScrollingBox {
    pub(crate) fn new(target: ScrollingBoxSource) -> Self {
        Self {
            target,
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
            ScrollingBoxSource::Element(element, _) => element
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
            ScrollingBoxSource::Element(element, _) => {
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
            ScrollingBoxSource::Element(element, _) => {
                element.client_rect().size.to_f32().cast_unit()
            },
            ScrollingBoxSource::Viewport(document) => {
                document.window().viewport_details().size.cast_unit()
            },
        };
        self.cached_size.set(Some(size));
        size
    }

    pub(crate) fn parent(&self) -> Option<ScrollingBox> {
        match &self.target {
            ScrollingBoxSource::Element(element, _) => {
                element.scrolling_box(ScrollContainerQueryFlags::empty())
            },
            ScrollingBoxSource::Viewport(_) => None,
        }
    }

    pub(crate) fn node(&self) -> &Node {
        match &self.target {
            ScrollingBoxSource::Element(element, _) => element.upcast(),
            ScrollingBoxSource::Viewport(document) => document.upcast(),
        }
    }

    pub(crate) fn scroll_to(&self, position: LayoutVector2D, behavior: ScrollBehavior) {
        match &self.target {
            ScrollingBoxSource::Element(element, _) => {
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
        let axes_overflow = match &self.target {
            ScrollingBoxSource::Element(_, axes_overflow) => axes_overflow,
            ScrollingBoxSource::Viewport(_) => return true,
        };
        let overflow = match axis {
            ScrollingBoxAxis::X => axes_overflow.x,
            ScrollingBoxAxis::Y => axes_overflow.x,
        };
        if !overflow.is_scrollable() || overflow == Overflow::Hidden {
            return false;
        }
        match axis {
            ScrollingBoxAxis::X => self.content_size().width > self.size().width,
            ScrollingBoxAxis::Y => self.content_size().height > self.size().height,
        }
    }
}
