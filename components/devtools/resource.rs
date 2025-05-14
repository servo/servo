/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::net::TcpStream;

use serde::Serialize;

use crate::protocol::JsonPacketStream;

#[derive(Serialize)]
pub(crate) struct ResourceAvailableReply<T: Serialize> {
    pub from: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub array: Vec<(String, Vec<T>)>,
}

pub(crate) trait ResourceAvailable {
    fn actor_name(&self) -> String;

    fn resource_available<T: Serialize>(
        &self,
        resource: T,
        resource_type: String,
        stream: &mut TcpStream,
    ) {
        self.resources_available(vec![resource], resource_type, stream);
    }

    fn resources_available<T: Serialize>(
        &self,
        resources: Vec<T>,
        resource_type: String,
        stream: &mut TcpStream,
    ) {
        let msg = ResourceAvailableReply::<T> {
            from: self.actor_name(),
            type_: "resources-available-array".into(),
            array: vec![(resource_type, resources)],
        };

        let _ = stream.write_json_packet(&msg);
    }
}
