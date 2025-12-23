/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use dom_struct::dom_struct;
use euclid::{Rect, Scale, Size2D};
use script_bindings::codegen::GenericBindings::VisualViewportBinding::VisualViewportMethods;
use script_bindings::codegen::GenericBindings::WindowBinding::WindowMethods;
use script_bindings::num::Finite;
use script_bindings::root::{Dom, DomRoot};
use script_bindings::script_runtime::CanGc;
use style_traits::CSSPixel;
use webrender_api::units::DevicePixel;

use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::eventtarget::EventTarget;
use crate::dom::window::Window;

/// <https://drafts.csswg.org/cssom-view/#the-visualviewport-interface>
#[dom_struct]
pub(crate) struct VisualViewport {
    eventtarget: EventTarget,

    /// The associated [`Window`] for this [`VisualViewport`],
    window: Dom<Window>,

    /// The rectangle bound of the [`VisualViewport`], relative to the layout viewport.
    #[no_trace]
    viewport_rect: Cell<Rect<f32, CSSPixel>>,

    /// The scale factor of [`VisualViewport`], which is also commonly known as pinch-zoom.
    /// <https://drafts.csswg.org/cssom-view/#scale-factor>
    #[no_trace]
    scale: Cell<Scale<f32, DevicePixel, DevicePixel>>,
}

impl VisualViewport {
    fn new_inherited(
        window: &Window,
        viewport_rect: Rect<f32, CSSPixel>,
        scale: Scale<f32, DevicePixel, DevicePixel>,
    ) -> Self {
        Self {
            eventtarget: EventTarget::new_inherited(),
            window: Dom::from_ref(window),
            viewport_rect: Cell::new(viewport_rect),
            scale: Cell::new(scale),
        }
    }

    /// The initial visual viewport based on a layout viewport relative to the initial containing block, where
    /// the dimension would be the same as layout viewport leaving the offset and the scale to its default value.
    pub(crate) fn new_from_layout_viewport(
        window: &Window,
        viewport_size: Size2D<f32, CSSPixel>,
        can_gc: CanGc,
    ) -> DomRoot<Self> {
        reflect_dom_object(
            Box::new(Self::new_inherited(
                window,
                Rect::from_size(viewport_size),
                Scale::identity(),
            )),
            window,
            can_gc,
        )
    }
}

impl VisualViewportMethods<crate::DomTypeHolder> for VisualViewport {
    /// <https://drafts.csswg.org/cssom-view/#dom-visualviewport-offsetleft>
    fn OffsetLeft(&self) -> Finite<f64> {
        // > 1. If the visual viewport’s associated document is not fully active, return 0.
        if !self.window.Document().is_fully_active() {
            return Finite::wrap(0.);
        }

        // > 2. Otherwise, return the offset of the left edge of the visual viewport from the left edge of the
        // >    layout viewport.
        Finite::wrap(self.viewport_rect.get().min_x() as f64)
    }

    /// <https://drafts.csswg.org/cssom-view/#dom-visualviewport-offsettop>
    fn OffsetTop(&self) -> Finite<f64> {
        // > 1. If the visual viewport’s associated document is not fully active, return 0.
        if !self.window.Document().is_fully_active() {
            return Finite::wrap(0.);
        }

        // > 2. Otherwise, return the offset of the top edge of the visual viewport from the top edge of the
        // >    layout viewport.
        Finite::wrap(self.viewport_rect.get().min_y() as f64)
    }

    /// <https://drafts.csswg.org/cssom-view/#dom-visualviewport-pageleft>
    fn PageLeft(&self) -> Finite<f64> {
        // > 1. If the visual viewport’s associated document is not fully active, return 0.
        if !self.window.Document().is_fully_active() {
            return Finite::wrap(0.);
        }

        // > 2. Otherwise, return the offset of the left edge of the visual viewport from the left edge of the
        // >    initial containing block of the layout viewport’s document.
        let page_left = self.viewport_rect.get().min_x() + self.window.scroll_offset().x;
        Finite::wrap(page_left as f64)
    }

    /// <https://drafts.csswg.org/cssom-view/#dom-visualviewport-pagetop>
    fn PageTop(&self) -> Finite<f64> {
        // > 1. If the visual viewport’s associated document is not fully active, return 0.
        if !self.window.Document().is_fully_active() {
            return Finite::wrap(0.);
        }

        // > 2. Otherwise, return the offset of the top edge of the visual viewport from the top edge of the
        // >    initial containing block of the layout viewport’s document.
        let page_top = self.viewport_rect.get().min_y() + self.window.scroll_offset().y;
        Finite::wrap(page_top as f64)
    }

    /// <https://drafts.csswg.org/cssom-view/#dom-visualviewport-width>
    fn Width(&self) -> Finite<f64> {
        // > 1. If the visual viewport’s associated document is not fully active, return 0.
        if !self.window.Document().is_fully_active() {
            return Finite::wrap(0.);
        }

        // > 2. Otherwise, return the width of the visual viewport excluding the width of any rendered vertical
        // >    classic scrollbar that is fixed to the visual viewport.
        // TODO(#41341): when classic scrollbar is implemented, exclude it's size from visual viewport width.
        Finite::wrap(self.viewport_rect.get().width() as f64)
    }

    /// <https://drafts.csswg.org/cssom-view/#dom-visualviewport-height>
    fn Height(&self) -> Finite<f64> {
        // > 1. If the visual viewport’s associated document is not fully active, return 0.
        if !self.window.Document().is_fully_active() {
            return Finite::wrap(0.);
        }

        // > 2. Otherwise, return the height of the visual viewport excluding the height of any rendered horizontal
        // >    classic scrollbar that is fixed to the visual viewport.
        // TODO(#41341): when classic scrollbar is implemented, exclude it's size from visual viewport height.
        Finite::wrap(self.viewport_rect.get().height() as f64)
    }

    /// <https://drafts.csswg.org/cssom-view/#dom-visualviewport-scale>
    fn Scale(&self) -> Finite<f64> {
        // > 1. If the visual viewport’s associated document is not fully active, return 0.
        if !self.window.Document().is_fully_active() {
            return Finite::wrap(0.);
        }

        // > 2. If there is no output device, return 1 and abort these steps.
        // TODO(#41341): check for output device.

        // > 3. Otherwise, return the visual viewport’s scale factor.
        Finite::wrap(self.scale.get().get() as f64)
    }

    // <https://drafts.csswg.org/cssom-view/#dom-visualviewport-onresize>
    event_handler!(resize, GetOnresize, SetOnresize);

    // <https://drafts.csswg.org/cssom-view/#dom-visualviewport-onscroll>
    event_handler!(scroll, GetOnscroll, SetOnscroll);

    // <https://drafts.csswg.org/cssom-view/#dom-visualviewport-onscrollend>
    event_handler!(scrollend, GetOnscrollend, SetOnscrollend);
}
