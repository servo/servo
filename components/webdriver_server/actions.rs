/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;
use std::thread;
use std::time::{Duration, Instant};

use base::id::BrowsingContextId;
use crossbeam_channel::Select;
use embedder_traits::{
    InputEvent, KeyboardEvent, MouseButtonAction, MouseButtonEvent, MouseMoveEvent,
    WebDriverCommandMsg, WebDriverScriptCommand, WheelDelta, WheelEvent, WheelMode,
};
use ipc_channel::ipc;
use keyboard_types::webdriver::KeyInputState;
use log::info;
use rustc_hash::FxHashSet;
use webdriver::actions::{
    ActionSequence, ActionsType, GeneralAction, KeyAction, KeyActionItem, KeyDownAction,
    KeyUpAction, NullActionItem, PointerAction, PointerActionItem, PointerDownAction,
    PointerMoveAction, PointerOrigin, PointerType, PointerUpAction, WheelAction, WheelActionItem,
    WheelScrollAction,
};
use webdriver::command::ActionsParameters;
use webdriver::error::{ErrorStatus, WebDriverError};
use webrender_api::units::DevicePoint;

use crate::{Handler, VerifyBrowsingContextIsOpen, WebElement, wait_for_ipc_response};

// Interval between wheelScroll and pointerMove increments in ms, based on common vsync
static POINTERMOVE_INTERVAL: u64 = 17;
static WHEELSCROLL_INTERVAL: u64 = 17;

// https://262.ecma-international.org/6.0/#sec-number.max_safe_integer
static MAXIMUM_SAFE_INTEGER: u64 = 9_007_199_254_740_991;

// A single action, corresponding to an `action object` in the spec.
// In the spec, `action item` refers to a plain JSON object.
// However, we use the name ActionItem here
// to be consistent with type names from webdriver crate.
#[derive(Debug, PartialEq)]
pub(crate) enum ActionItem {
    Null(NullActionItem),
    Key(KeyActionItem),
    Pointer(PointerActionItem),
    Wheel(WheelActionItem),
}

/// A set of actions with multiple sources executed within a single tick.
/// The `id` is used to identify the source of the actions.
pub(crate) type TickActions = Vec<(String, ActionItem)>;

/// Consumed by the `dispatch_actions` method.
pub(crate) type ActionsByTick = Vec<TickActions>;

/// <https://w3c.github.io/webdriver/#dfn-input-source-state>
pub(crate) enum InputSourceState {
    Null,
    Key(KeyInputState),
    Pointer(PointerInputState),
    Wheel,
}

/// <https://w3c.github.io/webdriver/#dfn-pointer-input-source>
/// TODO: subtype is used for <https://w3c.github.io/webdriver/#dfn-get-a-pointer-id>
/// Need to add pointer-id to the following struct
#[allow(dead_code)]
pub(crate) struct PointerInputState {
    subtype: PointerType,
    pressed: FxHashSet<u64>,
    pub(crate) pointer_id: u32,
    x: f64,
    y: f64,
}

impl PointerInputState {
    /// <https://w3c.github.io/webdriver/#dfn-create-a-pointer-input-source>
    pub(crate) fn new(subtype: PointerType, pointer_ids: FxHashSet<u32>) -> PointerInputState {
        PointerInputState {
            subtype,
            pressed: FxHashSet::default(),
            pointer_id: Self::get_pointer_id(subtype, pointer_ids),
            x: 0.0,
            y: 0.0,
        }
    }

    /// <https://w3c.github.io/webdriver/#dfn-get-a-pointer-id>
    fn get_pointer_id(subtype: PointerType, pointer_ids: FxHashSet<u32>) -> u32 {
        // Step 2 - 4: Let pointer ids be all the values in input state map which is
        // pointer input source. This is already done and passed by the caller.
        if subtype == PointerType::Mouse {
            for id in 0..=1 {
                if !pointer_ids.contains(&id) {
                    return id;
                }
            }
        }

        // We are dealing with subtype other than mouse, which has minimum id 2.
        1 + pointer_ids.into_iter().max().unwrap_or(1)
    }
}

