/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![crate_name = "webdriver_server"]
#![crate_type = "rlib"]
#![deny(unsafe_code)]

mod actions;
mod capabilities;
mod script_argument_extraction;
mod session;
mod timeout;
mod user_prompt;

use std::borrow::ToOwned;
use std::cell::{Cell, LazyCell, RefCell};
use std::collections::{BTreeMap, HashMap};
use std::io::Cursor;
use std::net::{SocketAddr, SocketAddrV4};
use std::thread::sleep;
use std::time::{Duration, Instant};
use std::{env, fmt, process, thread};

use base::generic_channel::{self, GenericSender, RoutedReceiver};
use base::id::{BrowsingContextId, WebViewId};
use base64::Engine;
use capabilities::ServoCapabilities;
use cookie::{CookieBuilder, Expiration, SameSite};
use crossbeam_channel::{Receiver, RecvTimeoutError, Sender, after, select, unbounded};
use embedder_traits::{
    CustomHandlersAutomationMode, EventLoopWaker, ImeEvent, InputEvent, JSValue,
    JavaScriptEvaluationError, JavaScriptEvaluationResultSerializationError, MouseButton,
    WebDriverCommandMsg, WebDriverFrameId, WebDriverJSResult, WebDriverLoadStatus,
    WebDriverScriptCommand,
};
use euclid::{Point2D, Rect, Size2D};
use http::method::Method;
use image::{DynamicImage, ImageFormat};
use ipc_channel::ipc::{self, IpcReceiver, TryRecvError};
use keyboard_types::webdriver::{Event as DispatchStringEvent, KeyInputState, send_keys};
use keyboard_types::{Code, Key, KeyState, KeyboardEvent, Location, NamedKey};
use log::{debug, error, info};
use serde::de::{Deserializer, MapAccess, Visitor};
use serde::ser::Serializer;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use servo_config::prefs::{self, PrefValue, Preferences};
use servo_geometry::DeviceIndependentIntRect;
use servo_url::ServoUrl;
use style_traits::CSSPixel;
use time::OffsetDateTime;
use uuid::Uuid;
#[cfg(not(any(target_env = "ohos", target_os = "android")))]
use webdriver::actions::PointerMoveAction;
use webdriver::actions::{
    ActionSequence, ActionsType, KeyAction, KeyActionItem, KeyDownAction, KeyUpAction,
    PointerAction, PointerActionItem, PointerActionParameters, PointerDownAction, PointerOrigin,
    PointerType, PointerUpAction,
};
use webdriver::capabilities::CapabilitiesMatching;
use webdriver::command::{
    ActionsParameters, AddCookieParameters, GetParameters, JavascriptCommandParameters,
    LocatorParameters, NewSessionParameters, NewWindowParameters, SendKeysParameters,
    SwitchToFrameParameters, SwitchToWindowParameters, TimeoutsParameters, WebDriverCommand,
    WebDriverExtensionCommand, WebDriverMessage, WindowRectParameters,
};
use webdriver::common::{
    Cookie, Date, LocatorStrategy, Parameters, ShadowRoot, WebElement, WebFrame, WebWindow,
};
use webdriver::error::{ErrorStatus, WebDriverError, WebDriverResult};
use webdriver::httpapi::WebDriverExtensionRoute;
use webdriver::response::{
    CloseWindowResponse, CookieResponse, CookiesResponse, ElementRectResponse, NewSessionResponse,
    NewWindowResponse, TimeoutsResponse, ValueResponse, WebDriverResponse, WindowRectResponse,
};
use webdriver::server::{self, Session, SessionTeardownKind, WebDriverHandler};

use crate::actions::{InputSourceState, PointerInputState};
use crate::session::{PageLoadStrategy, WebDriverSession};
use crate::timeout::{DEFAULT_PAGE_LOAD_TIMEOUT, SCREENSHOT_TIMEOUT};

fn extension_routes() -> Vec<(Method, &'static str, ServoExtensionRoute)> {
    vec![
        (
            Method::GET,
            "/session/{sessionId}/servo/prefs/get",
            ServoExtensionRoute::GetPrefs,
        ),
        (
            Method::POST,
            "/session/{sessionId}/servo/prefs/set",
            ServoExtensionRoute::SetPrefs,
        ),
        (
            Method::POST,
            "/session/{sessionId}/servo/prefs/reset",
            ServoExtensionRoute::ResetPrefs,
        ),
        (
            Method::DELETE,
            "/session/{sessionId}/servo/shutdown",
            ServoExtensionRoute::Shutdown,
        ),
        // <https://html.spec.whatwg.org/multipage/#set-rph-registration-mode>
        (
            Method::POST,
            "/session/{sessionId}/custom-handlers/set-mode",
            ServoExtensionRoute::CustomHandlersSetMode,
        ),
        (
            Method::POST,
            "/session/{sessionId}/servo/cookies/reset",
            ServoExtensionRoute::ResetAllCookies,
        ),
    ]
}

fn cookie_msg_to_cookie(cookie: cookie::Cookie) -> Cookie {
    Cookie {
        name: cookie.name().to_owned(),
        value: cookie.value().to_owned(),
        path: cookie.path().map(|s| s.to_owned()),
        domain: cookie.domain().map(|s| s.to_owned()),
        expiry: cookie.expires().and_then(|expiration| match expiration {
            Expiration::DateTime(date_time) => Some(Date(date_time.unix_timestamp() as u64)),
            Expiration::Session => None,
        }),
        secure: cookie.secure().unwrap_or(false),
        http_only: cookie.http_only().unwrap_or(false),
        same_site: cookie.same_site().map(|s| s.to_string()),
    }
}

pub fn start_server(
    port: u16,
    embedder_sender: Sender<WebDriverCommandMsg>,
    event_loop_waker: Box<dyn EventLoopWaker>,
) {
    let handler = Handler::new(embedder_sender, event_loop_waker);

    thread::Builder::new()
        .name("WebDriverHttpServer".to_owned())
        .spawn(move || {
            let address = SocketAddrV4::new("0.0.0.0".parse().unwrap(), port);
            match server::start(
                SocketAddr::V4(address),
                vec![],
                vec![],
                handler,
                extension_routes(),
            ) {
                Ok(listening) => info!("WebDriver server listening on {}", listening.socket),
                Err(e) => panic!("Unable to start WebDriver HTTP server {e:?}"),
            }
        })
        .expect("Thread spawning failed");
}

struct Handler {
    /// The threaded receiver on which we can block for a load-status.
    /// It will receive messages sent on the load_status_sender,
    /// and forwarded by the IPC router.
    load_status_receiver: RoutedReceiver<WebDriverLoadStatus>,
    /// The IPC sender which we can clone and pass along to the constellation,
    /// for it to send us a load-status. Messages sent on it
    /// will be forwarded to the load_status_receiver.
    load_status_sender: GenericSender<WebDriverLoadStatus>,

    session: Option<WebDriverSession>,

    /// A [`Sender`] that sends messages to the embedder that this `WebDriver instance controls.
    /// In addition to sending a message, we must always wake up the embedder's event loop so it
    /// knows that more messages are available for processing.
    embedder_sender: Sender<WebDriverCommandMsg>,

    /// An [`EventLoopWaker`] which is used to wake up the embedder event loop.
    event_loop_waker: Box<dyn EventLoopWaker>,

    /// A list of [`Receiver`]s that are used to track when input events are handled in the DOM.
    /// Once these receivers receive a response, we know that the event has been handled.
    ///
    /// TODO: Once we upgrade crossbeam-channel this can be replaced with a `WaitGroup`.
    pending_input_event_receivers: RefCell<Vec<Receiver<()>>>,

    /// Number of pending actions of which WebDriver is waiting for responses.
    num_pending_actions: Cell<u32>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum ServoExtensionRoute {
    GetPrefs,
    SetPrefs,
    ResetPrefs,
    /// TODO: Shutdown does not actually use sessionId.
    /// But the webdriver crate always checks existence of sessionID
    /// except for WebDriverCommand::Status.
    /// We have to either use our own fork, or relies on the current workaround:
    /// passing any dummy sessionID.
    Shutdown,
    CustomHandlersSetMode,
    ResetAllCookies,
}

impl WebDriverExtensionRoute for ServoExtensionRoute {
    type Command = ServoExtensionCommand;

    fn command(
        &self,
        _parameters: &Parameters,
        body_data: &Value,
    ) -> WebDriverResult<WebDriverCommand<ServoExtensionCommand>> {
        let command = match *self {
            ServoExtensionRoute::GetPrefs => {
                let parameters: GetPrefsParameters = serde_json::from_value(body_data.clone())?;
                ServoExtensionCommand::GetPrefs(parameters)
            },
            ServoExtensionRoute::SetPrefs => {
                let parameters: SetPrefsParameters = serde_json::from_value(body_data.clone())?;
                ServoExtensionCommand::SetPrefs(parameters)
            },
            ServoExtensionRoute::ResetPrefs => {
                let parameters: GetPrefsParameters = serde_json::from_value(body_data.clone())?;
                ServoExtensionCommand::ResetPrefs(parameters)
            },
            ServoExtensionRoute::CustomHandlersSetMode => {
                let parameters: CustomHandlersSetModeParameters =
                    serde_json::from_value(body_data.clone())?;
                ServoExtensionCommand::CustomHandlersSetMode(parameters)
            },
            ServoExtensionRoute::Shutdown => ServoExtensionCommand::Shutdown,
            ServoExtensionRoute::ResetAllCookies => ServoExtensionCommand::ResetAllCookies,
        };
        Ok(WebDriverCommand::Extension(command))
    }
}

#[derive(Clone, Debug)]
enum ServoExtensionCommand {
    GetPrefs(GetPrefsParameters),
    SetPrefs(SetPrefsParameters),
    ResetPrefs(GetPrefsParameters),
    CustomHandlersSetMode(CustomHandlersSetModeParameters),
    Shutdown,
    ResetAllCookies,
}

impl WebDriverExtensionCommand for ServoExtensionCommand {
    fn parameters_json(&self) -> Option<Value> {
        match *self {
            ServoExtensionCommand::GetPrefs(ref x) => serde_json::to_value(x).ok(),
            ServoExtensionCommand::SetPrefs(ref x) => serde_json::to_value(x).ok(),
            ServoExtensionCommand::ResetPrefs(ref x) => serde_json::to_value(x).ok(),
            ServoExtensionCommand::CustomHandlersSetMode(ref x) => serde_json::to_value(x).ok(),
            ServoExtensionCommand::Shutdown | ServoExtensionCommand::ResetAllCookies => None,
        }
    }
}

#[derive(Clone)]
struct SendableJSValue(JSValue);

impl Serialize for SendableJSValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self.0 {
            JSValue::Undefined | JSValue::Null => serializer.serialize_unit(),
            JSValue::Boolean(x) => serializer.serialize_bool(x),
            JSValue::Number(x) => {
                if x.fract() == 0.0 {
                    serializer.serialize_i64(x as i64)
                } else {
                    serializer.serialize_f64(x)
                }
            },
            JSValue::String(ref x) => serializer.serialize_str(x),
            JSValue::Element(ref x) => WebElement(x.clone()).serialize(serializer),
            JSValue::ShadowRoot(ref x) => ShadowRoot(x.clone()).serialize(serializer),
            JSValue::Frame(ref x) => WebFrame(x.clone()).serialize(serializer),
            JSValue::Window(ref x) => WebWindow(x.clone()).serialize(serializer),
            JSValue::Array(ref x) => x
                .iter()
                .map(|element| SendableJSValue(element.clone()))
                .collect::<Vec<SendableJSValue>>()
                .serialize(serializer),
            JSValue::Object(ref x) => x
                .iter()
                .map(|(k, v)| (k.clone(), SendableJSValue(v.clone())))
                .collect::<HashMap<String, SendableJSValue>>()
                .serialize(serializer),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct WebDriverPrefValue(PrefValue);

impl Serialize for WebDriverPrefValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self.0 {
            PrefValue::Bool(b) => serializer.serialize_bool(b),
            PrefValue::Str(ref s) => serializer.serialize_str(s),
            PrefValue::Float(f) => serializer.serialize_f64(f),
            PrefValue::Int(i) => serializer.serialize_i64(i),
            PrefValue::Array(ref v) => v
                .iter()
                .map(|value| WebDriverPrefValue(value.clone()))
                .collect::<Vec<WebDriverPrefValue>>()
                .serialize(serializer),
            PrefValue::UInt(u) => serializer.serialize_u64(u),
        }
    }
}

