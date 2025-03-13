/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Liberally derived from the [Firefox JS implementation](https://searchfox.org/mozilla-central/source/devtools/server/actors/webbrowser.js).
//! Connection point for remote devtools that wish to investigate a particular Browsing Context's contents.
//! Supports dynamic attaching and detaching which control notifications of navigation, etc.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::net::TcpStream;

use base::id::{BrowsingContextId, PipelineId};
use devtools_traits::DevtoolScriptControlMsg::{self, GetCssDatabase, WantsLiveNotifications};
use devtools_traits::{DevtoolsPageInfo, NavigationState};
use ipc_channel::ipc::{self, IpcSender};
use serde::Serialize;
use serde_json::{Map, Value};

use crate::actor::{Actor, ActorMessageStatus, ActorRegistry};
use crate::actors::inspector::InspectorActor;
use crate::actors::inspector::accessibility::AccessibilityActor;
use crate::actors::inspector::css_properties::CssPropertiesActor;
use crate::actors::reflow::ReflowActor;
use crate::actors::stylesheets::StyleSheetsActor;
use crate::actors::tab::TabDescriptorActor;
use crate::actors::thread::ThreadActor;
use crate::actors::watcher::{SessionContext, SessionContextType, WatcherActor};
use crate::protocol::JsonPacketStream;
use crate::{EmptyReplyMsg, StreamId};

#[derive(Serialize)]
struct FrameUpdateReply {
    from: String,
    #[serde(rename = "type")]
    type_: String,
    frames: Vec<FrameUpdateMsg>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FrameUpdateMsg {
    id: u32,
    is_top_level: bool,
    url: String,
    title: String,
}

#[derive(Serialize)]
struct ResourceAvailableReply<T: Serialize> {
    from: String,
    #[serde(rename = "type")]
    type_: String,
    array: Vec<(String, Vec<T>)>,
}

#[derive(Serialize)]
struct TabNavigated {
    from: String,
    #[serde(rename = "type")]
    type_: String,
    url: String,
    title: Option<String>,
    #[serde(rename = "nativeConsoleAPI")]
    native_console_api: bool,
    state: String,
    is_frame_switching: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BrowsingContextTraits {
    frames: bool,
    is_browsing_context: bool,
    log_in_page: bool,
    navigation: bool,
    supports_top_level_target_flag: bool,
    watchpoints: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowsingContextActorMsg {
    actor: String,
    title: String,
    url: String,
    #[serde(rename = "outerWindowID")]
    outer_window_id: u32,
    #[serde(rename = "browsingContextID")]
    browsing_context_id: u32,
    is_top_level_target: bool,
    traits: BrowsingContextTraits,
    // Implemented actors
    accessibility_actor: String,
    console_actor: String,
    css_properties_actor: String,
    inspector_actor: String,
    reflow_actor: String,
    style_sheets_actor: String,
    thread_actor: String,
    // Part of the official protocol, but not yet implemented.
    // animations_actor: String,
    // changes_actor: String,
    // framerate_actor: String,
    // manifest_actor: String,
    // memory_actor: String,
    // network_content_actor: String,
    // objects_manager: String,
    // performance_actor: String,
    // resonsive_actor: String,
    // storage_actor: String,
    // tracer_actor: String,
    // web_extension_inspected_window_actor: String,
    // web_socket_actor: String,
}

/// The browsing context actor encompasses all of the other supporting actors when debugging a web
/// view. To this extent, it contains a watcher actor that helps when communicating with the host,
/// as well as resource actors that each perform one debugging function.
pub(crate) struct BrowsingContextActor {
    pub name: String,
    pub title: RefCell<String>,
    pub url: RefCell<String>,
    pub active_pipeline: Cell<PipelineId>,
    pub browsing_context_id: BrowsingContextId,
    pub accessibility: String,
    pub console: String,
    pub css_properties: String,
    pub inspector: String,
    pub reflow: String,
    pub style_sheets: String,
    pub thread: String,
    pub _tab: String,
    pub script_chan: IpcSender<DevtoolScriptControlMsg>,
    pub streams: RefCell<HashMap<StreamId, TcpStream>>,
    pub watcher: String,
}

impl Actor for BrowsingContextActor {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn handle_message(
        &self,
        _registry: &ActorRegistry,
        msg_type: &str,
        _msg: &Map<String, Value>,
        stream: &mut TcpStream,
        _id: StreamId,
    ) -> Result<ActorMessageStatus, ()> {
        Ok(match msg_type {
            "listFrames" => {
                // TODO: Find out what needs to be listed here
                let msg = EmptyReplyMsg { from: self.name() };
                let _ = stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            },
            _ => ActorMessageStatus::Ignored,
        })
    }

    fn cleanup(&self, id: StreamId) {
        self.streams.borrow_mut().remove(&id);
        if self.streams.borrow().is_empty() {
            self.script_chan
                .send(WantsLiveNotifications(self.active_pipeline.get(), false))
                .unwrap();
        }
    }
}

impl BrowsingContextActor {
    pub(crate) fn new(
        console: String,
        browsing_context_id: BrowsingContextId,
        page_info: DevtoolsPageInfo,
        pipeline_id: PipelineId,
        script_sender: IpcSender<DevtoolScriptControlMsg>,
        actors: &mut ActorRegistry,
    ) -> BrowsingContextActor {
        let name = actors.new_name("target");
        let DevtoolsPageInfo {
            title,
            url,
            is_top_level_global,
        } = page_info;

        let accessibility = AccessibilityActor::new(actors.new_name("accessibility"));

        let properties = (|| {
            let (properties_sender, properties_receiver) = ipc::channel().ok()?;
            script_sender.send(GetCssDatabase(properties_sender)).ok()?;
            properties_receiver.recv().ok()
        })()
        .unwrap_or_default();
        let css_properties = CssPropertiesActor::new(actors.new_name("css-properties"), properties);

        let inspector = InspectorActor {
            name: actors.new_name("inspector"),
            walker: RefCell::new(None),
            page_style: RefCell::new(None),
            highlighter: RefCell::new(None),
            script_chan: script_sender.clone(),
            browsing_context: name.clone(),
        };

        let reflow = ReflowActor::new(actors.new_name("reflow"));

        let style_sheets = StyleSheetsActor::new(actors.new_name("stylesheets"));

        let tabdesc = TabDescriptorActor::new(actors, name.clone(), is_top_level_global);

        let thread = ThreadActor::new(actors.new_name("thread"));

        let watcher = WatcherActor::new(
            actors,
            name.clone(),
            SessionContext::new(SessionContextType::BrowserElement),
        );

        let target = BrowsingContextActor {
            name,
            script_chan: script_sender,
            title: RefCell::new(title),
            url: RefCell::new(url.into_string()),
            active_pipeline: Cell::new(pipeline_id),
            browsing_context_id,
            accessibility: accessibility.name(),
            console,
            css_properties: css_properties.name(),
            inspector: inspector.name(),
            reflow: reflow.name(),
            streams: RefCell::new(HashMap::new()),
            style_sheets: style_sheets.name(),
            _tab: tabdesc.name(),
            thread: thread.name(),
            watcher: watcher.name(),
        };

        actors.register(Box::new(accessibility));
        actors.register(Box::new(css_properties));
        actors.register(Box::new(inspector));
        actors.register(Box::new(reflow));
        actors.register(Box::new(style_sheets));
        actors.register(Box::new(tabdesc));
        actors.register(Box::new(thread));
        actors.register(Box::new(watcher));

        target
    }

    pub fn encodable(&self) -> BrowsingContextActorMsg {
        BrowsingContextActorMsg {
            actor: self.name(),
            traits: BrowsingContextTraits {
                is_browsing_context: true,
                frames: true,
                log_in_page: false,
                navigation: true,
                supports_top_level_target_flag: true,
                watchpoints: true,
            },
            title: self.title.borrow().clone(),
            url: self.url.borrow().clone(),
            //FIXME: shouldn't ignore pipeline namespace field
            browsing_context_id: self.browsing_context_id.index.0.get(),
            //FIXME: shouldn't ignore pipeline namespace field
            outer_window_id: self.active_pipeline.get().index.0.get(),
            is_top_level_target: true,
            accessibility_actor: self.accessibility.clone(),
            console_actor: self.console.clone(),
            css_properties_actor: self.css_properties.clone(),
            inspector_actor: self.inspector.clone(),
            reflow_actor: self.reflow.clone(),
            style_sheets_actor: self.style_sheets.clone(),
            thread_actor: self.thread.clone(),
        }
    }

    pub(crate) fn navigate(&self, state: NavigationState) {
        let (pipeline, title, url, state) = match state {
            NavigationState::Start(url) => (None, None, url, "start"),
            NavigationState::Stop(pipeline, info) => {
                (Some(pipeline), Some(info.title), info.url, "stop")
            },
        };
        if let Some(p) = pipeline {
            self.active_pipeline.set(p);
        }
        url.as_str().clone_into(&mut self.url.borrow_mut());
        if let Some(ref t) = title {
            self.title.borrow_mut().clone_from(t);
        }

        let msg = TabNavigated {
            from: self.name(),
            type_: "tabNavigated".to_owned(),
            url: url.as_str().to_owned(),
            title,
            native_console_api: true,
            state: state.to_owned(),
            is_frame_switching: false,
        };

        for stream in self.streams.borrow_mut().values_mut() {
            let _ = stream.write_json_packet(&msg);
        }
    }

    pub(crate) fn title_changed(&self, pipeline_id: PipelineId, title: String) {
        if pipeline_id != self.active_pipeline.get() {
            return;
        }
        *self.title.borrow_mut() = title;
    }

    pub(crate) fn frame_update(&self, stream: &mut TcpStream) {
        let _ = stream.write_json_packet(&FrameUpdateReply {
            from: self.name(),
            type_: "frameUpdate".into(),
            frames: vec![FrameUpdateMsg {
                id: self.browsing_context_id.index.0.get(),
                is_top_level: true,
                title: self.title.borrow().clone(),
                url: self.url.borrow().clone(),
            }],
        });
    }

    pub(crate) fn resource_available<T: Serialize>(&self, message: T, resource_type: String) {
        let msg = ResourceAvailableReply::<T> {
            from: self.name(),
            type_: "resources-available-array".into(),
            array: vec![(resource_type, vec![message])],
        };

        for stream in self.streams.borrow_mut().values_mut() {
            let _ = stream.write_json_packet(&msg);
        }
    }
}
