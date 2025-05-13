/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashSet;
use std::time::{Duration, Instant};
use std::{cmp, thread};

use constellation_traits::EmbedderToConstellationMessage;
use embedder_traits::{MouseButtonAction, WebDriverCommandMsg, WebDriverScriptCommand};
use ipc_channel::ipc;
use keyboard_types::webdriver::KeyInputState;
use webdriver::actions::{
    ActionSequence, ActionsType, GeneralAction, KeyAction, KeyActionItem, KeyDownAction,
    KeyUpAction, NullActionItem, PointerAction, PointerActionItem, PointerActionParameters,
    PointerDownAction, PointerMoveAction, PointerOrigin, PointerType, PointerUpAction, WheelAction,
    WheelActionItem, WheelScrollAction,
};
use webdriver::error::{ErrorStatus, WebDriverError};

use crate::{Handler, WebElement, wait_for_script_response};

// Interval between wheelScroll and pointerMove increments in ms, based on common vsync
static POINTERMOVE_INTERVAL: u64 = 17;
static WHEELSCROLL_INTERVAL: u64 = 17;

// https://w3c.github.io/webdriver/#dfn-input-source-state
pub(crate) enum InputSourceState {
    Null,
    Key(KeyInputState),
    Pointer(PointerInputState),
    Wheel,
}

// https://w3c.github.io/webdriver/#dfn-pointer-input-source
pub(crate) struct PointerInputState {
    subtype: PointerType,
    pressed: HashSet<u64>,
    x: i64,
    y: i64,
}

impl PointerInputState {
    pub fn new(subtype: &PointerType) -> PointerInputState {
        PointerInputState {
            subtype: match subtype {
                PointerType::Mouse => PointerType::Mouse,
                PointerType::Pen => PointerType::Pen,
                PointerType::Touch => PointerType::Touch,
            },
            pressed: HashSet::new(),
            x: 0,
            y: 0,
        }
    }
}

// https://w3c.github.io/webdriver/#dfn-computing-the-tick-duration
fn compute_tick_duration(tick_actions: &ActionSequence) -> u64 {
    let mut duration = 0;
    match &tick_actions.actions {
        ActionsType::Null { actions } => {
            for action in actions.iter() {
                let NullActionItem::General(GeneralAction::Pause(pause_action)) = action;
                duration = cmp::max(duration, pause_action.duration.unwrap_or(0));
            }
        },
        ActionsType::Pointer {
            parameters: _,
            actions,
        } => {
            for action in actions.iter() {
                let action_duration = match action {
                    PointerActionItem::General(GeneralAction::Pause(action)) => action.duration,
                    PointerActionItem::Pointer(PointerAction::Move(action)) => action.duration,
                    _ => None,
                };
                duration = cmp::max(duration, action_duration.unwrap_or(0));
            }
        },
        ActionsType::Key { actions: _ } => (),
        ActionsType::Wheel { actions } => {
            for action in actions.iter() {
                let action_duration = match action {
                    WheelActionItem::General(GeneralAction::Pause(action)) => action.duration,
                    WheelActionItem::Wheel(WheelAction::Scroll(action)) => action.duration,
                };
                duration = cmp::max(duration, action_duration.unwrap_or(0));
            }
        },
    }
    duration
}

impl Handler {
    // https://w3c.github.io/webdriver/#dfn-dispatch-actions
    pub(crate) fn dispatch_actions(
        &mut self,
        actions_by_tick: &[ActionSequence],
    ) -> Result<(), ErrorStatus> {
        for tick_actions in actions_by_tick.iter() {
            let tick_duration = compute_tick_duration(tick_actions);
            self.dispatch_tick_actions(tick_actions, tick_duration)?;
        }
        Ok(())
    }

    fn dispatch_general_action(&mut self, source_id: &str) {
        self.session_mut()
            .unwrap()
            .input_state_table
            .entry(source_id.to_string())
            .or_insert(InputSourceState::Null);
        // https://w3c.github.io/webdriver/#dfn-dispatch-a-pause-action
        // Nothing to be done
    }

