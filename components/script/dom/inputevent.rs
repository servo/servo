/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use embedder_traits::Cursor;
use euclid::Point2D;
use js::rust::HandleObject;
use style_traits::CSSPixel;

use crate::dom::bindings::codegen::Bindings::InputEventBinding::{self, InputEventMethods};
use crate::dom::bindings::codegen::Bindings::UIEventBinding::UIEvent_Binding::UIEventMethods;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::reflector::reflect_dom_object_with_proto;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::datatransfer::DataTransfer;
use crate::dom::node::Node;
use crate::dom::staticrange::StaticRange;
use crate::dom::uievent::UIEvent;
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct InputEvent {
    uievent: UIEvent,
    data: Option<DOMString>,
    is_composing: bool,
    input_type: DOMString,
}

impl InputEvent {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        window: &Window,
        proto: Option<HandleObject>,
        type_: DOMString,
        can_bubble: bool,
        cancelable: bool,
        view: Option<&Window>,
        detail: i32,
        data: Option<DOMString>,
        is_composing: bool,
        input_type: DOMString,
        can_gc: CanGc,
    ) -> DomRoot<InputEvent> {
        let ev = reflect_dom_object_with_proto(
            Box::new(InputEvent {
                uievent: UIEvent::new_inherited(),
                data,
                is_composing,
                input_type,
            }),
            window,
            proto,
            can_gc,
        );
        ev.uievent
            .InitUIEvent(type_, can_bubble, cancelable, view, detail);
        ev
    }
}

impl InputEventMethods<crate::DomTypeHolder> for InputEvent {
    /// <https://w3c.github.io/uievents/#dom-inputevent-inputevent>
    fn Constructor(
        window: &Window,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        type_: DOMString,
        init: &InputEventBinding::InputEventInit,
    ) -> Fallible<DomRoot<InputEvent>> {
        let event = InputEvent::new(
            window,
            proto,
            type_,
            init.parent.parent.bubbles,
            init.parent.parent.cancelable,
            init.parent.view.as_deref(),
            init.parent.detail,
            init.data.clone(),
            init.isComposing,
            init.inputType.clone(),
            can_gc,
        );
        Ok(event)
    }

    /// <https://w3c.github.io/uievents/#dom-inputevent-data>
    fn GetData(&self) -> Option<DOMString> {
        self.data.clone()
    }

    /// <https://w3c.github.io/uievents/#dom-inputevent-iscomposing>
    fn IsComposing(&self) -> bool {
        self.is_composing
    }

    // https://w3c.github.io/uievents/#dom-inputevent-inputtype
    fn InputType(&self) -> DOMString {
        self.input_type.clone()
    }

    // https://w3c.github.io/input-events/#dom-inputevent-datatransfer
    // TODO: Populate dataTransfer for contenteditable
    fn GetDataTransfer(&self) -> Option<DomRoot<DataTransfer>> {
        None
    }

    // https://w3c.github.io/input-events/#dom-inputevent-gettargetranges
    // TODO: Populate targetRanges for contenteditable
    fn GetTargetRanges(&self) -> Vec<DomRoot<StaticRange>> {
        Vec::new()
    }

    /// <https://dom.spec.whatwg.org/#dom-event-istrusted>
    fn IsTrusted(&self) -> bool {
        self.uievent.IsTrusted()
    }
}

/// A [`HitTestResult`] that is the result of doing a hit test based on a less-fine-grained
/// `CompositorHitTestResult` against our current layout.
pub(crate) struct HitTestResult {
    pub node: DomRoot<Node>,
    pub cursor: Cursor,
    pub point_in_node: Point2D<f32, CSSPixel>,
    pub point_in_frame: Point2D<f32, CSSPixel>,
    pub point_relative_to_initial_containing_block: Point2D<f32, CSSPixel>,
}