/// <https://w3c.github.io/webdriver/#dfn-computing-the-tick-duration>
fn compute_tick_duration(tick_actions: &TickActions) -> u64 {
    // Step 1. Let max duration be 0.
    // Step 2. For each action in tick actions:
    tick_actions
        .iter()
        .filter_map(|(_, action_item)| {
            // If action object has subtype property set to "pause" or
            // action object has type property set to "pointer" and subtype property set to "pointerMove",
            // or action object has type property set to "wheel" and subtype property set to "scroll",
            // let duration be equal to the duration property of action object.
            match action_item {
                ActionItem::Null(NullActionItem::General(GeneralAction::Pause(pause_action))) |
                ActionItem::Key(KeyActionItem::General(GeneralAction::Pause(pause_action))) |
                ActionItem::Pointer(PointerActionItem::General(GeneralAction::Pause(
                    pause_action,
                ))) |
                ActionItem::Wheel(WheelActionItem::General(GeneralAction::Pause(pause_action))) => {
                    pause_action.duration
                },
                ActionItem::Pointer(PointerActionItem::Pointer(PointerAction::Move(action))) => {
                    action.duration
                },
                ActionItem::Wheel(WheelActionItem::Wheel(WheelAction::Scroll(action))) => {
                    action.duration
                },
                _ => None,
            }
        })
        .max()
        .unwrap_or(0)
}

impl Handler {
    /// <https://w3c.github.io/webdriver/#dfn-dispatch-actions>
    pub(crate) fn dispatch_actions(
        &mut self,
        actions_by_tick: ActionsByTick,
        browsing_context: BrowsingContextId,
    ) -> Result<(), ErrorStatus> {
        // Step 1. Wait for an action queue token with input state.
        // Step 2. Let actions result be the result of dispatch actions inner.
        // Step 3. Dequeue input state's actions queue.
        // Step 4. Return actions result.
        self.dispatch_actions_inner(actions_by_tick, browsing_context)
    }

    /// <https://w3c.github.io/webdriver/#dfn-dispatch-actions-inner>
    fn dispatch_actions_inner(
        &mut self,
        actions_by_tick: ActionsByTick,
        browsing_context: BrowsingContextId,
    ) -> Result<(), ErrorStatus> {
        // Step 1. For each item tick actions in actions by tick
        for tick_actions in actions_by_tick.iter() {
            // Step 1.1. If browsing context is no longer open,
            // return error with error code no such window.
            self.verify_browsing_context_is_open(browsing_context)
                .map_err(|e| e.error)?;
            // Step 1.2. Let tick duration be the result of
            // computing the tick duration with argument tick actions.
            let tick_duration = compute_tick_duration(tick_actions);

            // FIXME: This is out of spec, but the test `perform_actions/invalid.py` requires
            // that duration more than `MAXIMUM_SAFE_INTEGER` is considered invalid.
            if tick_duration > MAXIMUM_SAFE_INTEGER {
                return Err(ErrorStatus::InvalidArgument);
            }

            let now = Instant::now();

            // Step 1.3. Try to dispatch tick actions
            self.dispatch_tick_actions(tick_actions, tick_duration)?;

            // Step 1.4. Wait for
            // The user agent event loop has spun enough times to process the DOM events
            // generated by the last invocation of the dispatch tick actions steps.
            self.wait_for_user_agent_handling_complete()?;
            // At least tick duration milliseconds have passed.
            let elapsed = now.elapsed().as_millis() as u64;
            if elapsed < tick_duration {
                let sleep_duration = tick_duration - elapsed;
                thread::sleep(Duration::from_millis(sleep_duration));
            }
        }

        // Step 2. Return success with data null.
        info!("Dispatch actions completed successfully");
        Ok(())
    }

    fn wait_for_user_agent_handling_complete(&self) -> Result<(), ErrorStatus> {
        let mut pending_event_receivers =
            std::mem::take(&mut *self.pending_input_event_receivers.borrow_mut());

        while !pending_event_receivers.is_empty() {
            let mut select = Select::new();
            for receiver in &pending_event_receivers {
                select.recv(receiver);
            }

            let operation = select.select();
            let index = operation.index();
            let _ = operation.recv(&pending_event_receivers[index]);

            pending_event_receivers.remove(index);
        }

        self.num_pending_actions.set(0);

        Ok(())
    }

