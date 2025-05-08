/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![crate_name = "webdriver_server"]
#![crate_type = "rlib"]
#![deny(unsafe_code)]

mod actions;
mod capabilities;

use std::borrow::ToOwned;
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap, VecDeque};
use std::io::Cursor;
use std::net::{SocketAddr, SocketAddrV4};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;
use std::{env, fmt, process, thread};

use base::id::{BrowsingContextId, WebViewId};
use base64::Engine;
use capabilities::ServoCapabilities;
use constellation_traits::{EmbedderToConstellationMessage, TraversalDirection};
use cookie::{CookieBuilder, Expiration};
use crossbeam_channel::{Receiver, Sender, after, select, unbounded};
use embedder_traits::{
    MouseButton, WebDriverCommandMsg, WebDriverCommandResponse, WebDriverCookieError,
    WebDriverFrameId, WebDriverJSError, WebDriverJSResult, WebDriverJSValue, WebDriverLoadStatus,
    WebDriverMessageId, WebDriverScriptCommand,
};
use euclid::{Rect, Size2D};
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
use servo_url::ServoUrl;
use style_traits::CSSPixel;
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
use webdriver::common::{Cookie, Date, LocatorStrategy, Parameters, WebElement};
use webdriver::error::{ErrorStatus, WebDriverError, WebDriverResult};
use webdriver::httpapi::WebDriverExtensionRoute;
use webdriver::response::{
    CloseWindowResponse, CookieResponse, CookiesResponse, ElementRectResponse, NewSessionResponse,
    NewWindowResponse, TimeoutsResponse, ValueResponse, WebDriverResponse, WindowRectResponse,
};
use webdriver::server::{self, Session, SessionTeardownKind, WebDriverHandler};

use crate::actions::{InputSourceState, PointerInputState};

#[derive(Default)]
pub struct IdGenerator {
    counter: AtomicUsize,
}

impl IdGenerator {
    pub const fn new() -> Self {
        Self {
            counter: AtomicUsize::new(0),
        }
    }

