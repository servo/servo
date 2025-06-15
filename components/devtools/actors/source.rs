/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::{Ref, RefCell};
use std::collections::BTreeSet;
use std::net::TcpStream;

use serde::Serialize;
use serde_json::{Map, Value};
use servo_url::ServoUrl;

use crate::StreamId;
use crate::actor::{Actor, ActorMessageStatus, ActorRegistry};
use crate::protocol::JsonPacketStream;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SourceData {
    pub actor: String,
    /// URL of the script, or URL of the page for inline scripts.
    pub url: String,
    pub is_black_boxed: bool,
    pub source_content: String,
}

#[derive(Serialize)]
pub(crate) struct SourcesReply {
    pub from: String,
    pub sources: Vec<SourceData>,
}

pub(crate) struct SourceManager {
    pub source_urls: RefCell<BTreeSet<SourceData>>,
}

#[derive(Clone, Debug)]
pub struct SourceActor {
    pub name: String,
    pub content: String,
    pub content_type: String,
}

#[derive(Serialize)]
struct SourceContentReply {
    from: String,
    #[serde(rename = "contentType")]
    content_type: String,
    source: String,
}

impl SourceManager {
    pub fn new() -> Self {
        Self {
            source_urls: RefCell::new(BTreeSet::default()),
        }
    }

    pub fn add_source(&self, url: ServoUrl, source_content: String, actor_name: String) {
        self.source_urls.borrow_mut().insert(SourceData {
            actor: actor_name,
            url: url.to_string(),
            is_black_boxed: false,
            source_content,
        });
    }

    pub fn sources(&self) -> Ref<BTreeSet<SourceData>> {
        self.source_urls.borrow()
    }
}

impl SourceActor {
    pub fn new(name: String, content: String, content_type: String) -> SourceActor {
        SourceActor {
            name,
            content,
            content_type,
        }
    }

    pub fn new_source(actors: &mut ActorRegistry, content: String, content_type: String) -> String {
        let source_actor_name = actors.new_name("source");

        let source_actor = SourceActor::new(source_actor_name.clone(), content, content_type);
        actors.register(Box::new(source_actor));

        source_actor_name
    }
}

impl Actor for SourceActor {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn handle_message(
        &self,
        _registry: &ActorRegistry,
        msg_type: &str,
        _msg: &Map<String, Value>,
        stream: &mut TcpStream,
        _id: StreamId,
    ) -> Result<ActorMessageStatus, ()> {
        Ok(match msg_type {
            // Client has requested contents of the source.
            "source" => {
                let reply = SourceContentReply {
                    from: self.name(),
                    content_type: self.content_type.clone(),
                    source: self.content.clone(),
                };
                let _ = stream.write_json_packet(&reply);
                ActorMessageStatus::Processed
            },
            _ => ActorMessageStatus::Ignored,
        })
    }
}
