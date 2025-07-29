/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};

use base::id::PipelineId;
use serde::Serialize;
use serde_json::{Map, Value};
use servo_url::ServoUrl;

use crate::StreamId;
use crate::actor::{Actor, ActorError, ActorRegistry};
use crate::protocol::ClientRequest;

/// A `sourceForm` as used in responses to thread `sources` requests.
///
/// For now, we also use this for sources in watcher `resource-available-array` messages,
/// but in Firefox those have extra fields.
///
/// <https://firefox-source-docs.mozilla.org/devtools/backend/protocol.html#loading-script-sources>
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SourceForm {
    pub actor: String,
    /// URL of the script, or URL of the page for inline scripts.
    pub url: String,
    pub is_black_boxed: bool,
}

#[derive(Serialize)]
pub(crate) struct SourcesReply {
    pub from: String,
    pub sources: Vec<SourceForm>,
}

pub(crate) struct SourceManager {
    source_actor_names: RefCell<BTreeSet<String>>,
}

#[derive(Clone, Debug)]
pub struct SourceActor {
    /// Actor name.
    pub name: String,

    /// URL of the script, or URL of the page for inline scripts.
    pub url: ServoUrl,

    /// The ‘black-boxed’ flag, which tells the debugger to avoid pausing inside this script.
    /// <https://firefox-source-docs.mozilla.org/devtools/backend/protocol.html#black-boxing-sources>
    pub is_black_boxed: bool,

    pub content: Option<String>,
    pub content_type: Option<String>,

    pub spidermonkey_id: u32,
}

#[derive(Serialize)]
struct SourceContentReply {
    from: String,
    #[serde(rename = "contentType")]
    content_type: Option<String>,
    source: String,
}

#[derive(Serialize)]
struct GetBreakableLinesReply {
    from: String,
    lines: Vec<usize>,
}

#[derive(Serialize)]
struct GetBreakpointPositionsCompressedReply {
    from: String,
    positions: BTreeMap<usize, Vec<usize>>,
}

impl SourceManager {
    pub fn new() -> Self {
        Self {
            source_actor_names: RefCell::new(BTreeSet::default()),
        }
    }

    pub fn add_source(&self, actor_name: &str) {
        self.source_actor_names
            .borrow_mut()
            .insert(actor_name.to_owned());
    }

    pub fn source_forms(&self, actors: &ActorRegistry) -> Vec<SourceForm> {
        self.source_actor_names
            .borrow()
            .iter()
            .map(|actor_name| actors.find::<SourceActor>(actor_name).source_form())
            .collect()
    }
}

impl SourceActor {
    pub fn new(
        name: String,
        url: ServoUrl,
        content: Option<String>,
        content_type: Option<String>,
        spidermonkey_id: u32,
    ) -> SourceActor {
        SourceActor {
            name,
            url,
            content,
            content_type,
            is_black_boxed: false,
            spidermonkey_id,
        }
    }

    pub fn new_registered(
        actors: &mut ActorRegistry,
        pipeline_id: PipelineId,
        url: ServoUrl,
        content: Option<String>,
        content_type: Option<String>,
        spidermonkey_id: u32,
    ) -> &SourceActor {
        let source_actor_name = actors.new_name("source");

        let source_actor = SourceActor::new(
            source_actor_name.clone(),
            url,
            content,
            content_type,
            spidermonkey_id,
        );
        actors.register(Box::new(source_actor));
        actors.register_source_actor(pipeline_id, &source_actor_name);

        actors.find(&source_actor_name)
    }

    pub fn source_form(&self) -> SourceForm {
        SourceForm {
            actor: self.name.clone(),
            url: self.url.to_string(),
            is_black_boxed: self.is_black_boxed,
        }
    }
}

impl Actor for SourceActor {
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
            // Client has requested contents of the source.
            "source" => {
                let reply = SourceContentReply {
                    from: self.name(),
                    content_type: self.content_type.clone(),
                    // TODO: do we want to wait instead of giving up immediately, in cases where the content could
                    // become available later (e.g. after a fetch)?
                    source: self
                        .content
                        .as_deref()
                        .unwrap_or("<!-- not available; please reload! -->")
                        .to_owned(),
                };
                request.reply_final(&reply)?
            },
            // Client wants to know which lines can have breakpoints.
            // Sent when opening a source in the Sources panel, and controls whether the line numbers can be clicked.
            "getBreakableLines" => {
                // Tell the client that every line is breakable.
                // TODO: determine which lines are actually breakable.
                let line_count = self
                    .content
                    .as_ref()
                    .map_or(0, |content| content.lines().count());
                let reply = GetBreakableLinesReply {
                    from: self.name(),
                    // Line numbers are one-based.
                    // <https://firefox-source-docs.mozilla.org/devtools/backend/protocol.html#source-locations>
                    lines: (1..=line_count).collect(),
                };
                request.reply_final(&reply)?
            },
            // Client wants to know which columns in the line can have breakpoints.
            // Sent when the user tries to set a breakpoint by clicking a line number in a source.
            "getBreakpointPositionsCompressed" => {
                // Tell the client that every column is breakable.
                // TODO: determine which columns are actually breakable.
                let mut positions = BTreeMap::default();
                if let Some(content) = self.content.as_ref() {
                    for (line_number, line) in content.lines().enumerate() {
                        // Column numbers are in UTF-16 code units, not Unicode scalar values or grapheme clusters.
                        let column_count = line.encode_utf16().count();
                        // Line number are one-based. Column numbers are zero-based.
                        // FIXME: the docs say column numbers are one-based, but this appears to be incorrect.
                        // <https://firefox-source-docs.mozilla.org/devtools/backend/protocol.html#source-locations>
                        positions.insert(line_number + 1, (0..column_count).collect());
                    }
                }
                let reply = GetBreakpointPositionsCompressedReply {
                    from: self.name(),
                    positions,
                };
                request.reply_final(&reply)?
            },
            _ => return Err(ActorError::UnrecognizedPacketType),
        };
        Ok(())
    }
}
