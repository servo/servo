/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![crate_name = "webdriver_server"]
#![crate_type = "rlib"]
#![deny(unsafe_code)]

mod actions;
mod capabilities;
mod user_prompt;

use std::borrow::ToOwned;
use std::cell::{Cell, LazyCell, RefCell};
use std::collections::{BTreeMap, HashMap};
use std::io::Cursor;
use std::net::{SocketAddr, SocketAddrV4};
use std::time::Duration;
use std::{env, fmt, process, thread};

use base::id::{BrowsingContextId, WebViewId};
use base64::Engine;
use capabilities::ServoCapabilities;
use cookie::{CookieBuilder, Expiration, SameSite};
use crossbeam_channel::{Receiver, Sender, after, select, unbounded};
use embedder_traits::{
    EventLoopWaker, MouseButton, WebDriverCommandMsg, WebDriverCommandResponse, WebDriverFrameId,
    WebDriverJSError, WebDriverJSResult, WebDriverJSValue, WebDriverLoadStatus, WebDriverMessageId,
    WebDriverScriptCommand,
};
use euclid::{Point2D, Rect, Size2D};
use http::method::Method;
use image::{DynamicImage, ImageFormat, RgbaImage};
use ipc_channel::ipc::{self, IpcReceiver, IpcSender};
use ipc_channel::router::ROUTER;
use keyboard_types::webdriver::send_keys;
use log::{debug, info};
use pixels::PixelFormat;
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
use webdriver::actions::{
    ActionSequence, ActionsType, PointerAction, PointerActionItem, PointerActionParameters,
    PointerDownAction, PointerMoveAction, PointerOrigin, PointerType, PointerUpAction,
};
use webdriver::capabilities::CapabilitiesMatching;
use webdriver::command::{
    ActionsParameters, AddCookieParameters, GetParameters, JavascriptCommandParameters,
    LocatorParameters, NewSessionParameters, NewWindowParameters, SendKeysParameters,
    SwitchToFrameParameters, SwitchToWindowParameters, TimeoutsParameters, WebDriverCommand,
    WebDriverExtensionCommand, WebDriverMessage, WindowRectParameters,
};
use webdriver::common::{Cookie, Date, LocatorStrategy, Parameters, ShadowRoot, WebElement};
use webdriver::error::{ErrorStatus, WebDriverError, WebDriverResult};
use webdriver::httpapi::WebDriverExtensionRoute;
use webdriver::response::{
    CloseWindowResponse, CookieResponse, CookiesResponse, ElementRectResponse, NewSessionResponse,
    NewWindowResponse, TimeoutsResponse, ValueResponse, WebDriverResponse, WindowRectResponse,
};
use webdriver::server::{self, Session, SessionTeardownKind, WebDriverHandler};

use crate::actions::{ActionItem, InputSourceState, PointerInputState};
use crate::user_prompt::{
    UserPromptHandler, default_unhandled_prompt_behavior, deserialize_unhandled_prompt_behaviour,
};

#[derive(Default)]
pub struct WebDriverMessageIdGenerator {
    counter: Cell<usize>,
}

impl WebDriverMessageIdGenerator {
    pub fn new() -> Self {
        Self {
            counter: Cell::new(0),
        }
    }

    /// Returns a unique ID.
    pub fn next(&self) -> WebDriverMessageId {
        let id = self.counter.get();
        self.counter.set(id + 1);
        WebDriverMessageId(id)
    }
}

fn extension_routes() -> Vec<(Method, &'static str, ServoExtensionRoute)> {
    vec![
        (
            Method::POST,
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
    webdriver_response_receiver: IpcReceiver<WebDriverCommandResponse>,
) {
    let handler = Handler::new(
        embedder_sender,
        event_loop_waker,
        webdriver_response_receiver,
    );

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
                Err(_) => panic!("Unable to start WebDriver HTTPD server"),
            }
        })
        .expect("Thread spawning failed");
}

/// Represents the current WebDriver session and holds relevant session state.
/// Currently, only 1 webview is supported per session.
/// So only there is only 1 InputState.
pub struct WebDriverSession {
    /// <https://www.w3.org/TR/webdriver2/#dfn-session-id>
    id: Uuid,

    /// <https://www.w3.org/TR/webdriver2/#dfn-current-top-level-browsing-context>
    webview_id: WebViewId,

    /// <https://www.w3.org/TR/webdriver2/#dfn-current-browsing-context>
    browsing_context_id: BrowsingContextId,

    /// <https://www.w3.org/TR/webdriver2/#dfn-window-handles>
    window_handles: HashMap<WebViewId, String>,

    /// Time to wait for injected scripts to run before interrupting them.  A [`None`] value
    /// specifies that the script should run indefinitely.
    script_timeout: Option<u64>,

    /// Time to wait for a page to finish loading upon navigation.
    load_timeout: u64,

    /// Time to wait for the element location strategy when retrieving elements, and when
    /// waiting for an element to become interactable.
    implicit_wait_timeout: u64,

    page_loading_strategy: String,

    strict_file_interactability: bool,

    user_prompt_handler: UserPromptHandler,

    /// <https://w3c.github.io/webdriver/#dfn-input-state-map>
    input_state_table: RefCell<HashMap<String, InputSourceState>>,

    /// <https://w3c.github.io/webdriver/#dfn-input-cancel-list>
    input_cancel_list: RefCell<Vec<(String, ActionItem)>>,
}

impl WebDriverSession {
    pub fn new(browsing_context_id: BrowsingContextId, webview_id: WebViewId) -> WebDriverSession {
        let mut window_handles = HashMap::new();
        let handle = Uuid::new_v4().to_string();
        window_handles.insert(webview_id, handle);

        WebDriverSession {
            id: Uuid::new_v4(),
            webview_id,
            browsing_context_id,

            window_handles,

            script_timeout: Some(30_000),
            load_timeout: 300_000,
            implicit_wait_timeout: 0,

            page_loading_strategy: "normal".to_string(),
            strict_file_interactability: false,
            user_prompt_handler: UserPromptHandler::new(),

            input_state_table: RefCell::new(HashMap::new()),
            input_cancel_list: RefCell::new(Vec::new()),
        }
    }
}

struct Handler {
    /// The threaded receiver on which we can block for a load-status.
    /// It will receive messages sent on the load_status_sender,
    /// and forwarded by the IPC router.
    load_status_receiver: Receiver<WebDriverLoadStatus>,
    /// The IPC sender which we can clone and pass along to the constellation,
    /// for it to send us a load-status. Messages sent on it
    /// will be forwarded to the load_status_receiver.
    load_status_sender: IpcSender<WebDriverLoadStatus>,

    session: Option<WebDriverSession>,

    /// A [`Sender`] that sends messages to the embedder that this `WebDriver instance controls.
    /// In addition to sending a message, we must always wake up the embedder's event loop so it
    /// knows that more messages are available for processing.
    embedder_sender: Sender<WebDriverCommandMsg>,

    /// An [`EventLoopWaker`] which is used to wake up the embedder event loop.
    event_loop_waker: Box<dyn EventLoopWaker>,

    /// Receiver notification from the constellation when a command is completed
    webdriver_response_receiver: IpcReceiver<WebDriverCommandResponse>,

    id_generator: WebDriverMessageIdGenerator,

    current_action_id: Cell<Option<WebDriverMessageId>>,

    /// Number of pending actions of which WebDriver is waiting for responses.
    num_pending_actions: Cell<u32>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[allow(clippy::enum_variant_names)]
enum ServoExtensionRoute {
    GetPrefs,
    SetPrefs,
    ResetPrefs,
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
        };
        Ok(WebDriverCommand::Extension(command))
    }
}

#[derive(Clone, Debug, PartialEq)]
#[allow(clippy::enum_variant_names)]
enum ServoExtensionCommand {
    GetPrefs(GetPrefsParameters),
    SetPrefs(SetPrefsParameters),
    ResetPrefs(GetPrefsParameters),
}

impl WebDriverExtensionCommand for ServoExtensionCommand {
    fn parameters_json(&self) -> Option<Value> {
        match *self {
            ServoExtensionCommand::GetPrefs(ref x) => serde_json::to_value(x).ok(),
            ServoExtensionCommand::SetPrefs(ref x) => serde_json::to_value(x).ok(),
            ServoExtensionCommand::ResetPrefs(ref x) => serde_json::to_value(x).ok(),
        }
    }
}

#[derive(Clone)]
struct SendableWebDriverJSValue(pub WebDriverJSValue);

