/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! This actor represents one DOM node. It is created by the Walker actor when it is traversing the
//! document tree.

use std::collections::HashMap;

use atomic_refcell::AtomicRefCell;
use devtools_traits::{
    AttrModification, DevtoolScriptControlMsg, EventListenerInfo, MatchedRule, NodeInfo,
    ShadowRootMode,
};
use malloc_size_of_derive::MallocSizeOf;
use serde::Serialize;
use serde_json::{self, Map, Value};
use servo_base::generic_channel;

use crate::actor::{Actor, ActorEncode, ActorError, ActorRegistry, new_actor_name};
use crate::actors::inspector::walker::WalkerActor;
use crate::protocol::ClientRequest;
use crate::{EmptyReplyMsg, StreamId};

/// Text node type constant. This is defined again to avoid depending on `script`, where it is defined originally.
/// See `script::dom::bindings::codegen::Bindings::NodeBinding::NodeConstants`.
const TEXT_NODE: u16 = 3;

/// The maximum length of a text node for it to appear as an inline child in the inspector.
const MAX_INLINE_LENGTH: usize = 50;

#[derive(Serialize)]
struct GetEventListenerInfoReply {
    from: String,
    events: Vec<DevtoolsEventListenerInfo>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DevtoolsEventListenerInfo {
    r#type: String,
    handler: String,
    origin: String,
    tags: String,
    capturing: bool,
    // This will always be an empty object, we just need a value that serializes to "{}".
    hide: Value,
    native: bool,
    source_actor: String,
    enabled: bool,
    is_user_defined: bool,
    event_listener_info_id: String,
}

#[derive(Serialize)]
struct GetUniqueSelectorReply {
    from: String,
    value: String,
}

#[derive(Serialize)]
struct GetXPathReply {
    from: String,
    value: String,
}

#[derive(Clone, Serialize)]
struct AttrMsg {
    name: String,
    value: String,
}

#[derive(Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct NodeActorMsg {
    pub actor: String,

    /// The ID of the shadow host of this node, if it is
    /// a shadow root
    host: Option<String>,
    #[serde(rename = "baseURI")]
    base_uri: String,
    causes_overflow: bool,
    container_type: Option<()>,
    pub display_name: String,
    display_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    inline_text_child: Option<Box<NodeActorMsg>>,
    is_after_pseudo_element: bool,
    is_anonymous: bool,
    is_before_pseudo_element: bool,
    is_direct_shadow_host_child: Option<bool>,
    /// Whether or not this node is displayed.
    ///
    /// Setting this value to `false` will cause the devtools to render the node name in gray.
    is_displayed: bool,
    #[serde(rename = "isInHTMLDocument")]
    is_in_html_document: Option<bool>,
    is_marker_pseudo_element: bool,
    is_native_anonymous: bool,
    is_scrollable: bool,
    is_shadow_host: bool,
    is_shadow_root: bool,
    is_top_level_document: bool,
    node_name: String,
    node_type: u16,
    node_value: Option<String>,
    pub num_children: usize,
    #[serde(skip_serializing_if = "String::is_empty")]
    parent: String,
    shadow_root_mode: Option<String>,
    traits: HashMap<String, ()>,
    attrs: Vec<AttrMsg>,

    /// The `DOCTYPE` name if this is a `DocumentType` node, `None` otherwise
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,

    /// The `DOCTYPE` public identifier if this is a `DocumentType` node, `None` otherwise
    #[serde(skip_serializing_if = "Option::is_none")]
    public_id: Option<String>,

    /// The `DOCTYPE` system identifier if this is a `DocumentType` node, `None` otherwise
    #[serde(skip_serializing_if = "Option::is_none")]
    system_id: Option<String>,

    has_event_listeners: bool,
}

#[derive(MallocSizeOf)]
pub(crate) struct NodeActor {
    name: String,
    pub walker_name: String,
    pub style_rules: AtomicRefCell<HashMap<MatchedRule, String>>,
    node_info: AtomicRefCell<NodeInfo>,
}

impl Actor for NodeActor {
    fn name(&self) -> String {
        self.name.clone()
    }

