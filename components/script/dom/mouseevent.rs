/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::default::Default;

use dom_struct::dom_struct;
use embedder_traits::CompositorHitTestResult;
use euclid::Point2D;
use js::rust::HandleObject;
use keyboard_types::Modifiers;
use script_bindings::codegen::GenericBindings::WindowBinding::WindowMethods;
use servo_config::pref;
use style_traits::CSSPixel;

use crate::dom::bindings::codegen::Bindings::EventBinding::Event_Binding::EventMethods;
use crate::dom::bindings::codegen::Bindings::MouseEventBinding;
use crate::dom::bindings::codegen::Bindings::MouseEventBinding::MouseEventMethods;
use crate::dom::bindings::codegen::Bindings::UIEventBinding::UIEventMethods;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::{DomGlobal, reflect_dom_object_with_proto};
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::event::{Event, EventBubbles, EventCancelable};
use crate::dom::eventtarget::EventTarget;
use crate::dom::node::Node;
use crate::dom::uievent::UIEvent;
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

/// <https://w3c.github.io/uievents/#interface-mouseevent>
#[dom_struct]
pub(crate) struct MouseEvent {
    uievent: UIEvent,

    /// The point on the screen of where this [`MouseEvent`] was originally triggered,
    /// to use during the dispatch phase.
    ///
    /// See:
    /// <https://w3c.github.io/uievents/#dom-mouseevent-screenx>
    /// <https://w3c.github.io/uievents/#dom-mouseevent-screeny>
    #[no_trace]
    screen_point: Cell<Point2D<i32, CSSPixel>>,

    /// The point in the viewport of where this [`MouseEvent`] was originally triggered,
    /// to use during the dispatch phase.
    ///
    /// See:
    /// <https://w3c.github.io/uievents/#dom-mouseevent-clientx>
    /// <https://w3c.github.io/uievents/#dom-mouseevent-clienty>
    #[no_trace]
    client_point: Cell<Point2D<i32, CSSPixel>>,

    /// The point in the initial containing block of where this [`MouseEvent`] was
    /// originally triggered to use during the dispatch phase.
    ///
    /// See:
    /// <https://w3c.github.io/uievents/#dom-mouseevent-pagex>
    /// <https://w3c.github.io/uievents/#dom-mouseevent-pagey>
    #[no_trace]
    page_point: Cell<Point2D<i32, CSSPixel>>,

    /// The keyboard modifiers that were active when this mouse event was triggered.
    #[no_trace]
    modifiers: Cell<Modifiers>,

    /// <https://w3c.github.io/uievents/#dom-mouseevent-button>
    button: Cell<i16>,

    /// <https://w3c.github.io/uievents/#dom-mouseevent-buttons>
    buttons: Cell<u16>,

    /// <https://w3c.github.io/uievents/#dom-mouseevent-relatedtarget>
    related_target: MutNullableDom<EventTarget>,
    #[no_trace]
    point_in_target: Cell<Option<Point2D<f32, CSSPixel>>>,
}

impl MouseEvent {
    pub(crate) fn new_inherited() -> MouseEvent {
        MouseEvent {
            uievent: UIEvent::new_inherited(),
            screen_point: Cell::new(Default::default()),
            client_point: Cell::new(Default::default()),
            page_point: Cell::new(Default::default()),
            modifiers: Cell::new(Modifiers::empty()),
            button: Cell::new(0),
            buttons: Cell::new(0),
            related_target: Default::default(),
            point_in_target: Cell::new(None),
        }
    }

    pub(crate) fn new_uninitialized(window: &Window, can_gc: CanGc) -> DomRoot<MouseEvent> {
        Self::new_uninitialized_with_proto(window, None, can_gc)
    }

