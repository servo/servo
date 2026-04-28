/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use devtools_traits::DevtoolScriptControlMsg;
use log::warn;
use malloc_size_of_derive::MallocSizeOf;
use serde::Serialize;
use serde_json::{Map, Value};
use servo_base::generic_channel;
use servo_base::generic_channel::GenericSender;

use crate::StreamId;
use crate::actor::{Actor, ActorError, ActorRegistry};
use crate::actors::browsing_context::BrowsingContextActor;
use crate::actors::long_string::{LongStringActor, LongStringObj};
use crate::protocol::ClientRequest;

/// <https://searchfox.org/mozilla-central/source/devtools/server/actors/resources/stylesheets.js>
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StyleSheetData {
    /// Unique identifier for this stylesheet.
    pub(crate) resource_id: String,
    /// Current browsing context id.
    pub(crate) browsing_context_id: u32,
    /// The URL of the stylesheet. Optional for inline stylesheets.
    pub(crate) href: Option<String>,
    /// The URL of the document that owns this stylesheet.
    pub(crate) node_href: String,
    /// Whether the stylesheet is disabled.
    pub(crate) disabled: bool,
    /// The title of the stylesheet.
    pub(crate) title: Option<String>,
    /// Whether this is a browser stylesheet.
    pub(crate) system: bool,
    /// Whether this stylesheet was created by DevTools.
    pub(crate) is_new: bool,
    /// Optional file name used for local files.
    pub(crate) file_name: Option<String>,
    /// Optional source map URL.
    #[serde(rename = "sourceMapURL")]
    pub(crate) source_map_url: Option<String>,
    #[serde(rename = "sourceMapBaseURL")]
    pub(crate) source_map_base_url: Option<String>,
    /// The index of this stylesheet in the document's stylesheet list.
    pub(crate) style_sheet_index: i32,
    /// whether the stylesheet was constructed using Web APIs.
    pub(crate) constructed: bool,
    /// Total count of individual CSS rules within that specific stylesheet.
    pub(crate) rule_count: u32,
    /// List of media query metadata (ex: @media, @keyframes).
    pub(crate) at_rules: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GetStyleSheetsReply {
    from: String,
    style_sheets: Vec<StyleSheetData>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GetTextReply {
    from: String,
    text: LongStringObj,
}

#[derive(MallocSizeOf)]
pub(crate) struct StyleSheetsActor {
    name: String,
    script_sender: GenericSender<DevtoolScriptControlMsg>,
    browsing_context_name: String,
}

impl Actor for StyleSheetsActor {
    fn name(&self) -> String {
        self.name.clone()
    }
    fn handle_message(
        &self,
        request: ClientRequest,
        registry: &ActorRegistry,
        msg_type: &str,
        msg: &Map<String, Value>,
        _id: StreamId,
    ) -> Result<(), ActorError> {
        let browsing_context_actor =
            registry.find::<BrowsingContextActor>(&self.browsing_context_name);
        match msg_type {
            "getStyleSheets" => {
                let style_sheets = self.get_stylesheets_data(&browsing_context_actor);
                let msg = GetStyleSheetsReply {
                    from: self.name(),
                    style_sheets,
                };
                request.reply_final(&msg)?
            },
            "getText" => {
                let resource_id = msg.get("resourceId").and_then(|v| v.as_str()).unwrap_or("");
                let index = resource_id
                    .split('-')
                    .next_back()
                    .unwrap_or("0")
                    .parse::<i32>()
                    .unwrap_or(0);
                let (tx, rx) = generic_channel::channel().unwrap();
                let _ = self
                    .script_sender
                    .send(DevtoolScriptControlMsg::GetStyleSheetText(
                        browsing_context_actor.pipeline_id(),
                        index,
                        tx,
                    ));
                let css_text = rx
                    .recv()
                    .map_err(|_| ActorError::Internal)?
                    .unwrap_or_else(|| {
                        warn!("Stylesheet fetched without text content");
                        "Error fetching CSS text".to_string()
                    });
                let long_string_name = LongStringActor::register(registry, css_text);
                let long_string_actor = registry.find::<LongStringActor>(&long_string_name);
                let msg = GetTextReply {
                    from: self.name(),
                    text: long_string_actor.long_string_obj(),
                };
                request.reply_final(&msg)?
            },
            _ => return Err(ActorError::UnrecognizedPacketType),
        };
        Ok(())
    }
}

impl StyleSheetsActor {
    pub fn register(
        registry: &ActorRegistry,
        script_sender: GenericSender<DevtoolScriptControlMsg>,
        browsing_context_name: String,
    ) -> String {
        let name = registry.new_name::<Self>();
        let actor = StyleSheetsActor {
            name: name.clone(),
            script_sender,
            browsing_context_name,
        };
        registry.register::<Self>(actor);
        name
    }

    pub(crate) fn get_stylesheets_data(
        &self,
        browsing_context_actor: &BrowsingContextActor,
    ) -> Vec<StyleSheetData> {
        let (tx, rx) = generic_channel::channel().unwrap();
        let _ = self
            .script_sender
            .send(DevtoolScriptControlMsg::GetStyleSheets(
                browsing_context_actor.pipeline_id(),
                tx,
            ));
        let style_sheets = rx.recv().unwrap_or_else(|_| vec![]);
        style_sheets
            .into_iter()
            .map(|info| StyleSheetData {
                resource_id: format!(
                    "{}-{}",
                    browsing_context_actor.browsing_context_id.value(),
                    info.style_sheet_index
                ),
                browsing_context_id: browsing_context_actor.browsing_context_id.value(),
                href: info.href.clone(),
                node_href: browsing_context_actor.url.borrow().clone(),
                disabled: info.disabled,
                title: (!info.title.is_empty()).then_some(info.title),
                system: info.system,
                is_new: false,
                file_name: None,
                source_map_url: Some("".to_string()),
                source_map_base_url: Some(
                    info.href
                        .unwrap_or_else(|| browsing_context_actor.url.borrow().clone()),
                ),
                style_sheet_index: info.style_sheet_index,
                constructed: false,
                rule_count: info.rule_count,
                at_rules: vec![], // TODO: Populate with media query metadata for the Style Editor sidebar.
            })
            .collect()
    }
}
