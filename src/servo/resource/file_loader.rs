export factory;

import comm::chan;
import task::spawn;
import resource_task::{ProgressMsg, Payload, Done};
import std::net::url::url;
import io::file_reader;
import result::{result, ok, err};

const READ_SIZE: uint = 1024;

fn factory(+url: url, progress_chan: chan<ProgressMsg>) {
    assert url.scheme == ~"file";

    do spawn {
        match file_reader(url.path) {
          ok(reader) => {
            while !reader.eof() {
                let data = reader.read_bytes(READ_SIZE);
                progress_chan.send(Payload(data));
            }
            progress_chan.send(Done(ok(())));
          }
          err(*) => {
            progress_chan.send(Done(err(())));
          }
        };
    }
}
