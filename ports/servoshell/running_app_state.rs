/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Shared state and methods for desktop and EGL implementations.

use std::cell::RefCell;

use servo::base::generic_channel::GenericSender;
use servo::base::id::WebViewId;
use servo::ipc_channel::ipc::IpcSender;
use servo::{TraversalId, WebDriverJSResult, WebDriverLoadStatus, WebDriverSenders};

pub struct RunningAppStateBase {
    pub(crate) webdriver_senders: RefCell<WebDriverSenders>,
}

impl RunningAppStateBase {
    pub fn new() -> Self {
        Self {
            webdriver_senders: RefCell::default(),
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
}
