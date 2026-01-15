/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::net::TcpStream;
use std::sync::Arc;

use devtools_traits::NetworkEvent;
use serde::Serialize;

use crate::actor::{ActorEncode, ActorRegistry};
use crate::actors::network_event::NetworkEventActor;
use crate::actors::watcher::WatcherActor;
use crate::resource::{ResourceArrayType, ResourceAvailable};

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Cause {
    #[serde(rename = "type")]
    pub(crate) type_: String,
    pub(crate) loading_document_uri: Option<String>,
}

pub(crate) fn handle_network_event(
    actors: Arc<ActorRegistry>,
    netevent_actor_name: String,
    mut connections: Vec<TcpStream>,
    network_event: NetworkEvent,
) {
    let actor = actors.find::<NetworkEventActor>(&netevent_actor_name);
    let watcher = actors.find::<WatcherActor>(&actor.watcher);

    match network_event {
        NetworkEvent::HttpRequest(httprequest) => {
            actor.add_request(httprequest);
            let msg = actor.encode(&actors);
            let resource = actor.resource_updates(&actors);

            for stream in &mut connections {
                watcher.resource_array(
                    msg.clone(),
                    "network-event".to_string(),
                    ResourceArrayType::Available,
                    stream,
                );

                // Also push initial resource update (request headers, cookies)
                watcher.resource_array(
                    resource.clone(),
                    "network-event".to_string(),
                    ResourceArrayType::Updated,
                    stream,
                );
            }
        },

        NetworkEvent::HttpRequestUpdate(httprequest) => {
            actor.add_request(httprequest);
            let resource = actor.resource_updates(&actors);

            for stream in &mut connections {
                watcher.resource_array(
                    resource.clone(),
                    "network-event".to_string(),
                    ResourceArrayType::Updated,
                    stream,
                );
            }
        },
        NetworkEvent::HttpResponse(httpresponse) => {
            actor.add_response(httpresponse);
            let resource = actor.resource_updates(&actors);

            for stream in &mut connections {
                watcher.resource_array(
                    resource.clone(),
                    "network-event".to_string(),
                    ResourceArrayType::Updated,
                    stream,
                );
            }
        },
        NetworkEvent::SecurityInfo(update) => {
            actor.add_security_info(update.security_info);
            let resource = actor.resource_updates(&actors);

            for stream in &mut connections {
                watcher.resource_array(
                    resource.clone(),
                    "network-event".to_string(),
                    ResourceArrayType::Updated,
                    stream,
                );
            }
        },
    }
}
