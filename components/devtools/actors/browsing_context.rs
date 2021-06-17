/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Liberally derived from the [Firefox JS implementation]
//! (http://mxr.mozilla.org/mozilla-central/source/toolkit/devtools/server/actors/webbrowser.js).
//! Connection point for remote devtools that wish to investigate a particular Browsing Context's contents.
//! Supports dynamic attaching and detaching which control notifications of navigation, etc.

use crate::actor::{Actor, ActorMessageStatus, ActorRegistry};
use crate::actors::emulation::EmulationActor;
use crate::actors::inspector::InspectorActor;
use crate::actors::performance::PerformanceActor;
use crate::actors::profiler::ProfilerActor;
use crate::actors::stylesheets::StyleSheetsActor;
use crate::actors::tab::TabDescriptorActor;
use crate::actors::thread::ThreadActor;
use crate::actors::timeline::TimelineActor;
use crate::protocol::JsonPacketStream;
use crate::StreamId;
use devtools_traits::DevtoolScriptControlMsg::{self, WantsLiveNotifications};
use devtools_traits::DevtoolsPageInfo;
use devtools_traits::NavigationState;
use ipc_channel::ipc::IpcSender;
use msg::constellation_msg::{BrowsingContextId, PipelineId};
use serde_json::{Map, Value};
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::net::TcpStream;

#[derive(Serialize)]
struct BrowsingContextTraits {
    isBrowsingContext: bool,
}

#[derive(Serialize)]
struct AttachedTraits {
    reconfigure: bool,
    frames: bool,
    logInPage: bool,
    canRewind: bool,
    watchpoints: bool,
}

#[derive(Serialize)]
struct BrowsingContextAttachedReply {
    from: String,
    #[serde(rename = "type")]
    type_: String,
    threadActor: String,
    cacheDisabled: bool,
    javascriptEnabled: bool,
    traits: AttachedTraits,
}

#[derive(Serialize)]
struct BrowsingContextDetachedReply {
    from: String,
    #[serde(rename = "type")]
    type_: String,
}

#[derive(Serialize)]
struct ReconfigureReply {
    from: String,
}

#[derive(Serialize)]
struct ListFramesReply {
    from: String,
    frames: Vec<FrameMsg>,
}

#[derive(Serialize)]
struct FrameMsg {
    id: u32,
    url: String,
    title: String,
    parentID: u32,
}

#[derive(Serialize)]
struct ListWorkersReply {
    from: String,
    workers: Vec<WorkerMsg>,
}

#[derive(Serialize)]
struct WorkerMsg {
    id: u32,
}

#[derive(Serialize)]
pub struct BrowsingContextActorMsg {
    actor: String,
    title: String,
    url: String,
    outerWindowID: u32,
    browsingContextId: u32,
    consoleActor: String,
    /*emulationActor: String,
    inspectorActor: String,
    timelineActor: String,
    profilerActor: String,
    performanceActor: String,
    styleSheetsActor: String,*/
    traits: BrowsingContextTraits,
    // Part of the official protocol, but not yet implemented.
    /*storageActor: String,
    memoryActor: String,
    framerateActor: String,
    reflowActor: String,
    cssPropertiesActor: String,
    animationsActor: String,
    webExtensionInspectedWindowActor: String,
    accessibilityActor: String,
    screenshotActor: String,
    changesActor: String,
    webSocketActor: String,
    manifestActor: String,*/
}

pub(crate) struct BrowsingContextActor {
    pub name: String,
    pub title: RefCell<String>,
    pub url: RefCell<String>,
    pub console: String,
    pub _emulation: String,
    pub _inspector: String,
    pub _timeline: String,
    pub _profiler: String,
    pub _performance: String,
    pub _styleSheets: String,
    pub thread: String,
    pub _tab: String,
    pub streams: RefCell<HashMap<StreamId, TcpStream>>,
    pub browsing_context_id: BrowsingContextId,
    pub active_pipeline: Cell<PipelineId>,
    pub script_chan: IpcSender<DevtoolScriptControlMsg>,
}

