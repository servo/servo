/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use file_loader;
use hyper::header::ContentType;
use hyper::http::RawStatus;
use hyper::mime::{Mime, SubLevel, TopLevel};
use mime_classifier::MimeClassifier;
use net_traits::ProgressMsg::Done;
use net_traits::response::HttpsState;
use net_traits::{LoadConsumer, LoadData, Metadata, NetworkError};
use resource_thread::{CancellationListener, send_error, start_sending_sniffed_opt};
use std::sync::Arc;
use url::Url;
use util::resource_files::resources_dir_path;

fn url_from_non_relative_scheme(load_data: &mut LoadData, filename: &str) {
    let mut path = resources_dir_path();
    path.push(filename);
    assert!(path.exists());
    load_data.url = Url::from_file_path(&*path).unwrap();
}

pub fn factory(mut load_data: LoadData,
               start_chan: LoadConsumer,
               classifier: Arc<MimeClassifier>,
               cancel_listener: CancellationListener) {
    let url = load_data.url.clone();
    match url.path() {
        "blank" => {
            let metadata = Metadata {
                final_url: load_data.url,
                content_type: Some(ContentType(Mime(TopLevel::Text, SubLevel::Html, vec![]))),
                charset: Some("utf-8".to_owned()),
                headers: None,
                status: Some(RawStatus(200, "OK".into())),
                https_state: HttpsState::None,
                referrer: None,
            };
            if let Ok(chan) = start_sending_sniffed_opt(start_chan,
                                                        metadata,
                                                        classifier,
                                                        &[],
                                                        load_data.context) {
                let _ = chan.send(Done(Ok(())));
            }
            return
        }
        "crash" => panic!("Loading the about:crash URL."),
        "failure" | "not-found" =>
            url_from_non_relative_scheme(&mut load_data, &(url.path().to_owned() + ".html")),
        "sslfail" => url_from_non_relative_scheme(&mut load_data, "badcert.html"),
        _ => {
            send_error(load_data.url, NetworkError::Internal("Unknown about: URL.".to_owned()), start_chan);
            return
        }
    };
    file_loader::factory(load_data, start_chan, classifier, cancel_listener)
}