impl<'de> Deserialize<'de> for WebDriverPrefValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct Visitor;

        impl ::serde::de::Visitor<'_> for Visitor {
            type Value = WebDriverPrefValue;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("preference value")
            }

            fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
            where
                E: ::serde::de::Error,
            {
                Ok(WebDriverPrefValue(PrefValue::Float(value)))
            }

            fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
            where
                E: ::serde::de::Error,
            {
                Ok(WebDriverPrefValue(PrefValue::Int(value)))
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: ::serde::de::Error,
            {
                Ok(WebDriverPrefValue(PrefValue::Int(value as i64)))
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: ::serde::de::Error,
            {
                Ok(WebDriverPrefValue(PrefValue::Str(String::from(value))))
            }

            fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
            where
                E: ::serde::de::Error,
            {
                Ok(WebDriverPrefValue(PrefValue::Bool(value)))
            }
        }

        deserializer.deserialize_any(Visitor)
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
struct GetPrefsParameters {
    prefs: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
struct SetPrefsParameters {
    #[serde(deserialize_with = "map_to_vec")]
    prefs: Vec<(String, WebDriverPrefValue)>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
struct CustomHandlersSetModeParameters {
    mode: String,
}

fn map_to_vec<'de, D>(de: D) -> Result<Vec<(String, WebDriverPrefValue)>, D::Error>
where
    D: Deserializer<'de>,
{
    de.deserialize_map(TupleVecMapVisitor)
}

struct TupleVecMapVisitor;

impl<'de> Visitor<'de> for TupleVecMapVisitor {
    type Value = Vec<(String, WebDriverPrefValue)>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a map")
    }

    #[inline]
    fn visit_unit<E>(self) -> Result<Self::Value, E> {
        Ok(Vec::new())
    }

    #[inline]
    fn visit_map<T>(self, mut access: T) -> Result<Self::Value, T::Error>
    where
        T: MapAccess<'de>,
    {
        let mut values = Vec::new();

        while let Some((key, value)) = access.next_entry()? {
            values.push((key, value));
        }

        Ok(values)
    }
}

enum VerifyBrowsingContextIsOpen {
    Yes,
    No,
}

enum ImplicitWait {
    Return,
    #[expect(dead_code, reason = "This will be used in the next patch")]
    Continue,
}

impl From<ImplicitWait> for bool {
    fn from(implicit_wait: ImplicitWait) -> Self {
        match implicit_wait {
            ImplicitWait::Return => true,
            ImplicitWait::Continue => false,
        }
    }
}

impl Handler {
    fn new(
        embedder_sender: Sender<WebDriverCommandMsg>,
        event_loop_waker: Box<dyn EventLoopWaker>,
    ) -> Handler {
        // Create a pair of both an IPC and a threaded channel,
        // keep the IPC sender to clone and pass to the constellation for each load,
        // and keep a threaded receiver to block on an incoming load-status.
        // Pass the others to the IPC router so that IPC messages are forwarded to the threaded receiver.
        // We need to use the router because IPC does not come with a timeout on receive/select.
        let (load_status_sender, receiver) = generic_channel::channel().unwrap();
        let load_status_receiver = receiver.route_preserving_errors();

        Handler {
            load_status_sender,
            load_status_receiver,
            session: None,
            embedder_sender,
            event_loop_waker,
            pending_input_event_receivers: Default::default(),
            num_pending_actions: Cell::new(0),
        }
    }

    fn browsing_context_id(&self) -> WebDriverResult<BrowsingContextId> {
        self.session()?
            .current_browsing_context_id()
            .ok_or_else(|| {
                WebDriverError::new(ErrorStatus::UnknownError, "No browsing context available")
            })
    }

    fn webview_id(&self) -> WebDriverResult<WebViewId> {
        self.session()?
            .current_webview_id()
            .ok_or_else(|| WebDriverError::new(ErrorStatus::UnknownError, "No webview available"))
    }

    fn send_input_event_to_embedder(&self, input_event: InputEvent) {
        let _ = self.send_message_to_embedder(WebDriverCommandMsg::InputEvent(
            self.verified_webview_id(),
            input_event,
            None,
        ));
    }

    fn send_blocking_input_event_to_embedder(&self, input_event: InputEvent) {
        let (result_sender, result_receiver) = unbounded();
        if self
            .send_message_to_embedder(WebDriverCommandMsg::InputEvent(
                self.verified_webview_id(),
                input_event,
                Some(result_sender),
            ))
            .is_ok()
        {
            self.pending_input_event_receivers
                .borrow_mut()
                .push(result_receiver);
        }
    }

    fn send_message_to_embedder(&self, msg: WebDriverCommandMsg) -> WebDriverResult<()> {
        self.embedder_sender.send(msg).map_err(|_| {
            WebDriverError::new(
                ErrorStatus::UnknownError,
                "Failed to send message to embedder",
            )
        })?;
        self.event_loop_waker.wake();
        Ok(())
    }

    fn add_load_status_sender(&self) -> WebDriverResult<()> {
        self.send_message_to_embedder(WebDriverCommandMsg::ScriptCommand(
            self.browsing_context_id()?,
            WebDriverScriptCommand::AddLoadStatusSender(
                self.webview_id()?,
                self.load_status_sender.clone(),
            ),
        ))
    }

    fn clear_load_status_sender(&self) -> WebDriverResult<()> {
        self.send_message_to_embedder(WebDriverCommandMsg::ScriptCommand(
            self.browsing_context_id()?,
            WebDriverScriptCommand::RemoveLoadStatusSender(self.webview_id()?),
        ))
    }

    // This function is called only if session and webview are verified.
    fn verified_webview_id(&self) -> WebViewId {
        self.session().unwrap().current_webview_id().unwrap()
    }

    fn focused_webview_id(&self) -> WebDriverResult<Option<WebViewId>> {
        let (sender, receiver) = generic_channel::oneshot().unwrap();
        self.send_message_to_embedder(WebDriverCommandMsg::GetFocusedWebView(sender))?;
        // Wait until the document is ready before returning the top-level browsing context id.
        wait_for_oneshot_response(receiver)
    }

    fn session(&self) -> WebDriverResult<&WebDriverSession> {
        match self.session {
            Some(ref x) => Ok(x),
            // https://w3c.github.io/webdriver/#ref-for-dfn-invalid-session-id-1
            None => Err(WebDriverError::new(
                ErrorStatus::InvalidSessionId,
                "Session not created",
            )),
        }
    }

    fn session_mut(&mut self) -> WebDriverResult<&mut WebDriverSession> {
        match self.session {
            Some(ref mut x) => Ok(x),
            // https://w3c.github.io/webdriver/#ref-for-dfn-invalid-session-id-1
            None => Err(WebDriverError::new(
                ErrorStatus::InvalidSessionId,
                "Session not created",
            )),
        }
    }

    /// <https://w3c.github.io/webdriver/#new-session>
    fn handle_new_session(
        &mut self,
        parameters: &NewSessionParameters,
    ) -> WebDriverResult<WebDriverResponse> {
        if let Ok(value) = env::var("DELAY_AFTER_ACCEPT") {
            let seconds = value.parse::<u64>().unwrap_or_default();
            println!("Waiting for {} seconds...", seconds);
            println!("lldb -p {}", process::id());
            thread::sleep(Duration::from_secs(seconds));
        }

        // Step 1. If the list of active HTTP sessions is not empty
        // return error with error code session not created.
        if self.session.is_some() {
            return Err(WebDriverError::new(
                ErrorStatus::SessionNotCreated,
                "Session already created",
            ));
        }

        // Step 2. Skip because the step is only applied to an intermediary node.
        // Step 3. Skip since all sessions are http for now.

        // Step 4. Let capabilities be the result of trying to process capabilities
        let mut servo_capabilities = ServoCapabilities::new();
        let processed_capabilities = parameters.match_browser(&mut servo_capabilities)?;

        // Step 5. If capabilities's is null, return error with error code session not created.
        let mut capabilities = match processed_capabilities {
            Some(capabilities) => capabilities,
            None => {
                return Err(WebDriverError::new(
                    ErrorStatus::SessionNotCreated,
                    "Session not created due to invalid capabilities",
                ));
            },
        };

        // Step 6. Create a session
        let session_id = self.create_session(&mut capabilities, &servo_capabilities)?;

        // Step 7. Let response be a JSON Object initialized with session's session ID and capabilities
        let response = NewSessionResponse::new(session_id.to_string(), Value::Object(capabilities));

        // Step 8. Set session' current top-level browsing context
        match self.focused_webview_id()? {
            Some(webview_id) => {
                self.session_mut()?.set_webview_id(webview_id);
                self.session_mut()?
                    .set_browsing_context_id(BrowsingContextId::from(webview_id));
            },
            None => {
                // This happens when there is no open webview.
                // We need to create a new one. See https://github.com/servo/servo/issues/37408
                let (sender, receiver) = generic_channel::oneshot().unwrap();

                self.send_message_to_embedder(WebDriverCommandMsg::NewWebView(
                    sender,
                    Some(self.load_status_sender.clone()),
                ))?;
                let webview_id = receiver
                    .recv()
                    .expect("IPC failure when creating new webview for new session");
                self.focus_webview(webview_id)?;
                self.session_mut()?.set_webview_id(webview_id);
                self.session_mut()?
                    .set_browsing_context_id(BrowsingContextId::from(webview_id));
                let _ = self.wait_document_ready(Some(3000));
            },
        };

        // Step 9. Set the request queue to a new queue.
        // Skip here because the requests are handled in the external crate.

        // Step 10. Return success with data body
        Ok(WebDriverResponse::NewSession(response))
    }

    /// <https://w3c.github.io/webdriver/#dfn-delete-session>
    fn handle_delete_session(&mut self) -> WebDriverResult<WebDriverResponse> {
        // Step 1. If session is http, close the session
        self.session = None;

        // Step 2. Return success with data null
        Ok(WebDriverResponse::DeleteSession)
    }

    /// <https://w3c.github.io/webdriver/#status>
    fn handle_status(&self) -> WebDriverResult<WebDriverResponse> {
        Ok(WebDriverResponse::Generic(ValueResponse(
            if self.session.is_none() {
                json!({ "ready": true, "message": "Ready for a new session" })
            } else {
                json!({ "ready": false, "message": "Not ready for a new session" })
            },
        )))
    }

    /// Send command to Script Thread with session's current browsing context.
    /// If `verify` is [`VerifyBrowsingContextIsOpen::Yes`],
    /// it would verify the existence of browsing context before sending.
    fn browsing_context_script_command(
        &self,
        cmd_msg: WebDriverScriptCommand,
        verify: VerifyBrowsingContextIsOpen,
    ) -> WebDriverResult<()> {
        let browsing_context_id = self.browsing_context_id()?;
        if let VerifyBrowsingContextIsOpen::Yes = verify {
            self.verify_browsing_context_is_open(browsing_context_id)?;
        }
        self.send_message_to_embedder(WebDriverCommandMsg::ScriptCommand(
            browsing_context_id,
            cmd_msg,
        ))?;
        Ok(())
    }

    /// Send command to Script Thread with session's current top-level browsing context.
    /// If `verify` is [`VerifyBrowsingContextIsOpen::Yes`],
    /// it would verify the existence of top-level browsing context before sending.
    fn top_level_script_command(
        &self,
        cmd_msg: WebDriverScriptCommand,
        verify: VerifyBrowsingContextIsOpen,
    ) -> WebDriverResult<()> {
        let webview_id = self.webview_id()?;
        if let VerifyBrowsingContextIsOpen::Yes = verify {
            self.verify_top_level_browsing_context_is_open(webview_id)?;
        }
        let browsing_context_id = BrowsingContextId::from(webview_id);
        self.send_message_to_embedder(WebDriverCommandMsg::ScriptCommand(
            browsing_context_id,
            cmd_msg,
        ))?;
        Ok(())
    }

    /// <https://w3c.github.io/webdriver/#navigate-to>
    fn handle_get(&mut self, parameters: &GetParameters) -> WebDriverResult<WebDriverResponse> {
        let webview_id = self.webview_id()?;
        // Step 2. If session's current top-level browsing context is no longer open,
        // return error with error code no such window.
        self.verify_top_level_browsing_context_is_open(webview_id)?;
        // Step 3. If URL is not an absolute URL or is not an absolute URL with fragment
        // or not a local scheme, return error with error code invalid argument.
        let url = ServoUrl::parse(&parameters.url)
            .map(|url| url.into_url())
            .map_err(|_| WebDriverError::new(ErrorStatus::InvalidArgument, "Invalid URL"))?;

        // Step 4. Handle any user prompt.
        self.handle_any_user_prompts(webview_id)?;

        let cmd_msg =
            WebDriverCommandMsg::LoadUrl(webview_id, url, self.load_status_sender.clone());
        self.send_message_to_embedder(cmd_msg)?;

        // Step 8.2.1: try to wait for navigation to complete.
        self.wait_for_navigation_complete()?;

        // Step 8.3. Set current browsing context with session and current top browsing context
        self.session_mut()?
            .set_browsing_context_id(BrowsingContextId::from(webview_id));

        Ok(WebDriverResponse::Void)
    }

    fn wait_document_ready(&self, timeout: Option<u64>) -> WebDriverResult<WebDriverResponse> {
        let timeout_channel = match timeout {
            Some(timeout) => after(Duration::from_millis(timeout)),
            None => crossbeam_channel::never(),
        };

        select! {
            recv(self.load_status_receiver) -> res => {
                match res {
                    // If the navigation is navigation to IFrame, no document state event is fired.
                    Ok(Ok(WebDriverLoadStatus::Blocked)) => {
                        // TODO: evaluate the correctness later
                        // Load status is block means an user prompt is shown.
                        // Alot of tests expect this to return success
                        // then the user prompt is handled in the next command.
                        // If user prompt can't be handler, next command returns
                        // an error anyway.
                        Ok(WebDriverResponse::Void)
                    },
                    Ok(Ok(WebDriverLoadStatus::Complete)) |
                    Ok(Ok(WebDriverLoadStatus::NavigationStop)) =>
                        Ok(WebDriverResponse::Void)
                    ,
                    _ => Err(WebDriverError::new(
                        ErrorStatus::UnknownError,
                        "Unexpected load status received while waiting for document ready state",
                    )),
                }
            },
            recv(timeout_channel) -> _ => Err(
                WebDriverError::new(ErrorStatus::Timeout, "Load timed out")
            ),
        }
    }

    /// <https://w3c.github.io/webdriver/#dfn-wait-for-navigation-to-complete>
    fn wait_for_navigation_complete(&self) -> WebDriverResult<WebDriverResponse> {
        debug!("waiting for load");

        let session = self.session()?;

        // Step 1. If session's page loading strategy is "none",
        // return success with data null.
        if session.page_loading_strategy() == PageLoadStrategy::None {
            return Ok(WebDriverResponse::Void);
        }

        // Step 2. If session's current browsing context is no longer open,
        // return success with data null.
        if self
            .verify_browsing_context_is_open(self.browsing_context_id()?)
            .is_err()
        {
            return Ok(WebDriverResponse::Void);
        }

        // Step 3. let timeout be the session's page load timeout.
        let timeout = session.session_timeouts().page_load;

        // TODO: Step 4. Implement timer parameter

        let result = self.wait_document_ready(timeout);
        debug!("finished waiting for load with {:?}", result);
        result
    }

    /// <https://w3c.github.io/webdriver/#dfn-wait-for-navigation-to-complete>
    fn wait_for_navigation(&self) -> WebDriverResult<WebDriverResponse> {
        let navigation_status = match self.load_status_receiver.try_recv() {
            Ok(Ok(status)) => status,
            // Empty channel means no navigation started. Nothing to wait for.
            Err(crossbeam_channel::TryRecvError::Empty) => {
                return Ok(WebDriverResponse::Void);
            },
            Err(crossbeam_channel::TryRecvError::Disconnected) => {
                return Err(WebDriverError::new(
                    ErrorStatus::UnknownError,
                    "Load status channel disconnected",
                ));
            },
            Ok(Err(ipc_error)) => {
                return Err(WebDriverError::new(
                    ErrorStatus::UnknownError,
                    format!("Load status channel ipc error: {ipc_error}"),
                ));
            },
        };

        match navigation_status {
            WebDriverLoadStatus::NavigationStart => self.wait_for_navigation_complete(),
            // If the load status is timeout, return an error
            WebDriverLoadStatus::Timeout => Err(WebDriverError::new(
                ErrorStatus::Timeout,
                "Navigation timed out",
            )),
            // If the load status is blocked, it means a user prompt is shown.
            // We should handle the user prompt in the next command.
            WebDriverLoadStatus::Blocked => Ok(WebDriverResponse::Void),
            WebDriverLoadStatus::NavigationStop | WebDriverLoadStatus::Complete => {
                unreachable!("Unexpected load status received")
            },
        }
    }

    /// <https://w3c.github.io/webdriver/#dfn-get-current-url>
    fn handle_current_url(&self) -> WebDriverResult<WebDriverResponse> {
        let webview_id = self.webview_id()?;

        // Step 1. If session's current top-level browsing context is no longer open,
        // return error with error code no such window.
        self.verify_top_level_browsing_context_is_open(webview_id)?;

        // Step 2. Handle any user prompt.
        self.handle_any_user_prompts(webview_id)?;

        let (sender, receiver) = ipc::channel().unwrap();
        self.top_level_script_command(
            WebDriverScriptCommand::GetUrl(sender),
            VerifyBrowsingContextIsOpen::No,
        )?;

        let url = wait_for_ipc_response(receiver)?;

        Ok(WebDriverResponse::Generic(ValueResponse(
            serde_json::to_value(url)?,
        )))
    }

    /// <https://w3c.github.io/webdriver/#get-window-rect>
    fn handle_window_rect(
        &self,
        verify: VerifyBrowsingContextIsOpen,
    ) -> WebDriverResult<WebDriverResponse> {
        let (sender, receiver) = generic_channel::oneshot().unwrap();
        let webview_id = self.webview_id()?;
        // Step 1. If session's current top-level browsing context is no longer open,
        // return error with error code no such window.
        if let VerifyBrowsingContextIsOpen::Yes = verify {
            self.verify_top_level_browsing_context_is_open(webview_id)?;
        }

        // Step 2. Handle any user prompt.
        self.handle_any_user_prompts(webview_id)?;

        self.send_message_to_embedder(WebDriverCommandMsg::GetWindowRect(webview_id, sender))?;

        let window_rect = wait_for_oneshot_response(receiver)?;
        let window_size_response = WindowRectResponse {
            x: window_rect.min.x,
            y: window_rect.min.y,
            width: window_rect.width(),
            height: window_rect.height(),
        };
        Ok(WebDriverResponse::WindowRect(window_size_response))
    }

    /// <https://w3c.github.io/webdriver/#set-window-rect>
    fn handle_set_window_rect(
        &self,
        params: &WindowRectParameters,
    ) -> WebDriverResult<WebDriverResponse> {
        // Step 9 - 10. Input Validation. Already done when deserialize.

        // Step 11. In case the Set Window Rect command is partially supported
        // (i.e. some combinations of arguments are supported but not others),
        // the implmentation is expected to continue with the remaining steps.
        // DO NOT return "unsupported operation".

        let webview_id = self.webview_id()?;
        // Step 12. If session's current top-level browsing context is no longer open,
        // return error with error code no such window.
        self.verify_top_level_browsing_context_is_open(webview_id)?;

        // Step 13. Handle any user prompt.
        self.handle_any_user_prompts(webview_id)?;

        // (TODO) Step 14. Fully exit fullscreen.
        // (TODO) Step 15. Restore the window.

        let current = LazyCell::new(|| {
            let WebDriverResponse::WindowRect(current) = self
                .handle_window_rect(VerifyBrowsingContextIsOpen::No)
                .unwrap()
            else {
                unreachable!("handle_window_size() must return WindowRect");
            };
            current
        });

        let (x, y, width, height) = (
            params.x.unwrap_or_else(|| current.x),
            params.y.unwrap_or_else(|| current.y),
            params.width.unwrap_or_else(|| current.width),
            params.height.unwrap_or_else(|| current.height),
        );
        let (sender, receiver) = generic_channel::oneshot().unwrap();
        // Step 16 - 17. Set the width/height in CSS pixels.
        // This should be done as long as one of width/height is not null.

        // Step 18 - 19. Set the screen x/y in CSS pixels.
        // This should be done as long as one of width/height is not null.
        self.send_message_to_embedder(WebDriverCommandMsg::SetWindowRect(
            webview_id,
            DeviceIndependentIntRect::from_origin_and_size(
                Point2D::new(x, y),
                Size2D::new(width, height),
            ),
            sender,
        ))?;

        let window_rect = wait_for_oneshot_response(receiver)?;
        debug!("Result window_rect: {window_rect:?}");
        let window_size_response = WindowRectResponse {
            x: window_rect.min.x,
            y: window_rect.min.y,
            width: window_rect.width(),
            height: window_rect.height(),
        };
        Ok(WebDriverResponse::WindowRect(window_size_response))
    }

    /// <https://w3c.github.io/webdriver/#maximize-window>
    fn handle_maximize_window(&mut self) -> WebDriverResult<WebDriverResponse> {
        // Step 1. If the remote end does not support the Maximize Window command for session's
        // current top-level browsing context for any reason,
        // return error with error code unsupported operation.
        let webview_id = self.webview_id()?;
        // Step 2. If session's current top-level browsing context is no longer open,
        // return error with error code no such window.
        self.verify_top_level_browsing_context_is_open(webview_id)?;

        // Step 3. Try to handle any user prompts with session.
        self.handle_any_user_prompts(self.webview_id()?)?;

        // Step 4. (TODO) Fully exit fullscreen.

        // Step 5. (TODO) Restore the window.

        // Step 6. Maximize the window of session's current top-level browsing context.
        let (sender, receiver) = generic_channel::oneshot().unwrap();
        self.send_message_to_embedder(WebDriverCommandMsg::MaximizeWebView(webview_id, sender))?;

        let window_rect = wait_for_oneshot_response(receiver)?;
        debug!("Result window_rect: {window_rect:?}");
        let window_size_response = WindowRectResponse {
            x: window_rect.min.x,
            y: window_rect.min.y,
            width: window_rect.width(),
            height: window_rect.height(),
        };
        Ok(WebDriverResponse::WindowRect(window_size_response))
    }

    fn handle_is_enabled(&self, element: &WebElement) -> WebDriverResult<WebDriverResponse> {
        // Step 1. If session's current browsing context is no longer open,
        // return error with error code no such window.
        let browsing_context = self.browsing_context_id()?;
        self.verify_browsing_context_is_open(browsing_context)?;

        // Step 2. Try to handle any user prompts with session.
        let webview_id = self.webview_id()?;
        self.handle_any_user_prompts(webview_id)?;

        let (sender, receiver) = ipc::channel().unwrap();
        self.browsing_context_script_command(
            WebDriverScriptCommand::IsEnabled(element.to_string(), sender),
            VerifyBrowsingContextIsOpen::No,
        )?;

        Ok(WebDriverResponse::Generic(ValueResponse(
            serde_json::to_value(wait_for_ipc_response_flatten(receiver)?)?,
        )))
    }

    fn handle_is_selected(&self, element: &WebElement) -> WebDriverResult<WebDriverResponse> {
        // Step 1. If session's current browsing context is no longer open,
        // return error with error code no such window.
        let browsing_context = self.browsing_context_id()?;
        self.verify_browsing_context_is_open(browsing_context)?;

        // Step 2. Try to handle any user prompts with session.
        let webview_id = self.webview_id()?;
        self.handle_any_user_prompts(webview_id)?;

        let (sender, receiver) = ipc::channel().unwrap();
        self.browsing_context_script_command(
            WebDriverScriptCommand::IsSelected(element.to_string(), sender),
            VerifyBrowsingContextIsOpen::No,
        )?;

        Ok(WebDriverResponse::Generic(ValueResponse(
            serde_json::to_value(wait_for_ipc_response_flatten(receiver)?)?,
        )))
    }

    /// <https://w3c.github.io/webdriver/#back>
    fn handle_go_back(&self) -> WebDriverResult<WebDriverResponse> {
        let webview_id = self.webview_id()?;
        // Step 1. If session's current top-level browsing context is no longer open,
        // return error with error code no such window.
        self.verify_top_level_browsing_context_is_open(webview_id)?;

        // Step 2. Handle any user prompt.
        self.handle_any_user_prompts(webview_id)?;

        self.send_message_to_embedder(WebDriverCommandMsg::GoBack(
            webview_id,
            self.load_status_sender.clone(),
        ))?;
        self.wait_for_navigation_complete()
    }

    /// <https://w3c.github.io/webdriver/#forward>
    fn handle_go_forward(&self) -> WebDriverResult<WebDriverResponse> {
        let webview_id = self.webview_id()?;
        // Step 1. If session's current top-level browsing context is no longer open,
        // return error with error code no such window.
        self.verify_top_level_browsing_context_is_open(webview_id)?;

        // Step 2. Handle any user prompt.
        self.handle_any_user_prompts(webview_id)?;

        self.send_message_to_embedder(WebDriverCommandMsg::GoForward(
            webview_id,
            self.load_status_sender.clone(),
        ))?;
        self.wait_for_navigation_complete()
    }

    /// <https://w3c.github.io/webdriver/#refresh>
    fn handle_refresh(&mut self) -> WebDriverResult<WebDriverResponse> {
        let webview_id = self.webview_id()?;
        // Step 1. If session's current top-level browsing context is no longer open,
        // return error with error code no such window.
        self.verify_top_level_browsing_context_is_open(webview_id)?;

        // Step 2. Handle any user prompt.
        self.handle_any_user_prompts(webview_id)?;

        let cmd_msg = WebDriverCommandMsg::Refresh(webview_id, self.load_status_sender.clone());
        self.send_message_to_embedder(cmd_msg)?;

        // Step 4.1: Try to wait for navigation to complete.
        self.wait_for_navigation_complete()?;

        // Step 5. Set current browsing context with session and current top browsing context.
        self.session_mut()?
            .set_browsing_context_id(BrowsingContextId::from(webview_id));

        Ok(WebDriverResponse::Void)
    }

    /// <https://w3c.github.io/webdriver/#get-title>
    fn handle_title(&self) -> WebDriverResult<WebDriverResponse> {
        let webview_id = self.webview_id()?;

        // Step 1. If session's current top-level browsing context is no longer open,
        // return error with error code no such window.
        self.verify_top_level_browsing_context_is_open(webview_id)?;
        // Step 2. Handle any user prompt.
        self.handle_any_user_prompts(webview_id)?;

        let (sender, receiver) = ipc::channel().unwrap();

        self.top_level_script_command(
            WebDriverScriptCommand::GetTitle(sender),
            VerifyBrowsingContextIsOpen::No,
        )?;

        // Step 3. Let title be the session's current top-level
        // browsing context's active document's title.
        let title = wait_for_ipc_response(receiver)?;
        Ok(WebDriverResponse::Generic(ValueResponse(
            serde_json::to_value(title)?,
        )))
    }

    /// <https://w3c.github.io/webdriver/#get-window-handle>
    fn handle_window_handle(&mut self) -> WebDriverResult<WebDriverResponse> {
        let webview_id = self.webview_id()?;

        // Step 1. If session's current top-level browsing context is no longer open,
        // return error with error code no such window.
        self.verify_top_level_browsing_context_is_open(webview_id)?;

        // Step 2. Return success with the window handle.
        let handle = self
            .get_window_handle(webview_id)
            .expect("Failed to get window handle of an existing webview");

        Ok(WebDriverResponse::Generic(ValueResponse(
            serde_json::to_value(handle)?,
        )))
    }

    /// <https://w3c.github.io/webdriver/#get-window-handles>
    fn handle_window_handles(&mut self) -> WebDriverResult<WebDriverResponse> {
        let mut handles = self.get_window_handles();
        handles.sort_unstable();

        Ok(WebDriverResponse::Generic(ValueResponse(
            serde_json::to_value(handles)?,
        )))
    }

    fn get_window_handle(&mut self, webview_id: WebViewId) -> Option<String> {
        self.get_window_handles()
            .iter()
            .find(|id| id == &&webview_id.to_string())
            .cloned()
    }

    fn get_window_handles(&self) -> Vec<String> {
        self.get_all_webview_ids()
            .into_iter()
            .map(|id| id.to_string())
            .collect()
    }

    fn get_all_webview_ids(&self) -> Vec<WebViewId> {
        let (sender, receiver) = generic_channel::oneshot().unwrap();
        self.send_message_to_embedder(WebDriverCommandMsg::GetAllWebViews(sender))
            .unwrap();
        wait_for_oneshot_response(receiver).unwrap_or_default()
    }

    /// <https://w3c.github.io/webdriver/#close-window>
    fn handle_close_window(&mut self) -> WebDriverResult<WebDriverResponse> {
        let webview_id = self.webview_id()?;
        // Step 1. If session's current top-level browsing context is no longer open,
        // return error with error code no such window.
        self.verify_top_level_browsing_context_is_open(webview_id)?;

        // Step 2. Handle any user prompt.
        self.handle_any_user_prompts(webview_id)?;

        // Step 3. Close session's current top-level browsing context.
        let (sender, receiver) = generic_channel::oneshot().unwrap();

        let cmd_msg = WebDriverCommandMsg::CloseWebView(webview_id, sender);
        self.send_message_to_embedder(cmd_msg)?;

        wait_for_oneshot_response(receiver)?;

        // Step 4. If there are no more open top-level browsing contexts, try to close the session.
        let window_handles = self.get_window_handles();

        if window_handles.is_empty() {
            self.session = None;
        }

        // Step 5. Return the result of running the remote end steps for the Get Window Handles command
        Ok(WebDriverResponse::CloseWindow(CloseWindowResponse(
            window_handles,
        )))
    }

    /// <https://w3c.github.io/webdriver/#new-window>
    fn handle_new_window(
        &mut self,
        _parameters: &NewWindowParameters,
    ) -> WebDriverResult<WebDriverResponse> {
        let (sender, receiver) = generic_channel::oneshot().unwrap();

        let webview_id = self.webview_id()?;

        // Step 2. If session's current top-level browsing context is no longer open,
        // return error with error code no such window.
        self.verify_top_level_browsing_context_is_open(webview_id)?;

        // Step 3. Handle any user prompt.
        self.handle_any_user_prompts(webview_id)?;

        let cmd_msg =
            WebDriverCommandMsg::NewWebView(sender, Some(self.load_status_sender.clone()));
        // Step 5. Create a new top-level browsing context by running the window open steps.
        // This MUST be done without invoking the focusing steps.
        self.send_message_to_embedder(cmd_msg)?;

        if let Ok(webview_id) = receiver.recv() {
            let _ = self.wait_for_navigation_complete();
            let handle = self
                .get_window_handle(webview_id)
                .expect("Failed to get window handle of an existing webview");

            Ok(WebDriverResponse::NewWindow(NewWindowResponse {
                handle,
                typ: "tab".to_string(),
            }))
        } else {
            Err(WebDriverError::new(
                ErrorStatus::UnknownError,
                "No webview ID received",
            ))
        }
    }

    /// <https://w3c.github.io/webdriver/#dfn-switch-to-frame>
    fn handle_switch_to_frame(
        &mut self,
        parameters: &SwitchToFrameParameters,
    ) -> WebDriverResult<WebDriverResponse> {
        use webdriver::common::FrameId;
        let frame_id = match parameters.id {
            // id is null
            FrameId::Top => {
                let webview_id = self.webview_id()?;
                // Step 1. If session's current top-level browsing context is no longer open,
                // return error with error code no such window.
                self.verify_top_level_browsing_context_is_open(webview_id)?;
                // Step 2. Try to handle any user prompts with session.
                self.handle_any_user_prompts(webview_id)?;
                // Step 3. Set the current browsing context with session and
                // session's current top-level browsing context.
                let browsing_context_id = BrowsingContextId::from(webview_id);
                self.session_mut()?
                    .set_browsing_context_id(browsing_context_id);

                // Step 4. Update any implementation-specific state that would result from
                // the user selecting session's current browsing context for interaction,
                // without altering OS-level focus.
                self.focus_browsing_context(browsing_context_id)?;
                return Ok(WebDriverResponse::Void);
            },
            // id is a Number object
            FrameId::Short(ref x) => {
                // (Already handled when deserializing in webdriver-crate)
                // Step 1. If id is less than 0 or greater than 2^16 – 1,
                // return error with error code invalid argument.
                WebDriverFrameId::Short(*x)
            },
            FrameId::Element(ref x) => WebDriverFrameId::Element(x.to_string()),
        };

        self.switch_to_frame(frame_id)
    }

    /// <https://w3c.github.io/webdriver/#switch-to-parent-frame>
    fn handle_switch_to_parent_frame(&mut self) -> WebDriverResult<WebDriverResponse> {
        let webview_id = self.webview_id()?;
        let browsing_context = self.browsing_context_id()?;

        // Step 1. If session's current browsing context is already the top-level browsing context:
        if browsing_context == webview_id {
            // Step 1.1. If session's current browsing context is no longer open,
            // return error with error code no such window.
            self.verify_browsing_context_is_open(browsing_context)?;
            // Step 1.2. Return success with data null.
            return Ok(WebDriverResponse::Void);
        }

        // Step 2. If session's current parent browsing context is no longer open,
        // return error with error code no such window.
        let (sender, receiver) = ipc::channel().unwrap();
        let cmd = WebDriverScriptCommand::GetParentFrameId(sender);
        self.browsing_context_script_command(cmd, VerifyBrowsingContextIsOpen::Yes)?;

        // Step 3. Handle any user prompt.
        self.handle_any_user_prompts(webview_id)?;

        // Step 4. If session's current parent browsing context is not null,
        // set the current browsing context with session and current parent browsing context.
        let browsing_context_id = wait_for_ipc_response_flatten(receiver)?;
        self.session_mut()?
            .set_browsing_context_id(browsing_context_id);
        // Step 5. Update any implementation-specific state that would result from
        // the user selecting session's current browsing context for interaction,
        // without altering OS-level focus.
        self.focus_browsing_context(browsing_context_id)?;
        Ok(WebDriverResponse::Void)
    }

    /// <https://w3c.github.io/webdriver/#switch-to-window>
    fn handle_switch_to_window(
        &mut self,
        parameters: &SwitchToWindowParameters,
    ) -> WebDriverResult<WebDriverResponse> {
        let Some(webview_id) = self
            .get_all_webview_ids()
            .into_iter()
            .find(|id| id.to_string() == parameters.handle)
        else {
            return Err(WebDriverError::new(
                ErrorStatus::NoSuchWindow,
                "No such window while switching to window",
            ));
        };

        let session = self.session_mut()?;
        session.set_webview_id(webview_id);
        session.set_browsing_context_id(BrowsingContextId::from(webview_id));

        // Step 5. Update any implementation-specific state that would result
        // from the user selecting session's current browsing context for interaction,
        // without altering OS-level focus.
        self.focus_webview(webview_id)?;

        Ok(WebDriverResponse::Void)
    }

    fn switch_to_frame(
        &mut self,
        frame_id: WebDriverFrameId,
    ) -> WebDriverResult<WebDriverResponse> {
        let (sender, receiver) = ipc::channel().unwrap();
        let cmd = WebDriverScriptCommand::GetBrowsingContextId(frame_id, sender);
        self.browsing_context_script_command(cmd, VerifyBrowsingContextIsOpen::Yes)?;
        self.handle_any_user_prompts(self.webview_id()?)?;

        let browsing_context_id = wait_for_ipc_response_flatten(receiver)?;
        self.session_mut()?
            .set_browsing_context_id(browsing_context_id);
        // Step 4. Update any implementation-specific state that would result from
        // the user selecting session's current browsing context for interaction,
        // without altering OS-level focus.
        self.focus_browsing_context(browsing_context_id)?;
        Ok(WebDriverResponse::Void)
    }

    /// <https://w3c.github.io/webdriver/#find-element>
    fn handle_find_element(
        &self,
        parameters: &LocatorParameters,
    ) -> WebDriverResult<WebDriverResponse> {
        // Step 1 - 9.
        let res = self.handle_find_elements(parameters)?;
        // Step 10. If result is empty, return error with error code no such element.
        // Otherwise, return the first element of result.
        unwrap_first_element_response(res)
    }

    /// The boolean in callback result indicates whether implicit_wait can early return
    /// before timeout with current result.
    fn implicit_wait<T>(
        &self,
        callback: impl Fn() -> Result<(bool, T), (bool, WebDriverError)>,
    ) -> Result<T, WebDriverError> {
        let now = Instant::now();
        let (implicit_wait, sleep_interval) = {
            let timeouts = self.session()?.session_timeouts();
            (
                Duration::from_millis(timeouts.implicit_wait.unwrap_or(0)),
                Duration::from_millis(timeouts.sleep_interval),
            )
        };

        loop {
            match callback() {
                Ok((can_early_return, value)) => {
                    if can_early_return || now.elapsed() >= implicit_wait {
                        return Ok(value);
                    }
                },
                Err((can_early_return, error)) => {
                    if can_early_return || now.elapsed() >= implicit_wait {
                        return Err(error);
                    }
                },
            }
            sleep(sleep_interval);
        }
    }

    /// <https://w3c.github.io/webdriver/#find-elements>
    fn handle_find_elements(
        &self,
        parameters: &LocatorParameters,
    ) -> WebDriverResult<WebDriverResponse> {
        // Step 4. If selector is undefined, return error with error code invalid argument.
        if parameters.value.is_empty() {
            return Err(WebDriverError::new(ErrorStatus::InvalidArgument, ""));
        }
        // Step 5. If session's current browsing context is no longer open,
        // return error with error code no such window.
        self.verify_browsing_context_is_open(self.browsing_context_id()?)?;

        // Step 6. Handle any user prompt.
        self.handle_any_user_prompts(self.webview_id()?)?;

        self.implicit_wait(|| {
            let (sender, receiver) = ipc::channel().unwrap();
            let cmd = match parameters.using {
                LocatorStrategy::CSSSelector => WebDriverScriptCommand::FindElementsCSSSelector(
                    parameters.value.clone(),
                    sender,
                ),
                LocatorStrategy::LinkText | LocatorStrategy::PartialLinkText => {
                    WebDriverScriptCommand::FindElementsLinkText(
                        parameters.value.clone(),
                        parameters.using == LocatorStrategy::PartialLinkText,
                        sender,
                    )
                },
                LocatorStrategy::TagName => {
                    WebDriverScriptCommand::FindElementsTagName(parameters.value.clone(), sender)
                },
                LocatorStrategy::XPath => WebDriverScriptCommand::FindElementsXpathSelector(
                    parameters.value.clone(),
                    sender,
                ),
            };
            self.browsing_context_script_command(cmd, VerifyBrowsingContextIsOpen::No)
                .map_err(|error| (ImplicitWait::Return.into(), error))?;
            wait_for_ipc_response_flatten(receiver)
                .map(|value| (!value.is_empty(), value))
                .map_err(|error| (ImplicitWait::Return.into(), error))
        })
        .and_then(|response| {
            let resp_value: Vec<WebElement> = response.into_iter().map(WebElement).collect();
            Ok(WebDriverResponse::Generic(ValueResponse(
                serde_json::to_value(resp_value)?,
            )))
        })
    }

    /// <https://w3c.github.io/webdriver/#find-element-from-element>
    fn handle_find_element_from_element(
        &self,
        element: &WebElement,
        parameters: &LocatorParameters,
    ) -> WebDriverResult<WebDriverResponse> {
        // Step 1 - 8.
        let res = self.handle_find_elements_from_element(element, parameters)?;
        // Step 9. If result is empty, return error with error code no such element.
        // Otherwise, return the first element of result.
        unwrap_first_element_response(res)
    }

    /// <https://w3c.github.io/webdriver/#find-elements-from-element>
    fn handle_find_elements_from_element(
        &self,
        element: &WebElement,
        parameters: &LocatorParameters,
    ) -> WebDriverResult<WebDriverResponse> {
        // Step 4. If selector is undefined, return error with error code invalid argument.
        if parameters.value.is_empty() {
            return Err(WebDriverError::new(ErrorStatus::InvalidArgument, ""));
        }
        // Step 5. If session's current browsing context is no longer open,
        // return error with error code no such window.
        self.verify_browsing_context_is_open(self.browsing_context_id()?)?;

        // Step 6. Handle any user prompt.
        self.handle_any_user_prompts(self.webview_id()?)?;

        self.implicit_wait(|| {
            let (sender, receiver) = ipc::channel().unwrap();

            let cmd = match parameters.using {
                LocatorStrategy::CSSSelector => {
                    WebDriverScriptCommand::FindElementElementsCSSSelector(
                        parameters.value.clone(),
                        element.to_string(),
                        sender,
                    )
                },
                LocatorStrategy::LinkText | LocatorStrategy::PartialLinkText => {
                    WebDriverScriptCommand::FindElementElementsLinkText(
                        parameters.value.clone(),
                        element.to_string(),
                        parameters.using == LocatorStrategy::PartialLinkText,
                        sender,
                    )
                },
                LocatorStrategy::TagName => WebDriverScriptCommand::FindElementElementsTagName(
                    parameters.value.clone(),
                    element.to_string(),
                    sender,
                ),
                LocatorStrategy::XPath => WebDriverScriptCommand::FindElementElementsXPathSelector(
                    parameters.value.clone(),
                    element.to_string(),
                    sender,
                ),
            };
            self.browsing_context_script_command(cmd, VerifyBrowsingContextIsOpen::No)
                .map_err(|error| (ImplicitWait::Return.into(), error))?;
            wait_for_ipc_response_flatten(receiver)
                .map(|value| (!value.is_empty(), value))
                .map_err(|error| (ImplicitWait::Return.into(), error))
        })
        .and_then(|response| {
            let resp_value: Vec<Value> = response
                .into_iter()
                .map(|x| serde_json::to_value(WebElement(x)).unwrap())
                .collect();
            Ok(WebDriverResponse::Generic(ValueResponse(
                serde_json::to_value(resp_value)?,
            )))
        })
    }

    /// <https://w3c.github.io/webdriver/#find-elements-from-shadow-root>
    fn handle_find_elements_from_shadow_root(
        &self,
        shadow_root: &ShadowRoot,
        parameters: &LocatorParameters,
    ) -> WebDriverResult<WebDriverResponse> {
        // Step 4. If selector is undefined, return error with error code invalid argument.
        if parameters.value.is_empty() {
            return Err(WebDriverError::new(ErrorStatus::InvalidArgument, ""));
        }

        // Step 5. If session's current browsing context is no longer open,
        // return error with error code no such window.
        self.verify_browsing_context_is_open(self.browsing_context_id()?)?;

        // Step 6. Handle any user prompt.
        self.handle_any_user_prompts(self.webview_id()?)?;

        self.implicit_wait(|| {
            let (sender, receiver) = ipc::channel().unwrap();

            let cmd = match parameters.using {
                LocatorStrategy::CSSSelector => {
                    WebDriverScriptCommand::FindShadowElementsCSSSelector(
                        parameters.value.clone(),
                        shadow_root.to_string(),
                        sender,
                    )
                },
                LocatorStrategy::LinkText | LocatorStrategy::PartialLinkText => {
                    WebDriverScriptCommand::FindShadowElementsLinkText(
                        parameters.value.clone(),
                        shadow_root.to_string(),
                        parameters.using == LocatorStrategy::PartialLinkText,
                        sender,
                    )
                },
                LocatorStrategy::TagName => WebDriverScriptCommand::FindShadowElementsTagName(
                    parameters.value.clone(),
                    shadow_root.to_string(),
                    sender,
                ),
                LocatorStrategy::XPath => WebDriverScriptCommand::FindShadowElementsXPathSelector(
                    parameters.value.clone(),
                    shadow_root.to_string(),
                    sender,
                ),
            };
            self.browsing_context_script_command(cmd, VerifyBrowsingContextIsOpen::No)
                .map_err(|error| (ImplicitWait::Return.into(), error))?;
            wait_for_ipc_response_flatten(receiver)
                .map(|value| (!value.is_empty(), value))
                .map_err(|error| (ImplicitWait::Return.into(), error))
        })
        .and_then(|response| {
            let resp_value: Vec<Value> = response
                .into_iter()
                .map(|x| serde_json::to_value(WebElement(x)).unwrap())
                .collect();
            Ok(WebDriverResponse::Generic(ValueResponse(
                serde_json::to_value(resp_value)?,
            )))
        })
    }

    /// <https://w3c.github.io/webdriver/#find-element-from-shadow-root>
    fn handle_find_element_from_shadow_root(
        &self,
        shadow_root: &ShadowRoot,
        parameters: &LocatorParameters,
    ) -> WebDriverResult<WebDriverResponse> {
        // Step 1 - 8.
        let res = self.handle_find_elements_from_shadow_root(shadow_root, parameters)?;
        // Step 9. If result is empty, return error with error code no such element.
        // Otherwise, return the first element of result.
        unwrap_first_element_response(res)
    }

    /// <https://w3c.github.io/webdriver/#get-element-shadow-root>
    fn handle_get_shadow_root(&self, element: WebElement) -> WebDriverResult<WebDriverResponse> {
        // Step 1. If session's current browsing context is no longer open,
        // return error with error code no such window.
        self.verify_browsing_context_is_open(self.browsing_context_id()?)?;
        // Step 2. Handle any user prompt.
        self.handle_any_user_prompts(self.webview_id()?)?;
        let (sender, receiver) = ipc::channel().unwrap();
        let cmd = WebDriverScriptCommand::GetElementShadowRoot(element.to_string(), sender);
        self.browsing_context_script_command(cmd, VerifyBrowsingContextIsOpen::No)?;
        // Step 5. If shadow root is null, return error with error code no such shadow root.
        let Some(value) = wait_for_ipc_response_flatten(receiver)? else {
            return Err(WebDriverError::new(ErrorStatus::NoSuchShadowRoot, ""));
        };
        Ok(WebDriverResponse::Generic(ValueResponse(
            serde_json::to_value(ShadowRoot(value))?,
        )))
    }

    /// <https://w3c.github.io/webdriver/#get-element-rect>
    fn handle_element_rect(&self, element: &WebElement) -> WebDriverResult<WebDriverResponse> {
        // Step 1. If session's current browsing context is no longer open,
        // return error with error code no such window.
        self.verify_browsing_context_is_open(self.browsing_context_id()?)?;
        // Step 2. Handle any user prompt.
        self.handle_any_user_prompts(self.webview_id()?)?;
        let (sender, receiver) = ipc::channel().unwrap();
        let cmd = WebDriverScriptCommand::GetElementRect(element.to_string(), sender);
        self.browsing_context_script_command(cmd, VerifyBrowsingContextIsOpen::No)?;
        let rect = wait_for_ipc_response_flatten(receiver)?;
        let response = ElementRectResponse {
            x: rect.origin.x,
            y: rect.origin.y,
            width: rect.size.width,
            height: rect.size.height,
        };
        Ok(WebDriverResponse::ElementRect(response))
    }

    /// <https://w3c.github.io/webdriver/#dfn-get-element-text>
    fn handle_element_text(&self, element: &WebElement) -> WebDriverResult<WebDriverResponse> {
        // Step 1. If session's current browsing context is no longer open,
        // return error with error code no such window.
        self.verify_browsing_context_is_open(self.browsing_context_id()?)?;
        // Step 2. Handle any user prompt.
        self.handle_any_user_prompts(self.webview_id()?)?;
        let (sender, receiver) = ipc::channel().unwrap();
        let cmd = WebDriverScriptCommand::GetElementText(element.to_string(), sender);
        self.browsing_context_script_command(cmd, VerifyBrowsingContextIsOpen::No)?;
        Ok(WebDriverResponse::Generic(ValueResponse(
            serde_json::to_value(wait_for_ipc_response_flatten(receiver)?)?,
        )))
    }

    ///<https://w3c.github.io/webdriver/#get-active-element>
    fn handle_active_element(&self) -> WebDriverResult<WebDriverResponse> {
        // Step 1. If session's current browsing context is no longer open,
        // return error with error code no such window.
        self.verify_browsing_context_is_open(self.browsing_context_id()?)?;
        // Step 2. Handle any user prompt.
        self.handle_any_user_prompts(self.webview_id()?)?;
        let (sender, receiver) = ipc::channel().unwrap();
        let cmd = WebDriverScriptCommand::GetActiveElement(sender);
        self.browsing_context_script_command(cmd, VerifyBrowsingContextIsOpen::No)?;
        let value =
            wait_for_ipc_response(receiver)?.map(|x| serde_json::to_value(WebElement(x)).unwrap());
        // Step 4. If active element is a non-null element, return success
        // with data set to web element reference object for session and active element.
        // Otherwise, return error with error code no such element.
        if value.is_some() {
            Ok(WebDriverResponse::Generic(ValueResponse(
                serde_json::to_value(value)?,
            )))
        } else {
            Err(WebDriverError::new(
                ErrorStatus::NoSuchElement,
                "No active element found",
            ))
        }
    }

    /// <https://w3c.github.io/webdriver/#get-computed-role>
    fn handle_computed_role(&self, element: &WebElement) -> WebDriverResult<WebDriverResponse> {
        // Step 1. If session's current browsing context is no longer open,
        // return error with error code no such window.
        self.verify_browsing_context_is_open(self.browsing_context_id()?)?;
        // Step 2. Handle any user prompt.
        self.handle_any_user_prompts(self.webview_id()?)?;
        let (sender, receiver) = ipc::channel().unwrap();
        let cmd = WebDriverScriptCommand::GetComputedRole(element.to_string(), sender);
        self.browsing_context_script_command(cmd, VerifyBrowsingContextIsOpen::No)?;
        Ok(WebDriverResponse::Generic(ValueResponse(
            serde_json::to_value(wait_for_ipc_response_flatten(receiver)?)?,
        )))
    }

    /// <https://w3c.github.io/webdriver/#get-element-tag-name>
    fn handle_element_tag_name(&self, element: &WebElement) -> WebDriverResult<WebDriverResponse> {
        // Step 1. If session's current browsing context is no longer open,
        // return error with error code no such window.
        self.verify_browsing_context_is_open(self.browsing_context_id()?)?;
        // Step 2. Handle any user prompt.
        self.handle_any_user_prompts(self.webview_id()?)?;
        let (sender, receiver) = ipc::channel().unwrap();
        let cmd = WebDriverScriptCommand::GetElementTagName(element.to_string(), sender);
        self.browsing_context_script_command(cmd, VerifyBrowsingContextIsOpen::No)?;
        Ok(WebDriverResponse::Generic(ValueResponse(
            serde_json::to_value(wait_for_ipc_response_flatten(receiver)?)?,
        )))
    }

    /// <https://w3c.github.io/webdriver/#get-element-attribute>
    fn handle_element_attribute(
        &self,
        element: &WebElement,
        name: &str,
    ) -> WebDriverResult<WebDriverResponse> {
        // Step 1. If session's current browsing context is no longer open,
        // return error with error code no such window.
        self.verify_browsing_context_is_open(self.browsing_context_id()?)?;
        // Step 2. Handle any user prompt.
        self.handle_any_user_prompts(self.webview_id()?)?;
        let (sender, receiver) = ipc::channel().unwrap();
        let cmd = WebDriverScriptCommand::GetElementAttribute(
            element.to_string(),
            name.to_owned(),
            sender,
        );
        self.browsing_context_script_command(cmd, VerifyBrowsingContextIsOpen::No)?;
        Ok(WebDriverResponse::Generic(ValueResponse(
            serde_json::to_value(wait_for_ipc_response_flatten(receiver)?)?,
        )))
    }

    /// <https://w3c.github.io/webdriver/#get-element-property>
    fn handle_element_property(
        &self,
        element: &WebElement,
        name: &str,
    ) -> WebDriverResult<WebDriverResponse> {
        // Step 1. If session's current browsing context is no longer open,
        // return error with error code no such window.
        self.verify_browsing_context_is_open(self.browsing_context_id()?)?;
        // Step 2. Handle any user prompt.
        self.handle_any_user_prompts(self.webview_id()?)?;
        let (sender, receiver) = ipc::channel().unwrap();

        let cmd = WebDriverScriptCommand::GetElementProperty(
            element.to_string(),
            name.to_owned(),
            sender,
        );
        self.browsing_context_script_command(cmd, VerifyBrowsingContextIsOpen::No)?;

        Ok(WebDriverResponse::Generic(ValueResponse(
            serde_json::to_value(SendableJSValue(wait_for_ipc_response_flatten(receiver)?))?,
        )))
    }

    /// <https://w3c.github.io/webdriver/#get-element-css-value>
    fn handle_element_css(
        &self,
        element: &WebElement,
        name: &str,
    ) -> WebDriverResult<WebDriverResponse> {
        // Step 1. If session's current browsing context is no longer open,
        // return error with error code no such window.
        self.verify_browsing_context_is_open(self.browsing_context_id()?)?;
        // Step 2. Handle any user prompt.
        self.handle_any_user_prompts(self.webview_id()?)?;
        let (sender, receiver) = ipc::channel().unwrap();
        let cmd =
            WebDriverScriptCommand::GetElementCSS(element.to_string(), name.to_owned(), sender);
        self.browsing_context_script_command(cmd, VerifyBrowsingContextIsOpen::No)?;
        Ok(WebDriverResponse::Generic(ValueResponse(
            serde_json::to_value(wait_for_ipc_response_flatten(receiver)?)?,
        )))
    }

    /// <https://w3c.github.io/webdriver/#get-all-cookies>
    fn handle_get_cookies(&self) -> WebDriverResult<WebDriverResponse> {
        // Step 1. If session's current browsing context is no longer open,
        // return error with error code no such window.
        self.verify_browsing_context_is_open(self.browsing_context_id()?)?;
        // Step 2. Handle any user prompt.
        self.handle_any_user_prompts(self.webview_id()?)?;
        let (sender, receiver) = ipc::channel().unwrap();
        let cmd = WebDriverScriptCommand::GetCookies(sender);
        self.browsing_context_script_command(cmd, VerifyBrowsingContextIsOpen::No)?;
        let cookies = wait_for_ipc_response_flatten(receiver)?;
        let response = cookies
            .into_iter()
            .map(|cookie| cookie_msg_to_cookie(cookie.into_inner()))
            .collect::<Vec<Cookie>>();
        Ok(WebDriverResponse::Cookies(CookiesResponse(response)))
    }

    /// <https://w3c.github.io/webdriver/#get-named-cookie>
    fn handle_get_cookie(&self, name: String) -> WebDriverResult<WebDriverResponse> {
        // Step 1. If session's current browsing context is no longer open,
        // return error with error code no such window.
        self.verify_browsing_context_is_open(self.browsing_context_id()?)?;
        // Step 2. Handle any user prompt.
        self.handle_any_user_prompts(self.webview_id()?)?;
        let (sender, receiver) = ipc::channel().unwrap();
        let cmd = WebDriverScriptCommand::GetCookie(name, sender);
        self.browsing_context_script_command(cmd, VerifyBrowsingContextIsOpen::No)?;
        let cookies = wait_for_ipc_response_flatten(receiver)?;
        let Some(response) = cookies
            .into_iter()
            .map(|cookie| cookie_msg_to_cookie(cookie.into_inner()))
            .next()
        else {
            return Err(WebDriverError::new(ErrorStatus::NoSuchCookie, ""));
        };
        Ok(WebDriverResponse::Cookie(CookieResponse(response)))
    }

    /// <https://w3c.github.io/webdriver/#add-cookie>
    fn handle_add_cookie(
        &self,
        params: &AddCookieParameters,
    ) -> WebDriverResult<WebDriverResponse> {
        // Step 1. If session's current browsing context is no longer open,
        // return error with error code no such window.
        self.verify_browsing_context_is_open(self.browsing_context_id()?)?;
        // Step 2. Handle any user prompt.
        self.handle_any_user_prompts(self.webview_id()?)?;
        let (sender, receiver) = ipc::channel().unwrap();

        let mut cookie_builder =
            CookieBuilder::new(params.name.to_owned(), params.value.to_owned())
                .secure(params.secure)
                .http_only(params.httpOnly);
        if let Some(ref domain) = params.domain {
            cookie_builder = cookie_builder.domain(domain.clone());
        }
        if let Some(ref path) = params.path {
            cookie_builder = cookie_builder.path(path.clone());
        }
        if let Some(ref expiry) = params.expiry {
            if let Ok(datetime) = OffsetDateTime::from_unix_timestamp(expiry.0 as i64) {
                cookie_builder = cookie_builder.expires(datetime);
            }
        }
        if let Some(ref same_site) = params.sameSite {
            cookie_builder = match same_site.as_str() {
                "None" => Ok(cookie_builder.same_site(SameSite::None)),
                "Lax" => Ok(cookie_builder.same_site(SameSite::Lax)),
                "Strict" => Ok(cookie_builder.same_site(SameSite::Strict)),
                _ => Err(WebDriverError::new(
                    ErrorStatus::InvalidArgument,
                    "invalid argument",
                )),
            }?;
        }

        let cmd = WebDriverScriptCommand::AddCookie(cookie_builder.build(), sender);
        self.browsing_context_script_command(cmd, VerifyBrowsingContextIsOpen::No)?;
        wait_for_ipc_response_flatten(receiver)?;
        Ok(WebDriverResponse::Void)
    }

    /// <https://w3c.github.io/webdriver/#delete-cookie>
    fn handle_delete_cookie(&self, name: String) -> WebDriverResult<WebDriverResponse> {
        // Step 1. If session's current browsing context is no longer open,
        // return error with error code no such window.
        self.verify_browsing_context_is_open(self.browsing_context_id()?)?;
        // Step 2. Handle any user prompt.
        self.handle_any_user_prompts(self.webview_id()?)?;
        let (sender, receiver) = ipc::channel().unwrap();
        let cmd = WebDriverScriptCommand::DeleteCookie(name, sender);
        self.browsing_context_script_command(cmd, VerifyBrowsingContextIsOpen::No)?;
        wait_for_ipc_response_flatten(receiver)?;
        Ok(WebDriverResponse::Void)
    }

    /// <https://w3c.github.io/webdriver/#delete-all-cookies>
    fn handle_delete_cookies(&self) -> WebDriverResult<WebDriverResponse> {
        // Step 1. If session's current browsing context is no longer open,
        // return error with error code no such window.
        self.verify_browsing_context_is_open(self.browsing_context_id()?)?;
        // Step 2. Handle any user prompt.
        self.handle_any_user_prompts(self.webview_id()?)?;
        let (sender, receiver) = ipc::channel().unwrap();
        let cmd = WebDriverScriptCommand::DeleteCookies(sender);
        self.browsing_context_script_command(cmd, VerifyBrowsingContextIsOpen::Yes)?;
        wait_for_ipc_response_flatten(receiver)?;
        Ok(WebDriverResponse::Void)
    }

    /// <https://w3c.github.io/webdriver/#get-timeouts>
    fn handle_get_timeouts(&mut self) -> WebDriverResult<WebDriverResponse> {
        let timeouts = self.session()?.session_timeouts();

        // FIXME: The specification says that all of these values can be `null`, but the `webdriver` crate
        // only supports setting `script` as null. When set to null, report these values as being the
        // default ones for now.
        let timeouts = TimeoutsResponse {
            script: timeouts.script,
            page_load: timeouts.page_load.unwrap_or(DEFAULT_PAGE_LOAD_TIMEOUT),
            implicit: timeouts.implicit_wait.unwrap_or(0),
        };

        Ok(WebDriverResponse::Timeouts(timeouts))
    }

    /// <https://w3c.github.io/webdriver/#set-timeouts>
    fn handle_set_timeouts(
        &mut self,
        parameters: &TimeoutsParameters,
    ) -> WebDriverResult<WebDriverResponse> {
        let session = self.session_mut()?;

        if let Some(timeout) = parameters.script {
            session.session_timeouts_mut().script = timeout;
        }
        if let Some(timeout) = parameters.page_load {
            session.session_timeouts_mut().page_load = Some(timeout);
        }
        if let Some(timeout) = parameters.implicit {
            session.session_timeouts_mut().implicit_wait = Some(timeout);
        }

        Ok(WebDriverResponse::Void)
    }

    /// <https://w3c.github.io/webdriver/#dfn-get-page-source>
    fn handle_get_page_source(&self) -> WebDriverResult<WebDriverResponse> {
        // Step 1. If session's current browsing context is no longer open,
        // return error with error code no such window.
        self.verify_browsing_context_is_open(self.browsing_context_id()?)?;
        // Step 2. Handle any user prompt.
        self.handle_any_user_prompts(self.webview_id()?)?;
        let (sender, receiver) = ipc::channel().unwrap();

        let cmd = WebDriverScriptCommand::GetPageSource(sender);
        self.browsing_context_script_command(cmd, VerifyBrowsingContextIsOpen::No)?;

        Ok(WebDriverResponse::Generic(ValueResponse(
            serde_json::to_value(wait_for_ipc_response_flatten(receiver)?)?,
        )))
    }

    /// <https://w3c.github.io/webdriver/#perform-actions>
    fn handle_perform_actions(
        &mut self,
        parameters: ActionsParameters,
    ) -> WebDriverResult<WebDriverResponse> {
        let browsing_context = self.browsing_context_id()?;
        // Step 1. If session's current browsing context is no longer open,
        // return error with error code no such window.
        self.verify_browsing_context_is_open(browsing_context)?;

        // Step 2. Handle any user prompt.
        self.handle_any_user_prompts(self.webview_id()?)?;

        // Step 5. Let actions by tick be the result of trying to extract an action sequence
        let actions_by_tick = self.extract_an_action_sequence(parameters.actions);

        // Step 6. Dispatch actions with current browsing context
        match self.dispatch_actions(actions_by_tick, browsing_context) {
            Ok(_) => Ok(WebDriverResponse::Void),
            Err(error) => Err(WebDriverError::new(error, "")),
        }
    }

    /// <https://w3c.github.io/webdriver/#dfn-release-actions>
    fn handle_release_actions(&mut self) -> WebDriverResult<WebDriverResponse> {
        let browsing_context_id = self.browsing_context_id()?;
        // Step 1. If session's current browsing context is no longer open,
        // return error with error code no such window.
        self.verify_browsing_context_is_open(browsing_context_id)?;

        // Step 2. User prompts.
        self.handle_any_user_prompts(self.webview_id()?)?;

        // TODO: Step 4. Actions options are not used yet.

        // Step 5. Not needed because "In a session that is only a HTTP session
        // only one command can run at a time, so this will never block."

        // Step 6. Let undo actions be input cancel list in reverse order.
        let undo_actions = self
            .session_mut()?
            .input_cancel_list
            .drain(..)
            .rev()
            .map(|(id, action_item)| Vec::from([(id, action_item)]))
            .collect();
        // Step 7. Dispatch undo actions with current browsing context.
        if let Err(err) = self.dispatch_actions(undo_actions, browsing_context_id) {
            return Err(WebDriverError::new(err, "Failed to dispatch undo actions"));
        }

        // Step 8. Reset the input state of session's current top-level browsing context.
        self.session_mut()?.input_state_table.clear();

        Ok(WebDriverResponse::Void)
    }

    /// <https://w3c.github.io/webdriver/#dfn-execute-script>
    fn handle_execute_script(
        &self,
        parameters: JavascriptCommandParameters,
    ) -> WebDriverResult<WebDriverResponse> {
        // Step 1. Let body and arguments be the result of trying to extract the script arguments
        // from a request with argument parameters.
        let (func_body, args_string) = self.extract_script_arguments(parameters)?;
        // This is pretty ugly; we really want something that acts like
        // new Function() and then takes the resulting function and executes
        // it with a vec of arguments.
        let script = format!(
            "(function() {{ {}\n }})({})",
            func_body,
            args_string.join(", ")
        );
        debug!("{}", script);

        // Step 2. If session's current browsing context is no longer open,
        // return error with error code no such window.
        self.verify_browsing_context_is_open(self.browsing_context_id()?)?;

        // Step 3. Handle any user prompt.
        self.handle_any_user_prompts(self.webview_id()?)?;

        let (sender, receiver) = ipc::channel().unwrap();
        let cmd = WebDriverScriptCommand::ExecuteScript(script, sender);
        self.browsing_context_script_command(cmd, VerifyBrowsingContextIsOpen::No)?;

        let timeout_duration = self
            .session()?
            .session_timeouts()
            .script
            .map(Duration::from_millis);
        let result = wait_for_script_ipc_response_with_timeout(receiver, timeout_duration)?;

        self.javascript_evaluation_result_to_webdriver_response(result)
    }

    fn handle_execute_async_script(
        &self,
        parameters: JavascriptCommandParameters,
    ) -> WebDriverResult<WebDriverResponse> {
        // Step 1. Let body and arguments be the result of trying to extract the script arguments
        // from a request with argument parameters.
        let (function_body, mut args_string) = self.extract_script_arguments(parameters)?;
        args_string.push("resolve".to_string());

        let joined_args = args_string.join(", ");
        let script = format!(
            r#"(function() {{
              let webdriverPromise = new Promise(function(resolve, reject) {{
                  (async function() {{
                    {function_body}
                  }})({joined_args})
                    .then((v) => {{}}, (err) => reject(err))
              }})
              .then((v) => window.webdriverCallback(v), (r) => window.webdriverException(r))
              .catch((r) => window.webdriverException(r));
            }})();"#,
        );
        debug!("{}", script);

        // Step 2. If session's current browsing context is no longer open,
        // return error with error code no such window.
        self.verify_browsing_context_is_open(self.browsing_context_id()?)?;

        // Step 3. Handle any user prompt.
        self.handle_any_user_prompts(self.webview_id()?)?;

        let (sender, receiver) = ipc::channel().unwrap();
        self.browsing_context_script_command(
            WebDriverScriptCommand::ExecuteAsyncScript(script, sender),
            VerifyBrowsingContextIsOpen::No,
        )?;

        let timeout_duration = self
            .session()?
            .session_timeouts()
            .script
            .map(Duration::from_millis);
        let result = wait_for_script_ipc_response_with_timeout(receiver, timeout_duration)?;

        self.javascript_evaluation_result_to_webdriver_response(result)
    }

    fn javascript_evaluation_result_to_webdriver_response(
        &self,
        result: WebDriverJSResult,
    ) -> WebDriverResult<WebDriverResponse> {
        match result {
            Ok(value) => Ok(WebDriverResponse::Generic(ValueResponse(
                serde_json::to_value(SendableJSValue(value))?,
            ))),
            Err(error) => {
                let message = format!("{error:?}");
                let status = match error {
                    JavaScriptEvaluationError::DocumentNotFound => ErrorStatus::NoSuchWindow,
                    JavaScriptEvaluationError::CompilationFailure => ErrorStatus::JavascriptError,
                    JavaScriptEvaluationError::EvaluationFailure(Some(error_info)) => {
                        return Err(WebDriverError::new_with_data(
                            ErrorStatus::JavascriptError,
                            error_info.message,
                            None,
                            error_info.stack,
                        ));
                    },
                    JavaScriptEvaluationError::EvaluationFailure(None) => {
                        ErrorStatus::JavascriptError
                    },
                    JavaScriptEvaluationError::InternalError => ErrorStatus::JavascriptError,
                    JavaScriptEvaluationError::SerializationError(serialization_error) => {
                        match serialization_error {
                            JavaScriptEvaluationResultSerializationError::DetachedShadowRoot => {
                                ErrorStatus::DetachedShadowRoot
                            },
                            JavaScriptEvaluationResultSerializationError::OtherJavaScriptError => {
                                ErrorStatus::JavascriptError
                            },
                            JavaScriptEvaluationResultSerializationError::StaleElementReference => {
                                ErrorStatus::StaleElementReference
                            },
                            JavaScriptEvaluationResultSerializationError::UnknownType => {
                                ErrorStatus::UnsupportedOperation
                            },
                        }
                    },
                    JavaScriptEvaluationError::WebViewNotReady => ErrorStatus::NoSuchWindow,
                };
                Err(WebDriverError::new(status, message))
            },
        }
    }

    /// <https://w3c.github.io/webdriver/#dfn-element-send-keys>
    fn handle_element_send_keys(
        &mut self,
        element: &WebElement,
        keys: &SendKeysParameters,
    ) -> WebDriverResult<WebDriverResponse> {
        // Step 3. If session's current browsing context is no longer open,
        // return error with error code no such window.
        self.verify_browsing_context_is_open(self.browsing_context_id()?)?;
        // Step 4. Handle any user prompt.
        self.handle_any_user_prompts(self.webview_id()?)?;

        let (sender, receiver) = ipc::channel().unwrap();
        let cmd = WebDriverScriptCommand::WillSendKeys(
            element.to_string(),
            keys.text.to_string(),
            self.session()?.strict_file_interactability(),
            sender,
        );
        self.browsing_context_script_command(cmd, VerifyBrowsingContextIsOpen::No)?;

        // File input and non-typeable form control should have
        // been handled in `webdriver_handler.rs`.
        if !wait_for_ipc_response_flatten(receiver)? {
            return Ok(WebDriverResponse::Void);
        }

        // Step 10. Let input id be a the result of generating a UUID.
        let id = Uuid::new_v4().to_string();

        // Step 12. Add an input source
        self.session_mut()?
            .input_state_table
            .insert(id.clone(), InputSourceState::Key(KeyInputState::new()));

        // Step 13. dispatch actions for a string
        // https://w3c.github.io/webdriver/#dfn-dispatch-actions-for-a-string
        let input_events = send_keys(&keys.text);

        for event in input_events {
            match event {
                DispatchStringEvent::Keyboard(event) => {
                    let raw_string = convert_keyboard_event_to_string(&event);
                    let key_action = match event.state {
                        KeyState::Down => KeyAction::Down(KeyDownAction { value: raw_string }),
                        KeyState::Up => KeyAction::Up(KeyUpAction { value: raw_string }),
                    };
                    let action_sequence = ActionSequence {
                        id: id.clone(),
                        actions: ActionsType::Key {
                            actions: vec![KeyActionItem::Key(key_action)],
                        },
                    };

                    let actions_by_tick = self.extract_an_action_sequence(vec![action_sequence]);
                    if let Err(e) =
                        self.dispatch_actions(actions_by_tick, self.browsing_context_id()?)
                    {
                        error!("handle_element_send_keys: dispatch_actions failed: {:?}", e);
                    }
                },
                DispatchStringEvent::Composition(event) => {
                    self.send_input_event_to_embedder(InputEvent::Ime(ImeEvent::Composition(
                        event,
                    )));
                },
            }
        }

        // Step 14. Remove an input source with input state and input id.
        // It is possible that we only dispatched keydown.
        // In that case, we cannot remove the id from input state table.
        // This is a bug in spec: https://github.com/servo/servo/issues/37579#issuecomment-2990762713
        if self
            .session()?
            .input_cancel_list
            .iter()
            .all(|(cancel_item_id, _)| &id != cancel_item_id)
        {
            self.session_mut()?.input_state_table.remove(&id);
        }

        Ok(WebDriverResponse::Void)
    }

    /// <https://w3c.github.io/webdriver/#element-clear>
    fn handle_element_clear(&self, element: &WebElement) -> WebDriverResult<WebDriverResponse> {
        // Step 1. If session's current browsing context is no longer open,
        // return ErrorStatus::NoSuchWindow.
        self.verify_browsing_context_is_open(self.browsing_context_id()?)?;

        // Step 2. Try to handle any user prompt.
        self.handle_any_user_prompts(self.webview_id()?)?;

        // Step 3-11 handled in script thread.
        let (sender, receiver) = ipc::channel().unwrap();
        let cmd = WebDriverScriptCommand::ElementClear(element.to_string(), sender);
        self.browsing_context_script_command(cmd, VerifyBrowsingContextIsOpen::No)?;

        wait_for_ipc_response_flatten(receiver)?;
        Ok(WebDriverResponse::Void)
    }

    /// <https://w3c.github.io/webdriver/#element-click>
    fn handle_element_click(&mut self, element: &WebElement) -> WebDriverResult<WebDriverResponse> {
        // Step 1. If session's current browsing context is no longer open,
        // return error with error code no such window.
        let browsing_context_id = self.browsing_context_id()?;
        self.verify_browsing_context_is_open(browsing_context_id)?;

        // Step 2. Handle any user prompts.
        self.handle_any_user_prompts(self.webview_id()?)?;

        let (sender, receiver) = ipc::channel().unwrap();

        // Steps 3-7 + Step 8 for <option> are handled in script thread.
        let cmd = WebDriverScriptCommand::ElementClick(element.to_string(), sender);
        self.browsing_context_script_command(cmd, VerifyBrowsingContextIsOpen::No)?;

        match wait_for_ipc_response_flatten(receiver)? {
            Some(element_id) => {
                // Load status sender should be set up before we dispatch actions
                // to ensure webdriver can capture any navigation events.
                self.add_load_status_sender()?;

                self.perform_element_click(element_id)?;

                // Step 11. Try to wait for navigation to complete with session.
                // Check if there is a navigation with script
                let res = self.wait_for_navigation()?;

                // Clear the load status sender
                self.clear_load_status_sender()?;

                Ok(res)
            },
            // Step 13
            None => Ok(WebDriverResponse::Void),
        }
    }

    /// <https://w3c.github.io/webdriver/#element-click>
    /// Step 8 for elements other than <option>
    /// There is currently no spec for touchscreen webdriver support.
    /// There is an ongoing discussion in W3C:
    /// <https://github.com/w3c/webdriver/issues/1925>
    #[cfg(any(target_env = "ohos", target_os = "android"))]
    fn perform_element_click(&mut self, element: String) -> WebDriverResult<WebDriverResponse> {
        // Step 8.1 - 8.4: Create UUID, create input source "pointer".
        let id = Uuid::new_v4().to_string();

        let pointer_ids = self.session()?.pointer_ids();
        let (x, y) = self
            .get_origin_relative_coordinates(
                &PointerOrigin::Element(WebElement(element.clone())),
                0.0,
                0.0,
                &id,
            )
            .map_err(|err| WebDriverError::new(err, ""))?;

        // Difference with Desktop: Create an input source with coordinates directly at the
        // element centre.
        self.session_mut()?.input_state_table.insert(
            id.clone(),
            InputSourceState::Pointer(PointerInputState::new(
                PointerType::Touch,
                pointer_ids,
                x,
                y,
            )),
        );

        // Step 8.11. Construct pointer down action.
        // Step 8.12. Set a property button to 0 on pointer down action.
        let pointer_down_action = PointerDownAction {
            button: i16::from(MouseButton::Left) as u64,
            ..Default::default()
        };

        // Step 8.13. Construct pointer up action.
        // Step 8.14. Set a property button to 0 on pointer up action.
        let pointer_up_action = PointerUpAction {
            button: i16::from(MouseButton::Left) as u64,
            ..Default::default()
        };

        // Difference with Desktop: We only need pointerdown and pointerup for touchscreen.
        let action_sequence = ActionSequence {
            id: id.clone(),
            actions: ActionsType::Pointer {
                parameters: PointerActionParameters {
                    pointer_type: PointerType::Touch,
                },
                actions: vec![
                    PointerActionItem::Pointer(PointerAction::Down(pointer_down_action)),
                    PointerActionItem::Pointer(PointerAction::Up(pointer_up_action)),
                ],
            },
        };

        // Step 8.16. Dispatch a list of actions with session's current browsing context
        let actions_by_tick = self.extract_an_action_sequence(vec![action_sequence]);
        if let Err(e) = self.dispatch_actions(actions_by_tick, self.browsing_context_id()?) {
            log::error!("handle_element_click: dispatch_actions failed: {:?}", e);
        }

        // Step 8.17 Remove an input source with input state and input id.
        self.session_mut()?.input_state_table.remove(&id);

        Ok(WebDriverResponse::Void)
    }

    /// <https://w3c.github.io/webdriver/#element-click>
    /// Step 8 for elements other than <option>,
    #[cfg(not(any(target_env = "ohos", target_os = "android")))]
    fn perform_element_click(&mut self, element: String) -> WebDriverResult<WebDriverResponse> {
        // Step 8.1 - 8.4: Create UUID, create input source "pointer".
        let id = Uuid::new_v4().to_string();

        let pointer_ids = self.session()?.pointer_ids();
        self.session_mut()?.input_state_table.insert(
            id.clone(),
            InputSourceState::Pointer(PointerInputState::new(
                PointerType::Mouse,
                pointer_ids,
                0.0,
                0.0,
            )),
        );

        // Step 8.7. Construct a pointer move action.
        // Step 8.8. Set a property x to 0 on pointer move action.
        // Step 8.9. Set a property y to 0 on pointer move action.
        // Step 8.10. Set a property origin to element on pointer move action.
        let pointer_move_action = PointerMoveAction {
            duration: None,
            origin: PointerOrigin::Element(WebElement(element)),
            x: 0.0,
            y: 0.0,
            ..Default::default()
        };

        // Step 8.11. Construct pointer down action.
        // Step 8.12. Set a property button to 0 on pointer down action.
        let pointer_down_action = PointerDownAction {
            button: i16::from(MouseButton::Left) as u64,
            ..Default::default()
        };

        // Step 8.13. Construct pointer up action.
        // Step 8.14. Set a property button to 0 on pointer up action.
        let pointer_up_action = PointerUpAction {
            button: i16::from(MouseButton::Left) as u64,
            ..Default::default()
        };

        let action_sequence = ActionSequence {
            id: id.clone(),
            actions: ActionsType::Pointer {
                parameters: PointerActionParameters {
                    pointer_type: PointerType::Mouse,
                },
                actions: vec![
                    PointerActionItem::Pointer(PointerAction::Move(pointer_move_action)),
                    PointerActionItem::Pointer(PointerAction::Down(pointer_down_action)),
                    PointerActionItem::Pointer(PointerAction::Up(pointer_up_action)),
                ],
            },
        };

        // Step 8.16. Dispatch a list of actions with session's current browsing context
        let actions_by_tick = self.extract_an_action_sequence(vec![action_sequence]);
        if let Err(e) = self.dispatch_actions(actions_by_tick, self.browsing_context_id()?) {
            error!("handle_element_click: dispatch_actions failed: {:?}", e);
        }

        // Step 8.17 Remove an input source with input state and input id.
        self.session_mut()?.input_state_table.remove(&id);

        Ok(WebDriverResponse::Void)
    }

    fn take_screenshot(&self, rect: Option<Rect<f32, CSSPixel>>) -> WebDriverResult<String> {
        // Spec: Take screenshot after running the animation frame callbacks.
        let _ = self.handle_execute_async_script(JavascriptCommandParameters {
            script: "requestAnimationFrame(() => arguments[0]());".to_string(),
            args: None,
        });
        if rect.as_ref().is_some_and(Rect::is_empty) {
            return Err(WebDriverError::new(
                ErrorStatus::UnknownError,
                "The requested `rect` has zero width and/or height",
            ));
        }

        let webview_id = self.webview_id()?;
        let (sender, receiver) = crossbeam_channel::unbounded();
        self.send_message_to_embedder(WebDriverCommandMsg::TakeScreenshot(
            webview_id, rect, sender,
        ))?;

        let result = match receiver.recv_timeout(SCREENSHOT_TIMEOUT) {
            Ok(result) => Ok(result),
            Err(RecvTimeoutError::Timeout) => Err(WebDriverError::new(
                ErrorStatus::Timeout,
                "Timed out waiting to take screenshot. Test likely didn't finish.",
            )),
            Err(RecvTimeoutError::Disconnected) => Err(WebDriverError::new(
                ErrorStatus::UnknownError,
                "Could not take screenshot because channel disconnected.",
            )),
        }?;

        let image = result.map_err(|error| {
            WebDriverError::new(
                ErrorStatus::UnknownError,
                format!("Failed to take screenshot: {error:?}"),
            )
        })?;

        let mut png_data = Cursor::new(Vec::new());
        DynamicImage::ImageRgba8(image)
            .write_to(&mut png_data, ImageFormat::Png)
            .unwrap();

        Ok(base64::engine::general_purpose::STANDARD.encode(png_data.get_ref()))
    }

    fn handle_take_screenshot(&self) -> WebDriverResult<WebDriverResponse> {
        // Step 1. If session's current top-level browsing context is no longer open,
        // return error with error code no such window.
        let webview_id = self.webview_id()?;
        self.verify_top_level_browsing_context_is_open(webview_id)?;

        self.handle_any_user_prompts(webview_id)?;

        // Step 2
        let encoded = self.take_screenshot(None)?;

        Ok(WebDriverResponse::Generic(ValueResponse(
            serde_json::to_value(encoded)?,
        )))
    }

    fn handle_take_element_screenshot(
        &self,
        element: &WebElement,
    ) -> WebDriverResult<WebDriverResponse> {
        // Step 1. If session's current top-level browsing context is no longer open,
        // return error with error code no such window.
        let webview_id = self.webview_id()?;
        self.verify_top_level_browsing_context_is_open(webview_id)?;

        // Step 2. Try to handle any user prompts with session.
        self.handle_any_user_prompts(webview_id)?;

        // Step 3. Trying to get element.
        // Step 4. Scroll into view into element.
        let (sender, receiver) = ipc::channel().unwrap();
        let cmd =
            WebDriverScriptCommand::ScrollAndGetBoundingClientRect(element.to_string(), sender);
        self.browsing_context_script_command(cmd, VerifyBrowsingContextIsOpen::Yes)?;

        let rect = wait_for_ipc_response_flatten(receiver)?;

        // Step 5
        let encoded = self.take_screenshot(Some(Rect::from_untyped(&rect)))?;

        // Step 6 return success with data encoded string.
        Ok(WebDriverResponse::Generic(ValueResponse(
            serde_json::to_value(encoded)?,
        )))
    }

    /// <https://html.spec.whatwg.org/multipage/#set-rph-registration-mode>
    fn handle_custom_handlers_set_mode(
        &self,
        parameters: &CustomHandlersSetModeParameters,
    ) -> WebDriverResult<WebDriverResponse> {
        // Step 2. Let mode be the result of getting a property named "mode" from parameters.
        // Step 3. If mode is not "autoAccept", "autoReject", or "none", return a WebDriver error with WebDriver error code invalid argument.
        let mode = match parameters.mode.as_str() {
            "autoAccept" => CustomHandlersAutomationMode::AutoAccept,
            "autoReject" => CustomHandlersAutomationMode::AutoReject,
            "none" => CustomHandlersAutomationMode::None,
            _ => {
                return Err(WebDriverError::new(
                    ErrorStatus::InvalidArgument,
                    "invalid argument",
                ));
            },
        };
        // Step 4. Let document be the current browsing context's active document.
        // Step 5. Set document's registerProtocolHandler() automation mode to mode.
        self.top_level_script_command(
            WebDriverScriptCommand::SetProtocolHandlerAutomationMode(mode),
            VerifyBrowsingContextIsOpen::Yes,
        )?;
        // Step 6. Return success with data null.
        Ok(WebDriverResponse::Void)
    }

    fn handle_get_prefs(
        &self,
        parameters: &GetPrefsParameters,
    ) -> WebDriverResult<WebDriverResponse> {
        let prefs = parameters
            .prefs
            .iter()
            .map(|item| {
                (
                    item.clone(),
                    serde_json::to_value(prefs::get().get_value(item)).unwrap(),
                )
            })
            .collect::<BTreeMap<_, _>>();

        Ok(WebDriverResponse::Generic(ValueResponse(
            serde_json::to_value(prefs)?,
        )))
    }

    fn handle_set_prefs(
        &self,
        parameters: &SetPrefsParameters,
    ) -> WebDriverResult<WebDriverResponse> {
        let mut current_preferences = prefs::get().clone();
        for (key, value) in parameters.prefs.iter() {
            current_preferences.set_value(key, value.0.clone());
        }
        prefs::set(current_preferences);

        Ok(WebDriverResponse::Void)
    }

    fn handle_reset_prefs(
        &self,
        parameters: &GetPrefsParameters,
    ) -> WebDriverResult<WebDriverResponse> {
        let (new_preferences, map) = if parameters.prefs.is_empty() {
            (Preferences::default(), BTreeMap::new())
        } else {
            // If we only want to reset some of the preferences.
            let mut new_preferences = prefs::get().clone();
            let default_preferences = Preferences::default();
            for key in parameters.prefs.iter() {
                new_preferences.set_value(key, default_preferences.get_value(key))
            }

            let map = parameters
                .prefs
                .iter()
                .map(|item| (item.clone(), new_preferences.get_value(item)))
                .collect::<BTreeMap<_, _>>();

            (new_preferences, map)
        };

        prefs::set(new_preferences);

        Ok(WebDriverResponse::Generic(ValueResponse(
            serde_json::to_value(map)?,
        )))
    }

    fn handle_shutdown(&self) -> WebDriverResult<WebDriverResponse> {
        self.send_message_to_embedder(WebDriverCommandMsg::Shutdown)?;
        Ok(WebDriverResponse::Void)
    }

    fn handle_reset_all_cookies(&self) -> WebDriverResult<WebDriverResponse> {
        let (sender, receiver) = unbounded();
        self.send_message_to_embedder(WebDriverCommandMsg::ResetAllCookies(sender))?;
        if receiver.recv().is_err() {
            log::warn!("Communication failure while clearing cookies; status unknown");
        }
        Ok(WebDriverResponse::Void)
    }

    fn verify_top_level_browsing_context_is_open(
        &self,
        webview_id: WebViewId,
    ) -> Result<(), WebDriverError> {
        let (sender, receiver) = generic_channel::oneshot().unwrap();
        self.send_message_to_embedder(WebDriverCommandMsg::IsWebViewOpen(webview_id, sender))?;
        if wait_for_oneshot_response(receiver)? {
            Ok(())
        } else {
            Err(WebDriverError::new(ErrorStatus::NoSuchWindow, ""))
        }
    }

    fn verify_browsing_context_is_open(
        &self,
        browsing_context_id: BrowsingContextId,
    ) -> Result<(), WebDriverError> {
        let (sender, receiver) = generic_channel::oneshot().unwrap();
        self.send_message_to_embedder(WebDriverCommandMsg::IsBrowsingContextOpen(
            browsing_context_id,
            sender,
        ))?;
        if !receiver.recv().unwrap_or(false) {
            Err(WebDriverError::new(
                ErrorStatus::NoSuchWindow,
                "No such window",
            ))
        } else {
            Ok(())
        }
    }

    fn focus_webview(&self, webview_id: WebViewId) -> WebDriverResult<()> {
        self.send_message_to_embedder(WebDriverCommandMsg::FocusWebView(webview_id))
    }

    fn focus_browsing_context(&self, browsing_cotext_id: BrowsingContextId) -> WebDriverResult<()> {
        self.send_message_to_embedder(WebDriverCommandMsg::FocusBrowsingContext(
            browsing_cotext_id,
        ))
    }
}

