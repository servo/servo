/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![crate_name = "webdriver_server"]
#![crate_type = "rlib"]
#![deny(unsafe_code)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate serde;

use base64;
use crossbeam_channel::Sender;
use euclid::TypedSize2D;
use hyper::Method;
use image::{DynamicImage, ImageFormat, RgbImage};
use ipc_channel::ipc::{self, IpcReceiver, IpcSender};
use keyboard_types::webdriver::send_keys;
use msg::constellation_msg::{BrowsingContextId, TopLevelBrowsingContextId, TraversalDirection};
use pixels::PixelFormat;
use regex::Captures;
use script_traits::webdriver_msg::{LoadStatus, WebDriverCookieError, WebDriverFrameId};
use script_traits::webdriver_msg::{
    WebDriverJSError, WebDriverJSResult, WebDriverJSValue, WebDriverScriptCommand,
};
use script_traits::{ConstellationMsg, LoadData, WebDriverCommandMsg};
use serde::de::{Deserialize, Deserializer, MapAccess, Visitor};
use serde::ser::{Serialize, Serializer};
use serde_json::{self, Value};
use servo_config::{prefs, prefs::PrefValue};
use servo_url::ServoUrl;
use std::borrow::ToOwned;
use std::collections::BTreeMap;
use std::fmt;
use std::net::{SocketAddr, SocketAddrV4};
use std::thread;
use std::time::Duration;
use uuid::Uuid;
use webdriver::command::{
    AddCookieParameters, GetParameters, JavascriptCommandParameters, LocatorParameters,
};
use webdriver::command::{SendKeysParameters, SwitchToFrameParameters, TimeoutsParameters};
use webdriver::command::{
    WebDriverCommand, WebDriverExtensionCommand, WebDriverMessage, WindowRectParameters,
};
use webdriver::common::{Cookie, Date, LocatorStrategy, WebElement};
use webdriver::error::{ErrorStatus, WebDriverError, WebDriverResult};
use webdriver::httpapi::WebDriverExtensionRoute;
use webdriver::response::{CookieResponse, CookiesResponse};
use webdriver::response::{ElementRectResponse, NewSessionResponse, ValueResponse};
use webdriver::response::{TimeoutsResponse, WebDriverResponse, WindowRectResponse};
use webdriver::server::{self, Session, WebDriverHandler};

fn extension_routes() -> Vec<(Method, &'static str, ServoExtensionRoute)> {
    return vec![
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
    ];
}

fn cookie_msg_to_cookie(cookie: cookie::Cookie) -> Cookie {
    Cookie {
        name: cookie.name().to_owned(),
        value: cookie.value().to_owned(),
        path: cookie.path().map(|s| s.to_owned()),
        domain: cookie.domain().map(|s| s.to_owned()),
        expiry: cookie
            .expires()
            .map(|time| Date(time.to_timespec().sec as u64)),
        secure: cookie.secure().unwrap_or(false),
        httpOnly: cookie.http_only().unwrap_or(false),
    }
}

pub fn start_server(port: u16, constellation_chan: Sender<ConstellationMsg>) {
    let handler = Handler::new(constellation_chan);
    thread::Builder::new()
        .name("WebdriverHttpServer".to_owned())
        .spawn(move || {
            let address = SocketAddrV4::new("0.0.0.0".parse().unwrap(), port);
            match server::start(SocketAddr::V4(address), handler, &extension_routes()) {
                Ok(listening) => info!("WebDriver server listening on {}", listening.socket),
                Err(_) => panic!("Unable to start WebDriver HTTPD server"),
            }
        })
        .expect("Thread spawning failed");
}

/// Represents the current WebDriver session and holds relevant session state.
struct WebDriverSession {
    id: Uuid,
    browsing_context_id: BrowsingContextId,
    top_level_browsing_context_id: TopLevelBrowsingContextId,

    /// Time to wait for injected scripts to run before interrupting them.  A [`None`] value
    /// specifies that the script should run indefinitely.
    script_timeout: Option<u64>,

    /// Time to wait for a page to finish loading upon navigation.
    load_timeout: u64,

    /// Time to wait for the element location strategy when retrieving elements, and when
    /// waiting for an element to become interactable.
    implicit_wait_timeout: u64,
}

impl WebDriverSession {
    pub fn new(
        browsing_context_id: BrowsingContextId,
        top_level_browsing_context_id: TopLevelBrowsingContextId,
    ) -> WebDriverSession {
        WebDriverSession {
            id: Uuid::new_v4(),
            browsing_context_id: browsing_context_id,
            top_level_browsing_context_id: top_level_browsing_context_id,

            script_timeout: Some(30_000),
            load_timeout: 300_000,
            implicit_wait_timeout: 0,
        }
    }
}

