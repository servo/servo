/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use file_loader;
use mime_classifier::MIMEClassifier;
use net_traits::{LoadConsumer, LoadData, NetworkError};
use resource_thread::{CancellationListener, send_error};
use std::path::Path;
use std::sync::Arc;
use url::Url;
use util::resource_files::resources_dir_path;

pub fn resolve_chrome_url(url: &Url) -> Result<Url, ()> {
    assert_eq!(url.scheme, "chrome");
    // Skip the initial //
    let non_relative_scheme_data = &url.non_relative_scheme_data().unwrap()[2..];
    let relative_path = Path::new(non_relative_scheme_data);
    // Don't allow chrome URLs access to files outside of the resources directory.
    if non_relative_scheme_data.find("..").is_some() ||
       relative_path.is_absolute() ||
       relative_path.has_root() {
        return Err(());
    }

    let mut path = resources_dir_path();
    path.push(non_relative_scheme_data);
    assert!(path.exists());
    return Ok(Url::from_file_path(&*path).unwrap());
}

pub fn factory(mut load_data: LoadData,
               start_chan: LoadConsumer,
               classifier: Arc<MIMEClassifier>,
               cancel_listener: CancellationListener) {
    let file_url = match resolve_chrome_url(&load_data.url) {
        Ok(url) => url,
        Err(_) => {
            send_error(load_data.url,
                       NetworkError::Internal("Invalid chrome URL.".to_owned()),
                       start_chan);
            return;
        }
    };
    load_data.url = file_url;
    file_loader::factory(load_data, start_chan, classifier, cancel_listener)
}