impl WebDriverHandler<ServoExtensionRoute> for Handler {
    fn handle_command(
        &mut self,
        _session: &Option<Session>,
        msg: WebDriverMessage<ServoExtensionRoute>,
    ) -> WebDriverResult<WebDriverResponse> {
        info!("{:?}", msg.command);

        // Drain the load status receiver to avoid incorrect status handling
        while self.load_status_receiver.try_recv().is_ok() {}

        // Unless we are trying to create/delete a new session, check status, or shutdown Servo,
        // we need to ensure that a session has previously been created.
        match msg.command {
            WebDriverCommand::NewSession(_) |
            WebDriverCommand::Status |
            WebDriverCommand::DeleteSession |
            WebDriverCommand::Extension(ServoExtensionCommand::Shutdown) |
            WebDriverCommand::Extension(ServoExtensionCommand::ResetAllCookies) => {},
            _ => {
                self.session()?;
            },
        }

        match msg.command {
            WebDriverCommand::NewSession(ref parameters) => self.handle_new_session(parameters),
            WebDriverCommand::DeleteSession => self.handle_delete_session(),
            WebDriverCommand::Status => self.handle_status(),
            WebDriverCommand::AddCookie(ref parameters) => self.handle_add_cookie(parameters),
            WebDriverCommand::Get(ref parameters) => self.handle_get(parameters),
            WebDriverCommand::GetCurrentUrl => self.handle_current_url(),
            WebDriverCommand::GetWindowRect => {
                self.handle_window_rect(VerifyBrowsingContextIsOpen::Yes)
            },
            WebDriverCommand::SetWindowRect(ref size) => self.handle_set_window_rect(size),
            WebDriverCommand::IsEnabled(ref element) => self.handle_is_enabled(element),
            WebDriverCommand::IsSelected(ref element) => self.handle_is_selected(element),
            WebDriverCommand::GoBack => self.handle_go_back(),
            WebDriverCommand::GoForward => self.handle_go_forward(),
            WebDriverCommand::Refresh => self.handle_refresh(),
            WebDriverCommand::GetTitle => self.handle_title(),
            WebDriverCommand::GetWindowHandle => self.handle_window_handle(),
            WebDriverCommand::GetWindowHandles => self.handle_window_handles(),
            WebDriverCommand::NewWindow(ref parameters) => self.handle_new_window(parameters),
            WebDriverCommand::CloseWindow => self.handle_close_window(),
            WebDriverCommand::MaximizeWindow => self.handle_maximize_window(),
            WebDriverCommand::SwitchToFrame(ref parameters) => {
                self.handle_switch_to_frame(parameters)
            },
            WebDriverCommand::SwitchToParentFrame => self.handle_switch_to_parent_frame(),
            WebDriverCommand::SwitchToWindow(ref parameters) => {
                self.handle_switch_to_window(parameters)
            },
            WebDriverCommand::FindElement(ref parameters) => self.handle_find_element(parameters),
            WebDriverCommand::FindElements(ref parameters) => self.handle_find_elements(parameters),
            WebDriverCommand::FindElementElement(ref element, ref parameters) => {
                self.handle_find_element_from_element(element, parameters)
            },
            WebDriverCommand::FindElementElements(ref element, ref parameters) => {
                self.handle_find_elements_from_element(element, parameters)
            },
            WebDriverCommand::FindShadowRootElements(ref shadow_root, ref parameters) => {
                self.handle_find_elements_from_shadow_root(shadow_root, parameters)
            },
            WebDriverCommand::FindShadowRootElement(ref shadow_root, ref parameters) => {
                self.handle_find_element_from_shadow_root(shadow_root, parameters)
            },
            WebDriverCommand::GetShadowRoot(element) => self.handle_get_shadow_root(element),
            WebDriverCommand::GetNamedCookie(name) => self.handle_get_cookie(name),
            WebDriverCommand::GetCookies => self.handle_get_cookies(),
            WebDriverCommand::GetActiveElement => self.handle_active_element(),
            WebDriverCommand::GetComputedRole(ref element) => self.handle_computed_role(element),
            WebDriverCommand::GetElementRect(ref element) => self.handle_element_rect(element),
            WebDriverCommand::GetElementText(ref element) => self.handle_element_text(element),
            WebDriverCommand::GetElementTagName(ref element) => {
                self.handle_element_tag_name(element)
            },
            WebDriverCommand::GetElementAttribute(ref element, ref name) => {
                self.handle_element_attribute(element, name)
            },
            WebDriverCommand::GetElementProperty(ref element, ref name) => {
                self.handle_element_property(element, name)
            },
            WebDriverCommand::GetCSSValue(ref element, ref name) => {
                self.handle_element_css(element, name)
            },
            WebDriverCommand::GetPageSource => self.handle_get_page_source(),
            WebDriverCommand::PerformActions(actions_parameters) => {
                self.handle_perform_actions(actions_parameters)
            },
            WebDriverCommand::ReleaseActions => self.handle_release_actions(),
            WebDriverCommand::ExecuteScript(x) => self.handle_execute_script(x),
            WebDriverCommand::ExecuteAsyncScript(x) => self.handle_execute_async_script(x),
            WebDriverCommand::ElementSendKeys(ref element, ref keys) => {
                self.handle_element_send_keys(element, keys)
            },
            WebDriverCommand::ElementClear(ref element) => self.handle_element_clear(element),
            WebDriverCommand::ElementClick(ref element) => self.handle_element_click(element),
            WebDriverCommand::DismissAlert => self.handle_dismiss_alert(),
            WebDriverCommand::AcceptAlert => self.handle_accept_alert(),
            WebDriverCommand::GetAlertText => self.handle_get_alert_text(),
            WebDriverCommand::SendAlertText(text) => self.handle_send_alert_text(text.text),
            WebDriverCommand::DeleteCookies => self.handle_delete_cookies(),
            WebDriverCommand::DeleteCookie(name) => self.handle_delete_cookie(name),
            WebDriverCommand::GetTimeouts => self.handle_get_timeouts(),
            WebDriverCommand::SetTimeouts(ref x) => self.handle_set_timeouts(x),
            WebDriverCommand::TakeScreenshot => self.handle_take_screenshot(),
            WebDriverCommand::TakeElementScreenshot(ref x) => {
                self.handle_take_element_screenshot(x)
            },
            WebDriverCommand::Extension(extension) => match extension {
                ServoExtensionCommand::GetPrefs(ref x) => self.handle_get_prefs(x),
                ServoExtensionCommand::SetPrefs(ref x) => self.handle_set_prefs(x),
                ServoExtensionCommand::ResetPrefs(ref x) => self.handle_reset_prefs(x),
                ServoExtensionCommand::CustomHandlersSetMode(ref x) => {
                    self.handle_custom_handlers_set_mode(x)
                },
                ServoExtensionCommand::Shutdown => self.handle_shutdown(),
                ServoExtensionCommand::ResetAllCookies => self.handle_reset_all_cookies(),
            },
            _ => Err(WebDriverError::new(
                ErrorStatus::UnsupportedOperation,
                format!("Command not implemented: {:?}", msg.command),
            )),
        }
    }

