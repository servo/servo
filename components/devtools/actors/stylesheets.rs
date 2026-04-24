/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use malloc_size_of_derive::MallocSizeOf;
use serde::Serialize;
use serde_json::{Map, Value};

use crate::StreamId;
use crate::actor::{Actor, ActorError, ActorRegistry};
use crate::protocol::ClientRequest;

/// <https://searchfox.org/mozilla-central/source/devtools/server/actors/resources/stylesheets.js>
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StyleSheetData {
    /// Unique identifier for this stylesheet.
    resource_id: String,
    /// The URL of the stylesheet. Optional for inline stylesheets.
    href: Option<String>,
    /// The URL of the document that owns this stylesheet.
    node_href: String,
    /// Whether the stylesheet is disabled.
    disabled: bool,
    /// The title of the stylesheet.
    title: String,
    /// Whether this is a browser stylesheet.
    system: bool,
    /// Whether this stylesheet was created by DevTools.
    is_new: bool,
    /// Optional source map URL.
    source_map_url: Option<String>,
    /// The index of this stylesheet in the document's stylesheet list.
    style_sheet_index: i32,
    // TODO: the following fields will be implemented later once we fetch the stylesheets
    // constructed: bool,
    // file_name: Option<String>,
    // at_rules: Vec<Rule>,
    // rule_count: u32,
    // source_map_base_url: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GetStyleSheetsReply {
    from: String,
    style_sheets: Vec<StyleSheetData>,
}

#[derive(MallocSizeOf)]
pub(crate) struct StyleSheetsActor {
    name: String,
}

impl Actor for StyleSheetsActor {
    fn name(&self) -> String {
        self.name.clone()
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
            "getStyleSheets" => {
                let msg = GetStyleSheetsReply {
                    from: self.name(),
                    // TODO: Fetch actual stylesheets from the script thread.
                    style_sheets: vec![],
                };
                request.reply_final(&msg)?
            },

            _ => return Err(ActorError::UnrecognizedPacketType),
        };
        Ok(())
    }
}

impl StyleSheetsActor {
    pub fn register(registry: &ActorRegistry) -> String {
        let name = registry.new_name::<Self>();
        let actor = StyleSheetsActor { name: name.clone() };
        registry.register::<Self>(actor);
        name
    }
}
