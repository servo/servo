/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use servo::webrender_api::units::DeviceIntRect;
use servo::{
    AlertResponse, ConfirmResponse, InputMethodType, LoadStatus, MediaSessionPlaybackState,
    PromptResponse,
};

/// Callbacks implemented by embedder. Called by our RunningAppState, generally on behalf of Servo.
pub trait HostTrait {
    /// Show a trusted alert from Servo or the embedder.
    fn show_trusted_alert(&self, message: String);
    /// Show a trusted Yes/No prompt from Servo or the embedder.
    fn show_trusted_yes_no_dialog(&self, message: String) -> YesNoResponse;
    /// Show an untrusted `alert()` dialog from web content.
    fn show_untrusted_alert(&self, message: String) -> AlertResponse;
    /// Show an untrusted `confirm()` dialog from web content.
    fn show_untrusted_confirm(&self, message: String) -> ConfirmResponse;
    /// Show an untrusted `prompt()` dialog from web content.
    fn show_untrusted_prompt(&self, message: String, default: String) -> PromptResponse;
    /// Show context menu
    fn show_context_menu(&self, title: Option<String>, items: Vec<String>);
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
    /// Allow Navigation.
    fn on_allow_navigation(&self, url: String) -> bool;
    /// Page URL has changed.
    fn on_url_changed(&self, url: String);
    /// Back/forward state has changed.
    /// Back/forward buttons need to be disabled/enabled.
    fn on_history_changed(&self, can_go_back: bool, can_go_forward: bool);
    /// Page animation state has changed. If animating, it's recommended
    /// that the embedder doesn't wait for the wake function to be called
    /// to call perform_updates. Usually, it means doing:
    /// ```rust
    /// while true {
    ///     servo.perform_updates();
    ///     servo.present_if_needed();
    /// }
    /// ```
    /// . This will end up calling flush
    /// which will call swap_buffer which will be blocking long enough to limit
    /// drawing at 60 FPS.
    /// If not animating, call perform_updates only when needed (when the embedder
    /// has events for Servo, or Servo has woken up the embedder event loop via
    /// EventLoopWaker).
    fn on_animating_changed(&self, animating: bool);
    /// Servo finished shutting down.
    fn on_shutdown_complete(&self);
    /// A text input is focused.
    fn on_ime_show(
        &self,
        input_type: InputMethodType,
        text: Option<(String, i32)>,
        multiline: bool,
        bounds: DeviceIntRect,
    );
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

pub enum YesNoResponse {
    /// The user chose Yes.
    Yes,
    /// The user chose No, or the dialog was otherwise dismissed or ignored.
    No,
}