impl Serialize for SendableWebDriverJSValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self.0 {
            WebDriverJSValue::Undefined => serializer.serialize_unit(),
            WebDriverJSValue::Null => serializer.serialize_unit(),
            WebDriverJSValue::Boolean(x) => serializer.serialize_bool(x),
            WebDriverJSValue::Int(x) => serializer.serialize_i32(x),
            WebDriverJSValue::Number(x) => serializer.serialize_f64(x),
            WebDriverJSValue::String(ref x) => serializer.serialize_str(x),
            WebDriverJSValue::Element(ref x) => x.serialize(serializer),
            WebDriverJSValue::Frame(ref x) => x.serialize(serializer),
            WebDriverJSValue::Window(ref x) => x.serialize(serializer),
            WebDriverJSValue::ArrayLike(ref x) => x
                .iter()
                .map(|element| SendableWebDriverJSValue(element.clone()))
                .collect::<Vec<SendableWebDriverJSValue>>()
                .serialize(serializer),
            WebDriverJSValue::Object(ref x) => x
                .iter()
                .map(|(k, v)| (k.clone(), SendableWebDriverJSValue(v.clone())))
                .collect::<HashMap<String, SendableWebDriverJSValue>>()
                .serialize(serializer),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct WebDriverPrefValue(pub PrefValue);

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

impl Handler {
    pub fn new(
        embedder_sender: Sender<WebDriverCommandMsg>,
        event_loop_waker: Box<dyn EventLoopWaker>,
        webdriver_response_receiver: IpcReceiver<WebDriverCommandResponse>,
    ) -> Handler {
        // Create a pair of both an IPC and a threaded channel,
        // keep the IPC sender to clone and pass to the constellation for each load,
        // and keep a threaded receiver to block on an incoming load-status.
        // Pass the others to the IPC router so that IPC messages are forwarded to the threaded receiver.
        // We need to use the router because IPC does not come with a timeout on receive/select.
        let (load_status_sender, receiver) = ipc::channel().unwrap();
        let (sender, load_status_receiver) = unbounded();
        ROUTER.route_ipc_receiver_to_crossbeam_sender(receiver, sender);

        Handler {
            load_status_sender,
            load_status_receiver,
            session: None,
            embedder_sender,
            event_loop_waker,
            webdriver_response_receiver,
            id_generator: WebDriverMessageIdGenerator::new(),
            current_action_id: Cell::new(None),
            num_pending_actions: Cell::new(0),
        }
    }

