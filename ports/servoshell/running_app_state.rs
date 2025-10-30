/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Shared state and methods for desktop and EGL implementations.

use std::cell::RefCell;
use std::collections::HashMap;

use crossbeam_channel::Sender;
use servo::base::generic_channel::GenericSender;
use servo::base::id::WebViewId;
use servo::ipc_channel::ipc::IpcSender;
use servo::webrender_api::units::DeviceVector2D;
use servo::{
    InputEvent, InputEventId, Scroll, TraversalId, WebDriverJSResult, WebDriverLoadStatus,
    WebDriverSenders, WebView, WheelEvent,
};

pub struct RunningAppStateBase {
    pub(crate) webdriver_senders: RefCell<WebDriverSenders>,

    /// A [`HashMap`] of pending WebDriver events. It is the WebDriver embedder's responsibility
    /// to inform the WebDriver server when the event has been fully handled. This map is used
    /// to report back to WebDriver when that happens.
    pub(crate) pending_webdriver_events: RefCell<HashMap<InputEventId, Sender<()>>>,
}

impl RunningAppStateBase {
    pub fn new() -> Self {
        Self {
            webdriver_senders: RefCell::default(),
            pending_webdriver_events: Default::default(),
        }
    }
}

pub trait RunningAppStateTrait {
    fn base(&self) -> &RunningAppStateBase;

    #[allow(dead_code)]
    fn base_mut(&mut self) -> &mut RunningAppStateBase;

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
        // TODO: Scroll events triggered by wheel events should happen as
        // a default event action in the compositor.
        let scroll_event = match &input_event {
            InputEvent::Wheel(WheelEvent { delta, point }) => {
                let scroll =
                    Scroll::Delta(DeviceVector2D::new(-delta.x as f32, -delta.y as f32).into());
                Some((scroll, *point))
            },
            _ => None,
        };

        let event_id = webview.notify_input_event(input_event);
        if let Some(response_sender) = response_sender {
            self.base()
                .pending_webdriver_events
                .borrow_mut()
                .insert(event_id, response_sender);
        }

        if let Some((scroll, scroll_point)) = scroll_event {
            webview.notify_scroll_event(scroll, scroll_point);
        }
    }
}