    fn teardown_session(&mut self, _session: SessionTeardownKind) {
        self.session = None;
    }
}

fn wait_for_oneshot_response<T>(
    receiver: generic_channel::GenericOneshotReceiver<T>,
) -> Result<T, WebDriverError>
where
    T: for<'de> Deserialize<'de> + Serialize,
{
    receiver
        .recv()
        .map_err(|_| WebDriverError::new(ErrorStatus::NoSuchWindow, ""))
}

fn wait_for_ipc_response<T>(receiver: IpcReceiver<T>) -> Result<T, WebDriverError>
where
    T: for<'de> Deserialize<'de> + Serialize,
{
    receiver
        .recv()
        .map_err(|_| WebDriverError::new(ErrorStatus::NoSuchWindow, ""))
}

/// This function is like `wait_for_ipc_response`, but works on a channel that
/// returns a `Result<T, ErrorStatus>`, mapping all errors into `WebDriverError`.
fn wait_for_ipc_response_flatten<T>(
    receiver: IpcReceiver<Result<T, ErrorStatus>>,
) -> Result<T, WebDriverError>
where
    T: for<'de> Deserialize<'de> + Serialize,
{
    match receiver.recv() {
        Ok(Ok(value)) => Ok(value),
        Ok(Err(error_status)) => Err(WebDriverError::new(error_status, "")),
        Err(_) => Err(WebDriverError::new(ErrorStatus::NoSuchWindow, "")),
    }
}

