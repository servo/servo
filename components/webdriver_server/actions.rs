/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::Handler;
use keyboard_types::webdriver::KeyInputState;
use script_traits::{ConstellationMsg, MouseButton, MouseEventType, WebDriverCommandMsg};
use std::cmp;
use std::collections::HashSet;
use webdriver::actions::{ActionSequence, ActionsType, GeneralAction, NullActionItem};
use webdriver::actions::{KeyAction, KeyActionItem, KeyDownAction, KeyUpAction};
use webdriver::actions::{
    PointerAction, PointerActionItem, PointerActionParameters, PointerDownAction,
};
use webdriver::actions::{PointerType, PointerUpAction};

// https://w3c.github.io/webdriver/#dfn-input-source-state
pub(crate) enum InputSourceState {
    Null,
    Key(KeyInputState),
    Pointer(PointerInputState),
}

// https://w3c.github.io/webdriver/#dfn-pointer-input-source
pub(crate) struct PointerInputState {
    subtype: PointerType,
    pressed: HashSet<u64>,
    x: u64,
    y: u64,
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
    }
    duration
}

fn u64_to_mouse_button(button: u64) -> Option<MouseButton> {
    if MouseButton::Left as u64 == button {
        Some(MouseButton::Left)
    } else if MouseButton::Middle as u64 == button {
        Some(MouseButton::Middle)
    } else if MouseButton::Right as u64 == button {
        Some(MouseButton::Right)
    } else {
        None
    }
}

impl Handler {
    // https://w3c.github.io/webdriver/#dfn-dispatch-actions
    pub(crate) fn dispatch_actions(&mut self, actions_by_tick: &[ActionSequence]) {
        for tick_actions in actions_by_tick.iter() {
            let tick_duration = compute_tick_duration(&tick_actions);
            self.dispatch_tick_actions(&tick_actions, tick_duration);
        }
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
    fn dispatch_tick_actions(&mut self, tick_actions: &ActionSequence, _tick_duration: u64) {
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
                                    self.dispatch_keydown_action(&source_id, &action)
                                },
                                KeyAction::Up(action) => {
                                    self.dispatch_keyup_action(&source_id, &action)
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
                                    self.dispatch_pointerdown_action(&source_id, &action)
                                },
                                PointerAction::Move(_action) => (),
                                PointerAction::Up(action) => {
                                    self.dispatch_pointerup_action(&source_id, &action)
                                },
                            }
                        },
                    }
                }
            },
        }
    }

    // https://w3c.github.io/webdriver/#dfn-dispatch-a-keydown-action
    fn dispatch_keydown_action(&mut self, source_id: &str, action: &KeyDownAction) {
        let session = self.session.as_mut().unwrap();

        let raw_key = action.value.chars().next().unwrap();
        let key_input_state = match session.input_state_table.get_mut(source_id).unwrap() {
            InputSourceState::Null => unreachable!(),
            InputSourceState::Key(key_input_state) => key_input_state,
            InputSourceState::Pointer(_) => unreachable!(),
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
            .send(ConstellationMsg::WebDriverCommand(cmd_msg))
            .unwrap();
    }

    // https://w3c.github.io/webdriver/#dfn-dispatch-a-keyup-action
    fn dispatch_keyup_action(&mut self, source_id: &str, action: &KeyUpAction) {
        let session = self.session.as_mut().unwrap();

        let raw_key = action.value.chars().next().unwrap();
        let key_input_state = match session.input_state_table.get_mut(source_id).unwrap() {
            InputSourceState::Null => unreachable!(),
            InputSourceState::Key(key_input_state) => key_input_state,
            InputSourceState::Pointer(_) => unreachable!(),
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
                .send(ConstellationMsg::WebDriverCommand(cmd_msg))
                .unwrap();
        }
    }

    // https://w3c.github.io/webdriver/#dfn-dispatch-a-pointerdown-action
    fn dispatch_pointerdown_action(&mut self, source_id: &str, action: &PointerDownAction) {
        let session = self.session.as_mut().unwrap();

        let pointer_input_state = match session.input_state_table.get_mut(source_id).unwrap() {
            InputSourceState::Null => unreachable!(),
            InputSourceState::Key(_) => unreachable!(),
            InputSourceState::Pointer(pointer_input_state) => pointer_input_state,
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
                    },
                ))],
            },
        });

        if let Some(button) = u64_to_mouse_button(action.button) {
            let cmd_msg = WebDriverCommandMsg::MouseButtonAction(
                MouseEventType::MouseDown,
                button,
                pointer_input_state.x as f32,
                pointer_input_state.y as f32,
            );
            self.constellation_chan
                .send(ConstellationMsg::WebDriverCommand(cmd_msg))
                .unwrap();
        }
    }

    // https://w3c.github.io/webdriver/#dfn-dispatch-a-pointerup-action
    fn dispatch_pointerup_action(&mut self, source_id: &str, action: &PointerUpAction) {
        let session = self.session.as_mut().unwrap();

        let pointer_input_state = match session.input_state_table.get_mut(source_id).unwrap() {
            InputSourceState::Null => unreachable!(),
            InputSourceState::Key(_) => unreachable!(),
            InputSourceState::Pointer(pointer_input_state) => pointer_input_state,
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
                    },
                ))],
            },
        });

        if let Some(button) = u64_to_mouse_button(action.button) {
            let cmd_msg = WebDriverCommandMsg::MouseButtonAction(
                MouseEventType::MouseUp,
                button,
                pointer_input_state.x as f32,
                pointer_input_state.y as f32,
            );
            self.constellation_chan
                .send(ConstellationMsg::WebDriverCommand(cmd_msg))
                .unwrap();
        }
    }
}
