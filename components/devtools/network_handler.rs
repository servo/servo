use crate::actor::ActorRegistry;
use crate::actors::network_event::{EventActor, NetworkEventActor, ResponseStartMsg};
use crate::protocol::JsonPacketStream;
use devtools_traits::NetworkEvent;
use serde::Serialize;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct NetworkEventMsg {
    from: String,
    #[serde(rename = "type")]
    type_: String,
    event_actor: EventActor,
}

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

pub fn handle_network_event(
    actors: Arc<Mutex<ActorRegistry>>,
    console_actor_name: String,
    netevent_actor_name: String,
    mut connections: Vec<TcpStream>,
    network_event: NetworkEvent,
) {
    let mut actors = actors.lock().unwrap();
    let actor = actors.find_mut::<NetworkEventActor>(&netevent_actor_name);

    match network_event {
        NetworkEvent::HttpRequest(httprequest) => {
            // Store the request information in the actor
            actor.add_request(httprequest);

            // Send a networkEvent message to the client
            let msg = NetworkEventMsg {
                from: console_actor_name,
                type_: "networkEvent".to_owned(),
                event_actor: actor.event_actor(),
            };
            for stream in &mut connections {
                let _ = stream.write_json_packet(&msg);
            }
        },
        NetworkEvent::HttpResponse(httpresponse) => {
            // Store the response information in the actor
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
