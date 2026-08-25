/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Liberally derived from the [Firefox JS implementation](https://searchfox.org/mozilla-central/source/devtools/server/actors/webbrowser.js).
//! Connection point for remote devtools that wish to investigate a particular Browsing Context's contents.
//! Supports dynamic attaching and detaching which control notifications of navigation, etc.

use std::sync::Arc;

use atomic_refcell::AtomicRefCell;
use devtools_traits::DevtoolScriptControlMsg::{self, GetCssDatabase, SimulateColorScheme};
use devtools_traits::DevtoolsPageInfo;
use embedder_traits::Theme;
use malloc_size_of_derive::MallocSizeOf;
use rustc_hash::FxHashMap;
use serde::Serialize;
use serde_json::{Map, Value};
use servo_base::generic_channel::{self, GenericSender, SendError};
use servo_base::id::PipelineId;

use crate::actor::{Actor, ActorEncode, ActorError, ActorRegistry, new_actor_name};
use crate::actors::inspector::InspectorActor;
use crate::actors::inspector::accessibility::AccessibilityActor;
use crate::actors::inspector::css_properties::CssPropertiesActor;
use crate::actors::reflow::ReflowActor;
use crate::actors::stylesheets::StyleSheetsActor;
use crate::actors::tab::TabDescriptorActor;
use crate::actors::thread::ThreadActor;
use crate::actors::watcher::{SessionContext, SessionContextType, WatcherActor};
use crate::id::{DevtoolsBrowserId, DevtoolsBrowsingContextId, DevtoolsOuterWindowId};
use crate::protocol::{ClientRequest, JsonPacketStream};
use crate::resource::ResourceAvailable;
use crate::{EmptyReplyMsg, StreamId};

#[derive(Serialize)]
struct ListWorkersReply {
    from: String,
    workers: Vec<()>,
}

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
#[serde(rename_all = "lowercase")]
enum TargetType {
    Frame,
    // Other target types not implemented yet.
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BrowsingContextActorMsg {
    actor: String,
    title: String,
    url: String,
    /// This correspond to webview_id
    #[serde(rename = "browserId")]
    browser_id: u32,
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
    target_type: TargetType,
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
#[derive(MallocSizeOf)]
pub(crate) struct BrowsingContextActor {
    name: String,
    /// This corresponds to webview_id
    pub browser_id: DevtoolsBrowserId,
    // TODO: Should these ids be atomic?
    active_pipeline_id: AtomicRefCell<PipelineId>,
    active_outer_window_id: AtomicRefCell<DevtoolsOuterWindowId>,
    pub browsing_context_id: DevtoolsBrowsingContextId,
    accessibility_name: String,
    pub console_name: String,
    css_properties_name: String,
    pub inspector_name: String,
    pub page_info: AtomicRefCell<DevtoolsPageInfo>,
    reflow_name: String,
    pub style_sheets_name: String,
    pub thread_name: String,
    // Different pipelines may run on different script threads.
    // These should be kept around even when the active pipeline is updated,
    // in case the browsing context revisits a pipeline via history navigation.
    // TODO: Each entry is stored forever; ideally there should be a way to
    //       detect when `ScriptThread`s are destroyed and remove the associated
    //       entries.
    script_chans: AtomicRefCell<FxHashMap<PipelineId, GenericSender<DevtoolScriptControlMsg>>>,
    pub watcher_name: String,
}

impl ResourceAvailable for BrowsingContextActor {
    fn actor_name(&self) -> String {
        self.name.clone()
    }
}

impl Actor for BrowsingContextActor {
    fn name(&self) -> &str {
        &self.name
    }

    fn handle_message(
        &self,
        request: ClientRequest,
        _registry: &ActorRegistry,
        msg_type: &str,
        _msg: &Map<String, Value>,
        _id: StreamId,
    ) -> Result<(), ActorError> {
        match msg_type {
            "listFrames" => {
                // TODO: Find out what needs to be listed here
                let msg = EmptyReplyMsg {
                    from: self.name().into(),
                };
                request.reply_final(&msg)?
            },
            "listWorkers" => {
                request.reply_final(&ListWorkersReply {
                    from: self.name().into(),
                    // TODO: Find out what needs to be listed here
                    workers: vec![],
                })?
            },
            _ => return Err(ActorError::UnrecognizedPacketType),
        };
        Ok(())
    }
}

impl BrowsingContextActor {
    #[expect(clippy::too_many_arguments)]
    pub(crate) fn register(
        registry: &ActorRegistry,
        console_name: String,
        browser_id: DevtoolsBrowserId,
        browsing_context_id: DevtoolsBrowsingContextId,
        page_info: DevtoolsPageInfo,
        pipeline_id: PipelineId,
        outer_window_id: DevtoolsOuterWindowId,
        script_sender: GenericSender<DevtoolScriptControlMsg>,
    ) -> Arc<Self> {
        let name = new_actor_name::<BrowsingContextActor>();

        let accessibility_actor = AccessibilityActor::register(registry);

        let properties = (|| {
            let (properties_sender, properties_receiver) = generic_channel::channel()?;
            script_sender.send(GetCssDatabase(properties_sender)).ok()?;
            properties_receiver.recv().ok()
        })()
        .unwrap_or_default();

        let css_properties_actor = CssPropertiesActor::register(registry, properties);

        let inspector_actor = InspectorActor::register(registry, name.clone());

        let reflow_actor = ReflowActor::register(registry);

        let style_sheets_actor =
            StyleSheetsActor::register(registry, script_sender.clone(), name.clone());

        let _ = TabDescriptorActor::register(registry, name.clone());

        let thread_actor =
            ThreadActor::register(registry, script_sender.clone(), Some(name.clone()));

        let watcher_actor = WatcherActor::register(
            registry,
            name.clone(),
            SessionContext::new(SessionContextType::BrowserElement),
        );

        let mut script_chans = FxHashMap::default();
        script_chans.insert(pipeline_id, script_sender);

        let actor = BrowsingContextActor {
            name,
            script_chans: script_chans.into(),
            active_pipeline_id: pipeline_id.into(),
            active_outer_window_id: outer_window_id.into(),
            browser_id,
            browsing_context_id,
            accessibility_name: accessibility_actor.name().into(),
            console_name,
            css_properties_name: css_properties_actor.name().into(),
            inspector_name: inspector_actor.name().into(),
            page_info: page_info.into(),
            reflow_name: reflow_actor.name().into(),
            style_sheets_name: style_sheets_actor.name().into(),
            thread_name: thread_actor.name().into(),
            watcher_name: watcher_actor.name().into(),
        };

        registry.register::<Self>(actor)
    }

    pub(crate) fn handle_new_global(
        &self,
        pipeline: PipelineId,
        script_sender: GenericSender<DevtoolScriptControlMsg>,
    ) {
        self.script_chans
            .borrow_mut()
            .insert(pipeline, script_sender);
    }

    pub(crate) fn title_changed(&self, pipeline_id: PipelineId, title: String) {
        if pipeline_id != self.pipeline_id() {
            return;
        }
        self.page_info.borrow_mut().title = title;
    }

    pub(crate) fn frame_update(&self, request: &mut ClientRequest) {
        let _ = request.write_json_packet(&FrameUpdateReply {
            from: self.name().into(),
            type_: "frameUpdate".into(),
            frames: vec![FrameUpdateMsg {
                id: self.browsing_context_id.value(),
                is_top_level: true,
                title: self.title(),
                url: self.url(),
            }],
        });
    }

    pub fn simulate_color_scheme(&self, theme: Theme) -> Result<(), ()> {
        self.script_chan()
            .send(SimulateColorScheme(self.pipeline_id(), theme))
            .map_err(|_| ())
    }

    pub(crate) fn pipeline_id(&self) -> PipelineId {
        *self.active_pipeline_id.borrow()
    }

    pub(crate) fn title(&self) -> String {
        self.page_info.borrow().title.clone()
    }

    pub(crate) fn url(&self) -> String {
        self.page_info.borrow().url.clone().into_string()
    }

    pub(crate) fn update_pipeline(
        &self,
        pipeline_id: PipelineId,
        outer_window_id: DevtoolsOuterWindowId,
        page_info: DevtoolsPageInfo,
    ) {
        *self.active_pipeline_id.borrow_mut() = pipeline_id;
        *self.active_outer_window_id.borrow_mut() = outer_window_id;
        *self.page_info.borrow_mut() = page_info;
    }

    pub(crate) fn outer_window_id(&self) -> DevtoolsOuterWindowId {
        *self.active_outer_window_id.borrow()
    }

    /// Returns the script sender for the active pipeline.
    pub(crate) fn script_chan(&self) -> GenericSender<DevtoolScriptControlMsg> {
        self.script_chans
            .borrow()
            .get(&self.pipeline_id())
            .unwrap()
            .clone()
    }

    pub(crate) fn instruct_script_to_send_live_updates(&self, should_send_updates: bool) {
        let result = self
            .script_chan()
            .send(DevtoolScriptControlMsg::WantsLiveNotifications(
                self.pipeline_id(),
                should_send_updates,
            ));

        // Notifying the script thread may fail with a "Disconnected" error if servo
        // as a whole is being shut down.
        debug_assert!(matches!(result, Ok(_) | Err(SendError::Disconnected)));
    }
}

impl ActorEncode<BrowsingContextActorMsg> for BrowsingContextActor {
    fn encode(&self, _: &ActorRegistry) -> BrowsingContextActorMsg {
        BrowsingContextActorMsg {
            actor: self.name().into(),
            traits: BrowsingContextTraits {
                is_browsing_context: true,
                frames: true,
                log_in_page: false,
                navigation: true,
                supports_top_level_target_flag: true,
                watchpoints: true,
            },
            title: self.title(),
            url: self.url(),
            browser_id: self.browser_id.value(),
            browsing_context_id: self.browsing_context_id.value(),
            outer_window_id: self.outer_window_id().value(),
            is_top_level_target: true,
            accessibility_actor: self.accessibility_name.clone(),
            console_actor: self.console_name.clone(),
            css_properties_actor: self.css_properties_name.clone(),
            inspector_actor: self.inspector_name.clone(),
            reflow_actor: self.reflow_name.clone(),
            style_sheets_actor: self.style_sheets_name.clone(),
            thread_actor: self.thread_name.clone(),
            target_type: TargetType::Frame,
        }
    }
}
