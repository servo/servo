/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use resource_task::{LoadResponse, Metadata, Done, LoadData, start_sending};
use file_loader;

use std::os;
use std::io::fs::PathExtensions;
use url::Url;
use http::status::Ok as StatusOk;


pub fn factory(mut load_data: LoadData, start_chan: Sender<LoadResponse>) {
    match load_data.url.non_relative_scheme_data().unwrap() {
        "blank" => {
            let chan = start_sending(start_chan, Metadata {
                final_url: load_data.url,
                content_type: Some(("text".to_string(), "html".to_string())),
                charset: Some("utf-8".to_string()),
                headers: None,
                status: StatusOk,
            });
            chan.send(Done(Ok(())));
            return
        }
        "crash" => fail!("Loading the about:crash URL."),
        "failure" => {
            // FIXME: Find a way to load this without relying on the `../src` directory.
            let mut path = os::self_exe_path().expect("can't get exe path");
            path.pop();
            if !path.join(Path::new("./tests/")).is_dir() {
                path.pop();
            }
            path.push_many(["tests", "html", "failure.html"]);
            assert!(path.exists());
            load_data.url = Url::from_file_path(&path).unwrap();
        }
        _ => {
            start_sending(start_chan, Metadata::default(load_data.url))
                .send(Done(Err("Unknown about: URL.".to_string())));
            return
        }
    };
    file_loader::factory(load_data, start_chan)
}
