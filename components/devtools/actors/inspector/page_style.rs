/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::net::TcpStream;

use base::id::PipelineId;
use devtools_traits::DevtoolScriptControlMsg::GetLayout;
use devtools_traits::{ComputedNodeLayout, DevtoolScriptControlMsg};
use ipc_channel::ipc::{self, IpcSender};
use serde::Serialize;
use serde_json::{self, Map, Value};

use crate::actor::{Actor, ActorMessageStatus, ActorRegistry};
use crate::protocol::JsonPacketStream;
use crate::StreamId;

#[derive(Serialize)]
struct GetAppliedReply {
    entries: Vec<AppliedEntry>,
    rules: Vec<AppliedRule>,
    sheets: Vec<AppliedSheet>,
    from: String,
}

#[derive(Serialize)]
struct GetComputedReply {
    computed: Vec<u32>, //XXX all css props
    from: String,
}

#[derive(Serialize)]
struct AppliedEntry {
    rule: String,
    pseudoElement: Value,
    isSystem: bool,
    matchedSelectors: Vec<String>,
}

#[derive(Serialize)]
struct AppliedRule {
    actor: String,
    #[serde(rename = "type")]
    type_: String,
    href: String,
    cssText: String,
    line: u32,
    column: u32,
    parentStyleSheet: String,
}

#[derive(Serialize)]
struct AppliedSheet {
    actor: String,
    href: String,
    nodeHref: String,
    disabled: bool,
    title: String,
    system: bool,
    styleSheetIndex: isize,
    ruleCount: usize,
}

#[derive(Serialize)]
struct GetLayoutReply {
    from: String,

    display: String,
    position: String,
    #[serde(rename = "z-index")]
    zIndex: String,
    #[serde(rename = "box-sizing")]
    boxSizing: String,

    // Would be nice to use a proper struct, blocked by
    // https://github.com/serde-rs/serde/issues/43
    autoMargins: serde_json::value::Value,
    #[serde(rename = "margin-top")]
    marginTop: String,
    #[serde(rename = "margin-right")]
    marginRight: String,
    #[serde(rename = "margin-bottom")]
    marginBottom: String,
    #[serde(rename = "margin-left")]
    marginLeft: String,

    #[serde(rename = "border-top-width")]
    borderTopWidth: String,
    #[serde(rename = "border-right-width")]
    borderRightWidth: String,
    #[serde(rename = "border-bottom-width")]
    borderBottomWidth: String,
    #[serde(rename = "border-left-width")]
    borderLeftWidth: String,

    #[serde(rename = "padding-top")]
    paddingTop: String,
    #[serde(rename = "padding-right")]
    paddingRight: String,
    #[serde(rename = "padding-bottom")]
    paddingBottom: String,
    #[serde(rename = "padding-left")]
    paddingLeft: String,

    width: f32,
    height: f32,
}

#[derive(Serialize)]
pub struct PageStyleMsg {
    pub actor: String,
}

pub struct PageStyleActor {
    pub name: String,
    pub script_chan: IpcSender<DevtoolScriptControlMsg>,
    pub pipeline: PipelineId,
}

impl Actor for PageStyleActor {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn handle_message(
        &self,
        registry: &ActorRegistry,
        msg_type: &str,
        msg: &Map<String, Value>,
        stream: &mut TcpStream,
        _id: StreamId,
    ) -> Result<ActorMessageStatus, ()> {
        Ok(match msg_type {
            "getApplied" => {
                //TODO: query script for relevant applied styles to node (msg.node)
                let msg = GetAppliedReply {
                    entries: vec![],
                    rules: vec![],
                    sheets: vec![],
                    from: self.name(),
                };
                let _ = stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            },

            "getComputed" => {
                //TODO: query script for relevant computed styles on node (msg.node)
                let msg = GetComputedReply {
                    computed: vec![],
                    from: self.name(),
                };
                let _ = stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            },

            //TODO: query script for box layout properties of node (msg.node)
            "getLayout" => {
                let target = msg.get("node").unwrap().as_str().unwrap();
                let (tx, rx) = ipc::channel().unwrap();
                self.script_chan
                    .send(GetLayout(
                        self.pipeline,
                        registry.actor_to_script(target.to_owned()),
                        tx,
                    ))
                    .unwrap();
                let ComputedNodeLayout {
                    display,
                    position,
                    zIndex,
                    boxSizing,
                    autoMargins,
                    marginTop,
                    marginRight,
                    marginBottom,
                    marginLeft,
                    borderTopWidth,
                    borderRightWidth,
                    borderBottomWidth,
                    borderLeftWidth,
                    paddingTop,
                    paddingRight,
                    paddingBottom,
                    paddingLeft,
                    width,
                    height,
                } = rx.recv().unwrap().ok_or(())?;

                let auto_margins = msg
                    .get("autoMargins")
                    .and_then(Value::as_bool)
                    .unwrap_or(false);

                // http://mxr.mozilla.org/mozilla-central/source/toolkit/devtools/server/actors/styles.js
                let msg = GetLayoutReply {
                    from: self.name(),
                    display,
                    position,
                    zIndex,
                    boxSizing,
                    autoMargins: if auto_margins {
                        let mut m = Map::new();
                        let auto = serde_json::value::Value::String("auto".to_owned());
                        if autoMargins.top {
                            m.insert("top".to_owned(), auto.clone());
                        }
                        if autoMargins.right {
                            m.insert("right".to_owned(), auto.clone());
                        }
                        if autoMargins.bottom {
                            m.insert("bottom".to_owned(), auto.clone());
                        }
                        if autoMargins.left {
                            m.insert("left".to_owned(), auto);
                        }
                        serde_json::value::Value::Object(m)
                    } else {
                        serde_json::value::Value::Null
                    },
                    marginTop,
                    marginRight,
                    marginBottom,
                    marginLeft,
                    borderTopWidth,
                    borderRightWidth,
                    borderBottomWidth,
                    borderLeftWidth,
                    paddingTop,
                    paddingRight,
                    paddingBottom,
                    paddingLeft,
                    width,
                    height,
                };
                let msg = serde_json::to_string(&msg).unwrap();
                let msg = serde_json::from_str::<Value>(&msg).unwrap();
                let _ = stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            },

            _ => ActorMessageStatus::Ignored,
        })
    }
}
