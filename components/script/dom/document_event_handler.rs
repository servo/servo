/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::array::from_ref;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::f64::consts::PI;
use std::mem;
use std::rc::Rc;
use std::time::{Duration, Instant};

use base::generic_channel::GenericCallback;
use constellation_traits::{KeyboardScroll, ScriptToConstellationMessage};
use embedder_traits::{
    Cursor, EditingActionEvent, EmbedderMsg, ImeEvent, InputEvent, InputEventAndId,
    InputEventResult, KeyboardEvent as EmbedderKeyboardEvent, MouseButton, MouseButtonAction,
    MouseButtonEvent, MouseLeftViewportEvent, ScrollEvent, TouchEvent as EmbedderTouchEvent,
    TouchEventType, TouchId, UntrustedNodeAddress, WheelEvent as EmbedderWheelEvent,
};
#[cfg(feature = "gamepad")]
use embedder_traits::{
    GamepadEvent as EmbedderGamepadEvent, GamepadSupportedHapticEffects, GamepadUpdateType,
};
use euclid::{Point2D, Vector2D};
use js::jsapi::JSAutoRealm;
use keyboard_types::{Code, Key, KeyState, Modifiers, NamedKey};
use layout_api::{ScrollContainerQueryFlags, node_id_from_scroll_id};
use script_bindings::codegen::GenericBindings::DocumentBinding::DocumentMethods;
use script_bindings::codegen::GenericBindings::EventBinding::EventMethods;
use script_bindings::codegen::GenericBindings::NavigatorBinding::NavigatorMethods;
use script_bindings::codegen::GenericBindings::NodeBinding::NodeMethods;
use script_bindings::codegen::GenericBindings::PerformanceBinding::PerformanceMethods;
use script_bindings::codegen::GenericBindings::TouchBinding::TouchMethods;
use script_bindings::codegen::GenericBindings::WindowBinding::{ScrollBehavior, WindowMethods};
use script_bindings::inheritance::Castable;
use script_bindings::match_domstring_ascii;
use script_bindings::num::Finite;
use script_bindings::reflector::DomObject;
use script_bindings::root::{Dom, DomRoot, DomSlice};
use script_bindings::script_runtime::CanGc;
use script_bindings::str::DOMString;
use script_traits::ConstellationInputEvent;
use servo_config::pref;
use style_traits::CSSPixel;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::root::MutNullableDom;
use crate::dom::clipboardevent::ClipboardEventType;
use crate::dom::document::{FireMouseEventType, FocusInitiator};
use crate::dom::event::{EventBubbles, EventCancelable, EventComposed, EventFlags};
#[cfg(feature = "gamepad")]
use crate::dom::gamepad::gamepad::{Gamepad, contains_user_gesture};
#[cfg(feature = "gamepad")]
use crate::dom::gamepad::gamepadevent::GamepadEventType;
use crate::dom::inputevent::HitTestResult;
use crate::dom::node::{self, Node, NodeTraits, ShadowIncluding};
use crate::dom::pointerevent::{PointerEvent, PointerId};
use crate::dom::scrolling_box::ScrollingBoxAxis;
use crate::dom::types::{
    ClipboardEvent, CompositionEvent, DataTransfer, Element, Event, EventTarget, GlobalScope,
    HTMLAnchorElement, KeyboardEvent, MouseEvent, Touch, TouchEvent, TouchList, WheelEvent, Window,
};
use crate::drag_data_store::{DragDataStore, Kind, Mode};
use crate::realms::enter_realm;

/// A data structure used for tracking the current click count. This can be
/// reset to 0 if a mouse button event happens at a sufficient distance or time
/// from the previous one.
///
/// From <https://w3c.github.io/uievents/#current-click-count>:
/// > Implementations MUST maintain the current click count when generating mouse
/// > events. This MUST be a non-negative integer indicating the number of consecutive
/// > clicks of a pointing device button within a specific time. The delay after which
/// > the count resets is specific to the environment configuration.
#[derive(Default, JSTraceable, MallocSizeOf)]
struct ClickCountingInfo {
    time: Option<Instant>,
    #[no_trace]
    point: Option<Point2D<f32, CSSPixel>>,
    #[no_trace]
    button: Option<MouseButton>,
    count: usize,
}

impl ClickCountingInfo {
    fn reset_click_count_if_necessary(
        &mut self,
        button: MouseButton,
        point_in_frame: Point2D<f32, CSSPixel>,
    ) {
        let (Some(previous_button), Some(previous_point), Some(previous_time)) =
            (self.button, self.point, self.time)
        else {
            assert_eq!(self.count, 0);
            return;
        };

        let double_click_timeout =
            Duration::from_millis(pref!(dom_document_dblclick_timeout) as u64);
        let double_click_distance_threshold = pref!(dom_document_dblclick_dist) as u64;

        // Calculate distance between this click and the previous click.
        let line = point_in_frame - previous_point;
        let distance = (line.dot(line) as f64).sqrt();
        if previous_button != button ||
            Instant::now().duration_since(previous_time) > double_click_timeout ||
            distance > double_click_distance_threshold as f64
        {
            self.count = 0;
            self.time = None;
            self.point = None;
        }
    }

    fn increment_click_count(
        &mut self,
        button: MouseButton,
        point: Point2D<f32, CSSPixel>,
    ) -> usize {
        self.time = Some(Instant::now());
        self.point = Some(point);
        self.button = Some(button);
        self.count += 1;
        self.count
    }
}

/// The [`DocumentEventHandler`] is a structure responsible for handling input events for
/// the [`crate::Document`] and storing data related to event handling. It exists to
/// decrease the size of the [`crate::Document`] structure.
#[derive(JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) struct DocumentEventHandler {
    /// The [`Window`] element for this [`DocumentEventHandler`].
    window: Dom<Window>,
    /// Pending input events, to be handled at the next rendering opportunity.
    #[no_trace]
    #[ignore_malloc_size_of = "InputEvent contains data from outside crates"]
    pending_input_events: DomRefCell<Vec<ConstellationInputEvent>>,
    /// The index of the last mouse move event in the pending input events queue.
    mouse_move_event_index: DomRefCell<Option<usize>>,
    /// <https://w3c.github.io/uievents/#event-type-dblclick>
    click_counting_info: DomRefCell<ClickCountingInfo>,
    #[no_trace]
    last_mouse_button_down_point: Cell<Option<Point2D<f32, CSSPixel>>>,
    /// The number of currently down buttons, used to decide which kind
    /// of pointer event to dispatch on MouseDown/MouseUp.
    down_button_count: Cell<u32>,
    /// The element that is currently hovered by the cursor.
    current_hover_target: MutNullableDom<Element>,
    /// The element that was most recently clicked.
    most_recently_clicked_element: MutNullableDom<Element>,
    /// The most recent mouse movement point, used for processing `mouseleave` events.
    #[no_trace]
    most_recent_mousemove_point: Cell<Option<Point2D<f32, CSSPixel>>>,
    /// The currently set [`Cursor`] or `None` if the `Document` isn't being hovered
    /// by the cursor.
    #[no_trace]
    current_cursor: Cell<Option<Cursor>>,
    /// <http://w3c.github.io/touch-events/#dfn-active-touch-point>
    active_touch_points: DomRefCell<Vec<Dom<Touch>>>,
    /// The active keyboard modifiers for the WebView. This is updated when receiving any input event.
    #[no_trace]
    active_keyboard_modifiers: Cell<Modifiers>,
    /// Map from touch identifier to pointer ID for active touch points
    active_pointer_ids: DomRefCell<HashMap<i32, i32>>,
    /// Counter for generating unique pointer IDs for touch inputs
    next_touch_pointer_id: Cell<i32>,
}

impl DocumentEventHandler {
    pub(crate) fn new(window: &Window) -> Self {
        Self {
            window: Dom::from_ref(window),
            pending_input_events: Default::default(),
            mouse_move_event_index: Default::default(),
            click_counting_info: Default::default(),
            last_mouse_button_down_point: Default::default(),
            down_button_count: Cell::new(0),
            current_hover_target: Default::default(),
            most_recently_clicked_element: Default::default(),
            most_recent_mousemove_point: Default::default(),
            current_cursor: Default::default(),
            active_touch_points: Default::default(),
            active_keyboard_modifiers: Default::default(),
            active_pointer_ids: Default::default(),
            next_touch_pointer_id: Cell::new(1),
        }
    }

