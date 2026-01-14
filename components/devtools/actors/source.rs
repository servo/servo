/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::{BTreeMap, BTreeSet};

use atomic_refcell::AtomicRefCell;
use base::generic_channel::{GenericSender, channel};
use base::id::PipelineId;
use devtools_traits::DevtoolScriptControlMsg;
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
    actor: String,
    /// URL of the script, or URL of the page for inline scripts.
    url: String,
    is_black_boxed: bool,
    /// `introductionType` in SpiderMonkey `CompileOptionsWrapper`.
    introduction_type: String,
}

#[derive(Serialize)]
pub(crate) struct SourcesReply {
    pub(crate) from: String,
    pub(crate) sources: Vec<SourceForm>,
}

pub(crate) struct SourceManager {
    source_actor_names: AtomicRefCell<BTreeSet<String>>,
}

#[derive(Clone, Debug)]
pub(crate) struct SourceActor {
    /// Actor name.
    pub(crate) name: String,

    /// URL of the script, or URL of the page for inline scripts.
    url: ServoUrl,

    /// The ‘black-boxed’ flag, which tells the debugger to avoid pausing inside this script.
    /// <https://firefox-source-docs.mozilla.org/devtools/backend/protocol.html#black-boxing-sources>
    is_black_boxed: bool,

    pub(crate) content: AtomicRefCell<Option<String>>,
    content_type: Option<String>,

    // TODO: use it in #37667, then remove this allow
    spidermonkey_id: u32,
    /// `introductionType` in SpiderMonkey `CompileOptionsWrapper`.
    introduction_type: String,

    script_sender: GenericSender<DevtoolScriptControlMsg>,
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
    // Line numbers are one-based.
    // <https://firefox-source-docs.mozilla.org/devtools/backend/protocol.html#source-locations>
    lines: BTreeSet<u32>,
}

#[derive(Serialize)]
struct GetBreakpointPositionsCompressedReply {
    from: String,
    // Column numbers are in UTF-16 code units, not Unicode scalar values or grapheme clusters.
    // Line number are one-based. Column numbers are zero-based.
    // FIXME: the docs say column numbers are one-based, but this appears to be incorrect.
    // <https://firefox-source-docs.mozilla.org/devtools/backend/protocol.html#source-locations>
    positions: BTreeMap<u32, BTreeSet<u32>>,
}

impl SourceManager {
    pub fn new() -> Self {
        Self {
            source_actor_names: AtomicRefCell::new(BTreeSet::default()),
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
        introduction_type: String,
        script_sender: GenericSender<DevtoolScriptControlMsg>,
    ) -> SourceActor {
        SourceActor {
            name,
            url,
            content: AtomicRefCell::new(content),
            content_type,
            is_black_boxed: false,
            spidermonkey_id,
            introduction_type,
            script_sender,
        }
    }

    #[expect(clippy::too_many_arguments)]
    pub fn new_registered(
        actors: &ActorRegistry,
        pipeline_id: PipelineId,
        url: ServoUrl,
        content: Option<String>,
        content_type: Option<String>,
        spidermonkey_id: u32,
        introduction_type: String,
        script_sender: GenericSender<DevtoolScriptControlMsg>,
    ) -> String {
        let source_actor_name = actors.new_name::<Self>();

        let source_actor = SourceActor::new(
            source_actor_name.clone(),
            url,
            content,
            content_type,
            spidermonkey_id,
            introduction_type,
            script_sender,
        );
        actors.register(source_actor);
        actors.register_source_actor(pipeline_id, &source_actor_name);

        source_actor_name
    }

    pub fn source_form(&self) -> SourceForm {
        SourceForm {
            actor: self.name.clone(),
            url: self.url.to_string(),
            is_black_boxed: self.is_black_boxed,
            introduction_type: self.introduction_type.clone(),
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
                    // TODO: if needed, fetch the page again, in the same way as in the original request.
                    // Fetch it from cache, even if the original request was non-idempotent (e.g. POST).
                    // If we can’t fetch it from cache, we should probably give up, because with a real
                    // fetch, the server could return a different response.
                    // TODO: do we want to wait instead of giving up immediately, in cases where the content could
                    // become available later (e.g. after a fetch)?
                    source: self
                        .content
                        .borrow()
                        .as_deref()
                        .unwrap_or("<!-- not available; please reload! -->")
                        .to_owned(),
                };
                request.reply_final(&reply)?
            },
            // Client wants to know which lines can have breakpoints.
            // Sent when opening a source in the Sources panel, and controls whether the line numbers can be clicked.
            "getBreakableLines" => {
                let Some((tx, rx)) = channel() else {
                    return Err(ActorError::Internal);
                };
                self.script_sender
                    .send(DevtoolScriptControlMsg::GetPossibleBreakpoints(
                        self.spidermonkey_id,
                        tx,
                    ))
                    .map_err(|_| ActorError::Internal)?;
                let result = rx.recv().map_err(|_| ActorError::Internal)?;
                let lines = result
                    .into_iter()
                    .map(|entry| entry.line_number)
                    .collect::<BTreeSet<_>>();
                let reply = GetBreakableLinesReply {
                    from: self.name(),
                    lines,
                };
                request.reply_final(&reply)?
            },
            // Client wants to know which columns in the line can have breakpoints.
            // Sent when the user tries to set a breakpoint by clicking a line number in a source.
            "getBreakpointPositionsCompressed" => {
                let Some((tx, rx)) = channel() else {
                    return Err(ActorError::Internal);
                };
                self.script_sender
                    .send(DevtoolScriptControlMsg::GetPossibleBreakpoints(
                        self.spidermonkey_id,
                        tx,
                    ))
                    .map_err(|_| ActorError::Internal)?;
                let result = rx.recv().map_err(|_| ActorError::Internal)?;
                let mut positions: BTreeMap<u32, BTreeSet<u32>> = BTreeMap::default();
                for entry in result {
                    // Line number are one-based. Column numbers are zero-based.
                    // FIXME: the docs say column numbers are one-based, but this appears to be incorrect.
                    // <https://firefox-source-docs.mozilla.org/devtools/backend/protocol.html#source-locations>
                    positions
                        .entry(entry.line_number)
                        .or_default()
                        .insert(entry.column_number - 1);
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