    /// <https://w3c.github.io/webdriver/#dfn-dispatch-tick-actions>
    fn dispatch_tick_actions(
        &mut self,
        tick_actions: &TickActions,
        tick_duration: u64,
    ) -> Result<(), ErrorStatus> {
        // Step 1. For each action object in tick actions:
        // Step 1.1. Let input_id be the value of the id property of action object.
        for (input_id, action) in tick_actions.iter() {
            // Step 6. Let subtype be action object's subtype.
            // Steps 7, 8. Try to run specific algorithm based on the action type.
            match action {
                ActionItem::Null(_) | ActionItem::Key(KeyActionItem::General(_)) => {
                    self.dispatch_pause_action(input_id);
                },
                ActionItem::Key(KeyActionItem::Key(KeyAction::Down(keydown_action))) => {
                    self.dispatch_keydown_action(input_id, keydown_action);
                    // Step 9. If subtype is "keyDown", append a copy of action
                    // object with the subtype property changed to "keyUp" to
                    // input state's input cancel list.
                    self.session_mut().unwrap().input_cancel_list.push((
                        input_id.clone(),
                        ActionItem::Key(KeyActionItem::Key(KeyAction::Up(KeyUpAction {
                            value: keydown_action.value.clone(),
                        }))),
                    ));
                },
                ActionItem::Key(KeyActionItem::Key(KeyAction::Up(keyup_action))) => {
                    self.dispatch_keyup_action(input_id, keyup_action);
                },
                ActionItem::Pointer(PointerActionItem::General(_)) => {
                    self.dispatch_pause_action(input_id);
                },
                ActionItem::Pointer(PointerActionItem::Pointer(PointerAction::Down(
                    pointer_down_action,
                ))) => {
                    self.dispatch_pointerdown_action(input_id, pointer_down_action);
                    // Step 10. If subtype is "pointerDown", append a copy of action
                    // object with the subtype property changed to "pointerUp" to
                    // input state's input cancel list.
                    self.session_mut().unwrap().input_cancel_list.push((
                        input_id.clone(),
                        ActionItem::Pointer(PointerActionItem::Pointer(PointerAction::Up(
                            PointerUpAction {
                                button: pointer_down_action.button,
                                ..Default::default()
                            },
                        ))),
                    ));
                },
                ActionItem::Pointer(PointerActionItem::Pointer(PointerAction::Move(
                    pointer_move_action,
                ))) => {
                    self.dispatch_pointermove_action(input_id, pointer_move_action, tick_duration)?;
                },
                ActionItem::Pointer(PointerActionItem::Pointer(PointerAction::Up(
                    pointer_up_action,
                ))) => {
                    self.dispatch_pointerup_action(input_id, pointer_up_action);
                },
                ActionItem::Wheel(WheelActionItem::General(_)) => {
                    self.dispatch_pause_action(input_id);
                },
                ActionItem::Wheel(WheelActionItem::Wheel(WheelAction::Scroll(scroll_action))) => {
                    self.dispatch_scroll_action(input_id, scroll_action, tick_duration)?;
                },
                _ => {},
            }
        }

        Ok(())
    }

    /// <https://w3c.github.io/webdriver/#dfn-dispatch-a-pause-action>
    fn dispatch_pause_action(&mut self, source_id: &str) {
        self.input_state_table_mut()
            .entry(source_id.to_string())
            .or_insert(InputSourceState::Null);
    }

    /// <https://w3c.github.io/webdriver/#dfn-dispatch-a-keydown-action>
    fn dispatch_keydown_action(&mut self, source_id: &str, action: &KeyDownAction) {
        let raw_key = action.value.chars().next().unwrap();
        let key_input_state = match self.input_state_table_mut().get_mut(source_id).unwrap() {
            InputSourceState::Key(key_input_state) => key_input_state,
            _ => unreachable!(),
        };

        let keyboard_event = key_input_state.dispatch_keydown(raw_key);

        // Step 12: Perform implementation-specific action dispatch steps on browsing
        // context equivalent to pressing a key on the keyboard in accordance with the
        // requirements of [UI-EVENTS], and producing the following events, as
        // appropriate, with the specified properties. This will always produce events
        // including at least a keyDown event.
        self.send_blocking_input_event_to_embedder(InputEvent::Keyboard(KeyboardEvent::new(
            keyboard_event,
        )));
    }

