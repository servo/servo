/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Shared state and methods for desktop and EGL implementations.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

use crossbeam_channel::{Receiver, Sender};
use euclid::Rect;
use image::{DynamicImage, ImageFormat, RgbaImage};
use log::{error, info, warn};
use servo::base::generic_channel::GenericSender;
use servo::base::id::WebViewId;
use servo::ipc_channel::ipc::IpcSender;
use servo::style_traits::CSSPixel;
use servo::{
    InputEvent, InputEventId, ScreenshotCaptureError, Servo, TraversalId, WebDriverCommandMsg,
    WebDriverJSResult, WebDriverLoadStatus, WebDriverScriptCommand, WebDriverSenders, WebView,
};
use url::Url;

use crate::prefs::ServoShellPreferences;

pub struct RunningAppStateBase {
    pub(crate) webdriver_senders: RefCell<WebDriverSenders>,

    /// A [`HashMap`] of pending WebDriver events. It is the WebDriver embedder's responsibility
    /// to inform the WebDriver server when the event has been fully handled. This map is used
    /// to report back to WebDriver when that happens.
    pub(crate) pending_webdriver_events: RefCell<HashMap<InputEventId, Sender<()>>>,

    /// A [`Receiver`] for receiving commands from a running WebDriver server, if WebDriver
    /// was enabled.
    pub(crate) webdriver_receiver: Option<Receiver<WebDriverCommandMsg>>,

    /// servoshell specific preferences created during startup of the application.
    pub(crate) servoshell_preferences: ServoShellPreferences,

    /// A handle to the Servo instance.
    pub(crate) servo: Servo,

    /// Whether or not the application has achieved stable image output. This is used
    /// for the `exit_after_stable_image` option.
    pub(crate) achieved_stable_image: Rc<Cell<bool>>,
}

impl RunningAppStateBase {
    pub fn new(
        servoshell_preferences: ServoShellPreferences,
        servo: Servo,
        webdriver_receiver: Option<Receiver<WebDriverCommandMsg>>,
    ) -> Self {
        Self {
            webdriver_senders: RefCell::default(),
            pending_webdriver_events: Default::default(),
            webdriver_receiver,
            servoshell_preferences,
            servo,
            achieved_stable_image: Default::default(),
        }
    }
}

pub trait RunningAppStateTrait {
    fn base(&self) -> &RunningAppStateBase;

    #[allow(dead_code)]
    fn base_mut(&mut self) -> &mut RunningAppStateBase;

    fn webview_by_id(&self, id: WebViewId) -> Option<WebView>;

    fn servoshell_preferences(&self) -> &ServoShellPreferences {
        &self.base().servoshell_preferences
    }

    fn servo(&self) -> &Servo {
        &self.base().servo
    }

    fn webdriver_receiver(&self) -> Option<&Receiver<WebDriverCommandMsg>> {
        self.base().webdriver_receiver.as_ref()
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
        webview_id: WebViewId,
        input_event: InputEvent,
        response_sender: Option<Sender<()>>,
    ) {
        if let Some(webview) = self.webview_by_id(webview_id) {
            let event_id = webview.notify_input_event(input_event);
            if let Some(response_sender) = response_sender {
                self.base()
                    .pending_webdriver_events
                    .borrow_mut()
                    .insert(event_id, response_sender);
            }
        } else {
            error!("Could not find WebView ({webview_id:?}) for WebDriver event: {input_event:?}");
        };
    }

    fn handle_webdriver_screenshot(
        &self,
        webview_id: WebViewId,
        rect: Option<Rect<f32, CSSPixel>>,
        result_sender: Sender<Result<RgbaImage, ScreenshotCaptureError>>,
    ) {
        if let Some(webview) = self.webview_by_id(webview_id) {
            let rect = rect.map(|rect| rect.to_box2d().into());
            webview.take_screenshot(rect, move |result| {
                if let Err(error) = result_sender.send(result) {
                    warn!("Failed to send response to TakeScreenshot: {error}");
                }
            });
        } else if let Err(error) =
            result_sender.send(Err(ScreenshotCaptureError::WebViewDoesNotExist))
        {
            error!("Failed to send response to TakeScreenshot: {error}");
        }
    }

    fn handle_webdriver_script_command(&self, script_command: &WebDriverScriptCommand) {
        match script_command {
            WebDriverScriptCommand::ExecuteScript(_webview_id, response_sender) |
            WebDriverScriptCommand::ExecuteAsyncScript(_webview_id, response_sender) => {
                // Give embedder a chance to interrupt the script command.
                // Webdriver only handles 1 script command at a time, so we can
                // safely set a new interrupt sender and remove the previous one here.
                self.set_script_command_interrupt_sender(Some(response_sender.clone()));
            },
            WebDriverScriptCommand::AddLoadStatusSender(webview_id, load_status_sender) => {
                self.set_load_status_sender(*webview_id, load_status_sender.clone());
            },
            WebDriverScriptCommand::RemoveLoadStatusSender(webview_id) => {
                self.remove_load_status_sender(*webview_id);
            },
            _ => {
                self.set_script_command_interrupt_sender(None);
            },
        }
    }

    fn handle_webdriver_load_url(
        &self,
        webview_id: WebViewId,
        url: Url,
        load_status_sender: GenericSender<WebDriverLoadStatus>,
    ) {
        if let Some(webview) = self.webview_by_id(webview_id) {
            info!("Loading URL in webview {}: {}", webview_id, url);
            self.set_load_status_sender(webview_id, load_status_sender);
            webview.load(url);
        }
    }

    /// If we are exiting after achieving a stable image or we want to save the display of the
    /// [`WebView`] to an image file, request a screenshot of the [`WebView`].
    fn maybe_request_screenshot(&self, webview: WebView) {
        let output_path = self.servoshell_preferences().output_image_path.clone();
        if !self.servoshell_preferences().exit_after_stable_image && output_path.is_none() {
            return;
        }

        // Never request more than a single screenshot for now.
        let achieved_stable_image = self.base().achieved_stable_image.clone();
        if achieved_stable_image.get() {
            return;
        }

        webview.take_screenshot(None, move |image| {
            achieved_stable_image.set(true);

            let Some(output_path) = output_path else {
                return;
            };

            let image = match image {
                Ok(image) => image,
                Err(error) => {
                    error!("Could not take screenshot: {error:?}");
                    return;
                },
            };

            let image_format = ImageFormat::from_path(&output_path).unwrap_or(ImageFormat::Png);
            if let Err(error) =
                DynamicImage::ImageRgba8(image).save_with_format(output_path, image_format)
            {
                error!("Failed to save screenshot: {error}.");
            }
        });
    }
}
