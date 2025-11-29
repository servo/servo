/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use servo::{GamepadHapticEffectType, IpcSender, WebView};

/// A dummy version of [`crate::desktop::GamepadSupport`] used to avoid conditional compilation in
/// servoshell and as a skeleton to implement gamepad support for platforms that do not
/// currently support it.
pub(crate) struct GamepadSupport;

impl GamepadSupport {
    pub(crate) fn maybe_new() -> Option<Self> {
        None
    }

    pub(crate) fn handle_gamepad_events(&mut self, _active_webview: WebView) {
        unreachable!("Dummy gamepad support should never be called.");
    }

    pub(crate) fn play_haptic_effect(
        &mut self,
        _index: usize,
        _effect_type: GamepadHapticEffectType,
        _effect_complete_sender: IpcSender<bool>,
    ) {
        unreachable!("Dummy gamepad support should never be called.");
    }

    pub(crate) fn stop_haptic_effect(&mut self, _index: usize) -> bool {
        unreachable!("Dummy gamepad support should never be called.");
    }
}
