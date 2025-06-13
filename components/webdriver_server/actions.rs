/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::{HashMap, HashSet};
use std::thread;
use std::time::{Duration, Instant};

use base::id::BrowsingContextId;
use constellation_traits::EmbedderToConstellationMessage;
use embedder_traits::{MouseButtonAction, WebDriverCommandMsg, WebDriverScriptCommand};
use ipc_channel::ipc;
use keyboard_types::webdriver::KeyInputState;
use log::{error, info};
use webdriver::actions::{
    ActionSequence, ActionsType, GeneralAction, KeyAction, KeyActionItem, KeyDownAction,
    KeyUpAction, NullActionItem, PointerAction, PointerActionItem, PointerDownAction,
    PointerMoveAction, PointerOrigin, PointerType, PointerUpAction, WheelAction, WheelActionItem,
    WheelScrollAction,
};
use webdriver::command::ActionsParameters;
use webdriver::error::{ErrorStatus, WebDriverError};

use crate::{Handler, WebElement, wait_for_script_response};

// Interval between wheelScroll and pointerMove increments in ms, based on common vsync
static POINTERMOVE_INTERVAL: u64 = 17;
static WHEELSCROLL_INTERVAL: u64 = 17;

// A single action, corresponding to an `action object` in the spec.
// In the spec, `action item` refers to a plain JSON object.
// However, we use the name ActionItem here
// to be consistent with type names from webdriver crate.
#[derive(Debug)]
pub(crate) enum ActionItem {
    Null(NullActionItem),
    Key(KeyActionItem),
    Pointer(PointerActionItem),
    Wheel(WheelActionItem),
}

// A set of actions with multiple sources executed within a single tick.
// The order in which they are performed is not guaranteed.
// The `id` is used to identify the source of the actions.
pub(crate) type TickActions = HashMap<String, ActionItem>;

// Consumed by the `dispatch_actions` method.
pub(crate) type ActionsByTick = Vec<TickActions>;

/// <https://w3c.github.io/webdriver/#dfn-input-source-state>
pub(crate) enum InputSourceState {
    Null,
    #[allow(dead_code)]
    Key(KeyInputState),
    Pointer(PointerInputState),
    #[allow(dead_code)]
    Wheel,
}

// https://w3c.github.io/webdriver/#dfn-pointer-input-source
// TODO: subtype is used for https://w3c.github.io/webdriver/#dfn-get-a-pointer-id
// Need to add pointer-id to the following struct
#[allow(dead_code)]
pub(crate) struct PointerInputState {
    subtype: PointerType,
    pressed: HashSet<u64>,
    x: f64,
    y: f64,
}

impl PointerInputState {
    pub fn new(subtype: PointerType) -> PointerInputState {
        PointerInputState {
            subtype,
            pressed: HashSet::new(),
            x: 0.0,
            y: 0.0,
        }
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
        &self,
        actions_by_tick: ActionsByTick,
        browsing_context: BrowsingContextId,
    ) -> Result<(), ErrorStatus> {
        // Step 1. Wait for an action queue token with input state.
        let new_token = self.id_generator.next();
        assert!(self.current_action_id.get().is_none());
        self.current_action_id.set(Some(new_token));

        // Step 2. Let actions result be the result of dispatch actions inner.
        let res = self.dispatch_actions_inner(actions_by_tick, browsing_context);

        // Step 3. Dequeue input state's actions queue.
        self.current_action_id.set(None);

        // Step 4. Return actions result.
        res
    }

    /// <https://w3c.github.io/webdriver/#dfn-dispatch-actions-inner>
    fn dispatch_actions_inner(
        &self,
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
            let now = Instant::now();

            // Step 1.3. Try to dispatch tick actions
            self.dispatch_tick_actions(tick_actions, tick_duration)?;

            // Step 1.4. Wait for
            // The user agent event loop has spun enough times to process the DOM events
            // generated by the last invocation of the dispatch tick actions steps.
            self.wait_for_user_agent_handling_complete(tick_actions)?;
            // At least tick duration milliseconds have passed.
            let elapsed = now.elapsed();
            if elapsed.as_millis() < tick_duration as u128 {
                let sleep_duration = tick_duration - elapsed.as_millis() as u64;
                thread::sleep(Duration::from_millis(sleep_duration));
            }
        }

