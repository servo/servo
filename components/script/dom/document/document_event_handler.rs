/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::array::from_ref;
use std::cell::{Cell, RefCell};
use std::cmp::Ordering;
use std::f64::consts::PI;
use std::mem;
use std::rc::Rc;
use std::str::FromStr;
use std::time::{Duration, Instant};

use embedder_traits::{
    Cursor, EditingActionEvent, EmbedderMsg, ImeEvent, InputEvent, InputEventId, InputEventOutcome,
    InputEventResult, KeyboardEvent as EmbedderKeyboardEvent, MouseButton, MouseButtonAction,
    MouseButtonEvent, MouseLeftViewportEvent, TouchEvent as EmbedderTouchEvent, TouchEventType,
    TouchId, UntrustedNodeAddress, WheelEvent as EmbedderWheelEvent,
};
#[cfg(feature = "gamepad")]
use embedder_traits::{
    GamepadEvent as EmbedderGamepadEvent, GamepadSupportedHapticEffects, GamepadUpdateType,
};
use euclid::{Point2D, Vector2D};
use js::context::JSContext;
use js::jsapi::JSAutoRealm;
use keyboard_types::{Code, Key, KeyState, Modifiers, NamedKey};
use layout_api::{ScrollContainerQueryFlags, node_id_from_scroll_id};
use rustc_hash::FxHashMap;
use script_bindings::codegen::GenericBindings::DocumentBinding::DocumentMethods;
use script_bindings::codegen::GenericBindings::ElementBinding::ScrollLogicalPosition;
use script_bindings::codegen::GenericBindings::EventBinding::EventMethods;
use script_bindings::codegen::GenericBindings::HTMLElementBinding::HTMLElementMethods;
use script_bindings::codegen::GenericBindings::HTMLLabelElementBinding::HTMLLabelElementMethods;
use script_bindings::codegen::GenericBindings::KeyboardEventBinding::KeyboardEventMethods;
use script_bindings::codegen::GenericBindings::NavigatorBinding::NavigatorMethods;
use script_bindings::codegen::GenericBindings::PerformanceBinding::PerformanceMethods;
use script_bindings::codegen::GenericBindings::ShadowRootBinding::ShadowRootMethods;
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
use servo_base::generic_channel::GenericCallback;
use servo_config::pref;
use servo_constellation_traits::{KeyboardScroll, ScriptToConstellationMessage};
use style::Atom;
use style_traits::CSSPixel;
use webrender_api::ExternalScrollId;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::inheritance::{ElementTypeId, HTMLElementTypeId, NodeTypeId};
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::root::MutNullableDom;
use crate::dom::bindings::trace::NoTrace;
use crate::dom::clipboardevent::ClipboardEventType;
use crate::dom::document::FireMouseEventType;
use crate::dom::document::focus::{FocusInitiator, FocusOperation, FocusableArea};
use crate::dom::event::{EventBubbles, EventCancelable, EventComposed, EventFlags};
#[cfg(feature = "gamepad")]
use crate::dom::gamepad::gamepad::{Gamepad, contains_user_gesture};
#[cfg(feature = "gamepad")]
use crate::dom::gamepad::gamepadevent::GamepadEventType;
use crate::dom::inputevent::HitTestResult;
use crate::dom::interactive_element_command::InteractiveElementCommand;
use crate::dom::keyboardevent::KeyboardEvent;
use crate::dom::node::{self, Node, NodeTraits, ShadowIncluding};
use crate::dom::pointerevent::{PointerEvent, PointerId};
use crate::dom::scrolling_box::{ScrollAxisState, ScrollRequirement, ScrollingBoxAxis};
use crate::dom::types::{
    ClipboardEvent, CompositionEvent, DataTransfer, Element, Event, EventTarget, GlobalScope,
    HTMLAnchorElement, HTMLElement, HTMLLabelElement, MouseEvent, Touch, TouchEvent, TouchList,
    WheelEvent, Window,
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
    /// The [`InputEventId`]s of mousemove events that have been coalesced.
    #[no_trace]
    #[ignore_malloc_size_of = "InputEventId contains data from outside crates"]
    coalesced_move_event_ids: DomRefCell<Vec<InputEventId>>,
    /// The index of the last wheel event in the pending input events queue.
    /// This is non-standard behaviour.
    /// According to <https://www.w3.org/TR/pointerevents/#dfn-coalesced-events>,
    /// we should only coalesce `pointermove` events.
    wheel_event_index: DomRefCell<Option<usize>>,
    /// The [`InputEventId`]s of wheel events that have been coalesced.
    #[no_trace]
    #[ignore_malloc_size_of = "InputEventId contains data from outside crates"]
    coalesced_wheel_event_ids: DomRefCell<Vec<InputEventId>>,
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
    active_pointer_ids: DomRefCell<FxHashMap<i32, i32>>,
    /// Counter for generating unique pointer IDs for touch inputs
    next_touch_pointer_id: Cell<i32>,
    /// A map holding information about currently registered access key handlers.
    access_key_handlers: DomRefCell<FxHashMap<NoTrace<Code>, Dom<HTMLElement>>>,
    /// <https://html.spec.whatwg.org/multipage/#sequential-focus-navigation-starting-point>
    sequential_focus_navigation_starting_point: MutNullableDom<Node>,
}

impl DocumentEventHandler {
    pub(crate) fn new(window: &Window) -> Self {
        Self {
            window: Dom::from_ref(window),
            pending_input_events: Default::default(),
            mouse_move_event_index: Default::default(),
            coalesced_move_event_ids: Default::default(),
            wheel_event_index: Default::default(),
            coalesced_wheel_event_ids: Default::default(),
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
            access_key_handlers: Default::default(),
            sequential_focus_navigation_starting_point: Default::default(),
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
                self.coalesced_move_event_ids
                    .borrow_mut()
                    .push(mouse_move_event.event.id);
                *mouse_move_event = event;
                return;
            }

            *self.mouse_move_event_index.borrow_mut() = Some(pending_input_events.len());
        }

