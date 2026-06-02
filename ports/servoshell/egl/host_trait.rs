/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use servo::{InputMethodControl, LoadStatus, MediaSessionPlaybackState};

/// Callbacks implemented by embedder. Called by our RunningAppState, generally on behalf of Servo.
pub trait HostTrait {
    fn show_alert(&self, message: String);
    /// Notify that the load status of the page has changed.
    /// Started:
    ///  - "Reload button" should be disabled.
    ///  - "Stop button" should be enabled.
    ///  - Throbber starts spinning.
    /// Complete:
    ///  - "Reload button" should be enabled.
    ///  - "Stop button" should be disabled.
    ///  - Throbber stops spinning.
    fn notify_load_status_changed(&self, load_status: LoadStatus);
    /// Page title has changed.
    fn on_title_changed(&self, title: Option<String>);
    /// Page URL has changed.
    fn on_url_changed(&self, url: String);
    /// Back/forward state has changed.
    /// Back/forward buttons need to be disabled/enabled.
    fn on_history_changed(&self, can_go_back: bool, can_go_forward: bool);
    /// Servo finished shutting down.
    fn on_shutdown_complete(&self);
    /// A text input is focused.
    fn on_ime_show(&self, input_method_control: InputMethodControl);
    /// Input lost focus
    fn on_ime_hide(&self);
    /// Called when we get the media session metadata/
    fn on_media_session_metadata(&self, title: String, artist: String, album: String);
    /// Called when the media session playback state changes.
    fn on_media_session_playback_state_change(&self, state: MediaSessionPlaybackState);
    /// Called when the media session position state is set.
    fn on_media_session_set_position_state(&self, duration: f64, position: f64, playback_rate: f64);
    /// Called when we get a panic message from constellation
    fn on_panic(&self, reason: String, backtrace: Option<String>);
}
