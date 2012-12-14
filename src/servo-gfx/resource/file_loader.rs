export factory;

use comm::Chan;
use task::spawn;
use resource::resource_task::{ProgressMsg, Payload, Done};
use std::net::url::Url;
use io::{file_reader, ReaderUtil};

const READ_SIZE: uint = 1024;

pub fn factory(url: Url, progress_chan: Chan<ProgressMsg>) {
    assert url.scheme == ~"file";

    do spawn |move url| {
        // FIXME: Resolve bug prevents us from moving the path out of the URL.
        match file_reader(&Path(url.path)) {
            Ok(reader) => {
                while !reader.eof() {
                    let data = reader.read_bytes(READ_SIZE);
                    progress_chan.send(Payload(move data));
                }
                progress_chan.send(Done(Ok(())));
            }
            Err(*) => {
                progress_chan.send(Done(Err(())));
            }
        };
    }
}
