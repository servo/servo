/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use hyper::header::{DispositionType, ContentDisposition, DispositionParam};
use hyper::header::{Headers, ContentType, ContentLength, Charset};
use hyper::http::RawStatus;
use ipc_channel::ipc::{self, IpcSender};
use mime::{Mime, Attr};
use mime_classifier::MIMEClassifier;
use net_traits::ProgressMsg::Done;
use net_traits::blob_url_store::{parse_blob_url, BlobURLStoreEntry, BlobURLStoreError, BlobURLStoreMsg};
use net_traits::response::HttpsState;
use net_traits::{LoadConsumer, LoadData, Metadata, NetworkError};
use resource_thread::{CancellationListener, send_error, start_sending_sniffed_opt};
use std::str;
use std::sync::Arc;


// TODO: Check on GET
// https://w3c.github.io/FileAPI/#requestResponseModel

pub fn factory(load_data: LoadData,
               start_chan: LoadConsumer,
               classifier: Arc<MIMEClassifier>,
               _cancel_listener: CancellationListener,
               blob_url_store: IpcSender<BlobURLStoreMsg>) {
    match parse_blob_url(&load_data.url) {
        None => {
            let format_err = NetworkError::Internal(format!("Invalid blob URL format {:?}", load_data.url));
            send_error(load_data.url.clone(), format_err, start_chan);
        }
        Some((uuid, _fragment)) => {
            let (tx, rx) = ipc::channel().unwrap();
            let _ = blob_url_store.send(BlobURLStoreMsg::Request(uuid, tx)).unwrap();

            match rx.recv().unwrap() {
                Ok(entry) => load_blob(&load_data, start_chan, classifier, entry),
                Err(e) => {
                    let err = match e {
                        BlobURLStoreError::InvalidKey =>
                            format!("Invalid blob URL key {:?}", uuid.simple().to_string()),
                        BlobURLStoreError::InvalidOrigin =>
                            format!("Invalid blob URL origin {:?}", load_data.url.origin()),
                    };
                    send_error(load_data.url.clone(), NetworkError::Internal(err), start_chan);
                }
            }
        }
    }
}

fn load_blob(load_data: &LoadData,
             start_chan: LoadConsumer,
             classifier: Arc<MIMEClassifier>,
             entry: BlobURLStoreEntry) {
    let content_type: Mime = entry.type_string.parse().unwrap_or(mime!(Text / Plain));
    let charset = content_type.get_param(Attr::Charset);

    let mut headers = Headers::new();

    if let Some(name) = entry.filename {
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
    headers.set(ContentLength(entry.size));

    let metadata = Metadata {
        final_url: load_data.url.clone(),
        content_type: Some(ContentType(content_type.clone())),
        charset: charset.map(|c| c.as_str().to_string()),
        headers: Some(headers),
        // https://w3c.github.io/FileAPI/#TwoHundredOK
        status: Some(RawStatus(200, "OK".into())),
        https_state: HttpsState::None,
    };

    if let Ok(chan) =
        start_sending_sniffed_opt(start_chan, metadata, classifier,
                                  &entry.bytes, load_data.context.clone()) {
        let _ = chan.send(Done(Ok(())));
    }
}
