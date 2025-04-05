/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::fs::File;
use std::future::{Future, ready};
use std::io::{BufReader, Seek, SeekFrom};
use std::pin::Pin;

use headers::{ContentType, HeaderMapExt, Range};
use http::Method;
use net_traits::request::Request;
use net_traits::response::{Response, ResponseBody};
use net_traits::{NetworkError, ResourceFetchTiming};
use tokio::sync::mpsc::unbounded_channel;

use crate::fetch::methods::{DoneChannel, FetchContext};
use crate::filemanager_thread::FILE_CHUNK_SIZE;
use crate::local_directory_listing;
use crate::protocols::{
    ProtocolHandler, get_range_request_bounds, partial_content, range_not_satisfiable_error,
};

#[derive(Default)]
pub struct FileProtocolHander {}

impl ProtocolHandler for FileProtocolHander {
    fn load(
        &self,
        request: &mut Request,
        done_chan: &mut DoneChannel,
        context: &FetchContext,
    ) -> Pin<Box<dyn Future<Output = Response> + Send>> {
        let url = request.current_url();

        if request.method != Method::GET {
            return Box::pin(ready(Response::network_error(NetworkError::Internal(
                "Unexpected method for file".into(),
            ))));
        }
        let response = if let Ok(file_path) = url.to_file_path() {
            if file_path.is_dir() {
                return Box::pin(ready(local_directory_listing::fetch(
                    request, url, file_path,
                )));
            }

            if let Ok(file) = File::open(file_path.clone()) {
                // Get range bounds (if any) and try to seek to the requested offset.
                // If seeking fails, bail out with a NetworkError.
                let file_size = match file.metadata() {
                    Ok(metadata) => Some(metadata.len()),
                    Err(_) => None,
                };

                let mut response =
                    Response::new(url, ResourceFetchTiming::new(request.timing_type()));

                let range_header = request.headers.typed_get::<Range>();
                let is_range_request = range_header.is_some();
                let Ok(range) = get_range_request_bounds(range_header, file_size.unwrap_or(0))
                    .get_final(file_size)
                else {
                    range_not_satisfiable_error(&mut response);
                    return Box::pin(ready(response));
                };
                let mut reader = BufReader::with_capacity(FILE_CHUNK_SIZE, file);
                if reader.seek(SeekFrom::Start(range.start as u64)).is_err() {
                    return Box::pin(ready(Response::network_error(NetworkError::Internal(
                        "Unexpected method for file".into(),
                    ))));
                }

                // Set response status to 206 if Range header is present.
                // At this point we should have already validated the header.
                if is_range_request {
                    partial_content(&mut response);
                }

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
                    range,
                );

                response
            } else {
                Response::network_error(NetworkError::Internal("Opening file failed".into()))
            }
        } else {
            Response::network_error(NetworkError::Internal(
                "Constructing file path failed".into(),
            ))
        };

        Box::pin(ready(response))
    }
}