    /// <https://w3c.github.io/webdriver/#dfn-dispatch-a-keyup-action>
    fn dispatch_keyup_action(&mut self, source_id: &str, action: &KeyUpAction) {
        let session = self.session_mut().unwrap();

        // Remove the last matching keyUp from `[input_cancel_list]` due to bugs in spec
        // See https://github.com/w3c/webdriver/issues/1905 &&
        // https://github.com/servo/servo/issues/37579#issuecomment-2990762713
        let input_cancel_list = &mut session.input_cancel_list;
        if let Some(pos) = input_cancel_list.iter().rposition(|(id, item)| {
            id == source_id &&
                matches!(item,
                        ActionItem::Key(KeyActionItem::Key(KeyAction::Up(KeyUpAction { value })))
                    if *value == action.value )
        }) {
            info!("dispatch_keyup_action: removing last matching keyup from input_cancel_list");
            input_cancel_list.remove(pos);
        }

        let raw_key = action.value.chars().next().unwrap();
        let key_input_state = match session.input_state_table.get_mut(source_id).unwrap() {
            InputSourceState::Key(key_input_state) => key_input_state,
            _ => unreachable!(),
        };

        // Step 12: Perform implementation-specific action dispatch steps on browsing
        // context equivalent to releasing a key on the keyboard in accordance with the
        // requirements of [UI-EVENTS], ...
        let Some(keyboard_event) = key_input_state.dispatch_keyup(raw_key) else {
            return;
        };
        self.send_blocking_input_event_to_embedder(InputEvent::Keyboard(KeyboardEvent::new(
            keyboard_event,
        )));
    }

    /// <https://w3c.github.io/webdriver/#dfn-dispatch-a-pointerdown-action>
    fn dispatch_pointerdown_action(&mut self, source_id: &str, action: &PointerDownAction) {
        let pointer_input_state = self.get_pointer_input_state_mut(source_id);
        // Step 3. If the source's pressed property contains button return success with data null.
        if pointer_input_state.pressed.contains(&action.button) {
            return;
        }

        let PointerInputState { x, y, .. } = *pointer_input_state;
        // Step 6. Add button to the set corresponding to source's pressed property
        pointer_input_state.pressed.insert(action.button);
        // Step 7 - 15: Variable namings already done.

        // Step 16. Perform implementation-specific action dispatch steps
        // TODO: We have not considered pen/touch pointer type
        self.send_blocking_input_event_to_embedder(InputEvent::MouseButton(MouseButtonEvent::new(
            MouseButtonAction::Down,
            action.button.into(),
            DevicePoint::new(x as f32, y as f32),
        )));

        // Step 17. Return success with data null.
    }

    /// <https://w3c.github.io/webdriver/#dfn-dispatch-a-pointerup-action>
    fn dispatch_pointerup_action(&mut self, source_id: &str, action: &PointerUpAction) {
        let pointer_input_state = self.get_pointer_input_state_mut(source_id);
        // Step 3. If the source's pressed property does not contain button, return success with data null.
        if !pointer_input_state.pressed.contains(&action.button) {
            return;
        }

        // Step 6. Remove button from the set corresponding to source's pressed property,
        pointer_input_state.pressed.remove(&action.button);
        let PointerInputState { x, y, .. } = *pointer_input_state;

        // Remove matching pointerUp(must be unique) from `[input_cancel_list]` due to bugs in spec
        // See https://github.com/w3c/webdriver/issues/1905 &&
        // https://github.com/servo/servo/issues/37579#issuecomment-2990762713
        let input_cancel_list = &mut self.session_mut().unwrap().input_cancel_list;
        if let Some(pos) = input_cancel_list.iter().position(|(id, item)| {
            id == source_id &&
                matches!(item, ActionItem::Pointer(PointerActionItem::Pointer(PointerAction::Up(
                    PointerUpAction { button, .. },
                ))) if *button == action.button )
        }) {
            info!("dispatch_pointerup_action: removing matching pointerup from input_cancel_list");
            input_cancel_list.remove(pos);
        }

        // Step 7. Perform implementation-specific action dispatch steps
        self.send_blocking_input_event_to_embedder(InputEvent::MouseButton(MouseButtonEvent::new(
            MouseButtonAction::Up,
            action.button.into(),
            DevicePoint::new(x as f32, y as f32),
        )));

        // Step 8. Return success with data null.
    }

