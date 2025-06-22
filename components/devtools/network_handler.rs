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
use crate::resource::{ResourceArrayType, ResourceAvailable};

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
            let (event_actor, resource_updates) = {
                let actor = actors.find_mut::<NetworkEventActor>(&netevent_actor_name);
                actor.add_request(httprequest);
                (actor.event_actor(), actor.resource_updates())
            };

            let browsing_context_actor =
                actors.find::<BrowsingContextActor>(&browsing_context_actor_name);
            for stream in &mut connections {
                // Notify that a new network event has started
                browsing_context_actor.resource_array(
                    event_actor.clone(),
                    "network-event".to_string(),
                    ResourceArrayType::Available,
                    stream,
                );

                // Also push initial resource update (request headers, cookies)
                browsing_context_actor.resource_array(
                    resource_updates.clone(),
                    "network-event".to_string(),
                    ResourceArrayType::Updated,
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
                actor.resource_updates()
            };

            let browsing_context_actor =
                actors.find::<BrowsingContextActor>(&browsing_context_actor_name);
            for stream in &mut connections {
                browsing_context_actor.resource_array(
                    resource.clone(),
                    "network-event".to_string(),
                    ResourceArrayType::Updated,
                    stream,
                );
            }
        },
    }
}