        // Step 2. Return success with data null.
        info!("Dispatch actions completed successfully");
        Ok(())
    }

    fn wait_for_user_agent_handling_complete(
        &self,
        tick_actions: &TickActions,
    ) -> Result<(), ErrorStatus> {
        // TODO: Add matches! for key actions
        // after implmenting webdriver id for key events.
        let count_non_null_actions_in_tick = tick_actions
            .iter()
            .filter(|(_, action)| {
                matches!(
                    action,
                    ActionItem::Pointer(PointerActionItem::Pointer(_)) |
                        ActionItem::Wheel(WheelActionItem::Wheel(_))
                )
            })
            .count();

        // To ensure we wait for all events to be processed, only the last event
        // in each tick action step holds the message id.
        // Whenever a new event is generated, the message id is passed to it.
        //
        // Wait for count_non_null_actions_in_tick number of responses
        for _ in 0..count_non_null_actions_in_tick {
            match self.constellation_receiver.recv() {
                Ok(response) => {
                    let current_waiting_id = self
                        .current_action_id
                        .get()
                        .expect("Current id should be set before dispatch_actions_inner is called");

                    if current_waiting_id != response.id {
                        error!("Dispatch actions completed with wrong id in response");
                        return Err(ErrorStatus::UnknownError);
                    }
                },
                Err(error) => {
                    error!("Dispatch actions completed with IPC error: {error}");
                    return Err(ErrorStatus::UnknownError);
                },
            };
        }

        Ok(())
    }

    /// <https://w3c.github.io/webdriver/#dfn-dispatch-tick-actions>
    fn dispatch_tick_actions(
        &self,
        tick_actions: &TickActions,
        tick_duration: u64,
    ) -> Result<(), ErrorStatus> {
        // Step 1. For each action object in tick actions:
        // Step 1.1. Let input_id be the value of the id property of action object.
        for (input_id, action) in tick_actions.iter() {
            // Step 6. Let subtype be action object's subtype.
            // Steps 7, 8. Try to run specific algorithm based on the action type.
            match action {
                ActionItem::Null(NullActionItem::General(_)) => {
                    self.dispatch_general_action(input_id);
                },
                ActionItem::Key(KeyActionItem::General(_)) => {
                    self.dispatch_general_action(input_id);
                },
                ActionItem::Key(KeyActionItem::Key(KeyAction::Down(keydown_action))) => {
                    self.dispatch_keydown_action(input_id, keydown_action);

                    // Step 9. If subtype is "keyDown", append a copy of action
                    // object with the subtype property changed to "keyUp" to
                    // input state's input cancel list.
                    self.session()
                        .unwrap()
                        .input_cancel_list
                        .borrow_mut()
                        .push(ActionItem::Key(KeyActionItem::Key(KeyAction::Up(
                            KeyUpAction {
                                value: keydown_action.value.clone(),
                            },
                        ))));
                },
                ActionItem::Key(KeyActionItem::Key(KeyAction::Up(keyup_action))) => {
                    self.dispatch_keyup_action(input_id, keyup_action);
                },
                ActionItem::Pointer(PointerActionItem::General(_)) => {
                    self.dispatch_general_action(input_id);
                },
                ActionItem::Pointer(PointerActionItem::Pointer(PointerAction::Down(
                    pointer_down_action,
                ))) => {
                    self.dispatch_pointerdown_action(input_id, pointer_down_action);

                    // Step 10. If subtype is "pointerDown", append a copy of action
                    // object with the subtype property changed to "pointerUp" to
                    // input state's input cancel list.
                    self.session().unwrap().input_cancel_list.borrow_mut().push(
                        ActionItem::Pointer(PointerActionItem::Pointer(PointerAction::Up(
                            PointerUpAction {
                                button: pointer_down_action.button,
                                ..Default::default()
                            },
                        ))),
                    );
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
                    self.dispatch_general_action(input_id);
                },
                ActionItem::Wheel(WheelActionItem::Wheel(WheelAction::Scroll(scroll_action))) => {
                    self.dispatch_scroll_action(scroll_action, tick_duration)?;
                },
                _ => {},
            }
        }

        Ok(())
    }

    /// <https://w3c.github.io/webdriver/#dfn-dispatch-a-pause-action>
    fn dispatch_general_action(&self, source_id: &str) {
        self.session()
            .unwrap()
            .input_state_table
            .borrow_mut()
            .entry(source_id.to_string())
            .or_insert(InputSourceState::Null);
    }

    /// <https://w3c.github.io/webdriver/#dfn-dispatch-a-keydown-action>
    fn dispatch_keydown_action(&self, source_id: &str, action: &KeyDownAction) {
        let session = self.session().unwrap();

        let raw_key = action.value.chars().next().unwrap();
        let mut input_state_table = session.input_state_table.borrow_mut();
        let key_input_state = match input_state_table.get_mut(source_id).unwrap() {
            InputSourceState::Key(key_input_state) => key_input_state,
            _ => unreachable!(),
        };

        let keyboard_event = key_input_state.dispatch_keydown(raw_key);

        // Step 12
        let cmd_msg = WebDriverCommandMsg::KeyboardAction(
            self.session().unwrap().browsing_context_id,
            keyboard_event,
        );
        self.constellation_chan
            .send(EmbedderToConstellationMessage::WebDriverCommand(cmd_msg))
            .unwrap();
    }

    /// <https://w3c.github.io/webdriver/#dfn-dispatch-a-keyup-action>
    fn dispatch_keyup_action(&self, source_id: &str, action: &KeyUpAction) {
        let session = self.session().unwrap();

        let raw_key = action.value.chars().next().unwrap();
        let mut input_state_table = session.input_state_table.borrow_mut();
        let key_input_state = match input_state_table.get_mut(source_id).unwrap() {
            InputSourceState::Key(key_input_state) => key_input_state,
            _ => unreachable!(),
        };

        if let Some(keyboard_event) = key_input_state.dispatch_keyup(raw_key) {
            // Step 12
            let cmd_msg = WebDriverCommandMsg::KeyboardAction(
                self.session().unwrap().browsing_context_id,
                keyboard_event,
            );
            self.constellation_chan
                .send(EmbedderToConstellationMessage::WebDriverCommand(cmd_msg))
                .unwrap();
        }
    }

    /// <https://w3c.github.io/webdriver/#dfn-dispatch-a-pointerdown-action>
    pub(crate) fn dispatch_pointerdown_action(&self, source_id: &str, action: &PointerDownAction) {
        let session = self.session().unwrap();

        let mut input_state_table = session.input_state_table.borrow_mut();
        let pointer_input_state = match input_state_table.get_mut(source_id).unwrap() {
            InputSourceState::Pointer(pointer_input_state) => pointer_input_state,
            _ => unreachable!(),
        };

        if pointer_input_state.pressed.contains(&action.button) {
            return;
        }
        pointer_input_state.pressed.insert(action.button);

        let msg_id = self.current_action_id.get();
        let cmd_msg = WebDriverCommandMsg::MouseButtonAction(
            session.webview_id,
            MouseButtonAction::Down,
            action.button.into(),
            pointer_input_state.x as f32,
            pointer_input_state.y as f32,
            msg_id,
            self.constellation_sender.clone(),
        );
        self.constellation_chan
            .send(EmbedderToConstellationMessage::WebDriverCommand(cmd_msg))
            .unwrap();
    }

    /// <https://w3c.github.io/webdriver/#dfn-dispatch-a-pointerup-action>
    pub(crate) fn dispatch_pointerup_action(&self, source_id: &str, action: &PointerUpAction) {
        let session = self.session().unwrap();

        let mut input_state_table = session.input_state_table.borrow_mut();
        let pointer_input_state = match input_state_table.get_mut(source_id).unwrap() {
            InputSourceState::Pointer(pointer_input_state) => pointer_input_state,
            _ => unreachable!(),
        };

        if !pointer_input_state.pressed.contains(&action.button) {
            return;
        }
        pointer_input_state.pressed.remove(&action.button);

        let msg_id = self.current_action_id.get();
        let cmd_msg = WebDriverCommandMsg::MouseButtonAction(
            session.webview_id,
            MouseButtonAction::Up,
            action.button.into(),
            pointer_input_state.x as f32,
            pointer_input_state.y as f32,
            msg_id,
            self.constellation_sender.clone(),
        );
        self.constellation_chan
            .send(EmbedderToConstellationMessage::WebDriverCommand(cmd_msg))
            .unwrap();
    }

    /// <https://w3c.github.io/webdriver/#dfn-dispatch-a-pointermove-action>
    pub(crate) fn dispatch_pointermove_action(
        &self,
        source_id: &str,
        action: &PointerMoveAction,
        tick_duration: u64,
    ) -> Result<(), ErrorStatus> {
        let tick_start = Instant::now();

        // Steps 1 - 2
        let x_offset = action.x;
        let y_offset = action.y;

        // Steps 3 - 4
        let (start_x, start_y) = match self
            .session()
            .unwrap()
            .input_state_table
            .borrow_mut()
            .get(source_id)
            .unwrap()
        {
            InputSourceState::Pointer(pointer_input_state) => {
                (pointer_input_state.x, pointer_input_state.y)
            },
            _ => unreachable!(),
        };

        let (x, y) = match action.origin {
            PointerOrigin::Viewport => (x_offset, y_offset),
            PointerOrigin::Pointer => (start_x + x_offset, start_y + y_offset),
            PointerOrigin::Element(ref web_element) => {
                let point = self.get_element_origin_relative_coordinates(web_element)?;
                (point.0 as f64, point.1 as f64)
            },
        };

        // Step 5 - 6
        self.check_viewport_bound(x, y)?;

        // Step 7
        let duration = match action.duration {
            Some(duration) => duration,
            None => tick_duration,
        };

        // Step 8
        if duration > 0 {
            thread::sleep(Duration::from_millis(POINTERMOVE_INTERVAL));
        }

        // Step 9 - 18
        self.perform_pointer_move(source_id, duration, start_x, start_y, x, y, tick_start);

        // Step 19
        Ok(())
    }

    /// <https://w3c.github.io/webdriver/#dfn-perform-a-pointer-move>
    #[allow(clippy::too_many_arguments)]
    fn perform_pointer_move(
        &self,
        source_id: &str,
        duration: u64,
        start_x: f64,
        start_y: f64,
        target_x: f64,
        target_y: f64,
        tick_start: Instant,
    ) {
        let session = self.session().unwrap();
        let mut input_state_table = session.input_state_table.borrow_mut();
        let pointer_input_state = match input_state_table.get_mut(source_id).unwrap() {
            InputSourceState::Pointer(pointer_input_state) => pointer_input_state,
            _ => unreachable!(),
        };

        loop {
            // Step 1
            let time_delta = tick_start.elapsed().as_millis();

            // Step 2
            let duration_ratio = if duration > 0 {
                time_delta as f64 / duration as f64
            } else {
                1.0
            };

            // Step 3
            let last = 1.0 - duration_ratio < 0.001;

            // Step 4
            let (x, y) = if last {
                (target_x, target_y)
            } else {
                (
                    duration_ratio * (target_x - start_x) + start_x,
                    duration_ratio * (target_y - start_y) + start_y,
                )
            };

            // Steps 5 - 6
            let current_x = pointer_input_state.x;
            let current_y = pointer_input_state.y;

            // Step 7
            // Actually "last" should not be checked here based on spec.
            // However, we need to send the webdriver id at the final perform.
            if x != current_x || y != current_y || last {
                // Step 7.2
                let msg_id = if last {
                    self.current_action_id.get()
                } else {
                    None
                };
                let cmd_msg = WebDriverCommandMsg::MouseMoveAction(
                    session.webview_id,
                    x as f32,
                    y as f32,
                    msg_id,
                    self.constellation_sender.clone(),
                );
                self.constellation_chan
                    .send(EmbedderToConstellationMessage::WebDriverCommand(cmd_msg))
                    .unwrap();
                // Step 7.3
                pointer_input_state.x = x;
                pointer_input_state.y = y;
            }

            // Step 8
            if last {
                return;
            }

            // Step 9
            thread::sleep(Duration::from_millis(POINTERMOVE_INTERVAL));
        }
    }

    /// <https://w3c.github.io/webdriver/#dfn-dispatch-a-scroll-action>
    fn dispatch_scroll_action(
        &self,
        action: &WheelScrollAction,
        tick_duration: u64,
    ) -> Result<(), ErrorStatus> {
        // Note: We have not implemented `extract an action sequence` which will calls
        // `process a wheel action` that validate many of the variable used here.
        // Hence, we do all the checking here until those functions is properly
        // implemented.
        // <https://w3c.github.io/webdriver/#dfn-process-a-wheel-action>

        let tick_start = Instant::now();

        // Step 1
        let Some(x_offset) = action.x else {
            return Err(ErrorStatus::InvalidArgument);
        };

        // Step 2
        let Some(y_offset) = action.y else {
            return Err(ErrorStatus::InvalidArgument);
        };

        // Step 3 - 4
        // Get coordinates relative to an origin.
        let (x, y) = match action.origin {
            PointerOrigin::Viewport => (x_offset, y_offset),
            PointerOrigin::Pointer => return Err(ErrorStatus::InvalidArgument),
            PointerOrigin::Element(ref web_element) => {
                self.get_element_origin_relative_coordinates(web_element)?
            },
        };

        // Step 5 - 6
        self.check_viewport_bound(x as _, y as _)?;

        // Step 7 - 8
        let Some(delta_x) = action.deltaX else {
            return Err(ErrorStatus::InvalidArgument);
        };

        let Some(delta_y) = action.deltaY else {
            return Err(ErrorStatus::InvalidArgument);
        };

        // Step 9
        let duration = match action.duration {
            Some(duration) => duration,
            None => tick_duration,
        };

        // Step 10
        if duration > 0 {
            thread::sleep(Duration::from_millis(WHEELSCROLL_INTERVAL));
        }

        // Step 11
        self.perform_scroll(duration, x, y, delta_x, delta_y, 0, 0, tick_start);

        // Step 12
        Ok(())
    }

    /// <https://w3c.github.io/webdriver/#dfn-perform-a-scroll>
    #[allow(clippy::too_many_arguments)]
    fn perform_scroll(
        &self,
        duration: u64,
        x: i64,
        y: i64,
        target_delta_x: i64,
        target_delta_y: i64,
        mut curr_delta_x: i64,
        mut curr_delta_y: i64,
        tick_start: Instant,
    ) {
        let session = self.session().unwrap();

        // Step 1
        let time_delta = tick_start.elapsed().as_millis();

        // Step 2
        let duration_ratio = if duration > 0 {
            time_delta as f64 / duration as f64
        } else {
            1.0
        };

        // Step 3
        let last = 1.0 - duration_ratio < 0.001;

        // Step 4
        let (delta_x, delta_y) = if last {
            (target_delta_x - curr_delta_x, target_delta_y - curr_delta_y)
        } else {
            (
                (duration_ratio * target_delta_x as f64) as i64 - curr_delta_x,
                (duration_ratio * target_delta_y as f64) as i64 - curr_delta_y,
            )
        };

        // Step 5
        // Actually "last" should not be checked here based on spec.
        // However, we need to send the webdriver id at the final perform.
        if delta_x != 0 || delta_y != 0 || last {
            // Perform implementation-specific action dispatch steps
            let msg_id = if last {
                self.current_action_id.get()
            } else {
                None
            };
            let cmd_msg = WebDriverCommandMsg::WheelScrollAction(
                session.webview_id,
                x as f32,
                y as f32,
                delta_x as f64,
                delta_y as f64,
                msg_id,
                self.constellation_sender.clone(),
            );
            self.constellation_chan
                .send(EmbedderToConstellationMessage::WebDriverCommand(cmd_msg))
                .unwrap();

            curr_delta_x += delta_x;
            curr_delta_y += delta_y;
        }

        // Step 6
        if last {
            return;
        }

        // Step 7
        // TODO: The two steps should be done in parallel
        // 7.1. Asynchronously wait for an implementation defined amount of time to pass.
        thread::sleep(Duration::from_millis(WHEELSCROLL_INTERVAL));
        // 7.2. Perform a scroll with arguments duration, x, y, target delta x,
        // target delta y, current delta x, current delta y.
        self.perform_scroll(
            duration,
            x,
            y,
            target_delta_x,
            target_delta_y,
            curr_delta_x,
            curr_delta_y,
            tick_start,
        );
    }

    fn check_viewport_bound(&self, x: f64, y: f64) -> Result<(), ErrorStatus> {
        let (sender, receiver) = ipc::channel().unwrap();
        let cmd_msg =
            WebDriverCommandMsg::GetWindowSize(self.session.as_ref().unwrap().webview_id, sender);
        self.constellation_chan
            .send(EmbedderToConstellationMessage::WebDriverCommand(cmd_msg))
            .unwrap();

        let viewport_size = match wait_for_script_response(receiver) {
            Ok(response) => response,
            Err(WebDriverError { error, .. }) => return Err(error),
        };
        if x < 0.0 || x > viewport_size.width.into() || y < 0.0 || y > viewport_size.height.into() {
            Err(ErrorStatus::MoveTargetOutOfBounds)
        } else {
            Ok(())
        }
    }

    fn get_element_origin_relative_coordinates(
        &self,
        web_element: &WebElement,
    ) -> Result<(i64, i64), ErrorStatus> {
        let (sender, receiver) = ipc::channel().unwrap();
        self.browsing_context_script_command(WebDriverScriptCommand::GetElementInViewCenterPoint(
            web_element.to_string(),
            sender,
        ))
        .unwrap();
        let response = match wait_for_script_response(receiver) {
            Ok(response) => response,
            Err(WebDriverError { error, .. }) => return Err(error),
        };
        match response? {
            Some(point) => Ok(point),
            None => Err(ErrorStatus::UnknownError),
        }
    }

    /// <https://w3c.github.io/webdriver/#dfn-extract-an-action-sequence>
    pub(crate) fn extract_an_action_sequence(&self, params: ActionsParameters) -> ActionsByTick {
        // Step 1. Let actions be the result of getting a property named "actions" from parameters.
        // Step 2 (ignored because params is already validated earlier). If actions is not a list,
        // return an error with status InvalidArgument.
        let actions = params.actions;

        self.actions_by_tick_from_sequence(actions)
    }

    pub(crate) fn actions_by_tick_from_sequence(
        &self,
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
                actions_by_tick.push(HashMap::new());
            }

            // Step 4.2.3.
            for (tick_index, action_item) in source_actions.into_iter().enumerate() {
                actions_by_tick[tick_index].insert(id.clone(), action_item);
            }
        }

        actions_by_tick
    }

    /// <https://w3c.github.io/webdriver/#dfn-process-an-input-source-action-sequence>
    pub(crate) fn process_an_input_source_action_sequence(
        &self,
        action_sequence: ActionSequence,
    ) -> Vec<ActionItem> {
        // Step 2. Let id be the value of the id property of action sequence.
        let id = action_sequence.id.clone();

        let mut input_state_table = self.session().unwrap().input_state_table.borrow_mut();

        match action_sequence.actions {
            ActionsType::Null {
                actions: null_actions,
            } => {
                input_state_table
                    .entry(id)
                    .or_insert(InputSourceState::Null);
                null_actions.into_iter().map(ActionItem::Null).collect()
            },
            ActionsType::Key {
                actions: key_actions,
            } => {
                input_state_table
                    .entry(id)
                    .or_insert(InputSourceState::Key(KeyInputState::new()));
                key_actions.into_iter().map(ActionItem::Key).collect()
            },
            ActionsType::Pointer {
                parameters: _,
                actions: pointer_actions,
            } => {
                input_state_table
                    .entry(id)
                    .or_insert(InputSourceState::Pointer(PointerInputState::new(
                        PointerType::Mouse,
                    )));
                pointer_actions
                    .into_iter()
                    .map(ActionItem::Pointer)
                    .collect()
            },
            ActionsType::Wheel {
                actions: wheel_actions,
            } => {
                input_state_table
                    .entry(id)
                    .or_insert(InputSourceState::Wheel);
                wheel_actions.into_iter().map(ActionItem::Wheel).collect()
            },
        }
    }
}
