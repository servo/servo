use std::cell::Cell;

use dom_struct::dom_struct;
use euclid::Rect;
use script_bindings::codegen::GenericBindings::VisualViewportBinding::VisualViewportMethods;
use script_bindings::num::Finite;
use script_bindings::root::DomRoot;
use script_bindings::script_runtime::CanGc;
use style_traits::CSSPixel;

use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::eventtarget::EventTarget;
use crate::dom::window::Window;

/// <https://drafts.csswg.org/cssom-view/#the-visualviewport-interface>
#[dom_struct]
pub(crate) struct VisualViewport {
    eventtarget: EventTarget,

    /// <https://drafts.csswg.org/cssom-view/#dom-visualviewport-offsetleft>
    offset_left: Cell<f64>,

    /// <https://drafts.csswg.org/cssom-view/#dom-visualviewport-offsettop>
    offset_top: Cell<f64>,

    /// <https://drafts.csswg.org/cssom-view/#dom-visualviewport-pageleft>
    page_left: Cell<f64>,

    /// <https://drafts.csswg.org/cssom-view/#dom-visualviewport-pagetop>
    page_top: Cell<f64>,

    /// <https://drafts.csswg.org/cssom-view/#dom-visualviewport-width>
    width: Cell<f64>,

    /// <https://drafts.csswg.org/cssom-view/#dom-visualviewport-height>
    height: Cell<f64>,

    /// <https://drafts.csswg.org/cssom-view/#dom-visualviewport-scale>
    scale: Cell<f64>,
}

impl VisualViewport {
    fn new_inherited(
        offset_left: f64,
        offset_top: f64,
        page_left: f64,
        page_top: f64,
        width: f64,
        height: f64,
        scale: f64,
    ) -> Self {
        Self {
            eventtarget: EventTarget::new_inherited(),
            offset_left: Cell::new(offset_left),
            offset_top: Cell::new(offset_top),
            page_left: Cell::new(page_left),
            page_top: Cell::new(page_top),
            width: Cell::new(width),
            height: Cell::new(height),
            scale: Cell::new(scale),
        }
    }

    /// The initial visual viewport based on a layout viewport relative to the initial containing block, where
    /// the dimension would be the same as layout viewport leaving the offset and the scale to it's default value.
    pub(crate) fn new_from_layout_viewport(
        window: &Window,
        viewport_rect: Rect<f32, CSSPixel>,
        can_gc: CanGc,
    ) -> DomRoot<Self> {
        let new_visual_viewport = Self::new_inherited(
            0.,
            0.,
            viewport_rect.min_x() as f64,
            viewport_rect.min_y() as f64,
            viewport_rect.width() as f64,
            viewport_rect.height() as f64,
            1.,
        );

        reflect_dom_object(Box::new(new_visual_viewport), window, can_gc)
    }
}

impl VisualViewportMethods<crate::DomTypeHolder> for VisualViewport {
    /// <https://drafts.csswg.org/cssom-view/#dom-visualviewport-offsetleft>
    fn OffsetLeft(&self) -> Finite<f64> {
        Finite::wrap(self.offset_left.get())
    }

    /// <https://drafts.csswg.org/cssom-view/#dom-visualviewport-offsettop>
    fn OffsetTop(&self) -> Finite<f64> {
        Finite::wrap(self.offset_top.get())
    }

    /// <https://drafts.csswg.org/cssom-view/#dom-visualviewport-pageleft>
    fn PageLeft(&self) -> Finite<f64> {
        Finite::wrap(self.page_left.get())
    }

    /// <https://drafts.csswg.org/cssom-view/#dom-visualviewport-pagetop>
    fn PageTop(&self) -> Finite<f64> {
        Finite::wrap(self.page_top.get())
    }

    /// <https://drafts.csswg.org/cssom-view/#dom-visualviewport-width>
    fn Width(&self) -> Finite<f64> {
        Finite::wrap(self.width.get())
    }

    /// <https://drafts.csswg.org/cssom-view/#dom-visualviewport-height>
    fn Height(&self) -> Finite<f64> {
        Finite::wrap(self.height.get())
    }

    /// <https://drafts.csswg.org/cssom-view/#dom-visualviewport-scale>
    fn Scale(&self) -> Finite<f64> {
        Finite::wrap(self.scale.get())
    }

    // <https://drafts.csswg.org/cssom-view/#dom-visualviewport-onresize>
    event_handler!(change, GetOnresize, SetOnresize);

    // <https://drafts.csswg.org/cssom-view/#dom-visualviewport-onscroll>
    event_handler!(scroll, GetOnscroll, SetOnscroll);

    // <https://drafts.csswg.org/cssom-view/#dom-visualviewport-onscrollend>
    event_handler!(scrollend, GetOnscrollend, SetOnscrollend);
}
