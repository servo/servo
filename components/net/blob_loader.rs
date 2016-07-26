/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use hyper::header::{DispositionType, ContentDisposition, DispositionParam};
use hyper::header::{Headers, ContentType, ContentLength, Charset};
use hyper::http::RawStatus;
use ipc_channel::ipc::{self, IpcSender};
use mime::{Mime, Attr};
use mime_classifier::MimeClassifier;
use net_traits::ProgressMsg::{Payload, Done};
use net_traits::blob_url_store::parse_blob_url;
use net_traits::filemanager_thread::{FileManagerThreadMsg, SelectedFileId, ReadFileProgress};
use net_traits::response::HttpsState;
use net_traits::{LoadConsumer, LoadData, Metadata, NetworkError};
use resource_thread::CancellationListener;
use resource_thread::{start_sending_sniffed_opt, send_error};
use std::boxed::FnBox;
use std::sync::Arc;
use util::thread::spawn_named;

// TODO: Check on GET
// https://w3c.github.io/FileAPI/#requestResponseModel

pub fn factory(filemanager_chan: IpcSender<FileManagerThreadMsg>)
               -> Box<FnBox(LoadData,
                            LoadConsumer,
                            Arc<MimeClassifier>,
                            CancellationListener) + Send> {
    box move |load_data: LoadData, start_chan, classifier, cancel_listener| {
        spawn_named(format!("blob loader for {}", load_data.url), move || {
            load_blob(load_data, start_chan, classifier, filemanager_chan, cancel_listener);
        })
    }
}

fn load_blob(load_data: LoadData, start_chan: LoadConsumer,
             classifier: Arc<MimeClassifier>,
             filemanager_chan: IpcSender<FileManagerThreadMsg>,
             // XXX(izgzhen): we should utilize cancel_listener, filed in #12589
             _cancel_listener: CancellationListener) {
    let (chan, recv) = ipc::channel().unwrap();
    if let Ok((id, origin, _fragment)) = parse_blob_url(&load_data.url.clone()) {
        let id = SelectedFileId(id.simple().to_string());
        let check_url_validity = true;
        let msg = FileManagerThreadMsg::ReadFile(chan, id, check_url_validity, origin);
        let _ = filemanager_chan.send(msg);

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
                    content_type: Some(ContentType(content_type.clone())),
                    charset: charset.map(|c| c.as_str().to_string()),
                    headers: Some(headers),
                    // https://w3c.github.io/FileAPI/#TwoHundredOK
                    status: Some(RawStatus(200, "OK".into())),
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
