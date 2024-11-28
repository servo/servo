/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use arboard::Clipboard;
use base::id::{BrowsingContextId, WebViewId};
use compositing_traits::ConstellationMsg;
use crossbeam_channel::Sender;
use embedder_traits::{CompositorEventVariant, EmbedderMsg, PromptDefinition};
use ipc_channel::ipc;
use script_traits::webdriver_msg::{WebDriverJSResult, WebDriverScriptCommand};
use script_traits::{TraversalDirection, WebDriverCommandMsg};
use servo_url::ServoUrl;
use url::Url;
use webrender_api::units::DeviceIntRect;

use crate::compositor::IOCompositor;
#[cfg(linux)]
use crate::context_menu::ContextMenuResult;
use crate::servo::send_to_constellation;
use crate::window::Window;

/// A web view is an area to display web browsing context. It's what user will treat as a "web page".
#[derive(Clone, Debug)]
pub struct WebView {
    /// Webview ID
    pub webview_id: WebViewId,
    /// The position and size of the webview.
    pub rect: DeviceIntRect,
}

impl WebView {
    /// Create a web view from Winit window.
    pub fn new(webview_id: WebViewId, rect: DeviceIntRect) -> Self {
        Self { webview_id, rect }
    }

    /// Set the webview size.
    pub fn set_size(&mut self, rect: DeviceIntRect) {
        self.rect = rect;
    }
}

/// A panel is a special web view that focus on controlling states around window.
/// It could be treated as the control panel or navigation bar of the window depending on usages.
///
/// At the moment, following Web API is supported:
/// - Close window: `window.close()`
/// - Navigate to previous page: `window.prompt('PREV')`
/// - Navigate to next page: `window.prompt('FORWARD')`
/// - Refresh the page: `window.prompt('REFRESH')`
/// - Minimize the window: `window.prompt('MINIMIZE')`
/// - Maximize the window: `window.prompt('MAXIMIZE')`
/// - Navigate to a specific URL: `window.prompt('NAVIGATE_TO:${url}')`
pub struct Panel {
    /// The panel's webview
    pub(crate) webview: WebView,
    /// The URL to load when the panel gets loaded
    pub(crate) initial_url: servo_url::ServoUrl,
}

impl Window {
    /// Handle servo messages with corresponding web view ID.
    pub fn handle_servo_messages_with_webview(
        &mut self,
        webview_id: WebViewId,
        message: EmbedderMsg,
        sender: &Sender<ConstellationMsg>,
        clipboard: Option<&mut Clipboard>,
        _compositor: &mut IOCompositor,
    ) {
        log::trace!("Servo WebView {webview_id:?} is handling Embedder message: {message:?}",);
        match message {
            EmbedderMsg::LoadStart |
            EmbedderMsg::HeadParsed |
            EmbedderMsg::WebViewOpened(_) |
            EmbedderMsg::WebViewClosed(_) => {
                // Most WebView messages are ignored because it's done by compositor.
                log::trace!("Servo WebView {webview_id:?} ignores this message: {message:?}")
            },
            EmbedderMsg::WebViewFocused(w) => {
                self.close_context_menu(sender);
                log::debug!(
                    "Servo Window {:?}'s webview {} has loaded completely.",
                    self.id(),
                    w
                );
            },
            EmbedderMsg::LoadComplete => {
                self.window.request_redraw();
                send_to_constellation(sender, ConstellationMsg::FocusWebView(webview_id));
            },
            EmbedderMsg::AllowNavigationRequest(id, _url) => {
                // TODO should provide a API for users to check url
                send_to_constellation(sender, ConstellationMsg::AllowNavigationResponse(id, true));
            },
            EmbedderMsg::GetClipboardContents(sender) => {
                let contents = clipboard
                    .map(|c| {
                        c.get_text().unwrap_or_else(|e| {
                            log::warn!(
                                "Servo WebView {webview_id:?} failed to get clipboard content: {}",
                                e
                            );
                            String::new()
                        })
                    })
                    .unwrap_or_default();
                if let Err(e) = sender.send(contents) {
                    log::warn!(
                        "Servo WebView {webview_id:?} failed to send clipboard content: {}",
                        e
                    );
                }
            },
            EmbedderMsg::SetClipboardContents(text) => {
                if let Some(c) = clipboard {
                    if let Err(e) = c.set_text(text) {
                        log::warn!(
                            "Servo WebView {webview_id:?} failed to set clipboard contents: {}",
                            e
                        );
                    }
                }
            },
            EmbedderMsg::HistoryChanged(list, index) => {
                self.update_history(&list, index);
                let url = list.get(index).unwrap();
                if let Some(panel) = self.panel.as_ref() {
                    let (tx, rx) = ipc::channel::<WebDriverJSResult>().unwrap();
                    send_to_constellation(
                        sender,
                        ConstellationMsg::WebDriverCommand(WebDriverCommandMsg::ScriptCommand(
                            BrowsingContextId::from(panel.webview.webview_id),
                            WebDriverScriptCommand::ExecuteScript(
                                format!("window.navbar.setNavbarUrl('{}')", url.as_str()),
                                tx,
                            ),
                        )),
                    );
                    let _ = rx.recv();
                }
            },
            EmbedderMsg::EventDelivered(event) => {
                if let CompositorEventVariant::MouseButtonEvent = event {
                    send_to_constellation(sender, ConstellationMsg::FocusWebView(webview_id));
                }
            },
            EmbedderMsg::ShowContextMenu(_sender, _title, _options) => {
                // TODO: Implement context menu
            },
            e => {
                log::trace!("Servo WebView isn't supporting this message yet: {e:?}")
            },
        }
    }

