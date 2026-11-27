# Copyright 2026 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

from concurrent.futures import Future

from geckordp.actors.events import Events
from geckordp.actors.web_console import WebConsoleActor

from .utils import Devtools


def assert_iframe_target_destroyed_after(script: str):
    with Devtools.connect(expected_targets=2) as devtools:
        iframe_target = devtools.targets[1]
        destroyed = Future()

        def on_target_destroyed(data):
            if data["target"]["actor"] == iframe_target["actor"] and not destroyed.done():
                destroyed.set_result(None)

        devtools.client.add_event_listener(
            devtools.watcher.actor_id,
            Events.Watcher.TARGET_DESTROYED_FORM,
            on_target_destroyed,
        )
        console = WebConsoleActor(devtools.client, devtools.targets[0]["consoleActor"])
        console.evaluate_js_async(script)
        destroyed.result(2)


class TestTarget:
    def test_iframe_target_destroyed_when_iframe_removed(self, run_servoshell, web_server_urls):
        run_servoshell(url=f"{web_server_urls[0]}/sources/test_sources_list_with_iframe.html")
        assert_iframe_target_destroyed_after("document.querySelector('iframe').remove()")

    def test_iframe_target_destroyed_when_page_navigates(self, run_servoshell, web_server_urls):
        run_servoshell(url=f"{web_server_urls[0]}/sources/test_sources_list_with_iframe.html")
        assert_iframe_target_destroyed_after(f"location.href = '{web_server_urls[0]}/sources/iframe.html'")