    /// Note a pending input event, to be processed at the next `update_the_rendering` task.
    pub(crate) fn note_pending_input_event(&self, event: ConstellationInputEvent) {
        let mut pending_input_events = self.pending_input_events.borrow_mut();
        if matches!(event.event.event, InputEvent::MouseMove(..)) {
            // First try to replace any existing mouse move event.
            if let Some(mouse_move_event) = self
                .mouse_move_event_index
                .borrow()
                .and_then(|index| pending_input_events.get_mut(index))
            {
                *mouse_move_event = event;
                return;
            }

            *self.mouse_move_event_index.borrow_mut() = Some(pending_input_events.len());
        }

        pending_input_events.push(event);
    }

    /// Whether or not this [`Document`] has any pending input events to be processed during
    /// "update the rendering."
    pub(crate) fn has_pending_input_events(&self) -> bool {
        !self.pending_input_events.borrow().is_empty()
    }

    pub(crate) fn alternate_action_keyboard_modifier_active(&self) -> bool {
        #[cfg(target_os = "macos")]
        {
            self.active_keyboard_modifiers
                .get()
                .contains(Modifiers::META)
        }
        #[cfg(not(target_os = "macos"))]
        {
            self.active_keyboard_modifiers
                .get()
                .contains(Modifiers::CONTROL)
        }
    }

    pub(crate) fn handle_pending_input_events(&self, can_gc: CanGc) {
        let _realm = enter_realm(&*self.window);

        // Reset the mouse event index.
        *self.mouse_move_event_index.borrow_mut() = None;
        let pending_input_events = mem::take(&mut *self.pending_input_events.borrow_mut());

        for event in pending_input_events {
            self.active_keyboard_modifiers
                .set(event.active_keyboard_modifiers);

            // TODO: For some of these we still aren't properly calculating whether or not
            // the event was handled or if `preventDefault()` was called on it. Each of
            // these cases needs to be examined and some of them either fire more than one
            // event or fire events later. We have to make a good decision about what to
            // return to the embedder when that happens.
            let result = match event.event.event.clone() {
                InputEvent::MouseButton(mouse_button_event) => {
                    self.handle_native_mouse_button_event(mouse_button_event, &event, can_gc);
                    InputEventResult::default()
                },
                InputEvent::MouseMove(_) => {
                    self.handle_native_mouse_move_event(&event, can_gc);
                    InputEventResult::default()
                },
                InputEvent::MouseLeftViewport(mouse_leave_event) => {
                    self.handle_mouse_left_viewport_event(&event, &mouse_leave_event, can_gc);
                    InputEventResult::default()
                },
                InputEvent::Touch(touch_event) => {
                    self.handle_touch_event(touch_event, &event, can_gc)
                },
                InputEvent::Wheel(wheel_event) => {
                    self.handle_wheel_event(wheel_event, &event, can_gc)
                },
                InputEvent::Keyboard(keyboard_event) => {
                    self.handle_keyboard_event(keyboard_event, can_gc)
                },
                InputEvent::Ime(ime_event) => self.handle_ime_event(ime_event, can_gc),
                #[cfg(feature = "gamepad")]
                InputEvent::Gamepad(gamepad_event) => {
                    self.handle_gamepad_event(gamepad_event);
                    InputEventResult::default()
                },
                InputEvent::EditingAction(editing_action_event) => {
                    self.handle_editing_action(None, editing_action_event, can_gc)
                },
                InputEvent::Scroll(scroll_event) => {
                    self.handle_embedder_scroll_event(scroll_event);
                    InputEventResult::default()
                },
            };

            self.notify_embedder_that_event_was_handled(event.event, result);
        }
    }

    fn notify_embedder_that_event_was_handled(
        &self,
        event: InputEventAndId,
        result: InputEventResult,
    ) {
        // Wait to to notify the embedder that the vent was handled until all pending DOM
        // event processing is finished.
        let id = event.id;
        let trusted_window = Trusted::new(&*self.window);
        self.window
            .as_global_scope()
            .task_manager()
            .dom_manipulation_task_source()
            .queue(task!(notify_webdriver_input_event_completed: move || {
                let window = trusted_window.root();
                window.send_to_embedder(
                    EmbedderMsg::InputEventHandled(window.webview_id(), id, result));
            }));
    }

    pub(crate) fn set_cursor(&self, cursor: Option<Cursor>) {
        if cursor == self.current_cursor.get() {
            return;
        }
        self.current_cursor.set(cursor);
        self.window.send_to_embedder(EmbedderMsg::SetCursor(
            self.window.webview_id(),
            cursor.unwrap_or_default(),
        ));
    }

    fn handle_mouse_left_viewport_event(
        &self,
        input_event: &ConstellationInputEvent,
        mouse_leave_event: &MouseLeftViewportEvent,
        can_gc: CanGc,
    ) {
        if let Some(current_hover_target) = self.current_hover_target.get() {
            let current_hover_target = current_hover_target.upcast::<Node>();
            for element in current_hover_target
                .inclusive_ancestors(ShadowIncluding::Yes)
                .filter_map(DomRoot::downcast::<Element>)
            {
                element.set_hover_state(false);
                element.set_active_state(false);
            }

            if let Some(hit_test_result) = self
                .most_recent_mousemove_point
                .get()
                .and_then(|point| self.window.hit_test_from_point_in_viewport(point))
            {
                MouseEvent::new_for_platform_motion_event(
                    &self.window,
                    FireMouseEventType::Out,
                    &hit_test_result,
                    input_event,
                    can_gc,
                )
                .upcast::<Event>()
                .fire(current_hover_target.upcast(), can_gc);
                self.handle_mouse_enter_leave_event(
                    DomRoot::from_ref(current_hover_target),
                    None,
                    FireMouseEventType::Leave,
                    &hit_test_result,
                    input_event,
                    can_gc,
                );
            }
        }

        // We do not want to always inform the embedder that cursor has been set to the
        // default cursor, in order to avoid a timing issue when moving between `<iframe>`
        // elements. There is currently no way to control which `SetCursor` message will
        // reach the embedder first. This is safer when leaving the `WebView` entirely.
        if !mouse_leave_event.focus_moving_to_another_iframe {
            // If focus is moving to another frame, it will decide what the new status
            // text is, but if this mouse leave event is leaving the WebView entirely,
            // then clear it.
            self.window
                .send_to_embedder(EmbedderMsg::Status(self.window.webview_id(), None));
            self.set_cursor(None);
        } else {
            self.current_cursor.set(None);
        }

        self.current_hover_target.set(None);
        self.most_recent_mousemove_point.set(None);
    }

    fn handle_mouse_enter_leave_event(
        &self,
        event_target: DomRoot<Node>,
        related_target: Option<DomRoot<Node>>,
        event_type: FireMouseEventType,
        hit_test_result: &HitTestResult,
        input_event: &ConstellationInputEvent,
        can_gc: CanGc,
    ) {
        assert!(matches!(
            event_type,
            FireMouseEventType::Enter | FireMouseEventType::Leave
        ));

        let common_ancestor = match related_target.as_ref() {
            Some(related_target) => event_target
                .common_ancestor(related_target, ShadowIncluding::Yes)
                .unwrap_or_else(|| DomRoot::from_ref(&*event_target)),
            None => DomRoot::from_ref(&*event_target),
        };

        // We need to create a target chain in case the event target shares
        // its boundaries with its ancestors.
        let mut targets = vec![];
        let mut current = Some(event_target);
        while let Some(node) = current {
            if node == common_ancestor {
                break;
            }
            current = node.GetParentNode();
            targets.push(node);
        }

        // The order for dispatching mouseenter events starts from the topmost
        // common ancestor of the event target and the related target.
        if event_type == FireMouseEventType::Enter {
            targets = targets.into_iter().rev().collect();
        }

        for target in targets {
            MouseEvent::new_for_platform_motion_event(
                &self.window,
                event_type,
                hit_test_result,
                input_event,
                can_gc,
            )
            .upcast::<Event>()
            .fire(target.upcast(), can_gc);
        }
    }