    /// <https://w3c.github.io/webdriver/#dfn-dispatch-a-pointermove-action>
    fn dispatch_pointermove_action(
        &mut self,
        source_id: &str,
        action: &PointerMoveAction,
        tick_duration: u64,
    ) -> Result<(), ErrorStatus> {
        let tick_start = Instant::now();

        // Step 1. Let x offset be equal to the x property of action object.
        let x_offset = action.x;

        // Step 2. Let y offset be equal to the y property of action object.
        let y_offset = action.y;

        // Step 3. Let origin be equal to the origin property of action object.
        let origin = &action.origin;

        // Step 4. Let (x, y) be the result of trying to get coordinates relative to an origin
        // with source, x offset, y offset, origin, browsing context, and actions options.

        let (x, y) = self.get_origin_relative_coordinates(origin, x_offset, y_offset, source_id)?;

        // Step 5. If x is less than 0 or greater than the width of the viewport in CSS pixels,
        // then return error with error code move target out of bounds.
        // Step 6. If y is less than 0 or greater than the height of the viewport in CSS pixels,
        // then return error with error code move target out of bounds.
        self.check_viewport_bound(x, y)?;

        // Step 7. Let duration be equal to action object's duration property
        // if it is not undefined, or tick duration otherwise.
        let duration = match action.duration {
            Some(duration) => duration,
            None => tick_duration,
        };

        // Step 8. If duration is greater than 0 and inside any implementation-defined bounds,
        // asynchronously wait for an implementation defined amount of time to pass.
        if duration > 0 {
            thread::sleep(Duration::from_millis(POINTERMOVE_INTERVAL));
        }

        let (start_x, start_y) = {
            let pointer_input_state = self.get_pointer_input_state(source_id);
            (pointer_input_state.x, pointer_input_state.y)
        };

        // Step 9 - 18
        // Perform a pointer move with arguments source, global key state, duration, start x, start y,
        // x, y, width, height, pressure, tangentialPressure, tiltX, tiltY, twist, altitudeAngle, azimuthAngle.
        // TODO: We have not considered pen/touch pointer type
        self.perform_pointer_move(source_id, duration, start_x, start_y, x, y, tick_start);

        // Step 19. Return success with data null.
        Ok(())
    }

    /// <https://w3c.github.io/webdriver/#dfn-perform-a-pointer-move>
    #[allow(clippy::too_many_arguments)]
    fn perform_pointer_move(
        &mut self,
        source_id: &str,
        duration: u64,
        start_x: f64,
        start_y: f64,
        target_x: f64,
        target_y: f64,
        tick_start: Instant,
    ) {
        loop {
            // Step 1. Let time delta be the time since the beginning of the
            // current tick, measured in milliseconds on a monotonic clock.
            let time_delta = tick_start.elapsed().as_millis();

            // Step 2. Let duration ratio be the ratio of time delta and duration,
            // if duration is greater than 0, or 1 otherwise.
            let duration_ratio = if duration > 0 {
                time_delta as f64 / duration as f64
            } else {
                1.0
            };

            // Step 3. If duration ratio is 1, or close enough to 1 that the
            // implementation will not further subdivide the move action,
            // let last be true. Otherwise let last be false.
            let last = 1.0 - duration_ratio < 0.001;

            // Step 4. If last is true, let x equal target x and y equal target y.
            // Otherwise
            // let x equal an approximation to duration ratio × (target x - start x) + start x,
            // and y equal an approximation to duration ratio × (target y - start y) + start y.
            let (x, y) = if last {
                (target_x, target_y)
            } else {
                (
                    duration_ratio * (target_x - start_x) + start_x,
                    duration_ratio * (target_y - start_y) + start_y,
                )
            };

            // Step 5 - 6: Let current x/y equal the x/y property of input state.
            let (current_x, current_y) = {
                let pointer_input_state = self.get_pointer_input_state(source_id);
                (pointer_input_state.x, pointer_input_state.y)
            };

            // Step 7. If x != current x or y != current y, run the following steps:
            // FIXME: Actually "last" should not be checked here based on spec.
            if x != current_x || y != current_y || last {
                // Step 7.1. Let buttons be equal to input state's buttons property.
                // Step 7.2. Perform implementation-specific action dispatch steps
                let input_event = InputEvent::MouseMove(MouseMoveEvent::new(DevicePoint::new(
                    x as f32, y as f32,
                )));
                if last {
                    self.send_blocking_input_event_to_embedder(input_event);
                } else {
                    self.send_input_event_to_embedder(input_event);
                }

                // Step 7.3. Let input state's x property equal x and y property equal y.
                let pointer_input_state = self.get_pointer_input_state_mut(source_id);
                pointer_input_state.x = x;
                pointer_input_state.y = y;
            }

            // Step 8. If last is true, return.
            if last {
                return;
            }

            // Step 9. Run the following substeps in parallel:
            // Step 9.1. Asynchronously wait for an implementationdefined amount of time to pass
            thread::sleep(Duration::from_millis(POINTERMOVE_INTERVAL));

            // Step 9.2. Perform a pointer move with arguments
            // input state, duration, start x, start y, target x, target y.
            // Notice that this simply repeat what we have done until last is true.
        }
    }

