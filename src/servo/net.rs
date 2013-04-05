/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

export uri, input_stream, channel, io_service, file_channel;

import uri::uri;

iface input_stream {
    fn close();
    fn read() -> ~[u8];
}

iface channel {
    fn uri() -> uri;
    fn open() -> input_stream;
}

iface io_service {
    fn new_uri(spec: ~str) -> uri;
    fn new_channel(uri: uri) -> channel;
}

class file_channel: channel {
    let bogus : int;

    new() {
        self.bogus = 0;
    }

    fn uri() -> uri { fail }
    fn open() -> input_stream { fail }
}