    fn increment_num_pending_actions(&self) {
        // Increase the num_pending_actions by one every time we dispatch non null actions.
        self.num_pending_actions
            .set(self.num_pending_actions.get() + 1);
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

    // This function is called only if session and webview are verified.
    fn verified_webview_id(&self) -> WebViewId {
        self.session().unwrap().webview_id
    }

    fn focus_webview_id(&self) -> WebDriverResult<WebViewId> {
        let (sender, receiver) = ipc::channel().unwrap();
        self.send_message_to_embedder(WebDriverCommandMsg::GetFocusedWebView(sender.clone()))?;
        // Wait until the document is ready before returning the top-level browsing context id.
        match wait_for_script_response(receiver)? {
            Some(webview_id) => Ok(webview_id),
            None => Err(WebDriverError::new(
                ErrorStatus::NoSuchWindow,
                "No focused webview found",
            )),
        }
    }

    fn session(&self) -> WebDriverResult<&WebDriverSession> {
        match self.session {
            Some(ref x) => Ok(x),
            None => Err(WebDriverError::new(
                ErrorStatus::SessionNotCreated,
                "Session not created",
            )),
        }
    }

    fn session_mut(&mut self) -> WebDriverResult<&mut WebDriverSession> {
        match self.session {
            Some(ref mut x) => Ok(x),
            None => Err(WebDriverError::new(
                ErrorStatus::SessionNotCreated,
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

        let mut servo_capabilities = ServoCapabilities::new();
        let processed_capabilities = parameters.match_browser(&mut servo_capabilities)?;

        // Step 1. If the list of active HTTP sessions is not empty
        // return error with error code session not created.
        if self.session.is_some() {
            Err(WebDriverError::new(
                ErrorStatus::SessionNotCreated,
                "Session already created",
            ))
        } else {
            match processed_capabilities {
                Some(mut processed) => {
                    let webview_id = self.focus_webview_id()?;
                    let browsing_context_id = BrowsingContextId::from(webview_id);
                    let mut session = WebDriverSession::new(browsing_context_id, webview_id);

                    match processed.get("pageLoadStrategy") {
                        Some(strategy) => session.page_loading_strategy = strategy.to_string(),
                        None => {
                            processed.insert(
                                "pageLoadStrategy".to_string(),
                                json!(session.page_loading_strategy),
                            );
                        },
                    }

                    match processed.get("strictFileInteractability") {
                        Some(strict_file_interactability) => {
                            session.strict_file_interactability =
                                strict_file_interactability.as_bool().unwrap()
                        },
                        None => {
                            processed.insert(
                                "strictFileInteractability".to_string(),
                                json!(session.strict_file_interactability),
                            );
                        },
                    }

                    match processed.get("proxy") {
                        Some(_) => (),
                        None => {
                            processed.insert("proxy".to_string(), json!({}));
                        },
                    }

                    if let Some(timeouts) = processed.get("timeouts") {
                        if let Some(script_timeout_value) = timeouts.get("script") {
                            session.script_timeout = script_timeout_value.as_u64();
                        }
                        if let Some(load_timeout_value) = timeouts.get("pageLoad") {
                            if let Some(load_timeout) = load_timeout_value.as_u64() {
                                session.load_timeout = load_timeout;
                            }
                        }
                        if let Some(implicit_wait_timeout_value) = timeouts.get("implicit") {
                            if let Some(implicit_wait_timeout) =
                                implicit_wait_timeout_value.as_u64()
                            {
                                session.implicit_wait_timeout = implicit_wait_timeout;
                            }
                        }
                    }
                    processed.insert(
                        "timeouts".to_string(),
                        json!({
                            "script": session.script_timeout,
                            "pageLoad": session.load_timeout,
                            "implicit": session.implicit_wait_timeout,
                        }),
                    );

                    match processed.get("acceptInsecureCerts") {
                        Some(_accept_insecure_certs) => {
                            // FIXME do something here?
                        },
                        None => {
                            processed.insert(
                                "acceptInsecureCerts".to_string(),
                                json!(servo_capabilities.accept_insecure_certs),
                            );
                        },
                    }

                    match processed.get("unhandledPromptBehavior") {
                        Some(unhandled_prompt_behavior) => {
                            session.user_prompt_handler = deserialize_unhandled_prompt_behaviour(
                                unhandled_prompt_behavior.clone(),
                            )?;
                        },
                        None => {
                            processed.insert(
                                "unhandledPromptBehavior".to_string(),
                                json!(default_unhandled_prompt_behavior()),
                            );
                        },
                    }

                    processed.insert(
                        "browserName".to_string(),
                        json!(servo_capabilities.browser_name),
                    );
                    processed.insert(
                        "browserVersion".to_string(),
                        json!(servo_capabilities.browser_version),
                    );
                    processed.insert(
                        "platformName".to_string(),
                        json!(
                            servo_capabilities
                                .platform_name
                                .unwrap_or("unknown".to_string())
                        ),
                    );
                    processed.insert(
                        "setWindowRect".to_string(),
                        json!(servo_capabilities.set_window_rect),
                    );
                    processed.insert(
                        "userAgent".to_string(),
                        servo_config::pref!(user_agent).into(),
                    );

                    let response =
                        NewSessionResponse::new(session.id.to_string(), Value::Object(processed));
                    self.session = Some(session);
                    Ok(WebDriverResponse::NewSession(response))
                },
                // Step 5. If capabilities's is null,
                // return error with error code session not created.
                None => Err(WebDriverError::new(
                    ErrorStatus::SessionNotCreated,
                    "Session not created due to invalid capabilities",
                )),
            }
        }
    }

    fn handle_delete_session(&mut self) -> WebDriverResult<WebDriverResponse> {
        self.session = None;
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
        let browsing_context_id = self.session()?.browsing_context_id;
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
        let webview_id = self.session()?.webview_id;
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
        let webview_id = self.session()?.webview_id;
        // Step 2. If session's current top-level browsing context is no longer open,
        // return error with error code no such window.
        self.verify_top_level_browsing_context_is_open(webview_id)?;
        // Step 3. If URL is not an absolute URL or is not an absolute URL with fragment
        // or not a local scheme, return error with error code invalid argument.
        let url = match ServoUrl::parse(&parameters.url[..]) {
            Ok(url) => url,
            Err(_) => {
                return Err(WebDriverError::new(
                    ErrorStatus::InvalidArgument,
                    "Invalid URL",
                ));
            },
        };

        // Step 4. Handle any user prompt.
        self.handle_any_user_prompts(webview_id)?;

        let cmd_msg =
            WebDriverCommandMsg::LoadUrl(webview_id, url, self.load_status_sender.clone());
        self.send_message_to_embedder(cmd_msg)?;

        // Step 8.2.1: try to wait for navigation to complete.
        self.wait_for_navigation_to_complete()?;

        // Step 8.3. Set current browsing context with session and current top browsing context
        self.session_mut()?.browsing_context_id = BrowsingContextId::from(webview_id);

        Ok(WebDriverResponse::Void)
    }

    /// <https://w3c.github.io/webdriver/#dfn-wait-for-navigation-to-complete>
    fn wait_for_navigation_to_complete(&self) -> WebDriverResult<WebDriverResponse> {
        debug!("waiting for load");

        let session = self.session()?;

        // Step 1. If session's page loading strategy is "none",
        // return success with data null.
        if session.page_loading_strategy == "none" {
            return Ok(WebDriverResponse::Void);
        }

        // Step 2. If session's current browsing context is no longer open,
        // return success with data null.
        if self
            .verify_browsing_context_is_open(session.browsing_context_id)
            .is_err()
        {
            return Ok(WebDriverResponse::Void);
        }

        // Step 3. let timeout be the session's page load timeout.
        let timeout = self.session()?.load_timeout;

        // TODO: Step 4. Implement timer parameter

        let result = select! {
            recv(self.load_status_receiver) -> res => {
                    match res {
                    Ok(WebDriverLoadStatus::Blocked) => {
                        // TODO: evaluate the correctness later
                        // Load status is block means an user prompt is shown.
                        // Alot of tests expect this to return success
                        // then the user prompt is handled in the next command.
                        // If user prompt can't be handler, next command returns
                        // an error anyway.
                        Ok(WebDriverResponse::Void)
                    }
                    _ => {
                        Ok(WebDriverResponse::Void)
                    }
                }
            },
            recv(after(Duration::from_millis(timeout))) -> _ => Err(
                WebDriverError::new(ErrorStatus::Timeout, "Load timed out")
            ),
        };
        debug!("finished waiting for load with {:?}", result);
        result
    }

    /// <https://w3c.github.io/webdriver/#dfn-get-current-url>
    fn handle_current_url(&self) -> WebDriverResult<WebDriverResponse> {
        // Step 1. If session's current top-level browsing context is no longer open,
        // return error with error code no such window.
        self.verify_top_level_browsing_context_is_open(self.session()?.webview_id)?;
        // Step 2. Handle any user prompt.
        self.handle_any_user_prompts(self.session()?.webview_id)?;

        let (sender, receiver) = ipc::channel().unwrap();
        self.top_level_script_command(
            WebDriverScriptCommand::GetUrl(sender),
            VerifyBrowsingContextIsOpen::No,
        )?;

        let url = wait_for_script_response(receiver)?;

        Ok(WebDriverResponse::Generic(ValueResponse(
            serde_json::to_value(url.as_str())?,
        )))
    }

    /// <https://w3c.github.io/webdriver/#get-window-rect>
    fn handle_window_rect(
        &self,
        verify: VerifyBrowsingContextIsOpen,
    ) -> WebDriverResult<WebDriverResponse> {
        let (sender, receiver) = ipc::channel().unwrap();
        let webview_id = self.session()?.webview_id;
        // Step 1. If session's current top-level browsing context is no longer open,
        // return error with error code no such window.
        if let VerifyBrowsingContextIsOpen::Yes = verify {
            self.verify_top_level_browsing_context_is_open(webview_id)?;
        }

        // Step 2. Handle any user prompt.
        self.handle_any_user_prompts(webview_id)?;

        self.send_message_to_embedder(WebDriverCommandMsg::GetWindowRect(webview_id, sender))?;

        let window_rect = wait_for_script_response(receiver)?;
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

        let webview_id = self.session()?.webview_id;
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
        let (sender, receiver) = ipc::channel().unwrap();
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

        let window_rect = wait_for_script_response(receiver)?;
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
        let webview_id = self.session()?.webview_id;
        // Step 2. If session's current top-level browsing context is no longer open,
        // return error with error code no such window.
        self.verify_top_level_browsing_context_is_open(webview_id)?;

        // Step 3. Try to handle any user prompts with session.
        self.handle_any_user_prompts(self.session()?.webview_id)?;

        // Step 4. (TODO) Fully exit fullscreen.

        // Step 5. (TODO) Restore the window.

        // Step 6. Maximize the window of session's current top-level browsing context.
        let (sender, receiver) = ipc::channel().unwrap();
        self.send_message_to_embedder(WebDriverCommandMsg::MaximizeWebView(webview_id, sender))?;

        let window_rect = wait_for_script_response(receiver)?;
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
        let (sender, receiver) = ipc::channel().unwrap();

        self.browsing_context_script_command(
            WebDriverScriptCommand::IsEnabled(element.to_string(), sender),
            VerifyBrowsingContextIsOpen::Yes,
        )?;

        match wait_for_script_response(receiver)? {
            Ok(is_enabled) => Ok(WebDriverResponse::Generic(ValueResponse(
                serde_json::to_value(is_enabled)?,
            ))),
            Err(error) => Err(WebDriverError::new(error, "")),
        }
    }

    fn handle_is_selected(&self, element: &WebElement) -> WebDriverResult<WebDriverResponse> {
        let (sender, receiver) = ipc::channel().unwrap();

        self.browsing_context_script_command(
            WebDriverScriptCommand::IsSelected(element.to_string(), sender),
            VerifyBrowsingContextIsOpen::Yes,
        )?;

        match wait_for_script_response(receiver)? {
            Ok(is_selected) => Ok(WebDriverResponse::Generic(ValueResponse(
                serde_json::to_value(is_selected)?,
            ))),
            Err(error) => Err(WebDriverError::new(error, "")),
        }
    }

    /// <https://w3c.github.io/webdriver/#back>
    fn handle_go_back(&self) -> WebDriverResult<WebDriverResponse> {
        let webview_id = self.session()?.webview_id;
        // Step 1. If session's current top-level browsing context is no longer open,
        // return error with error code no such window.
        self.verify_top_level_browsing_context_is_open(webview_id)?;

        // Step 2. Handle any user prompt.
        self.handle_any_user_prompts(webview_id)?;

        self.send_message_to_embedder(WebDriverCommandMsg::GoBack(
            webview_id,
            self.load_status_sender.clone(),
        ))?;
        self.wait_for_navigation_to_complete()
    }

    /// <https://w3c.github.io/webdriver/#forward>
    fn handle_go_forward(&self) -> WebDriverResult<WebDriverResponse> {
        let webview_id = self.session()?.webview_id;
        // Step 1. If session's current top-level browsing context is no longer open,
        // return error with error code no such window.
        self.verify_top_level_browsing_context_is_open(webview_id)?;

        // Step 2. Handle any user prompt.
        self.handle_any_user_prompts(webview_id)?;

        self.send_message_to_embedder(WebDriverCommandMsg::GoForward(
            webview_id,
            self.load_status_sender.clone(),
        ))?;
        self.wait_for_navigation_to_complete()
    }

    /// <https://w3c.github.io/webdriver/#refresh>
    fn handle_refresh(&mut self) -> WebDriverResult<WebDriverResponse> {
        let webview_id = self.session()?.webview_id;
        // Step 1. If session's current top-level browsing context is no longer open,
        // return error with error code no such window.
        self.verify_top_level_browsing_context_is_open(webview_id)?;

        // Step 2. Handle any user prompt.
        self.handle_any_user_prompts(webview_id)?;

        let cmd_msg = WebDriverCommandMsg::Refresh(webview_id, self.load_status_sender.clone());
        self.send_message_to_embedder(cmd_msg)?;

        // Step 4.1: Try to wait for navigation to complete.
        self.wait_for_navigation_to_complete()?;

        // Step 5. Set current browsing context with session and current top browsing context.
        self.session_mut()?.browsing_context_id = BrowsingContextId::from(webview_id);

        Ok(WebDriverResponse::Void)
    }

    /// <https://w3c.github.io/webdriver/#get-title>
    fn handle_title(&self) -> WebDriverResult<WebDriverResponse> {
        // Step 1. If session's current top-level browsing context is no longer open,
        // return error with error code no such window.
        self.verify_top_level_browsing_context_is_open(self.session()?.webview_id)?;
        // Step 2. Handle any user prompt.
        self.handle_any_user_prompts(self.session()?.webview_id)?;

        let (sender, receiver) = ipc::channel().unwrap();

        self.top_level_script_command(
            WebDriverScriptCommand::GetTitle(sender),
            VerifyBrowsingContextIsOpen::No,
        )?;

        // Step 3. Let title be the session's current top-level
        // browsing context's active document's title.
        let title = wait_for_script_response(receiver)?;
        Ok(WebDriverResponse::Generic(ValueResponse(
            serde_json::to_value(title)?,
        )))
    }

    /// <https://w3c.github.io/webdriver/#get-window-handle>
    fn handle_window_handle(&self) -> WebDriverResult<WebDriverResponse> {
        let session = self.session.as_ref().unwrap();
        // Step 1. If session's current top-level browsing context is no longer open,
        // return error with error code no such window.
        self.verify_top_level_browsing_context_is_open(session.webview_id)?;
        match session.window_handles.get(&session.webview_id) {
            Some(handle) => Ok(WebDriverResponse::Generic(ValueResponse(
                serde_json::to_value(handle)?,
            ))),
            None => Ok(WebDriverResponse::Void),
        }
    }

    /// <https://w3c.github.io/webdriver/#get-window-handles>
    fn handle_window_handles(&self) -> WebDriverResult<WebDriverResponse> {
        let handles = self
            .session
            .as_ref()
            .unwrap()
            .window_handles
            .values()
            .map(serde_json::to_value)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(WebDriverResponse::Generic(ValueResponse(
            serde_json::to_value(handles)?,
        )))
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

    /// <https://w3c.github.io/webdriver/#close-window>
    fn handle_close_window(&mut self) -> WebDriverResult<WebDriverResponse> {
        let webview_id = self.session()?.webview_id;
        // Step 1. If session's current top-level browsing context is no longer open,
        // return error with error code no such window.
        self.verify_top_level_browsing_context_is_open(webview_id)?;

        // Step 2. Handle any user prompt.
        self.handle_any_user_prompts(webview_id)?;

        // Step 3. Close session's current top-level browsing context.
        let session = self.session_mut().unwrap();
        session.window_handles.remove(&webview_id);
        let cmd_msg = WebDriverCommandMsg::CloseWebView(session.webview_id);
        self.send_message_to_embedder(cmd_msg)?;
        let window_handles: Vec<String> = self
            .session()
            .unwrap()
            .window_handles
            .values()
            .cloned()
            .collect();
        // Step 4. If there are no more open top-level browsing contexts, try to close the session.
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
        let (sender, receiver) = ipc::channel().unwrap();

        let session = self.session().unwrap();

        // Step 2. If session's current top-level browsing context is no longer open,
        // return error with error code no such window.
        self.verify_top_level_browsing_context_is_open(session.webview_id)?;

        // Step 3. Handle any user prompt.
        self.handle_any_user_prompts(session.webview_id)?;

        let cmd_msg = WebDriverCommandMsg::NewWebView(sender, self.load_status_sender.clone());
        // Step 5. Create a new top-level browsing context by running the window open steps.
        // This MUST be done without invoking the focusing steps.
        self.send_message_to_embedder(cmd_msg)?;

        let mut handle = self.session.as_ref().unwrap().id.to_string();
        if let Ok(new_webview_id) = receiver.recv() {
            let session = self.session_mut().unwrap();
            let new_handle = Uuid::new_v4().to_string();
            handle = new_handle.clone();
            session.window_handles.insert(new_webview_id, new_handle);
        }

        let _ = self.wait_for_navigation_to_complete();

        Ok(WebDriverResponse::NewWindow(NewWindowResponse {
            handle,
            typ: "tab".to_string(),
        }))
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
                let webview_id = self.session()?.webview_id;
                // Step 1. If session's current top-level browsing context is no longer open,
                // return error with error code no such window.
                self.verify_top_level_browsing_context_is_open(webview_id)?;
                // Step 2. Try to handle any user prompts with session.
                self.handle_any_user_prompts(webview_id)?;
                // Step 3. Set the current browsing context with session and
                // session's current top-level browsing context.
                self.session_mut()?.browsing_context_id = BrowsingContextId::from(webview_id);
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
        let webview_id = self.session()?.webview_id;
        let browsing_context = self.session()?.browsing_context_id;
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
        match wait_for_script_response(receiver)? {
            Ok(browsing_context_id) => {
                self.session_mut()?.browsing_context_id = browsing_context_id;
                Ok(WebDriverResponse::Void)
            },
            Err(error) => Err(WebDriverError::new(error, "")),
        }
    }

    // https://w3c.github.io/webdriver/#switch-to-window
    fn handle_switch_to_window(
        &mut self,
        parameters: &SwitchToWindowParameters,
    ) -> WebDriverResult<WebDriverResponse> {
        let session = self.session_mut().unwrap();
        if session.id.to_string() == parameters.handle {
            // There's only one main window, so there's nothing to do here.
            Ok(WebDriverResponse::Void)
        } else if let Some((webview_id, _)) = session
            .window_handles
            .iter()
            .find(|(_k, v)| **v == parameters.handle)
        {
            let webview_id = *webview_id;
            session.webview_id = webview_id;
            session.browsing_context_id = BrowsingContextId::from(webview_id);
            let (sender, receiver) = ipc::channel().unwrap();
            let msg = WebDriverCommandMsg::FocusWebView(webview_id, sender);
            self.send_message_to_embedder(msg)?;
            if wait_for_script_response(receiver)? {
                debug!("Focus new webview successfully");
            } else {
                debug!("Focus new webview failed, it may not exist anymore");
            }
            Ok(WebDriverResponse::Void)
        } else {
            Err(WebDriverError::new(
                ErrorStatus::NoSuchWindow,
                "No such window",
            ))
        }
    }

    fn switch_to_frame(
        &mut self,
        frame_id: WebDriverFrameId,
    ) -> WebDriverResult<WebDriverResponse> {
        let (sender, receiver) = ipc::channel().unwrap();
        let cmd = WebDriverScriptCommand::GetBrowsingContextId(frame_id, sender);
        self.browsing_context_script_command(cmd, VerifyBrowsingContextIsOpen::Yes)?;
        self.handle_any_user_prompts(self.session()?.webview_id)?;

        match wait_for_script_response(receiver)? {
            Ok(browsing_context_id) => {
                self.session_mut()?.browsing_context_id = browsing_context_id;
                Ok(WebDriverResponse::Void)
            },
            Err(error) => Err(WebDriverError::new(error, "")),
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
        self.verify_browsing_context_is_open(self.session()?.browsing_context_id)?;

        // Step 6. Handle any user prompt.
        self.handle_any_user_prompts(self.session()?.webview_id)?;

        let (sender, receiver) = ipc::channel().unwrap();
        match parameters.using {
            LocatorStrategy::CSSSelector => {
                let cmd = WebDriverScriptCommand::FindElementsCSSSelector(
                    parameters.value.clone(),
                    sender,
                );
                self.browsing_context_script_command(cmd, VerifyBrowsingContextIsOpen::No)?;
            },
            LocatorStrategy::LinkText | LocatorStrategy::PartialLinkText => {
                let cmd = WebDriverScriptCommand::FindElementsLinkText(
                    parameters.value.clone(),
                    parameters.using == LocatorStrategy::PartialLinkText,
                    sender,
                );
                self.browsing_context_script_command(cmd, VerifyBrowsingContextIsOpen::No)?;
            },
            LocatorStrategy::TagName => {
                let cmd =
                    WebDriverScriptCommand::FindElementsTagName(parameters.value.clone(), sender);
                self.browsing_context_script_command(cmd, VerifyBrowsingContextIsOpen::No)?;
            },
            LocatorStrategy::XPath => {
                let cmd = WebDriverScriptCommand::FindElementsXpathSelector(
                    parameters.value.clone(),
                    sender,
                );
                self.browsing_context_script_command(cmd, VerifyBrowsingContextIsOpen::No)?;
            },
        }

        match wait_for_script_response(receiver)? {
            Ok(value) => {
                let resp_value: Vec<WebElement> = value.into_iter().map(WebElement).collect();
                Ok(WebDriverResponse::Generic(ValueResponse(
                    serde_json::to_value(resp_value)?,
                )))
            },
            Err(error) => Err(WebDriverError::new(error, "")),
        }
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
        self.verify_browsing_context_is_open(self.session()?.browsing_context_id)?;

        // Step 6. Handle any user prompt.
        self.handle_any_user_prompts(self.session()?.webview_id)?;

        let (sender, receiver) = ipc::channel().unwrap();

        match parameters.using {
            LocatorStrategy::CSSSelector => {
                let cmd = WebDriverScriptCommand::FindElementElementsCSSSelector(
                    parameters.value.clone(),
                    element.to_string(),
                    sender,
                );
                self.browsing_context_script_command(cmd, VerifyBrowsingContextIsOpen::No)?;
            },
            LocatorStrategy::LinkText | LocatorStrategy::PartialLinkText => {
                let cmd = WebDriverScriptCommand::FindElementElementsLinkText(
                    parameters.value.clone(),
                    element.to_string(),
                    parameters.using == LocatorStrategy::PartialLinkText,
                    sender,
                );
                self.browsing_context_script_command(cmd, VerifyBrowsingContextIsOpen::No)?;
            },
            LocatorStrategy::TagName => {
                let cmd = WebDriverScriptCommand::FindElementElementsTagName(
                    parameters.value.clone(),
                    element.to_string(),
                    sender,
                );
                self.browsing_context_script_command(cmd, VerifyBrowsingContextIsOpen::No)?;
            },
            LocatorStrategy::XPath => {
                let cmd = WebDriverScriptCommand::FindElementElementsXPathSelector(
                    parameters.value.clone(),
                    element.to_string(),
                    sender,
                );
                self.browsing_context_script_command(cmd, VerifyBrowsingContextIsOpen::No)?;
            },
        }

        match wait_for_script_response(receiver)? {
            Ok(value) => {
                let resp_value: Vec<Value> = value
                    .into_iter()
                    .map(|x| serde_json::to_value(WebElement(x)).unwrap())
                    .collect();
                Ok(WebDriverResponse::Generic(ValueResponse(
                    serde_json::to_value(resp_value)?,
                )))
            },
            Err(error) => Err(WebDriverError::new(error, "")),
        }
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
        self.verify_browsing_context_is_open(self.session()?.browsing_context_id)?;

        // Step 6. Handle any user prompt.
        self.handle_any_user_prompts(self.session()?.webview_id)?;

        let (sender, receiver) = ipc::channel().unwrap();

        match parameters.using {
            LocatorStrategy::CSSSelector => {
                let cmd = WebDriverScriptCommand::FindShadowElementsCSSSelector(
                    parameters.value.clone(),
                    shadow_root.to_string(),
                    sender,
                );
                self.browsing_context_script_command(cmd, VerifyBrowsingContextIsOpen::No)?;
            },
            LocatorStrategy::LinkText | LocatorStrategy::PartialLinkText => {
                let cmd = WebDriverScriptCommand::FindShadowElementsLinkText(
                    parameters.value.clone(),
                    shadow_root.to_string(),
                    parameters.using == LocatorStrategy::PartialLinkText,
                    sender,
                );
                self.browsing_context_script_command(cmd, VerifyBrowsingContextIsOpen::No)?;
            },
            LocatorStrategy::TagName => {
                let cmd = WebDriverScriptCommand::FindShadowElementsTagName(
                    parameters.value.clone(),
                    shadow_root.to_string(),
                    sender,
                );
                self.browsing_context_script_command(cmd, VerifyBrowsingContextIsOpen::No)?;
            },
            LocatorStrategy::XPath => {
                let cmd = WebDriverScriptCommand::FindShadowElementsXPathSelector(
                    parameters.value.clone(),
                    shadow_root.to_string(),
                    sender,
                );
                self.browsing_context_script_command(cmd, VerifyBrowsingContextIsOpen::No)?;
            },
        }

        match wait_for_script_response(receiver)? {
            Ok(value) => {
                let resp_value: Vec<Value> = value
                    .into_iter()
                    .map(|x| serde_json::to_value(WebElement(x)).unwrap())
                    .collect();
                Ok(WebDriverResponse::Generic(ValueResponse(
                    serde_json::to_value(resp_value)?,
                )))
            },
            Err(error) => Err(WebDriverError::new(error, "")),
        }
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
        self.verify_browsing_context_is_open(self.session()?.browsing_context_id)?;
        // Step 2. Handle any user prompt.
        self.handle_any_user_prompts(self.session()?.webview_id)?;
        let (sender, receiver) = ipc::channel().unwrap();
        let cmd = WebDriverScriptCommand::GetElementShadowRoot(element.to_string(), sender);
        self.browsing_context_script_command(cmd, VerifyBrowsingContextIsOpen::No)?;
        match wait_for_script_response(receiver)? {
            Ok(value) => {
                let Some(value) = value else {
                    return Err(WebDriverError::new(ErrorStatus::NoSuchShadowRoot, ""));
                };
                Ok(WebDriverResponse::Generic(ValueResponse(
                    serde_json::to_value(ShadowRoot(value))?,
                )))
            },
            Err(error) => Err(WebDriverError::new(error, "")),
        }
    }

    /// <https://w3c.github.io/webdriver/#get-element-rect>
    fn handle_element_rect(&self, element: &WebElement) -> WebDriverResult<WebDriverResponse> {
        // Step 1. If session's current browsing context is no longer open,
        // return error with error code no such window.
        self.verify_browsing_context_is_open(self.session()?.browsing_context_id)?;
        // Step 2. Handle any user prompt.
        self.handle_any_user_prompts(self.session()?.webview_id)?;
        let (sender, receiver) = ipc::channel().unwrap();
        let cmd = WebDriverScriptCommand::GetElementRect(element.to_string(), sender);
        self.browsing_context_script_command(cmd, VerifyBrowsingContextIsOpen::No)?;
        match wait_for_script_response(receiver)? {
            Ok(rect) => {
                let response = ElementRectResponse {
                    x: rect.origin.x,
                    y: rect.origin.y,
                    width: rect.size.width,
                    height: rect.size.height,
                };
                Ok(WebDriverResponse::ElementRect(response))
            },
            Err(error) => Err(WebDriverError::new(error, "")),
        }
    }

    /// <https://w3c.github.io/webdriver/#dfn-get-element-text>
    fn handle_element_text(&self, element: &WebElement) -> WebDriverResult<WebDriverResponse> {
        // Step 1. If session's current browsing context is no longer open,
        // return error with error code no such window.
        self.verify_browsing_context_is_open(self.session()?.browsing_context_id)?;
        // Step 2. Handle any user prompt.
        self.handle_any_user_prompts(self.session()?.webview_id)?;
        let (sender, receiver) = ipc::channel().unwrap();
        let cmd = WebDriverScriptCommand::GetElementText(element.to_string(), sender);
        self.browsing_context_script_command(cmd, VerifyBrowsingContextIsOpen::No)?;
        match wait_for_script_response(receiver)? {
            Ok(value) => Ok(WebDriverResponse::Generic(ValueResponse(
                serde_json::to_value(value)?,
            ))),
            Err(error) => Err(WebDriverError::new(error, "")),
        }
    }

    ///<https://w3c.github.io/webdriver/#get-active-element>
    fn handle_active_element(&self) -> WebDriverResult<WebDriverResponse> {
        // Step 1. If session's current browsing context is no longer open,
        // return error with error code no such window.
        self.verify_browsing_context_is_open(self.session()?.browsing_context_id)?;
        // Step 2. Handle any user prompt.
        self.handle_any_user_prompts(self.session()?.webview_id)?;
        let (sender, receiver) = ipc::channel().unwrap();
        let cmd = WebDriverScriptCommand::GetActiveElement(sender);
        self.browsing_context_script_command(cmd, VerifyBrowsingContextIsOpen::No)?;
        let value = wait_for_script_response(receiver)?
            .map(|x| serde_json::to_value(WebElement(x)).unwrap());
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
        self.verify_browsing_context_is_open(self.session()?.browsing_context_id)?;
        // Step 2. Handle any user prompt.
        self.handle_any_user_prompts(self.session()?.webview_id)?;
        let (sender, receiver) = ipc::channel().unwrap();
        let cmd = WebDriverScriptCommand::GetComputedRole(element.to_string(), sender);
        self.browsing_context_script_command(cmd, VerifyBrowsingContextIsOpen::No)?;
        match wait_for_script_response(receiver)? {
            Ok(value) => Ok(WebDriverResponse::Generic(ValueResponse(
                serde_json::to_value(value)?,
            ))),
            Err(error) => Err(WebDriverError::new(error, "")),
        }
    }

    /// <https://w3c.github.io/webdriver/#get-element-tag-name>
    fn handle_element_tag_name(&self, element: &WebElement) -> WebDriverResult<WebDriverResponse> {
        // Step 1. If session's current browsing context is no longer open,
        // return error with error code no such window.
        self.verify_browsing_context_is_open(self.session()?.browsing_context_id)?;
        // Step 2. Handle any user prompt.
        self.handle_any_user_prompts(self.session()?.webview_id)?;
        let (sender, receiver) = ipc::channel().unwrap();
        let cmd = WebDriverScriptCommand::GetElementTagName(element.to_string(), sender);
        self.browsing_context_script_command(cmd, VerifyBrowsingContextIsOpen::No)?;
        match wait_for_script_response(receiver)? {
            Ok(value) => Ok(WebDriverResponse::Generic(ValueResponse(
                serde_json::to_value(value)?,
            ))),
            Err(error) => Err(WebDriverError::new(error, "")),
        }
    }

    /// <https://w3c.github.io/webdriver/#get-element-attribute>
    fn handle_element_attribute(
        &self,
        element: &WebElement,
        name: &str,
    ) -> WebDriverResult<WebDriverResponse> {
        // Step 1. If session's current browsing context is no longer open,
        // return error with error code no such window.
        self.verify_browsing_context_is_open(self.session()?.browsing_context_id)?;
        // Step 2. Handle any user prompt.
        self.handle_any_user_prompts(self.session()?.webview_id)?;
        let (sender, receiver) = ipc::channel().unwrap();
        let cmd = WebDriverScriptCommand::GetElementAttribute(
            element.to_string(),
            name.to_owned(),
            sender,
        );
        self.browsing_context_script_command(cmd, VerifyBrowsingContextIsOpen::No)?;
        match wait_for_script_response(receiver)? {
            Ok(value) => Ok(WebDriverResponse::Generic(ValueResponse(
                serde_json::to_value(value)?,
            ))),
            Err(error) => Err(WebDriverError::new(error, "")),
        }
    }

    /// <https://w3c.github.io/webdriver/#get-element-property>
    fn handle_element_property(
        &self,
        element: &WebElement,
        name: &str,
    ) -> WebDriverResult<WebDriverResponse> {
        // Step 1. If session's current browsing context is no longer open,
        // return error with error code no such window.
        self.verify_browsing_context_is_open(self.session()?.browsing_context_id)?;
        // Step 2. Handle any user prompt.
        self.handle_any_user_prompts(self.session()?.webview_id)?;
        let (sender, receiver) = ipc::channel().unwrap();

        let cmd = WebDriverScriptCommand::GetElementProperty(
            element.to_string(),
            name.to_owned(),
            sender,
        );
        self.browsing_context_script_command(cmd, VerifyBrowsingContextIsOpen::No)?;

        match wait_for_script_response(receiver)? {
            Ok(value) => Ok(WebDriverResponse::Generic(ValueResponse(
                serde_json::to_value(SendableWebDriverJSValue(value))?,
            ))),
            Err(error) => Err(WebDriverError::new(error, "")),
        }
    }

    /// <https://w3c.github.io/webdriver/#get-element-css-value>
    fn handle_element_css(
        &self,
        element: &WebElement,
        name: &str,
    ) -> WebDriverResult<WebDriverResponse> {
        // Step 1. If session's current browsing context is no longer open,
        // return error with error code no such window.
        self.verify_browsing_context_is_open(self.session()?.browsing_context_id)?;
        // Step 2. Handle any user prompt.
        self.handle_any_user_prompts(self.session()?.webview_id)?;
        let (sender, receiver) = ipc::channel().unwrap();
        let cmd =
            WebDriverScriptCommand::GetElementCSS(element.to_string(), name.to_owned(), sender);
        self.browsing_context_script_command(cmd, VerifyBrowsingContextIsOpen::No)?;
        match wait_for_script_response(receiver)? {
            Ok(value) => Ok(WebDriverResponse::Generic(ValueResponse(
                serde_json::to_value(value)?,
            ))),
            Err(error) => Err(WebDriverError::new(error, "")),
        }
    }

    /// <https://w3c.github.io/webdriver/#get-all-cookies>
    fn handle_get_cookies(&self) -> WebDriverResult<WebDriverResponse> {
        // Step 1. If session's current browsing context is no longer open,
        // return error with error code no such window.
        self.verify_browsing_context_is_open(self.session()?.browsing_context_id)?;
        // Step 2. Handle any user prompt.
        self.handle_any_user_prompts(self.session()?.webview_id)?;
        let (sender, receiver) = ipc::channel().unwrap();
        let cmd = WebDriverScriptCommand::GetCookies(sender);
        self.browsing_context_script_command(cmd, VerifyBrowsingContextIsOpen::No)?;
        let cookies = match wait_for_script_response(receiver)? {
            Ok(cookies) => cookies,
            Err(error) => return Err(WebDriverError::new(error, "")),
        };
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
        self.verify_browsing_context_is_open(self.session()?.browsing_context_id)?;
        // Step 2. Handle any user prompt.
        self.handle_any_user_prompts(self.session()?.webview_id)?;
        let (sender, receiver) = ipc::channel().unwrap();
        let cmd = WebDriverScriptCommand::GetCookie(name, sender);
        self.browsing_context_script_command(cmd, VerifyBrowsingContextIsOpen::No)?;
        let cookies = match wait_for_script_response(receiver)? {
            Ok(cookies) => cookies,
            Err(error) => return Err(WebDriverError::new(error, "")),
        };
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
        self.verify_browsing_context_is_open(self.session()?.browsing_context_id)?;
        // Step 2. Handle any user prompt.
        self.handle_any_user_prompts(self.session()?.webview_id)?;
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
        match wait_for_script_response(receiver)? {
            Ok(_) => Ok(WebDriverResponse::Void),
            Err(error) => Err(WebDriverError::new(error, "")),
        }
    }

    /// <https://w3c.github.io/webdriver/#delete-cookie>
    fn handle_delete_cookie(&self, name: String) -> WebDriverResult<WebDriverResponse> {
        // Step 1. If session's current browsing context is no longer open,
        // return error with error code no such window.
        self.verify_browsing_context_is_open(self.session()?.browsing_context_id)?;
        // Step 2. Handle any user prompt.
        self.handle_any_user_prompts(self.session()?.webview_id)?;
        let (sender, receiver) = ipc::channel().unwrap();
        let cmd = WebDriverScriptCommand::DeleteCookie(name, sender);
        self.browsing_context_script_command(cmd, VerifyBrowsingContextIsOpen::No)?;
        match wait_for_script_response(receiver)? {
            Ok(_) => Ok(WebDriverResponse::Void),
            Err(error) => Err(WebDriverError::new(error, "")),
        }
    }

    /// <https://w3c.github.io/webdriver/#delete-all-cookies>
    fn handle_delete_cookies(&self) -> WebDriverResult<WebDriverResponse> {
        // Step 1. If session's current browsing context is no longer open,
        // return error with error code no such window.
        self.verify_browsing_context_is_open(self.session()?.browsing_context_id)?;
        // Step 2. Handle any user prompt.
        self.handle_any_user_prompts(self.session()?.webview_id)?;
        let (sender, receiver) = ipc::channel().unwrap();
        let cmd = WebDriverScriptCommand::DeleteCookies(sender);
        self.browsing_context_script_command(cmd, VerifyBrowsingContextIsOpen::Yes)?;
        match wait_for_script_response(receiver)? {
            Ok(_) => Ok(WebDriverResponse::Void),
            Err(error) => Err(WebDriverError::new(error, "")),
        }
    }

    fn handle_get_timeouts(&mut self) -> WebDriverResult<WebDriverResponse> {
        let session = self
            .session
            .as_ref()
            .ok_or(WebDriverError::new(ErrorStatus::SessionNotCreated, ""))?;

        let timeouts = TimeoutsResponse {
            script: session.script_timeout,
            page_load: session.load_timeout,
            implicit: session.implicit_wait_timeout,
        };

        Ok(WebDriverResponse::Timeouts(timeouts))
    }

    fn handle_set_timeouts(
        &mut self,
        parameters: &TimeoutsParameters,
    ) -> WebDriverResult<WebDriverResponse> {
        let session = self
            .session
            .as_mut()
            .ok_or(WebDriverError::new(ErrorStatus::SessionNotCreated, ""))?;

        if let Some(timeout) = parameters.script {
            session.script_timeout = timeout;
        }
        if let Some(timeout) = parameters.page_load {
            session.load_timeout = timeout
        }
        if let Some(timeout) = parameters.implicit {
            session.implicit_wait_timeout = timeout
        }

        Ok(WebDriverResponse::Void)
    }

    /// <https://w3c.github.io/webdriver/#dfn-get-page-source>
    fn handle_get_page_source(&self) -> WebDriverResult<WebDriverResponse> {
        // Step 1. If session's current browsing context is no longer open,
        // return error with error code no such window.
        self.verify_browsing_context_is_open(self.session()?.browsing_context_id)?;
        // Step 2. Handle any user prompt.
        self.handle_any_user_prompts(self.session()?.webview_id)?;
        let (sender, receiver) = ipc::channel().unwrap();

        let cmd = WebDriverScriptCommand::GetPageSource(sender);
        self.browsing_context_script_command(cmd, VerifyBrowsingContextIsOpen::No)?;

        match wait_for_script_response(receiver)? {
            Ok(source) => Ok(WebDriverResponse::Generic(ValueResponse(
                serde_json::to_value(source)?,
            ))),
            Err(error) => Err(WebDriverError::new(error, "")),
        }
    }

    /// <https://w3c.github.io/webdriver/#perform-actions>
    fn handle_perform_actions(
        &mut self,
        parameters: ActionsParameters,
    ) -> WebDriverResult<WebDriverResponse> {
        let browsing_context = self.session()?.browsing_context_id;
        // Step 1. If session's current browsing context is no longer open,
        // return error with error code no such window.
        self.verify_browsing_context_is_open(browsing_context)?;

        // Step 2. Handle any user prompt.
        self.handle_any_user_prompts(self.session()?.webview_id)?;

        // Step 5. Let actions by tick be the result of trying to extract an action sequence
        let actions_by_tick = self.extract_an_action_sequence(parameters);

        // Step 6. Dispatch actions with current browsing context
        match self.dispatch_actions(actions_by_tick, browsing_context) {
            Ok(_) => Ok(WebDriverResponse::Void),
            Err(error) => Err(WebDriverError::new(error, "")),
        }
    }

    /// <https://w3c.github.io/webdriver/#dfn-release-actions>
    fn handle_release_actions(&mut self) -> WebDriverResult<WebDriverResponse> {
        let session = self.session()?;

        // Step 1. If session's current browsing context is no longer open,
        // return error with error code no such window.
        self.verify_browsing_context_is_open(session.browsing_context_id)?;

        // Step 2. User prompts. No user prompt implemented yet.
        self.handle_any_user_prompts(self.session()?.webview_id)?;

        // Skip: Step 3. We don't support "browsing context input state map" yet.

        // TODO: Step 4. Actions options are not used yet.

        // Step 5. Not needed because "In a session that is only a HTTP session
        // only one command can run at a time, so this will never block."

        // Step 6. Let undo actions be input cancel list in reverse order.

        let undo_actions = session
            .input_cancel_list
            .borrow_mut()
            .drain(..)
            .rev()
            .map(|(id, action_item)| HashMap::from([(id, action_item)]))
            .collect();
        // Step 7. Dispatch undo actions with current browsing context.
        if let Err(err) = self.dispatch_actions(undo_actions, session.browsing_context_id) {
            return Err(WebDriverError::new(err, "Failed to dispatch undo actions"));
        }

        // Step 8. Reset the input state of session's current top-level browsing context.
        session.input_state_table.borrow_mut().clear();

        Ok(WebDriverResponse::Void)
    }

    /// <https://w3c.github.io/webdriver/#dfn-execute-script>
    fn handle_execute_script(
        &self,
        parameters: &JavascriptCommandParameters,
    ) -> WebDriverResult<WebDriverResponse> {
        // Step 2. If session's current browsing context is no longer open,
        // return error with error code no such window.
        self.verify_browsing_context_is_open(self.session()?.browsing_context_id)?;
        // Step 3. Handle any user prompt.
        self.handle_any_user_prompts(self.session()?.webview_id)?;

        let func_body = &parameters.script;
        let args_string: Vec<_> = parameters
            .args
            .as_deref()
            .unwrap_or(&[])
            .iter()
            .map(webdriver_value_to_js_argument)
            .collect();

        // This is pretty ugly; we really want something that acts like
        // new Function() and then takes the resulting function and executes
        // it with a vec of arguments.
        let script = format!(
            "(function() {{ {}\n }})({})",
            func_body,
            args_string.join(", ")
        );
        debug!("{}", script);

        let (sender, receiver) = ipc::channel().unwrap();
        let cmd = WebDriverScriptCommand::ExecuteScript(script, sender);
        self.browsing_context_script_command(cmd, VerifyBrowsingContextIsOpen::No)?;
        let result = wait_for_script_response(receiver)?;
        self.postprocess_js_result(result)
    }

    fn handle_execute_async_script(
        &self,
        parameters: &JavascriptCommandParameters,
    ) -> WebDriverResult<WebDriverResponse> {
        // Step 2. If session's current browsing context is no longer open,
        // return error with error code no such window.
        self.verify_browsing_context_is_open(self.session()?.browsing_context_id)?;
        // Step 3. Handle any user prompt.
        self.handle_any_user_prompts(self.session()?.webview_id)?;

        let func_body = &parameters.script;
        let mut args_string: Vec<_> = parameters
            .args
            .as_deref()
            .unwrap_or(&[])
            .iter()
            .map(webdriver_value_to_js_argument)
            .collect();
        args_string.push("resolve".to_string());

        let timeout_script = if let Some(script_timeout) = self.session()?.script_timeout {
            format!("setTimeout(webdriverTimeout, {});", script_timeout)
        } else {
            "".into()
        };
        let script = format!(
            r#"(function() {{
              let webdriverPromise = new Promise(function(resolve, reject) {{
                  {}
                  (async function() {{
                    {}
                  }})({})
                    .then((v) => {{}}, (err) => reject(err))
              }})
              .then((v) => window.webdriverCallback(v), (r) => window.webdriverException(r))
              .catch((r) => window.webdriverException(r));
            }})();"#,
            timeout_script,
            func_body,
            args_string.join(", "),
        );
        debug!("{}", script);

        let (sender, receiver) = ipc::channel().unwrap();
        let cmd = WebDriverScriptCommand::ExecuteAsyncScript(script, sender);
        self.browsing_context_script_command(cmd, VerifyBrowsingContextIsOpen::No)?;
        let result = wait_for_script_response(receiver)?;
        self.postprocess_js_result(result)
    }

    fn postprocess_js_result(
        &self,
        result: WebDriverJSResult,
    ) -> WebDriverResult<WebDriverResponse> {
        match result {
            Ok(value) => Ok(WebDriverResponse::Generic(ValueResponse(
                serde_json::to_value(SendableWebDriverJSValue(value))?,
            ))),
            Err(WebDriverJSError::BrowsingContextNotFound) => Err(WebDriverError::new(
                ErrorStatus::NoSuchWindow,
                "Pipeline id not found in browsing context",
            )),
            Err(WebDriverJSError::JSException(_e)) => Err(WebDriverError::new(
                ErrorStatus::JavascriptError,
                "JS evaluation raised an exception",
            )),
            Err(WebDriverJSError::JSError) => Err(WebDriverError::new(
                ErrorStatus::JavascriptError,
                "JS evaluation raised an unknown exception",
            )),
            Err(WebDriverJSError::StaleElementReference) => Err(WebDriverError::new(
                ErrorStatus::StaleElementReference,
                "Stale element",
            )),
            Err(WebDriverJSError::Timeout) => {
                Err(WebDriverError::new(ErrorStatus::ScriptTimeout, ""))
            },
            Err(WebDriverJSError::UnknownType) => Err(WebDriverError::new(
                ErrorStatus::UnsupportedOperation,
                "Unsupported return type",
            )),
        }
    }

    /// <https://w3c.github.io/webdriver/#dfn-element-send-keys>
    fn handle_element_send_keys(
        &self,
        element: &WebElement,
        keys: &SendKeysParameters,
    ) -> WebDriverResult<WebDriverResponse> {
        // Step 3. If session's current browsing context is no longer open,
        // return error with error code no such window.
        self.verify_browsing_context_is_open(self.session()?.browsing_context_id)?;
        // Step 4. Handle any user prompt.
        self.handle_any_user_prompts(self.session()?.webview_id)?;

        let (sender, receiver) = ipc::channel().unwrap();
        let cmd = WebDriverScriptCommand::WillSendKeys(
            element.to_string(),
            keys.text.to_string(),
            self.session()?.strict_file_interactability,
            sender,
        );
        self.browsing_context_script_command(cmd, VerifyBrowsingContextIsOpen::No)?;

        // TODO: distinguish the not found and not focusable cases
        // File input and non-typeable form control should have
        // been handled in `webdriver_handler.rs`.
        if !wait_for_script_response(receiver)?.map_err(|error| WebDriverError::new(error, ""))? {
            return Ok(WebDriverResponse::Void);
        }

        let input_events = send_keys(&keys.text);

        // TODO: there's a race condition caused by the focus command and the
        // send keys command being two separate messages,
        // so the constellation may have changed state between them.
        // TODO: We should use `dispatch_action` to send the keys.
        let cmd_msg = WebDriverCommandMsg::SendKeys(self.session()?.webview_id, input_events);
        self.send_message_to_embedder(cmd_msg)?;

        Ok(WebDriverResponse::Void)
    }

    /// <https://w3c.github.io/webdriver/#element-clear>
    fn handle_element_clear(&self, element: &WebElement) -> WebDriverResult<WebDriverResponse> {
        // Step 1. If session's current browsing context is no longer open,
        // return ErrorStatus::NoSuchWindow.
        self.verify_browsing_context_is_open(self.session()?.browsing_context_id)?;

        // Step 2. Try to handle any user prompt.
        self.handle_any_user_prompts(self.session()?.webview_id)?;

        // Step 3-11 handled in script thread.
        let (sender, receiver) = ipc::channel().unwrap();
        let cmd = WebDriverScriptCommand::ElementClear(element.to_string(), sender);
        self.browsing_context_script_command(cmd, VerifyBrowsingContextIsOpen::No)?;

        match wait_for_script_response(receiver)? {
            Ok(_) => Ok(WebDriverResponse::Void),
            Err(error) => Err(WebDriverError::new(error, "")),
        }
    }

    /// <https://w3c.github.io/webdriver/#element-click>
    fn handle_element_click(&mut self, element: &WebElement) -> WebDriverResult<WebDriverResponse> {
        // Step 1. If session's current browsing context is no longer open,
        // return error with error code no such window.
        let browsing_context_id = self.session()?.browsing_context_id;
        self.verify_browsing_context_is_open(browsing_context_id)?;

        // Step 2. Handle any user prompts.
        self.handle_any_user_prompts(self.session()?.webview_id)?;

        let (sender, receiver) = ipc::channel().unwrap();
        let webview_id = self.session()?.webview_id;
        let browsing_context_id = self.session()?.browsing_context_id;

        // Steps 1 - 7 + Step 8 for <option>
        let cmd = WebDriverScriptCommand::ElementClick(element.to_string(), sender);
        self.browsing_context_script_command(cmd, VerifyBrowsingContextIsOpen::No)?;

        match wait_for_script_response(receiver)? {
            Ok(element_id) => match element_id {
                Some(element_id) => {
                    // Load status sender should be set up before we dispatch actions
                    self.send_message_to_embedder(WebDriverCommandMsg::AddLoadStatusSender(
                        webview_id,
                        self.load_status_sender.clone(),
                    ))?;

                    self.perform_element_click(element_id)?;

                    // Step 11. Try to wait for navigation to complete with session.
                    // The most reliable way to try to wait for a potential navigation
                    // which is caused by element click to check with script thread
                    let (sender, receiver) = ipc::channel().unwrap();
                    self.send_message_to_embedder(WebDriverCommandMsg::ScriptCommand(
                        browsing_context_id,
                        WebDriverScriptCommand::IsDocumentReadyStateComplete(sender),
                    ))?;

                    if wait_for_script_response(receiver)? {
                        self.load_status_receiver.recv().map_err(|_| {
                            WebDriverError::new(
                                ErrorStatus::UnknownError,
                                "Failed to receive load status",
                            )
                        })?;
                    } else {
                        self.send_message_to_embedder(
                            WebDriverCommandMsg::RemoveLoadStatusSender(webview_id),
                        )?;
                    }

                    // Step 13
                    Ok(WebDriverResponse::Void)
                },
                // Step 13
                None => Ok(WebDriverResponse::Void),
            },
            Err(error) => Err(WebDriverError::new(error, "")),
        }
    }

    fn perform_element_click(&mut self, element: String) -> WebDriverResult<WebDriverResponse> {
        // Step 8 for elements other than <option>
        let id = Uuid::new_v4().to_string();

        // Step 8.1
        self.session_mut()?.input_state_table.borrow_mut().insert(
            id.clone(),
            InputSourceState::Pointer(PointerInputState::new(PointerType::Mouse)),
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
        let actions_by_tick = self.actions_by_tick_from_sequence(vec![action_sequence]);
        if let Err(e) = self.dispatch_actions(actions_by_tick, self.session()?.browsing_context_id)
        {
            log::error!("handle_element_click: dispatch_actions failed: {:?}", e);
        }

        // Step 8.17 Remove an input source with input state and input id.
        self.session_mut()?
            .input_state_table
            .borrow_mut()
            .remove(&id);

        Ok(WebDriverResponse::Void)
    }

    fn take_screenshot(&self, rect: Option<Rect<f32, CSSPixel>>) -> WebDriverResult<String> {
        // Step 1. If session's current top-level browsing context is no longer open,
        // return error with error code no such window.
        let webview_id = self.session()?.webview_id;
        self.verify_top_level_browsing_context_is_open(webview_id)?;

        self.handle_any_user_prompts(webview_id)?;

        let mut img = None;

        let interval = 1000;
        let iterations = 30000 / interval;

        for _ in 0..iterations {
            let (sender, receiver) = ipc::channel().unwrap();

            self.send_message_to_embedder(WebDriverCommandMsg::TakeScreenshot(
                webview_id, rect, sender,
            ))?;

            if let Some(x) = wait_for_script_response(receiver)? {
                img = Some(x);
                break;
            };

            thread::sleep(Duration::from_millis(interval));
        }

        let img = match img {
            Some(img) => img,
            None => {
                return Err(WebDriverError::new(
                    ErrorStatus::Timeout,
                    "Taking screenshot timed out",
                ));
            },
        };

        // The compositor always sends RGBA pixels.
        assert_eq!(
            img.format,
            PixelFormat::RGBA8,
            "Unexpected screenshot pixel format"
        );

        let rgb = RgbaImage::from_raw(
            img.metadata.width,
            img.metadata.height,
            img.first_frame().bytes.to_vec(),
        )
        .unwrap();
        let mut png_data = Cursor::new(Vec::new());
        DynamicImage::ImageRgba8(rgb)
            .write_to(&mut png_data, ImageFormat::Png)
            .unwrap();

        Ok(base64::engine::general_purpose::STANDARD.encode(png_data.get_ref()))
    }

    fn handle_take_screenshot(&self) -> WebDriverResult<WebDriverResponse> {
        let encoded = self.take_screenshot(None)?;

        Ok(WebDriverResponse::Generic(ValueResponse(
            serde_json::to_value(encoded)?,
        )))
    }

    fn handle_take_element_screenshot(
        &self,
        element: &WebElement,
    ) -> WebDriverResult<WebDriverResponse> {
        let (sender, receiver) = ipc::channel().unwrap();

        let cmd = WebDriverScriptCommand::GetBoundingClientRect(element.to_string(), sender);
        self.browsing_context_script_command(cmd, VerifyBrowsingContextIsOpen::Yes)?;

        match wait_for_script_response(receiver)? {
            Ok(rect) => {
                let encoded = self.take_screenshot(Some(Rect::from_untyped(&rect)))?;

                Ok(WebDriverResponse::Generic(ValueResponse(
                    serde_json::to_value(encoded)?,
                )))
            },
            Err(error) => Err(WebDriverError::new(error, "Element not found")),
        }
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

    fn verify_top_level_browsing_context_is_open(
        &self,
        webview_id: WebViewId,
    ) -> Result<(), WebDriverError> {
        let (sender, receiver) = ipc::channel().unwrap();
        self.send_message_to_embedder(WebDriverCommandMsg::IsWebViewOpen(webview_id, sender))?;
        if !receiver.recv().unwrap_or(false) {
            Err(WebDriverError::new(
                ErrorStatus::NoSuchWindow,
                "No such window",
            ))
        } else {
            Ok(())
        }
    }

    fn verify_browsing_context_is_open(
        &self,
        browsing_context_id: BrowsingContextId,
    ) -> Result<(), WebDriverError> {
        let (sender, receiver) = ipc::channel().unwrap();
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
}

impl WebDriverHandler<ServoExtensionRoute> for Handler {
    fn handle_command(
        &mut self,
        _session: &Option<Session>,
        msg: WebDriverMessage<ServoExtensionRoute>,
    ) -> WebDriverResult<WebDriverResponse> {
        info!("{:?}", msg.command);

        // Unless we are trying to create a new session, we need to ensure that a
        // session has previously been created
        match msg.command {
            WebDriverCommand::NewSession(_) | WebDriverCommand::Status => {},
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
            WebDriverCommand::ExecuteScript(ref x) => self.handle_execute_script(x),
            WebDriverCommand::ExecuteAsyncScript(ref x) => self.handle_execute_async_script(x),
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
            WebDriverCommand::Extension(ref extension) => match *extension {
                ServoExtensionCommand::GetPrefs(ref x) => self.handle_get_prefs(x),
                ServoExtensionCommand::SetPrefs(ref x) => self.handle_set_prefs(x),
                ServoExtensionCommand::ResetPrefs(ref x) => self.handle_reset_prefs(x),
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

/// <https://w3c.github.io/webdriver/#dfn-web-element-identifier>
const ELEMENT_IDENTIFIER: &str = "element-6066-11e4-a52e-4f735466cecf";
/// <https://w3c.github.io/webdriver/#dfn-web-frame-identifier>
const FRAME_IDENTIFIER: &str = "frame-075b-4da1-b6ba-e579c2d3230a";
/// <https://w3c.github.io/webdriver/#dfn-web-window-identifier>
const WINDOW_IDENTIFIER: &str = "window-fcc6-11e5-b4f8-330a88ab9d7f";
/// <https://w3c.github.io/webdriver/#dfn-shadow-root-identifier>
const SHADOW_ROOT_IDENTIFIER: &str = "shadow-6066-11e4-a52e-4f735466cecf";

fn webdriver_value_to_js_argument(v: &Value) -> String {
    match v {
        Value::String(s) => format!("\"{}\"", s),
        Value::Null => "null".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::Array(list) => {
            let elems = list
                .iter()
                .map(|v| webdriver_value_to_js_argument(v).to_string())
                .collect::<Vec<_>>();
            format!("[{}]", elems.join(", "))
        },
        Value::Object(map) => {
            let key = map.keys().next().map(String::as_str);
            match (key, map.values().next()) {
                (Some(ELEMENT_IDENTIFIER), Some(id)) => {
                    return format!("window.webdriverElement({})", id);
                },
                (Some(FRAME_IDENTIFIER), Some(id)) => {
                    return format!("window.webdriverFrame({})", id);
                },
                (Some(WINDOW_IDENTIFIER), Some(id)) => {
                    return format!("window.webdriverWindow({})", id);
                },
                (Some(SHADOW_ROOT_IDENTIFIER), Some(id)) => {
                    return format!("window.webdriverShadowRoot({})", id);
                },
                _ => {},
            }
            let elems = map
                .iter()
                .map(|(k, v)| format!("{}: {}", k, webdriver_value_to_js_argument(v)))
                .collect::<Vec<_>>();
            format!("{{{}}}", elems.join(", "))
        },
    }
}

// TODO: This waits for not only the script response
// need to make another name
fn wait_for_script_response<T>(receiver: IpcReceiver<T>) -> Result<T, WebDriverError>
where
    T: for<'de> Deserialize<'de> + Serialize,
{
    receiver
        .recv()
        .map_err(|_| WebDriverError::new(ErrorStatus::NoSuchWindow, ""))
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
