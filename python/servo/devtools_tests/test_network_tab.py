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
from .utils import collect_network_response
from .utils import get_request_headers


def evaluate_js(devtools, code):
    console_actor = devtools.targets[0]["consoleActor"]
    response = devtools.client.send_receive({"to": console_actor, "type": "evaluateJSAsync", "text": code})
    assert "error" not in response, response


class TestNetworkTab:
    def test_option_method_showed(self, run_servoshell, web_server_urls):
        run_servoshell(url=f"{web_server_urls[0]}/cors_preflight/test.html")
        with Devtools.connect() as devtools:
            preflight, seen = collect_network_events(
                devtools=devtools,
                predicate=lambda event: event.get("method") == "OPTIONS" and web_server_urls[1] in event.get("url", ""),
                trigger=lambda: evaluate_js(
                    devtools,
                    f"""
                    fetch("{web_server_urls[1]}/cors_preflight/test.html", {{
                        method: "PUT",
                        headers: {{ "X-Custom-Header": "1" }},
                    }}).catch(() => {{}});
                """,
                ),
            )

            # Make sure it's really a preflight, not some unrelated OPTIONS request
            headers = get_request_headers(devtools, preflight)
            assert headers.get("access-control-request-method") == "PUT"
            assert "x-custom-header" in headers.get("access-control-request-headers", "")

            # Optionally check the actual PUT request also shows up
            assert any(event.get("method") == "PUT" for event in seen)

    def test_fetch_response_fields(self, run_servoshell, web_server_urls):
        run_servoshell(url=f"{web_server_urls[0]}/network/test.html")
        with Devtools.connect() as devtools:
            response = collect_network_response(
                devtools=devtools,
                url_predicate=lambda url: url.endswith("/network/data.json"),
                trigger=lambda: evaluate_js(devtools, f'fetch("{web_server_urls[0]}/network/data.json");'),
            )

            assert response["status"] == "200 OK"
            assert response["mimeType"] == "application/json"
            # data.json is 32 bytes and is served uncompressed with a Content-Length header
            assert response["transferredSize"] == 32
            assert response["contentSize"] == 32
