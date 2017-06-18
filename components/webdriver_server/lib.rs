/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![crate_name = "webdriver_server"]
#![crate_type = "rlib"]

#![deny(unsafe_code)]

extern crate base64;
extern crate cookie as cookie_rs;
extern crate euclid;
extern crate hyper;
extern crate image;
extern crate ipc_channel;
#[macro_use]
extern crate log;
extern crate msg;
extern crate net_traits;
extern crate regex;
extern crate rustc_serialize;
extern crate script_traits;
extern crate servo_config;
extern crate servo_url;
extern crate uuid;
extern crate webdriver;

mod keys;

use euclid::Size2D;
use hyper::method::Method::{self, Post};
use image::{DynamicImage, ImageFormat, RgbImage};
use ipc_channel::ipc::{self, IpcReceiver, IpcSender};
use keys::keycodes_to_keys;
use msg::constellation_msg::{BrowsingContextId, TopLevelBrowsingContextId, TraversalDirection};
use net_traits::image::base::PixelFormat;
use regex::Captures;
use rustc_serialize::json::{Json, ToJson};
use script_traits::{ConstellationMsg, LoadData, WebDriverCommandMsg};
use script_traits::webdriver_msg::{LoadStatus, WebDriverCookieError, WebDriverFrameId};
use script_traits::webdriver_msg::{WebDriverJSError, WebDriverJSResult, WebDriverScriptCommand};
use servo_config::prefs::{PREFS, PrefValue};
use servo_url::ServoUrl;
use std::borrow::ToOwned;
use std::collections::BTreeMap;
use std::net::{SocketAddr, SocketAddrV4};
use std::sync::mpsc::Sender;
use std::thread;
use std::time::Duration;
use uuid::Uuid;
use webdriver::command::{AddCookieParameters, GetParameters, JavascriptCommandParameters};
use webdriver::command::{LocatorParameters, Parameters};
use webdriver::command::{SendKeysParameters, SwitchToFrameParameters, TimeoutsParameters};
use webdriver::command::{WebDriverCommand, WebDriverExtensionCommand, WebDriverMessage};
use webdriver::command::WindowSizeParameters;
use webdriver::common::{Date, LocatorStrategy, Nullable, WebElement};
use webdriver::error::{ErrorStatus, WebDriverError, WebDriverResult};
use webdriver::httpapi::WebDriverExtensionRoute;
use webdriver::response::{Cookie, CookieResponse};
use webdriver::response::{ElementRectResponse, NewSessionResponse, ValueResponse};
use webdriver::response::{WebDriverResponse, WindowSizeResponse};
use webdriver::server::{self, Session, WebDriverHandler};

fn extension_routes() -> Vec<(Method, &'static str, ServoExtensionRoute)> {
    return vec![(Post, "/session/{sessionId}/servo/prefs/get", ServoExtensionRoute::GetPrefs),
                (Post, "/session/{sessionId}/servo/prefs/set", ServoExtensionRoute::SetPrefs),
                (Post, "/session/{sessionId}/servo/prefs/reset", ServoExtensionRoute::ResetPrefs)]
}

fn cookie_msg_to_cookie(cookie: cookie_rs::Cookie) -> Cookie {
    Cookie {
        name: cookie.name().to_owned(),
        value: cookie.value().to_owned(),
        path: match cookie.path() {
            Some(path) => Nullable::Value(path.to_string()),
            None => Nullable::Null
        },
        domain: match cookie.domain() {
            Some(domain) => Nullable::Value(domain.to_string()),
            None => Nullable::Null
        },
        expiry: match cookie.expires() {
            Some(time) => Nullable::Value(Date::new(time.to_timespec().sec as u64)),
            None => Nullable::Null
        },
        secure: cookie.secure(),
        httpOnly: cookie.http_only(),
    }
}

pub fn start_server(port: u16, constellation_chan: Sender<ConstellationMsg>) {
    let handler = Handler::new(constellation_chan);
    thread::Builder::new().name("WebdriverHttpServer".to_owned()).spawn(move || {
        let address = SocketAddrV4::new("0.0.0.0".parse().unwrap(), port);
        match server::start(SocketAddr::V4(address), handler, &extension_routes()) {
            Ok(listening) => info!("WebDriver server listening on {}", listening.socket),
            Err(_) => panic!("Unable to start WebDriver HTTPD server"),
        }
    }).expect("Thread spawning failed");
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
    load_timeout: Option<u64>,

    /// Time to wait for the element location strategy when retrieving elements, and when
    /// waiting for an element to become interactable.
    implicit_wait_timeout: Option<u64>,
}