impl Actor for BrowsingContextActor {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn handle_message(
        &self,
        _registry: &ActorRegistry,
        msg_type: &str,
        msg: &Map<String, Value>,
        stream: &mut TcpStream,
        id: StreamId,
    ) -> Result<ActorMessageStatus, ()> {
        Ok(match msg_type {
            "reconfigure" => {
                if let Some(options) = msg.get("options").and_then(|o| o.as_object()) {
                    if let Some(val) = options.get("performReload") {
                        if val.as_bool().unwrap_or(false) {
                            let _ = self
                                .script_chan
                                .send(DevtoolScriptControlMsg::Reload(self.active_pipeline.get()));
                        }
                    }
                }
                let _ = stream.write_json_packet(&ReconfigureReply { from: self.name() });
                ActorMessageStatus::Processed
            },

            // https://docs.firefox-dev.tools/backend/protocol.html#listing-browser-tabs
            // (see "To attach to a _targetActor_")
            "attach" => {
                let msg = BrowsingContextAttachedReply {
                    from: self.name(),
                    type_: "tabAttached".to_owned(),
                    threadActor: self.thread.clone(),
                    cacheDisabled: false,
                    javascriptEnabled: true,
                    traits: AttachedTraits {
                        reconfigure: false,
                        frames: true,
                        logInPage: false,
                        canRewind: false,
                        watchpoints: false,
                    },
                };

                if stream.write_json_packet(&msg).is_err() {
                    return Ok(ActorMessageStatus::Processed);
                }
                self.streams
                    .borrow_mut()
                    .insert(id, stream.try_clone().unwrap());
                self.script_chan
                    .send(WantsLiveNotifications(self.active_pipeline.get(), true))
                    .unwrap();
                ActorMessageStatus::Processed
            },

            "detach" => {
                let msg = BrowsingContextDetachedReply {
                    from: self.name(),
                    type_: "detached".to_owned(),
                };
                let _ = stream.write_json_packet(&msg);
                self.cleanup(id);
                ActorMessageStatus::Processed
            },

            "listFrames" => {
                let msg = ListFramesReply {
                    from: self.name(),
                    frames: vec![FrameMsg {
                        //FIXME: shouldn't ignore pipeline namespace field
                        id: self.active_pipeline.get().index.0.get(),
                        parentID: 0,
                        url: self.url.borrow().clone(),
                        title: self.title.borrow().clone(),
                    }],
                };
                let _ = stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            },

            "listWorkers" => {
                let msg = ListWorkersReply {
                    from: self.name(),
                    workers: vec![],
                };
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
        id: BrowsingContextId,
        page_info: DevtoolsPageInfo,
        pipeline: PipelineId,
        script_sender: IpcSender<DevtoolScriptControlMsg>,
        actors: &mut ActorRegistry,
    ) -> BrowsingContextActor {
        let emulation = EmulationActor::new(actors.new_name("emulation"));

        let name = actors.new_name("target");

        let inspector = InspectorActor {
            name: actors.new_name("inspector"),
            walker: RefCell::new(None),
            pageStyle: RefCell::new(None),
            highlighter: RefCell::new(None),
            script_chan: script_sender.clone(),
            browsing_context: name.clone(),
        };

        let timeline =
            TimelineActor::new(actors.new_name("timeline"), pipeline, script_sender.clone());

        let profiler = ProfilerActor::new(actors.new_name("profiler"));
        let performance = PerformanceActor::new(actors.new_name("performance"));

        // the strange switch between styleSheets and stylesheets is due
        // to an inconsistency in devtools. See Bug #1498893 in bugzilla
        let styleSheets = StyleSheetsActor::new(actors.new_name("stylesheets"));
        let thread = ThreadActor::new(actors.new_name("context"));

        let DevtoolsPageInfo { title, url } = page_info;

        let tabdesc = TabDescriptorActor::new(actors, name.clone());

        let target = BrowsingContextActor {
            name: name,
            script_chan: script_sender,
            title: RefCell::new(String::from(title)),
            url: RefCell::new(url.into_string()),
            console: console,
            _emulation: emulation.name(),
            _inspector: inspector.name(),
            _timeline: timeline.name(),
            _profiler: profiler.name(),
            _performance: performance.name(),
            _styleSheets: styleSheets.name(),
            _tab: tabdesc.name(),
            thread: thread.name(),
            streams: RefCell::new(HashMap::new()),
            browsing_context_id: id,
            active_pipeline: Cell::new(pipeline),
        };

        actors.register(Box::new(emulation));
        actors.register(Box::new(inspector));
        actors.register(Box::new(timeline));
        actors.register(Box::new(profiler));
        actors.register(Box::new(performance));
        actors.register(Box::new(styleSheets));
        actors.register(Box::new(thread));
        actors.register(Box::new(tabdesc));

        target
    }

    pub fn encodable(&self) -> BrowsingContextActorMsg {
        BrowsingContextActorMsg {
            actor: self.name(),
            traits: BrowsingContextTraits {
                isBrowsingContext: true,
            },
            title: self.title.borrow().clone(),
            url: self.url.borrow().clone(),
            //FIXME: shouldn't ignore pipeline namespace field
            browsingContextId: self.browsing_context_id.index.0.get(),
            //FIXME: shouldn't ignore pipeline namespace field
            outerWindowID: self.active_pipeline.get().index.0.get(),
            consoleActor: self.console.clone(),
            /*emulationActor: self.emulation.clone(),
            inspectorActor: self.inspector.clone(),
            timelineActor: self.timeline.clone(),
            profilerActor: self.profiler.clone(),
            performanceActor: self.performance.clone(),
            styleSheetsActor: self.styleSheets.clone(),*/
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
        *self.url.borrow_mut() = url.as_str().to_owned();
        if let Some(ref t) = title {
            *self.title.borrow_mut() = t.clone();
        }

        let msg = TabNavigated {
            from: self.name(),
            type_: "tabNavigated".to_owned(),
            url: url.as_str().to_owned(),
            title: title,
            nativeConsoleAPI: true,
            state: state.to_owned(),
            isFrameSwitching: false,
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
}

#[derive(Serialize)]
struct TabNavigated {
    from: String,
    #[serde(rename = "type")]
    type_: String,
    url: String,
    title: Option<String>,
    nativeConsoleAPI: bool,
    state: String,
    isFrameSwitching: bool,
}
