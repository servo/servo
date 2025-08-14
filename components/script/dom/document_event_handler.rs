/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::array::from_ref;
use std::cell::{Cell, RefCell};
use std::f64::consts::PI;
use std::mem;
use std::rc::Rc;
use std::time::{Duration, Instant};

use constellation_traits::ScriptToConstellationMessage;
use embedder_traits::{
    Cursor, EditingActionEvent, EmbedderMsg, GamepadEvent as EmbedderGamepadEvent,
    GamepadSupportedHapticEffects, GamepadUpdateType, ImeEvent, InputEvent,
    KeyboardEvent as EmbedderKeyboardEvent, MouseButton, MouseButtonAction, MouseButtonEvent,
    MouseLeaveEvent, ScrollEvent, TouchEvent as EmbedderTouchEvent, TouchEventType, TouchId,
    UntrustedNodeAddress, WheelEvent as EmbedderWheelEvent,
};
use euclid::Point2D;
use ipc_channel::ipc;
use keyboard_types::{Code, Key, KeyState, Modifiers, NamedKey};
use layout_api::node_id_from_scroll_id;
use script_bindings::codegen::GenericBindings::DocumentBinding::DocumentMethods;
use script_bindings::codegen::GenericBindings::EventBinding::EventMethods;
use script_bindings::codegen::GenericBindings::NavigatorBinding::NavigatorMethods;
use script_bindings::codegen::GenericBindings::NodeBinding::NodeMethods;
use script_bindings::codegen::GenericBindings::PerformanceBinding::PerformanceMethods;
use script_bindings::codegen::GenericBindings::TouchBinding::TouchMethods;
use script_bindings::codegen::GenericBindings::WindowBinding::WindowMethods;
use script_bindings::inheritance::Castable;
use script_bindings::num::Finite;
use script_bindings::root::{Dom, DomRoot, DomSlice};
use script_bindings::script_runtime::CanGc;
use script_bindings::str::DOMString;
use script_traits::ConstellationInputEvent;
use servo_config::pref;
use style_traits::CSSPixel;
use xml5ever::{local_name, ns};

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::root::MutNullableDom;
use crate::dom::clipboardevent::ClipboardEventType;
use crate::dom::document::{FireMouseEventType, FocusInitiator, TouchEventResult};
use crate::dom::event::{EventBubbles, EventCancelable, EventDefault};
use crate::dom::gamepad::contains_user_gesture;
use crate::dom::gamepadevent::GamepadEventType;
use crate::dom::inputevent::HitTestResult;
use crate::dom::node::{self, Node, ShadowIncluding};
use crate::dom::pointerevent::PointerId;
use crate::dom::types::{
    ClipboardEvent, CompositionEvent, DataTransfer, Element, Event, EventTarget, Gamepad,
    GlobalScope, HTMLAnchorElement, KeyboardEvent, MouseEvent, PointerEvent, Touch, TouchEvent,
    TouchList, WheelEvent, Window,
};
use crate::drag_data_store::{DragDataStore, Kind, Mode};
use crate::realms::enter_realm;

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
    #[ignore_malloc_size_of = "CompositorEvent contains data from outside crates"]
    pending_input_events: DomRefCell<Vec<ConstellationInputEvent>>,
    /// The index of the last mouse move event in the pending compositor events queue.
    mouse_move_event_index: DomRefCell<Option<usize>>,
    /// <https://w3c.github.io/uievents/#event-type-dblclick>
    #[ignore_malloc_size_of = "Defined in std"]
    #[no_trace]
    last_click_info: DomRefCell<Option<(Instant, Point2D<f32, CSSPixel>)>>,
    /// The element that is currently hovered by the cursor.
    current_hover_target: MutNullableDom<Element>,
    /// The most recent mouse movement point, used for processing `mouseleave` events.
    #[no_trace]
    most_recent_mousemove_point: Point2D<f32, CSSPixel>,
    /// The currently set [`Cursor`] or `None` if the `Document` isn't being hovered
    /// by the cursor.
    #[no_trace]
    current_cursor: Cell<Option<Cursor>>,
    /// <http://w3c.github.io/touch-events/#dfn-active-touch-point>
    active_touch_points: DomRefCell<Vec<Dom<Touch>>>,
    /// The active keyboard modifiers for the WebView. This is updated when receiving any input event.
    #[no_trace]
    active_keyboard_modifiers: Cell<Modifiers>,
}

