# Copyright 2026 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

import time

from .utils import Devtools


class TestTabActor:
    def test_get_favicon_none(self, run_servoshell, web_server_urls):
        run_servoshell(url=f"{web_server_urls[0]}/tab/page1.html")
        with Devtools.connect() as devtools:
            response = devtools.tab.get_favicon()
            favicon = response.get("favicon") if isinstance(response, dict) else response
            assert favicon is None

    def test_get_favicon(self, run_servoshell, web_server_urls):
        run_servoshell(url=f"{web_server_urls[0]}/tab/favicon.html")
        with Devtools.connect() as devtools:
            favicon = None
            start_time = time.time()
            while time.time() - start_time < 5:
                response = devtools.tab.get_favicon()
                favicon = response.get("favicon") if isinstance(response, dict) else response
                if favicon is not None:
                    break
                time.sleep(0.1)

            assert favicon is not None
            assert isinstance(favicon, list)
            assert len(favicon) > 0