    /// <https://w3c.github.io/uievents/#handle-native-mouse-move>
    fn handle_native_mouse_move_event(&self, input_event: &ConstellationInputEvent, can_gc: CanGc) {
        // Ignore all incoming events without a hit test.
        let Some(hit_test_result) = self.window.hit_test_from_input_event(input_event) else {
            return;
        };

        let old_mouse_move_point = self
            .most_recent_mousemove_point
            .replace(Some(hit_test_result.point_in_frame));
        if old_mouse_move_point == Some(hit_test_result.point_in_frame) {
            return;
        }

        // Update the cursor when the mouse moves, if it has changed.
        self.set_cursor(Some(hit_test_result.cursor));

        let Some(new_target) = hit_test_result
            .node
            .inclusive_ancestors(ShadowIncluding::Yes)
            .find_map(DomRoot::downcast::<Element>)
        else {
            return;
        };

        let target_has_changed = self
            .current_hover_target
            .get()
            .is_none_or(|old_target| old_target != new_target);

        // Here we know the target has changed, so we must update the state,
        // dispatch mouseout to the previous one, mouseover to the new one.
        if target_has_changed {
            // Dispatch mouseout and mouseleave to previous target.
            if let Some(old_target) = self.current_hover_target.get() {
                let old_target_is_ancestor_of_new_target = old_target
                    .upcast::<Node>()
                    .is_ancestor_of(new_target.upcast::<Node>());

                // If the old target is an ancestor of the new target, this can be skipped
                // completely, since the node's hover state will be reset below.
                if !old_target_is_ancestor_of_new_target {
                    for element in old_target
                        .upcast::<Node>()
                        .inclusive_ancestors(ShadowIncluding::No)
                        .filter_map(DomRoot::downcast::<Element>)
                    {
                        element.set_hover_state(false);
                        element.set_active_state(false);
                    }
                }

                MouseEvent::new_for_platform_motion_event(
                    &self.window,
                    FireMouseEventType::Out,
                    &hit_test_result,
                    input_event,
                    can_gc,
                )
                .upcast::<Event>()
                .fire(old_target.upcast(), can_gc);

                if !old_target_is_ancestor_of_new_target {
                    let event_target = DomRoot::from_ref(old_target.upcast::<Node>());
                    let moving_into = Some(DomRoot::from_ref(new_target.upcast::<Node>()));
                    self.handle_mouse_enter_leave_event(
                        event_target,
                        moving_into,
                        FireMouseEventType::Leave,
                        &hit_test_result,
                        input_event,
                        can_gc,
                    );
                }
            }

            // Dispatch mouseover and mouseenter to new target.
            for element in new_target
                .upcast::<Node>()
                .inclusive_ancestors(ShadowIncluding::Yes)
                .filter_map(DomRoot::downcast::<Element>)
            {
                element.set_hover_state(true);
            }

            MouseEvent::new_for_platform_motion_event(
                &self.window,
                FireMouseEventType::Over,
                &hit_test_result,
                input_event,
                can_gc,
            )
            .upcast::<Event>()
            .dispatch(new_target.upcast(), false, can_gc);

            let moving_from = self
                .current_hover_target
                .get()
                .map(|old_target| DomRoot::from_ref(old_target.upcast::<Node>()));
            let event_target = DomRoot::from_ref(new_target.upcast::<Node>());
            self.handle_mouse_enter_leave_event(
                event_target,
                moving_from,
                FireMouseEventType::Enter,
                &hit_test_result,
                input_event,
                can_gc,
            );
        }

        // Send mousemove event to topmost target, unless it's an iframe, in which case
        // `Paint` should have also sent an event to the inner document.
        let mouse_event = MouseEvent::new_for_platform_motion_event(
            &self.window,
            FireMouseEventType::Move,
            &hit_test_result,
            input_event,
            can_gc,
        );

        // Send pointermove event before mousemove.
        let pointer_event = mouse_event.to_pointer_event("pointermove", can_gc);
        pointer_event
            .upcast::<Event>()
            .fire(new_target.upcast(), can_gc);

        // Send mousemove event to topmost target, unless it's an iframe, in which case
        // `Paint` should have also sent an event to the inner document.
        mouse_event
            .upcast::<Event>()
            .fire(new_target.upcast(), can_gc);

        self.update_current_hover_target_and_status(Some(new_target));
    }

    fn update_current_hover_target_and_status(&self, new_hover_target: Option<DomRoot<Element>>) {
        let current_hover_target = self.current_hover_target.get();
        if current_hover_target == new_hover_target {
            return;
        }

        let previous_hover_target = self.current_hover_target.get();
        self.current_hover_target.set(new_hover_target.as_deref());

        // If the new hover target is an anchor with a status value, inform the embedder
        // of the new value.
        if let Some(target) = self.current_hover_target.get() {
            if let Some(anchor) = target
                .upcast::<Node>()
                .inclusive_ancestors(ShadowIncluding::Yes)
                .find_map(DomRoot::downcast::<HTMLAnchorElement>)
            {
                let status = anchor
                    .full_href_url_for_user_interface()
                    .map(|url| url.to_string());
                self.window
                    .send_to_embedder(EmbedderMsg::Status(self.window.webview_id(), status));
                return;
            }
        }

        // No state was set above, which means that the new value of the status in the embedder
        // should be `None`. Set that now. If `previous_hover_target` is `None` that means this
        // is the first mouse move event we are seeing after getting the cursor. In that case,
        // we also clear the status.
        if previous_hover_target.is_none_or(|previous_hover_target| {
            previous_hover_target
                .upcast::<Node>()
                .inclusive_ancestors(ShadowIncluding::Yes)
                .any(|node| node.is::<HTMLAnchorElement>())
        }) {
            self.window
                .send_to_embedder(EmbedderMsg::Status(self.window.webview_id(), None));
        }
    }

    pub(crate) fn handle_refresh_cursor(&self) {
        let Some(most_recent_mousemove_point) = self.most_recent_mousemove_point.get() else {
            return;
        };

        let Some(hit_test_result) = self
            .window
            .hit_test_from_point_in_viewport(most_recent_mousemove_point)
        else {
            return;
        };

        self.set_cursor(Some(hit_test_result.cursor));
    }