        if let InputEvent::Wheel(ref new_wheel_event) = event.event.event {
            // Coalesce with any existing pending wheel event by summing deltas.
            if let Some(existing_constellation_wheel_event) = self
                .wheel_event_index
                .borrow()
                .and_then(|index| pending_input_events.get_mut(index))
            {
                if let InputEvent::Wheel(ref mut existing_wheel_event) =
                    existing_constellation_wheel_event.event.event
                {
                    if existing_wheel_event.delta.mode == new_wheel_event.delta.mode {
                        self.coalesced_wheel_event_ids
                            .borrow_mut()
                            .push(existing_constellation_wheel_event.event.id);
                        existing_wheel_event.delta.x += new_wheel_event.delta.x;
                        existing_wheel_event.delta.y += new_wheel_event.delta.y;
                        existing_wheel_event.delta.z += new_wheel_event.delta.z;
                        existing_wheel_event.point = new_wheel_event.point;
                        existing_constellation_wheel_event.event.id = event.event.id;
                        return;
                    }
                }
            }

            *self.wheel_event_index.borrow_mut() = Some(pending_input_events.len());
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

    pub(crate) fn handle_pending_input_events(&self, cx: &mut JSContext) {
        debug_assert!(
            !self.pending_input_events.borrow().is_empty(),
            "handle_pending_input_events called with no events"
        );
        let _realm = enter_realm(&*self.window);

        // Reset the mouse and wheel event indices.
        *self.mouse_move_event_index.borrow_mut() = None;
        *self.wheel_event_index.borrow_mut() = None;
        let pending_input_events = mem::take(&mut *self.pending_input_events.borrow_mut());
        let mut coalesced_move_event_ids =
            mem::take(&mut *self.coalesced_move_event_ids.borrow_mut());
        let mut coalesced_wheel_event_ids =
            mem::take(&mut *self.coalesced_wheel_event_ids.borrow_mut());

        let mut input_event_outcomes = Vec::with_capacity(
            pending_input_events.len() +
                coalesced_move_event_ids.len() +
                coalesced_wheel_event_ids.len(),
        );
        // TODO: For some of these we still aren't properly calculating whether or not
        // the event was handled or if `preventDefault()` was called on it. Each of
        // these cases needs to be examined and some of them either fire more than one
        // event or fire events later. We have to make a good decision about what to
        // return to the embedder when that happens.
        for event in pending_input_events {
            self.active_keyboard_modifiers
                .set(event.active_keyboard_modifiers);
            let result = match event.event.event {
                InputEvent::MouseButton(mouse_button_event) => {
                    self.handle_native_mouse_button_event(cx, mouse_button_event, &event);
                    InputEventResult::default()
                },
                InputEvent::MouseMove(_) => {
                    self.handle_native_mouse_move_event(cx, &event);
                    input_event_outcomes.extend(
                        mem::take(&mut coalesced_move_event_ids)
                            .into_iter()
                            .map(|id| InputEventOutcome {
                                id,
                                result: InputEventResult::default(),
                            }),
                    );
                    InputEventResult::default()
                },
                InputEvent::MouseLeftViewport(mouse_leave_event) => {
                    self.handle_mouse_left_viewport_event(cx, &event, &mouse_leave_event);
                    InputEventResult::default()
                },
                InputEvent::Touch(touch_event) => self.handle_touch_event(cx, touch_event, &event),
                InputEvent::Wheel(wheel_event) => {
                    let result = self.handle_wheel_event(cx, wheel_event, &event);
                    input_event_outcomes.extend(
                        mem::take(&mut coalesced_wheel_event_ids)
                            .into_iter()
                            .map(|id| InputEventOutcome { id, result }),
                    );
                    result
                },
                InputEvent::Keyboard(keyboard_event) => {
                    self.handle_keyboard_event(cx, keyboard_event)
                },
                InputEvent::Ime(ime_event) => self.handle_ime_event(cx, ime_event),
                #[cfg(feature = "gamepad")]
                InputEvent::Gamepad(gamepad_event) => {
                    self.handle_gamepad_event(gamepad_event);
                    InputEventResult::default()
                },
                InputEvent::EditingAction(editing_action_event) => {
                    self.handle_editing_action(cx, None, editing_action_event)
                },
            };

            input_event_outcomes.push(InputEventOutcome {
                id: event.event.id,
                result,
            });
        }

        self.notify_embedder_that_events_were_handled(input_event_outcomes);
    }

    fn notify_embedder_that_events_were_handled(
        &self,
        input_event_outcomes: Vec<InputEventOutcome>,
    ) {
        // Wait to to notify the embedder that the event was handled until all pending DOM
        // event processing is finished.
        let trusted_window = Trusted::new(&*self.window);
        self.window
            .as_global_scope()
            .task_manager()
            .dom_manipulation_task_source()
            .queue(task!(notify_webdriver_input_event_completed: move || {
                let window = trusted_window.root();
                window.send_to_embedder(
                    EmbedderMsg::InputEventsHandled(window.webview_id(), input_event_outcomes));
            }));
    }

    /// When an event should be fired on the element that has focus, this returns the target. If
    /// there is no associated element with the focused area (such as when the viewport is focused),
    /// then the body is returned. If no body is returned then the `Window` is returned.
    fn target_for_events_following_focus(&self) -> DomRoot<EventTarget> {
        let document = self.window.Document();
        match &*document.focus_handler().focused_area() {
            FocusableArea::Node { node, .. } => DomRoot::from_ref(node.upcast()),
            FocusableArea::Viewport => document
                .GetBody()
                .map(DomRoot::upcast)
                .unwrap_or_else(|| DomRoot::from_ref(self.window.upcast())),
        }
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
        cx: &mut JSContext,
        input_event: &ConstellationInputEvent,
        mouse_leave_event: &MouseLeftViewportEvent,
    ) {
        if let Some(current_hover_target) = self.current_hover_target.get() {
            let current_hover_target = current_hover_target.upcast::<Node>();
            for element in current_hover_target
                .inclusive_ancestors(ShadowIncluding::Yes)
                .filter_map(DomRoot::downcast::<Element>)
            {
                element.set_hover_state(false);
                self.element_for_activation(element).set_active_state(false);
            }

            if let Some(hit_test_result) = self
                .most_recent_mousemove_point
                .get()
                .and_then(|point| self.window.hit_test_from_point_in_viewport(point))
            {
                let mouse_out_event = MouseEvent::new_for_platform_motion_event(
                    cx,
                    &self.window,
                    FireMouseEventType::Out,
                    &hit_test_result,
                    input_event,
                );

                // Fire pointerout before mouseout
                mouse_out_event
                    .to_pointer_hover_event("pointerout", CanGc::from_cx(cx))
                    .upcast::<Event>()
                    .fire(current_hover_target.upcast(), CanGc::from_cx(cx));

                mouse_out_event
                    .upcast::<Event>()
                    .fire(current_hover_target.upcast(), CanGc::from_cx(cx));

                self.handle_mouse_enter_leave_event(
                    cx,
                    DomRoot::from_ref(current_hover_target),
                    None,
                    FireMouseEventType::Leave,
                    &hit_test_result,
                    input_event,
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
        cx: &mut JSContext,
        event_target: DomRoot<Node>,
        related_target: Option<DomRoot<Node>>,
        event_type: FireMouseEventType,
        hit_test_result: &HitTestResult,
        input_event: &ConstellationInputEvent,
    ) {
        assert!(matches!(
            event_type,
            FireMouseEventType::Enter | FireMouseEventType::Leave
        ));

        let common_ancestor = match related_target.as_ref() {
            Some(related_target) => event_target
                .common_ancestor_in_flat_tree(related_target)
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
            current = node.parent_in_flat_tree();
            targets.push(node);
        }

        // The order for dispatching mouseenter/pointerenter events starts from the topmost
        // common ancestor of the event target and the related target.
        if event_type == FireMouseEventType::Enter {
            targets = targets.into_iter().rev().collect();
        }

        let pointer_event_name = match event_type {
            FireMouseEventType::Enter => "pointerenter",
            FireMouseEventType::Leave => "pointerleave",
            _ => unreachable!(),
        };

        for target in targets {
            let mouse_event = MouseEvent::new_for_platform_motion_event(
                cx,
                &self.window,
                event_type,
                hit_test_result,
                input_event,
            );
            mouse_event
                .upcast::<Event>()
                .set_related_target(related_target.as_ref().map(|target| target.upcast()));

            // Fire pointer event before mouse event
            mouse_event
                .to_pointer_hover_event(pointer_event_name, CanGc::from_cx(cx))
                .upcast::<Event>()
                .fire(target.upcast(), CanGc::from_cx(cx));

            // Fire mouse event
            mouse_event
                .upcast::<Event>()
                .fire(target.upcast(), CanGc::from_cx(cx));
        }
    }

    /// <https://w3c.github.io/uievents/#handle-native-mouse-move>
    fn handle_native_mouse_move_event(
        &self,
        cx: &mut JSContext,
        input_event: &ConstellationInputEvent,
    ) {
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

        let old_hover_target = self.current_hover_target.get();
        let target_has_changed = old_hover_target
            .as_ref()
            .is_none_or(|old_target| *old_target != new_target);

        // Here we know the target has changed, so we must update the state,
        // dispatch mouseout to the previous one, mouseover to the new one.
        if target_has_changed {
            // Dispatch pointerout/mouseout and pointerleave/mouseleave to previous target.
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
                        self.element_for_activation(element).set_active_state(false);
                    }
                }

                let mouse_out_event = MouseEvent::new_for_platform_motion_event(
                    cx,
                    &self.window,
                    FireMouseEventType::Out,
                    &hit_test_result,
                    input_event,
                );
                mouse_out_event
                    .upcast::<Event>()
                    .set_related_target(Some(new_target.upcast()));

                // Fire pointerout before mouseout
                mouse_out_event
                    .to_pointer_hover_event("pointerout", CanGc::from_cx(cx))
                    .upcast::<Event>()
                    .fire(old_target.upcast(), CanGc::from_cx(cx));

                mouse_out_event
                    .upcast::<Event>()
                    .fire(old_target.upcast(), CanGc::from_cx(cx));

                if !old_target_is_ancestor_of_new_target {
                    let event_target = DomRoot::from_ref(old_target.upcast::<Node>());
                    let moving_into = Some(DomRoot::from_ref(new_target.upcast::<Node>()));
                    self.handle_mouse_enter_leave_event(
                        cx,
                        event_target,
                        moving_into,
                        FireMouseEventType::Leave,
                        &hit_test_result,
                        input_event,
                    );
                }
            }

            // Dispatch pointerover/mouseover and pointerenter/mouseenter to new target.
            for element in new_target
                .upcast::<Node>()
                .inclusive_ancestors(ShadowIncluding::Yes)
                .filter_map(DomRoot::downcast::<Element>)
            {
                element.set_hover_state(true);
            }

            let mouse_over_event = MouseEvent::new_for_platform_motion_event(
                cx,
                &self.window,
                FireMouseEventType::Over,
                &hit_test_result,
                input_event,
            );
            mouse_over_event
                .upcast::<Event>()
                .set_related_target(old_hover_target.as_ref().map(|target| target.upcast()));

            // Fire pointerover before mouseover
            mouse_over_event
                .to_pointer_hover_event("pointerover", CanGc::from_cx(cx))
                .upcast::<Event>()
                .dispatch(new_target.upcast(), false, CanGc::from_cx(cx));

            mouse_over_event.upcast::<Event>().dispatch(
                new_target.upcast(),
                false,
                CanGc::from_cx(cx),
            );

            let moving_from =
                old_hover_target.map(|old_target| DomRoot::from_ref(old_target.upcast::<Node>()));
            let event_target = DomRoot::from_ref(new_target.upcast::<Node>());
            self.handle_mouse_enter_leave_event(
                cx,
                event_target,
                moving_from,
                FireMouseEventType::Enter,
                &hit_test_result,
                input_event,
            );
        }

        // Send mousemove event to topmost target, unless it's an iframe, in which case
        // `Paint` should have also sent an event to the inner document.
        let mouse_event = MouseEvent::new_for_platform_motion_event(
            cx,
            &self.window,
            FireMouseEventType::Move,
            &hit_test_result,
            input_event,
        );

        // Send pointermove event before mousemove.
        let pointer_event =
            mouse_event.to_pointer_event(Atom::from("pointermove"), CanGc::from_cx(cx));
        pointer_event.upcast::<Event>().set_composed(true);
        pointer_event
            .upcast::<Event>()
            .fire(new_target.upcast(), CanGc::from_cx(cx));

        // Send mousemove event to topmost target, unless it's an iframe, in which case
        // `Paint` should have also sent an event to the inner document.
        mouse_event
            .upcast::<Event>()
            .fire(new_target.upcast(), CanGc::from_cx(cx));

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

    fn element_for_activation(&self, element: DomRoot<Element>) -> DomRoot<Element> {
        let node: &Node = element.upcast();
        if node.is_in_ua_widget() {
            if let Some(containing_shadow_root) = node.containing_shadow_root() {
                return containing_shadow_root.Host();
            }
        }

        // If the element is a label, the activable element is the control element.
        if node.type_id() ==
            NodeTypeId::Element(ElementTypeId::HTMLElement(
                HTMLElementTypeId::HTMLLabelElement,
            ))
        {
            let label = element.downcast::<HTMLLabelElement>().unwrap();
            if let Some(control) = label.GetControl() {
                return DomRoot::from_ref(control.upcast::<Element>());
            }
        }

        element
    }

    /// <https://w3c.github.io/uievents/#mouseevent-algorithms>
    /// Handles native mouse down, mouse up, mouse click.
    fn handle_native_mouse_button_event(
        &self,
        cx: &mut JSContext,
        event: MouseButtonEvent,
        input_event: &ConstellationInputEvent,
    ) {
        // Ignore all incoming events without a hit test.
        let Some(hit_test_result) = self.window.hit_test_from_input_event(input_event) else {
            return;
        };

        debug!(
            "{:?}: at {:?}",
            event.action, hit_test_result.point_in_frame
        );

        // Set the sequential focus navigation starting point for any mouse button down event, no
        // matter if the target is not a node.
        if event.action == MouseButtonAction::Down {
            self.set_sequential_focus_navigation_starting_point(&hit_test_result.node);
        }

        let Some(element) = hit_test_result
            .node
            .inclusive_ancestors(ShadowIncluding::Yes)
            .find_map(DomRoot::downcast::<Element>)
        else {
            return;
        };

        let node = element.upcast::<Node>();
        debug!("{:?} on {:?}", event.action, node.debug_str());

        // <https://html.spec.whatwg.org/multipage/#selector-active>
        // If the element is being actively pointed at the element is being activated.
        // Disabled elements can also be activated.
        if event.action == MouseButtonAction::Down {
            self.element_for_activation(element.clone())
                .set_active_state(true);
        }
        if event.action == MouseButtonAction::Up {
            self.element_for_activation(element.clone())
                .set_active_state(false);
        }

        // https://w3c.github.io/uievents/#hit-test
        // Prevent mouse event if element is disabled.
        // TODO: also inert.
        if element.is_actually_disabled() {
            return;
        }

        let mouse_event_type = match event.action {
            embedder_traits::MouseButtonAction::Up => atom!("mouseup"),
            embedder_traits::MouseButtonAction::Down => atom!("mousedown"),
        };

        // From <https://w3c.github.io/pointerevents/#dfn-mousedown>
        // and <https://w3c.github.io/pointerevents/#mouseup>:
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
            cx,
            mouse_event_type,
            event,
            input_event.pressed_mouse_buttons,
            &self.window,
            &hit_test_result,
            input_event.active_keyboard_modifiers,
            self.click_counting_info.borrow().count + 1,
        ));

        match event.action {
            MouseButtonAction::Down => {
                self.last_mouse_button_down_point
                    .set(Some(hit_test_result.point_in_frame));

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
                    .to_pointer_event(event_type.into(), CanGc::from_cx(cx));

                pointer_event
                    .upcast::<Event>()
                    .fire(node.upcast(), CanGc::from_cx(cx));

                self.down_button_count.set(down_button_count + 1);

                // Step 7. Let result = dispatch event at target
                let result = dom_event.dispatch(node.upcast(), false, CanGc::from_cx(cx));

                // Step 8. If result is true and target is a focusable area
                // that is click focusable, then Run the focusing steps at target.
                if result {
                    // Note that this differs from the specification, because we are going to look
                    // for the first inclusive ancestor that is click focusable and then focus it.
                    // See documentation for [`Node::find_click_focusable_area`].
                    self.window.Document().focus_handler().focus(
                        FocusOperation::Focus(node.find_click_focusable_area()),
                        FocusInitiator::Local,
                        CanGc::from_cx(cx),
                    );
                }

                // Step 9. If mbutton is the secondary mouse button, then
                // Maybe show context menu with native, target.
                if let MouseButton::Right = event.button {
                    self.maybe_show_context_menu(
                        node.upcast(),
                        &hit_test_result,
                        input_event,
                        CanGc::from_cx(cx),
                    );
                }
            },
            // https://w3c.github.io/pointerevents/#dfn-handle-native-mouse-up
            MouseButtonAction::Up => {
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
                    .to_pointer_event(event_type.into(), CanGc::from_cx(cx));

                pointer_event
                    .upcast::<Event>()
                    .fire(node.upcast(), CanGc::from_cx(cx));

                // Step 7. dispatch event at target.
                dom_event.dispatch(node.upcast(), false, CanGc::from_cx(cx));

                // Click counts should still work for other buttons even though they
                // do not trigger "click" and "dblclick" events, so we increment
                // even when those events are not fired.
                self.click_counting_info
                    .borrow_mut()
                    .increment_click_count(event.button, hit_test_result.point_in_frame);

                self.maybe_trigger_click_for_mouse_button_down_event(
                    cx,
                    event,
                    input_event,
                    &hit_test_result,
                    &element,
                );
            },
        }
    }

