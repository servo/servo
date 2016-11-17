/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use file_loader;
use hyper::header::ContentType;
use hyper::mime::{Mime, SubLevel, TopLevel};
use hyper_serde::Serde;
use mime_classifier::MimeClassifier;
use net_traits::{LoadConsumer, LoadData, Metadata, NetworkError};
use net_traits::ProgressMsg::Done;
use net_traits::response::HttpsState;
use resource_thread::{CancellationListener, send_error, start_sending_sniffed_opt};
use servo_url::ServoUrl;
use std::io;
use std::sync::Arc;
use url::Url;
use util::resource_files::resources_dir_path;

fn url_from_non_relative_scheme(load_data: &mut LoadData, filename: &str) -> io::Result<()> {
    let mut path = try!(resources_dir_path());
    path.push(filename);
    assert!(path.exists());
    load_data.url = ServoUrl::from_url(Url::from_file_path(&*path).unwrap());
    Ok(())
}

pub fn factory(mut load_data: LoadData,
               start_chan: LoadConsumer,
               classifier: Arc<MimeClassifier>,
               cancel_listener: CancellationListener) {
    let url = load_data.url.clone();
    let res = match url.path() {
        "blank" => {
            let metadata = Metadata {
                final_url: load_data.url,
                content_type:
                    Some(Serde(ContentType(Mime(TopLevel::Text, SubLevel::Html, vec![])))),
                charset: Some("utf-8".to_owned()),
                headers: None,
                status: Some((200, b"OK".to_vec())),
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
    if res.is_ok() {
        file_loader::factory(load_data, start_chan, classifier, cancel_listener)
    } else {
        send_error(load_data.url, NetworkError::Internal("Could not access resource folder".to_owned()), start_chan);
    }
}