impl WebDriverSession {
    pub fn new(browsing_context_id: BrowsingContextId,
               top_level_browsing_context_id: TopLevelBrowsingContextId)
               -> WebDriverSession
    {
        WebDriverSession {
            id: Uuid::new_v4(),
            browsing_context_id: browsing_context_id,
            top_level_browsing_context_id: top_level_browsing_context_id,

            script_timeout: Some(30_000),
            load_timeout: Some(300_000),
            implicit_wait_timeout: Some(0),
        }
    }
}

struct Handler {
    session: Option<WebDriverSession>,
    constellation_chan: Sender<ConstellationMsg>,
    resize_timeout: u32,
}

#[derive(Clone, Copy, PartialEq)]
enum ServoExtensionRoute {
    GetPrefs,
    SetPrefs,
    ResetPrefs,
}

impl WebDriverExtensionRoute for ServoExtensionRoute {
    type Command = ServoExtensionCommand;

    fn command(&self,
               _captures: &Captures,
               body_data: &Json) -> WebDriverResult<WebDriverCommand<ServoExtensionCommand>> {
        let command = match *self {
            ServoExtensionRoute::GetPrefs => {
                let parameters: GetPrefsParameters = Parameters::from_json(&body_data)?;
                ServoExtensionCommand::GetPrefs(parameters)
            }
            ServoExtensionRoute::SetPrefs => {
                let parameters: SetPrefsParameters = Parameters::from_json(&body_data)?;
                ServoExtensionCommand::SetPrefs(parameters)
            }
            ServoExtensionRoute::ResetPrefs => {
                let parameters: GetPrefsParameters = Parameters::from_json(&body_data)?;
                ServoExtensionCommand::ResetPrefs(parameters)
            }
        };
        Ok(WebDriverCommand::Extension(command))
    }
}

#[derive(Clone, PartialEq)]
enum ServoExtensionCommand {
    GetPrefs(GetPrefsParameters),
    SetPrefs(SetPrefsParameters),
    ResetPrefs(GetPrefsParameters),
}

impl WebDriverExtensionCommand for ServoExtensionCommand {
    fn parameters_json(&self) -> Option<Json> {
        match *self {
            ServoExtensionCommand::GetPrefs(ref x) => Some(x.to_json()),
            ServoExtensionCommand::SetPrefs(ref x) => Some(x.to_json()),
            ServoExtensionCommand::ResetPrefs(ref x) => Some(x.to_json()),
        }
    }
}

#[derive(Clone, PartialEq)]
struct GetPrefsParameters {
    prefs: Vec<String>
}

impl Parameters for GetPrefsParameters {
    fn from_json(body: &Json) -> WebDriverResult<GetPrefsParameters> {
        let data = body.as_object().ok_or(
            WebDriverError::new(ErrorStatus::InvalidArgument,
                                "Message body was not an object"))?;
        let prefs_value = data.get("prefs").ok_or(
            WebDriverError::new(ErrorStatus::InvalidArgument,
                                "Missing prefs key"))?;
        let items = prefs_value.as_array().ok_or(
            WebDriverError::new(
                ErrorStatus::InvalidArgument,
                "prefs was not an array"))?;
        let params = items.iter().map(|x| x.as_string().map(|y| y.to_owned()).ok_or(
            WebDriverError::new(ErrorStatus::InvalidArgument,
                                "Pref is not a string"))).collect::<Result<Vec<_>, _>>()?;
        Ok(GetPrefsParameters {
            prefs: params
        })
    }
}

impl ToJson for GetPrefsParameters {
    fn to_json(&self) -> Json {
        let mut data = BTreeMap::new();
        data.insert("prefs".to_owned(), self.prefs.to_json());
        Json::Object(data)
    }
}

#[derive(Clone, PartialEq)]
struct SetPrefsParameters {
    prefs: Vec<(String, PrefValue)>
}

impl Parameters for SetPrefsParameters {
    fn from_json(body: &Json) -> WebDriverResult<SetPrefsParameters> {
        let data = body.as_object().ok_or(
            WebDriverError::new(ErrorStatus::InvalidArgument,
                                "Message body was not an object"))?;
        let items = data.get("prefs").ok_or(
            WebDriverError::new(ErrorStatus::InvalidArgument,
                                "Missing prefs key"))?.as_object().ok_or(
            WebDriverError::new(
                ErrorStatus::InvalidArgument,
                "prefs was not an array"))?;
        let mut params = Vec::with_capacity(items.len());
        for (name, val) in items.iter() {
            let value = PrefValue::from_json(val.clone()).or(
                Err(WebDriverError::new(ErrorStatus::InvalidArgument,
                                        "Pref is not a boolean or string")))?;
            let key = name.to_owned();
            params.push((key, value));
        }
        Ok(SetPrefsParameters {
            prefs: params
        })
    }
}