    /// <https://w3c.github.io/pointerevents/#handle-native-mouse-click>
    /// <https://w3c.github.io/pointerevents/#handle-native-mouse-double-click>
    fn maybe_trigger_click_for_mouse_button_down_event(
        &self,
        cx: &mut JSContext,
        event: MouseButtonEvent,
        input_event: &ConstellationInputEvent,
        hit_test_result: &HitTestResult,
        element: &Element,
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

        // From <https://w3c.github.io/pointerevents/#click>
        // > The click event type MUST be dispatched on the topmost event target indicated by the
        // > pointer, when the user presses down and releases the primary pointer button.
        //
        // For nodes inside a text input UA shadow DOM, dispatch dblclick at the shadow host.
        // TODO: This should likely be handled via event retargeting.
        let element = match hit_test_result.node.find_click_focusable_area() {
            FocusableArea::Node { node, .. } => DomRoot::downcast::<Element>(node),
            _ => None,
        }
        .unwrap_or_else(|| DomRoot::from_ref(element));
        self.most_recently_clicked_element.set(Some(&*element));

        let click_count = self.click_counting_info.borrow().count;
        element.set_click_in_progress(true);
        MouseEvent::for_platform_button_event(
            cx,
            atom!("click"),
            event,
            input_event.pressed_mouse_buttons,
            &self.window,
            hit_test_result,
            input_event.active_keyboard_modifiers,
            click_count,
        )
        .upcast::<Event>()
        .dispatch(element.upcast(), false, CanGc::from_cx(cx));
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
                cx,
                Atom::from("dblclick"),
                event,
                input_event.pressed_mouse_buttons,
                &self.window,
                hit_test_result,
                input_event.active_keyboard_modifiers,
                2,
            )
            .upcast::<Event>()
            .dispatch(element.upcast(), false, CanGc::from_cx(cx));
        }
    }

    /// <https://www.w3.org/TR/pointerevents4/#maybe-show-context-menu>
    fn maybe_show_context_menu(
        &self,
        target: &EventTarget,
        hit_test_result: &HitTestResult,
        input_event: &ConstellationInputEvent,
        can_gc: CanGc,
    ) {
        // <https://w3c.github.io/pointerevents/#contextmenu>
        let menu_event = PointerEvent::new(
            &self.window,                // window
            "contextmenu".into(),        // type
            EventBubbles::Bubbles,       // can_bubble
            EventCancelable::Cancelable, // cancelable
            Some(&self.window),          // view
            0,                           // detail
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
        menu_event.upcast::<Event>().set_composed(true);

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
        cx: &mut JSContext,
        event: EmbedderTouchEvent,
        input_event: &ConstellationInputEvent,
    ) -> InputEventResult {
        // Ignore all incoming events without a hit test.
        let Some(hit_test_result) = self.window.hit_test_from_input_event(input_event) else {
            self.update_active_touch_points_when_early_return(event);
            return Default::default();
        };

        let TouchId(identifier) = event.touch_id;

        let Some(element) = hit_test_result
            .node
            .inclusive_ancestors(ShadowIncluding::Yes)
            .find_map(DomRoot::downcast::<Element>)
        else {
            self.update_active_touch_points_when_early_return(event);
            return Default::default();
        };

        let current_target = DomRoot::upcast::<EventTarget>(element.clone());
        let window = &*self.window;

        let client_x = Finite::wrap(hit_test_result.point_in_frame.x as f64);
        let client_y = Finite::wrap(hit_test_result.point_in_frame.y as f64);
        let page_x =
            Finite::wrap(hit_test_result.point_in_frame.x as f64 + window.PageXOffset() as f64);
        let page_y =
            Finite::wrap(hit_test_result.point_in_frame.y as f64 + window.PageYOffset() as f64);

        // This is used to construct pointerevent and touchdown event.
        let pointer_touch = Touch::new(
            window,
            identifier,
            &current_target,
            client_x,
            client_y, // TODO: Get real screen coordinates?
            client_x,
            client_y,
            page_x,
            page_y,
            CanGc::from_cx(cx),
        );

        // Dispatch pointer event before updating active touch points and before touch event.
        let pointer_event_name = match event.event_type {
            TouchEventType::Down => "pointerdown",
            TouchEventType::Move => "pointermove",
            TouchEventType::Up => "pointerup",
            TouchEventType::Cancel => "pointercancel",
        };

        // Get or create pointer ID for this touch
        let pointer_id = self.get_or_create_pointer_id_for_touch(identifier);
        let is_primary = self.is_primary_pointer(pointer_id);

        // For touch devices (which don't support hover), fire pointerover/pointerenter
        // <https://w3c.github.io/pointerevents/#mapping-for-devices-that-do-not-support-hover>
        if matches!(event.event_type, TouchEventType::Down) {
            // Fire pointerover
            let pointer_over = pointer_touch.to_pointer_event(
                window,
                "pointerover",
                pointer_id,
                is_primary,
                input_event.active_keyboard_modifiers,
                true, // cancelable
                Some(hit_test_result.point_in_node),
                CanGc::from_cx(cx),
            );
            pointer_over
                .upcast::<Event>()
                .fire(&current_target, CanGc::from_cx(cx));

            // Fire pointerenter hierarchically (from topmost ancestor to target)
            self.fire_pointer_event_for_touch(
                &element,
                &pointer_touch,
                pointer_id,
                "pointerenter",
                is_primary,
                input_event,
                &hit_test_result,
                CanGc::from_cx(cx),
            );
        }

        let pointer_event = pointer_touch.to_pointer_event(
            window,
            pointer_event_name,
            pointer_id,
            is_primary,
            input_event.active_keyboard_modifiers,
            event.is_cancelable(),
            Some(hit_test_result.point_in_node),
            CanGc::from_cx(cx),
        );
        pointer_event
            .upcast::<Event>()
            .fire(&current_target, CanGc::from_cx(cx));

        // For touch devices, fire pointerout/pointerleave after pointerup/pointercancel
        // <https://w3c.github.io/pointerevents/#mapping-for-devices-that-do-not-support-hover>
        if matches!(
            event.event_type,
            TouchEventType::Up | TouchEventType::Cancel
        ) {
            // Fire pointerout
            let pointer_out = pointer_touch.to_pointer_event(
                window,
                "pointerout",
                pointer_id,
                is_primary,
                input_event.active_keyboard_modifiers,
                true, // cancelable
                Some(hit_test_result.point_in_node),
                CanGc::from_cx(cx),
            );
            pointer_out
                .upcast::<Event>()
                .fire(&current_target, CanGc::from_cx(cx));

            // Fire pointerleave hierarchically (from target to topmost ancestor)
            self.fire_pointer_event_for_touch(
                &element,
                &pointer_touch,
                pointer_id,
                "pointerleave",
                is_primary,
                input_event,
                &hit_test_result,
                CanGc::from_cx(cx),
            );
        }

        let (touch_dispatch_target, changed_touch) = match event.event_type {
            TouchEventType::Down => {
                // Add a new touch point
                self.active_touch_points
                    .borrow_mut()
                    .push(Dom::from_ref(&*pointer_touch));
                // <https://html.spec.whatwg.org/multipage/#selector-active>
                // If the element is being actively pointed at the element is being activated.
                self.element_for_activation(element).set_active_state(true);
                (current_target, pointer_touch)
            },
            _ => {
                // From <https://w3c.github.io/touch-events/#dfn-touchend>:
                // > For move/up/cancel:
                // > The target of this event must be the same Element on which the touch
                // > point started when it was first placed on the surface, even if the touch point
                // > has since moved outside the interactive area of the target element.
                let mut active_touch_points = self.active_touch_points.borrow_mut();
                let Some(index) = active_touch_points
                    .iter()
                    .position(|point| point.Identifier() == identifier)
                else {
                    warn!("No active touch point for {:?}", event.event_type);
                    return Default::default();
                };
                // This is the original target that was selected during `touchstart` event handling.
                let original_target = active_touch_points[index].Target();

                let touch_with_touchstart_target = Touch::new(
                    window,
                    identifier,
                    &original_target,
                    client_x,
                    client_y,
                    client_x,
                    client_y,
                    page_x,
                    page_y,
                    CanGc::from_cx(cx),
                );

                // Update or remove the stored touch
                match event.event_type {
                    TouchEventType::Move => {
                        active_touch_points[index] = Dom::from_ref(&*touch_with_touchstart_target);
                    },
                    TouchEventType::Up | TouchEventType::Cancel => {
                        active_touch_points.swap_remove(index);
                        self.remove_pointer_id_for_touch(identifier);
                        // <https://html.spec.whatwg.org/multipage/#selector-active>
                        // If the element is being actively pointed at the element is being activated.
                        self.element_for_activation(element).set_active_state(false);
                    },
                    TouchEventType::Down => unreachable!("Should have been handled above"),
                }
                (original_target, touch_with_touchstart_target)
            },
        };

        rooted_vec!(let mut target_touches);
        target_touches.extend(
            self.active_touch_points
                .borrow()
                .iter()
                .filter(|touch| touch.Target() == touch_dispatch_target)
                .cloned(),
        );

        let event_name = match event.event_type {
            TouchEventType::Down => "touchstart",
            TouchEventType::Move => "touchmove",
            TouchEventType::Up => "touchend",
            TouchEventType::Cancel => "touchcancel",
        };

        let touch_event = TouchEvent::new(
            window,
            event_name.into(),
            EventBubbles::Bubbles,
            EventCancelable::from(event.is_cancelable()),
            EventComposed::Composed,
            Some(window),
            0i32,
            &TouchList::new(
                window,
                self.active_touch_points.borrow().r(),
                CanGc::from_cx(cx),
            ),
            &TouchList::new(window, from_ref(&&*changed_touch), CanGc::from_cx(cx)),
            &TouchList::new(window, target_touches.r(), CanGc::from_cx(cx)),
            // FIXME: modifier keys
            false,
            false,
            false,
            false,
            CanGc::from_cx(cx),
        );
        let event = touch_event.upcast::<Event>();
        event.fire(&touch_dispatch_target, CanGc::from_cx(cx));
        event.flags().into()
    }

    /// Updates the active touch points when a hit test fails early.
    ///
    /// - For `Down`: No action needed; a failed down event won't create an active point.
    /// - For `Move`: No action needed; position information is unavailable, so we cannot update.
    /// - For `Up`/`Cancel`: Remove the corresponding touch point and its pointer ID mapping.
    ///
    /// When a touchup or touchcancel occurs at that touch point,
    /// a warning is triggered: Received touchup/touchcancel event for a non-active touch point.
    fn update_active_touch_points_when_early_return(&self, event: EmbedderTouchEvent) {
        match event.event_type {
            TouchEventType::Down | TouchEventType::Move => {},
            TouchEventType::Up | TouchEventType::Cancel => {
                let mut active_touch_points = self.active_touch_points.borrow_mut();
                if let Some(index) = active_touch_points
                    .iter()
                    .position(|t| t.Identifier() == event.touch_id.0)
                {
                    active_touch_points.swap_remove(index);
                    self.remove_pointer_id_for_touch(event.touch_id.0);
                } else {
                    warn!(
                        "Received {:?} for a non-active touch point {}",
                        event.event_type, event.touch_id.0
                    );
                }
            },
        }
    }

    /// The entry point for all key processing for web content
    fn handle_keyboard_event(
        &self,
        cx: &mut JSContext,
        keyboard_event: EmbedderKeyboardEvent,
    ) -> InputEventResult {
        let target = &self.target_for_events_following_focus();
        let keyevent = KeyboardEvent::new_with_platform_keyboard_event(
            cx,
            &self.window,
            keyboard_event.event.state.event_type().into(),
            &keyboard_event.event,
        );

        let event = keyevent.upcast::<Event>();

        event.set_composed(true);

        event.fire(target, CanGc::from_cx(cx));

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
            let keypress_event = KeyboardEvent::new_with_platform_keyboard_event(
                cx,
                &self.window,
                atom!("keypress"),
                &keyboard_event.event,
            );
            keypress_event.upcast::<Event>().set_composed(true);
            let event = keypress_event.upcast::<Event>();
            event.fire(target, CanGc::from_cx(cx));
            flags = event.flags();
        }

        flags.into()
    }

    fn handle_ime_event(&self, cx: &mut JSContext, event: ImeEvent) -> InputEventResult {
        let document = self.window.Document();
        let composition_event = match event {
            ImeEvent::Dismissed => {
                document.focus_handler().focus(
                    FocusOperation::Focus(FocusableArea::Viewport),
                    FocusInitiator::Local,
                    CanGc::from_cx(cx),
                );
                return Default::default();
            },
            ImeEvent::Composition(composition_event) => composition_event,
        };

        // spec: https://w3c.github.io/uievents/#compositionstart
        // spec: https://w3c.github.io/uievents/#compositionupdate
        // spec: https://w3c.github.io/uievents/#compositionend
        // > Event.target : focused element processing the composition
        let focused_area = document.focus_handler().focused_area();
        let Some(focused_element) = focused_area.element() else {
            // Event is only dispatched if there is a focused element.
            return Default::default();
        };

        let cancelable = composition_event.state == keyboard_types::CompositionState::Start;
        let event = CompositionEvent::new(
            &self.window,
            composition_event.state.event_type().into(),
            true,
            cancelable,
            Some(&self.window),
            0,
            DOMString::from(composition_event.data),
            CanGc::from_cx(cx),
        );

        let event = event.upcast::<Event>();
        event.fire(focused_element.upcast(), CanGc::from_cx(cx));
        event.flags().into()
    }

    fn handle_wheel_event(
        &self,
        cx: &mut JSContext,
        event: EmbedderWheelEvent,
        input_event: &ConstellationInputEvent,
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
        debug!(
            "wheel: on {:?} at {:?}",
            node.debug_str(),
            hit_test_result.point_in_frame
        );

        // https://w3c.github.io/uievents/#event-wheelevents
        let dom_event = WheelEvent::new(
            &self.window,
            "wheel".into(),
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
            CanGc::from_cx(cx),
        );

        let dom_event = dom_event.upcast::<Event>();
        dom_event.set_trusted(true);
        dom_event.set_composed(true);
        dom_event.fire(node.upcast(), CanGc::from_cx(cx));

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
                    CanGc::deprecated_note(),
                );
                navigator.set_gamepad(selected_index as usize, &gamepad, CanGc::deprecated_note());
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
                        gamepad.update_connected(false, gamepad.exposed(), CanGc::deprecated_note());
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
                                                gamepad.notify_event(GamepadEventType::Connected, CanGc::deprecated_note());
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
        cx: &mut JSContext,
        element: Option<DomRoot<Element>>,
        action: EditingActionEvent,
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
            self.fire_clipboard_event(element.clone(), clipboard_event_type, CanGc::from_cx(cx));

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
                    self.fire_clipboard_event(element, ClipboardEventType::Change, CanGc::from_cx(cx));
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
            clipboard_event_type.as_str().into(),
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
        let target = target
            .map(DomRoot::upcast)
            .unwrap_or_else(|| self.target_for_events_following_focus());

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
                    let data = DOMString::from(text_contents);
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

    /// Handle a scroll event triggered by user interactions from the embedder.
    /// <https://drafts.csswg.org/cssom-view/#scrolling-events>
    #[expect(unsafe_code)]
    pub(crate) fn handle_embedder_scroll_event(&self, scrolled_node: ExternalScrollId) {
        // If it is a viewport scroll.
        let document = self.window.Document();
        if scrolled_node.is_root() {
            document.handle_viewport_scroll_event();
        } else {
            // Otherwise, check whether it is for a relevant element within the document. For a `::before` or `::after`
            // pseudo element we follow Gecko or Chromium's behavior to ensure that the event reaches the originating
            // node.
            let node_id = node_id_from_scroll_id(scrolled_node.0 as usize);
            let node = unsafe {
                node::from_untrusted_node_address(UntrustedNodeAddress::from_id(node_id))
            };
            let Some(element) = node
                .inclusive_ancestors(ShadowIncluding::Yes)
                .find_map(DomRoot::downcast::<Element>)
            else {
                return;
            };

            element.handle_scroll_event();
        }
    }

    /// <https://w3c.github.io/uievents/#keydown>
    ///
    /// > If the key is the Enter or (Space) key and the current focus is on a state-changing element,
    /// > the default action MUST be to dispatch a click event, and a DOMActivate event if that event
    /// > type is supported by the user agent.
    pub(crate) fn maybe_dispatch_simulated_click(
        &self,
        node: &Node,
        event: &KeyboardEvent,
        can_gc: CanGc,
    ) -> bool {
        if event.key() != Key::Named(NamedKey::Enter) && event.original_code() != Some(Code::Space)
        {
            return false;
        }

        // Check whether this node is a state-changing element. Note that the specification doesn't
        // seem to have a good definition of what "state-changing" means, so we merely check to
        // see if the element is activatable here.
        if node
            .downcast::<Element>()
            .and_then(Element::as_maybe_activatable)
            .is_none()
        {
            return false;
        }

        node.fire_synthetic_pointer_event_not_trusted(atom!("click"), can_gc);
        true
    }

    pub(crate) fn run_default_keyboard_event_handler(
        &self,
        node: &Node,
        event: &KeyboardEvent,
        can_gc: CanGc,
    ) {
        if event.upcast::<Event>().type_() != atom!("keydown") {
            return;
        }

        if self.maybe_dispatch_simulated_click(node, event, can_gc) {
            return;
        }

        if self.maybe_handle_accesskey(event, can_gc) {
            return;
        }

        let mut is_space = false;
        let scroll = match event.key() {
            Key::Named(NamedKey::ArrowDown) => KeyboardScroll::Down,
            Key::Named(NamedKey::ArrowLeft) => KeyboardScroll::Left,
            Key::Named(NamedKey::ArrowRight) => KeyboardScroll::Right,
            Key::Named(NamedKey::ArrowUp) => KeyboardScroll::Up,
            Key::Named(NamedKey::End) => KeyboardScroll::End,
            Key::Named(NamedKey::Home) => KeyboardScroll::Home,
            Key::Named(NamedKey::PageDown) => KeyboardScroll::PageDown,
            Key::Named(NamedKey::PageUp) => KeyboardScroll::PageUp,
            Key::Character(string) if &string == " " => {
                is_space = true;
                if event.modifiers().contains(Modifiers::SHIFT) {
                    KeyboardScroll::PageUp
                } else {
                    KeyboardScroll::PageDown
                }
            },
            Key::Named(NamedKey::Tab) => {
                // From <https://w3c.github.io/uievents/#keydown>:
                //
                // > If the key is the Tab key, the default action MUST be to shift the document focus
                // > from the currently focused element (if any) to the new focused element, as
                // > described in Focus Event Types
                self.sequential_focus_navigation_via_keyboard_event(event, can_gc);
                return;
            },
            _ => return,
        };

        if !event.modifiers().is_empty() && !is_space {
            return;
        }

        self.do_keyboard_scroll(scroll);
    }

    pub(crate) fn set_sequential_focus_navigation_starting_point(&self, node: &Node) {
        self.sequential_focus_navigation_starting_point
            .set(Some(node));
    }

    pub(crate) fn sequential_focus_navigation_starting_point(&self) -> Option<DomRoot<Node>> {
        self.sequential_focus_navigation_starting_point
            .get()
            .filter(|node| node.is_connected())
    }

    fn sequential_focus_navigation_via_keyboard_event(&self, event: &KeyboardEvent, can_gc: CanGc) {
        let direction = if event.modifiers().contains(Modifiers::SHIFT) {
            SequentialFocusDirection::Backward
        } else {
            SequentialFocusDirection::Forward
        };

        self.sequential_focus_navigation(direction, can_gc);
    }

    /// <<https://html.spec.whatwg.org/multipage/#sequential-focus-navigation:currently-focused-area-of-a-top-level-traversable>
    fn sequential_focus_navigation(&self, direction: SequentialFocusDirection, can_gc: CanGc) {
        // > When the user requests that focus move from the currently focused area of a top-level
        // > traversable to the next or previous focusable area (e.g., as the default action of
        // > pressing the tab key), or when the user requests that focus sequentially move to a
        // > top-level traversable in the first place (e.g., from the browser's location bar), the
        // > user agent must use the following algorithm:

        // > 1. Let starting point be the currently focused area of a top-level traversable, if the
        // > user requested to move focus sequentially from there, or else the top-level traversable
        // > itself, if the user instead requested to move focus from outside the top-level
        // > traversable.
        //
        // TODO: We do not yet implement support for doing sequential focus navigation between traversibles
        // according to the specification, so the implementation is currently adapted to work with a single
        // traversible.
        //
        // Note: Here `None` represents the current traversible.
        let mut starting_point = self
            .window
            .Document()
            .focus_handler()
            .focused_area()
            .element()
            .map(|element| DomRoot::from_ref(element.upcast::<Node>()));

        // > 2. If there is a sequential focus navigation starting point defined and it is inside
        // > starting point, then let starting point be the sequential focus navigation starting point
        // > instead.
        if let Some(sequential_focus_navigation_starting_point) =
            self.sequential_focus_navigation_starting_point()
        {
            if starting_point.as_ref().is_none_or(|starting_point| {
                starting_point.is_ancestor_of(&sequential_focus_navigation_starting_point)
            }) {
                starting_point = Some(sequential_focus_navigation_starting_point);
            }
        }

        // > 3. Let direction be "forward" if the user requested the next control, and "backward" if
        // > the user requested the previous control.
        //
        // Note: This is handled by the `direction` argument to this method.

        // > 4. Loop: Let selection mechanism be "sequential" if starting point is a navigable or if
        // > starting point is in its Document's sequential focus navigation order.
        // > Otherwise, starting point is not in its Document's sequential focus navigation order;
        // > let selection mechanism be "DOM".
        // TODO: Implement this.

        // > 5. Let candidate be the result of running the sequential navigation search algorithm
        // > with starting point, direction, and selection mechanism.
        let candidate = starting_point
            .map(|starting_point| {
                self.find_element_for_tab_focus_following_element(direction, starting_point)
            })
            .unwrap_or_else(|| self.find_first_tab_focusable_element(direction));

        // > 6. If candidate is not null, then run the focusing steps for candidate and return.
        if let Some(candidate) = candidate {
            self.focus_and_scroll_to_element_for_key_event(&candidate, can_gc);
            return;
        }

        // > 7. Otherwise, unset the sequential focus navigation starting point.
        self.sequential_focus_navigation_starting_point.clear();

        // > 8. If starting point is a top-level traversable, or a focusable area in the top-level
        // > traversable, the user agent should transfer focus to its own controls appropriately (if
        // > any), honouring direction, and then return.
        // TODO: Implement this.

        // > 9. Otherwise, starting point is a focusable area in a child navigable. Set starting
        // > point to that child navigable's parent and return to the step labeled loop.
        // TODO: Implement this.
    }

    fn find_element_for_tab_focus_following_element(
        &self,
        direction: SequentialFocusDirection,
        starting_point: DomRoot<Node>,
    ) -> Option<DomRoot<Element>> {
        let root_node = self.window.Document().GetDocumentElement()?;
        let focused_element_tab_index = starting_point
            .downcast::<Element>()
            .and_then(Element::explicitly_set_tab_index)
            .unwrap_or_default();
        let mut winning_node_and_tab_index: Option<(DomRoot<Element>, i32)> = None;
        let mut saw_focused_element = false;

        for node in root_node
            .upcast::<Node>()
            .traverse_preorder(ShadowIncluding::Yes)
        {
            if node == starting_point {
                saw_focused_element = true;
                continue;
            }

            let Some(candidate_element) = DomRoot::downcast::<Element>(node) else {
                continue;
            };
            if !candidate_element.is_sequentially_focusable() {
                continue;
            }

            let candidate_element_tab_index = candidate_element
                .explicitly_set_tab_index()
                .unwrap_or_default();
            let ordering =
                compare_tab_indices(focused_element_tab_index, candidate_element_tab_index);
            match direction {
                SequentialFocusDirection::Forward => {
                    // If moving forward the first element with equal tab index after the current
                    // element is the winner.
                    if saw_focused_element && ordering == Ordering::Equal {
                        return Some(candidate_element);
                    }
                    // If the candidate element does not have a lesser tab index, then discard it.
                    if ordering != Ordering::Less {
                        continue;
                    }
                    let Some((_, winning_tab_index)) = winning_node_and_tab_index else {
                        // If this candidate has a tab index which is one greater than the current
                        // tab index, then we know it is the winner, because we give precedence to
                        // elements earlier in the DOM.
                        if candidate_element_tab_index == focused_element_tab_index + 1 {
                            return Some(candidate_element);
                        }

                        winning_node_and_tab_index =
                            Some((candidate_element, candidate_element_tab_index));
                        continue;
                    };
                    // If the candidate element has a lesser tab index than than the current winner,
                    // then it becomes the winner.
                    if compare_tab_indices(candidate_element_tab_index, winning_tab_index) ==
                        Ordering::Less
                    {
                        winning_node_and_tab_index =
                            Some((candidate_element, candidate_element_tab_index))
                    }
                },
                SequentialFocusDirection::Backward => {
                    // If moving backward the last element with an equal tab index that precedes
                    // the focused element in the DOM is the winner.
                    if !saw_focused_element && ordering == Ordering::Equal {
                        winning_node_and_tab_index =
                            Some((candidate_element, candidate_element_tab_index));
                        continue;
                    }
                    // If the candidate does not have a greater tab index, then discard it.
                    if ordering != Ordering::Greater {
                        continue;
                    }
                    let Some((_, winning_tab_index)) = winning_node_and_tab_index else {
                        winning_node_and_tab_index =
                            Some((candidate_element, candidate_element_tab_index));
                        continue;
                    };
                    // If the candidate element's tab index is not less than the current winner,
                    // then it becomes the new winner. This means that when the tab indices are
                    // equal, we give preference to the last one in DOM order.
                    if compare_tab_indices(candidate_element_tab_index, winning_tab_index) !=
                        Ordering::Less
                    {
                        winning_node_and_tab_index =
                            Some((candidate_element, candidate_element_tab_index))
                    }
                },
            }
        }

        Some(winning_node_and_tab_index?.0)
    }

    fn find_first_tab_focusable_element(
        &self,
        direction: SequentialFocusDirection,
    ) -> Option<DomRoot<Element>> {
        let root_node = self.window.Document().GetDocumentElement()?;
        let mut winning_node_and_tab_index: Option<(DomRoot<Element>, i32)> = None;
        for node in root_node
            .upcast::<Node>()
            .traverse_preorder(ShadowIncluding::Yes)
        {
            let Some(candidate_element) = DomRoot::downcast::<Element>(node) else {
                continue;
            };
            if !candidate_element.is_sequentially_focusable() {
                continue;
            }

            let candidate_element_tab_index = candidate_element
                .explicitly_set_tab_index()
                .unwrap_or_default();
            match direction {
                SequentialFocusDirection::Forward => {
                    // We can immediately return the first time we find an element with the lowest
                    // possible tab index (1). We are guaranteed not to find any lower tab index
                    // and all other equal tab indices are later in the DOM.
                    if candidate_element_tab_index == 1 {
                        return Some(candidate_element);
                    }

                    // Only promote a candidate to the current winner if it has a lesser tab
                    // index than the current winner or there is currently no winer.
                    if winning_node_and_tab_index
                        .as_ref()
                        .is_none_or(|(_, winning_tab_index)| {
                            compare_tab_indices(candidate_element_tab_index, *winning_tab_index) ==
                                Ordering::Less
                        })
                    {
                        winning_node_and_tab_index =
                            Some((candidate_element, candidate_element_tab_index));
                    }
                },
                SequentialFocusDirection::Backward => {
                    // Only promote a candidate to winner if it has tab index equal to or
                    // greater than the winner's tab index. This gives precedence to elements
                    // later in the DOM.
                    if winning_node_and_tab_index
                        .as_ref()
                        .is_none_or(|(_, winning_tab_index)| {
                            compare_tab_indices(candidate_element_tab_index, *winning_tab_index) !=
                                Ordering::Less
                        })
                    {
                        winning_node_and_tab_index =
                            Some((candidate_element, candidate_element_tab_index));
                    }
                },
            }
        }

        Some(winning_node_and_tab_index?.0)
    }

    pub(crate) fn do_keyboard_scroll(&self, scroll: KeyboardScroll) {
        let scroll_axis = match scroll {
            KeyboardScroll::Left | KeyboardScroll::Right => ScrollingBoxAxis::X,
            _ => ScrollingBoxAxis::Y,
        };

        let document = self.window.Document();
        let mut scrolling_box = document
            .focus_handler()
            .focused_area()
            .element()
            .or(self.most_recently_clicked_element.get().as_deref())
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

    /// Fire pointerenter events hierarchically from topmost ancestor to target element.
    /// Fire pointerleave events hierarchically from target element to topmost ancestor.
    /// Used for touch devices that don't support hover.
    #[allow(clippy::too_many_arguments)]
    fn fire_pointer_event_for_touch(
        &self,
        target_element: &Element,
        touch: &Touch,
        pointer_id: i32,
        event_name: &str,
        is_primary: bool,
        input_event: &ConstellationInputEvent,
        hit_test_result: &HitTestResult,
        can_gc: CanGc,
    ) {
        // Collect ancestors from target to root
        let mut targets: Vec<DomRoot<Node>> = vec![];
        let mut current: Option<DomRoot<Node>> = Some(DomRoot::from_ref(target_element.upcast()));
        while let Some(node) = current {
            targets.push(DomRoot::from_ref(&*node));
            current = node.parent_in_flat_tree();
        }

        // Reverse to dispatch from topmost ancestor to target
        if event_name == "pointerenter" {
            targets.reverse();
        }

        for target in targets {
            let pointer_event = touch.to_pointer_event(
                &self.window,
                event_name,
                pointer_id,
                is_primary,
                input_event.active_keyboard_modifiers,
                false,
                Some(hit_test_result.point_in_node),
                can_gc,
            );
            pointer_event
                .upcast::<Event>()
                .fire(target.upcast(), can_gc);
        }
    }

    pub(crate) fn has_assigned_access_key(&self, element: &HTMLElement) -> bool {
        self.access_key_handlers
            .borrow()
            .values()
            .any(|value| &**value == element)
    }

    pub(crate) fn unassign_access_key(&self, element: &HTMLElement) {
        self.access_key_handlers
            .borrow_mut()
            .retain(|_, value| &**value != element)
    }

    pub(crate) fn assign_access_key(&self, element: &HTMLElement, code: Code) {
        let mut access_key_handlers = self.access_key_handlers.borrow_mut();
        // If an element is already assigned this access key, ignore the request.
        access_key_handlers
            .entry(code.into())
            .or_insert(Dom::from_ref(element));
    }

    fn maybe_handle_accesskey(&self, event: &KeyboardEvent, can_gc: CanGc) -> bool {
        #[cfg(target_os = "macos")]
        let access_key_modifiers = Modifiers::CONTROL | Modifiers::ALT;
        #[cfg(not(target_os = "macos"))]
        let access_key_modifiers = Modifiers::SHIFT | Modifiers::ALT;

        if event.modifiers() != access_key_modifiers {
            return false;
        }

        let Ok(code) = Code::from_str(&event.Code().str()) else {
            return false;
        };

        let Some(html_element) = self
            .access_key_handlers
            .borrow()
            .get(&code.into())
            .map(|html_element| html_element.as_rooted())
        else {
            return false;
        };

        // From <https://html.spec.whatwg.org/multipage/#the-accesskey-attribute>:
        // > When the user presses the key combination corresponding to the assigned access key for
        // > an element, if the element defines a command, the command's Hidden State facet is false
        // > (visible), the command's Disabled State facet is also false (enabled), the element is in
        // > a document that has a non-null browsing context, and neither the element nor any of its
        // > ancestors has a hidden attribute specified, then the user agent must trigger the Action
        // > of the command.
        let Ok(command) = InteractiveElementCommand::try_from(&*html_element) else {
            return false;
        };

        if command.disabled() || command.hidden() {
            return false;
        }

        let node = html_element.upcast::<Node>();
        if !node.is_connected() {
            return false;
        }

        for node in node.inclusive_ancestors(ShadowIncluding::Yes) {
            if node
                .downcast::<HTMLElement>()
                .is_some_and(|html_element| html_element.Hidden())
            {
                return false;
            }
        }

        // This behavior is unspecified, but all browsers do this. When activating the element it is
        // focused and scrolled into view.
        self.focus_and_scroll_to_element_for_key_event(html_element.upcast(), can_gc);
        command.perform_action(can_gc);
        true
    }

    fn focus_and_scroll_to_element_for_key_event(&self, element: &Element, can_gc: CanGc) {
        element
            .upcast::<Node>()
            .run_the_focusing_steps(None, can_gc);
        let scroll_axis = ScrollAxisState {
            position: ScrollLogicalPosition::Center,
            requirement: ScrollRequirement::IfNotVisible,
        };
        element.scroll_into_view_with_options(
            ScrollBehavior::Auto,
            scroll_axis,
            scroll_axis,
            None,
            None,
        );
    }
}

