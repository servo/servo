/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use file_loader;
use hyper::header::ContentType;
use hyper::http::RawStatus;
use hyper::mime::{Mime, SubLevel, TopLevel};
use mime_classifier::MIMEClassifier;
use net_traits::ProgressMsg::Done;
use net_traits::{LoadConsumer, LoadData, Metadata};
use resource_task::{CancellationListener, send_error, start_sending_sniffed_opt};
use std::fs::PathExt;
use std::sync::Arc;
use url::Url;
use util::resource_files::resources_dir_path;

pub fn factory(mut load_data: LoadData,
               start_chan: LoadConsumer,
               classifier: Arc<MIMEClassifier>,
               cancel_listener: CancellationListener) {
    let url = load_data.url.clone();
    let non_relative_scheme_data = url.non_relative_scheme_data().unwrap();
    match non_relative_scheme_data {
        "blank" => {
            let metadata = Metadata {
                final_url: load_data.url,
                content_type: Some(ContentType(Mime(TopLevel::Text, SubLevel::Html, vec![]))),
                charset: Some("utf-8".to_owned()),
                headers: None,
                status: Some(RawStatus(200, "OK".into())),
            };
            if let Ok(chan) = start_sending_sniffed_opt(start_chan, metadata, classifier, &[]) {
                let _ = chan.send(Done(Ok(())));
            }
            return
        }
        "crash" => panic!("Loading the about:crash URL."),
        "failure" | "not-found" => {
            let mut path = resources_dir_path();
            let file_name = non_relative_scheme_data.to_owned() + ".html";
            path.push(&file_name);
            assert!(path.exists());
            load_data.url = Url::from_file_path(&*path).unwrap();
        }
        _ => {
            send_error(load_data.url, "Unknown about: URL.".to_owned(), start_chan);
            return
        }
    };
    file_loader::factory(load_data, start_chan, classifier, cancel_listener)
}