    /// <https://w3c.github.io/uievents/#mouseevent-algorithms>
    /// Handles native mouse down, mouse up, mouse click.
    fn handle_native_mouse_button_event(
        &self,
        event: MouseButtonEvent,
        input_event: &ConstellationInputEvent,
        can_gc: CanGc,
    ) {
        // Ignore all incoming events without a hit test.
        let Some(hit_test_result) = self.window.hit_test_from_input_event(input_event) else {
            return;
        };

        debug!(
            "{:?}: at {:?}",
            event.action, hit_test_result.point_in_frame
        );

        let Some(element) = hit_test_result
            .node
            .inclusive_ancestors(ShadowIncluding::Yes)
            .find_map(DomRoot::downcast::<Element>)
        else {
            return;
        };

        let node = element.upcast::<Node>();
        debug!("{:?} on {:?}", event.action, node.debug_str());

        // https://w3c.github.io/uievents/#hit-test
        // Prevent mouse event if element is disabled.
        // TODO: also inert.
        if element.is_actually_disabled() {
            return;
        }

        let mouse_event_type_string = match event.action {
            embedder_traits::MouseButtonAction::Up => "mouseup",
            embedder_traits::MouseButtonAction::Down => "mousedown",
        };

        // From <https://w3c.github.io/uievents/#event-type-mousedown>
        // and <https://w3c.github.io/uievents/#event-type-mouseup>:
        //
        // UIEvent.detail: indicates the current click count incremented by one. For
        // example, if no click happened before the mousedown, detail will contain
        // the value 1
        if event.action == MouseButtonAction::Down {
            self.click_counting_info
                .borrow_mut()
                .reset_click_count_if_necessary(event.button, hit_test_result.point_in_frame);
        }

        let dom_event = DomRoot::upcast::<Event>(MouseEvent::for_platform_button_event(
            mouse_event_type_string,
            event,
            input_event.pressed_mouse_buttons,
            &self.window,
            &hit_test_result,
            input_event.active_keyboard_modifiers,
            self.click_counting_info.borrow().count + 1,
            can_gc,
        ));

        let activatable = element.as_maybe_activatable();
        match event.action {
            MouseButtonAction::Down => {
                self.last_mouse_button_down_point
                    .set(Some(hit_test_result.point_in_frame));

                if let Some(a) = activatable {
                    a.enter_formal_activation_state();
                }

                // Step 6. Dispatch pointerdown event.
                let down_button_count = self.down_button_count.get();

                let event_type = if down_button_count == 0 {
                    "pointerdown"
                } else {
                    "pointermove"
                };
                let pointer_event = dom_event
                    .downcast::<MouseEvent>()
                    .unwrap()
                    .to_pointer_event(event_type, can_gc);

                pointer_event.upcast::<Event>().fire(node.upcast(), can_gc);

                self.down_button_count.set(down_button_count + 1);

                // For a node within a text input UA shadow DOM,
                // delegate the focus target into its shadow host.
                // TODO: This focus delegation should be done
                // with shadow DOM delegateFocus attribute.
                let target_el = element.find_focusable_shadow_host_if_necessary();

                let document = self.window.Document();
                document.begin_focus_transaction();

                // Try to focus `el`. If it's not focusable, focus the document instead.
                document.request_focus(None, FocusInitiator::Local, can_gc);
                document.request_focus(target_el.as_deref(), FocusInitiator::Local, can_gc);

                // Step 7. Let result = dispatch event at target
                let result = dom_event.dispatch(node.upcast(), false, can_gc);

                // Step 8. If result is true and target is a focusable area
                // that is click focusable, then Run the focusing steps at target.
                if result && document.has_focus_transaction() {
                    document.commit_focus_transaction(FocusInitiator::Local, can_gc);
                }

                // Step 9. If mbutton is the secondary mouse button, then
                // Maybe show context menu with native, target.
                if let MouseButton::Right = event.button {
                    self.maybe_show_context_menu(
                        node.upcast(),
                        &hit_test_result,
                        input_event,
                        can_gc,
                    );
                }
            },
            // https://w3c.github.io/uievents/#handle-native-mouse-up
            MouseButtonAction::Up => {
                if let Some(a) = activatable {
                    a.exit_formal_activation_state();
                }

                // Step 6. Dispatch pointerup event.
                let down_button_count = self.down_button_count.get();

                if down_button_count > 0 {
                    self.down_button_count.set(down_button_count - 1);
                }

                let event_type = if down_button_count == 0 {
                    "pointerup"
                } else {
                    "pointermove"
                };
                let pointer_event = dom_event
                    .downcast::<MouseEvent>()
                    .unwrap()
                    .to_pointer_event(event_type, can_gc);

                pointer_event.upcast::<Event>().fire(node.upcast(), can_gc);

                // Step 7. dispatch event at target.
                dom_event.dispatch(node.upcast(), false, can_gc);

                // Click counts should still work for other buttons even though they
                // do not trigger "click" and "dblclick" events, so we increment
                // even when those events are not fired.
                self.click_counting_info
                    .borrow_mut()
                    .increment_click_count(event.button, hit_test_result.point_in_frame);

                self.maybe_trigger_click_for_mouse_button_down_event(
                    event,
                    input_event,
                    &hit_test_result,
                    &element,
                    can_gc,
                );
            },
        }
    }

    /// <https://w3c.github.io/uievents/#handle-native-mouse-click>
    /// <https://w3c.github.io/uievents/#event-type-dblclick>
    fn maybe_trigger_click_for_mouse_button_down_event(
        &self,
        event: MouseButtonEvent,
        input_event: &ConstellationInputEvent,
        hit_test_result: &HitTestResult,
        element: &Element,
        can_gc: CanGc,
    ) {
        if event.button != MouseButton::Left {
            return;
        }

        let Some(last_mouse_button_down_point) = self.last_mouse_button_down_point.take() else {
            return;
        };

        let distance = last_mouse_button_down_point.distance_to(hit_test_result.point_in_frame);
        let maximum_click_distance = 10.0 * self.window.device_pixel_ratio().get();
        if distance > maximum_click_distance {
            return;
        }

        // From <https://w3c.github.io/uievents/#event-type-click>
        // > The click event type MUST be dispatched on the topmost event target indicated by the
        // > pointer, when the user presses down and releases the primary pointer button.
        // For nodes inside a text input UA shadow DOM, dispatch dblclick at the shadow host.
        let delegated = element.find_focusable_shadow_host_if_necessary();
        let element = delegated.as_deref().unwrap_or(element);
        self.most_recently_clicked_element.set(Some(element));

        let click_count = self.click_counting_info.borrow().count;
        element.set_click_in_progress(true);
        MouseEvent::for_platform_button_event(
            "click",
            event,
            input_event.pressed_mouse_buttons,
            &self.window,
            hit_test_result,
            input_event.active_keyboard_modifiers,
            click_count,
            can_gc,
        )
        .upcast::<Event>()
        .dispatch(element.upcast(), false, can_gc);
        element.set_click_in_progress(false);

        // The firing of "dbclick" events is dependent on the platform, so we have
        // some flexibility here. Some browsers on some platforms only fire a
        // "dbclick" when the click count is 2 and others essentially fire one for
        // every 2 clicks in a sequence. In all cases, browsers set the click count
        // `detail` property to 2.
        //
        // We follow the latter approach here, considering that every sequence of
        // even numbered clicks is a series of double clicks.
        if click_count % 2 == 0 {
            MouseEvent::for_platform_button_event(
                "dblclick",
                event,
                input_event.pressed_mouse_buttons,
                &self.window,
                hit_test_result,
                input_event.active_keyboard_modifiers,
                2,
                can_gc,
            )
            .upcast::<Event>()
            .dispatch(element.upcast(), false, can_gc);
        }
    }

    /// <https://www.w3.org/TR/uievents/#maybe-show-context-menu>
    fn maybe_show_context_menu(
        &self,
        target: &EventTarget,
        hit_test_result: &HitTestResult,
        input_event: &ConstellationInputEvent,
        can_gc: CanGc,
    ) {
        // <https://w3c.github.io/uievents/#contextmenu>
        let menu_event = PointerEvent::new(
            &self.window,                   // window
            DOMString::from("contextmenu"), // type
            EventBubbles::Bubbles,          // can_bubble
            EventCancelable::Cancelable,    // cancelable
            Some(&self.window),             // view
            0,                              // detail
            hit_test_result.point_in_frame.to_i32(),
            hit_test_result.point_in_frame.to_i32(),
            hit_test_result
                .point_relative_to_initial_containing_block
                .to_i32(),
            input_event.active_keyboard_modifiers,
            2i16, // button, right mouse button
            input_event.pressed_mouse_buttons,
            None,                     // related_target
            None,                     // point_in_target
            PointerId::Mouse as i32,  // pointer_id
            1,                        // width
            1,                        // height
            0.5,                      // pressure
            0.0,                      // tangential_pressure
            0,                        // tilt_x
            0,                        // tilt_y
            0,                        // twist
            PI / 2.0,                 // altitude_angle
            0.0,                      // azimuth_angle
            DOMString::from("mouse"), // pointer_type
            true,                     // is_primary
            vec![],                   // coalesced_events
            vec![],                   // predicted_events
            can_gc,
        );

        // Step 3. Let result = dispatch menuevent at target.
        let result = menu_event.upcast::<Event>().fire(target, can_gc);

        // Step 4. If result is true, then show the UA context menu
        if result {
            self.window
                .Document()
                .embedder_controls()
                .show_context_menu(hit_test_result);
        };
    }

