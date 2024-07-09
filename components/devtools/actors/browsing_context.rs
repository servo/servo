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
use devtools_traits::DevtoolScriptControlMsg::{self, WantsLiveNotifications};
use devtools_traits::{ConsoleLog, DevtoolsPageInfo, NavigationState, PageError};
use ipc_channel::ipc::IpcSender;
use serde::Serialize;
use serde_json::{Map, Value};

use crate::actor::{Actor, ActorMessageStatus, ActorRegistry};
use crate::actors::configuration::{TargetConfigurationActor, ThreadConfigurationActor};
use crate::actors::emulation::EmulationActor;
use crate::actors::inspector::InspectorActor;
use crate::actors::performance::PerformanceActor;
use crate::actors::profiler::ProfilerActor;
use crate::actors::stylesheets::StyleSheetsActor;
use crate::actors::tab::TabDescriptorActor;
use crate::actors::thread::ThreadActor;
use crate::actors::timeline::TimelineActor;
use crate::actors::watcher::{SessionContext, SessionContextType, WatcherActor};
use crate::protocol::JsonPacketStream;
use crate::StreamId;

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
struct ResourceAvailableReply {
    from: String,
    #[serde(rename = "type")]
    type_: String,
    resources: Vec<ResourceAvailableMsg>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ResourceAvailableMsg {
    #[serde(rename = "hasNativeConsoleAPI")]
    has_native_console_api: Option<bool>,
    name: String,
    #[serde(rename = "newURI")]
    new_uri: Option<String>,
    resource_type: String,
    time: u64,
    title: Option<String>,
    url: Option<String>,
}

#[derive(Serialize)]
struct ConsoleMsg {
    from: String,
    #[serde(rename = "type")]
    type_: String,
    resources: Vec<ConsoleMessageResource>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ConsoleMessageResource {
    message: ConsoleLog,
    resource_type: String,
}

#[derive(Serialize)]
struct PageErrorMsg {
    from: String,
    #[serde(rename = "type")]
    type_: String,
    resources: Vec<PageErrorResource>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PageErrorResource {
    page_error: PageError,
    resource_type: String,
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
    is_browsing_context: bool,
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
    console_actor: String,
    thread_actor: String,
    traits: BrowsingContextTraits,
    // Part of the official protocol, but not yet implemented.
    // emulation_actor: String,
    // inspector_actor: String,
    // timeline_actor: String,
    // profiler_actor: String,
    // performance_actor: String,
    // style_sheets_actor: String,
    // storage_actor: String,
    // memory_actor: String,
    // framerate_actor: String,
    // reflow_actor: String,
    // css_properties_actor: String,
    // animations_actor: String,
    // web_extension_inspected_window_actor: String,
    // accessibility_actor: String,
    // screenshot_actor: String,
    // changes_actor: String,
    // web_socket_actor: String,
    // manifest_actor: String,
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
    pub console: String,
    pub _emulation: String,
    pub _inspector: String,
    pub _performance: String,
    pub _profiler: String,
    pub _style_sheets: String,
    pub target_configuration: String,
    pub thread_configuration: String,
    pub thread: String,
    pub _timeline: String,
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
        _msg_type: &str,
        _msg: &Map<String, Value>,
        _stream: &mut TcpStream,
        _id: StreamId,
    ) -> Result<ActorMessageStatus, ()> {
        Ok(ActorMessageStatus::Ignored)
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
        id: BrowsingContextId,
        page_info: DevtoolsPageInfo,
        pipeline: PipelineId,
        script_sender: IpcSender<DevtoolScriptControlMsg>,
        actors: &mut ActorRegistry,
    ) -> BrowsingContextActor {
        let name = actors.new_name("target");
        let DevtoolsPageInfo { title, url } = page_info;

        let emulation = EmulationActor::new(actors.new_name("emulation"));

        let inspector = InspectorActor {
            name: actors.new_name("inspector"),
            walker: RefCell::new(None),
            page_style: RefCell::new(None),
            highlighter: RefCell::new(None),
            script_chan: script_sender.clone(),
            browsing_context: name.clone(),
        };

        let performance = PerformanceActor::new(actors.new_name("performance"));

        let profiler = ProfilerActor::new(actors.new_name("profiler"));

        // the strange switch between styleSheets and stylesheets is due
        // to an inconsistency in devtools. See Bug #1498893 in bugzilla
        let style_sheets = StyleSheetsActor::new(actors.new_name("stylesheets"));

        let tabdesc = TabDescriptorActor::new(actors, name.clone());

        let target_configuration =
            TargetConfigurationActor::new(actors.new_name("target-configuration"));

        let thread_configuration =
            ThreadConfigurationActor::new(actors.new_name("thread-configuration"));

        let thread = ThreadActor::new(actors.new_name("context"));

        let timeline =
            TimelineActor::new(actors.new_name("timeline"), pipeline, script_sender.clone());

        let watcher = WatcherActor::new(
            actors.new_name("watcher"),
            name.clone(),
            SessionContext::new(SessionContextType::BrowserElement),
        );

        let target = BrowsingContextActor {
            name,
            script_chan: script_sender,
            title: RefCell::new(title),
            url: RefCell::new(url.into_string()),
            active_pipeline: Cell::new(pipeline),
            browsing_context_id: id,
            console,
            _emulation: emulation.name(),
            _inspector: inspector.name(),
            _performance: performance.name(),
            _profiler: profiler.name(),
            streams: RefCell::new(HashMap::new()),
            _style_sheets: style_sheets.name(),
            _tab: tabdesc.name(),
            target_configuration: target_configuration.name(),
            thread_configuration: thread_configuration.name(),
            thread: thread.name(),
            _timeline: timeline.name(),
            watcher: watcher.name(),
        };

        actors.register(Box::new(emulation));
        actors.register(Box::new(inspector));
        actors.register(Box::new(performance));
        actors.register(Box::new(profiler));
        actors.register(Box::new(style_sheets));
        actors.register(Box::new(tabdesc));
        actors.register(Box::new(target_configuration));
        actors.register(Box::new(thread_configuration));
        actors.register(Box::new(thread));
        actors.register(Box::new(timeline));
        actors.register(Box::new(watcher));

        target
    }