    /// Returns a unique ID.
    pub fn next(&self) -> WebDriverMessageId {
        WebDriverMessageId(self.counter.fetch_add(1, Ordering::SeqCst))
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

pub fn start_server(port: u16, constellation_chan: Sender<EmbedderToConstellationMessage>) {
    let handler = Handler::new(constellation_chan);
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
pub struct WebDriverSession {
    id: Uuid,
    browsing_context_id: BrowsingContextId,
    webview_id: WebViewId,

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

    unhandled_prompt_behavior: String,

    // https://w3c.github.io/webdriver/#dfn-input-state-table
    input_state_table: RefCell<HashMap<String, InputSourceState>>,

    // https://w3c.github.io/webdriver/#dfn-input-cancel-list
    input_cancel_list: RefCell<Vec<ActionSequence>>,

    // https://w3c.github.io/webdriver/#dfn-actions-queue
    actions_queue: RefCell<VecDeque<WebDriverMessageId>>,
}

impl WebDriverSession {
    pub fn new(browsing_context_id: BrowsingContextId, webview_id: WebViewId) -> WebDriverSession {
        let mut window_handles = HashMap::new();
        let handle = Uuid::new_v4().to_string();
        window_handles.insert(webview_id, handle);

        WebDriverSession {
            id: Uuid::new_v4(),
            browsing_context_id,
            webview_id,

            window_handles,

            script_timeout: Some(30_000),
            load_timeout: 300_000,
            implicit_wait_timeout: 0,

            page_loading_strategy: "normal".to_string(),
            strict_file_interactability: false,
            unhandled_prompt_behavior: "dismiss and notify".to_string(),

            input_state_table: RefCell::new(HashMap::new()),
            input_cancel_list: RefCell::new(Vec::new()),
            actions_queue: RefCell::new(VecDeque::new()),
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

    /// The channel for sending Webdriver messages to the constellation.
    constellation_chan: Sender<EmbedderToConstellationMessage>,

    /// The IPC sender which we can clone and pass along to the constellation
    constellation_sender: IpcSender<WebDriverCommandResponse>,

    /// Receiver notification from the constellation when a command is completed
    constellation_receiver: IpcReceiver<WebDriverCommandResponse>,

    id_generator: IdGenerator,

    resize_timeout: u32,
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

impl Handler {
    pub fn new(constellation_chan: Sender<EmbedderToConstellationMessage>) -> Handler {
        // Create a pair of both an IPC and a threaded channel,
        // keep the IPC sender to clone and pass to the constellation for each load,
        // and keep a threaded receiver to block on an incoming load-status.
        // Pass the others to the IPC router so that IPC messages are forwarded to the threaded receiver.
        // We need to use the router because IPC does not come with a timeout on receive/select.
        let (load_status_sender, receiver) = ipc::channel().unwrap();
        let (sender, load_status_receiver) = unbounded();
        ROUTER.route_ipc_receiver_to_crossbeam_sender(receiver, sender);

        let (constellation_sender, constellation_receiver) = ipc::channel().unwrap();

        Handler {
            load_status_sender,
            load_status_receiver,
            session: None,
            constellation_chan,
            constellation_sender,
            constellation_receiver,
            id_generator: IdGenerator::new(),
            resize_timeout: 500,
        }
    }

    fn focus_webview_id(&self) -> WebDriverResult<WebViewId> {
        debug!("Getting focused context.");
        let interval = 20;
        let iterations = 30_000 / interval;
        let (sender, receiver) = ipc::channel().unwrap();

        for _ in 0..iterations {
            let msg =
                EmbedderToConstellationMessage::GetFocusTopLevelBrowsingContext(sender.clone());
            self.constellation_chan.send(msg).unwrap();
            // Wait until the document is ready before returning the top-level browsing context id.
            if let Some(x) = receiver.recv().unwrap() {
                debug!("Focused context is {}", x);
                return Ok(x);
            }
            thread::sleep(Duration::from_millis(interval));
        }

        debug!("Timed out getting focused context.");
        Err(WebDriverError::new(
            ErrorStatus::Timeout,
            "Failed to get window handle",
        ))
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

        if self.session.is_none() {
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
                            session.unhandled_prompt_behavior =
                                unhandled_prompt_behavior.to_string()
                        },
                        None => {
                            processed.insert(
                                "unhandledPromptBehavior".to_string(),
                                json!(session.unhandled_prompt_behavior),
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

                    let response =
                        NewSessionResponse::new(session.id.to_string(), Value::Object(processed));
                    self.session = Some(session);

                    Ok(WebDriverResponse::NewSession(response))
                },
                None => Ok(WebDriverResponse::Void),
            }
        } else {
            Err(WebDriverError::new(
                ErrorStatus::UnknownError,
                "Session already created",
            ))
        }
    }

    fn handle_delete_session(&mut self) -> WebDriverResult<WebDriverResponse> {
        self.session = None;
        Ok(WebDriverResponse::DeleteSession)
    }

    // https://w3c.github.io/webdriver/#status
    fn handle_status(&self) -> WebDriverResult<WebDriverResponse> {
        Ok(WebDriverResponse::Generic(ValueResponse(
            if self.session.is_none() {
                json!({ "ready": true, "message": "Ready for a new session" })
            } else {
                json!({ "ready": false, "message": "Not ready for a new session" })
            },
        )))
    }

    fn browsing_context_script_command(
        &self,
        cmd_msg: WebDriverScriptCommand,
    ) -> WebDriverResult<()> {
        let browsing_context_id = self.session()?.browsing_context_id;
        let msg = EmbedderToConstellationMessage::WebDriverCommand(
            WebDriverCommandMsg::ScriptCommand(browsing_context_id, cmd_msg),
        );
        self.constellation_chan.send(msg).unwrap();
        Ok(())
    }

    fn top_level_script_command(&self, cmd_msg: WebDriverScriptCommand) -> WebDriverResult<()> {
        let browsing_context_id = BrowsingContextId::from(self.session()?.webview_id);
        let msg = EmbedderToConstellationMessage::WebDriverCommand(
            WebDriverCommandMsg::ScriptCommand(browsing_context_id, cmd_msg),
        );
        self.constellation_chan.send(msg).unwrap();
        Ok(())
    }

    fn handle_get(&self, parameters: &GetParameters) -> WebDriverResult<WebDriverResponse> {
        let url = match ServoUrl::parse(&parameters.url[..]) {
            Ok(url) => url,
            Err(_) => {
                return Err(WebDriverError::new(
                    ErrorStatus::InvalidArgument,
                    "Invalid URL",
                ));
            },
        };

        let webview_id = self.session()?.webview_id;

        let cmd_msg =
            WebDriverCommandMsg::LoadUrl(webview_id, url, self.load_status_sender.clone());
        self.constellation_chan
            .send(EmbedderToConstellationMessage::WebDriverCommand(cmd_msg))
            .unwrap();

        self.wait_for_load()
    }

    fn wait_for_load(&self) -> WebDriverResult<WebDriverResponse> {
        debug!("waiting for load");
        let timeout = self.session()?.load_timeout;
        let result = select! {
            recv(self.load_status_receiver) -> _ => Ok(WebDriverResponse::Void),
            recv(after(Duration::from_millis(timeout))) -> _ => Err(
                WebDriverError::new(ErrorStatus::Timeout, "Load timed out")
            ),
        };
        debug!("finished waiting for load with {:?}", result);
        result
    }

    fn handle_current_url(&self) -> WebDriverResult<WebDriverResponse> {
        let (sender, receiver) = ipc::channel().unwrap();
        self.top_level_script_command(WebDriverScriptCommand::GetUrl(sender))?;

        let url = wait_for_script_response(receiver)?;

        Ok(WebDriverResponse::Generic(ValueResponse(
            serde_json::to_value(url.as_str())?,
        )))
    }

    fn handle_window_size(&self) -> WebDriverResult<WebDriverResponse> {
        let (sender, receiver) = ipc::channel().unwrap();
        let webview_id = self.session()?.webview_id;
        let cmd_msg = WebDriverCommandMsg::GetWindowSize(webview_id, sender);

        self.constellation_chan
            .send(EmbedderToConstellationMessage::WebDriverCommand(cmd_msg))
            .unwrap();

        let window_size = wait_for_script_response(receiver)?;
        let window_size_response = WindowRectResponse {
            x: 0,
            y: 0,
            width: window_size.width as i32,
            height: window_size.height as i32,
        };
        Ok(WebDriverResponse::WindowRect(window_size_response))
    }

    fn handle_set_window_size(
        &self,
        params: &WindowRectParameters,
    ) -> WebDriverResult<WebDriverResponse> {
        let (sender, receiver) = ipc::channel().unwrap();

        // We don't current allow modifying the window x/y positions, so we can just
        // return the current window rectangle.
        if params.width.is_none() || params.height.is_none() {
            return self.handle_window_size();
        }

        let width = params.width.unwrap_or(0);
        let height = params.height.unwrap_or(0);
        let size = Size2D::new(width as u32, height as u32);
        let webview_id = self.session()?.webview_id;
        let cmd_msg = WebDriverCommandMsg::SetWindowSize(webview_id, size.to_i32(), sender.clone());

        self.constellation_chan
            .send(EmbedderToConstellationMessage::WebDriverCommand(cmd_msg))
            .unwrap();

        let timeout = self.resize_timeout;
        let constellation_chan = self.constellation_chan.clone();
        thread::spawn(move || {
            // On timeout, we send a GetWindowSize message to the constellation,
            // which will give the current window size.
            thread::sleep(Duration::from_millis(timeout as u64));
            let cmd_msg = WebDriverCommandMsg::GetWindowSize(webview_id, sender);
            constellation_chan
                .send(EmbedderToConstellationMessage::WebDriverCommand(cmd_msg))
                .unwrap();
        });

        let window_size = wait_for_script_response(receiver)?;
        let window_size_response = WindowRectResponse {
            x: 0,
            y: 0,
            width: window_size.width as i32,
            height: window_size.height as i32,
        };
        Ok(WebDriverResponse::WindowRect(window_size_response))
    }

    fn handle_is_enabled(&self, element: &WebElement) -> WebDriverResult<WebDriverResponse> {
        let (sender, receiver) = ipc::channel().unwrap();

        self.top_level_script_command(WebDriverScriptCommand::IsEnabled(
            element.to_string(),
            sender,
        ))?;

        match wait_for_script_response(receiver)? {
            Ok(is_enabled) => Ok(WebDriverResponse::Generic(ValueResponse(
                serde_json::to_value(is_enabled)?,
            ))),
            Err(error) => Err(WebDriverError::new(error, "")),
        }
    }

    fn handle_is_selected(&self, element: &WebElement) -> WebDriverResult<WebDriverResponse> {
        let (sender, receiver) = ipc::channel().unwrap();

        self.top_level_script_command(WebDriverScriptCommand::IsSelected(
            element.to_string(),
            sender,
        ))?;

        match wait_for_script_response(receiver)? {
            Ok(is_selected) => Ok(WebDriverResponse::Generic(ValueResponse(
                serde_json::to_value(is_selected)?,
            ))),
            Err(error) => Err(WebDriverError::new(error, "")),
        }
    }

    fn handle_go_back(&self) -> WebDriverResult<WebDriverResponse> {
        let webview_id = self.session()?.webview_id;
        let direction = TraversalDirection::Back(1);
        let msg = EmbedderToConstellationMessage::TraverseHistory(webview_id, direction);
        self.constellation_chan.send(msg).unwrap();
        Ok(WebDriverResponse::Void)
    }

    fn handle_go_forward(&self) -> WebDriverResult<WebDriverResponse> {
        let webview_id = self.session()?.webview_id;
        let direction = TraversalDirection::Forward(1);
        let msg = EmbedderToConstellationMessage::TraverseHistory(webview_id, direction);
        self.constellation_chan.send(msg).unwrap();
        Ok(WebDriverResponse::Void)
    }

    fn handle_refresh(&self) -> WebDriverResult<WebDriverResponse> {
        let webview_id = self.session()?.webview_id;

        let cmd_msg = WebDriverCommandMsg::Refresh(webview_id, self.load_status_sender.clone());
        self.constellation_chan
            .send(EmbedderToConstellationMessage::WebDriverCommand(cmd_msg))
            .unwrap();

        self.wait_for_load()
    }

    fn handle_title(&self) -> WebDriverResult<WebDriverResponse> {
        let (sender, receiver) = ipc::channel().unwrap();

        self.top_level_script_command(WebDriverScriptCommand::GetTitle(sender))?;

        let value = wait_for_script_response(receiver)?;
        Ok(WebDriverResponse::Generic(ValueResponse(
            serde_json::to_value(value)?,
        )))
    }

    fn handle_window_handle(&self) -> WebDriverResult<WebDriverResponse> {
        let session = self.session.as_ref().unwrap();
        match session.window_handles.get(&session.webview_id) {
            Some(handle) => Ok(WebDriverResponse::Generic(ValueResponse(
                serde_json::to_value(handle)?,
            ))),
            None => Ok(WebDriverResponse::Void),
        }
    }

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

    fn handle_find_element(
        &self,
        parameters: &LocatorParameters,
    ) -> WebDriverResult<WebDriverResponse> {
        let (sender, receiver) = ipc::channel().unwrap();

        match parameters.using {
            LocatorStrategy::CSSSelector => {
                let cmd = WebDriverScriptCommand::FindElementCSS(parameters.value.clone(), sender);
                self.browsing_context_script_command(cmd)?;
            },
            LocatorStrategy::LinkText | LocatorStrategy::PartialLinkText => {
                let cmd = WebDriverScriptCommand::FindElementLinkText(
                    parameters.value.clone(),
                    parameters.using == LocatorStrategy::PartialLinkText,
                    sender,
                );
                self.browsing_context_script_command(cmd)?;
            },
            LocatorStrategy::TagName => {
                let cmd =
                    WebDriverScriptCommand::FindElementTagName(parameters.value.clone(), sender);
                self.browsing_context_script_command(cmd)?;
            },
            _ => {
                return Err(WebDriverError::new(
                    ErrorStatus::UnsupportedOperation,
                    "Unsupported locator strategy",
                ));
            },
        }

        match wait_for_script_response(receiver)? {
            Ok(value) => {
                let value_resp = serde_json::to_value(
                    value.map(|x| serde_json::to_value(WebElement(x)).unwrap()),
                )?;
                Ok(WebDriverResponse::Generic(ValueResponse(value_resp)))
            },
            Err(error) => Err(WebDriverError::new(error, "")),
        }
    }

    fn handle_close_window(&mut self) -> WebDriverResult<WebDriverResponse> {
        {
            let session = self.session_mut().unwrap();
            session.window_handles.remove(&session.webview_id);
            let cmd_msg = WebDriverCommandMsg::CloseWebView(session.webview_id);
            self.constellation_chan
                .send(EmbedderToConstellationMessage::WebDriverCommand(cmd_msg))
                .unwrap();
        }

        Ok(WebDriverResponse::CloseWindow(CloseWindowResponse(
            self.session()
                .unwrap()
                .window_handles
                .values()
                .cloned()
                .collect(),
        )))
    }

    fn handle_new_window(
        &mut self,
        _parameters: &NewWindowParameters,
    ) -> WebDriverResult<WebDriverResponse> {
        let (sender, receiver) = ipc::channel().unwrap();

        let session = self.session().unwrap();
        let cmd_msg = WebDriverCommandMsg::NewWebView(
            session.webview_id,
            sender,
            self.load_status_sender.clone(),
        );
        self.constellation_chan
            .send(EmbedderToConstellationMessage::WebDriverCommand(cmd_msg))
            .unwrap();

        let mut handle = self.session.as_ref().unwrap().id.to_string();
        if let Ok(new_webview_id) = receiver.recv() {
            let session = self.session_mut().unwrap();
            session.webview_id = new_webview_id;
            session.browsing_context_id = BrowsingContextId::from(new_webview_id);
            let new_handle = Uuid::new_v4().to_string();
            handle = new_handle.clone();
            session.window_handles.insert(new_webview_id, new_handle);
        }

        let _ = self.wait_for_load();

        Ok(WebDriverResponse::NewWindow(NewWindowResponse {
            handle,
            typ: "tab".to_string(),
        }))
    }

    fn handle_switch_to_frame(
        &mut self,
        parameters: &SwitchToFrameParameters,
    ) -> WebDriverResult<WebDriverResponse> {
        use webdriver::common::FrameId;
        let frame_id = match parameters.id {
            FrameId::Top => {
                let session = self.session_mut()?;
                session.browsing_context_id = BrowsingContextId::from(session.webview_id);
                return Ok(WebDriverResponse::Void);
            },
            FrameId::Short(ref x) => WebDriverFrameId::Short(*x),
            FrameId::Element(ref x) => WebDriverFrameId::Element(x.to_string()),
        };

        self.switch_to_frame(frame_id)
    }

    fn handle_switch_to_parent_frame(&mut self) -> WebDriverResult<WebDriverResponse> {
        self.switch_to_frame(WebDriverFrameId::Parent)
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

            let msg = EmbedderToConstellationMessage::FocusWebView(webview_id);
            self.constellation_chan.send(msg).unwrap();
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
        if let WebDriverFrameId::Short(_) = frame_id {
            return Err(WebDriverError::new(
                ErrorStatus::UnsupportedOperation,
                "Selecting frame by id not supported",
            ));
        }

        let (sender, receiver) = ipc::channel().unwrap();
        let cmd = WebDriverScriptCommand::GetBrowsingContextId(frame_id, sender);
        self.browsing_context_script_command(cmd)?;

        match wait_for_script_response(receiver)? {
            Ok(browsing_context_id) => {
                self.session_mut()?.browsing_context_id = browsing_context_id;
                Ok(WebDriverResponse::Void)
            },
            Err(error) => Err(WebDriverError::new(error, "")),
        }
    }

    // https://w3c.github.io/webdriver/#find-elements
    fn handle_find_elements(
        &self,
        parameters: &LocatorParameters,
    ) -> WebDriverResult<WebDriverResponse> {
        let (sender, receiver) = ipc::channel().unwrap();

        match parameters.using {
            LocatorStrategy::CSSSelector => {
                let cmd = WebDriverScriptCommand::FindElementsCSS(parameters.value.clone(), sender);
                self.browsing_context_script_command(cmd)?;
            },
            LocatorStrategy::LinkText | LocatorStrategy::PartialLinkText => {
                let cmd = WebDriverScriptCommand::FindElementsLinkText(
                    parameters.value.clone(),
                    parameters.using == LocatorStrategy::PartialLinkText,
                    sender,
                );
                self.browsing_context_script_command(cmd)?;
            },
            LocatorStrategy::TagName => {
                let cmd =
                    WebDriverScriptCommand::FindElementsTagName(parameters.value.clone(), sender);
                self.browsing_context_script_command(cmd)?;
            },
            _ => {
                return Err(WebDriverError::new(
                    ErrorStatus::UnsupportedOperation,
                    "Unsupported locator strategy",
                ));
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

    // https://w3c.github.io/webdriver/#find-element-from-element
    fn handle_find_element_element(
        &self,
        element: &WebElement,
        parameters: &LocatorParameters,
    ) -> WebDriverResult<WebDriverResponse> {
        let (sender, receiver) = ipc::channel().unwrap();

        match parameters.using {
            LocatorStrategy::CSSSelector => {
                let cmd = WebDriverScriptCommand::FindElementElementCSS(
                    parameters.value.clone(),
                    element.to_string(),
                    sender,
                );
                self.browsing_context_script_command(cmd)?;
            },
            LocatorStrategy::LinkText | LocatorStrategy::PartialLinkText => {
                let cmd = WebDriverScriptCommand::FindElementElementLinkText(
                    parameters.value.clone(),
                    element.to_string(),
                    parameters.using == LocatorStrategy::PartialLinkText,
                    sender,
                );
                self.browsing_context_script_command(cmd)?;
            },
            LocatorStrategy::TagName => {
                let cmd = WebDriverScriptCommand::FindElementElementTagName(
                    parameters.value.clone(),
                    element.to_string(),
                    sender,
                );
                self.browsing_context_script_command(cmd)?;
            },
            _ => {
                return Err(WebDriverError::new(
                    ErrorStatus::UnsupportedOperation,
                    "Unsupported locator strategy",
                ));
            },
        }

        match wait_for_script_response(receiver)? {
            Ok(value) => {
                let value_resp = serde_json::to_value(
                    value.map(|x| serde_json::to_value(WebElement(x)).unwrap()),
                )?;
                Ok(WebDriverResponse::Generic(ValueResponse(value_resp)))
            },
            Err(error) => Err(WebDriverError::new(error, "")),
        }
    }

    // https://w3c.github.io/webdriver/#find-elements-from-element
    fn handle_find_elements_from_element(
        &self,
        element: &WebElement,
        parameters: &LocatorParameters,
    ) -> WebDriverResult<WebDriverResponse> {
        let (sender, receiver) = ipc::channel().unwrap();

        match parameters.using {
            LocatorStrategy::CSSSelector => {
                let cmd = WebDriverScriptCommand::FindElementElementsCSS(
                    parameters.value.clone(),
                    element.to_string(),
                    sender,
                );
                self.browsing_context_script_command(cmd)?;
            },
            LocatorStrategy::LinkText | LocatorStrategy::PartialLinkText => {
                let cmd = WebDriverScriptCommand::FindElementElementsLinkText(
                    parameters.value.clone(),
                    element.to_string(),
                    parameters.using == LocatorStrategy::PartialLinkText,
                    sender,
                );
                self.browsing_context_script_command(cmd)?;
            },
            LocatorStrategy::TagName => {
                let cmd = WebDriverScriptCommand::FindElementElementsTagName(
                    parameters.value.clone(),
                    element.to_string(),
                    sender,
                );
                self.browsing_context_script_command(cmd)?;
            },
            _ => {
                return Err(WebDriverError::new(
                    ErrorStatus::UnsupportedOperation,
                    "Unsupported locator strategy",
                ));
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

    // https://w3c.github.io/webdriver/webdriver-spec.html#get-element-rect
    fn handle_element_rect(&self, element: &WebElement) -> WebDriverResult<WebDriverResponse> {
        let (sender, receiver) = ipc::channel().unwrap();
        let cmd = WebDriverScriptCommand::GetElementRect(element.to_string(), sender);
        self.browsing_context_script_command(cmd)?;
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

    fn handle_element_text(&self, element: &WebElement) -> WebDriverResult<WebDriverResponse> {
        let (sender, receiver) = ipc::channel().unwrap();
        let cmd = WebDriverScriptCommand::GetElementText(element.to_string(), sender);
        self.browsing_context_script_command(cmd)?;
        match wait_for_script_response(receiver)? {
            Ok(value) => Ok(WebDriverResponse::Generic(ValueResponse(
                serde_json::to_value(value)?,
            ))),
            Err(error) => Err(WebDriverError::new(error, "")),
        }
    }

    fn handle_active_element(&self) -> WebDriverResult<WebDriverResponse> {
        let (sender, receiver) = ipc::channel().unwrap();
        let cmd = WebDriverScriptCommand::GetActiveElement(sender);
        self.browsing_context_script_command(cmd)?;
        let value = wait_for_script_response(receiver)?
            .map(|x| serde_json::to_value(WebElement(x)).unwrap());
        Ok(WebDriverResponse::Generic(ValueResponse(
            serde_json::to_value(value)?,
        )))
    }

    fn handle_computed_role(&self, element: &WebElement) -> WebDriverResult<WebDriverResponse> {
        let (sender, receiver) = ipc::channel().unwrap();
        let cmd = WebDriverScriptCommand::GetComputedRole(element.to_string(), sender);
        self.browsing_context_script_command(cmd)?;
        match wait_for_script_response(receiver)? {
            Ok(value) => Ok(WebDriverResponse::Generic(ValueResponse(
                serde_json::to_value(value)?,
            ))),
            Err(error) => Err(WebDriverError::new(error, "")),
        }
    }

    fn handle_element_tag_name(&self, element: &WebElement) -> WebDriverResult<WebDriverResponse> {
        let (sender, receiver) = ipc::channel().unwrap();
        let cmd = WebDriverScriptCommand::GetElementTagName(element.to_string(), sender);
        self.browsing_context_script_command(cmd)?;
        match wait_for_script_response(receiver)? {
            Ok(value) => Ok(WebDriverResponse::Generic(ValueResponse(
                serde_json::to_value(value)?,
            ))),
            Err(error) => Err(WebDriverError::new(error, "")),
        }
    }

    fn handle_element_attribute(
        &self,
        element: &WebElement,
        name: &str,
    ) -> WebDriverResult<WebDriverResponse> {
        let (sender, receiver) = ipc::channel().unwrap();
        let cmd = WebDriverScriptCommand::GetElementAttribute(
            element.to_string(),
            name.to_owned(),
            sender,
        );
        self.browsing_context_script_command(cmd)?;
        match wait_for_script_response(receiver)? {
            Ok(value) => Ok(WebDriverResponse::Generic(ValueResponse(
                serde_json::to_value(value)?,
            ))),
            Err(error) => Err(WebDriverError::new(error, "")),
        }
    }

    fn handle_element_property(
        &self,
        element: &WebElement,
        name: &str,
    ) -> WebDriverResult<WebDriverResponse> {
        let (sender, receiver) = ipc::channel().unwrap();

        let cmd = WebDriverScriptCommand::GetElementProperty(
            element.to_string(),
            name.to_owned(),
            sender,
        );
        self.browsing_context_script_command(cmd)?;

        match wait_for_script_response(receiver)? {
            Ok(value) => Ok(WebDriverResponse::Generic(ValueResponse(
                serde_json::to_value(SendableWebDriverJSValue(value))?,
            ))),
            Err(error) => Err(WebDriverError::new(error, "")),
        }
    }

    fn handle_element_css(
        &self,
        element: &WebElement,
        name: &str,
    ) -> WebDriverResult<WebDriverResponse> {
        let (sender, receiver) = ipc::channel().unwrap();
        let cmd =
            WebDriverScriptCommand::GetElementCSS(element.to_string(), name.to_owned(), sender);
        self.browsing_context_script_command(cmd)?;
        match wait_for_script_response(receiver)? {
            Ok(value) => Ok(WebDriverResponse::Generic(ValueResponse(
                serde_json::to_value(value)?,
            ))),
            Err(error) => Err(WebDriverError::new(error, "")),
        }
    }

    fn handle_get_cookies(&self) -> WebDriverResult<WebDriverResponse> {
        let (sender, receiver) = ipc::channel().unwrap();
        let cmd = WebDriverScriptCommand::GetCookies(sender);
        self.browsing_context_script_command(cmd)?;
        let cookies = wait_for_script_response(receiver)?;
        let response = cookies
            .into_iter()
            .map(|cookie| cookie_msg_to_cookie(cookie.into_inner()))
            .collect::<Vec<Cookie>>();
        Ok(WebDriverResponse::Cookies(CookiesResponse(response)))
    }

    fn handle_get_cookie(&self, name: String) -> WebDriverResult<WebDriverResponse> {
        let (sender, receiver) = ipc::channel().unwrap();
        let cmd = WebDriverScriptCommand::GetCookie(name, sender);
        self.browsing_context_script_command(cmd)?;
        let cookies = wait_for_script_response(receiver)?;
        let Some(response) = cookies
            .into_iter()
            .map(|cookie| cookie_msg_to_cookie(cookie.into_inner()))
            .next()
        else {
            return Err(WebDriverError::new(ErrorStatus::NoSuchCookie, ""));
        };
        Ok(WebDriverResponse::Cookie(CookieResponse(response)))
    }

    fn handle_add_cookie(
        &self,
        params: &AddCookieParameters,
    ) -> WebDriverResult<WebDriverResponse> {
        let (sender, receiver) = ipc::channel().unwrap();

        let cookie_builder = CookieBuilder::new(params.name.to_owned(), params.value.to_owned())
            .secure(params.secure)
            .http_only(params.httpOnly);
        let cookie_builder = match params.domain {
            Some(ref domain) => cookie_builder.domain(domain.to_owned()),
            _ => cookie_builder,
        };
        let cookie_builder = match params.path {
            Some(ref path) => cookie_builder.path(path.to_owned()),
            _ => cookie_builder,
        };

        let cmd = WebDriverScriptCommand::AddCookie(cookie_builder.build(), sender);
        self.browsing_context_script_command(cmd)?;
        match wait_for_script_response(receiver)? {
            Ok(_) => Ok(WebDriverResponse::Void),
            Err(response) => match response {
                WebDriverCookieError::InvalidDomain => Err(WebDriverError::new(
                    ErrorStatus::InvalidCookieDomain,
                    "Invalid cookie domain",
                )),
                WebDriverCookieError::UnableToSetCookie => Err(WebDriverError::new(
                    ErrorStatus::UnableToSetCookie,
                    "Unable to set cookie",
                )),
            },
        }
    }

    fn handle_delete_cookie(&self, name: String) -> WebDriverResult<WebDriverResponse> {
        let (sender, receiver) = ipc::channel().unwrap();
        let cmd = WebDriverScriptCommand::DeleteCookie(name, sender);
        self.browsing_context_script_command(cmd)?;
        match wait_for_script_response(receiver)? {
            Ok(_) => Ok(WebDriverResponse::Void),
            Err(error) => Err(WebDriverError::new(error, "")),
        }
    }

    fn handle_delete_cookies(&self) -> WebDriverResult<WebDriverResponse> {
        let (sender, receiver) = ipc::channel().unwrap();
        let cmd = WebDriverScriptCommand::DeleteCookies(sender);
        self.browsing_context_script_command(cmd)?;
        match wait_for_script_response(receiver)? {
            Ok(_) => Ok(WebDriverResponse::Void),
            Err(error) => Err(WebDriverError::new(error, "")),
        }
    }

    // https://w3c.github.io/webdriver/#dismiss-alert
    fn handle_dismiss_alert(&mut self) -> WebDriverResult<WebDriverResponse> {
        // Since user prompts are not yet implement this will always succeed
        Ok(WebDriverResponse::Void)
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

    fn handle_get_page_source(&self) -> WebDriverResult<WebDriverResponse> {
        let (sender, receiver) = ipc::channel().unwrap();

        let cmd = WebDriverScriptCommand::GetPageSource(sender);
        self.browsing_context_script_command(cmd)?;

        match wait_for_script_response(receiver)? {
            Ok(source) => Ok(WebDriverResponse::Generic(ValueResponse(
                serde_json::to_value(source)?,
            ))),
            Err(error) => Err(WebDriverError::new(error, "")),
        }
    }

    fn handle_perform_actions(
        &mut self,
        parameters: &ActionsParameters,
    ) -> WebDriverResult<WebDriverResponse> {
        match self.dispatch_actions(&parameters.actions) {
            Ok(_) => Ok(WebDriverResponse::Void),
            Err(error) => Err(WebDriverError::new(error, "")),
        }
    }

    fn handle_release_actions(&mut self) -> WebDriverResult<WebDriverResponse> {
        let input_cancel_list = self.session().unwrap().input_cancel_list.borrow();
        if let Err(error) = self.dispatch_actions(&input_cancel_list) {
            return Err(WebDriverError::new(error, ""));
        }

        let session = self.session()?;
        session.input_state_table.borrow_mut().clear();

        Ok(WebDriverResponse::Void)
    }

    fn handle_execute_script(
        &self,
        parameters: &JavascriptCommandParameters,
    ) -> WebDriverResult<WebDriverResponse> {
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
        let command = WebDriverScriptCommand::ExecuteScript(script, sender);
        self.browsing_context_script_command(command)?;
        let result = wait_for_script_response(receiver)?;
        self.postprocess_js_result(result)
    }

    fn handle_execute_async_script(
        &self,
        parameters: &JavascriptCommandParameters,
    ) -> WebDriverResult<WebDriverResponse> {
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
        let command = WebDriverScriptCommand::ExecuteAsyncScript(script, sender);
        self.browsing_context_script_command(command)?;
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

    fn handle_element_send_keys(
        &self,
        element: &WebElement,
        keys: &SendKeysParameters,
    ) -> WebDriverResult<WebDriverResponse> {
        let browsing_context_id = self.session()?.browsing_context_id;

        let (sender, receiver) = ipc::channel().unwrap();

        let cmd = WebDriverScriptCommand::FocusElement(element.to_string(), sender);
        let cmd_msg = WebDriverCommandMsg::ScriptCommand(browsing_context_id, cmd);
        self.constellation_chan
            .send(EmbedderToConstellationMessage::WebDriverCommand(cmd_msg))
            .unwrap();

        // TODO: distinguish the not found and not focusable cases
        wait_for_script_response(receiver)?.map_err(|error| WebDriverError::new(error, ""))?;

        let input_events = send_keys(&keys.text);

        // TODO: there's a race condition caused by the focus command and the
        // send keys command being two separate messages,
        // so the constellation may have changed state between them.
        let cmd_msg = WebDriverCommandMsg::SendKeys(browsing_context_id, input_events);
        self.constellation_chan
            .send(EmbedderToConstellationMessage::WebDriverCommand(cmd_msg))
            .unwrap();

        Ok(WebDriverResponse::Void)
    }

    // https://w3c.github.io/webdriver/#element-click
    fn handle_element_click(&mut self, element: &WebElement) -> WebDriverResult<WebDriverResponse> {
        let (sender, receiver) = ipc::channel().unwrap();

        // Steps 1 - 7
        let command = WebDriverScriptCommand::ElementClick(element.to_string(), sender);
        self.browsing_context_script_command(command)?;

        match wait_for_script_response(receiver)? {
            Ok(element_id) => match element_id {
                Some(element_id) => {
                    let id = Uuid::new_v4().to_string();

                    // Step 8.1
                    self.session_mut()?.input_state_table.borrow_mut().insert(
                        id.clone(),
                        InputSourceState::Pointer(PointerInputState::new(&PointerType::Mouse)),
                    );

                    // Step 8.7. Construct a pointer move action.
                    // Step 8.8. Set a property x to 0 on pointer move action.
                    // Step 8.9. Set a property y to 0 on pointer move action.
                    // Step 8.10. Set a property origin to element on pointer move action.
                    let pointer_move_action = PointerMoveAction {
                        duration: None,
                        origin: PointerOrigin::Element(WebElement(element_id)),
                        x: 0.0,
                        y: 0.0,
                        ..Default::default()
                    };
                    let pointer_move_action = ActionSequence {
                        id: id.clone(),
                        actions: ActionsType::Pointer {
                            parameters: PointerActionParameters {
                                pointer_type: PointerType::Mouse,
                            },
                            actions: vec![PointerActionItem::Pointer(PointerAction::Move(
                                pointer_move_action,
                            ))],
                        },
                    };

                    // Step 8.11. Construct pointer down action.
                    // Step 8.12. Set a property button to 0 on pointer down action.
                    let pointer_down_action = PointerDownAction {
                        button: i16::from(MouseButton::Left) as u64,
                        ..Default::default()
                    };
                    let pointer_down_action = ActionSequence {
                        id: id.clone(),
                        actions: ActionsType::Pointer {
                            parameters: PointerActionParameters {
                                pointer_type: PointerType::Mouse,
                            },
                            actions: vec![PointerActionItem::Pointer(PointerAction::Down(
                                pointer_down_action,
                            ))],
                        },
                    };

                    // Step 8.13. Construct pointer up action.
                    // Step 8.14. Set a property button to 0 on pointer up action.
                    let pointer_up_action = PointerUpAction {
                        button: i16::from(MouseButton::Left) as u64,
                        ..Default::default()
                    };
                    let pointer_up_action = ActionSequence {
                        id: id.clone(),
                        actions: ActionsType::Pointer {
                            parameters: PointerActionParameters {
                                pointer_type: PointerType::Mouse,
                            },
                            actions: vec![PointerActionItem::Pointer(PointerAction::Up(
                                pointer_up_action,
                            ))],
                        },
                    };

                    let _ = self.dispatch_actions(&[
                        pointer_move_action,
                        pointer_down_action,
                        pointer_up_action,
                    ]);

                    // Step 8.17 Remove an input source with input state and input id.
                    self.session_mut()?
                        .input_state_table
                        .borrow_mut()
                        .remove(&id);

                    // Step 13
                    Ok(WebDriverResponse::Void)
                },
                // Step 13
                None => Ok(WebDriverResponse::Void),
            },
            Err(error) => Err(WebDriverError::new(error, "")),
        }
    }

    fn take_screenshot(&self, rect: Option<Rect<f32, CSSPixel>>) -> WebDriverResult<String> {
        let mut img = None;

        let interval = 1000;
        let iterations = 30000 / interval;

        for _ in 0..iterations {
            let (sender, receiver) = ipc::channel().unwrap();

            let cmd_msg =
                WebDriverCommandMsg::TakeScreenshot(self.session()?.webview_id, rect, sender);
            self.constellation_chan
                .send(EmbedderToConstellationMessage::WebDriverCommand(cmd_msg))
                .unwrap();

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

        let rgb = RgbaImage::from_raw(img.width, img.height, img.bytes().to_vec()).unwrap();
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

        let command = WebDriverScriptCommand::GetBoundingClientRect(element.to_string(), sender);
        self.browsing_context_script_command(command)?;

        match wait_for_script_response(receiver)? {
            Ok(rect) => {
                let encoded = self.take_screenshot(Some(Rect::from_untyped(&rect)))?;

                Ok(WebDriverResponse::Generic(ValueResponse(
                    serde_json::to_value(encoded)?,
                )))
            },
            Err(_) => Err(WebDriverError::new(
                ErrorStatus::StaleElementReference,
                "Element not found",
            )),
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
            WebDriverCommand::GetWindowRect => self.handle_window_size(),
            WebDriverCommand::SetWindowRect(ref size) => self.handle_set_window_size(size),
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
                self.handle_find_element_element(element, parameters)
            },
            WebDriverCommand::FindElementElements(ref element, ref parameters) => {
                self.handle_find_elements_from_element(element, parameters)
            },
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
            WebDriverCommand::PerformActions(ref x) => self.handle_perform_actions(x),
            WebDriverCommand::ReleaseActions => self.handle_release_actions(),
            WebDriverCommand::ExecuteScript(ref x) => self.handle_execute_script(x),
            WebDriverCommand::ExecuteAsyncScript(ref x) => self.handle_execute_async_script(x),
            WebDriverCommand::ElementSendKeys(ref element, ref keys) => {
                self.handle_element_send_keys(element, keys)
            },
            WebDriverCommand::ElementClick(ref element) => self.handle_element_click(element),
            WebDriverCommand::DismissAlert => self.handle_dismiss_alert(),
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

fn wait_for_script_response<T>(receiver: IpcReceiver<T>) -> Result<T, WebDriverError>
where
    T: for<'de> Deserialize<'de> + Serialize,
{
    receiver
        .recv()
        .map_err(|_| WebDriverError::new(ErrorStatus::NoSuchWindow, ""))
}