    fn handle_touch_event(
        &self,
        event: EmbedderTouchEvent,
        input_event: &ConstellationInputEvent,
        can_gc: CanGc,
    ) -> InputEventResult {
        // Ignore all incoming events without a hit test.
        let Some(hit_test_result) = self.window.hit_test_from_input_event(input_event) else {
            self.update_active_touch_points_when_early_return(event);
            return Default::default();
        };

        let TouchId(identifier) = event.id;
        let event_name = match event.event_type {
            TouchEventType::Down => "touchstart",
            TouchEventType::Move => "touchmove",
            TouchEventType::Up => "touchend",
            TouchEventType::Cancel => "touchcancel",
        };

        let Some(el) = hit_test_result
            .node
            .inclusive_ancestors(ShadowIncluding::Yes)
            .find_map(DomRoot::downcast::<Element>)
        else {
            self.update_active_touch_points_when_early_return(event);
            return Default::default();
        };

        let target = DomRoot::upcast::<EventTarget>(el);
        let window = &*self.window;

        let client_x = Finite::wrap(hit_test_result.point_in_frame.x as f64);
        let client_y = Finite::wrap(hit_test_result.point_in_frame.y as f64);
        let page_x =
            Finite::wrap(hit_test_result.point_in_frame.x as f64 + window.PageXOffset() as f64);
        let page_y =
            Finite::wrap(hit_test_result.point_in_frame.y as f64 + window.PageYOffset() as f64);

        let touch = Touch::new(
            window, identifier, &target, client_x,
            client_y, // TODO: Get real screen coordinates?
            client_x, client_y, page_x, page_y, can_gc,
        );

        // Dispatch pointer event before updating active touch points and before touch event.
        let pointer_event_type = match event.event_type {
            TouchEventType::Down => "pointerdown",
            TouchEventType::Move => "pointermove",
            TouchEventType::Up => "pointerup",
            TouchEventType::Cancel => "pointercancel",
        };

        // Get or create pointer ID for this touch
        let pointer_id = self.get_or_create_pointer_id_for_touch(identifier);
        let is_primary = self.is_primary_pointer(pointer_id);

        let pointer_event = touch.to_pointer_event(
            window,
            pointer_event_type,
            pointer_id,
            is_primary,
            input_event.active_keyboard_modifiers,
            event.is_cancelable(),
            Some(hit_test_result.point_in_node),
            can_gc,
        );
        pointer_event.upcast::<Event>().fire(&target, can_gc);

        match event.event_type {
            TouchEventType::Down => {
                // Add a new touch point
                self.active_touch_points
                    .borrow_mut()
                    .push(Dom::from_ref(&*touch));
            },
            TouchEventType::Move => {
                // Replace an existing touch point
                let mut active_touch_points = self.active_touch_points.borrow_mut();
                match active_touch_points
                    .iter_mut()
                    .find(|t| t.Identifier() == identifier)
                {
                    Some(t) => *t = Dom::from_ref(&*touch),
                    None => warn!("Got a touchmove event for a non-active touch point"),
                }
            },
            TouchEventType::Up | TouchEventType::Cancel => {
                // Remove an existing touch point
                let mut active_touch_points = self.active_touch_points.borrow_mut();
                match active_touch_points
                    .iter()
                    .position(|t| t.Identifier() == identifier)
                {
                    Some(i) => {
                        active_touch_points.swap_remove(i);
                        // Remove the pointer ID mapping when touch ends
                        self.remove_pointer_id_for_touch(identifier);
                    },
                    None => warn!("Got a touchend event for a non-active touch point"),
                }
            },
        }

        rooted_vec!(let mut target_touches);
        let touches = {
            let touches = self.active_touch_points.borrow();
            target_touches.extend(touches.iter().filter(|t| t.Target() == target).cloned());
            TouchList::new(window, touches.r(), can_gc)
        };

        let touch_event = TouchEvent::new(
            window,
            DOMString::from(event_name),
            EventBubbles::Bubbles,
            EventCancelable::from(event.is_cancelable()),
            EventComposed::Composed,
            Some(window),
            0i32,
            &touches,
            &TouchList::new(window, from_ref(&&*touch), can_gc),
            &TouchList::new(window, target_touches.r(), can_gc),
            // FIXME: modifier keys
            false,
            false,
            false,
            false,
            can_gc,
        );

        let event = touch_event.upcast::<Event>();
        event.fire(&target, can_gc);
        event.flags().into()
    }

    // If hittest fails, we still need to update the active point information.
    fn update_active_touch_points_when_early_return(&self, event: EmbedderTouchEvent) {
        match event.event_type {
            TouchEventType::Down => {
                // If the touchdown fails, we don't need to do anything.
                // When a touchmove or touchdown occurs at that touch point,
                // a warning is triggered: Got a touchmove/touchend event for a non-active touch point
            },
            TouchEventType::Move => {
                // The failure of touchmove does not affect the number of active points.
                // Since there is no position information when it fails, we do not need to update.
            },
            TouchEventType::Up | TouchEventType::Cancel => {
                // Remove an existing touch point
                let mut active_touch_points = self.active_touch_points.borrow_mut();
                match active_touch_points
                    .iter()
                    .position(|t| t.Identifier() == event.id.0)
                {
                    Some(i) => {
                        active_touch_points.swap_remove(i);
                        // Also remove pointer ID mapping when touch ends/cancels on early return
                        self.remove_pointer_id_for_touch(event.id.0);
                    },
                    None => warn!("Got a touchend event for a non-active touch point"),
                }
            },
        }
    }

    /// The entry point for all key processing for web content
    fn handle_keyboard_event(
        &self,
        keyboard_event: EmbedderKeyboardEvent,
        can_gc: CanGc,
    ) -> InputEventResult {
        let document = self.window.Document();
        let focused = document.get_focused_element();
        let body = document.GetBody();

        let target = match (&focused, &body) {
            (Some(focused), _) => focused.upcast(),
            (&None, Some(body)) => body.upcast(),
            (&None, &None) => self.window.upcast(),
        };

        let keyevent = KeyboardEvent::new(
            &self.window,
            DOMString::from(keyboard_event.event.state.event_type()),
            true,
            true,
            Some(&self.window),
            0,
            keyboard_event.event.key.clone(),
            DOMString::from(keyboard_event.event.code.to_string()),
            keyboard_event.event.location as u32,
            keyboard_event.event.repeat,
            keyboard_event.event.is_composing,
            keyboard_event.event.modifiers,
            0,
            keyboard_event.event.key.legacy_keycode(),
            can_gc,
        );

        let event = keyevent.upcast::<Event>();
        event.fire(target, can_gc);

        let mut flags = event.flags();
        if flags.contains(EventFlags::Canceled) {
            return flags.into();
        }

        // https://w3c.github.io/uievents/#keys-cancelable-keys
        // it MUST prevent the respective beforeinput and input
        // (and keypress if supported) events from being generated
        // TODO: keypress should be deprecated and superceded by beforeinput

        let is_character_value_key = matches!(
            keyboard_event.event.key,
            Key::Character(_) | Key::Named(NamedKey::Enter)
        );
        if keyboard_event.event.state == KeyState::Down &&
            is_character_value_key &&
            !keyboard_event.event.is_composing
        {
            // https://w3c.github.io/uievents/#keypress-event-order
            let keypress_event = KeyboardEvent::new(
                &self.window,
                DOMString::from("keypress"),
                true,
                true,
                Some(&self.window),
                0,
                keyboard_event.event.key.clone(),
                DOMString::from(keyboard_event.event.code.to_string()),
                keyboard_event.event.location as u32,
                keyboard_event.event.repeat,
                keyboard_event.event.is_composing,
                keyboard_event.event.modifiers,
                keyboard_event.event.key.legacy_charcode(),
                0,
                can_gc,
            );
            let event = keypress_event.upcast::<Event>();
            event.fire(target, can_gc);
            flags = event.flags();
        }

        if flags.contains(EventFlags::Canceled) {
            return flags.into();
        }

        // This behavior is unspecced
        // We are supposed to dispatch synthetic click activation for Space and/or Return,
        // however *when* we do it is up to us.
        // Here, we're dispatching it after the key event so the script has a chance to cancel it
        // https://www.w3.org/Bugs/Public/show_bug.cgi?id=27337
        if (keyboard_event.event.key == Key::Named(NamedKey::Enter) ||
            keyboard_event.event.code == Code::Space) &&
            keyboard_event.event.state == KeyState::Up
        {
            if let Some(elem) = target.downcast::<Element>() {
                elem.upcast::<Node>()
                    .fire_synthetic_pointer_event_not_trusted(DOMString::from("click"), can_gc);
            }
        }

        flags.into()
    }