    /// Handle servo messages with main panel. Return true it requests a new window.
    pub fn handle_servo_messages_with_panel(
        &mut self,
        panel_id: WebViewId,
        message: EmbedderMsg,
        sender: &Sender<ConstellationMsg>,
        clipboard: Option<&mut Clipboard>,
        _compositor: &mut IOCompositor,
    ) -> bool {
        log::trace!("Servo Panel {panel_id:?} is handling Embedder message: {message:?}",);
        match message {
            EmbedderMsg::LoadStart |
            EmbedderMsg::HeadParsed |
            EmbedderMsg::WebViewOpened(_) |
            EmbedderMsg::WebViewClosed(_) => {
                // Most WebView messages are ignored because it's done by compositor.
                log::trace!("Servo Panel ignores this message: {message:?}")
            },
            EmbedderMsg::WebViewFocused(w) => {
                self.close_context_menu(sender);
                log::debug!(
                    "Servo Window {:?}'s panel {} has loaded completely.",
                    self.id(),
                    w
                );
            },
            EmbedderMsg::LoadComplete => {
                self.window.request_redraw();
                send_to_constellation(sender, ConstellationMsg::FocusWebView(panel_id));

                self.create_webview(sender, self.panel.as_ref().unwrap().initial_url.clone());
            },
            EmbedderMsg::AllowNavigationRequest(id, _url) => {
                // The panel shouldn't navigate to other pages.
                send_to_constellation(sender, ConstellationMsg::AllowNavigationResponse(id, false));
            },
            EmbedderMsg::HistoryChanged(..) | EmbedderMsg::ChangePageTitle(..) => {
                log::trace!("Servo Panel ignores this message: {message:?}")
            },
            EmbedderMsg::Prompt(definition, _origin) => {
                match definition {
                    PromptDefinition::Input(msg, _, prompt_sender) => {
                        let _ = prompt_sender.send(None);
                        if let Some(webview) = &self.webview {
                            let id = webview.webview_id;

                            if msg.starts_with("NAVIGATE_TO:") {
                                let unparsed_url = msg.strip_prefix("NAVIGATE_TO:").unwrap();
                                let url = match Url::parse(unparsed_url) {
                                    Ok(url_parsed) => url_parsed,
                                    Err(e) => {
                                        if e == url::ParseError::RelativeUrlWithoutBase {
                                            Url::parse(&format!("https://{}", unparsed_url))
                                                .unwrap()
                                        } else {
                                            panic!("Servo Panel failed to parse URL: {}", e);
                                        }
                                    },
                                };

                                send_to_constellation(
                                    sender,
                                    ConstellationMsg::LoadUrl(id, ServoUrl::from_url(url)),
                                );
                            } else {
                                match msg.as_str() {
                                    "PREV" => {
                                        send_to_constellation(
                                            sender,
                                            ConstellationMsg::TraverseHistory(
                                                id,
                                                TraversalDirection::Back(1),
                                            ),
                                        );
                                        // TODO Set EmbedderMsg::Status to None
                                    },
                                    "FORWARD" => {
                                        send_to_constellation(
                                            sender,
                                            ConstellationMsg::TraverseHistory(
                                                id,
                                                TraversalDirection::Forward(1),
                                            ),
                                        );
                                        // TODO Set EmbedderMsg::Status to None
                                    },
                                    "REFRESH" => {
                                        send_to_constellation(sender, ConstellationMsg::Reload(id));
                                    },
                                    "NEW_WINDOW" => {
                                        return true;
                                    },
                                    "MINIMIZE" => {
                                        self.window.set_minimized(true);
                                    },
                                    "MAXIMIZE" | "DBCLICK_PANEL" => {
                                        let is_maximized = self.window.is_maximized();
                                        self.window.set_maximized(!is_maximized);
                                    },
                                    "DRAG_WINDOW" => {
                                        let _ = self.window.drag_window();
                                    },
                                    e => log::trace!(
                                        "Servo Panel isn't supporting this prompt message yet: {e}"
                                    ),
                                }
                            }
                        }
                    },
                    _ => log::trace!("Servo Panel isn't supporting this prompt yet"),
                }
            },
            EmbedderMsg::GetClipboardContents(sender) => {
                let contents = clipboard
                    .map(|c| {
                        c.get_text().unwrap_or_else(|e| {
                            log::warn!("Servo Panel failed to get clipboard content: {}", e);
                            String::new()
                        })
                    })
                    .unwrap_or_default();
                if let Err(e) = sender.send(contents) {
                    log::warn!("Servo Panel failed to send clipboard content: {}", e);
                }
            },
            EmbedderMsg::SetClipboardContents(text) => {
                if let Some(c) = clipboard {
                    if let Err(e) = c.set_text(text) {
                        log::warn!("Servo Panel failed to set clipboard contents: {}", e);
                    }
                }
            },
            EmbedderMsg::EventDelivered(event) => {
                if let CompositorEventVariant::MouseButtonEvent = event {
                    send_to_constellation(sender, ConstellationMsg::FocusWebView(panel_id));
                }
            },
            e => {
                log::trace!("Servo Panel isn't supporting this message yet: {e:?}")
            },
        }
        false
    }

    /// Handle servo messages with main panel. Return true it requests a new window.
    #[cfg(linux)]
    pub fn handle_servo_messages_with_context_menu(
        &mut self,
        webview_id: WebViewId,
        message: EmbedderMsg,
        sender: &Sender<ConstellationMsg>,
        _clipboard: Option<&mut Clipboard>,
        _compositor: &mut IOCompositor,
    ) -> bool {
        log::trace!("Servo Context Menu {webview_id:?} is handling Embedder message: {message:?}",);
        match message {
            EmbedderMsg::Prompt(definition, _origin) => match definition {
                PromptDefinition::Input(msg, _, prompt_sender) => {
                    let _ = prompt_sender.send(None);
                    if msg.starts_with("CONTEXT_MENU:") {
                        let json_str_msg = msg.strip_prefix("CONTEXT_MENU:").unwrap();
                        let result =
                            serde_json::from_str::<ContextMenuResult>(json_str_msg).unwrap();

                        self.handle_context_menu_event(sender, result);
                    }
                },
                _ => log::trace!("Servo context menu isn't supporting this prompt yet"),
            },
            e => {
                log::trace!("Servo context menu isn't supporting this message yet: {e:?}")
            },
        }
        false
    }
}
