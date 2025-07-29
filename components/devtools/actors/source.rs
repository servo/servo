/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;
use std::collections::BTreeSet;

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
}

#[derive(Serialize)]
struct SourceContentReply {
    from: String,
    #[serde(rename = "contentType")]
    content_type: Option<String>,
    source: String,
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
    ) -> SourceActor {
        SourceActor {
            name,
            url,
            content,
            content_type,
            is_black_boxed: false,
        }
    }

    pub fn new_registered(
        actors: &mut ActorRegistry,
        pipeline_id: PipelineId,
        url: ServoUrl,
        content: Option<String>,
        content_type: Option<String>,
    ) -> &SourceActor {
        let source_actor_name = actors.new_name("source");

        let source_actor = SourceActor::new(source_actor_name.clone(), url, content, content_type);
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
            _ => return Err(ActorError::UnrecognizedPacketType),
        };
        Ok(())
    }
}
