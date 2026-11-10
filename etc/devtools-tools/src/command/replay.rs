use std::{
    collections::BTreeMap,
    fs::File,
    io::{BufReader, Read},
    net::TcpListener,
};

use jane_eyre::eyre;
use serde_json::Value;
use serde_jsonlines::BufReadExt;
use tracing::{debug, info, warn};

use crate::protocol::{
    DevtoolsRead, DevtoolsWrite, EmptyResponse, Message, ResourcesAvailableArray, WrappedMessage,
};

#[derive(clap::Args, Debug)]
pub struct Replay {
    json_path: String,
}

pub fn main(args: Replay) -> eyre::Result<()> {
    // Allow records to be commented out by prefixing the line with `//`.
    let mut file = String::default();
    File::open(args.json_path)?.read_to_string(&mut file)?;
    let file = file
        .split_terminator('\n')
        .filter(|x| !x.starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");
    let file = BufReader::new(file.as_bytes());

    // Collect messages by parsing and unwrapping records from `devtools_parser.py --json`.
    let mut messages = vec![];
    for line in file.json_lines::<WrappedMessage>() {
        messages.push(line?.message);
    }

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
    #[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
    struct ResourceType(String);
    let mut transactions: BTreeMap<(Actor, Type), Vec<(&Message, &Message)>> = BTreeMap::default();
    let mut spontaneous_messages: BTreeMap<Actor, Vec<&Message>> = BTreeMap::default();
    let mut target_available_form_messages = vec![];
    let mut frame_update_messages = vec![];
    let mut resources_available: BTreeMap<ResourceType, BTreeMap<Actor, Vec<Value>>> =
        BTreeMap::default();
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
                    // watcher resources-available-array model: on watcher watchResources, send resources with the
                    // given resourceTypes, from their original actors
                    let message: ResourcesAvailableArray =
                        serde_json::from_str(&serde_json::to_string(message)?)?;
                    for (resource_type, resources) in message.array {
                        resources_available
                            .entry(ResourceType(resource_type))
                            .or_default()
                            .entry(Actor(from.clone()))
                            .or_default()
                            .extend(resources);
                    }
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
        let Message::Client { to, r#type, rest } = &message else {
            panic!("Not a client message: {message:?}")
        };
        let Some(r#type) = r#type else {
            panic!("Message from client has no type! {message:?}")
        };
        let queue = transactions
            .entry((Actor(to.clone()), Type(r#type.clone())))
            .or_default();

        let Some((expected_message, reply_message)) = (!queue.is_empty()).then(|| queue.remove(0))
        else {
            warn!(?message, "No reply found for message");
            continue;
        };

        match &**r#type {
            "updateConfiguration" => {
                // thread-configuration updateConfiguration is sometimes sent in groups of non-deterministic order,
                // such as one for shouldPauseOnDebuggerStatement, one for observeWasm and pauseWorkersUntilAttach, and
                // one for ignoreCaughtExceptions and pauseOnExceptions. It looks like the response is always empty, so
                // let’s just send an empty response without checking that the requests came in the correct order.
                // TODO: this could make thread-configuration queues go out of sync, if there are other request types
                stream.write_json_packet(&EmptyResponse { from: to.clone() })?;
                continue;
            },
            _ => {},
        }

        // The key idea here is that for each actor and type, the client will send us the same requests.
        assert_eq!(&message, expected_message);
        stream.write_json_packet(reply_message)?;

        match &**r#type {
            "watchTargets" => {
                for spontaneous_message in target_available_form_messages.drain(..) {
                    debug!(?spontaneous_message);
                    stream.write_json_packet(spontaneous_message)?;
                }
                for spontaneous_message in frame_update_messages.drain(..) {
                    debug!(?spontaneous_message);
                    stream.write_json_packet(spontaneous_message)?;
                }
            },
            "watchResources" => {
                let resource_types = rest.get("resourceTypes");
                let resource_types = resource_types
                    .iter()
                    .flat_map(|x| x.as_array())
                    .flat_map(|x| x.iter())
                    .flat_map(|x| x.as_str());
                for resource_type in resource_types {
                    let resources = resources_available
                        .entry(ResourceType(resource_type.to_owned()))
                        .or_default();
                    for (actor, resources) in resources.iter() {
                        stream.write_json_packet(&ResourcesAvailableArray {
                            from: actor.0.clone(),
                            r#type: "resources-available-array".to_owned(),
                            array: vec![(resource_type.to_owned(), resources.clone())],
                        })?;
                    }
                }
            },
            _ => {},
        }
    }
}
