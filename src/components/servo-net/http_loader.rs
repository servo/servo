/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use resource_task::{Payload, Done, LoaderTask};

use core::comm::SharedChan;
use core::task;
use http_client::uv_http_request;
use http_client;

pub fn factory() -> LoaderTask {
	let f: LoaderTask = |url, progress_chan| {
		assert!(url.scheme == ~"http");

		let progress_chan = SharedChan::new(progress_chan);
		do task::spawn {
			debug!("http_loader: requesting via http: %?", url.clone());
			let mut request = uv_http_request(url.clone());
			let errored = @mut false;
			let url = url.clone();
			{
				let progress_chan = progress_chan.clone();
				do request.begin |event| {
					let url = url.clone();
					match event {
						http_client::Status(*) => { }
						http_client::Payload(data) => {
							debug!("http_loader: got data from %?", url);
							let data = data.take();
							progress_chan.send(Payload(data));
						}
						http_client::Error(*) => {
							debug!("http_loader: error loading %?", url);
							*errored = true;
							progress_chan.send(Done(Err(())));
						}
					}
				}
			}

			if !*errored {
				progress_chan.send(Done(Ok(())));
			}
		}
	};
	f
}
