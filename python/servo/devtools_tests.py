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

from servo.command_base import BuildType

# Set this to true to log requests in the internal web servers.
LOG_REQUESTS = False


class DevtoolsTests(unittest.IsolatedAsyncioTestCase):
    # /path/to/servo/python/servo
    script_path = None
    build_type: Optional[BuildType] = None

    def __init__(self, methodName="runTest"):
        super().__init__(methodName)
        self.servoshell = None
        self.base_urls = None
        self.web_servers = None
        self.web_server_threads = None

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
        self.assert_sources_list(
            2,
            set(
                [
                    tuple(
                        [
                            f"{self.base_urls[0]}/classic.js",
                            f"{self.base_urls[0]}/test.html",
                            f"{self.base_urls[1]}/classic.js",
                            f"{self.base_urls[0]}/test.html",
                        ]
                    ),
                    tuple([f"{self.base_urls[0]}/worker.js"]),
                ]
            ),
        )

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
        self.run_servoshell(url=f'data:text/html,<script src="{self.base_urls[0]}/classic.js"></script>')
        self.assert_sources_list(1, set([tuple([f"{self.base_urls[0]}/classic.js"])]))

    def test_sources_list_with_data_empty_inline_module_script(self):
        self.run_servoshell(url="data:text/html,<script type=module></script>")
        self.assert_sources_list(1, set([tuple()]))

    def test_sources_list_with_data_inline_module_script(self):
        self.run_servoshell(url="data:text/html,<script type=module>;</script>")
        self.assert_sources_list(1, set([tuple(["data:text/html,<script type=module>;</script>"])]))

    def test_source_content_inline_script(self):
        script_tag = "<script>console.log('Hello, world!')</script>"
        self.run_servoshell(url=f"data:text/html,{script_tag}")
        self.assert_source_content(f"data:text/html,{script_tag}", script_tag)

    def test_source_content_external_script(self):
        self.start_web_server(test_dir=os.path.join(DevtoolsTests.script_path, "devtools_tests/sources"))
        self.run_servoshell(url=f'data:text/html,<script src="{self.base_urls[0]}/classic.js"></script>')
        expected_content = 'console.log("external classic");\n'
        self.assert_source_content(f"{self.base_urls[0]}/classic.js", expected_content)

    def test_source_content_html_file(self):
        self.start_web_server(test_dir=self.get_test_path("sources"))
        self.run_servoshell()
        expected_content = open(self.get_test_path("sources/test.html")).read()
        self.assert_source_content(f"{self.base_urls[0]}/test.html", expected_content)

    def test_source_content_with_inline_module_import_external(self):
        self.start_web_server(test_dir=self.get_test_path("sources_content_with_inline_module_import_external"))
        self.run_servoshell()
        expected_content = open(
            self.get_test_path("sources_content_with_inline_module_import_external/test.html")
        ).read()
        self.assert_source_content(f"{self.base_urls[0]}/test.html", expected_content)

    # Test case that uses innerHTML and would actually need the HTML parser
    # (innerHTML has a fast path for values that don’t contain b'&' | b'\0' | b'<' | b'\r')
    def test_source_content_inline_script_with_inner_html(self):
        script_tag = '<div id="el"></div><script>el.innerHTML="<p>test"</script>'
        self.run_servoshell(url=f"data:text/html,{script_tag}")
        self.assert_source_content(f"data:text/html,{script_tag}", script_tag)

    # Test case that uses outerHTML and would actually need the HTML parser
    # (innerHTML has a fast path for values that don’t contain b'&' | b'\0' | b'<' | b'\r')
    def test_source_content_inline_script_with_outer_html(self):
        script_tag = '<div id="el"></div><script>el.outerHTML="<p>test"</script>'
        self.run_servoshell(url=f"data:text/html,{script_tag}")
        self.assert_source_content(f"data:text/html,{script_tag}", script_tag)

    # Test case that uses DOMParser and would actually need the HTML parser
    # (innerHTML has a fast path for values that don’t contain b'&' | b'\0' | b'<' | b'\r')
    def test_source_content_inline_script_with_domparser(self):
        script_tag = '<script>(new DOMParser).parseFromString("<p>test","text/html")</script>'
        self.run_servoshell(url=f"data:text/html,{script_tag}")
        self.assert_source_content(f"data:text/html,{script_tag}", script_tag)

    # Test case that uses XMLHttpRequest#responseXML and would actually need the HTML parser
    # (innerHTML has a fast path for values that don’t contain b'&' | b'\0' | b'<' | b'\r')
    def test_source_content_inline_script_with_responsexml(self):
        self.start_web_server(test_dir=self.get_test_path("sources_content_with_responsexml"))
        self.run_servoshell()
        expected_content = open(self.get_test_path("sources_content_with_responsexml/test.html")).read()
        self.assert_source_content(f"{self.base_urls[0]}/test.html", expected_content)

    # Sets `base_url` and `web_server` and `web_server_thread`.
    def start_web_server(self, *, test_dir=None, num_servers=2):
        assert self.base_urls is None and self.web_servers is None and self.web_server_threads is None
        if test_dir is None:
            test_dir = os.path.join(DevtoolsTests.script_path, "devtools_tests")
        base_urls = [Future() for i in range(num_servers)]
        self.web_servers = [None for i in range(num_servers)]
        self.web_server_threads = [None for i in range(num_servers)]

        class Handler(http.server.SimpleHTTPRequestHandler):
            def __init__(self, *args, **kwargs):
                super().__init__(*args, directory=test_dir, **kwargs)

            def log_message(self, format, *args):
                if LOG_REQUESTS:
                    return super().log_message(format, *args)

        def server_thread(index):
            # There may be client sockets still open in TIME_WAIT state from previous tests, and they may stay open for
            # some minutes. Set SO_REUSEADDR to avoid bind failure with EADDRINUSE in these cases.
            # <https://stackoverflow.com/questions/14388706>
            socketserver.TCPServer.allow_reuse_address = True
            # Listen on all IPv4 interfaces, port 10000 + index.
            web_server = socketserver.TCPServer(("127.0.0.1", 10000 + index), Handler)
            base_url = f"http://127.0.0.1:{web_server.server_address[1]}"
            base_urls[index].set_result(base_url)
            self.web_servers[index] = web_server
            web_server.serve_forever()

        # Start a web server for the test.
        for index in range(num_servers):
            thread = Thread(target=server_thread, args=[index])
            self.web_server_threads[index] = thread
            thread.start()
        self.base_urls = [base_url.result(1) for base_url in base_urls]

    # Sets `servoshell`.
    def run_servoshell(self, *, url=None):
        # Change this setting if you want to debug Servo.
        os.environ["RUST_LOG"] = "error,devtools=warn"

        # Run servoshell.
        if url is None:
            url = f"{self.base_urls[0]}/test.html"
        self.servoshell = subprocess.Popen([f"target/{self.build_type.directory_name()}/servo", "--devtools=6080", url])

        # FIXME: Don’t do this
        time.sleep(1)

    def tearDown(self):
        # Terminate servoshell.
        if self.servoshell is not None:
            self.servoshell.terminate()
            self.servoshell = None

        # Stop the web servers.
        if self.web_servers is not None:
            for web_server in self.web_servers:
                web_server.shutdown()
                web_server.server_close()
            os.system("ss -lnp | egrep '1000[01]'")
            self.web_servers = None
        if self.web_server_threads is not None:
            for web_server_thread in self.web_server_threads:
                web_server_thread.join()
            self.web_server_threads = None
        if self.base_urls is not None:
            self.base_urls = None

    def _setup_devtools_client(self, expected_targets=1):
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
            watcher.actor_id,
            Events.Watcher.TARGET_AVAILABLE_FORM,
            on_target,
        )
        watcher.watch_targets(WatcherActor.Targets.FRAME)
        watcher.watch_targets(WatcherActor.Targets.WORKER)

        result: Optional[Exception] = done.result(1)
        if result:
            raise result

        return client, watcher, targets

    def assert_sources_list(self, expected_targets: int, expected_urls_by_target: set[tuple[str]]):
        client, watcher, targets = self._setup_devtools_client(expected_targets)
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

    def assert_source_content(self, source_url: str, expected_content: str):
        client, watcher, targets = self._setup_devtools_client()

        done = Future()
        source_actors = {}

        def on_source_resource(data):
            for [resource_type, sources] in data["array"]:
                try:
                    self.assertEqual(resource_type, "source")
                    for source in sources:
                        if source["url"] == source_url:
                            source_actors[source_url] = source["actor"]
                            done.set_result(None)
                except Exception as e:
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

        # We found at least one source with the given url.
        self.assertIn(source_url, source_actors)
        source_actor = source_actors[source_url]

        response = client.send_receive({"to": source_actor, "type": "source"})

        self.assertEqual(response["source"], expected_content)

        client.disconnect()

    def get_test_path(self, path: str) -> str:
        return os.path.join(DevtoolsTests.script_path, os.path.join("devtools_tests", path))


def run_tests(script_path, build_type: BuildType):
    DevtoolsTests.script_path = script_path
    DevtoolsTests.build_type = build_type
    verbosity = 1 if logging.getLogger().level >= logging.WARN else 2
    suite = unittest.TestLoader().loadTestsFromTestCase(DevtoolsTests)
    return unittest.TextTestRunner(verbosity=verbosity).run(suite).wasSuccessful()
