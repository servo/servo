use serde::Serialize;
use serde_json::{Map, Value};

use crate::StreamId;
use crate::actor::{Actor, ActorError, ActorRegistry};
use crate::protocol::ClientRequest;

pub struct LongStringActor {
    pub name: String,
    pub full_string: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SubstringReply {
    from: String,
    substring: String,
}

impl Actor for LongStringActor {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn handle_message(
        &self,
        request: ClientRequest,
        _registry: &ActorRegistry,
        msg_type: &str,
        msg: &Map<String, Value>,
        _id: StreamId,
    ) -> Result<(), ActorError> {
        match msg_type {
            "substring" => {
                let start = msg.get("start").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                let end = msg
                    .get("end")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(self.full_string.len() as u64) as usize;
                let substring: String = self
                    .full_string
                    .chars()
                    .skip(start)
                    .take(end - start)
                    .collect();
                let reply = SubstringReply {
                    from: self.name(),
                    substring,
                };
                request.reply_final(&reply)?
            },
            _ => return Err(ActorError::UnrecognizedPacketType),
        }
        Ok(())
    }
}
