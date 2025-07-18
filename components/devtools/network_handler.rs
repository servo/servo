/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::net::TcpStream;
use std::sync::{Arc, Mutex};

use devtools_traits::NetworkEvent;
use serde::Serialize;

use crate::actor::ActorRegistry;
use crate::actors::network_event::NetworkEventActor;
use crate::actors::watcher::WatcherActor;
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
) {
    let mut actors = actors.lock().unwrap();
    let actor = actors.find_mut::<NetworkEventActor>(&netevent_actor_name);
    let watcher_name = actor.watcher_name.clone();
    match network_event {
        NetworkEvent::HttpRequest(httprequest) => {
            actor.add_request(httprequest);

            let event_actor = actor.event_actor();
            let resource_updates = actor.resource_updates();
            let watcher_actor = actors.find::<WatcherActor>(&watcher_name);

            for stream in &mut connections {
                watcher_actor.resource_array(
                    event_actor.clone(),
                    "network-event".to_string(),
                    ResourceArrayType::Available,
                    stream,
                );

                // Also push initial resource update (request headers, cookies)
                watcher_actor.resource_array(
                    resource_updates.clone(),
                    "network-event".to_string(),
                    ResourceArrayType::Updated,
                    stream,
                );
            }
        },

        NetworkEvent::HttpRequestUpdate(httprequest) => {
            actor.add_request(httprequest);
            let resource = actor.resource_updates();
            let watcher_actor = actors.find::<WatcherActor>(&watcher_name);

            for stream in &mut connections {
                watcher_actor.resource_array(
                    resource.clone(),
                    "network-event".to_string(),
                    ResourceArrayType::Updated,
                    stream,
                );
            }
        },
        NetworkEvent::HttpResponse(httpresponse) => {
            // Store the response information in the actor
            actor.add_response(httpresponse);
            let resource = actor.resource_updates();
            let watcher_actor = actors.find::<WatcherActor>(&watcher_name);

            for stream in &mut connections {
                watcher_actor.resource_array(
                    resource.clone(),
                    "network-event".to_string(),
                    ResourceArrayType::Updated,
                    stream,
                );
            }
        },
    }
}