    // https://w3c.github.io/webdriver/#dfn-dispatch-tick-actions
    fn dispatch_tick_actions(
        &mut self,
        tick_actions: &ActionSequence,
        tick_duration: u64,
    ) -> Result<(), ErrorStatus> {
        let source_id = &tick_actions.id;
        match &tick_actions.actions {
            ActionsType::Null { actions } => {
                for _action in actions.iter() {
                    self.dispatch_general_action(source_id);
                }
            },
            ActionsType::Key { actions } => {
                for action in actions.iter() {
                    match action {
                        KeyActionItem::General(_action) => {
                            self.dispatch_general_action(source_id);
                        },
                        KeyActionItem::Key(action) => {
                            self.session_mut()
                                .unwrap()
                                .input_state_table
                                .entry(source_id.to_string())
                                .or_insert(InputSourceState::Key(KeyInputState::new()));
                            match action {
                                KeyAction::Down(action) => {
                                    self.dispatch_keydown_action(source_id, action)
                                },
                                KeyAction::Up(action) => {
                                    self.dispatch_keyup_action(source_id, action)
                                },
                            };
                        },
                    }
                }
            },
            ActionsType::Pointer {
                parameters,
                actions,
            } => {
                for action in actions.iter() {
                    match action {
                        PointerActionItem::General(_action) => {
                            self.dispatch_general_action(source_id);
                        },
                        PointerActionItem::Pointer(action) => {
                            self.session_mut()
                                .unwrap()
                                .input_state_table
                                .entry(source_id.to_string())
                                .or_insert(InputSourceState::Pointer(PointerInputState::new(
                                    &parameters.pointer_type,
                                )));
                            match action {
                                PointerAction::Cancel => (),
                                PointerAction::Down(action) => {
                                    self.dispatch_pointerdown_action(source_id, action)
                                },
                                PointerAction::Move(action) => self.dispatch_pointermove_action(
                                    source_id,
                                    action,
                                    tick_duration,
                                )?,
                                PointerAction::Up(action) => {
                                    self.dispatch_pointerup_action(source_id, action)
                                },
                            }
                        },
                    }
                }
            },
            ActionsType::Wheel { actions } => {
                for action in actions.iter() {
                    match action {
                        WheelActionItem::General(_action) => {
                            self.dispatch_general_action(source_id)
                        },
                        WheelActionItem::Wheel(action) => {
                            self.session_mut()
                                .unwrap()
                                .input_state_table
                                .entry(source_id.to_string())
                                .or_insert(InputSourceState::Wheel);
                            match action {
                                WheelAction::Scroll(action) => {
                                    self.dispatch_scroll_action(action, tick_duration)?
                                },
                            }
                        },
                    }
                }
            },
        }

        Ok(())
    }

    // https://w3c.github.io/webdriver/#dfn-dispatch-a-keydown-action
    fn dispatch_keydown_action(&mut self, source_id: &str, action: &KeyDownAction) {
        let session = self.session.as_mut().unwrap();

        let raw_key = action.value.chars().next().unwrap();
        let key_input_state = match session.input_state_table.get_mut(source_id).unwrap() {
            InputSourceState::Key(key_input_state) => key_input_state,
            _ => unreachable!(),
        };

        session.input_cancel_list.push(ActionSequence {
            id: source_id.into(),
            actions: ActionsType::Key {
                actions: vec![KeyActionItem::Key(KeyAction::Up(KeyUpAction {
                    value: action.value.clone(),
                }))],
            },
        });

        let keyboard_event = key_input_state.dispatch_keydown(raw_key);
        let cmd_msg =
            WebDriverCommandMsg::KeyboardAction(session.browsing_context_id, keyboard_event);
        self.constellation_chan
            .send(EmbedderToConstellationMessage::WebDriverCommand(cmd_msg))
            .unwrap();
    }

    // https://w3c.github.io/webdriver/#dfn-dispatch-a-keyup-action
    fn dispatch_keyup_action(&mut self, source_id: &str, action: &KeyUpAction) {
        let session = self.session.as_mut().unwrap();

        let raw_key = action.value.chars().next().unwrap();
        let key_input_state = match session.input_state_table.get_mut(source_id).unwrap() {
            InputSourceState::Key(key_input_state) => key_input_state,
            _ => unreachable!(),
        };

        session.input_cancel_list.push(ActionSequence {
            id: source_id.into(),
            actions: ActionsType::Key {
                actions: vec![KeyActionItem::Key(KeyAction::Up(KeyUpAction {
                    value: action.value.clone(),
                }))],
            },
        });

        if let Some(keyboard_event) = key_input_state.dispatch_keyup(raw_key) {
            let cmd_msg =
                WebDriverCommandMsg::KeyboardAction(session.browsing_context_id, keyboard_event);
            self.constellation_chan
                .send(EmbedderToConstellationMessage::WebDriverCommand(cmd_msg))
                .unwrap();
        }
    }

