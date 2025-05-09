# Copyright 2013 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

from concurrent.futures import Future
import logging
from geckordp.actors.root import RootActor
from geckordp.actors.descriptors.tab import TabActor
from geckordp.actors.watcher import WatcherActor
from geckordp.actors.resources import Resources
from geckordp.actors.events import Events
from geckordp.rdp_client import RDPClient
import http.server
import os.path
import socketserver
import subprocess
import time
from threading import Thread
from typing import Optional
import unittest


# Set this to true to log requests in the internal web servers.
LOG_REQUESTS = False


class DevtoolsTests(unittest.IsolatedAsyncioTestCase):
    # /path/to/servo/python/servo
    script_path = None

    def __init__(self, methodName="runTest"):
        super().__init__(methodName)
        self.servoshell = None
        self.base_url = None
        self.web_server = None
        self.web_server_thread = None

    # Classic script vs module script:
    # - <https://html.spec.whatwg.org/multipage/#classic-script>
    # - <https://html.spec.whatwg.org/multipage/#module-script>
    # Worker scripts can be classic or module:
    # - <https://html.spec.whatwg.org/multipage/#fetch-a-classic-worker-script>
    # - <https://html.spec.whatwg.org/multipage/#fetch-a-module-worker-script-tree>
    # Non-worker(?) script sources can be inline, external, or blob.
    # Worker script sources can be external or blob.
    def test_sources_list(self):
        self.start_web_server(test_dir=os.path.join(DevtoolsTests.script_path, "devtools_tests/sources"))
        self.run_servoshell()
        self.assert_sources_list(2, set([
            tuple([f"{self.base_url}/classic.js", f"{self.base_url}/test.html", "https://servo.org/js/load-table.js"]),
            tuple([f"{self.base_url}/worker.js"]),
        ]))

    def test_sources_list_with_data_no_scripts(self):
        self.run_servoshell(url="data:text/html,")
        self.assert_sources_list(1, set([tuple()]))

    def test_sources_list_with_data_empty_inline_classic_script(self):
        self.run_servoshell(url="data:text/html,<script></script>")
        self.assert_sources_list(1, set([tuple()]))

    def test_sources_list_with_data_inline_classic_script(self):
        self.run_servoshell(url="data:text/html,<script>;</script>")
        self.assert_sources_list(1, set([tuple(["data:text/html,<script>;</script>"])]))

    def test_sources_list_with_data_external_classic_script(self):
        self.start_web_server(test_dir=os.path.join(DevtoolsTests.script_path, "devtools_tests/sources"))
        self.run_servoshell(url=f"data:text/html,<script src=\"{self.base_url}/classic.js\"></script>")
        self.assert_sources_list(1, set([tuple([f"{self.base_url}/classic.js"])]))

    def test_sources_list_with_data_empty_inline_module_script(self):
        self.run_servoshell(url="data:text/html,<script type=module></script>")
        self.assert_sources_list(1, set([tuple()]))

    def test_sources_list_with_data_inline_module_script(self):
        self.run_servoshell(url="data:text/html,<script type=module>;</script>")
        self.assert_sources_list(1, set([tuple(["data:text/html,<script type=module>;</script>"])]))

    # Sets `base_url` and `web_server` and `web_server_thread`.
    def start_web_server(self, *, test_dir=None):
        if test_dir is None:
            test_dir = os.path.join(DevtoolsTests.script_path, "devtools_tests")
        base_url = Future()

        class Handler(http.server.SimpleHTTPRequestHandler):
            def __init__(self, *args, **kwargs):
                super().__init__(*args, directory=test_dir, **kwargs)

            def log_message(self, format, *args):
                if LOG_REQUESTS:
                    return super().log_message(format, *args)

        def server_thread():
            self.web_server = socketserver.TCPServer(("0.0.0.0", 0), Handler)
            base_url.set_result(f"http://127.0.0.1:{self.web_server.server_address[1]}")
            self.web_server.serve_forever()

        # Start a web server for the test.
        self.web_server_thread = Thread(target=server_thread)
        self.web_server_thread.start()
        self.base_url = base_url.result(1)

    # Sets `servoshell`.
    def run_servoshell(self, *, url=None):
        # Change this setting if you want to debug Servo.
        os.environ["RUST_LOG"] = "error,devtools=warn"

        # Run servoshell.
        if url is None:
            url = f"{self.base_url}/test.html"
        self.servoshell = subprocess.Popen(["target/release/servo", "--devtools=6080", url])

        # FIXME: Don’t do this
        time.sleep(1)

    def tearDown(self):
        # Terminate servoshell.
        if self.servoshell is not None:
            self.servoshell.terminate()
            self.servoshell = None

        # Stop the web server.
        if self.web_server is not None:
            self.web_server.shutdown()
            self.web_server = None
        if self.web_server_thread is not None:
            self.web_server_thread.join()
            self.web_server_thread = None
        if self.base_url is not None:
            self.base_url = None

    def assert_sources_list(self, expected_targets: int, expected_urls_by_target: set[tuple[str]]):
        client = RDPClient()
        client.connect("127.0.0.1", 6080)
        root = RootActor(client)
        tabs = root.list_tabs()
        tab_dict = tabs[0]
        tab = TabActor(client, tab_dict["actor"])
        watcher = tab.get_watcher()
        watcher = WatcherActor(client, watcher["actor"])

        done = Future()
        targets = []

        def on_target(data):
            try:
                targets.append(data["target"])
                if len(targets) == expected_targets:
                    done.set_result(None)
            except Exception as e:
                # Raising here does nothing, for some reason.
                # Send the exception back so it can be raised.
                done.set_result(e)

        client.add_event_listener(
            watcher.actor_id, Events.Watcher.TARGET_AVAILABLE_FORM, on_target,
        )
        watcher.watch_targets(WatcherActor.Targets.FRAME)
        watcher.watch_targets(WatcherActor.Targets.WORKER)

        result: Optional[Exception] = done.result(1)
        if result:
            raise result
        done = Future()
        # NOTE: breaks if two targets have the same list of source urls.
        # This should really be a multiset, but Python does not have multisets.
        actual_urls_by_target: set[tuple[str]] = set()

        def on_source_resource(data):
            for [resource_type, sources] in data["array"]:
                try:
                    self.assertEqual(resource_type, "source")
                    source_urls = tuple([source["url"] for source in sources])
                    self.assertFalse(source_urls in sources)  # See NOTE above
                    actual_urls_by_target.add(source_urls)
                    if len(actual_urls_by_target) == expected_targets:
                        done.set_result(None)
                except Exception as e:
                    # Raising here does nothing, for some reason.
                    # Send the exception back so it can be raised.
                    done.set_result(e)

        for target in targets:
            client.add_event_listener(
                target["actor"],
                Events.Watcher.RESOURCES_AVAILABLE_ARRAY,
                on_source_resource,
            )
        watcher.watch_resources([Resources.SOURCE])

        result: Optional[Exception] = done.result(1)
        if result:
            raise result
        self.assertEqual(actual_urls_by_target, expected_urls_by_target)
        client.disconnect()


def run_tests(script_path):
    DevtoolsTests.script_path = script_path
    verbosity = 1 if logging.getLogger().level >= logging.WARN else 2
    suite = unittest.TestLoader().loadTestsFromTestCase(DevtoolsTests)
    return unittest.TextTestRunner(verbosity=verbosity).run(suite).wasSuccessful()
