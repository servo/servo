/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use resource_task::{TargetedLoadResponse, Metadata, Done, LoadData, start_sending, ResponseSenders};
use file_loader;

use std::io::fs::PathExtensions;
use url::Url;
use hyper::http::RawStatus;
use servo_util::resource_files::resources_dir_path;


pub fn factory(mut load_data: LoadData, start_chan: Sender<TargetedLoadResponse>) {
    let senders = ResponseSenders {
        immediate_consumer: start_chan.clone(),
        eventual_consumer: load_data.consumer.clone(),
    };
    match load_data.url.non_relative_scheme_data().unwrap() {
        "blank" => {
            let chan = start_sending(senders, Metadata {
                final_url: load_data.url,
                content_type: Some(("text".to_string(), "html".to_string())),
                charset: Some("utf-8".to_string()),
                headers: None,
                status: Some(RawStatus(200, "OK".into_string()))
            });
            chan.send(Done(Ok(())));
            return
        }
        "crash" => panic!("Loading the about:crash URL."),
        "failure" => {
            let mut path = resources_dir_path();
            path.push("failure.html");
            assert!(path.exists());
            load_data.url = Url::from_file_path(&path).unwrap();
        }
        _ => {
            start_sending(senders, Metadata::default(load_data.url))
                .send(Done(Err("Unknown about: URL.".to_string())));
            return
        }
    };
    file_loader::factory(load_data, start_chan)
}