    fn handle_ime_event(&self, event: ImeEvent, can_gc: CanGc) -> InputEventResult {
        let document = self.window.Document();
        let composition_event = match event {
            ImeEvent::Dismissed => {
                document.request_focus(
                    document.GetBody().as_ref().map(|e| e.upcast()),
                    FocusInitiator::Local,
                    can_gc,
                );
                return Default::default();
            },
            ImeEvent::Composition(composition_event) => composition_event,
        };

        // spec: https://w3c.github.io/uievents/#compositionstart
        // spec: https://w3c.github.io/uievents/#compositionupdate
        // spec: https://w3c.github.io/uievents/#compositionend
        // > Event.target : focused element processing the composition
        let focused = document.get_focused_element();
        let target = if let Some(elem) = &focused {
            elem.upcast()
        } else {
            // Event is only dispatched if there is a focused element.
            return Default::default();
        };

        let cancelable = composition_event.state == keyboard_types::CompositionState::Start;
        let event = CompositionEvent::new(
            &self.window,
            DOMString::from(composition_event.state.event_type()),
            true,
            cancelable,
            Some(&self.window),
            0,
            DOMString::from(composition_event.data),
            can_gc,
        );

        let event = event.upcast::<Event>();
        event.fire(target, can_gc);
        event.flags().into()
    }

    fn handle_wheel_event(
        &self,
        event: EmbedderWheelEvent,
        input_event: &ConstellationInputEvent,
        can_gc: CanGc,
    ) -> InputEventResult {
        // Ignore all incoming events without a hit test.
        let Some(hit_test_result) = self.window.hit_test_from_input_event(input_event) else {
            return Default::default();
        };

        let Some(el) = hit_test_result
            .node
            .inclusive_ancestors(ShadowIncluding::Yes)
            .find_map(DomRoot::downcast::<Element>)
        else {
            return Default::default();
        };

        let node = el.upcast::<Node>();
        let wheel_event_type_string = "wheel".to_owned();
        debug!(
            "{}: on {:?} at {:?}",
            wheel_event_type_string,
            node.debug_str(),
            hit_test_result.point_in_frame
        );

        // https://w3c.github.io/uievents/#event-wheelevents
        let dom_event = WheelEvent::new(
            &self.window,
            DOMString::from(wheel_event_type_string),
            EventBubbles::Bubbles,
            EventCancelable::Cancelable,
            Some(&self.window),
            0i32,
            hit_test_result.point_in_frame.to_i32(),
            hit_test_result.point_in_frame.to_i32(),
            hit_test_result
                .point_relative_to_initial_containing_block
                .to_i32(),
            input_event.active_keyboard_modifiers,
            0i16,
            input_event.pressed_mouse_buttons,
            None,
            None,
            // winit defines positive wheel delta values as revealing more content left/up.
            // https://docs.rs/winit-gtk/latest/winit/event/enum.MouseScrollDelta.html
            // This is the opposite of wheel delta in uievents
            // https://w3c.github.io/uievents/#dom-wheeleventinit-deltaz
            Finite::wrap(-event.delta.x),
            Finite::wrap(-event.delta.y),
            Finite::wrap(-event.delta.z),
            event.delta.mode as u32,
            can_gc,
        );

        let dom_event = dom_event.upcast::<Event>();
        dom_event.set_trusted(true);
        dom_event.fire(node.upcast(), can_gc);

        dom_event.flags().into()
    }

    #[cfg(feature = "gamepad")]
    fn handle_gamepad_event(&self, gamepad_event: EmbedderGamepadEvent) {
        match gamepad_event {
            EmbedderGamepadEvent::Connected(index, name, bounds, supported_haptic_effects) => {
                self.handle_gamepad_connect(
                    index.0,
                    name,
                    bounds.axis_bounds,
                    bounds.button_bounds,
                    supported_haptic_effects,
                );
            },
            EmbedderGamepadEvent::Disconnected(index) => {
                self.handle_gamepad_disconnect(index.0);
            },
            EmbedderGamepadEvent::Updated(index, update_type) => {
                self.receive_new_gamepad_button_or_axis(index.0, update_type);
            },
        };
    }

    /// <https://www.w3.org/TR/gamepad/#dfn-gamepadconnected>
    #[cfg(feature = "gamepad")]
    fn handle_gamepad_connect(
        &self,
        // As the spec actually defines how to set the gamepad index, the GilRs index
        // is currently unused, though in practice it will almost always be the same.
        // More infra is currently needed to track gamepads across windows.
        _index: usize,
        name: String,
        axis_bounds: (f64, f64),
        button_bounds: (f64, f64),
        supported_haptic_effects: GamepadSupportedHapticEffects,
    ) {
        // TODO: 2. If document is not null and is not allowed to use the "gamepad" permission,
        //          then abort these steps.
        let trusted_window = Trusted::new(&*self.window);
        self.window
            .upcast::<GlobalScope>()
            .task_manager()
            .gamepad_task_source()
            .queue(task!(gamepad_connected: move || {
                let window = trusted_window.root();

                let navigator = window.Navigator();
                let selected_index = navigator.select_gamepad_index();
                let gamepad = Gamepad::new(
                    &window,
                    selected_index,
                    name,
                    "standard".into(),
                    axis_bounds,
                    button_bounds,
                    supported_haptic_effects,
                    false,
                    CanGc::note(),
                );
                navigator.set_gamepad(selected_index as usize, &gamepad, CanGc::note());
            }));
    }

    /// <https://www.w3.org/TR/gamepad/#dfn-gamepaddisconnected>
    #[cfg(feature = "gamepad")]
    fn handle_gamepad_disconnect(&self, index: usize) {
        let trusted_window = Trusted::new(&*self.window);
        self.window
            .upcast::<GlobalScope>()
            .task_manager()
            .gamepad_task_source()
            .queue(task!(gamepad_disconnected: move || {
                let window = trusted_window.root();
                let navigator = window.Navigator();
                if let Some(gamepad) = navigator.get_gamepad(index) {
                    if window.Document().is_fully_active() {
                        gamepad.update_connected(false, gamepad.exposed(), CanGc::note());
                        navigator.remove_gamepad(index);
                    }
                }
            }));
    }

    /// <https://www.w3.org/TR/gamepad/#receiving-inputs>
    #[cfg(feature = "gamepad")]
    fn receive_new_gamepad_button_or_axis(&self, index: usize, update_type: GamepadUpdateType) {
        let trusted_window = Trusted::new(&*self.window);

        // <https://w3c.github.io/gamepad/#dfn-update-gamepad-state>
        self.window.upcast::<GlobalScope>().task_manager().gamepad_task_source().queue(
                task!(update_gamepad_state: move || {
                    let window = trusted_window.root();
                    let navigator = window.Navigator();
                    if let Some(gamepad) = navigator.get_gamepad(index) {
                        let current_time = window.Performance().Now();
                        gamepad.update_timestamp(*current_time);
                        match update_type {
                            GamepadUpdateType::Axis(index, value) => {
                                gamepad.map_and_normalize_axes(index, value);
                            },
                            GamepadUpdateType::Button(index, value) => {
                                gamepad.map_and_normalize_buttons(index, value);
                            }
                        };
                        if !navigator.has_gamepad_gesture() && contains_user_gesture(update_type) {
                            navigator.set_has_gamepad_gesture(true);
                            navigator.GetGamepads()
                                .iter()
                                .filter_map(|g| g.as_ref())
                                .for_each(|gamepad| {
                                    gamepad.set_exposed(true);
                                    gamepad.update_timestamp(*current_time);
                                    let new_gamepad = Trusted::new(&**gamepad);
                                    if window.Document().is_fully_active() {
                                        window.upcast::<GlobalScope>().task_manager().gamepad_task_source().queue(
                                            task!(update_gamepad_connect: move || {
                                                let gamepad = new_gamepad.root();
                                                gamepad.notify_event(GamepadEventType::Connected, CanGc::note());
                                            })
                                        );
                                    }
                                });
                        }
                    }
                })
            );
    }

