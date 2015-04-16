/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use net_traits::{LoadData, Metadata};
use mime_classifier::MIMEClassifier;
use resource_task::ResourceConsumer;
use file_loader;

use url::Url;
use hyper::header::ContentType;
use hyper::http::RawStatus;
use hyper::mime::{Mime, TopLevel, SubLevel};
use util::resource_files::resources_dir_path;

use std::borrow::IntoCow;
use std::fs::PathExt;
use std::sync::Arc;

pub fn factory(mut resource_consumer: ResourceConsumer, mut load_data: LoadData, classifier: Arc<MIMEClassifier>) {
    match load_data.url.non_relative_scheme_data().unwrap() {
        "blank" => {
            resource_consumer.start(Metadata {
                final_url: load_data.url,
                content_type: Some(ContentType(Mime(TopLevel::Text, SubLevel::Html, vec![]))),
                charset: Some("utf-8".to_string()),
                headers: None,
                status: Some(RawStatus(200, "OK".into_cow())),
            });
            resource_consumer.complete();
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
            resource_consumer.start(Metadata::default(load_data.url));
            resource_consumer.error("Unknown about: URL.".to_string());
            return
        }
    };
    file_loader::factory(resource_consumer, load_data, classifier)
}
