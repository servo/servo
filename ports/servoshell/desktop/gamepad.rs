/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;
use std::sync::mpsc::{Sender, channel};
use std::thread;
use std::time::Duration;

use gilrs::ff::{BaseEffect, BaseEffectType, Effect, EffectBuilder, Repeat, Replay, Ticks};
use gilrs::{Event, EventType, GamepadId, Gilrs};
use log::{debug, warn};
use servo::{
    GamepadDelegate, GamepadEvent, GamepadHapticEffectRequest, GamepadHapticEffectRequestType,
    GamepadHapticEffectType, GamepadIndex, GamepadInputBounds, GamepadSupportedHapticEffects,
    GamepadUpdateType, InputEvent, WebView,
};
use winit::event_loop::EventLoopProxy;

use crate::desktop::event_loop::AppEvent;

pub struct HapticEffect {
    pub effect: Effect,
    pub request: GamepadHapticEffectRequest,
}

pub(crate) struct ServoshellGamepadDelegate {
    sender: Sender<GamepadHapticEffectRequest>,
}

impl ServoshellGamepadDelegate {
    pub(crate) fn maybe_new(event_loop_proxy: EventLoopProxy<AppEvent>) -> Option<Self> {
        let (tx, rx) = channel::<GamepadHapticEffectRequest>();

        let _ = thread::Builder::new()
            .name(String::from("GamepadThread"))
            .spawn(move || {
                let mut haptic_effects: HashMap<usize, HapticEffect> = HashMap::new();
                let mut handle = match Gilrs::new() {
                    Ok(handle) => handle,
                    Err(error) => {
                        warn!("Error creating gamepad input connection ({error})");
                        return;
                    },
                };

                let mut connected_gamepads: Vec<GamepadId> =
                    handle.gamepads().map(|(id, _)| id).collect();

                loop {
                    while let Some(event) =
                        handle.next_event_blocking(Some(Duration::from_millis(100)))
                    {
                        let id: usize = event.id.into();
                        let gamepad = handle.gamepad(event.id);
                        let name = gamepad.name();
                        let index = GamepadIndex(id);

                        if let Some(index) = connected_gamepads
                            .iter()
                            .position(|&gamepad_id| event.id == gamepad_id)
                        {
                            handle.insert_event(Event::new(event.id, EventType::Connected));
                            handle.insert_event(event);

                            connected_gamepads.swap_remove(index);

                            continue;
                        }

                        if matches!(&event.event, EventType::ForceFeedbackEffectCompleted) {
                            match haptic_effects.remove(&id) {
                                Some(haptic_effect) => haptic_effect.request.succeeded(),
                                None => warn!("Failed to find haptic effect for id {id}"),
                            }

                            continue;
                        }

                        if event_loop_proxy
                            .send_event(AppEvent::Gamepad(event, name.to_owned(), index))
                            .is_err()
                        {
                            warn!("Error sending gamepad event to event loop proxy");
                            return;
                        }
                    }

                    while let Ok(request) = rx.try_recv() {
                        match request.request_type() {
                            GamepadHapticEffectRequestType::Play(effect_type) => {
                                Self::play_haptic_effect(
                                    &mut haptic_effects,
                                    &effect_type.clone(),
                                    request,
                                    &mut handle,
                                );
                            },
                            GamepadHapticEffectRequestType::Stop => {
                                Self::stop_haptic_effect(&mut haptic_effects, request);
                            },
                        }
                    }
                }
            });

        Some(Self { sender: tx })
    }

    /// Handle updates to connected gamepads from GilRs
    pub(crate) fn handle_gamepad_events(
        &self,
        event: Event,
        name: String,
        index: GamepadIndex,
        active_webview: WebView,
    ) {
        let mut gamepad_event: Option<GamepadEvent> = None;
        match event.event {
            EventType::ButtonPressed(button, _) => {
                let mapped_index = Self::map_gamepad_button(button);
                // We only want to send this for a valid digital button, aka on/off only
                if !matches!(mapped_index, 6 | 7 | 17) {
                    let update_type = GamepadUpdateType::Button(mapped_index, 1.0);
                    gamepad_event = Some(GamepadEvent::Updated(index, update_type));
                }
            },
            EventType::ButtonReleased(button, _) => {
                let mapped_index = Self::map_gamepad_button(button);
                // We only want to send this for a valid digital button, aka on/off only
                if !matches!(mapped_index, 6 | 7 | 17) {
                    let update_type = GamepadUpdateType::Button(mapped_index, 0.0);
                    gamepad_event = Some(GamepadEvent::Updated(index, update_type));
                }
            },
            EventType::ButtonChanged(button, value, _) => {
                let mapped_index = Self::map_gamepad_button(button);
                // We only want to send this for a valid non-digital button, aka the triggers
                if matches!(mapped_index, 6 | 7) {
                    let update_type = GamepadUpdateType::Button(mapped_index, value as f64);
                    gamepad_event = Some(GamepadEvent::Updated(index, update_type));
                }
            },
            EventType::AxisChanged(axis, value, _) => {
                // Map axis index and value to represent Standard Gamepad axis
                // <https://www.w3.org/TR/gamepad/#dfn-represents-a-standard-gamepad-axis>
                let mapped_axis: usize = match axis {
                    gilrs::Axis::LeftStickX => 0,
                    gilrs::Axis::LeftStickY => 1,
                    gilrs::Axis::RightStickX => 2,
                    gilrs::Axis::RightStickY => 3,
                    _ => 4, // Other axes do not map to "standard" gamepad mapping and are ignored
                };
                if mapped_axis < 4 {
                    // The Gamepad spec designates down as positive and up as negative.
                    // GilRs does the inverse of this, so correct for it here.
                    let axis_value = match mapped_axis {
                        0 | 2 => value,
                        1 | 3 => -value,
                        _ => 0., // Should not reach here
                    };
                    let update_type = GamepadUpdateType::Axis(mapped_axis, axis_value as f64);
                    gamepad_event = Some(GamepadEvent::Updated(index, update_type));
                }
            },
            EventType::Connected => {
                let bounds = GamepadInputBounds {
                    axis_bounds: (-1.0, 1.0),
                    button_bounds: (0.0, 1.0),
                };
                // GilRs does not yet support trigger rumble
                let supported_haptic_effects = GamepadSupportedHapticEffects {
                    supports_dual_rumble: true,
                    supports_trigger_rumble: false,
                };
                gamepad_event = Some(GamepadEvent::Connected(
                    index,
                    name,
                    bounds,
                    supported_haptic_effects,
                ));
            },
            EventType::Disconnected => {
                gamepad_event = Some(GamepadEvent::Disconnected(index));
            },
            _ => {},
        }

        if let Some(event) = gamepad_event {
            active_webview.notify_input_event(InputEvent::Gamepad(event));
        }
    }