impl ToJson for SetPrefsParameters {
    fn to_json(&self) -> Json {
        let mut data = BTreeMap::new();
        data.insert("prefs".to_owned(), self.prefs.to_json());
        Json::Object(data)
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
        Err(WebDriverError::new(ErrorStatus::Timeout,
                                "Failed to get window handle"))
    }

    fn session(&self) -> WebDriverResult<&WebDriverSession> {
        match self.session {
            Some(ref x) => Ok(x),
            None => Err(WebDriverError::new(ErrorStatus::SessionNotCreated,
                                            "Session not created"))
        }
    }

    fn session_mut(&mut self) -> WebDriverResult<&mut WebDriverSession> {
        match self.session {
            Some(ref mut x) => Ok(x),
            None => Err(WebDriverError::new(ErrorStatus::SessionNotCreated,
                                            "Session not created"))
        }
    }

    fn handle_new_session(&mut self) -> WebDriverResult<WebDriverResponse> {
        debug!("new session");
        if self.session.is_none() {
            let top_level_browsing_context_id = self.focus_top_level_browsing_context_id()?;
            let browsing_context_id = BrowsingContextId::from(top_level_browsing_context_id);
            let session = WebDriverSession::new(browsing_context_id, top_level_browsing_context_id);
            let mut capabilities = BTreeMap::new();
            capabilities.insert("browserName".to_owned(), "servo".to_json());
            capabilities.insert("browserVersion".to_owned(), "0.0.1".to_json());
            capabilities.insert("acceptInsecureCerts".to_owned(), false.to_json());
            let response = NewSessionResponse::new(session.id.to_string(), Json::Object(capabilities));
            debug!("new session created {}.", session.id);
            self.session = Some(session);
            Ok(WebDriverResponse::NewSession(response))
        } else {
            debug!("new session failed.");
            Err(WebDriverError::new(ErrorStatus::UnknownError,
                                    "Session already created"))
        }
    }

    fn handle_delete_session(&mut self) -> WebDriverResult<WebDriverResponse> {
        self.session = None;
        Ok(WebDriverResponse::Void)
    }

    fn browsing_context_script_command(&self, cmd_msg: WebDriverScriptCommand) -> WebDriverResult<()> {
        let browsing_context_id = self.session()?.browsing_context_id;
        let msg = ConstellationMsg::WebDriverCommand(WebDriverCommandMsg::ScriptCommand(browsing_context_id, cmd_msg));
        self.constellation_chan.send(msg).unwrap();
        Ok(())
    }

    fn top_level_script_command(&self, cmd_msg: WebDriverScriptCommand) -> WebDriverResult<()> {
        let browsing_context_id = BrowsingContextId::from(self.session()?.top_level_browsing_context_id);
        let msg = ConstellationMsg::WebDriverCommand(WebDriverCommandMsg::ScriptCommand(browsing_context_id, cmd_msg));
        self.constellation_chan.send(msg).unwrap();
        Ok(())
    }

    fn handle_get(&self, parameters: &GetParameters) -> WebDriverResult<WebDriverResponse> {
        let url = match ServoUrl::parse(&parameters.url[..]) {
            Ok(url) => url,
            Err(_) => return Err(WebDriverError::new(ErrorStatus::InvalidArgument,
                                               "Invalid URL"))
        };

        let top_level_browsing_context_id = self.session()?.top_level_browsing_context_id;

        let (sender, receiver) = ipc::channel().unwrap();

        let load_data = LoadData::new(url, None, None, None);
        let cmd_msg = WebDriverCommandMsg::LoadUrl(top_level_browsing_context_id, load_data, sender.clone());
        self.constellation_chan.send(ConstellationMsg::WebDriverCommand(cmd_msg)).unwrap();

        self.wait_for_load(sender, receiver)
    }

