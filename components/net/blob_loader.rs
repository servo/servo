/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use hyper::header::{DispositionType, ContentDisposition, DispositionParam};
use hyper::header::{Headers, ContentType, ContentLength, Charset};
use hyper::http::RawStatus;
use mime::{Mime, Attr};
use mime_classifier::MimeClassifier;
use net_traits::ProgressMsg::Done;
use net_traits::blob_url_store::BlobBuf;
use net_traits::response::HttpsState;
use net_traits::{LoadConsumer, LoadData, Metadata};
use resource_thread::start_sending_sniffed_opt;
use std::sync::Arc;

// TODO: Check on GET
// https://w3c.github.io/FileAPI/#requestResponseModel

pub fn load_blob(load_data: LoadData, start_chan: LoadConsumer,
                 classifier: Arc<MimeClassifier>, blob_buf: BlobBuf) {
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
    };

    if let Ok(chan) =
        start_sending_sniffed_opt(start_chan, metadata, classifier,
                                  &blob_buf.bytes, load_data.context.clone()) {
        let _ = chan.send(Done(Ok(())));
    }
}