    fn new_uninitialized_with_proto(
        window: &Window,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<MouseEvent> {
        reflect_dom_object_with_proto(Box::new(MouseEvent::new_inherited()), window, proto, can_gc)
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        window: &Window,
        type_: DOMString,
        can_bubble: EventBubbles,
        cancelable: EventCancelable,
        view: Option<&Window>,
        detail: i32,
        screen_point: Point2D<i32, CSSPixel>,
        client_point: Point2D<i32, CSSPixel>,
        page_point: Point2D<i32, CSSPixel>,
        modifiers: Modifiers,
        button: i16,
        buttons: u16,
        related_target: Option<&EventTarget>,
        point_in_target: Option<Point2D<f32, CSSPixel>>,
        can_gc: CanGc,
    ) -> DomRoot<MouseEvent> {
        Self::new_with_proto(
            window,
            None,
            type_,
            can_bubble,
            cancelable,
            view,
            detail,
            screen_point,
            client_point,
            page_point,
            modifiers,
            button,
            buttons,
            related_target,
            point_in_target,
            can_gc,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn new_with_proto(
        window: &Window,
        proto: Option<HandleObject>,
        type_: DOMString,
        can_bubble: EventBubbles,
        cancelable: EventCancelable,
        view: Option<&Window>,
        detail: i32,
        screen_point: Point2D<i32, CSSPixel>,
        client_point: Point2D<i32, CSSPixel>,
        page_point: Point2D<i32, CSSPixel>,
        modifiers: Modifiers,
        button: i16,
        buttons: u16,
        related_target: Option<&EventTarget>,
        point_in_target: Option<Point2D<f32, CSSPixel>>,
        can_gc: CanGc,
    ) -> DomRoot<MouseEvent> {
        let ev = MouseEvent::new_uninitialized_with_proto(window, proto, can_gc);
        ev.initialize_mouse_event(
            type_,
            can_bubble,
            cancelable,
            view,
            detail,
            screen_point,
            client_point,
            page_point,
            modifiers,
            button,
            buttons,
            related_target,
            point_in_target,
        );
        ev
    }

    /// <https://w3c.github.io/uievents/#initialize-a-mouseevent>
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn initialize_mouse_event(
        &self,
        type_: DOMString,
        can_bubble: EventBubbles,
        cancelable: EventCancelable,
        view: Option<&Window>,
        detail: i32,
        screen_point: Point2D<i32, CSSPixel>,
        client_point: Point2D<i32, CSSPixel>,
        page_point: Point2D<i32, CSSPixel>,
        modifiers: Modifiers,
        button: i16,
        buttons: u16,
        related_target: Option<&EventTarget>,
        point_in_target: Option<Point2D<f32, CSSPixel>>,
    ) {
        self.uievent.initialize_ui_event(
            type_,
            view.map(|window| window.upcast::<EventTarget>()),
            can_bubble,
            cancelable,
        );

        self.uievent.set_detail(detail);
        self.screen_point.set(screen_point);
        self.client_point.set(client_point);
        self.page_point.set(page_point);
        self.modifiers.set(modifiers);
        self.button.set(button);
        self.buttons.set(buttons);
        self.related_target.set(related_target);
        self.point_in_target.set(point_in_target);
    }

    pub(crate) fn point_in_target(&self) -> Option<Point2D<f32, CSSPixel>> {
        self.point_in_target.get()
    }

    /// Create a [MouseEvent] triggered by the embedder
    pub(crate) fn for_platform_mouse_event(
        event: embedder_traits::MouseButtonEvent,
        pressed_mouse_buttons: u16,
        window: &Window,
        hit_test_result: &CompositorHitTestResult,
        modifiers: Modifiers,
        can_gc: CanGc,
    ) -> DomRoot<Self> {
        let mouse_event_type_string = match event.action {
            embedder_traits::MouseButtonAction::Click => "click",
            embedder_traits::MouseButtonAction::Up => "mouseup",
            embedder_traits::MouseButtonAction::Down => "mousedown",
        };

        let client_point = hit_test_result.point_in_viewport.to_i32();
        let page_point = hit_test_result
            .point_relative_to_initial_containing_block
            .to_i32();

        let click_count = 1;
        let mouse_event = MouseEvent::new(
            window,
            mouse_event_type_string.into(),
            EventBubbles::Bubbles,
            EventCancelable::Cancelable,
            Some(window),
            click_count,
            client_point, // TODO: Get real screen coordinates?
            client_point,
            page_point,
            modifiers,
            event.button.into(),
            pressed_mouse_buttons,
            None,
            Some(hit_test_result.point_relative_to_item),
            can_gc,
        );

        mouse_event.upcast::<Event>().set_trusted(true);
        mouse_event.upcast::<Event>().set_composed(true);

        mouse_event
    }
}

impl MouseEventMethods<crate::DomTypeHolder> for MouseEvent {
    /// <https://w3c.github.io/uievents/#dom-mouseevent-mouseevent>
    fn Constructor(
        window: &Window,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        type_: DOMString,
        init: &MouseEventBinding::MouseEventInit,
    ) -> Fallible<DomRoot<MouseEvent>> {
        let bubbles = EventBubbles::from(init.parent.parent.parent.bubbles);
        let cancelable = EventCancelable::from(init.parent.parent.parent.cancelable);
        let scroll_offset = window.scroll_offset(can_gc);
        let page_point = Point2D::new(
            scroll_offset.x as i32 + init.clientX,
            scroll_offset.y as i32 + init.clientY,
        );
        let event = MouseEvent::new_with_proto(
            window,
            proto,
            type_,
            bubbles,
            cancelable,
            init.parent.parent.view.as_deref(),
            init.parent.parent.detail,
            Point2D::new(init.screenX, init.screenY),
            Point2D::new(init.clientX, init.clientY),
            page_point,
            init.parent.modifiers(),
            init.button,
            init.buttons,
            init.relatedTarget.as_deref(),
            None,
            can_gc,
        );
        event
            .upcast::<Event>()
            .set_composed(init.parent.parent.parent.composed);
        Ok(event)
    }

    /// <https://w3c.github.io/uievents/#widl-MouseEvent-screenX>
    fn ScreenX(&self) -> i32 {
        self.screen_point.get().x
    }

    /// <https://w3c.github.io/uievents/#widl-MouseEvent-screenY>
    fn ScreenY(&self) -> i32 {
        self.screen_point.get().y
    }

    /// <https://w3c.github.io/uievents/#widl-MouseEvent-clientX>
    fn ClientX(&self) -> i32 {
        self.client_point.get().x
    }

    /// <https://w3c.github.io/uievents/#widl-MouseEvent-clientY>
    fn ClientY(&self) -> i32 {
        self.client_point.get().y
    }

    /// <https://drafts.csswg.org/cssom-view/#dom-mouseevent-pagex>
    fn PageX(&self) -> i32 {
        // The pageX attribute must follow these steps:
        // > 1. If the event’s dispatch flag is set, return the horizontal coordinate of the
        // > position where the event occurred relative to the origin of the initial containing
        // > block and terminate these steps.
        if self.upcast::<Event>().dispatching() {
            return self.page_point.get().x;
        }

        // > 2. Let offset be the value of the scrollX attribute of the event’s associated
        // > Window object, if there is one, or zero otherwise.
        // > 3. Return the sum of offset and the value of the event’s clientX attribute.
        self.global().as_window().ScrollX() + self.ClientX()
    }

    /// <https://drafts.csswg.org/cssom-view/#dom-mouseevent-pagey>
    fn PageY(&self) -> i32 {
        // The pageY attribute must follow these steps:
        // > 1. If the event’s dispatch flag is set, return the vertical coordinate of the
        // > position where the event occurred relative to the origin of the initial
        // > containing block and terminate these steps.
        if self.upcast::<Event>().dispatching() {
            return self.page_point.get().y;
        }

        // > 2. Let offset be the value of the scrollY attribute of the event’s associated
        // > Window object, if there is one, or zero otherwise.
        // > 3. Return the sum of offset and the value of the event’s clientY attribute.
        self.global().as_window().ScrollY() + self.ClientY()
    }

    /// <https://drafts.csswg.org/cssom-view/#dom-mouseevent-x>
    fn X(&self) -> i32 {
        self.ClientX()
    }

    /// <https://drafts.csswg.org/cssom-view/#dom-mouseevent-y>
    fn Y(&self) -> i32 {
        self.ClientY()
    }

    /// <https://drafts.csswg.org/cssom-view/#dom-mouseevent-offsetx>
    fn OffsetX(&self, can_gc: CanGc) -> i32 {
        // > The offsetX attribute must follow these steps:
        // > 1. If the event’s dispatch flag is set, return the x-coordinate of the position
        // >    where the event occurred relative to the origin of the padding edge of the
        // >    target node, ignoring the transforms that apply to the element and its
        // >    ancestors, and terminate these steps.
        let event = self.upcast::<Event>();
        if event.dispatching() {
            let Some(target) = event.GetTarget() else {
                return 0;
            };
            let Some(node) = target.downcast::<Node>() else {
                return 0;
            };
            return self.ClientX() - node.client_rect(can_gc).origin.x;
        }

        // > 2. Return the value of the event’s pageX attribute.
        self.PageX()
    }

    /// <https://drafts.csswg.org/cssom-view/#dom-mouseevent-offsety>
    fn OffsetY(&self, can_gc: CanGc) -> i32 {
        // > The offsetY attribute must follow these steps:
        // > 1. If the event’s dispatch flag is set, return the y-coordinate of the
        // >    position where the event occurred relative to the origin of the padding edge of
        // >    the target node, ignoring the transforms that apply to the element and its
        // >    ancestors, and terminate these steps.
        let event = self.upcast::<Event>();
        if event.dispatching() {
            let Some(target) = event.GetTarget() else {
                return 0;
            };
            let Some(node) = target.downcast::<Node>() else {
                return 0;
            };
            return self.ClientY() - node.client_rect(can_gc).origin.y;
        }

        // 2. Return the value of the event’s pageY attribute.
        self.PageY()
    }

    /// <https://w3c.github.io/uievents/#dom-mouseevent-ctrlkey>
    fn CtrlKey(&self) -> bool {
        self.modifiers.get().contains(Modifiers::CONTROL)
    }

    /// <https://w3c.github.io/uievents/#dom-mouseevent-shiftkey>
    fn ShiftKey(&self) -> bool {
        self.modifiers.get().contains(Modifiers::SHIFT)
    }

    /// <https://w3c.github.io/uievents/#dom-mouseevent-altkey>
    fn AltKey(&self) -> bool {
        self.modifiers.get().contains(Modifiers::ALT)
    }

    /// <https://w3c.github.io/uievents/#dom-mouseevent-metakey>
    fn MetaKey(&self) -> bool {
        self.modifiers.get().contains(Modifiers::META)
    }

    /// <https://w3c.github.io/uievents/#dom-mouseevent-button>
    fn Button(&self) -> i16 {
        self.button.get()
    }

    /// <https://w3c.github.io/uievents/#dom-mouseevent-buttons>
    fn Buttons(&self) -> u16 {
        self.buttons.get()
    }

    /// <https://w3c.github.io/uievents/#widl-MouseEvent-relatedTarget>
    fn GetRelatedTarget(&self) -> Option<DomRoot<EventTarget>> {
        self.related_target.get()
    }

    // See discussion at:
    //  - https://github.com/servo/servo/issues/6643
    //  - https://bugzilla.mozilla.org/show_bug.cgi?id=1186125
    // This returns the same result as current gecko.
    // https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/which
    fn Which(&self) -> i32 {
        if pref!(dom_mouse_event_which_enabled) {
            (self.button.get() + 1) as i32
        } else {
            0
        }
    }

    /// <https://w3c.github.io/uievents/#widl-MouseEvent-initMouseEvent>
    fn InitMouseEvent(
        &self,
        type_arg: DOMString,
        can_bubble_arg: bool,
        cancelable_arg: bool,
        view_arg: Option<&Window>,
        detail_arg: i32,
        screen_x_arg: i32,
        screen_y_arg: i32,
        client_x_arg: i32,
        client_y_arg: i32,
        ctrl_key_arg: bool,
        alt_key_arg: bool,
        shift_key_arg: bool,
        meta_key_arg: bool,
        button_arg: i16,
        related_target_arg: Option<&EventTarget>,
        can_gc: CanGc,
    ) {
        if self.upcast::<Event>().dispatching() {
            return;
        }

        self.upcast::<UIEvent>().InitUIEvent(
            type_arg,
            can_bubble_arg,
            cancelable_arg,
            view_arg,
            detail_arg,
        );
        self.screen_point
            .set(Point2D::new(screen_x_arg, screen_y_arg));
        self.client_point
            .set(Point2D::new(client_x_arg, client_y_arg));

        let global = self.global();
        let scroll_offset = global.as_window().scroll_offset(can_gc);
        self.page_point.set(Point2D::new(
            scroll_offset.x as i32 + client_x_arg,
            scroll_offset.y as i32 + client_y_arg,
        ));

        let mut modifiers = Modifiers::empty();
        if ctrl_key_arg {
            modifiers.insert(Modifiers::CONTROL);
        }
        if alt_key_arg {
            modifiers.insert(Modifiers::ALT);
        }
        if shift_key_arg {
            modifiers.insert(Modifiers::SHIFT);
        }
        if meta_key_arg {
            modifiers.insert(Modifiers::META);
        }
        self.modifiers.set(modifiers);

        self.button.set(button_arg);
        self.related_target.set(related_target_arg);
    }

    /// <https://dom.spec.whatwg.org/#dom-event-istrusted>
    fn IsTrusted(&self) -> bool {
        self.uievent.IsTrusted()
    }
}