    fn wait_for_load(&self,
                     sender: IpcSender<LoadStatus>,
                     receiver: IpcReceiver<LoadStatus>)
                     -> WebDriverResult<WebDriverResponse> {
        let timeout = self.session()?.load_timeout;
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(timeout.unwrap()));
            let _ = sender.send(LoadStatus::LoadTimeout);
        });

        // wait to get a load event
        match receiver.recv().unwrap() {
            LoadStatus::LoadComplete => Ok(WebDriverResponse::Void),
            LoadStatus::LoadTimeout => {
                Err(WebDriverError::new(ErrorStatus::Timeout, "Load timed out"))
            }
        }
    }

    fn handle_current_url(&self) -> WebDriverResult<WebDriverResponse> {
        let (sender, receiver) = ipc::channel().unwrap();
        self.top_level_script_command(WebDriverScriptCommand::GetUrl(sender))?;

        let url = receiver.recv().unwrap();

        Ok(WebDriverResponse::Generic(ValueResponse::new(url.as_str().to_json())))
    }

    fn handle_window_size(&self) -> WebDriverResult<WebDriverResponse> {
        let (sender, receiver) = ipc::channel().unwrap();
        let top_level_browsing_context_id = self.session()?.top_level_browsing_context_id;
        let cmd_msg = WebDriverCommandMsg::GetWindowSize(top_level_browsing_context_id, sender);

        self.constellation_chan.send(ConstellationMsg::WebDriverCommand(cmd_msg)).unwrap();

        let window_size = receiver.recv().unwrap();
        let vp = window_size.initial_viewport;
        let window_size_response = WindowSizeResponse::new(vp.width as u64, vp.height as u64);
        Ok(WebDriverResponse::WindowSize(window_size_response))
    }

    fn handle_set_window_size(&self, params: &WindowSizeParameters) -> WebDriverResult<WebDriverResponse> {
        let (sender, receiver) = ipc::channel().unwrap();
        let size = Size2D::new(params.width as u32, params.height as u32);
        let top_level_browsing_context_id = self.session()?.top_level_browsing_context_id;
        let cmd_msg = WebDriverCommandMsg::SetWindowSize(top_level_browsing_context_id, size, sender.clone());

        self.constellation_chan.send(ConstellationMsg::WebDriverCommand(cmd_msg)).unwrap();

        let timeout = self.resize_timeout;
        let constellation_chan = self.constellation_chan.clone();
        thread::spawn(move || {
            // On timeout, we send a GetWindowSize message to the constellation,
            // which will give the current window size.
            thread::sleep(Duration::from_millis(timeout as u64));
            let cmd_msg = WebDriverCommandMsg::GetWindowSize(top_level_browsing_context_id, sender);
            constellation_chan.send(ConstellationMsg::WebDriverCommand(cmd_msg)).unwrap();
        });

        let window_size = receiver.recv().unwrap();
        let vp = window_size.initial_viewport;
        let window_size_response = WindowSizeResponse::new(vp.width as u64, vp.height as u64);
        Ok(WebDriverResponse::WindowSize(window_size_response))
    }

    fn handle_is_enabled(&self, element: &WebElement) -> WebDriverResult<WebDriverResponse> {
        let (sender, receiver) = ipc::channel().unwrap();

        self.top_level_script_command(WebDriverScriptCommand::IsEnabled(element.id.clone(), sender))?;

        match receiver.recv().unwrap() {
            Ok(is_enabled) => Ok(WebDriverResponse::Generic(ValueResponse::new(is_enabled.to_json()))),
            Err(_) => Err(WebDriverError::new(ErrorStatus::StaleElementReference, "Element not found"))
        }
    }

    fn handle_is_selected(&self, element: &WebElement) -> WebDriverResult<WebDriverResponse> {
        let (sender, receiver) = ipc::channel().unwrap();

        self.top_level_script_command(WebDriverScriptCommand::IsSelected(element.id.clone(), sender))?;

        match receiver.recv().unwrap() {
            Ok(is_selected) => Ok(WebDriverResponse::Generic(ValueResponse::new(is_selected.to_json()))),
            Err(_) => Err(WebDriverError::new(ErrorStatus::StaleElementReference, "Element not found"))
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
        self.constellation_chan.send(ConstellationMsg::WebDriverCommand(cmd_msg)).unwrap();

        self.wait_for_load(sender, receiver)
    }

    fn handle_title(&self) -> WebDriverResult<WebDriverResponse> {
        let (sender, receiver) = ipc::channel().unwrap();

        self.top_level_script_command(WebDriverScriptCommand::GetTitle(sender))?;

        let value = receiver.recv().unwrap();
        Ok(WebDriverResponse::Generic(ValueResponse::new(value.to_json())))
    }

    fn handle_window_handle(&self) -> WebDriverResult<WebDriverResponse> {
        // For now we assume there's only one window so just use the session
        // id as the window id
        let handle = self.session.as_ref().unwrap().id.to_string();
        Ok(WebDriverResponse::Generic(ValueResponse::new(handle.to_json())))
    }

    fn handle_window_handles(&self) -> WebDriverResult<WebDriverResponse> {
        // For now we assume there's only one window so just use the session
        // id as the window id
        let handles = vec![self.session.as_ref().unwrap().id.to_string().to_json()];
        Ok(WebDriverResponse::Generic(ValueResponse::new(handles.to_json())))
    }

    fn handle_find_element(&self, parameters: &LocatorParameters) -> WebDriverResult<WebDriverResponse> {
        if parameters.using != LocatorStrategy::CSSSelector {
            return Err(WebDriverError::new(ErrorStatus::UnsupportedOperation,
                                           "Unsupported locator strategy"))
        }

        let (sender, receiver) = ipc::channel().unwrap();

        let cmd = WebDriverScriptCommand::FindElementCSS(parameters.value.clone(), sender);
        self.browsing_context_script_command(cmd)?;

        match receiver.recv().unwrap() {
            Ok(value) => {
                let value_resp = value.map(|x| WebElement::new(x).to_json()).to_json();
                Ok(WebDriverResponse::Generic(ValueResponse::new(value_resp)))
            }
            Err(_) => Err(WebDriverError::new(ErrorStatus::InvalidSelector,
                                              "Invalid selector"))
        }
    }

    fn handle_switch_to_frame(&mut self, parameters: &SwitchToFrameParameters) -> WebDriverResult<WebDriverResponse> {
        use webdriver::common::FrameId;
        let frame_id = match parameters.id {
            FrameId::Null => {
                let session = self.session_mut()?;
                session.browsing_context_id = BrowsingContextId::from(session.top_level_browsing_context_id);
                return Ok(WebDriverResponse::Void);
            },
            FrameId::Short(ref x) => WebDriverFrameId::Short(*x),
            FrameId::Element(ref x) => WebDriverFrameId::Element(x.id.clone())
        };

        self.switch_to_frame(frame_id)
    }


    fn handle_switch_to_parent_frame(&mut self) -> WebDriverResult<WebDriverResponse> {
        self.switch_to_frame(WebDriverFrameId::Parent)
    }

    fn switch_to_frame(&mut self, frame_id: WebDriverFrameId) -> WebDriverResult<WebDriverResponse> {
        if let WebDriverFrameId::Short(_) = frame_id {
            return Err(WebDriverError::new(ErrorStatus::UnsupportedOperation,
                                           "Selecting frame by id not supported"));
        }

        let (sender, receiver) = ipc::channel().unwrap();
        let cmd = WebDriverScriptCommand::GetBrowsingContextId(frame_id, sender);
        self.browsing_context_script_command(cmd)?;

        let browsing_context_id = receiver.recv().unwrap()
            .or(Err(WebDriverError::new(ErrorStatus::NoSuchFrame, "Frame does not exist")))?;

        self.session_mut()?.browsing_context_id = browsing_context_id;
        Ok(WebDriverResponse::Void)
    }


    fn handle_find_elements(&self, parameters: &LocatorParameters) -> WebDriverResult<WebDriverResponse> {
        if parameters.using != LocatorStrategy::CSSSelector {
            return Err(WebDriverError::new(ErrorStatus::UnsupportedOperation,
                                           "Unsupported locator strategy"))
        }

        let (sender, receiver) = ipc::channel().unwrap();
        let cmd = WebDriverScriptCommand::FindElementsCSS(parameters.value.clone(), sender);
        self.browsing_context_script_command(cmd)?;
        match receiver.recv().unwrap() {
            Ok(value) => {
                let resp_value: Vec<Json> = value.into_iter().map(
                    |x| WebElement::new(x).to_json()).collect();
                Ok(WebDriverResponse::Generic(ValueResponse::new(resp_value.to_json())))
            }
            Err(_) => Err(WebDriverError::new(ErrorStatus::InvalidSelector,
                                              "Invalid selector"))
        }
    }

    // https://w3c.github.io/webdriver/webdriver-spec.html#get-element-rect
    fn handle_element_rect(&self, element: &WebElement) -> WebDriverResult<WebDriverResponse> {
        let (sender, receiver) = ipc::channel().unwrap();
        let cmd = WebDriverScriptCommand::GetElementRect(element.id.clone(), sender);
        self.browsing_context_script_command(cmd)?;
        match receiver.recv().unwrap() {
            Ok(rect) => {
                let response = ElementRectResponse::new(rect.origin.x, rect.origin.y,
                                                        rect.size.width, rect.size.height);
                Ok(WebDriverResponse::ElementRect(response))
            },
            Err(_) => Err(WebDriverError::new(ErrorStatus::StaleElementReference,
                                              "Unable to find element in document"))
        }
    }

    fn handle_element_text(&self, element: &WebElement) -> WebDriverResult<WebDriverResponse> {
        let (sender, receiver) = ipc::channel().unwrap();
        let cmd = WebDriverScriptCommand::GetElementText(element.id.clone(), sender);
        self.browsing_context_script_command(cmd)?;
        match receiver.recv().unwrap() {
            Ok(value) => Ok(WebDriverResponse::Generic(ValueResponse::new(value.to_json()))),
            Err(_) => Err(WebDriverError::new(ErrorStatus::StaleElementReference,
                                              "Unable to find element in document"))
        }
    }

    fn handle_active_element(&self) -> WebDriverResult<WebDriverResponse> {
        let (sender, receiver) = ipc::channel().unwrap();
        let cmd = WebDriverScriptCommand::GetActiveElement(sender);
        self.browsing_context_script_command(cmd)?;
        let value = receiver.recv().unwrap().map(|x| WebElement::new(x).to_json());
        Ok(WebDriverResponse::Generic(ValueResponse::new(value.to_json())))
    }

    fn handle_element_tag_name(&self, element: &WebElement) -> WebDriverResult<WebDriverResponse> {
        let (sender, receiver) = ipc::channel().unwrap();
        let cmd = WebDriverScriptCommand::GetElementTagName(element.id.clone(), sender);
        self.browsing_context_script_command(cmd)?;
        match receiver.recv().unwrap() {
            Ok(value) => Ok(WebDriverResponse::Generic(ValueResponse::new(value.to_json()))),
            Err(_) => Err(WebDriverError::new(ErrorStatus::StaleElementReference,
                                              "Unable to find element in document"))
        }
    }

    fn handle_element_attribute(&self, element: &WebElement, name: &str) -> WebDriverResult<WebDriverResponse> {
        let (sender, receiver) = ipc::channel().unwrap();
        let cmd = WebDriverScriptCommand::GetElementAttribute(element.id.clone(), name.to_owned(), sender);
        self.browsing_context_script_command(cmd)?;
        match receiver.recv().unwrap() {
            Ok(value) => Ok(WebDriverResponse::Generic(ValueResponse::new(value.to_json()))),
            Err(_) => Err(WebDriverError::new(ErrorStatus::StaleElementReference,
                                              "Unable to find element in document"))
        }
    }

    fn handle_element_css(&self, element: &WebElement, name: &str) -> WebDriverResult<WebDriverResponse> {
        let (sender, receiver) = ipc::channel().unwrap();
        let cmd = WebDriverScriptCommand::GetElementCSS(element.id.clone(), name.to_owned(), sender);
        self.browsing_context_script_command(cmd)?;
        match receiver.recv().unwrap() {
            Ok(value) => Ok(WebDriverResponse::Generic(ValueResponse::new(value.to_json()))),
            Err(_) => Err(WebDriverError::new(ErrorStatus::StaleElementReference,
                                              "Unable to find element in document"))
        }
    }

    fn handle_get_cookies(&self) -> WebDriverResult<WebDriverResponse> {
        let (sender, receiver) = ipc::channel().unwrap();
        let cmd = WebDriverScriptCommand::GetCookies(sender);
        self.browsing_context_script_command(cmd)?;
        let cookies = receiver.recv().unwrap();
        let response = cookies.into_iter().map(|cookie| {
            cookie_msg_to_cookie(cookie.into_inner())
        }).collect::<Vec<Cookie>>();
        Ok(WebDriverResponse::Cookie(CookieResponse::new(response)))
    }

    fn handle_get_cookie(&self, name: &str) -> WebDriverResult<WebDriverResponse> {
        let (sender, receiver) = ipc::channel().unwrap();
        let cmd = WebDriverScriptCommand::GetCookie(name.to_owned(), sender);
        self.browsing_context_script_command(cmd)?;
        let cookies = receiver.recv().unwrap();
        let response = cookies.into_iter().map(|cookie| {
            cookie_msg_to_cookie(cookie.into_inner())
        }).collect::<Vec<Cookie>>();
        Ok(WebDriverResponse::Cookie(CookieResponse::new(response)))
    }

    fn handle_add_cookie(&self, params: &AddCookieParameters) -> WebDriverResult<WebDriverResponse> {
        let (sender, receiver) = ipc::channel().unwrap();

        let cookie = cookie_rs::Cookie::build(params.name.to_owned(), params.value.to_owned())
            .secure(params.secure)
            .http_only(params.httpOnly);
        let cookie = match params.domain {
            Nullable::Value(ref domain) => cookie.domain(domain.to_owned()),
            _ => cookie,
        };
        let cookie = match params.path {
            Nullable::Value(ref path) => cookie.path(path.to_owned()).finish(),
            _ => cookie.finish(),
        };

        let cmd = WebDriverScriptCommand::AddCookie(cookie, sender);
        self.browsing_context_script_command(cmd)?;
        match receiver.recv().unwrap() {
            Ok(_) => Ok(WebDriverResponse::Void),
            Err(response) => match response {
                WebDriverCookieError::InvalidDomain => Err(WebDriverError::new(ErrorStatus::InvalidCookieDomain,
                                                                               "Invalid cookie domain")),
                WebDriverCookieError::UnableToSetCookie => Err(WebDriverError::new(ErrorStatus::UnableToSetCookie,
                                                                                   "Unable to set cookie"))
            }
        }
    }

    fn handle_set_timeouts(&mut self,
                           parameters: &TimeoutsParameters)
                           -> WebDriverResult<WebDriverResponse> {
        let mut session = self.session
            .as_mut()
            .ok_or(WebDriverError::new(ErrorStatus::SessionNotCreated, ""))?;

        session.script_timeout = parameters.script;
        session.load_timeout = parameters.page_load;
        session.implicit_wait_timeout = parameters.implicit;

        Ok(WebDriverResponse::Void)
    }

    fn handle_execute_script(&self, parameters: &JavascriptCommandParameters)
                             -> WebDriverResult<WebDriverResponse> {
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

    fn handle_execute_async_script(&self,
                                   parameters: &JavascriptCommandParameters)
                                   -> WebDriverResult<WebDriverResponse> {
        let func_body = &parameters.script;
        let args_string = "window.webdriverCallback";

        let script = match self.session()?.script_timeout {
            Some(timeout) => {
                format!("setTimeout(webdriverTimeout, {}); (function(callback) {{ {} }})({})",
                        timeout,
                        func_body,
                        args_string)
            }
            None => format!("(function(callback) {{ {} }})({})", func_body, args_string),
        };

        let (sender, receiver) = ipc::channel().unwrap();
        let command = WebDriverScriptCommand::ExecuteAsyncScript(script, sender);
        self.browsing_context_script_command(command)?;
        let result = receiver.recv().unwrap();
        self.postprocess_js_result(result)
    }

    fn postprocess_js_result(&self, result: WebDriverJSResult) -> WebDriverResult<WebDriverResponse> {
        match result {
            Ok(value) => Ok(WebDriverResponse::Generic(ValueResponse::new(value.to_json()))),
            Err(WebDriverJSError::Timeout) => Err(WebDriverError::new(ErrorStatus::Timeout, "")),
            Err(WebDriverJSError::UnknownType) => Err(WebDriverError::new(
                ErrorStatus::UnsupportedOperation, "Unsupported return type")),
            Err(WebDriverJSError::BrowsingContextNotFound) => Err(WebDriverError::new(
                ErrorStatus::JavascriptError, "Pipeline id not found in browsing context"))
        }
    }

    fn handle_element_send_keys(&self,
                                element: &WebElement,
                                keys: &SendKeysParameters) -> WebDriverResult<WebDriverResponse> {
        let browsing_context_id = self.session()?.browsing_context_id;

        let (sender, receiver) = ipc::channel().unwrap();

        let cmd = WebDriverScriptCommand::FocusElement(element.id.clone(), sender);
        let cmd_msg = WebDriverCommandMsg::ScriptCommand(browsing_context_id, cmd);
        self.constellation_chan.send(ConstellationMsg::WebDriverCommand(cmd_msg)).unwrap();

        // TODO: distinguish the not found and not focusable cases
        receiver.recv().unwrap().or_else(|_| Err(WebDriverError::new(
            ErrorStatus::StaleElementReference, "Element not found or not focusable")))?;

        let keys = keycodes_to_keys(&keys.value).or_else(|_|
            Err(WebDriverError::new(ErrorStatus::UnsupportedOperation, "Failed to convert keycodes")))?;

        // TODO: there's a race condition caused by the focus command and the
        // send keys command being two separate messages,
        // so the constellation may have changed state between them.
        let cmd_msg = WebDriverCommandMsg::SendKeys(browsing_context_id, keys);
        self.constellation_chan.send(ConstellationMsg::WebDriverCommand(cmd_msg)).unwrap();

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
            self.constellation_chan.send(ConstellationMsg::WebDriverCommand(cmd_msg)).unwrap();

            if let Some(x) = receiver.recv().unwrap() {
                img = Some(x);
                break;
            };

            thread::sleep(Duration::from_millis(interval))
        }

        let img = match img {
            Some(img) => img,
            None => return Err(WebDriverError::new(ErrorStatus::Timeout,
                                                   "Taking screenshot timed out")),
        };

        // The compositor always sends RGB pixels.
        assert!(img.format == PixelFormat::RGB8, "Unexpected screenshot pixel format");
        let rgb = RgbImage::from_raw(img.width, img.height, img.bytes.to_vec()).unwrap();

        let mut png_data = Vec::new();
        DynamicImage::ImageRgb8(rgb).save(&mut png_data, ImageFormat::PNG).unwrap();

        let encoded = base64::encode(&png_data);
        Ok(WebDriverResponse::Generic(ValueResponse::new(encoded.to_json())))
    }

    fn handle_get_prefs(&self,
                        parameters: &GetPrefsParameters) -> WebDriverResult<WebDriverResponse> {
        let prefs = parameters.prefs
            .iter()
            .map(|item| (item.clone(), PREFS.get(item).to_json()))
            .collect::<BTreeMap<_, _>>();

        Ok(WebDriverResponse::Generic(ValueResponse::new(prefs.to_json())))
    }

    fn handle_set_prefs(&self,
                        parameters: &SetPrefsParameters) -> WebDriverResult<WebDriverResponse> {
        for &(ref key, ref value) in parameters.prefs.iter() {
            PREFS.set(key, value.clone());
        }
        Ok(WebDriverResponse::Void)
    }

    fn handle_reset_prefs(&self,
                          parameters: &GetPrefsParameters) -> WebDriverResult<WebDriverResponse> {
        let prefs = if parameters.prefs.len() == 0 {
            PREFS.reset_all();
            BTreeMap::new()
        } else {
            parameters.prefs
                .iter()
                .map(|item| (item.clone(), PREFS.reset(item).to_json()))
                .collect::<BTreeMap<_, _>>()
        };
        Ok(WebDriverResponse::Generic(ValueResponse::new(prefs.to_json())))
    }
}