    // https://w3c.github.io/webdriver/#dfn-dispatch-a-pointerdown-action
    pub(crate) fn dispatch_pointerdown_action(
        &mut self,
        source_id: &str,
        action: &PointerDownAction,
    ) {
        let session = self.session.as_mut().unwrap();

        let pointer_input_state = match session.input_state_table.get_mut(source_id).unwrap() {
            InputSourceState::Pointer(pointer_input_state) => pointer_input_state,
            _ => unreachable!(),
        };

        if pointer_input_state.pressed.contains(&action.button) {
            return;
        }
        pointer_input_state.pressed.insert(action.button);

        session.input_cancel_list.push(ActionSequence {
            id: source_id.into(),
            actions: ActionsType::Pointer {
                parameters: PointerActionParameters {
                    pointer_type: match pointer_input_state.subtype {
                        PointerType::Mouse => PointerType::Mouse,
                        PointerType::Pen => PointerType::Pen,
                        PointerType::Touch => PointerType::Touch,
                    },
                },
                actions: vec![PointerActionItem::Pointer(PointerAction::Up(
                    PointerUpAction {
                        button: action.button,
                        ..Default::default()
                    },
                ))],
            },
        });

        let cmd_msg = WebDriverCommandMsg::MouseButtonAction(
            session.webview_id,
            MouseButtonAction::Down,
            action.button.into(),
            pointer_input_state.x as f32,
            pointer_input_state.y as f32,
        );
        self.constellation_chan
            .send(EmbedderToConstellationMessage::WebDriverCommand(cmd_msg))
            .unwrap();
    }

    // https://w3c.github.io/webdriver/#dfn-dispatch-a-pointerup-action
    pub(crate) fn dispatch_pointerup_action(&mut self, source_id: &str, action: &PointerUpAction) {
        let session = self.session.as_mut().unwrap();

        let pointer_input_state = match session.input_state_table.get_mut(source_id).unwrap() {
            InputSourceState::Pointer(pointer_input_state) => pointer_input_state,
            _ => unreachable!(),
        };

        if !pointer_input_state.pressed.contains(&action.button) {
            return;
        }
        pointer_input_state.pressed.remove(&action.button);

        session.input_cancel_list.push(ActionSequence {
            id: source_id.into(),
            actions: ActionsType::Pointer {
                parameters: PointerActionParameters {
                    pointer_type: match pointer_input_state.subtype {
                        PointerType::Mouse => PointerType::Mouse,
                        PointerType::Pen => PointerType::Pen,
                        PointerType::Touch => PointerType::Touch,
                    },
                },
                actions: vec![PointerActionItem::Pointer(PointerAction::Down(
                    PointerDownAction {
                        button: action.button,
                        ..Default::default()
                    },
                ))],
            },
        });

        let cmd_msg = WebDriverCommandMsg::MouseButtonAction(
            session.webview_id,
            MouseButtonAction::Up,
            action.button.into(),
            pointer_input_state.x as f32,
            pointer_input_state.y as f32,
        );
        self.constellation_chan
            .send(EmbedderToConstellationMessage::WebDriverCommand(cmd_msg))
            .unwrap();
    }

    // https://w3c.github.io/webdriver/#dfn-dispatch-a-pointermove-action
    pub(crate) fn dispatch_pointermove_action(
        &mut self,
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
            .session
            .as_ref()
            .unwrap()
            .input_state_table
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
                self.get_element_origin_relative_coordinates(web_element)?
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
        &mut self,
        source_id: &str,
        duration: u64,
        start_x: i64,
        start_y: i64,
        target_x: i64,
        target_y: i64,
        tick_start: Instant,
    ) {
        let session = self.session.as_mut().unwrap();
        let pointer_input_state = match session.input_state_table.get_mut(source_id).unwrap() {
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
                    (duration_ratio * (target_x - start_x) as f64) as i64 + start_x,
                    (duration_ratio * (target_y - start_y) as f64) as i64 + start_y,
                )
            };

            // Steps 5 - 6
            let current_x = pointer_input_state.x;
            let current_y = pointer_input_state.y;

            // Step 7
            if x != current_x || y != current_y {
                // Step 7.2
                let cmd_msg =
                    WebDriverCommandMsg::MouseMoveAction(session.webview_id, x as f32, y as f32);
                //TODO: Need Synchronization here before updating `pointer_input_state`
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
        &mut self,
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
        self.check_viewport_bound(x, y)?;

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
        &mut self,
        duration: u64,
        x: i64,
        y: i64,
        target_delta_x: i64,
        target_delta_y: i64,
        mut curr_delta_x: i64,
        mut curr_delta_y: i64,
        tick_start: Instant,
    ) {
        let session = self.session.as_mut().unwrap();

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
        if delta_x != 0 || delta_y != 0 {
            // Perform implementation-specific action dispatch steps
            let cmd_msg = WebDriverCommandMsg::WheelScrollAction(
                session.webview_id,
                x as f32,
                y as f32,
                delta_x as f64,
                delta_y as f64,
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

    fn check_viewport_bound(&self, x: i64, y: i64) -> Result<(), ErrorStatus> {
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
        if x < 0 || x as f32 > viewport_size.width || y < 0 || y as f32 > viewport_size.height {
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
}
