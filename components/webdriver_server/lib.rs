/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![crate_name = "webdriver_server"]
#![crate_type = "rlib"]

#![feature(plugin)]
#![plugin(plugins)]

#![deny(unsafe_code)]

extern crate compositing;
extern crate hyper;
extern crate image;
extern crate ipc_channel;
extern crate msg;
extern crate regex;
extern crate rustc_serialize;
extern crate url;
extern crate util;
extern crate uuid;
extern crate webdriver;

mod keys;

use compositing::CompositorMsg as ConstellationMsg;
use hyper::method::Method::{self, Post};
use image::{DynamicImage, ImageFormat, RgbImage};
use ipc_channel::ipc::{self, IpcReceiver, IpcSender};
use keys::keycodes_to_keys;
use msg::constellation_msg::{FrameId, LoadData, PipelineId};
use msg::constellation_msg::{NavigationDirection, PixelFormat, WebDriverCommandMsg};
use msg::webdriver_msg::{LoadStatus, WebDriverFrameId, WebDriverJSError, WebDriverJSResult, WebDriverScriptCommand};
use regex::Captures;
use rustc_serialize::base64::{CharacterSet, Config, Newline, ToBase64};
use rustc_serialize::json::{Json, ToJson};
use std::borrow::ToOwned;
use std::collections::BTreeMap;
use std::net::{SocketAddr, SocketAddrV4};
use std::sync::mpsc::Sender;
use std::thread;
use std::time::Duration;
use url::Url;
use util::prefs::{get_pref, reset_all_prefs, reset_pref, set_pref, PrefValue};
use util::thread::spawn_named;
use uuid::Uuid;
use webdriver::command::{GetParameters, JavascriptCommandParameters, LocatorParameters};
use webdriver::command::{Parameters, SendKeysParameters, SwitchToFrameParameters, TimeoutsParameters};
use webdriver::command::{WebDriverCommand, WebDriverExtensionCommand, WebDriverMessage};
use webdriver::common::{LocatorStrategy, WebElement};
use webdriver::error::{ErrorStatus, WebDriverError, WebDriverResult};
use webdriver::httpapi::{WebDriverExtensionRoute};
use webdriver::response::{ElementRectResponse, NewSessionResponse, ValueResponse};
use webdriver::response::{WebDriverResponse, WindowSizeResponse};
use webdriver::server::{self, Session, WebDriverHandler};

fn extension_routes() -> Vec<(Method, &'static str, ServoExtensionRoute)> {
    return vec![(Post, "/session/{sessionId}/servo/prefs/get", ServoExtensionRoute::GetPrefs),
                (Post, "/session/{sessionId}/servo/prefs/set", ServoExtensionRoute::SetPrefs),
                (Post, "/session/{sessionId}/servo/prefs/reset", ServoExtensionRoute::ResetPrefs)]
}

pub fn start_server(port: u16, constellation_chan: Sender<ConstellationMsg>) {
    let handler = Handler::new(constellation_chan);
    spawn_named("WebdriverHttpServer".to_owned(), move || {
        let address = SocketAddrV4::new("0.0.0.0".parse().unwrap(), port);
        server::start(SocketAddr::V4(address), handler,
                      extension_routes());
    });
}

struct WebDriverSession {
    id: Uuid,
    frame_id: Option<FrameId>
}