    /// <https://www.w3.org/TR/clipboard-apis/#clipboard-actions>
    pub(crate) fn handle_editing_action(
        &self,
        element: Option<DomRoot<Element>>,
        action: EditingActionEvent,
        can_gc: CanGc,
    ) -> InputEventResult {
        let clipboard_event_type = match action {
            EditingActionEvent::Copy => ClipboardEventType::Copy,
            EditingActionEvent::Cut => ClipboardEventType::Cut,
            EditingActionEvent::Paste => ClipboardEventType::Paste,
        };

        // The script_triggered flag is set if the action runs because of a script, e.g. document.execCommand()
        let script_triggered = false;

        // The script_may_access_clipboard flag is set
        // if action is paste and the script thread is allowed to read from clipboard or
        // if action is copy or cut and the script thread is allowed to modify the clipboard
        let script_may_access_clipboard = false;

        // Step 1 If the script-triggered flag is set and the script-may-access-clipboard flag is unset
        if script_triggered && !script_may_access_clipboard {
            return InputEventResult::empty();
        }

        // Step 2 Fire a clipboard event
        let clipboard_event =
            self.fire_clipboard_event(element.clone(), clipboard_event_type, can_gc);

        // Step 3 If a script doesn't call preventDefault()
        // the event will be handled inside target's VirtualMethods::handle_event
        let event = clipboard_event.upcast::<Event>();
        if !event.IsTrusted() {
            return event.flags().into();
        }

        // Step 4 If the event was canceled, then
        if event.DefaultPrevented() {
            let event_type = event.Type();
            match_domstring_ascii!(event_type,

                "copy" => {
                    // Step 4.1 Call the write content to the clipboard algorithm,
                    // passing on the DataTransferItemList items, a clear-was-called flag and a types-to-clear list.
                    if let Some(clipboard_data) = clipboard_event.get_clipboard_data() {
                        let drag_data_store =
                            clipboard_data.data_store().expect("This shouldn't fail");
                        self.write_content_to_the_clipboard(&drag_data_store);
                    }
                },
                "cut" => {
                    // Step 4.1 Call the write content to the clipboard algorithm,
                    // passing on the DataTransferItemList items, a clear-was-called flag and a types-to-clear list.
                    if let Some(clipboard_data) = clipboard_event.get_clipboard_data() {
                        let drag_data_store =
                            clipboard_data.data_store().expect("This shouldn't fail");
                        self.write_content_to_the_clipboard(&drag_data_store);
                    }

                    // Step 4.2 Fire a clipboard event named clipboardchange
                    self.fire_clipboard_event(element, ClipboardEventType::Change, can_gc);
                },
                // Step 4.1 Return false.
                // Note: This function deviates from the specification a bit by returning
                // the `InputEventResult` below.
                "paste" => (),
                _ => (),
            )
        }

        // Step 5: Return true from the action.
        // In this case we are returning the `InputEventResult` instead of true or false.
        event.flags().into()
    }

    /// <https://www.w3.org/TR/clipboard-apis/#fire-a-clipboard-event>
    pub(crate) fn fire_clipboard_event(
        &self,
        target: Option<DomRoot<Element>>,
        clipboard_event_type: ClipboardEventType,
        can_gc: CanGc,
    ) -> DomRoot<ClipboardEvent> {
        let clipboard_event = ClipboardEvent::new(
            &self.window,
            None,
            DOMString::from(clipboard_event_type.as_str()),
            EventBubbles::Bubbles,
            EventCancelable::Cancelable,
            None,
            can_gc,
        );

        // Step 1 Let clear_was_called be false
        // Step 2 Let types_to_clear an empty list
        let mut drag_data_store = DragDataStore::new();

        // Step 4 let clipboard-entry be the sequence number of clipboard content, null if the OS doesn't support it.

        // Step 5 let trusted be true if the event is generated by the user agent, false otherwise
        let trusted = true;

        // Step 6 if the context is editable:
        let document = self.window.Document();
        let target = target.or(document.get_focused_element());
        let target = target
            .map(|target| DomRoot::from_ref(target.upcast()))
            .or_else(|| {
                document
                    .GetBody()
                    .map(|body| DomRoot::from_ref(body.upcast()))
            })
            .unwrap_or_else(|| DomRoot::from_ref(self.window.upcast()));

        // Step 6.2 else TODO require Selection see https://github.com/w3c/clipboard-apis/issues/70
        // Step 7
        match clipboard_event_type {
            ClipboardEventType::Copy | ClipboardEventType::Cut => {
                // Step 7.2.1
                drag_data_store.set_mode(Mode::ReadWrite);
            },
            ClipboardEventType::Paste => {
                let (callback, receiver) =
                    GenericCallback::new_blocking().expect("Could not create callback");
                self.window.send_to_embedder(EmbedderMsg::GetClipboardText(
                    self.window.webview_id(),
                    callback,
                ));
                let text_contents = receiver
                    .recv()
                    .map(Result::unwrap_or_default)
                    .unwrap_or_default();

                // Step 7.1.1
                drag_data_store.set_mode(Mode::ReadOnly);
                // Step 7.1.2 If trusted or the implementation gives script-generated events access to the clipboard
                if trusted {
                    // Step 7.1.2.1 For each clipboard-part on the OS clipboard:

                    // Step 7.1.2.1.1 If clipboard-part contains plain text, then
                    let data = DOMString::from(text_contents.to_string());
                    let type_ = DOMString::from("text/plain");
                    let _ = drag_data_store.add(Kind::Text { data, type_ });

                    // Step 7.1.2.1.2 TODO If clipboard-part represents file references, then for each file reference
                    // Step 7.1.2.1.3 TODO If clipboard-part contains HTML- or XHTML-formatted text then

                    // Step 7.1.3 Update clipboard-event-data’s files to match clipboard-event-data’s items
                    // Step 7.1.4 Update clipboard-event-data’s types to match clipboard-event-data’s items
                }
            },
            ClipboardEventType::Change => (),
        }

        // Step 3
        let clipboard_event_data = DataTransfer::new(
            &self.window,
            Rc::new(RefCell::new(Some(drag_data_store))),
            can_gc,
        );

        // Step 8
        clipboard_event.set_clipboard_data(Some(&clipboard_event_data));

        // Step 9
        let event = clipboard_event.upcast::<Event>();
        event.set_trusted(trusted);

        // Step 10 Set event’s composed to true.
        event.set_composed(true);

        // Step 11
        event.dispatch(&target, false, can_gc);

        DomRoot::from(clipboard_event)
    }

    /// <https://www.w3.org/TR/clipboard-apis/#write-content-to-the-clipboard>
    fn write_content_to_the_clipboard(&self, drag_data_store: &DragDataStore) {
        // Step 1
        if drag_data_store.list_len() > 0 {
            // Step 1.1 Clear the clipboard.
            self.window
                .send_to_embedder(EmbedderMsg::ClearClipboard(self.window.webview_id()));
            // Step 1.2
            for item in drag_data_store.iter_item_list() {
                match item {
                    Kind::Text { data, .. } => {
                        // Step 1.2.1.1 Ensure encoding is correct per OS and locale conventions
                        // Step 1.2.1.2 Normalize line endings according to platform conventions
                        // Step 1.2.1.3
                        self.window.send_to_embedder(EmbedderMsg::SetClipboardText(
                            self.window.webview_id(),
                            data.to_string(),
                        ));
                    },
                    Kind::File { .. } => {
                        // Step 1.2.2 If data is of a type listed in the mandatory data types list, then
                        // Step 1.2.2.1 Place part on clipboard with the appropriate OS clipboard format description
                        // Step 1.2.3 Else this is left to the implementation
                    },
                }
            }
        } else {
            // Step 2.1
            if drag_data_store.clear_was_called {
                // Step 2.1.1 If types-to-clear list is empty, clear the clipboard
                self.window
                    .send_to_embedder(EmbedderMsg::ClearClipboard(self.window.webview_id()));
                // Step 2.1.2 Else remove the types in the list from the clipboard
                // As of now this can't be done with Arboard, and it's possible that will be removed from the spec
            }
        }
    }