impl DocumentEventHandler {
    pub(crate) fn new(window: &Window) -> Self {
        Self {
            window: Dom::from_ref(window),
            pending_input_events: Default::default(),
            mouse_move_event_index: Default::default(),
            last_click_info: Default::default(),
            current_hover_target: Default::default(),
            most_recent_mousemove_point: Default::default(),
            current_cursor: Default::default(),
            active_touch_points: Default::default(),
            active_keyboard_modifiers: Default::default(),
        }
    }

    /// Note a pending compositor event, to be processed at the next `update_the_rendering` task.
    pub(crate) fn note_pending_input_event(&self, event: ConstellationInputEvent) {
        let mut pending_compositor_events = self.pending_input_events.borrow_mut();
        if matches!(event.event, InputEvent::MouseMove(..)) {
            // First try to replace any existing mouse move event.
            if let Some(mouse_move_event) = self
                .mouse_move_event_index
                .borrow()
                .and_then(|index| pending_compositor_events.get_mut(index))
            {
                *mouse_move_event = event;
                return;
            }

            *self.mouse_move_event_index.borrow_mut() = Some(pending_compositor_events.len());
        }

        pending_compositor_events.push(event);
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

            match event.event.clone() {
                InputEvent::MouseButton(mouse_button_event) => {
                    self.handle_mouse_button_event(mouse_button_event, &event, can_gc);
                },
                InputEvent::MouseMove(_) => {
                    self.handle_mouse_move_event(&event, can_gc);
                },
                InputEvent::MouseLeave(mouse_leave_event) => {
                    self.handle_mouse_leave_event(&event, &mouse_leave_event, can_gc);
                },
                InputEvent::Touch(touch_event) => {
                    self.handle_touch_event(touch_event, &event, can_gc);
                },
                InputEvent::Wheel(wheel_event) => {
                    self.handle_wheel_event(wheel_event, &event, can_gc);
                },
                InputEvent::Keyboard(keyboard_event) => {
                    self.handle_keyboard_event(keyboard_event, can_gc);
                },
                InputEvent::Ime(ime_event) => {
                    self.handle_ime_event(ime_event, can_gc);
                },
                InputEvent::Gamepad(gamepad_event) => {
                    self.handle_gamepad_event(gamepad_event);
                },
                InputEvent::EditingAction(editing_action_event) => {
                    self.handle_editing_action(editing_action_event, can_gc);
                },
                InputEvent::Scroll(scroll_event) => {
                    self.handle_embedder_scroll_event(scroll_event);
                },
            }

            self.notify_webdriver_input_event_completed(event.event);
        }
    }

    fn notify_webdriver_input_event_completed(&self, event: InputEvent) {
        let Some(id) = event.webdriver_message_id() else {
            return;
        };

        // Webdriver should be notified once all current dom events have been processed.
        let trusted_window = Trusted::new(&*self.window);
        self.window
            .as_global_scope()
            .task_manager()
            .dom_manipulation_task_source()
            .queue(task!(notify_webdriver_input_event_completed: move || {
                let window = trusted_window.root();
                window.send_to_constellation(ScriptToConstellationMessage::WebDriverInputComplete(id));
            }));
    }

    pub(crate) fn set_cursor(&self, cursor: Cursor) {
        if Some(cursor) == self.current_cursor.get() {
            return;
        }
        self.current_cursor.set(Some(cursor));
        self.window
            .send_to_embedder(EmbedderMsg::SetCursor(self.window.webview_id(), cursor));
    }

    fn handle_mouse_leave_event(
        &self,
        input_event: &ConstellationInputEvent,
        mouse_leave_event: &MouseLeaveEvent,
        can_gc: CanGc,
    ) {
        if let Some(current_hover_target) = self.current_hover_target.get() {
            let current_hover_target = current_hover_target.upcast::<Node>();
            for element in current_hover_target
                .inclusive_ancestors(ShadowIncluding::No)
                .filter_map(DomRoot::downcast::<Element>)
            {
                element.set_hover_state(false);
                element.set_active_state(false);
            }

            if let Some(hit_test_result) = self
                .window
                .hit_test_from_point_in_viewport(self.most_recent_mousemove_point)
            {
                MouseEvent::new_simple(
                    &self.window,
                    FireMouseEventType::Out,
                    EventBubbles::Bubbles,
                    EventCancelable::Cancelable,
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

        self.current_cursor.set(None);
        self.current_hover_target.set(None);

        // If focus is moving to another frame, it will decide what the new status text is, but if
        // this mouse leave event is leaving the WebView entirely, then clear it.
        if !mouse_leave_event.focus_moving_to_another_iframe {
            self.window
                .send_to_embedder(EmbedderMsg::Status(self.window.webview_id(), None));
        }
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
                .common_ancestor(related_target, ShadowIncluding::No)
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
            MouseEvent::new_simple(
                &self.window,
                event_type,
                EventBubbles::DoesNotBubble,
                EventCancelable::NotCancelable,
                hit_test_result,
                input_event,
                can_gc,
            )
            .upcast::<Event>()
            .fire(target.upcast(), can_gc);
        }
    }

    fn handle_mouse_move_event(&self, input_event: &ConstellationInputEvent, can_gc: CanGc) {
        // Ignore all incoming events without a hit test.
        let Some(hit_test_result) = self.window.hit_test_from_input_event(input_event) else {
            return;
        };

        // Update the cursor when the mouse moves, if it has changed.
        self.set_cursor(hit_test_result.cursor);

        let Some(new_target) = hit_test_result
            .node
            .inclusive_ancestors(ShadowIncluding::No)
            .filter_map(DomRoot::downcast::<Element>)
            .next()
        else {
            return;
        };

        let target_has_changed = self
            .current_hover_target
            .get()
            .as_ref()
            .is_none_or(|old_target| old_target != &new_target);

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

                MouseEvent::new_simple(
                    &self.window,
                    FireMouseEventType::Out,
                    EventBubbles::Bubbles,
                    EventCancelable::Cancelable,
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
                .inclusive_ancestors(ShadowIncluding::No)
                .filter_map(DomRoot::downcast::<Element>)
            {
                element.set_hover_state(true);
            }

            MouseEvent::new_simple(
                &self.window,
                FireMouseEventType::Over,
                EventBubbles::Bubbles,
                EventCancelable::Cancelable,
                &hit_test_result,
                input_event,
                can_gc,
            )
            .upcast::<Event>()
            .fire(new_target.upcast(), can_gc);

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

        // Send mousemove event to topmost target, unless it's an iframe, in which case the
        // compositor should have also sent an event to the inner document.
        MouseEvent::new_simple(
            &self.window,
            FireMouseEventType::Move,
            EventBubbles::Bubbles,
            EventCancelable::Cancelable,
            &hit_test_result,
            input_event,
            can_gc,
        )
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
                .inclusive_ancestors(ShadowIncluding::No)
                .filter_map(DomRoot::downcast::<HTMLAnchorElement>)
                .next()
            {
                let status = anchor
                    .upcast::<Element>()
                    .get_attribute(&ns!(), &local_name!("href"))
                    .and_then(|href| {
                        let value = href.value();
                        let url = self.window.get_url();
                        url.join(&value).map(|url| url.to_string()).ok()
                    });
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
                .inclusive_ancestors(ShadowIncluding::No)
                .filter_map(DomRoot::downcast::<HTMLAnchorElement>)
                .next()
                .is_some()
        }) {
            self.window
                .send_to_embedder(EmbedderMsg::Status(self.window.webview_id(), None));
        }
    }

    /// <https://w3c.github.io/uievents/#mouseevent-algorithms>
    fn handle_mouse_button_event(
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

        let Some(el) = hit_test_result
            .node
            .inclusive_ancestors(ShadowIncluding::Yes)
            .filter_map(DomRoot::downcast::<Element>)
            .next()
        else {
            return;
        };

        let node = el.upcast::<Node>();
        debug!("{:?} on {:?}", event.action, node.debug_str());

        // https://w3c.github.io/uievents/#hit-test
        // Prevent mouse event if element is disabled.
        // TODO: also inert.
        if el.is_actually_disabled() {
            return;
        }

        let dom_event = DomRoot::upcast::<Event>(MouseEvent::for_platform_mouse_event(
            event,
            input_event.pressed_mouse_buttons,
            &self.window,
            &hit_test_result,
            input_event.active_keyboard_modifiers,
            can_gc,
        ));

        let activatable = el.as_maybe_activatable();
        match event.action {
            // https://w3c.github.io/uievents/#handle-native-mouse-click
            MouseButtonAction::Click => {
                el.set_click_in_progress(true);
                dom_event.dispatch(node.upcast(), false, can_gc);
                el.set_click_in_progress(false);

                self.maybe_fire_dblclick(node, &hit_test_result, input_event, can_gc);
            },
            // https://w3c.github.io/uievents/#handle-native-mouse-down
            MouseButtonAction::Down => {
                if let Some(a) = activatable {
                    a.enter_formal_activation_state();
                }

                // (TODO) Step 6. Maybe send pointerdown event with `dom_event`.

                // For a node within a text input UA shadow DOM,
                // delegate the focus target into its shadow host.
                // TODO: This focus delegation should be done
                // with shadow DOM delegateFocus attribute.
                let target_el = el.find_focusable_shadow_host_if_necessary();

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

                // (TODO) Step 6. Maybe send pointerup event with `dom_event``.

                // Step 7. dispatch event at target.
                dom_event.dispatch(node.upcast(), false, can_gc);
            },
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
            let (sender, receiver) = ipc::channel().expect("Failed to create IPC channel.");
            self.window.send_to_embedder(EmbedderMsg::ShowContextMenu(
                self.window.webview_id(),
                sender,
                None,
                vec![],
            ));
            let _ = receiver.recv().unwrap();
        };
    }

    fn maybe_fire_dblclick(
        &self,
        target: &Node,
        hit_test_result: &HitTestResult,
        input_event: &ConstellationInputEvent,
        can_gc: CanGc,
    ) {
        // https://w3c.github.io/uievents/#event-type-dblclick
        let now = Instant::now();
        let point_in_frame = hit_test_result.point_in_frame;
        let opt = self.last_click_info.borrow_mut().take();

        if let Some((last_time, last_pos)) = opt {
            let double_click_timeout =
                Duration::from_millis(pref!(dom_document_dblclick_timeout) as u64);
            let double_click_distance_threshold = pref!(dom_document_dblclick_dist) as u64;

            // Calculate distance between this click and the previous click.
            let line = point_in_frame - last_pos;
            let dist = (line.dot(line) as f64).sqrt();

            if now.duration_since(last_time) < double_click_timeout &&
                dist < double_click_distance_threshold as f64
            {
                // A double click has occurred if this click is within a certain time and dist. of previous click.
                let click_count = 2;

                let event = MouseEvent::new(
                    &self.window,
                    DOMString::from("dblclick"),
                    EventBubbles::Bubbles,
                    EventCancelable::Cancelable,
                    Some(&self.window),
                    click_count,
                    point_in_frame.to_i32(),
                    point_in_frame.to_i32(),
                    hit_test_result
                        .point_relative_to_initial_containing_block
                        .to_i32(),
                    input_event.active_keyboard_modifiers,
                    0i16,
                    input_event.pressed_mouse_buttons,
                    None,
                    None,
                    can_gc,
                );
                event.upcast::<Event>().fire(target.upcast(), can_gc);

                // When a double click occurs, self.last_click_info is left as None so that a
                // third sequential click will not cause another double click.
                return;
            }
        }

        // Update last_click_info with the time and position of the click.
        *self.last_click_info.borrow_mut() = Some((now, point_in_frame));
    }

    fn handle_touch_event(
        &self,
        event: EmbedderTouchEvent,
        input_event: &ConstellationInputEvent,
        can_gc: CanGc,
    ) {
        let result = self.handle_touch_event_inner(event, input_event, can_gc);
        if let (TouchEventResult::Processed(handled), true) = (result, event.is_cancelable()) {
            let sequence_id = event.expect_sequence_id();
            let result = if handled {
                embedder_traits::TouchEventResult::DefaultAllowed(sequence_id, event.event_type)
            } else {
                embedder_traits::TouchEventResult::DefaultPrevented(sequence_id, event.event_type)
            };
            self.window
                .send_to_constellation(ScriptToConstellationMessage::TouchEventProcessed(result));
        }
    }

    fn handle_touch_event_inner(
        &self,
        event: EmbedderTouchEvent,
        input_event: &ConstellationInputEvent,
        can_gc: CanGc,
    ) -> TouchEventResult {
        // Ignore all incoming events without a hit test.
        let Some(hit_test_result) = self.window.hit_test_from_input_event(input_event) else {
            self.update_active_touch_points_when_early_return(event);
            return TouchEventResult::Forwarded;
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
            .inclusive_ancestors(ShadowIncluding::No)
            .filter_map(DomRoot::downcast::<Element>)
            .next()
        else {
            self.update_active_touch_points_when_early_return(event);
            return TouchEventResult::Forwarded;
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

        let event = TouchEvent::new(
            window,
            DOMString::from(event_name),
            EventBubbles::Bubbles,
            EventCancelable::from(event.is_cancelable()),
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

        TouchEventResult::Processed(event.upcast::<Event>().fire(&target, can_gc))
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
                    },
                    None => warn!("Got a touchend event for a non-active touch point"),
                }
            },
        }
    }

    /// The entry point for all key processing for web content
    fn handle_keyboard_event(&self, keyboard_event: EmbedderKeyboardEvent, can_gc: CanGc) {
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
        let mut cancel_state = event.get_cancel_state();

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
            !keyboard_event.event.is_composing &&
            cancel_state != EventDefault::Prevented
        {
            // https://w3c.github.io/uievents/#keypress-event-order
            let event = KeyboardEvent::new(
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
            let ev = event.upcast::<Event>();
            ev.fire(target, can_gc);
            cancel_state = ev.get_cancel_state();
        }

        if cancel_state == EventDefault::Allowed {
            self.window.send_to_embedder(EmbedderMsg::Keyboard(
                self.window.webview_id(),
                keyboard_event.clone(),
            ));

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
        }
    }

    fn handle_ime_event(&self, event: ImeEvent, can_gc: CanGc) {
        let document = self.window.Document();
        let composition_event = match event {
            ImeEvent::Dismissed => {
                document.request_focus(
                    document.GetBody().as_ref().map(|e| e.upcast()),
                    FocusInitiator::Local,
                    can_gc,
                );
                return;
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
            return;
        };

        let cancelable = composition_event.state == keyboard_types::CompositionState::Start;
        CompositionEvent::new(
            &self.window,
            DOMString::from(composition_event.state.event_type()),
            true,
            cancelable,
            Some(&self.window),
            0,
            DOMString::from(composition_event.data),
            can_gc,
        )
        .upcast::<Event>()
        .fire(target, can_gc);
    }

    fn handle_wheel_event(
        &self,
        event: EmbedderWheelEvent,
        input_event: &ConstellationInputEvent,
        can_gc: CanGc,
    ) {
        // Ignore all incoming events without a hit test.
        let Some(hit_test_result) = self.window.hit_test_from_input_event(input_event) else {
            return;
        };

        let Some(el) = hit_test_result
            .node
            .inclusive_ancestors(ShadowIncluding::No)
            .filter_map(DomRoot::downcast::<Element>)
            .next()
        else {
            return;
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

        let target = node.upcast();
        dom_event.fire(target, can_gc);
    }

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
    fn handle_editing_action(&self, action: EditingActionEvent, can_gc: CanGc) -> bool {
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
            return false;
        }

        // Step 2 Fire a clipboard event
        let event = ClipboardEvent::new(
            &self.window,
            None,
            DOMString::from(clipboard_event_type.as_str()),
            EventBubbles::Bubbles,
            EventCancelable::Cancelable,
            None,
            can_gc,
        );
        self.fire_clipboard_event(&event, clipboard_event_type, can_gc);

        // Step 3 If a script doesn't call preventDefault()
        // the event will be handled inside target's VirtualMethods::handle_event

        let e = event.upcast::<Event>();

        if !e.IsTrusted() {
            return false;
        }

        // Step 4 If the event was canceled, then
        if e.DefaultPrevented() {
            match e.Type().str() {
                "copy" => {
                    // Step 4.1 Call the write content to the clipboard algorithm,
                    // passing on the DataTransferItemList items, a clear-was-called flag and a types-to-clear list.
                    if let Some(clipboard_data) = event.get_clipboard_data() {
                        let drag_data_store =
                            clipboard_data.data_store().expect("This shouldn't fail");
                        self.write_content_to_the_clipboard(&drag_data_store);
                    }
                },
                "cut" => {
                    // Step 4.1 Call the write content to the clipboard algorithm,
                    // passing on the DataTransferItemList items, a clear-was-called flag and a types-to-clear list.
                    if let Some(clipboard_data) = event.get_clipboard_data() {
                        let drag_data_store =
                            clipboard_data.data_store().expect("This shouldn't fail");
                        self.write_content_to_the_clipboard(&drag_data_store);
                    }

                    // Step 4.2 Fire a clipboard event named clipboardchange
                    self.fire_clipboardchange_event(can_gc);
                },
                "paste" => return false,
                _ => (),
            }
        }
        //Step 5
        true
    }

    /// <https://www.w3.org/TR/clipboard-apis/#fire-a-clipboard-event>
    fn fire_clipboard_event(
        &self,
        event: &ClipboardEvent,
        action: ClipboardEventType,
        can_gc: CanGc,
    ) {
        // Step 1 Let clear_was_called be false
        // Step 2 Let types_to_clear an empty list
        let mut drag_data_store = DragDataStore::new();

        // Step 4 let clipboard-entry be the sequence number of clipboard content, null if the OS doesn't support it.

        // Step 5 let trusted be true if the event is generated by the user agent, false otherwise
        let trusted = true;

        // Step 6 if the context is editable:
        let document = self.window.Document();
        let focused = document.get_focused_element();
        let body = document.GetBody();

        let target = match (&focused, &body) {
            (Some(focused), _) => focused.upcast(),
            (&None, Some(body)) => body.upcast(),
            (&None, &None) => self.window.upcast(),
        };
        // Step 6.2 else TODO require Selection see https://github.com/w3c/clipboard-apis/issues/70

        // Step 7
        match action {
            ClipboardEventType::Copy | ClipboardEventType::Cut => {
                // Step 7.2.1
                drag_data_store.set_mode(Mode::ReadWrite);
            },
            ClipboardEventType::Paste => {
                let (sender, receiver) = ipc::channel().unwrap();
                self.window
                    .send_to_constellation(ScriptToConstellationMessage::ForwardToEmbedder(
                        EmbedderMsg::GetClipboardText(self.window.webview_id(), sender),
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
        event.set_clipboard_data(Some(&clipboard_event_data));
        let event = event.upcast::<Event>();
        // Step 9
        event.set_trusted(trusted);
        // Step 10 Set event’s composed to true.
        event.set_composed(true);
        // Step 11
        event.dispatch(target, false, can_gc);
    }

    pub(crate) fn fire_clipboardchange_event(&self, can_gc: CanGc) {
        let clipboardchange_event = ClipboardEvent::new(
            &self.window,
            None,
            DOMString::from("clipboardchange"),
            EventBubbles::Bubbles,
            EventCancelable::Cancelable,
            None,
            can_gc,
        );
        self.fire_clipboard_event(&clipboardchange_event, ClipboardEventType::Change, can_gc);
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
    #[allow(unsafe_code)]
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
                .inclusive_ancestors(ShadowIncluding::No)
                .filter_map(DomRoot::downcast::<Element>)
                .next()
            else {
                return;
            };

            document.handle_element_scroll_event(&element);
        }
    }
}