    /// The node actor can handle the following messages:
    ///
    /// - `modifyAttributes`: Asks the script to change a value in the attribute of the
    ///   corresponding node
    ///
    /// - `getUniqueSelector`: Returns the display name of this node
    fn handle_message(
        &self,
        request: ClientRequest,
        registry: &ActorRegistry,
        msg_type: &str,
        msg: &Map<String, Value>,
        _id: StreamId,
    ) -> Result<(), ActorError> {
        let browsing_context_actor = registry
            .find::<WalkerActor>(&self.walker_name)
            .browsing_context_actor(registry);
        let script_chan = browsing_context_actor.script_chan();
        let pipeline_id = browsing_context_actor.pipeline_id();

        match msg_type {
            "modifyAttributes" => {
                let mods = msg
                    .get("modifications")
                    .ok_or(ActorError::MissingParameter)?
                    .as_array()
                    .ok_or(ActorError::BadParameterType)?;
                let modifications: Vec<AttrModification> = mods
                    .iter()
                    .filter_map(|json_mod| {
                        serde_json::from_str(&serde_json::to_string(json_mod).ok()?).ok()
                    })
                    .collect();

                script_chan
                    .send(DevtoolScriptControlMsg::ModifyAttribute(
                        browsing_context_actor.pipeline_id(),
                        registry.actor_to_script(self.name()),
                        modifications,
                    ))
                    .map_err(|_| ActorError::Internal)?;

                let reply = EmptyReplyMsg { from: self.name() };
                request.reply_final(&reply)?
            },
            "getEventListenerInfo" => {
                let target = msg
                    .get("to")
                    .ok_or(ActorError::MissingParameter)?
                    .as_str()
                    .ok_or(ActorError::BadParameterType)?;

                let (tx, rx) = generic_channel::channel().ok_or(ActorError::Internal)?;
                script_chan
                    .send(DevtoolScriptControlMsg::GetEventListenerInfo(
                        pipeline_id,
                        registry.actor_to_script(target.to_owned()),
                        tx,
                    ))
                    .unwrap();
                let event_listeners = rx.recv().map_err(|_| ActorError::Internal)?;

                let msg = GetEventListenerInfoReply {
                    from: self.name(),
                    events: event_listeners.into_iter().map(From::from).collect(),
                };
                request.reply_final(&msg)?
            },
            "getUniqueSelector" => {
                let (tx, rx) = generic_channel::channel().unwrap();
                script_chan
                    .send(DevtoolScriptControlMsg::GetDocumentElement(pipeline_id, tx))
                    .unwrap();
                let node_info = rx
                    .recv()
                    .map_err(|_| ActorError::Internal)?
                    .ok_or(ActorError::Internal)?;

                self.update(node_info);
                let msg = GetUniqueSelectorReply {
                    from: self.name(),
                    value: self.node_info.borrow().node_name.to_lowercase(),
                };
                request.reply_final(&msg)?
            },
            "getXPath" => {
                let target = msg
                    .get("to")
                    .ok_or(ActorError::MissingParameter)?
                    .as_str()
                    .ok_or(ActorError::BadParameterType)?;

                let (tx, rx) = generic_channel::channel().unwrap();
                script_chan
                    .send(DevtoolScriptControlMsg::GetXPath(
                        pipeline_id,
                        registry.actor_to_script(target.to_owned()),
                        tx,
                    ))
                    .unwrap();

                let xpath_selector = rx.recv().map_err(|_| ActorError::Internal)?;
                let msg = GetXPathReply {
                    from: self.name(),
                    value: xpath_selector,
                };
                request.reply_final(&msg)?
            },

            _ => return Err(ActorError::UnrecognizedPacketType),
        };
        Ok(())
    }
}

impl NodeActor {
    pub fn register_or_update(
        registry: &ActorRegistry,
        walker_name: &str,
        node_info: NodeInfo,
    ) -> String {
        let unique_id = &node_info.unique_id;

        if !registry.script_actor_registered(unique_id) {
            let name = new_actor_name::<Self>();
            registry.register_script_actor(unique_id.clone(), name.clone());

            let actor = Self {
                name: name.clone(),
                walker_name: walker_name.into(),
                style_rules: AtomicRefCell::new(HashMap::new()),
                node_info: AtomicRefCell::new(node_info),
            };

            registry.register(actor);
            name
        } else {
            let name = registry.script_to_actor(unique_id);
            let actor = registry.find::<NodeActor>(&name);
            actor.update(node_info);
            name
        }
    }