/// <https://html.spec.whatwg.org/multipage/#sequential-focus-direction>
///
/// > A sequential focus direction is one of two possible values: "forward", or "backward". They are
/// > used in the below algorithms to describe the direction in which sequential focus travels at the
/// > user's request.
#[derive(Clone, Copy, PartialEq)]
enum SequentialFocusDirection {
    Forward,
    Backward,
}

fn compare_tab_indices(a: i32, b: i32) -> Ordering {
    if a == b {
        Ordering::Equal
    } else if a == 0 {
        Ordering::Greater
    } else if b == 0 {
        Ordering::Less
    } else {
        a.cmp(&b)
    }
}

pub(crate) fn character_to_code(character: char) -> Option<Code> {
    Some(match character.to_ascii_lowercase() {
        '`' => Code::Backquote,
        '\\' => Code::Backslash,
        '[' | '{' => Code::BracketLeft,
        ']' | '}' => Code::BracketRight,
        ',' | '<' => Code::Comma,
        '0' => Code::Digit0,
        '1' => Code::Digit1,
        '2' => Code::Digit2,
        '3' => Code::Digit3,
        '4' => Code::Digit4,
        '5' => Code::Digit5,
        '6' => Code::Digit6,
        '7' => Code::Digit7,
        '8' => Code::Digit8,
        '9' => Code::Digit9,
        '=' => Code::Equal,
        'a' => Code::KeyA,
        'b' => Code::KeyB,
        'c' => Code::KeyC,
        'd' => Code::KeyD,
        'e' => Code::KeyE,
        'f' => Code::KeyF,
        'g' => Code::KeyG,
        'h' => Code::KeyH,
        'i' => Code::KeyI,
        'j' => Code::KeyJ,
        'k' => Code::KeyK,
        'l' => Code::KeyL,
        'm' => Code::KeyM,
        'n' => Code::KeyN,
        'o' => Code::KeyO,
        'p' => Code::KeyP,
        'q' => Code::KeyQ,
        'r' => Code::KeyR,
        's' => Code::KeyS,
        't' => Code::KeyT,
        'u' => Code::KeyU,
        'v' => Code::KeyV,
        'w' => Code::KeyW,
        'x' => Code::KeyX,
        'y' => Code::KeyY,
        'z' => Code::KeyZ,
        '-' => Code::Minus,
        '.' => Code::Period,
        '\'' | '"' => Code::Quote,
        ';' => Code::Semicolon,
        '/' => Code::Slash,
        ' ' => Code::Space,
        _ => return None,
    })
}