struct Handler {
    session: Option<WebDriverSession>,
    constellation_chan: Sender<ConstellationMsg>,
    script_timeout: u32,
    load_timeout: u32,
    implicit_wait_timeout: u32
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
                let parameters: GetPrefsParameters = try!(Parameters::from_json(&body_data));
                ServoExtensionCommand::GetPrefs(parameters)
            }
            ServoExtensionRoute::SetPrefs => {
                let parameters: SetPrefsParameters = try!(Parameters::from_json(&body_data));
                ServoExtensionCommand::SetPrefs(parameters)
            }
            ServoExtensionRoute::ResetPrefs => {
                let parameters: GetPrefsParameters = try!(Parameters::from_json(&body_data));
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
        let data = try!(body.as_object().ok_or(
            WebDriverError::new(ErrorStatus::InvalidArgument,
                                "Message body was not an object")));
        let prefs_value = try!(data.get("prefs").ok_or(
            WebDriverError::new(ErrorStatus::InvalidArgument,
                                "Missing prefs key")));
        let items = try!(prefs_value.as_array().ok_or(
            WebDriverError::new(
                ErrorStatus::InvalidArgument,
                "prefs was not an array")));
        let params = try!(items.iter().map(|x| x.as_string().map(|y| y.to_owned()).ok_or(
            WebDriverError::new(ErrorStatus::InvalidArgument,
                                "Pref is not a string"))).collect::<Result<Vec<_>, _>>());
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
        let data = try!(body.as_object().ok_or(
            WebDriverError::new(ErrorStatus::InvalidArgument,
                                "Message body was not an object")));
        let items = try!(try!(data.get("prefs").ok_or(
            WebDriverError::new(ErrorStatus::InvalidArgument,
                                "Missing prefs key"))).as_object().ok_or(
            WebDriverError::new(
                ErrorStatus::InvalidArgument,
                "prefs was not an array")));
        let mut params = Vec::with_capacity(items.len());
        for (name, val) in items.iter() {
            let value = try!(PrefValue::from_json(val.clone()).or(
                Err(WebDriverError::new(ErrorStatus::InvalidArgument,
                                        "Pref is not a boolean or string"))));
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

impl WebDriverSession {
    pub fn new() -> WebDriverSession {
        WebDriverSession {
            id: Uuid::new_v4(),
            frame_id: None
        }
    }
}

impl Handler {
    pub fn new(constellation_chan: Sender<ConstellationMsg>) -> Handler {
        Handler {
            session: None,
            constellation_chan: constellation_chan,
            script_timeout: 30_000,
            load_timeout: 300_000,
            implicit_wait_timeout: 0
        }
    }

    fn root_pipeline(&self) -> WebDriverResult<PipelineId> {
        let interval = 20;
        let iterations = 30_000 / interval;

        for _ in 0..iterations {
            if let Some(x) = self.pipeline(None) {
                return Ok(x)
            };

            thread::sleep(Duration::from_millis(interval));
        };

        Err(WebDriverError::new(ErrorStatus::Timeout,
                                "Failed to get root window handle"))
    }

    fn frame_pipeline(&self) -> WebDriverResult<PipelineId> {
        if let Some(ref session) = self.session {
            match self.pipeline(session.frame_id) {
                Some(x) => Ok(x),
                None => Err(WebDriverError::new(ErrorStatus::NoSuchFrame,
                                                "Frame got closed"))
            }
        } else {
            panic!("Command tried to access session but session is None");
        }
    }

    fn session(&self) -> WebDriverResult<&WebDriverSession> {
        match self.session {
            Some(ref x) => Ok(x),
            None => Err(WebDriverError::new(ErrorStatus::SessionNotCreated,
                                            "Session not created"))
        }
    }

    fn set_frame_id(&mut self, frame_id: Option<FrameId>) -> WebDriverResult<()> {
        match self.session {
            Some(ref mut x) => {
                x.frame_id = frame_id;
                Ok(())
            },
            None => Err(WebDriverError::new(ErrorStatus::SessionNotCreated,
                                            "Session not created"))
        }
    }

    fn pipeline(&self, frame_id: Option<FrameId>) -> Option<PipelineId> {
        let (sender, receiver) = ipc::channel().unwrap();
        self.constellation_chan.send(ConstellationMsg::GetPipeline(frame_id, sender)).unwrap();


        receiver.recv().unwrap()
    }

    fn handle_new_session(&mut self) -> WebDriverResult<WebDriverResponse> {
        if self.session.is_none() {
            let session = WebDriverSession::new();
            let mut capabilities = BTreeMap::new();
            capabilities.insert("browserName".to_owned(), "servo".to_json());
            capabilities.insert("browserVersion".to_owned(), "0.0.1".to_json());
            capabilities.insert("acceptSslCerts".to_owned(), false.to_json());
            capabilities.insert("takeScreenshot".to_owned(), true.to_json());
            capabilities.insert("takeElementScreenshot".to_owned(), false.to_json());
            let rv = Ok(WebDriverResponse::NewSession(
                NewSessionResponse::new(
                    session.id.to_string(),
                    Json::Object(capabilities))));
            self.session = Some(session);
            rv
        } else {
            Err(WebDriverError::new(ErrorStatus::UnknownError,
                                    "Session already created"))
        }
    }

    #[inline]
    fn frame_script_command(&self, cmd_msg: WebDriverScriptCommand) -> WebDriverResult<()> {
        Ok(self.constellation_chan.send(ConstellationMsg::WebDriverCommand(
                WebDriverCommandMsg::ScriptCommand(try!(self.frame_pipeline()), cmd_msg))).unwrap())
    }

    #[inline]
    fn root_script_command(&self, cmd_msg: WebDriverScriptCommand) -> WebDriverResult<()> {
        Ok(self.constellation_chan.send(ConstellationMsg::WebDriverCommand(
                WebDriverCommandMsg::ScriptCommand(try!(self.root_pipeline()), cmd_msg))).unwrap())
    }

    fn handle_get(&self, parameters: &GetParameters) -> WebDriverResult<WebDriverResponse> {
        let url = match Url::parse(&parameters.url[..]) {
            Ok(url) => url,
            Err(_) => return Err(WebDriverError::new(ErrorStatus::InvalidArgument,
                                               "Invalid URL"))
        };

        let pipeline_id = try!(self.root_pipeline());

        let (sender, receiver) = ipc::channel().unwrap();

        let load_data = LoadData::new(url);
        let cmd_msg = WebDriverCommandMsg::LoadUrl(pipeline_id, load_data, sender.clone());
        self.constellation_chan.send(ConstellationMsg::WebDriverCommand(cmd_msg)).unwrap();

        self.wait_for_load(sender, receiver)
    }

    fn wait_for_load(&self,
                     sender: IpcSender<LoadStatus>,
                     receiver: IpcReceiver<LoadStatus>) -> WebDriverResult<WebDriverResponse> {
        let timeout = self.load_timeout;
        let timeout_chan = sender;
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(timeout as u64));
            let _ = timeout_chan.send(LoadStatus::LoadTimeout);
        });

        //Wait to get a load event
        match receiver.recv().unwrap() {
            LoadStatus::LoadComplete => Ok(WebDriverResponse::Void),
            LoadStatus::LoadTimeout => Err(WebDriverError::new(ErrorStatus::Timeout,
                                                               "Load timed out"))
        }
    }

    fn handle_current_url(&self) -> WebDriverResult<WebDriverResponse> {
        let (sender, receiver) = ipc::channel().unwrap();
        try!(self.root_script_command(WebDriverScriptCommand::GetUrl(sender)));

        let url = receiver.recv().unwrap();

        Ok(WebDriverResponse::Generic(ValueResponse::new(url.as_str().to_json())))
    }

    fn handle_window_size(&self) -> WebDriverResult<WebDriverResponse> {
        let (sender, receiver) = ipc::channel().unwrap();

        try!(self.root_script_command(WebDriverScriptCommand::GetWindowSize(sender)));

        match receiver.recv().unwrap() {
            Some(window_size) => {
                let vp = window_size.visible_viewport;
                let window_size_response = WindowSizeResponse::new(vp.width.get() as u64, vp.height.get() as u64);
                Ok(WebDriverResponse::WindowSize(window_size_response))
            },
            None => Err(WebDriverError::new(ErrorStatus::NoSuchWindow, "Unable to determine window size"))
        }
    }

    fn handle_is_enabled(&self, element: &WebElement) -> WebDriverResult<WebDriverResponse> {
        let (sender, receiver) = ipc::channel().unwrap();

        try!(self.root_script_command(WebDriverScriptCommand::IsEnabled(element.id.clone(), sender)));

        match receiver.recv().unwrap() {
            Ok(is_enabled) => Ok(WebDriverResponse::Generic(ValueResponse::new(is_enabled.to_json()))),
            Err(_) => Err(WebDriverError::new(ErrorStatus::StaleElementReference, "Element not found"))
        }
    }

    fn handle_is_selected(&self, element: &WebElement) -> WebDriverResult<WebDriverResponse> {
        let (sender, receiver) = ipc::channel().unwrap();

        try!(self.root_script_command(WebDriverScriptCommand::IsSelected(element.id.clone(), sender)));

        match receiver.recv().unwrap() {
            Ok(is_selected) => Ok(WebDriverResponse::Generic(ValueResponse::new(is_selected.to_json()))),
            Err(_) => Err(WebDriverError::new(ErrorStatus::StaleElementReference, "Element not found"))
        }
    }

    fn handle_go_back(&self) -> WebDriverResult<WebDriverResponse> {
        self.constellation_chan.send(ConstellationMsg::Navigate(None, NavigationDirection::Back)).unwrap();
        Ok(WebDriverResponse::Void)
    }

    fn handle_go_forward(&self) -> WebDriverResult<WebDriverResponse> {
        self.constellation_chan.send(ConstellationMsg::Navigate(None, NavigationDirection::Forward)).unwrap();
        Ok(WebDriverResponse::Void)
    }

    fn handle_refresh(&self) -> WebDriverResult<WebDriverResponse> {
        let pipeline_id = try!(self.root_pipeline());

        let (sender, receiver) = ipc::channel().unwrap();

        let cmd_msg = WebDriverCommandMsg::Refresh(pipeline_id, sender.clone());
        self.constellation_chan.send(ConstellationMsg::WebDriverCommand(cmd_msg)).unwrap();

        self.wait_for_load(sender, receiver)
    }

    fn handle_title(&self) -> WebDriverResult<WebDriverResponse> {
        let (sender, receiver) = ipc::channel().unwrap();

        try!(self.root_script_command(WebDriverScriptCommand::GetTitle(sender)));

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

        try!(self.frame_script_command(WebDriverScriptCommand::FindElementCSS(parameters.value.clone(),
                                                                              sender)));

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
                self.set_frame_id(None).unwrap();
                return Ok(WebDriverResponse::Void)
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
        let pipeline_id = try!(self.frame_pipeline());
        let (sender, receiver) = ipc::channel().unwrap();
        let cmd = WebDriverScriptCommand::GetFrameId(frame_id, sender);
        {
            self.constellation_chan.send(ConstellationMsg::WebDriverCommand(
                WebDriverCommandMsg::ScriptCommand(pipeline_id, cmd))).unwrap();
        }

        let frame = match receiver.recv().unwrap() {
            Ok(Some(pipeline_id)) => {
                let (sender, receiver) = ipc::channel().unwrap();
                self.constellation_chan.send(ConstellationMsg::GetFrame(pipeline_id, sender)).unwrap();
                receiver.recv().unwrap()
            },
            Ok(None) => None,
            Err(_) => {
                return Err(WebDriverError::new(ErrorStatus::NoSuchFrame,
                                               "Frame does not exist"));
            }
        };

        self.set_frame_id(frame).unwrap();
        Ok(WebDriverResponse::Void)
    }


    fn handle_find_elements(&self, parameters: &LocatorParameters) -> WebDriverResult<WebDriverResponse> {
        if parameters.using != LocatorStrategy::CSSSelector {
            return Err(WebDriverError::new(ErrorStatus::UnsupportedOperation,
                                           "Unsupported locator strategy"))
        }

        let (sender, receiver) = ipc::channel().unwrap();
        try!(self.frame_script_command(WebDriverScriptCommand::FindElementCSS(parameters.value.clone(),
                                                                              sender)));
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
        try!(self.frame_script_command(WebDriverScriptCommand::GetElementRect(element.id.clone(), sender)));
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
        try!(self.frame_script_command(WebDriverScriptCommand::GetElementText(element.id.clone(), sender)));
        match receiver.recv().unwrap() {
            Ok(value) => Ok(WebDriverResponse::Generic(ValueResponse::new(value.to_json()))),
            Err(_) => Err(WebDriverError::new(ErrorStatus::StaleElementReference,
                                              "Unable to find element in document"))
        }
    }

    fn handle_active_element(&self) -> WebDriverResult<WebDriverResponse> {
        let (sender, receiver) = ipc::channel().unwrap();
        try!(self.frame_script_command(WebDriverScriptCommand::GetActiveElement(sender)));
        let value = receiver.recv().unwrap().map(|x| WebElement::new(x).to_json());
        Ok(WebDriverResponse::Generic(ValueResponse::new(value.to_json())))
    }

    fn handle_element_tag_name(&self, element: &WebElement) -> WebDriverResult<WebDriverResponse> {
        let (sender, receiver) = ipc::channel().unwrap();
        try!(self.frame_script_command(WebDriverScriptCommand::GetElementTagName(element.id.clone(), sender)));
        match receiver.recv().unwrap() {
            Ok(value) => Ok(WebDriverResponse::Generic(ValueResponse::new(value.to_json()))),
            Err(_) => Err(WebDriverError::new(ErrorStatus::StaleElementReference,
                                              "Unable to find element in document"))
        }
    }

    fn handle_element_attribute(&self, element: &WebElement, name: &str) -> WebDriverResult<WebDriverResponse> {
        let (sender, receiver) = ipc::channel().unwrap();
        try!(self.frame_script_command(WebDriverScriptCommand::GetElementAttribute(element.id.clone(), name.to_owned(),
                                                                                   sender)));
        match receiver.recv().unwrap() {
            Ok(value) => Ok(WebDriverResponse::Generic(ValueResponse::new(value.to_json()))),
            Err(_) => Err(WebDriverError::new(ErrorStatus::StaleElementReference,
                                              "Unable to find element in document"))
        }
    }

    fn handle_element_css(&self, element: &WebElement, name: &str) -> WebDriverResult<WebDriverResponse> {
        let (sender, receiver) = ipc::channel().unwrap();
        try!(self.frame_script_command(WebDriverScriptCommand::GetElementCSS(element.id.clone(), name.to_owned(),
                                                                             sender)));
        match receiver.recv().unwrap() {
            Ok(value) => Ok(WebDriverResponse::Generic(ValueResponse::new(value.to_json()))),
            Err(_) => Err(WebDriverError::new(ErrorStatus::StaleElementReference,
                                              "Unable to find element in document"))
        }
    }

    fn handle_set_timeouts(&mut self, parameters: &TimeoutsParameters) -> WebDriverResult<WebDriverResponse> {
        //TODO: this conversion is crazy, spec should limit these to u32 and check upstream
        let value = parameters.ms as u32;
        match &parameters.type_[..] {
            "implicit" => self.implicit_wait_timeout = value,
            "page load" => self.load_timeout = value,
            "script" => self.script_timeout = value,
            x => return Err(WebDriverError::new(ErrorStatus::InvalidSelector,
                                                format!("Unknown timeout type {}", x)))
        }
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
        self.execute_script(command, receiver)
    }

    fn handle_execute_async_script(&self,
                                   parameters: &JavascriptCommandParameters) -> WebDriverResult<WebDriverResponse> {
        let func_body = &parameters.script;
        let args_string = "window.webdriverCallback";

        let script = format!(
            "setTimeout(webdriverTimeout, {}); (function(callback) {{ {} }})({})",
            self.script_timeout, func_body, args_string);

        let (sender, receiver) = ipc::channel().unwrap();
        let command = WebDriverScriptCommand::ExecuteAsyncScript(script, sender);
        self.execute_script(command, receiver)
    }

    fn execute_script(&self,
                      command: WebDriverScriptCommand,
                      receiver: IpcReceiver<WebDriverJSResult>)
                      -> WebDriverResult<WebDriverResponse> {
        let pipeline_id = try!(self.frame_pipeline());

        let cmd_msg = WebDriverCommandMsg::ScriptCommand(pipeline_id, command);
        self.constellation_chan.send(ConstellationMsg::WebDriverCommand(cmd_msg)).unwrap();

        match receiver.recv().unwrap() {
            Ok(value) => Ok(WebDriverResponse::Generic(ValueResponse::new(value.to_json()))),
            Err(WebDriverJSError::Timeout) => Err(WebDriverError::new(ErrorStatus::Timeout, "")),
            Err(WebDriverJSError::UnknownType) => Err(WebDriverError::new(
                ErrorStatus::UnsupportedOperation, "Unsupported return type"))
        }
    }

    fn handle_element_send_keys(&self,
                                element: &WebElement,
                                keys: &SendKeysParameters) -> WebDriverResult<WebDriverResponse> {
        let pipeline_id = try!(self.frame_pipeline());

        let (sender, receiver) = ipc::channel().unwrap();

        let cmd = WebDriverScriptCommand::FocusElement(element.id.clone(), sender);
        let cmd_msg = WebDriverCommandMsg::ScriptCommand(pipeline_id, cmd);
        self.constellation_chan.send(ConstellationMsg::WebDriverCommand(cmd_msg)).unwrap();

        // TODO: distinguish the not found and not focusable cases
        try!(receiver.recv().unwrap().or_else(|_| Err(WebDriverError::new(
            ErrorStatus::StaleElementReference, "Element not found or not focusable"))));

        let keys = try!(keycodes_to_keys(&keys.value).or_else(|_|
            Err(WebDriverError::new(ErrorStatus::UnsupportedOperation, "Failed to convert keycodes"))));

        let cmd_msg = WebDriverCommandMsg::SendKeys(pipeline_id, keys);
        self.constellation_chan.send(ConstellationMsg::WebDriverCommand(cmd_msg)).unwrap();

        Ok(WebDriverResponse::Void)
    }

    fn handle_take_screenshot(&self) -> WebDriverResult<WebDriverResponse> {
        let mut img = None;
        let pipeline_id = try!(self.root_pipeline());

        let interval = 20;
        let iterations = 30_000 / interval;

        for _ in 0..iterations {
            let (sender, receiver) = ipc::channel().unwrap();
            let cmd_msg = WebDriverCommandMsg::TakeScreenshot(pipeline_id, sender);
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

        let config = Config {
            char_set: CharacterSet::Standard,
            newline: Newline::LF,
            pad: true,
            line_length: None
        };
        let encoded = png_data.to_base64(config);
        Ok(WebDriverResponse::Generic(ValueResponse::new(encoded.to_json())))
    }

    fn handle_get_prefs(&self,
                        parameters: &GetPrefsParameters) -> WebDriverResult<WebDriverResponse> {
        let prefs = parameters.prefs
            .iter()
            .map(|item| (item.clone(), get_pref(item).to_json()))
            .collect::<BTreeMap<_, _>>();

        Ok(WebDriverResponse::Generic(ValueResponse::new(prefs.to_json())))
    }

    fn handle_set_prefs(&self,
                        parameters: &SetPrefsParameters) -> WebDriverResult<WebDriverResponse> {
        for &(ref key, ref value) in parameters.prefs.iter() {
            set_pref(key, value.clone());
        }
        Ok(WebDriverResponse::Void)
    }

    fn handle_reset_prefs(&self,
                          parameters: &GetPrefsParameters) -> WebDriverResult<WebDriverResponse> {
        let prefs = if parameters.prefs.len() == 0 {
            reset_all_prefs();
            BTreeMap::new()
        } else {
            parameters.prefs
                .iter()
                .map(|item| (item.clone(), reset_pref(item).to_json()))
                .collect::<BTreeMap<_, _>>()
        };
        Ok(WebDriverResponse::Generic(ValueResponse::new(prefs.to_json())))
    }
}

impl WebDriverHandler<ServoExtensionRoute> for Handler {
    fn handle_command(&mut self,
                      _session: &Option<Session>,
                      msg: &WebDriverMessage<ServoExtensionRoute>) -> WebDriverResult<WebDriverResponse> {

        // Unless we are trying to create a new session, we need to ensure that a
        // session has previously been created
        match msg.command {
            WebDriverCommand::NewSession => {},
            _ => {
                try!(self.session());
            }
        }

        match msg.command {
            WebDriverCommand::NewSession => self.handle_new_session(),
            WebDriverCommand::Get(ref parameters) => self.handle_get(parameters),
            WebDriverCommand::GetCurrentUrl => self.handle_current_url(),
            WebDriverCommand::GetWindowSize => self.handle_window_size(),
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
        self.session = None;
    }
}
