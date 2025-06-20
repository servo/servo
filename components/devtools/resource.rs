/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::net::TcpStream;

use serde::Serialize;

use crate::protocol::JsonPacketStream;

pub enum ResourceArrayType {
    Available,
    Updated,
}

#[derive(Serialize)]
pub(crate) struct ResourceAvailableReply<T: Serialize> {
    pub from: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub array: Vec<(String, Vec<T>)>,
}

pub(crate) trait ResourceAvailable {
    fn actor_name(&self) -> String;

    fn resource_array<T: Serialize>(
        &self,
        resource: T,
        resource_type: String,
        array_type: ResourceArrayType,
        stream: &mut TcpStream,
    ) {
        self.resources_array(vec![resource], resource_type, array_type, stream);
    }

    fn resources_array<T: Serialize>(
        &self,
        resources: Vec<T>,
        resource_type: String,
        array_type: ResourceArrayType,
        stream: &mut TcpStream,
    ) {
        let msg = ResourceAvailableReply::<T> {
            from: self.actor_name(),
            type_: match array_type {
                ResourceArrayType::Available => "resources-available-array".to_string(),
                ResourceArrayType::Updated => "resources-updated-array".to_string(),
            },
            array: vec![(resource_type, resources)],
        };

        let _ = stream.write_json_packet(&msg);
    }
}
