/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Loads resources using a mapping from well-known shortcuts to resource: urls.
//! Recognized shorcuts:
//! - servo:newtab

use std::future::Future;
use std::pin::Pin;

use net::fetch::methods::{DoneChannel, FetchContext};
use net::protocols::ProtocolHandler;
use net_traits::request::Request;
use net_traits::response::Response;

use crate::desktop::protocols::resource::ResourceProtocolHander;

#[derive(Default)]
pub struct ServoProtocolHander {}

impl ProtocolHandler for ServoProtocolHander {
    fn load(
        &self,
        request: &mut Request,
        done_chan: &mut DoneChannel,
        context: &FetchContext,
    ) -> Pin<Box<dyn Future<Output = Response> + Send>> {
        let url = request.current_url();

        match url.path() {
            "newtab" => ResourceProtocolHander::response_for_path(
                request,
                done_chan,
                context,
                "/newtab.html",
            ),
            _ => Box::pin(std::future::ready(Response::network_internal_error(
                "Invalid shortcut",
            ))),
        }
    }
}
