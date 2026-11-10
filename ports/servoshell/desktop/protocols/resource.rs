/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! This protocol handler loads files from the <resources_dir_path()>/protocol/resource directory,
//! sanitizing the path to prevent path escape attacks.
//! For security reasons, loads are only allowed if the referrer has a 'resource' or
//! 'servo' scheme.

use std::fs::File;
use std::future::Future;
use std::io::{BufReader, Read};
use std::pin::Pin;
use std::sync::Arc;

use flate2::read::GzDecoder;
use headers::{ContentType, HeaderMapExt};
use servo::protocol_handler::{
    DoneChannel, FILE_CHUNK_SIZE, FetchContext, NetworkError, ProtocolHandler, RelativePos,
    Request, ResourceFetchTiming, Response, ResponseBody,
};
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
            return Box::pin(std::future::ready(Response::network_error(
                NetworkError::ResourceLoadError("Invalid path".to_owned()),
            )));
        }

        let path = if let Some(path) = path.strip_prefix("/") {
            path
        } else {
            return Box::pin(std::future::ready(Response::network_error(
                NetworkError::ResourceLoadError("Invalid path".to_owned()),
            )));
        };

        let file_path = crate::resources::resource_protocol_dir_path().join(path);
        // Packaged builds ship some large resources gzip-compressed (see `./mach package`).
        let (open_path, compressed) = if file_path.is_file() {
            (file_path.clone(), false)
        } else {
            (
                crate::resources::resource_protocol_dir_path().join(format!("{path}.gz")),
                true,
            )
        };

        if !open_path.is_file() {
            return Box::pin(std::future::ready(Response::network_error(
                NetworkError::ResourceLoadError("Invalid path".to_owned()),
            )));
        }

        let response = if let Ok(file) = File::open(open_path) {
            let mut response = Response::new(
                request.current_url(),
                ResourceFetchTiming::new(request.timing_type()),
            );
            let file: Box<dyn Read + Send> = if compressed {
                Box::new(GzDecoder::new(file))
            } else {
                Box::new(file)
            };
            let reader = BufReader::with_capacity(FILE_CHUNK_SIZE, file);

            // Set Content-Type header.
            let mime = mime_guess::from_path(file_path).first_or_octet_stream();
            Arc::make_mut(&mut response.headers).typed_insert(ContentType::from(mime));

            // Setup channel to receive cross-thread messages about the file fetch
            // operation.
            let (mut done_sender, done_receiver) = unbounded_channel();
            *done_chan = Some((done_sender.clone(), done_receiver));

            *response.body.lock() = ResponseBody::Receiving(vec![]);

            context.filemanager.fetch_file_in_chunks(
                &mut done_sender,
                reader,
                response.body.clone(),
                context.cancellation_listener.clone(),
                RelativePos::full_range(),
            );

            response
        } else {
            Response::network_error(NetworkError::ResourceLoadError(
                "Opening file failed".to_owned(),
            ))
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
