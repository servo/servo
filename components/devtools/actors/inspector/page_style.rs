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
#[serde(rename_all = "camelCase")]
struct AppliedEntry {
    rule: String,
    pseudo_element: Value,
    is_system: bool,
    matched_selectors: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AppliedRule {
    actor: String,
    #[serde(rename = "type")]
    type_: String,
    href: String,
    css_text: String,
    line: u32,
    column: u32,
    parent_style_sheet: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AppliedSheet {
    actor: String,
    href: String,
    node_href: String,
    disabled: bool,
    title: String,
    system: bool,
    style_sheet_index: isize,
    rule_count: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
struct GetLayoutReply {
    from: String,

    display: String,
    position: String,
    z_index: String,
    box_sizing: String,

    // Would be nice to use a proper struct, blocked by
    // https://github.com/serde-rs/serde/issues/43
    auto_margins: serde_json::value::Value,
    margin_top: String,
    margin_right: String,
    margin_bottom: String,
    margin_left: String,

    border_top_width: String,
    border_right_width: String,
    border_bottom_width: String,
    border_left_width: String,

    padding_top: String,
    padding_right: String,
    padding_bottom: String,
    padding_left: String,

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
                    z_index,
                    box_sizing,
                    auto_margins,
                    margin_top,
                    margin_right,
                    margin_bottom,
                    margin_left,
                    border_top_width,
                    border_right_width,
                    border_bottom_width,
                    border_left_width,
                    padding_top,
                    padding_right,
                    padding_bottom,
                    padding_left,
                    width,
                    height,
                } = rx.recv().unwrap().ok_or(())?;

                let msg_auto_margins = msg
                    .get("autoMargins")
                    .and_then(Value::as_bool)
                    .unwrap_or(false);

                // http://mxr.mozilla.org/mozilla-central/source/toolkit/devtools/server/actors/styles.js
                let msg = GetLayoutReply {
                    from: self.name(),
                    display,
                    position,
                    z_index,
                    box_sizing,
                    auto_margins: if msg_auto_margins {
                        let mut m = Map::new();
                        let auto = serde_json::value::Value::String("auto".to_owned());
                        if auto_margins.top {
                            m.insert("top".to_owned(), auto.clone());
                        }
                        if auto_margins.right {
                            m.insert("right".to_owned(), auto.clone());
                        }
                        if auto_margins.bottom {
                            m.insert("bottom".to_owned(), auto.clone());
                        }
                        if auto_margins.left {
                            m.insert("left".to_owned(), auto);
                        }
                        serde_json::value::Value::Object(m)
                    } else {
                        serde_json::value::Value::Null
                    },
                    margin_top,
                    margin_right,
                    margin_bottom,
                    margin_left,
                    border_top_width,
                    border_right_width,
                    border_bottom_width,
                    border_left_width,
                    padding_top,
                    padding_right,
                    padding_bottom,
                    padding_left,
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