struct Handler {
    session: Option<WebDriverSession>,
    constellation_chan: Sender<ConstellationMsg>,
    resize_timeout: u32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum ServoExtensionRoute {
    GetPrefs,
    SetPrefs,
    ResetPrefs,
}

impl WebDriverExtensionRoute for ServoExtensionRoute {
    type Command = ServoExtensionCommand;

    fn command(
        &self,
        _captures: &Captures,
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
            WebDriverJSValue::Number(x) => serializer.serialize_f64(x),
            WebDriverJSValue::String(ref x) => serializer.serialize_str(&x),
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
            PrefValue::Str(ref s) => serializer.serialize_str(&s),
            PrefValue::Float(f) => serializer.serialize_f64(f),
            PrefValue::Int(i) => serializer.serialize_i64(i),
            PrefValue::Missing => serializer.serialize_unit(),
        }
    }
}

impl<'de> Deserialize<'de> for WebDriverPrefValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> ::serde::de::Visitor<'de> for Visitor {
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
    pub fn new(constellation_chan: Sender<ConstellationMsg>) -> Handler {
        Handler {
            session: None,
            constellation_chan: constellation_chan,
            resize_timeout: 500,
        }
    }

    fn focus_top_level_browsing_context_id(&self) -> WebDriverResult<TopLevelBrowsingContextId> {
        debug!("Getting focused context.");
        let interval = 20;
        let iterations = 30_000 / interval;
        let (sender, receiver) = ipc::channel().unwrap();

        for _ in 0..iterations {
            let msg = ConstellationMsg::GetFocusTopLevelBrowsingContext(sender.clone());
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

    fn handle_new_session(&mut self) -> WebDriverResult<WebDriverResponse> {
        debug!("new session");
        if self.session.is_none() {
            let top_level_browsing_context_id = self.focus_top_level_browsing_context_id()?;
            let browsing_context_id = BrowsingContextId::from(top_level_browsing_context_id);
            let session = WebDriverSession::new(browsing_context_id, top_level_browsing_context_id);
            let mut capabilities = serde_json::Map::new();
            capabilities.insert("browserName".to_owned(), serde_json::to_value("servo")?);
            capabilities.insert("browserVersion".to_owned(), serde_json::to_value("0.0.1")?);
            capabilities.insert(
                "acceptInsecureCerts".to_owned(),
                serde_json::to_value(false)?,
            );
            let response =
                NewSessionResponse::new(session.id.to_string(), Value::Object(capabilities));
            debug!("new session created {}.", session.id);
            self.session = Some(session);
            Ok(WebDriverResponse::NewSession(response))
        } else {
            debug!("new session failed.");
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

    fn browsing_context_script_command(
        &self,
        cmd_msg: WebDriverScriptCommand,
    ) -> WebDriverResult<()> {
        let browsing_context_id = self.session()?.browsing_context_id;
        let msg = ConstellationMsg::WebDriverCommand(WebDriverCommandMsg::ScriptCommand(
            browsing_context_id,
            cmd_msg,
        ));
        self.constellation_chan.send(msg).unwrap();
        Ok(())
    }

    fn top_level_script_command(&self, cmd_msg: WebDriverScriptCommand) -> WebDriverResult<()> {
        let browsing_context_id =
            BrowsingContextId::from(self.session()?.top_level_browsing_context_id);
        let msg = ConstellationMsg::WebDriverCommand(WebDriverCommandMsg::ScriptCommand(
            browsing_context_id,
            cmd_msg,
        ));
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

        let top_level_browsing_context_id = self.session()?.top_level_browsing_context_id;

        let (sender, receiver) = ipc::channel().unwrap();

        let load_data = LoadData::new(url.origin(), url, None, None, None);
        let cmd_msg =
            WebDriverCommandMsg::LoadUrl(top_level_browsing_context_id, load_data, sender.clone());
        self.constellation_chan
            .send(ConstellationMsg::WebDriverCommand(cmd_msg))
            .unwrap();

        self.wait_for_load(sender, receiver)
    }

    fn wait_for_load(
        &self,
        sender: IpcSender<LoadStatus>,
        receiver: IpcReceiver<LoadStatus>,
    ) -> WebDriverResult<WebDriverResponse> {
        let timeout = self.session()?.load_timeout;
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(timeout));
            let _ = sender.send(LoadStatus::LoadTimeout);
        });

        // wait to get a load event
        match receiver.recv().unwrap() {
            LoadStatus::LoadComplete => Ok(WebDriverResponse::Void),
            LoadStatus::LoadTimeout => {
                Err(WebDriverError::new(ErrorStatus::Timeout, "Load timed out"))
            },
        }
    }

    fn handle_current_url(&self) -> WebDriverResult<WebDriverResponse> {
        let (sender, receiver) = ipc::channel().unwrap();
        self.top_level_script_command(WebDriverScriptCommand::GetUrl(sender))?;

        let url = receiver.recv().unwrap();

        Ok(WebDriverResponse::Generic(ValueResponse(
            serde_json::to_value(url.as_str())?,
        )))
    }

    fn handle_window_size(&self) -> WebDriverResult<WebDriverResponse> {
        let (sender, receiver) = ipc::channel().unwrap();
        let top_level_browsing_context_id = self.session()?.top_level_browsing_context_id;
        let cmd_msg = WebDriverCommandMsg::GetWindowSize(top_level_browsing_context_id, sender);

        self.constellation_chan
            .send(ConstellationMsg::WebDriverCommand(cmd_msg))
            .unwrap();

        let window_size = receiver.recv().unwrap();
        let vp = window_size.initial_viewport;
        let window_size_response = WindowRectResponse {
            x: 0,
            y: 0,
            width: vp.width as i32,
            height: vp.height as i32,
        };
        Ok(WebDriverResponse::WindowRect(window_size_response))
    }

    fn handle_set_window_size(
        &self,
        params: &WindowRectParameters,
    ) -> WebDriverResult<WebDriverResponse> {
        let (sender, receiver) = ipc::channel().unwrap();
        let width = match params.width {
            Some(v) => v,
            None => 0,
        };
        let height = match params.height {
            Some(v) => v,
            None => 0,
        };
        let size = TypedSize2D::new(width as u32, height as u32);
        let top_level_browsing_context_id = self.session()?.top_level_browsing_context_id;
        let cmd_msg = WebDriverCommandMsg::SetWindowSize(
            top_level_browsing_context_id,
            size.to_i32(),
            sender.clone(),
        );

        self.constellation_chan
            .send(ConstellationMsg::WebDriverCommand(cmd_msg))
            .unwrap();

        let timeout = self.resize_timeout;
        let constellation_chan = self.constellation_chan.clone();
        thread::spawn(move || {
            // On timeout, we send a GetWindowSize message to the constellation,
            // which will give the current window size.
            thread::sleep(Duration::from_millis(timeout as u64));
            let cmd_msg = WebDriverCommandMsg::GetWindowSize(top_level_browsing_context_id, sender);
            constellation_chan
                .send(ConstellationMsg::WebDriverCommand(cmd_msg))
                .unwrap();
        });

        let window_size = receiver.recv().unwrap();
        let vp = window_size.initial_viewport;
        let window_size_response = WindowRectResponse {
            x: 0,
            y: 0,
            width: vp.width as i32,
            height: vp.height as i32,
        };
        Ok(WebDriverResponse::WindowRect(window_size_response))
    }

    fn handle_is_enabled(&self, element: &WebElement) -> WebDriverResult<WebDriverResponse> {
        let (sender, receiver) = ipc::channel().unwrap();

        self.top_level_script_command(WebDriverScriptCommand::IsEnabled(
            element.id.clone(),
            sender,
        ))?;

        match receiver.recv().unwrap() {
            Ok(is_enabled) => Ok(WebDriverResponse::Generic(ValueResponse(
                serde_json::to_value(is_enabled)?,
            ))),
            Err(_) => Err(WebDriverError::new(
                ErrorStatus::StaleElementReference,
                "Element not found",
            )),
        }
    }

    fn handle_is_selected(&self, element: &WebElement) -> WebDriverResult<WebDriverResponse> {
        let (sender, receiver) = ipc::channel().unwrap();

        self.top_level_script_command(WebDriverScriptCommand::IsSelected(
            element.id.clone(),
            sender,
        ))?;

        match receiver.recv().unwrap() {
            Ok(is_selected) => Ok(WebDriverResponse::Generic(ValueResponse(
                serde_json::to_value(is_selected)?,
            ))),
            Err(_) => Err(WebDriverError::new(
                ErrorStatus::StaleElementReference,
                "Element not found",
            )),
        }
    }

    fn handle_go_back(&self) -> WebDriverResult<WebDriverResponse> {
        let top_level_browsing_context_id = self.session()?.top_level_browsing_context_id;
        let direction = TraversalDirection::Back(1);
        let msg = ConstellationMsg::TraverseHistory(top_level_browsing_context_id, direction);
        self.constellation_chan.send(msg).unwrap();
        Ok(WebDriverResponse::Void)
    }

    fn handle_go_forward(&self) -> WebDriverResult<WebDriverResponse> {
        let top_level_browsing_context_id = self.session()?.top_level_browsing_context_id;
        let direction = TraversalDirection::Forward(1);
        let msg = ConstellationMsg::TraverseHistory(top_level_browsing_context_id, direction);
        self.constellation_chan.send(msg).unwrap();
        Ok(WebDriverResponse::Void)
    }

    fn handle_refresh(&self) -> WebDriverResult<WebDriverResponse> {
        let top_level_browsing_context_id = self.session()?.top_level_browsing_context_id;

        let (sender, receiver) = ipc::channel().unwrap();

        let cmd_msg = WebDriverCommandMsg::Refresh(top_level_browsing_context_id, sender.clone());
        self.constellation_chan
            .send(ConstellationMsg::WebDriverCommand(cmd_msg))
            .unwrap();

        self.wait_for_load(sender, receiver)
    }

    fn handle_title(&self) -> WebDriverResult<WebDriverResponse> {
        let (sender, receiver) = ipc::channel().unwrap();

        self.top_level_script_command(WebDriverScriptCommand::GetTitle(sender))?;

        let value = receiver.recv().unwrap();
        Ok(WebDriverResponse::Generic(ValueResponse(
            serde_json::to_value(value)?,
        )))
    }

    fn handle_window_handle(&self) -> WebDriverResult<WebDriverResponse> {
        // For now we assume there's only one window so just use the session
        // id as the window id
        let handle = self.session.as_ref().unwrap().id.to_string();
        Ok(WebDriverResponse::Generic(ValueResponse(
            serde_json::to_value(handle)?,
        )))
    }

    fn handle_window_handles(&self) -> WebDriverResult<WebDriverResponse> {
        // For now we assume there's only one window so just use the session
        // id as the window id
        let handles = vec![serde_json::to_value(
            self.session.as_ref().unwrap().id.to_string(),
        )?];
        Ok(WebDriverResponse::Generic(ValueResponse(
            serde_json::to_value(handles)?,
        )))
    }

    fn handle_find_element(
        &self,
        parameters: &LocatorParameters,
    ) -> WebDriverResult<WebDriverResponse> {
        if parameters.using != LocatorStrategy::CSSSelector {
            return Err(WebDriverError::new(
                ErrorStatus::UnsupportedOperation,
                "Unsupported locator strategy",
            ));
        }

        let (sender, receiver) = ipc::channel().unwrap();

        let cmd = WebDriverScriptCommand::FindElementCSS(parameters.value.clone(), sender);
        self.browsing_context_script_command(cmd)?;

        match receiver.recv().unwrap() {
            Ok(value) => {
                let value_resp = serde_json::to_value(
                    value.map(|x| serde_json::to_value(WebElement::new(x)).unwrap()),
                )?;
                Ok(WebDriverResponse::Generic(ValueResponse(value_resp)))
            },
            Err(_) => Err(WebDriverError::new(
                ErrorStatus::InvalidSelector,
                "Invalid selector",
            )),
        }
    }

    fn handle_switch_to_frame(
        &mut self,
        parameters: &SwitchToFrameParameters,
    ) -> WebDriverResult<WebDriverResponse> {
        use webdriver::common::FrameId;
        let frame_id = match parameters.id {
            None => {
                let session = self.session_mut()?;
                session.browsing_context_id =
                    BrowsingContextId::from(session.top_level_browsing_context_id);
                return Ok(WebDriverResponse::Void);
            },
            Some(FrameId::Short(ref x)) => WebDriverFrameId::Short(*x),
            Some(FrameId::Element(ref x)) => WebDriverFrameId::Element(x.id.clone()),
        };

        self.switch_to_frame(frame_id)
    }

    fn handle_switch_to_parent_frame(&mut self) -> WebDriverResult<WebDriverResponse> {
        self.switch_to_frame(WebDriverFrameId::Parent)
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

        let browsing_context_id = receiver.recv().unwrap().or(Err(WebDriverError::new(
            ErrorStatus::NoSuchFrame,
            "Frame does not exist",
        )))?;

        self.session_mut()?.browsing_context_id = browsing_context_id;
        Ok(WebDriverResponse::Void)
    }

    fn handle_find_elements(
        &self,
        parameters: &LocatorParameters,
    ) -> WebDriverResult<WebDriverResponse> {
        if parameters.using != LocatorStrategy::CSSSelector {
            return Err(WebDriverError::new(
                ErrorStatus::UnsupportedOperation,
                "Unsupported locator strategy",
            ));
        }

        let (sender, receiver) = ipc::channel().unwrap();
        let cmd = WebDriverScriptCommand::FindElementsCSS(parameters.value.clone(), sender);
        self.browsing_context_script_command(cmd)?;
        match receiver.recv().unwrap() {
            Ok(value) => {
                let resp_value: Vec<Value> = value
                    .into_iter()
                    .map(|x| serde_json::to_value(WebElement::new(x)).unwrap())
                    .collect();
                Ok(WebDriverResponse::Generic(ValueResponse(
                    serde_json::to_value(resp_value)?,
                )))
            },
            Err(_) => Err(WebDriverError::new(
                ErrorStatus::InvalidSelector,
                "Invalid selector",
            )),
        }
    }

    // https://w3c.github.io/webdriver/#find-element-from-element
    fn handle_find_element_element(
        &self,
        element: &WebElement,
        parameters: &LocatorParameters,
    ) -> WebDriverResult<WebDriverResponse> {
        if parameters.using != LocatorStrategy::CSSSelector {
            return Err(WebDriverError::new(
                ErrorStatus::UnsupportedOperation,
                "Unsupported locator strategy",
            ));
        }

        let (sender, receiver) = ipc::channel().unwrap();
        let cmd = WebDriverScriptCommand::FindElementElementCSS(
            parameters.value.clone(),
            element.id.clone(),
            sender,
        );

        self.browsing_context_script_command(cmd)?;

        match receiver.recv().unwrap() {
            Ok(value) => {
                let value_resp = serde_json::to_value(
                    value.map(|x| serde_json::to_value(WebElement::new(x)).unwrap()),
                )?;
                Ok(WebDriverResponse::Generic(ValueResponse(value_resp)))
            },
            Err(_) => Err(WebDriverError::new(
                ErrorStatus::InvalidSelector,
                "Invalid selector",
            )),
        }
    }

    // https://w3c.github.io/webdriver/#find-elements-from-element
    fn handle_find_elements_from_element(
        &self,
        element: &WebElement,
        parameters: &LocatorParameters,
    ) -> WebDriverResult<WebDriverResponse> {
        if parameters.using != LocatorStrategy::CSSSelector {
            return Err(WebDriverError::new(
                ErrorStatus::UnsupportedOperation,
                "Unsupported locator strategy",
            ));
        }

        let (sender, receiver) = ipc::channel().unwrap();
        let cmd = WebDriverScriptCommand::FindElementElementsCSS(
            parameters.value.clone(),
            element.id.clone(),
            sender,
        );

        self.browsing_context_script_command(cmd)?;

        match receiver.recv().unwrap() {
            Ok(value) => {
                let value_resp = value
                    .into_iter()
                    .map(|x| serde_json::to_value(WebElement::new(x)).unwrap())
                    .collect::<Vec<Value>>();
                let value_resp = serde_json::Value::Array(value_resp);
                Ok(WebDriverResponse::Generic(ValueResponse(value_resp)))
            },
            Err(_) => Err(WebDriverError::new(
                ErrorStatus::InvalidSelector,
                "Invalid Selector",
            )),
        }
    }

    // https://w3c.github.io/webdriver/webdriver-spec.html#get-element-rect
    fn handle_element_rect(&self, element: &WebElement) -> WebDriverResult<WebDriverResponse> {
        let (sender, receiver) = ipc::channel().unwrap();
        let cmd = WebDriverScriptCommand::GetElementRect(element.id.clone(), sender);
        self.browsing_context_script_command(cmd)?;
        match receiver.recv().unwrap() {
            Ok(rect) => {
                let response = ElementRectResponse {
                    x: rect.origin.x,
                    y: rect.origin.y,
                    width: rect.size.width,
                    height: rect.size.height,
                };
                Ok(WebDriverResponse::ElementRect(response))
            },
            Err(_) => Err(WebDriverError::new(
                ErrorStatus::StaleElementReference,
                "Unable to find element in document",
            )),
        }
    }

    fn handle_element_text(&self, element: &WebElement) -> WebDriverResult<WebDriverResponse> {
        let (sender, receiver) = ipc::channel().unwrap();
        let cmd = WebDriverScriptCommand::GetElementText(element.id.clone(), sender);
        self.browsing_context_script_command(cmd)?;
        match receiver.recv().unwrap() {
            Ok(value) => Ok(WebDriverResponse::Generic(ValueResponse(
                serde_json::to_value(value)?,
            ))),
            Err(_) => Err(WebDriverError::new(
                ErrorStatus::StaleElementReference,
                "Unable to find element in document",
            )),
        }
    }

    fn handle_active_element(&self) -> WebDriverResult<WebDriverResponse> {
        let (sender, receiver) = ipc::channel().unwrap();
        let cmd = WebDriverScriptCommand::GetActiveElement(sender);
        self.browsing_context_script_command(cmd)?;
        let value = receiver
            .recv()
            .unwrap()
            .map(|x| serde_json::to_value(WebElement::new(x)).unwrap());
        Ok(WebDriverResponse::Generic(ValueResponse(
            serde_json::to_value(value)?,
        )))
    }

    fn handle_element_tag_name(&self, element: &WebElement) -> WebDriverResult<WebDriverResponse> {
        let (sender, receiver) = ipc::channel().unwrap();
        let cmd = WebDriverScriptCommand::GetElementTagName(element.id.clone(), sender);
        self.browsing_context_script_command(cmd)?;
        match receiver.recv().unwrap() {
            Ok(value) => Ok(WebDriverResponse::Generic(ValueResponse(
                serde_json::to_value(value)?,
            ))),
            Err(_) => Err(WebDriverError::new(
                ErrorStatus::StaleElementReference,
                "Unable to find element in document",
            )),
        }
    }

    fn handle_element_attribute(
        &self,
        element: &WebElement,
        name: &str,
    ) -> WebDriverResult<WebDriverResponse> {
        let (sender, receiver) = ipc::channel().unwrap();
        let cmd = WebDriverScriptCommand::GetElementAttribute(
            element.id.clone(),
            name.to_owned(),
            sender,
        );
        self.browsing_context_script_command(cmd)?;
        match receiver.recv().unwrap() {
            Ok(value) => Ok(WebDriverResponse::Generic(ValueResponse(
                serde_json::to_value(value)?,
            ))),
            Err(_) => Err(WebDriverError::new(
                ErrorStatus::StaleElementReference,
                "Unable to find element in document",
            )),
        }
    }

    fn handle_element_css(
        &self,
        element: &WebElement,
        name: &str,
    ) -> WebDriverResult<WebDriverResponse> {
        let (sender, receiver) = ipc::channel().unwrap();
        let cmd =
            WebDriverScriptCommand::GetElementCSS(element.id.clone(), name.to_owned(), sender);
        self.browsing_context_script_command(cmd)?;
        match receiver.recv().unwrap() {
            Ok(value) => Ok(WebDriverResponse::Generic(ValueResponse(
                serde_json::to_value(value)?,
            ))),
            Err(_) => Err(WebDriverError::new(
                ErrorStatus::StaleElementReference,
                "Unable to find element in document",
            )),
        }
    }

    fn handle_get_cookies(&self) -> WebDriverResult<WebDriverResponse> {
        let (sender, receiver) = ipc::channel().unwrap();
        let cmd = WebDriverScriptCommand::GetCookies(sender);
        self.browsing_context_script_command(cmd)?;
        let cookies = receiver.recv().unwrap();
        let response = cookies
            .into_iter()
            .map(|cookie| cookie_msg_to_cookie(cookie.into_inner()))
            .collect::<Vec<Cookie>>();
        Ok(WebDriverResponse::Cookies(CookiesResponse(response)))
    }

    fn handle_get_cookie(&self, name: &str) -> WebDriverResult<WebDriverResponse> {
        let (sender, receiver) = ipc::channel().unwrap();
        let cmd = WebDriverScriptCommand::GetCookie(name.to_owned(), sender);
        self.browsing_context_script_command(cmd)?;
        let cookies = receiver.recv().unwrap();
        let response = cookies
            .into_iter()
            .map(|cookie| cookie_msg_to_cookie(cookie.into_inner()))
            .next()
            .unwrap();
        Ok(WebDriverResponse::Cookie(CookieResponse(response)))
    }

    fn handle_add_cookie(
        &self,
        params: &AddCookieParameters,
    ) -> WebDriverResult<WebDriverResponse> {
        let (sender, receiver) = ipc::channel().unwrap();

        let cookie = cookie::Cookie::build(params.name.to_owned(), params.value.to_owned())
            .secure(params.secure)
            .http_only(params.httpOnly);
        let cookie = match params.domain {
            Some(ref domain) => cookie.domain(domain.to_owned()),
            _ => cookie,
        };
        let cookie = match params.path {
            Some(ref path) => cookie.path(path.to_owned()).finish(),
            _ => cookie.finish(),
        };

        let cmd = WebDriverScriptCommand::AddCookie(cookie, sender);
        self.browsing_context_script_command(cmd)?;
        match receiver.recv().unwrap() {
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

    fn handle_delete_cookies(&self) -> WebDriverResult<WebDriverResponse> {
        let (sender, receiver) = ipc::channel().unwrap();
        let cmd = WebDriverScriptCommand::DeleteCookies(sender);
        self.browsing_context_script_command(cmd)?;
        match receiver.recv().unwrap() {
            Ok(_) => Ok(WebDriverResponse::Void),
            Err(_) => Err(WebDriverError::new(
                ErrorStatus::NoSuchWindow,
                "No such window found.",
            )),
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

    fn handle_execute_script(
        &self,
        parameters: &JavascriptCommandParameters,
    ) -> WebDriverResult<WebDriverResponse> {
        let func_body = &parameters.script;
        let args_string = "";

        // This is pretty ugly; we really want something that acts like
        // new Function() and then takes the resulting function and executes
        // it with a vec of arguments.
        let script = format!("(function() {{ {} }})({})", func_body, args_string);

        let (sender, receiver) = ipc::channel().unwrap();
        let command = WebDriverScriptCommand::ExecuteScript(script, sender);
        self.browsing_context_script_command(command)?;
        let result = receiver.recv().unwrap();
        self.postprocess_js_result(result)
    }

    fn handle_execute_async_script(
        &self,
        parameters: &JavascriptCommandParameters,
    ) -> WebDriverResult<WebDriverResponse> {
        let func_body = &parameters.script;
        let args_string = "window.webdriverCallback";

        let timeout_script = if let Some(script_timeout) = self.session()?.script_timeout {
            format!("setTimeout(webdriverTimeout, {});", script_timeout)
        } else {
            "".into()
        };
        let script = format!(
            "{} (function(callback) {{ {} }})({})",
            timeout_script, func_body, args_string
        );

        let (sender, receiver) = ipc::channel().unwrap();
        let command = WebDriverScriptCommand::ExecuteAsyncScript(script, sender);
        self.browsing_context_script_command(command)?;
        let result = receiver.recv().unwrap();
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
            Err(WebDriverJSError::Timeout) => Err(WebDriverError::new(ErrorStatus::Timeout, "")),
            Err(WebDriverJSError::UnknownType) => Err(WebDriverError::new(
                ErrorStatus::UnsupportedOperation,
                "Unsupported return type",
            )),
            Err(WebDriverJSError::BrowsingContextNotFound) => Err(WebDriverError::new(
                ErrorStatus::JavascriptError,
                "Pipeline id not found in browsing context",
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

        let cmd = WebDriverScriptCommand::FocusElement(element.id.clone(), sender);
        let cmd_msg = WebDriverCommandMsg::ScriptCommand(browsing_context_id, cmd);
        self.constellation_chan
            .send(ConstellationMsg::WebDriverCommand(cmd_msg))
            .unwrap();

        // TODO: distinguish the not found and not focusable cases
        receiver.recv().unwrap().or_else(|_| {
            Err(WebDriverError::new(
                ErrorStatus::StaleElementReference,
                "Element not found or not focusable",
            ))
        })?;

        let input_events = send_keys(&keys.text);

        // TODO: there's a race condition caused by the focus command and the
        // send keys command being two separate messages,
        // so the constellation may have changed state between them.
        let cmd_msg = WebDriverCommandMsg::SendKeys(browsing_context_id, input_events);
        self.constellation_chan
            .send(ConstellationMsg::WebDriverCommand(cmd_msg))
            .unwrap();

        Ok(WebDriverResponse::Void)
    }

    fn handle_take_screenshot(&self) -> WebDriverResult<WebDriverResponse> {
        let mut img = None;
        let top_level_id = self.session()?.top_level_browsing_context_id;

        let interval = 1000;
        let iterations = 30_000 / interval;

        for _ in 0..iterations {
            let (sender, receiver) = ipc::channel().unwrap();
            let cmd_msg = WebDriverCommandMsg::TakeScreenshot(top_level_id, sender);
            self.constellation_chan
                .send(ConstellationMsg::WebDriverCommand(cmd_msg))
                .unwrap();

            if let Some(x) = receiver.recv().unwrap() {
                img = Some(x);
                break;
            };

            thread::sleep(Duration::from_millis(interval))
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

        // The compositor always sends RGB pixels.
        assert_eq!(
            img.format,
            PixelFormat::RGB8,
            "Unexpected screenshot pixel format"
        );
        let rgb = RgbImage::from_raw(img.width, img.height, img.bytes.to_vec()).unwrap();

        let mut png_data = Vec::new();
        DynamicImage::ImageRgb8(rgb)
            .write_to(&mut png_data, ImageFormat::PNG)
            .unwrap();

        let encoded = base64::encode(&png_data);
        Ok(WebDriverResponse::Generic(ValueResponse(
            serde_json::to_value(encoded)?,
        )))
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
                    serde_json::to_value(prefs::pref_map().get(item)).unwrap(),
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
        for &(ref key, ref value) in parameters.prefs.iter() {
            prefs::pref_map()
                .set(key, value.0.clone())
                .expect("Failed to set preference");
        }
        Ok(WebDriverResponse::Void)
    }

    fn handle_reset_prefs(
        &self,
        parameters: &GetPrefsParameters,
    ) -> WebDriverResult<WebDriverResponse> {
        let prefs = if parameters.prefs.len() == 0 {
            prefs::pref_map().reset_all();
            BTreeMap::new()
        } else {
            parameters
                .prefs
                .iter()
                .map(|item| {
                    (
                        item.clone(),
                        serde_json::to_value(
                            prefs::pref_map().reset(item).unwrap_or(PrefValue::Missing),
                        )
                        .unwrap(),
                    )
                })
                .collect::<BTreeMap<_, _>>()
        };
        Ok(WebDriverResponse::Generic(ValueResponse(
            serde_json::to_value(prefs)?,
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
            WebDriverCommand::NewSession(_) => {},
            _ => {
                self.session()?;
            },
        }

        match msg.command {
            WebDriverCommand::NewSession(_) => self.handle_new_session(),
            WebDriverCommand::DeleteSession => self.handle_delete_session(),
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
            WebDriverCommand::SwitchToFrame(ref parameters) => {
                self.handle_switch_to_frame(parameters)
            },
            WebDriverCommand::SwitchToParentFrame => self.handle_switch_to_parent_frame(),
            WebDriverCommand::FindElement(ref parameters) => self.handle_find_element(parameters),
            WebDriverCommand::FindElements(ref parameters) => self.handle_find_elements(parameters),
            WebDriverCommand::FindElementElement(ref element, ref parameters) => {
                self.handle_find_element_element(element, parameters)
            },
            WebDriverCommand::FindElementElements(ref element, ref parameters) => {
                self.handle_find_elements_from_element(element, parameters)
            },
            WebDriverCommand::GetNamedCookie(ref name) => self.handle_get_cookie(name),
            WebDriverCommand::GetCookies => self.handle_get_cookies(),
            WebDriverCommand::GetActiveElement => self.handle_active_element(),
            WebDriverCommand::GetElementRect(ref element) => self.handle_element_rect(element),
            WebDriverCommand::GetElementText(ref element) => self.handle_element_text(element),
            WebDriverCommand::GetElementTagName(ref element) => {
                self.handle_element_tag_name(element)
            },
            WebDriverCommand::GetElementAttribute(ref element, ref name) => {
                self.handle_element_attribute(element, name)
            },
            WebDriverCommand::GetCSSValue(ref element, ref name) => {
                self.handle_element_css(element, name)
            },
            WebDriverCommand::ExecuteScript(ref x) => self.handle_execute_script(x),
            WebDriverCommand::ExecuteAsyncScript(ref x) => self.handle_execute_async_script(x),
            WebDriverCommand::ElementSendKeys(ref element, ref keys) => {
                self.handle_element_send_keys(element, keys)
            },
            WebDriverCommand::DeleteCookies => self.handle_delete_cookies(),
            WebDriverCommand::GetTimeouts => self.handle_get_timeouts(),
            WebDriverCommand::SetTimeouts(ref x) => self.handle_set_timeouts(x),
            WebDriverCommand::TakeScreenshot => self.handle_take_screenshot(),
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

    fn delete_session(&mut self, _session: &Option<Session>) {
        // Servo doesn't support multiple sessions, so we exit on session deletion
        let _ = self
            .constellation_chan
            .send(ConstellationMsg::Exit)
            .unwrap();
        self.session = None;
    }
}