    pub fn encodable(&self) -> BrowsingContextActorMsg {
        BrowsingContextActorMsg {
            actor: self.name(),
            traits: BrowsingContextTraits {
                is_browsing_context: true,
            },
            title: self.title.borrow().clone(),
            url: self.url.borrow().clone(),
            //FIXME: shouldn't ignore pipeline namespace field
            browsing_context_id: self.browsing_context_id.index.0.get(),
            //FIXME: shouldn't ignore pipeline namespace field
            outer_window_id: self.active_pipeline.get().index.0.get(),
            is_top_level_target: true,
            console_actor: self.console.clone(),
            thread_actor: self.thread.clone(),
            // emulation_actor: self.emulation.clone(),
            // inspector_actor: self.inspector.clone(),
            // performance_actor: self.performance.clone(),
            // profiler_actor: self.profiler.clone(),
            // style_sheets_actor: self.style_sheets.clone(),
            // timeline_actor: self.timeline.clone(),
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

    pub(crate) fn title_changed(&self, pipeline: PipelineId, title: String) {
        if pipeline != self.active_pipeline.get() {
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

    pub(crate) fn document_event(&self, stream: &mut TcpStream) {
        // TODO: This is a hacky way of sending the 3 messages
        //       Figure out if there needs work to be done here, ensure the page is loaded
        for (i, &name) in ["dom-loading", "dom-interactive", "dom-complete"]
            .iter()
            .enumerate()
        {
            let _ = stream.write_json_packet(&ResourceAvailableReply {
                from: self.name(),
                type_: "resource-available-form".into(),
                resources: vec![ResourceAvailableMsg {
                    has_native_console_api: Some(true),
                    name: name.into(),
                    new_uri: None,
                    resource_type: "document-event".into(),
                    time: i as u64,
                    title: Some(self.title.borrow().clone()),
                    url: Some(self.url.borrow().clone()),
                }],
            });
        }
    }

    pub(crate) fn console_message(&self, message: ConsoleLog) {
        let msg = ConsoleMsg {
            from: self.name(),
            type_: "resource-available-form".into(),
            resources: vec![ConsoleMessageResource {
                message,
                resource_type: "console-message".into(),
            }],
        };

        for stream in self.streams.borrow_mut().values_mut() {
            let _ = stream.write_json_packet(&msg);
        }
    }

    pub(crate) fn page_error(&self, page_error: PageError) {
        let msg = PageErrorMsg {
            from: self.name(),
            type_: "resource-available-form".into(),
            resources: vec![PageErrorResource {
                page_error,
                resource_type: "error-message".into(),
            }],
        };

        for stream in self.streams.borrow_mut().values_mut() {
            let _ = stream.write_json_packet(&msg);
        }
    }
}