    // Map button index and value to represent Standard Gamepad button
    // <https://www.w3.org/TR/gamepad/#dfn-represents-a-standard-gamepad-button>
    fn map_gamepad_button(button: gilrs::Button) -> usize {
        match button {
            gilrs::Button::South => 0,
            gilrs::Button::East => 1,
            gilrs::Button::West => 2,
            gilrs::Button::North => 3,
            gilrs::Button::LeftTrigger => 4,
            gilrs::Button::RightTrigger => 5,
            gilrs::Button::LeftTrigger2 => 6,
            gilrs::Button::RightTrigger2 => 7,
            gilrs::Button::Select => 8,
            gilrs::Button::Start => 9,
            gilrs::Button::LeftThumb => 10,
            gilrs::Button::RightThumb => 11,
            gilrs::Button::DPadUp => 12,
            gilrs::Button::DPadDown => 13,
            gilrs::Button::DPadLeft => 14,
            gilrs::Button::DPadRight => 15,
            gilrs::Button::Mode => 16,
            _ => 17, // Other buttons do not map to "standard" gamepad mapping and are ignored
        }
    }

    fn play_haptic_effect(
        haptic_effects: &mut HashMap<usize, HapticEffect>,
        effect_type: &GamepadHapticEffectType,
        request: GamepadHapticEffectRequest,
        handle: &mut Gilrs,
    ) {
        let index = request.gamepad_index();
        let GamepadHapticEffectType::DualRumble(params) = effect_type;

        let Some(connected_gamepad) = handle
            .gamepads()
            .find(|gamepad| usize::from(gamepad.0) == index)
        else {
            debug!("Couldn't find connected gamepad to play haptic effect on");
            request.failed();
            return;
        };

        let start_delay = Ticks::from_ms(params.start_delay as u32);
        let duration = Ticks::from_ms(params.duration as u32);
        let strong_magnitude = (params.strong_magnitude * u16::MAX as f64).round() as u16;
        let weak_magnitude = (params.weak_magnitude * u16::MAX as f64).round() as u16;

        let scheduling = Replay {
            after: start_delay,
            play_for: duration,
            with_delay: Ticks::from_ms(0),
        };
        let effect = EffectBuilder::new()
            .add_effect(BaseEffect {
                kind: BaseEffectType::Strong {
                    magnitude: strong_magnitude,
                },
                scheduling,
                envelope: Default::default(),
            })
            .add_effect(BaseEffect {
                kind: BaseEffectType::Weak {
                    magnitude: weak_magnitude,
                },
                scheduling,
                envelope: Default::default(),
            })
            .repeat(Repeat::For(start_delay + duration))
            .add_gamepad(&connected_gamepad.1)
            .finish(handle)
            .expect(
                "Failed to create haptic effect, ensure connected gamepad supports force feedback.",
            );

        haptic_effects.insert(index, HapticEffect { effect, request });
        haptic_effects[&index]
            .effect
            .play()
            .expect("Failed to play haptic effect.");
    }

    fn stop_haptic_effect(
        haptic_effects: &mut HashMap<usize, HapticEffect>,
        request: GamepadHapticEffectRequest,
    ) {
        let index = request.gamepad_index();

        let Some(haptic_effect) = haptic_effects.get(&index) else {
            request.failed();
            return;
        };

        let stopped_successfully = match haptic_effect.effect.stop() {
            Ok(()) => true,
            Err(e) => {
                debug!("Failed to stop haptic effect: {:?}", e);
                false
            },
        };
        haptic_effects.remove(&index);

        if stopped_successfully {
            request.succeeded();
        } else {
            request.failed();
        }
    }
}

impl GamepadDelegate for ServoshellGamepadDelegate {
    fn handle_haptic_effect_request(&self, request: GamepadHapticEffectRequest) {
        if self.sender.send(request).is_err() {
            warn!("Haptic effect couldn't be played!")
        }
    }
}
