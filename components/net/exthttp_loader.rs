/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use data_loader;
use mime_classifier::MIMEClassifier;
use net_traits::{LoadConsumer, LoadData};
use resource_thread::CancellationListener;
use std::borrow::ToOwned;
use std::sync::Arc;
use url::Url;
use util::thread::spawn_named;
use webbrowser;

pub fn factory(load_data: LoadData,
               senders: LoadConsumer,
               classifier: Arc<MIMEClassifier>,
               cancel_listener: CancellationListener) {
    assert!(load_data.url.scheme() == "exthttp" || load_data.url.scheme() == "exthttps");
    spawn_named("exthttp_loader".to_owned(), move || {
        let mut http_url = load_data.url.clone();
        let valid_http_url = match load_data.url.scheme() {
            "exthttp" => http_url.set_scheme("http"),
            "exthttps" => http_url.set_scheme("https"),
            _ => unreachable!()
        }.is_ok();
        let url_redirect = if !valid_http_url {
            Url::parse("data:text/plain,Invalid url").unwrap()
        } else if webbrowser::open(http_url.as_str()).is_ok() {
            Url::parse("data:text/plain,External browser succesfully opened").unwrap()
        } else {
            Url::parse("data:text/plain,Failed to open external browser").unwrap()
        };
        let load_data_redirect = LoadData::new(load_data.context, url_redirect, None, None, None);
        data_loader::factory(load_data_redirect, senders, classifier, cancel_listener);
    });
}