fn wait_for_script_ipc_response_with_timeout<T>(
    receiver: IpcReceiver<T>,
    timeout: Option<Duration>,
) -> Result<T, WebDriverError>
where
    T: for<'de> Deserialize<'de> + Serialize,
{
    let Some(timeout) = timeout else {
        return wait_for_ipc_response(receiver);
    };
    receiver
        .try_recv_timeout(timeout)
        .map_err(|error| match error {
            TryRecvError::IpcError(_) => WebDriverError::new(ErrorStatus::NoSuchWindow, ""),
            TryRecvError::Empty => WebDriverError::new(ErrorStatus::ScriptTimeout, ""),
        })
}

fn unwrap_first_element_response(res: WebDriverResponse) -> WebDriverResult<WebDriverResponse> {
    if let WebDriverResponse::Generic(ValueResponse(values)) = res {
        let arr = values.as_array().unwrap();
        if let Some(first) = arr.first() {
            Ok(WebDriverResponse::Generic(ValueResponse(first.clone())))
        } else {
            Err(WebDriverError::new(ErrorStatus::NoSuchElement, ""))
        }
    } else {
        unreachable!()
    }
}

fn convert_keyboard_event_to_string(event: &KeyboardEvent) -> String {
    let key = &event.key;
    let named_key = match key {
        Key::Character(s) => return s.to_string(),
        Key::Named(named_key) => named_key,
    };

    match event.location {
        Location::Left | Location::Standard => match named_key {
            NamedKey::Unidentified => '\u{E000}'.to_string(),
            NamedKey::Cancel => '\u{E001}'.to_string(),
            NamedKey::Help => '\u{E002}'.to_string(),
            NamedKey::Backspace => '\u{E003}'.to_string(),
            NamedKey::Tab => '\u{E004}'.to_string(),
            NamedKey::Clear => '\u{E005}'.to_string(),
            NamedKey::Enter => match event.code {
                Code::NumpadEnter => '\u{E007}'.to_string(),
                _ => '\u{E006}'.to_string(),
            },
            NamedKey::Shift => '\u{E008}'.to_string(),
            NamedKey::Control => '\u{E009}'.to_string(),
            NamedKey::Alt => '\u{E00A}'.to_string(),
            NamedKey::Pause => '\u{E00B}'.to_string(),
            NamedKey::Escape => '\u{E00C}'.to_string(),
            NamedKey::PageUp => '\u{E00E}'.to_string(),
            NamedKey::PageDown => '\u{E00F}'.to_string(),
            NamedKey::End => '\u{E010}'.to_string(),
            NamedKey::Home => '\u{E011}'.to_string(),
            NamedKey::ArrowLeft => '\u{E012}'.to_string(),
            NamedKey::ArrowUp => '\u{E013}'.to_string(),
            NamedKey::ArrowRight => '\u{E014}'.to_string(),
            NamedKey::ArrowDown => '\u{E015}'.to_string(),
            NamedKey::Insert => '\u{E016}'.to_string(),
            NamedKey::Delete => '\u{E017}'.to_string(),
            NamedKey::F1 => '\u{E031}'.to_string(),
            NamedKey::F2 => '\u{E032}'.to_string(),
            NamedKey::F3 => '\u{E033}'.to_string(),
            NamedKey::F4 => '\u{E034}'.to_string(),
            NamedKey::F5 => '\u{E035}'.to_string(),
            NamedKey::F6 => '\u{E036}'.to_string(),
            NamedKey::F7 => '\u{E037}'.to_string(),
            NamedKey::F8 => '\u{E038}'.to_string(),
            NamedKey::F9 => '\u{E039}'.to_string(),
            NamedKey::F10 => '\u{E03A}'.to_string(),
            NamedKey::F11 => '\u{E03B}'.to_string(),
            NamedKey::F12 => '\u{E03C}'.to_string(),
            NamedKey::Meta => '\u{E03D}'.to_string(),
            NamedKey::ZenkakuHankaku => '\u{E040}'.to_string(),
            _ => {
                error!("Unexpected NamedKey on send_keys");
                '\u{E000}'.to_string()
            },
        },
        Location::Right | Location::Numpad => match named_key {
            NamedKey::Shift => '\u{E050}'.to_string(),
            NamedKey::Control => '\u{E051}'.to_string(),
            NamedKey::Alt => '\u{E052}'.to_string(),
            NamedKey::Meta => '\u{E053}'.to_string(),
            NamedKey::PageUp => '\u{E054}'.to_string(),
            NamedKey::PageDown => '\u{E055}'.to_string(),
            NamedKey::End => '\u{E056}'.to_string(),
            NamedKey::Home => '\u{E057}'.to_string(),
            NamedKey::ArrowLeft => '\u{E058}'.to_string(),
            NamedKey::ArrowUp => '\u{E059}'.to_string(),
            NamedKey::ArrowRight => '\u{E05A}'.to_string(),
            NamedKey::ArrowDown => '\u{E05B}'.to_string(),
            NamedKey::Insert => '\u{E05C}'.to_string(),
            NamedKey::Delete => '\u{E05D}'.to_string(),
            _ => {
                error!("Unexpected NamedKey on send_keys");
                '\u{E000}'.to_string()
            },
        },
    }
}