    /// Handle scroll event triggered by user interactions from embedder side.
    /// <https://drafts.csswg.org/cssom-view/#scrolling-events>
    #[expect(unsafe_code)]
    fn handle_embedder_scroll_event(&self, event: ScrollEvent) {
        // If it is a viewport scroll.
        let document = self.window.Document();
        if event.external_id.is_root() {
            document.handle_viewport_scroll_event();
        } else {
            // Otherwise, check whether it is for a relevant element within the document.
            let Some(node_id) = node_id_from_scroll_id(event.external_id.0 as usize) else {
                return;
            };
            let node = unsafe {
                node::from_untrusted_node_address(UntrustedNodeAddress::from_id(node_id))
            };
            let Some(element) = node
                .inclusive_ancestors(ShadowIncluding::Yes)
                .find_map(DomRoot::downcast::<Element>)
            else {
                return;
            };

            document.handle_element_scroll_event(&element);
        }
    }

    pub(crate) fn run_default_keyboard_event_handler(&self, event: &KeyboardEvent) {
        if event.upcast::<Event>().type_() != atom!("keydown") {
            return;
        }
        if !event.modifiers().is_empty() {
            return;
        }
        let scroll = match event.key() {
            Key::Named(NamedKey::ArrowDown) => KeyboardScroll::Down,
            Key::Named(NamedKey::ArrowLeft) => KeyboardScroll::Left,
            Key::Named(NamedKey::ArrowRight) => KeyboardScroll::Right,
            Key::Named(NamedKey::ArrowUp) => KeyboardScroll::Up,
            Key::Named(NamedKey::End) => KeyboardScroll::End,
            Key::Named(NamedKey::Home) => KeyboardScroll::Home,
            Key::Named(NamedKey::PageDown) => KeyboardScroll::PageDown,
            Key::Named(NamedKey::PageUp) => KeyboardScroll::PageUp,
            _ => return,
        };
        self.do_keyboard_scroll(scroll);
    }

    pub(crate) fn do_keyboard_scroll(&self, scroll: KeyboardScroll) {
        let scroll_axis = match scroll {
            KeyboardScroll::Left | KeyboardScroll::Right => ScrollingBoxAxis::X,
            _ => ScrollingBoxAxis::Y,
        };

        let document = self.window.Document();
        let mut scrolling_box = document
            .get_focused_element()
            .or(self.most_recently_clicked_element.get())
            .and_then(|element| element.scrolling_box(ScrollContainerQueryFlags::Inclusive))
            .unwrap_or_else(|| {
                document.viewport_scrolling_box(ScrollContainerQueryFlags::Inclusive)
            });

        while !scrolling_box.can_keyboard_scroll_in_axis(scroll_axis) {
            // Always fall back to trying to scroll the entire document.
            if scrolling_box.is_viewport() {
                break;
            }
            let parent = scrolling_box.parent().unwrap_or_else(|| {
                document.viewport_scrolling_box(ScrollContainerQueryFlags::Inclusive)
            });
            scrolling_box = parent;
        }

        let calculate_current_scroll_offset_and_delta = || {
            const LINE_HEIGHT: f32 = 76.0;
            const LINE_WIDTH: f32 = 76.0;

            let current_scroll_offset = scrolling_box.scroll_position();
            (
                current_scroll_offset,
                match scroll {
                    KeyboardScroll::Home => Vector2D::new(0.0, -current_scroll_offset.y),
                    KeyboardScroll::End => Vector2D::new(
                        0.0,
                        -current_scroll_offset.y + scrolling_box.content_size().height -
                            scrolling_box.size().height,
                    ),
                    KeyboardScroll::PageDown => {
                        Vector2D::new(0.0, scrolling_box.size().height - 2.0 * LINE_HEIGHT)
                    },
                    KeyboardScroll::PageUp => {
                        Vector2D::new(0.0, 2.0 * LINE_HEIGHT - scrolling_box.size().height)
                    },
                    KeyboardScroll::Up => Vector2D::new(0.0, -LINE_HEIGHT),
                    KeyboardScroll::Down => Vector2D::new(0.0, LINE_HEIGHT),
                    KeyboardScroll::Left => Vector2D::new(-LINE_WIDTH, 0.0),
                    KeyboardScroll::Right => Vector2D::new(LINE_WIDTH, 0.0),
                },
            )
        };

        // If trying to scroll the viewport of this `Window` and this is the root `Document`
        // of the `WebView`, then send the srolling operation to the renderer, so that it
        // can properly pan any pinch zoom viewport.
        let parent_pipeline = self.window.parent_info();
        if scrolling_box.is_viewport() && parent_pipeline.is_none() {
            let (_, delta) = calculate_current_scroll_offset_and_delta();
            self.window
                .paint_api()
                .scroll_viewport_by_delta(self.window.webview_id(), delta);
        }

        // If this is the viewport and we cannot scroll, try to ask a parent viewport to scroll,
        // if we are inside an `<iframe>`.
        if !scrolling_box.can_keyboard_scroll_in_axis(scroll_axis) {
            assert!(scrolling_box.is_viewport());

            let window_proxy = document.window().window_proxy();
            if let Some(iframe) = window_proxy.frame_element() {
                // When the `<iframe>` is local (in this ScriptThread), we can
                // synchronously chain up the keyboard scrolling event.
                let cx = GlobalScope::get_cx();
                let iframe_window = iframe.owner_window();
                let _ac = JSAutoRealm::new(*cx, iframe_window.reflector().get_jsobject().get());
                iframe_window
                    .Document()
                    .event_handler()
                    .do_keyboard_scroll(scroll);
            } else if let Some(parent_pipeline) = parent_pipeline {
                // Otherwise, if we have a parent (presumably from a different origin)
                // asynchronously ask the Constellation to forward the event to the parent
                // pipeline, if we have one.
                document.window().send_to_constellation(
                    ScriptToConstellationMessage::ForwardKeyboardScroll(parent_pipeline, scroll),
                );
            };
            return;
        }

        let (current_scroll_offset, delta) = calculate_current_scroll_offset_and_delta();
        scrolling_box.scroll_to(delta + current_scroll_offset, ScrollBehavior::Auto);
    }

    /// Get or create a pointer ID for the given touch identifier.
    /// Returns the pointer ID to use for this touch.
    fn get_or_create_pointer_id_for_touch(&self, touch_id: i32) -> i32 {
        let mut active_pointer_ids = self.active_pointer_ids.borrow_mut();

        if let Some(&pointer_id) = active_pointer_ids.get(&touch_id) {
            return pointer_id;
        }

        let pointer_id = self.next_touch_pointer_id.get();
        active_pointer_ids.insert(touch_id, pointer_id);
        self.next_touch_pointer_id.set(pointer_id + 1);
        pointer_id
    }

    /// Remove the pointer ID mapping for the given touch identifier.
    fn remove_pointer_id_for_touch(&self, touch_id: i32) {
        self.active_pointer_ids.borrow_mut().remove(&touch_id);
    }

    /// Check if this is the primary pointer (for touch events).
    /// The first touch to make contact is the primary pointer.
    fn is_primary_pointer(&self, pointer_id: i32) -> bool {
        // For touch, the primary pointer is the one with the smallest pointer ID
        // that is still active.
        self.active_pointer_ids
            .borrow()
            .values()
            .min()
            .is_some_and(|primary_pointer| *primary_pointer == pointer_id)
    }
}
