/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use embedder_traits::GamepadHapticEffectType;

pub enum GamepadHapticEffectRequestType {
    Play(GamepadHapticEffectType),
    Stop,
}

pub struct GamepadHapticEffectRequest {
    gamepad_index: usize,
    request_type: GamepadHapticEffectRequestType,
    callback: Option<Box<dyn FnOnce(bool)>>,
}

impl GamepadHapticEffectRequest {
    pub(crate) fn new(
        gamepad_index: usize,
        request_type: GamepadHapticEffectRequestType,
        callback: Box<dyn FnOnce(bool)>,
    ) -> Self {
        Self {
            gamepad_index,
            request_type,
            callback: Some(callback),
        }
    }

    pub fn gamepad_index(&self) -> usize {
        self.gamepad_index
    }

    pub fn request_type(&self) -> &GamepadHapticEffectRequestType {
        &self.request_type
    }

    pub fn failed(mut self) {
        if let Some(callback) = self.callback.take() {
            callback(false);
        }
    }

    pub fn succeeded(mut self) {
        if let Some(callback) = self.callback.take() {
            callback(true);
        }
    }
}

impl Drop for GamepadHapticEffectRequest {
    fn drop(&mut self) {
        if let Some(callback) = self.callback.take() {
            callback(false);
        }
    }
}

pub trait GamepadProvider {
    /// Handle a request to play or stop a haptic effect on a connected gamepad.
    fn handle_haptic_effect_request(&self, _request: GamepadHapticEffectRequest) {}
}

pub(crate) struct DefaultGamepadProvider;

impl GamepadProvider for DefaultGamepadProvider {}
