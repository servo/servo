# Copyright 2013 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

from .utils import Devtools
from .utils import collect_network_events
from .utils import get_request_headers


class TestCorsPreflight:
    def test_option_method_showed(self, run_servoshell, web_server_urls):
        run_servoshell(url=f"{web_server_urls[0]}/cors_preflight/test.html?baseurl={web_server_urls[1]}")
        with Devtools.connect() as devtools:
            preflight, seen = collect_network_events(
                devtools,
                lambda e: e.get("method") == "OPTIONS" and web_server_urls[1] in e.get("url", ""),
            )

            # Make sure it's really a preflight, not some unrelated OPTIONS request
            headers = get_request_headers(devtools, preflight)
            assert headers.get("access-control-request-method") == "PUT"
            assert "x-custom-header" in headers.get("access-control-request-headers", "")

            # Optionally check the actual PUT request also shows up
            assert any(e.get("method") == "PUT" for e in seen)
