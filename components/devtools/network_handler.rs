/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::net::TcpStream;
use std::sync::{Arc, Mutex};

use devtools_traits::NetworkEvent;
use serde::Serialize;

use crate::actor::ActorRegistry;
use crate::actors::browsing_context::BrowsingContextActor;
use crate::actors::network_event::NetworkEventActor;
use crate::resource::ResourceAvailable;

#[derive(Clone, Serialize)]
struct ResourcesUpdatedArray {
    updates: Vec<UpdateEntry>,
}

#[derive(Clone, Serialize)]
struct UpdateEntry {
    #[serde(rename = "updateType")]
    update_type: String,
    data: serde_json::Value,
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
            // Scope mutable borrow
            let resource = {
                let actor = actors.find_mut::<NetworkEventActor>(&netevent_actor_name);
                // Store the response information in the actor
                actor.add_response(httpresponse);
                ResourcesUpdatedArray {
                    updates: vec![
                        UpdateEntry {
                            update_type: "requestHeaders".to_owned(),
                            data: serde_json::to_value(actor.request_headers()).unwrap(),
                        },
                        UpdateEntry {
                            update_type: "requestCookies".to_owned(),
                            data: serde_json::to_value(actor.request_cookies()).unwrap(),
                        },
                        UpdateEntry {
                            update_type: "responseStart".to_owned(),
                            data: serde_json::to_value(actor.response_start()).unwrap(),
                        },
                        UpdateEntry {
                            update_type: "eventTimings".to_owned(),
                            data: serde_json::to_value(EventTimingsUpdateMsg {
                                total_time: actor.total_time().as_millis() as u64,
                            })
                            .unwrap(),
                        },
                        UpdateEntry {
                            update_type: "securityInfo".to_owned(),
                            data: serde_json::to_value(SecurityInfoUpdateMsg {
                                state: "insecure".to_owned(),
                            })
                            .unwrap(),
                        },
                        UpdateEntry {
                            update_type: "responseContent".to_owned(),
                            data: serde_json::to_value(actor.response_content()).unwrap(),
                        },
                        UpdateEntry {
                            update_type: "responseCookies".to_owned(),
                            data: serde_json::to_value(actor.response_cookies()).unwrap(),
                        },
                        UpdateEntry {
                            update_type: "responseHeaders".to_owned(),
                            data: serde_json::to_value(actor.response_headers()).unwrap(),
                        },
                    ],
                }
            };

            let browsing_context_actor =
                actors.find::<BrowsingContextActor>(&browsing_context_actor_name);
            for stream in &mut connections {
                browsing_context_actor.resource_available(
                    resource.clone(),
                    "resources-updated".to_string(),
                    stream,
                );
            }
        },
    }
}
