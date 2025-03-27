use core::str;
use std::{
    collections::BTreeMap,
    io::{self, BufRead, BufReader, Read, Write},
    net::{TcpListener, TcpStream},
};

use clap::Parser;
use jane_eyre::eyre;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_jsonlines::json_lines;
use tracing::{debug, info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(clap::Parser, Debug)]
struct Args {
    json_path: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct WrappedMessage {
    message: Message,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(untagged)]
enum Message {
    Server {
        from: String,
        r#type: Option<String>,
        #[serde(flatten)]
        rest: BTreeMap<String, Value>,
    },
    Client {
        to: String,
        r#type: Option<String>,
        #[serde(flatten)]
        rest: BTreeMap<String, Value>,
    },
}

fn main() -> eyre::Result<()> {
    jane_eyre::install()?;
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    let args = Args::parse();

    // Collect messages by parsing and unwrapping records from `devtools_parser.py --json`.
    let messages = json_lines::<WrappedMessage, _>(args.json_path)?
        .map(|result| result.map(|record| record.message))
        .collect::<io::Result<Vec<_>>>()?;

    let Some((initial_message, subsequent_messages)) = messages.split_first() else {
        warn!("No initial message! Exiting");
        return Ok(());
    };

    // Organise client requests into queues by actor.
    #[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
    struct Actor(String);
    let mut requests: BTreeMap<Actor, Vec<&Message>> = BTreeMap::default();
    for message in subsequent_messages.iter() {
        if let client_message @ Message::Client { to, .. } = message {
            requests
                .entry(Actor(to.clone()))
                .or_default()
                .push(client_message);
        }
    }

    // Match up server responses with client requests, organised into queues by actor and type.
    #[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
    struct Type(String);
    let mut transactions: BTreeMap<(Actor, Type), Vec<(&Message, &Message)>> = BTreeMap::default();
    let mut spontaneous_messages: BTreeMap<Actor, Vec<&Message>> = BTreeMap::default();
    let mut target_available_form_messages = vec![];
    let mut frame_update_messages = vec![];
    let mut resources_available_array_messages = vec![];
    for message in subsequent_messages.iter() {
        if let server_message @ Message::Server { from, r#type, .. } = message {
            // Server messages with types are probably spontaneous messages (often in request/reply/notify pattern),
            // e.g. watcher resources-available-array, watcher target-available-form, thread newSource.
            // Each of these messages need custom logic defining when they should be replayed, but this logic may not
            // be perfect, so it’s always a *model* of the real behaviour.
            // <https://firefox-source-docs.mozilla.org/devtools/backend/protocol.html#common-patterns-of-actor-communication>
            match r#type.as_deref() {
                Some("target-available-form") => {
                    // watcher target-available-form model: send all on watcher watchTargets
                    target_available_form_messages.push(message);
                },
                Some("frameUpdate") => {
                    // watcher frameUpdate model: send all on watcher watchTargets, after all watcher target-available-form
                    frame_update_messages.push(message);
                },
                Some("resources-available-array") => {
                    // watcher resources-available-array model: send all on watcher watchResources
                    resources_available_array_messages.push(message);
                },
                Some(r#type) => {
                    // TODO: figure out how to realistically replay other spontaneous messages at the right time
                    //       (maybe the client will be completely deterministic, but if not, we need some way to anchor them)
                    spontaneous_messages
                        .entry(Actor(from.clone()))
                        .or_default()
                        .push(server_message);
                    warn!(%r#type, ?server_message, "Spontaneous");
                },
                None => {
                    let client_message = requests.entry(Actor(from.clone())).or_default().remove(0);
                    let Message::Client { to, r#type, .. } = client_message else {
                        unreachable!("Guaranteed by code populating it")
                    };
                    let Some(r#type) = r#type else {
                        panic!("Message from client has no type! {client_message:?}")
                    };
                    assert_eq!(to, from);
                    transactions
                        .entry((Actor(from.clone()), Type(r#type.clone())))
                        .or_default()
                        .push((client_message, server_message));
                },
            }
        }
    }

    let listen_addr = "0.0.0.0:6080";
    info!(%listen_addr, "Listening");
    let (mut stream, remote_addr) = TcpListener::bind(listen_addr)?.accept()?;
    info!(%remote_addr, "Accepted connection");
    let mut reader = BufReader::new(stream.try_clone()?);
    stream.write_json_packet(initial_message)?;
    loop {
        let message = reader.read_packet()?;
        debug!(?message);
        let Message::Client { to, r#type, .. } = &message else {
            panic!("Not a client message: {message:?}")
        };
        let Some(r#type) = r#type else {
            panic!("Message from client has no type! {message:?}")
        };
        let queue = transactions
            .get_mut(&(Actor(to.clone()), Type(r#type.clone())))
            .expect("No queue for actor and type");
        let (expected_message, reply_message) = queue.remove(0);

        // The key idea here is that for each actor and type, the client will send us the same requests.
        assert_eq!(&message, expected_message);
        stream.write_json_packet(reply_message)?;

        match &**r#type {
            "watchTargets" => {
                for spontaneous_message in target_available_form_messages.drain(..) {
                    stream.write_json_packet(spontaneous_message)?;
                }
                for spontaneous_message in frame_update_messages.drain(..) {
                    stream.write_json_packet(spontaneous_message)?;
                }
            },
            "watchResources" => {
                for spontaneous_message in resources_available_array_messages.drain(..) {
                    stream.write_json_packet(spontaneous_message)?;
                }
            },
            _ => {},
        }
    }
}

// <https://firefox-source-docs.mozilla.org/devtools/backend/protocol.html#id1>
trait DevtoolsWrite {
    /// <https://firefox-source-docs.mozilla.org/devtools/backend/protocol.html#json-packets>
    fn write_json_packet(&mut self, message: &Message) -> eyre::Result<()>;
}
trait DevtoolsRead {
    fn read_packet(&mut self) -> eyre::Result<Message>;
}

impl DevtoolsWrite for TcpStream {
    fn write_json_packet(&mut self, message: &Message) -> eyre::Result<()> {
        let result = serde_json::to_string(message)?;
        let result = format!("{}:{}", result.len(), result);
        self.write_all(result.as_bytes())?;
        Ok(())
    }
}
impl DevtoolsRead for BufReader<TcpStream> {
    fn read_packet(&mut self) -> eyre::Result<Message> {
        let mut prefix = vec![];
        self.read_until(b':', &mut prefix)?;
        let Some(prefix) = prefix.strip_suffix(b":") else {
            panic!("Unexpected EOF")
        };
        let prefix = str::from_utf8(&prefix)?;

        // TODO: implement bulk packets
        let len = prefix.parse::<usize>()?;
        let mut result = vec![0u8; len];
        self.read_exact(&mut result)?;

        Ok(serde_json::from_slice(&result)?)
    }
}
