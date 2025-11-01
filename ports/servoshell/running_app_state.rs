/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Shared state and methods for desktop and EGL implementations.

use std::cell::RefCell;
use std::collections::HashMap;

use crossbeam_channel::Sender;
use euclid::Rect;
use image::RgbaImage;
use log::warn;
use servo::base::generic_channel::GenericSender;
use servo::base::id::WebViewId;
use servo::ipc_channel::ipc::IpcSender;
use servo::style_traits::CSSPixel;
use servo::{
    InputEvent, InputEventId, ScreenshotCaptureError, Servo, TraversalId, WebDriverJSResult,
    WebDriverLoadStatus, WebDriverSenders, WebView,
};

use crate::prefs::ServoShellPreferences;

pub struct RunningAppStateBase {
    pub(crate) webdriver_senders: RefCell<WebDriverSenders>,

    /// A [`HashMap`] of pending WebDriver events. It is the WebDriver embedder's responsibility
    /// to inform the WebDriver server when the event has been fully handled. This map is used
    /// to report back to WebDriver when that happens.
    pub(crate) pending_webdriver_events: RefCell<HashMap<InputEventId, Sender<()>>>,

    /// servoshell specific preferences created during startup of the application.
    pub(crate) servoshell_preferences: ServoShellPreferences,

    /// A handle to the Servo instance.
    pub(crate) servo: Servo,
}

impl RunningAppStateBase {
    pub fn new(servoshell_preferences: ServoShellPreferences, servo: Servo) -> Self {
        Self {
            webdriver_senders: RefCell::default(),
            pending_webdriver_events: Default::default(),
            servoshell_preferences,
            servo,
        }
    }
}

pub trait RunningAppStateTrait {
    fn base(&self) -> &RunningAppStateBase;

    #[allow(dead_code)]
    fn base_mut(&mut self) -> &mut RunningAppStateBase;

    fn servoshell_preferences(&self) -> &ServoShellPreferences {
        &self.base().servoshell_preferences
    }

    fn servo(&self) -> &Servo {
        &self.base().servo
    }

    fn set_pending_traversal(
        &self,
        traversal_id: TraversalId,
        sender: GenericSender<WebDriverLoadStatus>,
    ) {
        self.base()
            .webdriver_senders
            .borrow_mut()
            .pending_traversals
            .insert(traversal_id, sender);
    }

    fn set_load_status_sender(
        &self,
        webview_id: WebViewId,
        sender: GenericSender<WebDriverLoadStatus>,
    ) {
        self.base()
            .webdriver_senders
            .borrow_mut()
            .load_status_senders
            .insert(webview_id, sender);
    }

    fn remove_load_status_sender(&self, webview_id: WebViewId) {
        self.base()
            .webdriver_senders
            .borrow_mut()
            .load_status_senders
            .remove(&webview_id);
    }

    fn set_script_command_interrupt_sender(&self, sender: Option<IpcSender<WebDriverJSResult>>) {
        self.base()
            .webdriver_senders
            .borrow_mut()
            .script_evaluation_interrupt_sender = sender;
    }

    fn handle_webdriver_input_event(
        &self,
        webview: WebView,
        input_event: InputEvent,
        response_sender: Option<Sender<()>>,
    ) {
        let event_id = webview.notify_input_event(input_event);
        if let Some(response_sender) = response_sender {
            self.base()
                .pending_webdriver_events
                .borrow_mut()
                .insert(event_id, response_sender);
        }
    }

    fn handle_webdriver_screenshot(
        &self,
        webview: WebView,
        rect: Option<Rect<f32, CSSPixel>>,
        result_sender: Sender<Result<RgbaImage, ScreenshotCaptureError>>,
    ) {
        let rect = rect.map(|rect| rect.to_box2d().into());
        webview.take_screenshot(rect, move |result| {
            if let Err(error) = result_sender.send(result) {
                warn!("Failed to send response to TakeScreenshot: {error}");
            }
        });
    }
}
