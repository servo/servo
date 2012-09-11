export factory;

use comm::Chan;
use task::spawn;
use resource_task::{ProgressMsg, Payload, Done};
use std::net::url::Url;
use io::{file_reader, ReaderUtil};

const READ_SIZE: uint = 1024;

fn factory(+url: Url, progress_chan: Chan<ProgressMsg>) {
    assert url.scheme == ~"file";

    do spawn {
        match file_reader(&Path(url.path)) {
          Ok(reader) => {
            while !reader.eof() {
                let data = reader.read_bytes(READ_SIZE);
                progress_chan.send(Payload(data));
            }
            progress_chan.send(Done(Ok(())));
          }
          Err(*) => {
            progress_chan.send(Done(Err(())));
          }
        };
    }
}