    /// <https://w3c.github.io/webdriver/#dfn-dispatch-a-scroll-action>
    fn dispatch_scroll_action(
        &self,
        source_id: &str,
        action: &WheelScrollAction,
        tick_duration: u64,
    ) -> Result<(), ErrorStatus> {
        // TODO: We should verify each variable when processing a wheel action.
        // <https://w3c.github.io/webdriver/#dfn-process-a-wheel-action>

        let tick_start = Instant::now();

        // Step 1. Let x offset be equal to the x property of action object.
        let Some(x_offset) = action.x else {
            return Err(ErrorStatus::InvalidArgument);
        };

        // Step 2. Let y offset be equal to the y property of action object.
        let Some(y_offset) = action.y else {
            return Err(ErrorStatus::InvalidArgument);
        };

        // Step 3. Let origin be equal to the origin property of action object.
        let origin = &action.origin;

        // Pointer origin isn't currently supported for wheel input source
        // See: https://github.com/w3c/webdriver/issues/1758

        if origin == &PointerOrigin::Pointer {
            return Err(ErrorStatus::InvalidArgument);
        }

        // Step 4. Let (x, y) be the result of trying to get coordinates relative to an origin
        // with source, x offset, y offset, origin, browsing context, and actions options.
        let (x, y) =
            self.get_origin_relative_coordinates(origin, x_offset as _, y_offset as _, source_id)?;

        // Step 5. If x is less than 0 or greater than the width of the viewport in CSS pixels,
        // then return error with error code move target out of bounds.
        // Step 6. If y is less than 0 or greater than the height of the viewport in CSS pixels,
        // then return error with error code move target out of bounds.
        self.check_viewport_bound(x, y)?;

        // Step 7. Let delta x be equal to the deltaX property of action object.
        let Some(delta_x) = action.deltaX else {
            return Err(ErrorStatus::InvalidArgument);
        };

        // Step 8. Let delta y be equal to the deltaY property of action object.
        let Some(delta_y) = action.deltaY else {
            return Err(ErrorStatus::InvalidArgument);
        };

        // Step 9. Let duration be equal to action object's duration property
        // if it is not undefined, or tick duration otherwise.
        let duration = match action.duration {
            Some(duration) => duration,
            None => tick_duration,
        };

        // Step 10. If duration is greater than 0 and inside any implementation-defined bounds,
        // asynchronously wait for an implementation defined amount of time to pass.
        if duration > 0 {
            thread::sleep(Duration::from_millis(WHEELSCROLL_INTERVAL));
        }

        // Step 11. Perform a scroll with arguments global key state, duration, x, y, delta x, delta y, 0, 0.
        self.perform_scroll(
            duration,
            x,
            y,
            delta_x as _,
            delta_y as _,
            0.0,
            0.0,
            tick_start,
        );

        // Step 12. Return success with data null.
        Ok(())
    }

