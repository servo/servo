/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::WebDriverSession;
use crossbeam_channel::Sender;
use keyboard_types::webdriver::KeyInputState;
use script_traits::{ConstellationMsg, WebDriverCommandMsg};
use std::cmp;
use std::collections::HashSet;
use webdriver::actions::{ActionSequence, ActionsType, GeneralAction, NullActionItem};
use webdriver::actions::{KeyAction, KeyActionItem, KeyDownAction, KeyUpAction};
use webdriver::actions::{PointerAction, PointerActionItem, PointerType};

// https://w3c.github.io/webdriver/#dfn-input-source-state
pub enum InputSourceState {
    Null,
    Key(KeyInputState),
    Pointer(PointerInputState),
}

// https://w3c.github.io/webdriver/#dfn-pointer-input-source
pub struct PointerInputState {
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

impl WebDriverSession {
    // https://w3c.github.io/webdriver/#dfn-dispatch-actions
    pub fn dispatch_actions(
        &mut self,
        constellation_chan: Sender<ConstellationMsg>,
        actions_by_tick: &Vec<ActionSequence>,
    ) {
        for tick_actions in actions_by_tick.iter() {
            let tick_duration = compute_tick_duration(&tick_actions);
            self.dispatch_tick_actions(&constellation_chan, &tick_actions, tick_duration);
        }
    }

    fn dispatch_general_action(&mut self, source_id: &str) {
        self.input_state_table
            .entry(source_id.to_string())
            .or_insert(InputSourceState::Null);
        // https://w3c.github.io/webdriver/#dfn-dispatch-a-pause-action
        // Nothing to be done
    }

    // https://w3c.github.io/webdriver/#dfn-dispatch-tick-actions
    fn dispatch_tick_actions(
        &mut self,
        constellation_chan: &Sender<ConstellationMsg>,
        tick_actions: &ActionSequence,
        tick_duration: u64,
    ) {
        let source_id = tick_actions.id.as_ref().unwrap();
        match &tick_actions.actions {
            ActionsType::Null { actions } => {
                for action in actions.iter() {
                    self.dispatch_general_action(source_id);
                }
            },
            ActionsType::Key { actions } => {
                for action in actions.iter() {
                    match action {
                        KeyActionItem::General(action) => {
                            self.dispatch_general_action(source_id);
                        },
                        KeyActionItem::Key(action) => {
                            self.input_state_table
                                .entry(source_id.to_string())
                                .or_insert(InputSourceState::Key(KeyInputState::new()));
                            match action {
                                KeyAction::Down(action) => self.dispatch_keydown_action(
                                    constellation_chan,
                                    &source_id,
                                    &action,
                                    tick_duration,
                                ),
                                KeyAction::Up(action) => self.dispatch_keyup_action(
                                    constellation_chan,
                                    &source_id,
                                    &action,
                                    tick_duration,
                                ),
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
                        PointerActionItem::General(action) => {
                            self.dispatch_general_action(source_id);
                        },
                        PointerActionItem::Pointer(action) => {
                            self.input_state_table
                                .entry(source_id.to_string())
                                .or_insert(InputSourceState::Pointer(PointerInputState::new(
                                    &parameters.pointer_type,
                                )));
                            match action {
                                PointerAction::Cancel => (),
                                PointerAction::Down(action) => (),
                                PointerAction::Move(action) => (),
                                PointerAction::Up(action) => (),
                            }
                        },
                    }
                }
            },
        }
    }

    // https://w3c.github.io/webdriver/#dfn-dispatch-a-keydown-action
    fn dispatch_keydown_action(
        &mut self,
        constellation_chan: &Sender<ConstellationMsg>,
        source_id: &str,
        action: &KeyDownAction,
        tick_duration: u64,
    ) {
        let raw_key = action.value.chars().next().unwrap();
        let key_input_state = match self.input_state_table.get_mut(source_id).unwrap() {
            InputSourceState::Null => unreachable!(),
            InputSourceState::Key(key_input_state) => key_input_state,
            InputSourceState::Pointer(_) => unreachable!(),
        };

        self.input_cancel_list.push(ActionSequence {
            id: Some(source_id.into()),
            actions: ActionsType::Key {
                actions: vec![KeyActionItem::Key(KeyAction::Up(KeyUpAction {
                    value: action.value.clone(),
                }))],
            },
        });

        let keyboard_event = key_input_state.dispatch_keydown(raw_key);
        let cmd_msg = WebDriverCommandMsg::KeyboardAction(self.browsing_context_id, keyboard_event);
        constellation_chan
            .send(ConstellationMsg::WebDriverCommand(cmd_msg))
            .unwrap();
    }

    // https://w3c.github.io/webdriver/#dfn-dispatch-a-keyup-action
    fn dispatch_keyup_action(
        &mut self,
        constellation_chan: &Sender<ConstellationMsg>,
        source_id: &str,
        action: &KeyUpAction,
        tick_duration: u64,
    ) {
        let raw_key = action.value.chars().next().unwrap();
        let key_input_state = match self.input_state_table.get_mut(source_id).unwrap() {
            InputSourceState::Null => unreachable!(),
            InputSourceState::Key(key_input_state) => key_input_state,
            InputSourceState::Pointer(_) => unreachable!(),
        };

        self.input_cancel_list.push(ActionSequence {
            id: Some(source_id.into()),
            actions: ActionsType::Key {
                actions: vec![KeyActionItem::Key(KeyAction::Up(KeyUpAction {
                    value: action.value.clone(),
                }))],
            },
        });

        match key_input_state.dispatch_keyup(raw_key) {
            Some(keyboard_event) => {
                let cmd_msg =
                    WebDriverCommandMsg::KeyboardAction(self.browsing_context_id, keyboard_event);
                constellation_chan
                    .send(ConstellationMsg::WebDriverCommand(cmd_msg))
                    .unwrap();
            },
            _ => (),
        }
    }
}