    fn update(&self, node_info: NodeInfo) {
        *self.node_info.borrow_mut() = node_info;
    }
}

impl ActorEncode<NodeActorMsg> for NodeActor {
    fn encode(&self, registry: &ActorRegistry) -> NodeActorMsg {
        let node_info = self.node_info.borrow();

        let actor = self.name();
        let host = node_info.host.as_ref().and_then(|host_id| {
            if registry.script_actor_registered(host_id) {
                Some(registry.script_to_actor(host_id))
            } else {
                None
            }
        });

        let browsing_context = registry
            .find::<WalkerActor>(&self.walker_name)
            .browsing_context_actor(registry);
        let script_chan = browsing_context.script_chan();
        let pipeline = browsing_context.pipeline_id();
        let script_id = registry.actor_to_script(actor.clone());

        // If a node only has a single text node as a child with a small enough text,
        // return it with this node as an `inlineTextChild`.
        let inline_text_child = (|| {
            // TODO: Also return if this node is a flex element.
            if node_info.num_children != 1 || node_info.node_name == "SLOT" {
                return None;
            }

            let (tx, rx) = generic_channel::channel()?;
            script_chan
                .send(DevtoolScriptControlMsg::GetChildren(
                    pipeline,
                    script_id.clone(),
                    tx,
                ))
                .unwrap();
            let mut children = rx.recv().ok()??;

            let child = children.pop()?;
            let node_name = NodeActor::register_or_update(registry, &self.walker_name, child);
            let msg = registry.encode::<NodeActor, _>(&node_name);

            // If the node child is not a text node, do not represent it inline.
            if msg.node_type != TEXT_NODE {
                return None;
            }

            // If the text node child is too big, do not represent it inline.
            if msg.node_value.clone().unwrap_or_default().len() > MAX_INLINE_LENGTH {
                return None;
            }

            Some(Box::new(msg))
        })();

        NodeActorMsg {
            actor,
            host,
            base_uri: node_info.base_uri.clone(),
            display_name: node_info.node_name.to_lowercase(),
            display_type: node_info.display.clone(),
            inline_text_child,
            is_displayed: node_info.is_displayed,
            is_in_html_document: Some(true),
            is_shadow_host: node_info.is_shadow_host,
            is_shadow_root: node_info.shadow_root_mode.is_some(),
            is_top_level_document: node_info.is_top_level_document,
            node_name: node_info.node_name.clone(),
            node_type: node_info.node_type,
            node_value: node_info.node_value.clone(),
            num_children: node_info.num_children,
            parent: registry.script_to_actor(&node_info.parent),
            shadow_root_mode: node_info
                .shadow_root_mode
                .as_ref()
                .map(ShadowRootMode::to_string),
            attrs: node_info
                .attrs
                .iter()
                .map(|attr| AttrMsg {
                    name: attr.name.clone(),
                    value: attr.value.clone(),
                })
                .collect(),
            name: node_info.doctype_name.clone(),
            public_id: node_info.doctype_public_identifier.clone(),
            system_id: node_info.doctype_system_identifier.clone(),
            has_event_listeners: node_info.has_event_listeners,
            ..Default::default()
        }
    }
}

impl From<EventListenerInfo> for DevtoolsEventListenerInfo {
    fn from(event_listener_info: EventListenerInfo) -> Self {
        Self {
            r#type: event_listener_info.event_type,
            handler: "todo".to_owned(),
            capturing: event_listener_info.capturing,
            origin: "todo".to_owned(),
            tags: "".to_owned(),
            hide: Value::Object(Default::default()),
            native: false,
            source_actor: "todo".to_owned(),
            enabled: true,
            is_user_defined: false,
            event_listener_info_id: "todo".to_owned(),
        }
    }
}