    /// <https://w3c.github.io/webdriver/#dfn-perform-a-scroll>
    #[allow(clippy::too_many_arguments)]
    fn perform_scroll(
        &self,
        duration: u64,
        x: f64,
        y: f64,
        target_delta_x: f64,
        target_delta_y: f64,
        mut curr_delta_x: f64,
        mut curr_delta_y: f64,
        tick_start: Instant,
    ) {
        loop {
            // Step 1. Let time delta be the time since the beginning of the current tick,
            // measured in milliseconds on a monotonic clock.
            let time_delta = tick_start.elapsed().as_millis();

            // Step 2. Let duration ratio be the ratio of time delta and duration,
            // if duration is greater than 0, or 1 otherwise.
            let duration_ratio = if duration > 0 {
                time_delta as f64 / duration as f64
            } else {
                1.0
            };

            // Step 3. If duration ratio is 1, or close enough to 1 that
            // the implementation will not further subdivide the move action,
            // let last be true. Otherwise let last be false.
            let last = 1.0 - duration_ratio < 0.001;

            // Step 4. If last is true,
            // let delta x equal target delta x - current delta x and delta y equal target delta y - current delta y.
            // Otherwise
            // let delta x equal an approximation to duration ratio × target delta x - current delta x,
            // and delta y equal an approximation to duration ratio × target delta y - current delta y.
            let (delta_x, delta_y) = if last {
                (target_delta_x - curr_delta_x, target_delta_y - curr_delta_y)
            } else {
                (
                    duration_ratio * target_delta_x - curr_delta_x,
                    duration_ratio * target_delta_y - curr_delta_y,
                )
            };

            // Step 5. If delta x != 0 or delta y != 0, run the following steps:
            // Actually "last" should not be checked here based on spec.
            // However, we need to send the webdriver id at the final perform.
            if delta_x != 0.0 || delta_y != 0.0 || last {
                // Step 5.1. Perform implementation-specific action dispatch steps
                let delta = WheelDelta {
                    x: -delta_x,
                    y: -delta_y,
                    z: 0.0,
                    mode: WheelMode::DeltaPixel,
                };
                let point = DevicePoint::new(x as f32, y as f32);
                let input_event = InputEvent::Wheel(WheelEvent::new(delta, point));
                if last {
                    self.send_blocking_input_event_to_embedder(input_event);
                } else {
                    self.send_input_event_to_embedder(input_event);
                }

                // Step 5.2. Let current delta x property equal delta x + current delta x
                // and current delta y property equal delta y + current delta y.
                curr_delta_x += delta_x;
                curr_delta_y += delta_y;
            }

            // Step 6. If last is true, return.
            if last {
                return;
            }

            // Step 7
            // TODO: The two steps should be done in parallel
            // 7.1. Asynchronously wait for an implementation defined amount of time to pass.
            thread::sleep(Duration::from_millis(WHEELSCROLL_INTERVAL));
            // 7.2. Perform a scroll with arguments duration, x, y, target delta x,
            // target delta y, current delta x, current delta y.
            // Notice that this simply repeat what we have done until last is true.
        }
    }

    /// Verify that the given coordinates are within the boundary of the viewport.
    /// If x or y is less than 0 or greater than the width of the viewport in CSS pixels,
    /// then return error with error code move target out of bounds.
    fn check_viewport_bound(&self, x: f64, y: f64) -> Result<(), ErrorStatus> {
        if x < 0.0 || y < 0.0 {
            return Err(ErrorStatus::MoveTargetOutOfBounds);
        }
        let (sender, receiver) = ipc::channel().unwrap();
        let cmd_msg = WebDriverCommandMsg::GetViewportSize(self.verified_webview_id(), sender);
        self.send_message_to_embedder(cmd_msg)
            .map_err(|_| ErrorStatus::UnknownError)?;

        let viewport_size = match wait_for_ipc_response(receiver) {
            Ok(response) => response,
            Err(WebDriverError { error, .. }) => return Err(error),
        };
        if x > viewport_size.width.into() || y > viewport_size.height.into() {
            Err(ErrorStatus::MoveTargetOutOfBounds)
        } else {
            Ok(())
        }
    }

    /// <https://w3c.github.io/webdriver/#dfn-get-coordinates-relative-to-an-origin>
    fn get_origin_relative_coordinates(
        &self,
        origin: &PointerOrigin,
        x_offset: f64,
        y_offset: f64,
        source_id: &str,
    ) -> Result<(f64, f64), ErrorStatus> {
        match origin {
            PointerOrigin::Viewport => Ok((x_offset, y_offset)),
            PointerOrigin::Pointer => {
                // Step 1. Let start x be equal to the x property of source.
                // Step 2. Let start y be equal to the y property of source.
                let (start_x, start_y) = {
                    let pointer_input_state = self.get_pointer_input_state(source_id);
                    (pointer_input_state.x, pointer_input_state.y)
                };
                // Step 3. Let x equal start x + x offset and y equal start y + y offset.
                Ok((start_x + x_offset, start_y + y_offset))
            },
            PointerOrigin::Element(web_element) => {
                // Steps 1 - 3
                let (x_element, y_element) = self.get_element_in_view_center_point(web_element)?;
                // Step 4. Let x equal x element + x offset, and y equal y element + y offset.
                Ok((x_element as f64 + x_offset, y_element as f64 + y_offset))
            },
        }
    }

