/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use filemanager_thread::{FileManager, UIProvider};
use hyper::header::{Charset, ContentLength, ContentType, Headers};
use hyper::header::{ContentDisposition, DispositionParam, DispositionType};
use hyper_serde::Serde;
use ipc_channel::ipc;
use mime::{Attr, Mime};
use mime_classifier::MimeClassifier;
use net_traits::{LoadConsumer, LoadData, Metadata, NetworkError};
use net_traits::ProgressMsg::{Done, Payload};
use net_traits::blob_url_store::parse_blob_url;
use net_traits::filemanager_thread::{FileManagerThreadMsg, ReadFileProgress};
use net_traits::response::HttpsState;
use resource_thread::{send_error, start_sending_sniffed_opt};
use resource_thread::CancellationListener;
use servo_url::ServoUrl;
use std::boxed::FnBox;
use std::sync::Arc;
use util::thread::spawn_named;

// TODO: Check on GET
// https://w3c.github.io/FileAPI/#requestResponseModel

pub fn factory<UI: 'static + UIProvider>(filemanager: FileManager<UI>)
              -> Box<FnBox(LoadData, LoadConsumer, Arc<MimeClassifier>, CancellationListener) + Send> {
    box move |load_data: LoadData, start_chan, classifier, cancel_listener| {
        spawn_named(format!("blob loader for {}", load_data.url), move || {
            load_blob(load_data, start_chan, classifier, filemanager, cancel_listener);
        })
    }
}

fn load_blob<UI: 'static + UIProvider>
            (load_data: LoadData, start_chan: LoadConsumer,
             classifier: Arc<MimeClassifier>,
             filemanager: FileManager<UI>,
             cancel_listener: CancellationListener) {
    let (chan, recv) = ipc::channel().unwrap();
    if let Ok((id, origin, _fragment)) = parse_blob_url(&load_data.url.clone()) {
        let check_url_validity = true;
        let msg = FileManagerThreadMsg::ReadFile(chan, id, check_url_validity, origin);
        let _ = filemanager.handle(msg, Some(cancel_listener));

        // Receive first chunk
        match recv.recv().unwrap() {
            Ok(ReadFileProgress::Meta(blob_buf)) => {
                let content_type: Mime = blob_buf.type_string.parse().unwrap_or(mime!(Text / Plain));
                let charset = content_type.get_param(Attr::Charset);

                let mut headers = Headers::new();

                if let Some(name) = blob_buf.filename {
                    let charset = charset.and_then(|c| c.as_str().parse().ok());
                    headers.set(ContentDisposition {
                        disposition: DispositionType::Inline,
                        parameters: vec![
                            DispositionParam::Filename(charset.unwrap_or(Charset::Us_Ascii),
                                                       None, name.as_bytes().to_vec())
                        ]
                    });
                }

                headers.set(ContentType(content_type.clone()));
                headers.set(ContentLength(blob_buf.size as u64));

                let metadata = Metadata {
                    final_url: load_data.url.clone(),
                    content_type: Some(Serde(ContentType(content_type.clone()))),
                    charset: charset.map(|c| c.as_str().to_string()),
                    headers: Some(Serde(headers)),
                    // https://w3c.github.io/FileAPI/#TwoHundredOK
                    status: Some((200, b"OK".to_vec())),
                    https_state: HttpsState::None,
                    referrer: None,
                };

                if let Ok(chan) =
                    start_sending_sniffed_opt(start_chan, metadata, classifier,
                                              &blob_buf.bytes, load_data.context.clone()) {
                    let _ = chan.send(Payload(blob_buf.bytes));

                    loop {
                        match recv.recv().unwrap() {
                            Ok(ReadFileProgress::Partial(bytes)) => {
                                let _ = chan.send(Payload(bytes));
                            }
                            Ok(ReadFileProgress::EOF) => {
                                let _ = chan.send(Done(Ok(())));
                                return;
                            }
                            Ok(_) => {
                                let err = NetworkError::Internal("Invalid filemanager reply".to_string());
                                let _ = chan.send(Done(Err(err)));
                                return;
                            }
                            Err(e) => {
                                let err = NetworkError::Internal(format!("{:?}", e));
                                let _ = chan.send(Done(Err(err)));
                                return;
                            }
                        }
                    }
                }
            }
            Ok(_) => {
                let err = NetworkError::Internal("Invalid filemanager reply".to_string());
                send_error(load_data.url, err, start_chan);
            }
            Err(e) => {
                let err = NetworkError::Internal(format!("{:?}", e));
                send_error(load_data.url, err, start_chan);
            }
        }
    } else {
        let e = format!("Invalid blob URL format {:?}", load_data.url);
        let format_err = NetworkError::Internal(e);
        send_error(load_data.url.clone(), format_err, start_chan);
    }
}

/// https://fetch.spec.whatwg.org/#concept-basic-fetch (partial)
// TODO: make async.
pub fn load_blob_sync<UI: 'static + UIProvider>
            (url: ServoUrl,
             filemanager: FileManager<UI>)
             -> Result<(Headers, Vec<u8>), NetworkError> {
    let (id, origin) = match parse_blob_url(&url) {
        Ok((id, origin, _fragment)) => (id, origin),
        Err(()) => {
            let e = format!("Invalid blob URL format {:?}", url);
            return Err(NetworkError::Internal(e));
        }
    };

    let (sender, receiver) = ipc::channel().unwrap();
    let check_url_validity = true;
    let msg = FileManagerThreadMsg::ReadFile(sender, id, check_url_validity, origin);
    let _ = filemanager.handle(msg, None);

    let blob_buf = match receiver.recv().unwrap() {
        Ok(ReadFileProgress::Meta(blob_buf)) => blob_buf,
        Ok(_) => {
            return Err(NetworkError::Internal("Invalid filemanager reply".to_string()));
        }
        Err(e) => {
            return Err(NetworkError::Internal(format!("{:?}", e)));
        }
    };

    let content_type: Mime = blob_buf.type_string.parse().unwrap_or(mime!(Text / Plain));
    let charset = content_type.get_param(Attr::Charset);

    let mut headers = Headers::new();

    if let Some(name) = blob_buf.filename {
        let charset = charset.and_then(|c| c.as_str().parse().ok());
        headers.set(ContentDisposition {
            disposition: DispositionType::Inline,
            parameters: vec![
                DispositionParam::Filename(charset.unwrap_or(Charset::Us_Ascii),
                                           None, name.as_bytes().to_vec())
            ]
        });
    }

    // Basic fetch, Step 4.
    headers.set(ContentLength(blob_buf.size as u64));
    // Basic fetch, Step 5.
    headers.set(ContentType(content_type.clone()));

    let mut bytes = blob_buf.bytes;
    loop {
        match receiver.recv().unwrap() {
            Ok(ReadFileProgress::Partial(ref mut new_bytes)) => {
                bytes.append(new_bytes);
            }
            Ok(ReadFileProgress::EOF) => {
                return Ok((headers, bytes));
            }
            Ok(_) => {
                return Err(NetworkError::Internal("Invalid filemanager reply".to_string()));
            }
            Err(e) => {
                return Err(NetworkError::Internal(format!("{:?}", e)));
            }
        }
    }
}
