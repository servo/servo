# Copyright 2013 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

import json

from .utils import Devtools
from .utils import collect_network_events
from .utils import collect_network_response
from .utils import get_request_headers


def evaluate_js(devtools, code):
    console_actor = devtools.targets[0]["consoleActor"]
    response = devtools.client.send_receive({"to": console_actor, "type": "evaluateJSAsync", "text": code})
    assert "error" not in response, response


def fetch_and_collect_response(devtools, url):
    return collect_network_response(
        devtools=devtools,
        url_predicate=lambda event_url: event_url == url,
        trigger=lambda: evaluate_js(devtools, f"fetch({json.dumps(url)});"),
    )


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
            _, response = fetch_and_collect_response(devtools, f"{web_server_urls[0]}/network/data.json")

            assert response["status"] == "200 OK"
            assert response["mimeType"] == "application/json"
            # data.json is 32 bytes and is served uncompressed with a Content-Length header
            assert response["transferredSize"] == 32
            assert response["contentSize"] == 32

    def test_get_response_content(self, run_servoshell, web_server_urls):
        run_servoshell(url=f"{web_server_urls[0]}/network/test.html")
        with Devtools.connect() as devtools:
            event, _ = fetch_and_collect_response(devtools, f"{web_server_urls[0]}/network/data.json")

            reply = devtools.client.send_receive({"to": event["actor"], "type": "getResponseContent"})

            assert reply["from"] == event["actor"]
            assert reply["contentDiscarded"] is False
            content = reply["content"]
            assert content["mimeType"] == "application/json"
            assert content["size"] == 32
            assert content["isContentEncoded"] is False
            # Text bodies are sent as a long string, whose start is included in the reply
            assert content["text"]["type"] == "longString"
            assert content["text"]["length"] == 32
            assert json.loads(content["text"]["initial"]) == {"message": "hello from servo"}

    def test_get_response_headers(self, run_servoshell, web_server_urls):
        run_servoshell(url=f"{web_server_urls[0]}/network/test.html")
        with Devtools.connect() as devtools:
            event, _ = fetch_and_collect_response(devtools, f"{web_server_urls[0]}/network/data.json")

            reply = devtools.client.send_receive({"to": event["actor"], "type": "getResponseHeaders"})

            assert reply["from"] == event["actor"]
            headers = {header["name"].lower(): header["value"] for header in reply["headers"]}
            assert headers["content-type"] == "application/json"
            assert headers["content-length"] == "32"
            assert reply["headerSize"] == len(reply["rawHeaders"])
            raw_headers = {
                name.lower(): value.strip()
                for name, value in (line.split(":", 1) for line in reply["rawHeaders"].splitlines())
            }
            assert raw_headers == headers

    def test_get_event_timings(self, run_servoshell, web_server_urls):
        run_servoshell(url=f"{web_server_urls[0]}/network/test.html")
        with Devtools.connect() as devtools:
            event, _ = fetch_and_collect_response(devtools, f"{web_server_urls[0]}/network/data.json")

            reply = devtools.client.send_receive({"to": event["actor"], "type": "getEventTimings"})

            assert reply["from"] == event["actor"]
            # The values depend on the machine, so check the shape of the reply and that the total adds up
            timings = reply["timings"]
            assert set(timings) == {"blocked", "dns", "connect", "send", "wait", "receive"}
            assert set(reply["offsets"]) == set(timings)
            assert all(isinstance(value, int) and value >= 0 for value in timings.values())
            assert all(isinstance(value, int) and value >= 0 for value in reply["offsets"].values())
            assert reply["totalTime"] == sum(timings[phase] for phase in timings if phase != "blocked")

    def test_get_security_info(self, run_servoshell, web_server_urls):
        run_servoshell(url=f"{web_server_urls[0]}/network/test.html")
        with Devtools.connect() as devtools:
            event, response = fetch_and_collect_response(devtools, f"{web_server_urls[0]}/network/data.json")

            reply = devtools.client.send_receive({"to": event["actor"], "type": "getSecurityInfo"})

            assert reply["from"] == event["actor"]
            # The test server uses plain HTTP, so there is no TLS information to report
            security_info = reply["securityInfo"]
            assert security_info["state"] == "insecure"
            assert response["securityInfoAvailable"] is True
            assert response["securityState"] == security_info["state"]
            assert "protocolVersion" not in security_info
            assert "cipherSuite" not in security_info

    def test_not_found_response(self, run_servoshell, web_server_urls):
        run_servoshell(url=f"{web_server_urls[0]}/network/test.html")
        with Devtools.connect() as devtools:
            event, response = fetch_and_collect_response(devtools, f"{web_server_urls[0]}/network/missing.json")

            assert response["status"] == "404 Not Found"
            assert response["statusText"] == "File not found"
            assert response["mimeType"] == "text/html;charset=utf-8"
            assert response["responseHeadersAvailable"] is True
            assert response["responseContentAvailable"] is True

            # The server's error page is still available as the response content
            reply = devtools.client.send_receive({"to": event["actor"], "type": "getResponseContent"})
            assert reply["contentDiscarded"] is False
            content = reply["content"]
            assert content["mimeType"] == response["mimeType"]
            assert content["size"] == response["contentSize"] == response["transferredSize"]
            assert content["size"] > 0
            assert "File not found" in content["text"]["initial"]
