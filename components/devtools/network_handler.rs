/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::sync::Arc;

use devtools_traits::NetworkEvent;
use serde::Serialize;

use crate::actor::{ActorEncode, ActorRegistry};
use crate::actors::network_event::NetworkEventActor;
use crate::actors::watcher::WatcherActor;
use crate::protocol::DevtoolsConnection;
use crate::resource::{ResourceArrayType, ResourceAvailable};

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Cause {
    #[serde(rename = "type")]
    pub type_: String,
    pub loading_document_uri: Option<String>,
}

pub(crate) fn handle_network_event(
    registry: Arc<ActorRegistry>,
    network_event_name: String,
    mut connections: Vec<DevtoolsConnection>,
    network_event: NetworkEvent,
) {
    let network_event_actor = registry.find::<NetworkEventActor>(&network_event_name);
    let watcher_actor = registry.find::<WatcherActor>(&network_event_actor.watcher_name);

    match network_event {
        NetworkEvent::HttpRequest(httprequest) => {
            network_event_actor.add_request(httprequest);
            let msg = network_event_actor.encode(&registry);
            let resource = network_event_actor.resource_updates(&registry);

            for stream in &mut connections {
                watcher_actor.resource_array(
                    msg.clone(),
                    "network-event".to_string(),
                    ResourceArrayType::Available,
                    stream,
                );

                // Also push initial resource update (request headers, cookies)
                watcher_actor.resource_array(
                    resource.clone(),
                    "network-event".to_string(),
                    ResourceArrayType::Updated,
                    stream,
                );
            }
        },

        NetworkEvent::HttpRequestUpdate(httprequest) => {
            network_event_actor.add_request(httprequest);
            let resource = network_event_actor.resource_updates(&registry);

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
            network_event_actor.add_response(httpresponse);
            let resource = network_event_actor.resource_updates(&registry);

            for stream in &mut connections {
                watcher_actor.resource_array(
                    resource.clone(),
                    "network-event".to_string(),
                    ResourceArrayType::Updated,
                    stream,
                );
            }
        },
        NetworkEvent::SecurityInfo(update) => {
            network_event_actor.add_security_info(update.security_info);
            let resource = network_event_actor.resource_updates(&registry);

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
