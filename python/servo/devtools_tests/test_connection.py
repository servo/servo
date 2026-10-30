# Copyright 2013 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

from concurrent.futures import Future

from geckordp.actors.events import Events
from geckordp.actors.root import RootActor

from .utils import Devtools


class TestConnection:
    def test_selected_tab(self, run_servoshell):
        run_servoshell(url="data:text/html,<h1>Test</h1>")

        with Devtools.connect() as devtools:
            root = RootActor(devtools.client)
            tabs = root.list_tabs()

            assert len(tabs) >= 1

            selected_tabs = [tab for tab in tabs if tab.get("selected")]
            assert len(selected_tabs) == 1

    def test_navigation(self, run_servoshell, web_server_urls):
        run_servoshell(url=f"{web_server_urls[0]}/tab/page1.html")
        with Devtools.connect() as devtools:
            for message_data, target_path in [
                ({"type": "navigateTo", "url": f"{web_server_urls[0]}/tab/page2.html"}, "/tab/page2.html"),
                ({"type": "goBack"}, "/tab/page1.html"),
                ({"type": "goForward"}, "/tab/page2.html"),
            ]:
                done = Future()

                def on_target_available(data):
                    if data.get("target", {}).get("url", "").endswith(target_path):
                        done.set_result(None)

                devtools.client.add_event_listener(
                    devtools.watcher.actor_id,
                    Events.Watcher.TARGET_AVAILABLE_FORM,
                    on_target_available,
                )
                devtools.client.send_receive({"to": devtools.tab.actor_id, **message_data})

                done.result(1)
