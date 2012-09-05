export factory;

import comm::Chan;
import task::spawn;
import resource_task::{ProgressMsg, Payload, Done};
import std::net::url::Url;
import io::{file_reader, ReaderUtil};

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
