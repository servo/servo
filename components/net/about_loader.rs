/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use file_loader;
use hyper::header::ContentType;
use hyper::http::RawStatus;
use hyper::mime::{Mime, SubLevel, TopLevel};
use mime_classifier::MIMEClassifier;
use net_traits::ProgressMsg::{Done, Payload};
use net_traits::{LoadConsumer, LoadData, Metadata};
use resource_task::{CancellationListener, send_error, start_sending_sniffed_opt};
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
        // See https://fetch.spec.whatwg.org/#concept-basic-fetch
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
        // See https://fetch.spec.whatwg.org/#concept-basic-fetch
        "unicorn" => {
            let metadata = Metadata {
                final_url: load_data.url,
                content_type: Some(ContentType(Mime(TopLevel::Image, SubLevel::Ext("svg+xml".into()), vec![]))),
                charset: Some("utf-8".to_owned()),
                headers: None,
                status: Some(RawStatus(200, "OK".into())),
            };
            if let Ok(chan) = start_sending_sniffed_opt(start_chan, metadata, classifier, &[]) {
                let _ = chan.send(Payload("<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 2361 1572\" fill=\"hotpink\"><path d=\"m1648 1570c-42-12 6-93 55-94 46 0 50-10 46-123-5-122-7-126-119-202-81-55-145-128-175-201-12-30-26-59-31-63s-58-4-119 0c-70 6-131 5-170-2-59-10-84-21-220-93-68-36-72-37-145-31-41 4-100 11-131 17-55 10-57 12-62 47-16 95 3 152 49 152 14 0 29 8 34 19 14 25 13 101-0 101-12 0-60-46-60-58 0-4-17-18-38-31l-38-23-2-115c-1-64 2-124 8-133s24-21 42-25c59-12 128-37 128-46 0-5-4-9-10-9-14 0-28-30-45-95-12-47-13-69-3-124 11-62 10-71-10-109-12-23-22-50-22-60s-7-27-15-37c-18-23-19-18-5 24 8 24 7 50-5 110-10 48-14 96-10 120 8 47-9 72-48 72-40 0-66-26-77-81-6-28-30-88-53-133-23-45-45-88-48-96-4-8-22-20-41-26-26-9-34-17-34-36 0-22 4-24 37-21l37 3-9-33c-12-43-6-47 31-20l30 22 26-20c14-11 39-38 55-61 39-53 63-62 139-49 46 8 64 8 75-2 8-7 15-8 15-4-0 15-14 30-34 37-37 14-6 19 44 7 49-12 53-11 90 15 28 19 48 26 69 23 37-6 29 10-16 28-19 8-32 19-28 24 4 6 15 5 30-2 18-8 35-7 71 5 27 9 58 16 71 16 32 0 29 17-7 35-16 9-30 17-30 20 0 2 22 2 49-2 44-5 52-3 96 31 27 20 54 34 62 32 25-10 14 4-16 19-16 8-39 15-50 15-29 0-26 16 20 87 45 68 96 101 189 123 149 35 239 59 268 71 27 12 36 11 67-4 21-10 41-29 47-45 23-59 39-78 80-101 60-32 141-27 175 12 23 28 25 34 43 178 15 118 36 182 72 224 28 32 35 35 90 35 75 0 125-21 167-68l33-37-17 40c-16 41-65 98-100 117-11 6-42 17-70 24l-50 12 62 1c48 0 72-5 116-28 50-25 55-26 45-8-17 33-98 115-136 139-29 18-51 22-113 22-71 1-80-2-115-30-21-17-86-28-99-128-7-56 0-176 0-176s18-102-6-175c-19-57-81-86-123-20-19 30-43 60-54 67-18 12-18 13 6 59 34 67 38 144 14 260l-20 95 24 35c13 20 40 51 59 70 40 38 41 50 29 252-6 92-9 107-25 111-10 3-19 12-19 20s-7 18-17 20c-32 10-87 15-105 10zm-1228-1255c0-18-2-19-16-8-12 10-13 15-3 21 18 11 18 11 18-13zm743 1151c-12-5-23-14-23-20 0-17 57-69 76-69 21 0 130-65 167-99 47-43 36-101-38-198-30-39-73-148-63-158 2-2 30-5 63-7l60-3 32 60c41 77 38 69 63 145 40 115 41 112-31 166-34 27-79 62-98 79-20 17-43 34-53 38-10 3-22 17-27 30-5 14-13 27-17 29-19 12-90 16-111 7zm-913-440c0-23 28-113 44-145 6-11 32-51 57-90 26-39 50-81 53-95 5-21 22-30 103-59 53-19 102-36 108-38 6-2 18 11 27 30l16 34-92 28c-105 32-126 47-161 122-16 34-35 58-50 63-32 13-40 42-22 85l15 36-37 25c-45 30-62 31-62 4zm-48-843c-41-18-25-52 19-39 21 6 23 10 14 28-9 17-15 19-33 11zm-74-25c-28-6-31-32-4-32 13 0 26 4 29 8 8 13-8 28-25 24zm-78-37c0-9 6-12 15-9 19 7 19 24 0 24-8 0-15-7-15-15zm-50-15c0-5 7-7 15-4 19 7 19 14 0 14-8 0-15-4-15-10z\"/></svg>".as_bytes().into()));
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