    /// <https://w3c.github.io/webdriver/#dfn-center-point>
    fn get_element_in_view_center_point(
        &self,
        web_element: &WebElement,
    ) -> Result<(i64, i64), ErrorStatus> {
        let (sender, receiver) = ipc::channel().unwrap();
        // Step 1. Let element be the result of trying to run actions options'
        // get element origin steps with origin and browsing context.
        self.browsing_context_script_command(
            WebDriverScriptCommand::GetElementInViewCenterPoint(web_element.to_string(), sender),
            VerifyBrowsingContextIsOpen::No,
        )
        .unwrap();

        // Step 2. If element is null, return error with error code no such element.
        let response = match wait_for_ipc_response(receiver) {
            Ok(response) => response,
            Err(WebDriverError { error, .. }) => return Err(error),
        };

        // Step 3. Let x element and y element be the result of calculating the in-view center point of element.
        match response? {
            Some(point) => Ok(point),
            None => Err(ErrorStatus::UnknownError),
        }
    }

    /// <https://w3c.github.io/webdriver/#dfn-extract-an-action-sequence>
    pub(crate) fn extract_an_action_sequence(
        &mut self,
        params: ActionsParameters,
    ) -> ActionsByTick {
        // Step 1. Let actions be the result of getting a property named "actions" from parameters.
        // Step 2. (ignored because params is already validated earlier). If actions is not a list,
        // return an error with status InvalidArgument.
        self.actions_by_tick_from_sequence(params.actions)
    }

    pub(crate) fn actions_by_tick_from_sequence(
        &mut self,
        actions: Vec<ActionSequence>,
    ) -> ActionsByTick {
        // Step 3. Let actions by tick be an empty list.
        let mut actions_by_tick: ActionsByTick = Vec::new();

        // Step 4. For each value action sequence corresponding to an indexed property in actions
        for action_sequence in actions {
            // Store id before moving action_sequence
            let id = action_sequence.id.clone();
            // Step 4.1. Let source actions be the result of trying to process an input source action sequence
            let source_actions = self.process_an_input_source_action_sequence(action_sequence);

            // Step 4.2.2. Ensure we have enough ticks to hold all actions
            while actions_by_tick.len() < source_actions.len() {
                actions_by_tick.push(Vec::new());
            }

            // Step 4.2.3. Append action to the List at index i in actions by tick.
            for (tick_index, action_item) in source_actions.into_iter().enumerate() {
                actions_by_tick[tick_index].push((id.clone(), action_item));
            }
        }

        actions_by_tick
    }

    /// <https://w3c.github.io/webdriver/#dfn-process-an-input-source-action-sequence>
    fn process_an_input_source_action_sequence(
        &mut self,
        action_sequence: ActionSequence,
    ) -> Vec<ActionItem> {
        // Step 2. Let id be the value of the id property of action sequence.
        let id = action_sequence.id.clone();
        match action_sequence.actions {
            ActionsType::Null {
                actions: null_actions,
            } => {
                self.input_state_table_mut()
                    .entry(id)
                    .or_insert(InputSourceState::Null);
                null_actions.into_iter().map(ActionItem::Null).collect()
            },
            ActionsType::Key {
                actions: key_actions,
            } => {
                self.input_state_table_mut()
                    .entry(id)
                    .or_insert(InputSourceState::Key(KeyInputState::new()));
                key_actions.into_iter().map(ActionItem::Key).collect()
            },
            ActionsType::Pointer {
                parameters: _,
                actions: pointer_actions,
            } => {
                let pointer_ids = self.session().unwrap().pointer_ids();
                self.input_state_table_mut()
                    .entry(id)
                    .or_insert(InputSourceState::Pointer(PointerInputState::new(
                        PointerType::Mouse,
                        pointer_ids,
                    )));
                pointer_actions
                    .into_iter()
                    .map(ActionItem::Pointer)
                    .collect()
            },
            ActionsType::Wheel {
                actions: wheel_actions,
            } => {
                self.input_state_table_mut()
                    .entry(id)
                    .or_insert(InputSourceState::Wheel);
                wheel_actions.into_iter().map(ActionItem::Wheel).collect()
            },
        }
    }

    fn input_state_table_mut(&mut self) -> &mut HashMap<String, InputSourceState> {
        &mut self.session_mut().unwrap().input_state_table
    }

    fn get_pointer_input_state_mut(&mut self, source_id: &str) -> &mut PointerInputState {
        let InputSourceState::Pointer(pointer_input_state) =
            self.input_state_table_mut().get_mut(source_id).unwrap()
        else {
            unreachable!();
        };
        pointer_input_state
    }

    fn get_pointer_input_state(&self, source_id: &str) -> &PointerInputState {
        let InputSourceState::Pointer(pointer_input_state) = self
            .session()
            .unwrap()
            .input_state_table
            .get(source_id)
            .unwrap()
        else {
            unreachable!();
        };
        pointer_input_state
    }
}
