/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use file_loader;
use mime_classifier::MIMEClassifier;
use net_traits::ProgressMsg::Done;
use net_traits::{LoadData, Metadata, LoadConsumer};
use resource_task::start_sending;

use hyper::header::ContentType;
use hyper::http::RawStatus;
use hyper::mime::{Mime, TopLevel, SubLevel};
use url::Url;
use util::resource_files::resources_dir_path;

use std::fs::PathExt;
use std::sync::Arc;

pub fn factory(mut load_data: LoadData, start_chan: LoadConsumer, classifier: Arc<MIMEClassifier>) {
    match load_data.url.non_relative_scheme_data().unwrap() {
        "blank" => {
            let chan = start_sending(start_chan, Metadata {
                final_url: load_data.url,
                content_type: Some(ContentType(Mime(TopLevel::Text, SubLevel::Html, vec![]))),
                charset: Some("utf-8".to_string()),
                headers: None,
                status: Some(RawStatus(200, "OK".into())),
            });
            chan.send(Done(Ok(()))).unwrap();
            return
        }
        "crash" => panic!("Loading the about:crash URL."),
        "failure" => {
            let mut path = resources_dir_path();
            path.push("failure.html");
            assert!(path.exists());
            load_data.url = Url::from_file_path(&*path).unwrap();
        }
        _ => {
            start_sending(start_chan, Metadata::default(load_data.url))
                .send(Done(Err("Unknown about: URL.".to_string())))
                .unwrap();
            return
        }
    };
    file_loader::factory(load_data, start_chan, classifier)
}
