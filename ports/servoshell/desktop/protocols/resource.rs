/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! This protocol handler loads files from the <resources_dir_path()>/protocol/resource directory,
//! sanitizing the path to prevent path escape attacks.
//! For security reasons, loads are only allowed if the referrer has a 'resource' or
//! 'servo' scheme.

use std::fs::File;
use std::future::Future;
use std::io::BufReader;
use std::pin::Pin;

use headers::{ContentType, HeaderMapExt};
use net::fetch::methods::{DoneChannel, FetchContext};
use net::filemanager_thread::FILE_CHUNK_SIZE;
use net::protocols::ProtocolHandler;
use net_traits::ResourceFetchTiming;
use net_traits::filemanager_thread::RelativePos;
use net_traits::request::Request;
use net_traits::response::{Response, ResponseBody};
use tokio::sync::mpsc::unbounded_channel;

#[derive(Default)]
pub struct ResourceProtocolHandler {}

impl ResourceProtocolHandler {
    pub fn response_for_path(
        request: &mut Request,
        done_chan: &mut DoneChannel,
        context: &FetchContext,
        path: &str,
    ) -> Pin<Box<dyn Future<Output = Response> + Send>> {
        if path.contains("..") || !path.starts_with("/") {
            return Box::pin(std::future::ready(Response::network_internal_error(
                "Invalid path",
            )));
        }

        let path = if let Some(path) = path.strip_prefix("/") {
            path
        } else {
            return Box::pin(std::future::ready(Response::network_internal_error(
                "Invalid path",
            )));
        };

        let file_path = crate::resources::resources_dir_path()
            .join("resource_protocol")
            .join(path);

        if !file_path.exists() || file_path.is_dir() {
            return Box::pin(std::future::ready(Response::network_internal_error(
                "Invalid path",
            )));
        }

        let response = if let Ok(file) = File::open(file_path.clone()) {
            let mut response = Response::new(
                request.current_url(),
                ResourceFetchTiming::new(request.timing_type()),
            );
            let reader = BufReader::with_capacity(FILE_CHUNK_SIZE, file);

            // Set Content-Type header.
            let mime = mime_guess::from_path(file_path).first_or_octet_stream();
            response.headers.typed_insert(ContentType::from(mime));

            // Setup channel to receive cross-thread messages about the file fetch
            // operation.
            let (mut done_sender, done_receiver) = unbounded_channel();
            *done_chan = Some((done_sender.clone(), done_receiver));

            *response.body.lock().unwrap() = ResponseBody::Receiving(vec![]);

            context.filemanager.lock().unwrap().fetch_file_in_chunks(
                &mut done_sender,
                reader,
                response.body.clone(),
                context.cancellation_listener.clone(),
                RelativePos::full_range(),
            );

            response
        } else {
            Response::network_internal_error("Opening file failed")
        };

        Box::pin(std::future::ready(response))
    }
}

impl ProtocolHandler for ResourceProtocolHandler {
    fn load(
        &self,
        request: &mut Request,
        done_chan: &mut DoneChannel,
        context: &FetchContext,
    ) -> Pin<Box<dyn Future<Output = Response> + Send>> {
        let url = request.current_url();

        // TODO: Check referrer.
        //       We unexpectedly get `NoReferrer` for all requests from the newtab page.

        Self::response_for_path(request, done_chan, context, url.path())
    }
}
