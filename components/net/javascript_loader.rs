/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use http::headers::content_type::MediaType;
use resource_task::{Done, Payload, Metadata, LoadData, LoadResponse, LoaderTask, start_sending};
use url::NonRelativeSchemeData;

pub fn factory() -> LoaderTask {
    proc(url, start_chan) {
        // NB: we don't spawn a new task.
        load(url, start_chan)
    }
}

fn load(load_data: LoadData, start_chan: Sender<LoadResponse>) {
    let url = load_data.url;
    assert!("javascript" == url.scheme.as_slice());

    let mut metadata = Metadata::default(url.clone());

    // Split out content type and data.
    let mut scheme_data = match url.scheme_data {
        NonRelativeSchemeData(scheme_data) => scheme_data,
        _ => fail!("Expected a non-relative scheme URL.")
    };

    metadata.set_content_type(&Some(MediaType::new("application".to_string(),
                                                   "javascript".to_string(),
                                                   vec![])));

    let progress_chan = start_sending(start_chan, metadata);
    progress_chan.send(Payload(scheme_data.as_bytes().to_vec()));
    progress_chan.send(Done(Ok(())));
}
