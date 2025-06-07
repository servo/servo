/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::net::TcpStream;
use std::sync::{Arc, Mutex};

use devtools_traits::NetworkEvent;
use serde::Serialize;

use crate::actor::ActorRegistry;
use crate::actors::browsing_context::BrowsingContextActor;
use crate::actors::network_event::{EventActor, NetworkEventActor, ResponseStartMsg};
use crate::protocol::JsonPacketStream;
use crate::resource::ResourceAvailable;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct NetworkEventUpdateMsg {
    from: String,
    #[serde(rename = "type")]
    type_: String,
    update_type: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ResponseStartUpdateMsg {
    from: String,
    #[serde(rename = "type")]
    type_: String,
    update_type: String,
    response: ResponseStartMsg,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct EventTimingsUpdateMsg {
    total_time: u64,
}

#[derive(Serialize)]
struct SecurityInfoUpdateMsg {
    state: String,
}
#[derive(Clone, Serialize)]
pub struct Cause {
    #[serde(rename = "type")]
    pub type_: String,
    #[serde(rename = "loadingDocumentUri")]
    pub loading_document_uri: Option<String>,
}

pub(crate) fn handle_network_event(
    actors: Arc<Mutex<ActorRegistry>>,
    netevent_actor_name: String,
    mut connections: Vec<TcpStream>,
    network_event: NetworkEvent,
    browsing_context_actor_name: String,
) {
    let mut actors = actors.lock().unwrap();
    match network_event {
        NetworkEvent::HttpRequest(httprequest) => {
            // Scope mutable borrow
            let event_actor = {
                let actor = actors.find_mut::<NetworkEventActor>(&netevent_actor_name);
                actor.add_request(httprequest);
                actor.event_actor()
            };

            let browsing_context_actor =
                actors.find::<BrowsingContextActor>(&browsing_context_actor_name);
            for stream in &mut connections {
                browsing_context_actor.resource_available(
                    event_actor.clone(),
                    "network-event".to_string(),
                    stream,
                );
            }
        },
        NetworkEvent::HttpResponse(httpresponse) => {
            // Store the response information in the actor
            let actor = actors.find_mut::<NetworkEventActor>(&netevent_actor_name);
            actor.add_response(httpresponse);

            let msg = NetworkEventUpdateMsg {
                from: netevent_actor_name.clone(),
                type_: "networkEventUpdate".to_owned(),
                update_type: "requestHeaders".to_owned(),
            };
            for stream in &mut connections {
                let _ = stream.write_merged_json_packet(&msg, &actor.request_headers());
            }

            let msg = NetworkEventUpdateMsg {
                from: netevent_actor_name.clone(),
                type_: "networkEventUpdate".to_owned(),
                update_type: "requestCookies".to_owned(),
            };
            for stream in &mut connections {
                let _ = stream.write_merged_json_packet(&msg, &actor.request_cookies());
            }

            // Send a networkEventUpdate (responseStart) to the client
            let msg = ResponseStartUpdateMsg {
                from: netevent_actor_name.clone(),
                type_: "networkEventUpdate".to_owned(),
                update_type: "responseStart".to_owned(),
                response: actor.response_start(),
            };

            for stream in &mut connections {
                let _ = stream.write_json_packet(&msg);
            }
            let msg = NetworkEventUpdateMsg {
                from: netevent_actor_name.clone(),
                type_: "networkEventUpdate".to_owned(),
                update_type: "eventTimings".to_owned(),
            };
            let extra = EventTimingsUpdateMsg {
                total_time: actor.total_time().as_millis() as u64,
            };
            for stream in &mut connections {
                let _ = stream.write_merged_json_packet(&msg, &extra);
            }

            let msg = NetworkEventUpdateMsg {
                from: netevent_actor_name.clone(),
                type_: "networkEventUpdate".to_owned(),
                update_type: "securityInfo".to_owned(),
            };
            let extra = SecurityInfoUpdateMsg {
                state: "insecure".to_owned(),
            };
            for stream in &mut connections {
                let _ = stream.write_merged_json_packet(&msg, &extra);
            }

            let msg = NetworkEventUpdateMsg {
                from: netevent_actor_name.clone(),
                type_: "networkEventUpdate".to_owned(),
                update_type: "responseContent".to_owned(),
            };
            for stream in &mut connections {
                let _ = stream.write_merged_json_packet(&msg, &actor.response_content());
            }

            let msg = NetworkEventUpdateMsg {
                from: netevent_actor_name.clone(),
                type_: "networkEventUpdate".to_owned(),
                update_type: "responseCookies".to_owned(),
            };
            for stream in &mut connections {
                let _ = stream.write_merged_json_packet(&msg, &actor.response_cookies());
            }

            let msg = NetworkEventUpdateMsg {
                from: netevent_actor_name,
                type_: "networkEventUpdate".to_owned(),
                update_type: "responseHeaders".to_owned(),
            };
            for stream in &mut connections {
                let _ = stream.write_merged_json_packet(&msg, &actor.response_headers());
            }
        },
    }
}