impl WebDriverHandler<ServoExtensionRoute> for Handler {
    fn handle_command(&mut self,
                      _session: &Option<Session>,
                      msg: WebDriverMessage<ServoExtensionRoute>) -> WebDriverResult<WebDriverResponse> {
        // Unless we are trying to create a new session, we need to ensure that a
        // session has previously been created
        match msg.command {
            WebDriverCommand::NewSession(_) => {},
            _ => {
                self.session()?;
            }
        }

        match msg.command {
            WebDriverCommand::NewSession(_) => self.handle_new_session(),
            WebDriverCommand::DeleteSession => self.handle_delete_session(),
            WebDriverCommand::AddCookie(ref parameters) => self.handle_add_cookie(parameters),
            WebDriverCommand::Get(ref parameters) => self.handle_get(parameters),
            WebDriverCommand::GetCurrentUrl => self.handle_current_url(),
            WebDriverCommand::GetWindowSize => self.handle_window_size(),
            WebDriverCommand::SetWindowSize(ref size) => self.handle_set_window_size(size),
            WebDriverCommand::IsEnabled(ref element) => self.handle_is_enabled(element),
            WebDriverCommand::IsSelected(ref element) => self.handle_is_selected(element),
            WebDriverCommand::GoBack => self.handle_go_back(),
            WebDriverCommand::GoForward => self.handle_go_forward(),
            WebDriverCommand::Refresh => self.handle_refresh(),
            WebDriverCommand::GetTitle => self.handle_title(),
            WebDriverCommand::GetWindowHandle => self.handle_window_handle(),
            WebDriverCommand::GetWindowHandles => self.handle_window_handles(),
            WebDriverCommand::SwitchToFrame(ref parameters) => self.handle_switch_to_frame(parameters),
            WebDriverCommand::SwitchToParentFrame => self.handle_switch_to_parent_frame(),
            WebDriverCommand::FindElement(ref parameters) => self.handle_find_element(parameters),
            WebDriverCommand::FindElements(ref parameters) => self.handle_find_elements(parameters),
            WebDriverCommand::GetNamedCookie(ref name) => self.handle_get_cookie(name),
            WebDriverCommand::GetCookies => self.handle_get_cookies(),
            WebDriverCommand::GetActiveElement => self.handle_active_element(),
            WebDriverCommand::GetElementRect(ref element) => self.handle_element_rect(element),
            WebDriverCommand::GetElementText(ref element) => self.handle_element_text(element),
            WebDriverCommand::GetElementTagName(ref element) => self.handle_element_tag_name(element),
            WebDriverCommand::GetElementAttribute(ref element, ref name) =>
                self.handle_element_attribute(element, name),
            WebDriverCommand::GetCSSValue(ref element, ref name) =>
                self.handle_element_css(element, name),
            WebDriverCommand::ExecuteScript(ref x) => self.handle_execute_script(x),
            WebDriverCommand::ExecuteAsyncScript(ref x) => self.handle_execute_async_script(x),
            WebDriverCommand::ElementSendKeys(ref element, ref keys) =>
                self.handle_element_send_keys(element, keys),
            WebDriverCommand::SetTimeouts(ref x) => self.handle_set_timeouts(x),
            WebDriverCommand::TakeScreenshot => self.handle_take_screenshot(),
            WebDriverCommand::Extension(ref extension) => {
                match *extension {
                    ServoExtensionCommand::GetPrefs(ref x) => self.handle_get_prefs(x),
                    ServoExtensionCommand::SetPrefs(ref x) => self.handle_set_prefs(x),
                    ServoExtensionCommand::ResetPrefs(ref x) => self.handle_reset_prefs(x),
                }
            }
            _ => Err(WebDriverError::new(ErrorStatus::UnsupportedOperation,
                                         "Command not implemented"))
        }
    }

    fn delete_session(&mut self, _session: &Option<Session>) {
        // Servo doesn't support multiple sessions, so we exit on session deletion
        let _ = self.constellation_chan.send(ConstellationMsg::Exit);
        self.session = None;
    }
}
