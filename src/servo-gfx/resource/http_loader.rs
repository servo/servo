use core::comm::SharedChan;
use core::task::spawn;
use resource::resource_task::{Payload, Done, LoaderTask};
use http_client;
use http_client::{uv_http_request};

pub fn factory() -> LoaderTask {
	let f: LoaderTask = |url, progress_chan| {
		assert!(url.scheme == ~"http");

		let progress_chan = SharedChan(progress_chan);
		do spawn {
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
