# Copyright 2013 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

from concurrent.futures import Future
from dataclasses import dataclass
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


@dataclass(frozen=True)
class Source:
    introduction_type: str
    url: str


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

    # Sources list

    def test_sources_list(self):
        self.start_web_server(test_dir=os.path.join(DevtoolsTests.script_path, "devtools_tests/sources"))
        self.run_servoshell()
        self.assert_sources_list(
            set(
                [
                    # TODO: update expectations when we fix ES modules
                    tuple(
                        [
                            Source("srcScript", f"{self.base_urls[0]}/classic.js"),
                            Source("inlineScript", f"{self.base_urls[0]}/test.html"),
                            Source("inlineScript", f"{self.base_urls[0]}/test.html"),
                            Source("srcScript", f"{self.base_urls[1]}/classic.js"),
                            Source("importedModule", f"{self.base_urls[0]}/module.js"),
                        ]
                    ),
                    tuple([Source("Worker", f"{self.base_urls[0]}/classic_worker.js")]),
                ]
            ),
        )

    def test_sources_list_with_data_no_scripts(self):
        self.run_servoshell(url="data:text/html,")
        self.assert_sources_list(set([tuple()]))

    # Sources list for `introductionType` = `inlineScript` and `srcScript`

    def test_sources_list_with_data_empty_inline_classic_script(self):
        self.run_servoshell(url="data:text/html,<script></script>")
        self.assert_sources_list(set([tuple()]))

    def test_sources_list_with_data_inline_classic_script(self):
        self.run_servoshell(url="data:text/html,<script>;</script>")
        self.assert_sources_list(set([tuple([Source("inlineScript", "data:text/html,<script>;</script>")])]))

    def test_sources_list_with_data_external_classic_script(self):
        self.start_web_server(test_dir=os.path.join(DevtoolsTests.script_path, "devtools_tests/sources"))
        self.run_servoshell(url=f'data:text/html,<script src="{self.base_urls[0]}/classic.js"></script>')
        self.assert_sources_list(set([tuple([Source("srcScript", f"{self.base_urls[0]}/classic.js")])]))

    def test_sources_list_with_data_empty_inline_module_script(self):
        self.run_servoshell(url="data:text/html,<script type=module></script>")
        self.assert_sources_list(set([tuple()]))

    def test_sources_list_with_data_inline_module_script(self):
        self.run_servoshell(url="data:text/html,<script type=module>;</script>")
        self.assert_sources_list(
            set([tuple([Source("inlineScript", "data:text/html,<script type=module>;</script>")])])
        )

    def test_sources_list_with_data_external_module_script(self):
        self.start_web_server(test_dir=os.path.join(DevtoolsTests.script_path, "devtools_tests/sources"))
        self.run_servoshell(url=f"{self.base_urls[0]}/test_sources_list_with_data_external_module_script.html")
        self.assert_sources_list(set([tuple([Source("srcScript", f"{self.base_urls[0]}/module.js")])]))

    # Sources list for `introductionType` = `importedModule`

    def test_sources_list_with_static_import_module(self):
        self.start_web_server(test_dir=os.path.join(DevtoolsTests.script_path, "devtools_tests/sources"))
        self.run_servoshell(url=f"{self.base_urls[0]}/test_sources_list_with_static_import_module.html")
        self.assert_sources_list(
            set(
                [
                    tuple(
                        [
                            Source(
                                "inlineScript", f"{self.base_urls[0]}/test_sources_list_with_static_import_module.html"
                            ),
                            Source("importedModule", f"{self.base_urls[0]}/module.js"),
                        ]
                    )
                ]
            ),
        )

    def test_sources_list_with_dynamic_import_module(self):
        self.start_web_server(test_dir=os.path.join(DevtoolsTests.script_path, "devtools_tests/sources"))
        self.run_servoshell(url=f"{self.base_urls[0]}/test_sources_list_with_dynamic_import_module.html")
        self.assert_sources_list(
            set(
                [
                    tuple(
                        [
                            Source(
                                "inlineScript", f"{self.base_urls[0]}/test_sources_list_with_dynamic_import_module.html"
                            ),
                            Source("importedModule", f"{self.base_urls[0]}/module.js"),
                        ]
                    )
                ]
            ),
        )

    # Sources list for `introductionType` = `Worker`

    def test_sources_list_with_classic_worker(self):
        self.start_web_server(test_dir=os.path.join(DevtoolsTests.script_path, "devtools_tests/sources"))
        self.run_servoshell(url=f"{self.base_urls[0]}/test_sources_list_with_classic_worker.html")
        self.assert_sources_list(
            set(
                [
                    tuple(
                        [
                            Source("inlineScript", f"{self.base_urls[0]}/test_sources_list_with_classic_worker.html"),
                        ]
                    ),
                    tuple(
                        [
                            Source("Worker", f"{self.base_urls[0]}/classic_worker.js"),
                        ]
                    ),
                ]
            ),
        )

    def test_sources_list_with_module_worker(self):
        self.start_web_server(test_dir=os.path.join(DevtoolsTests.script_path, "devtools_tests/sources"))
        self.run_servoshell(url=f"{self.base_urls[0]}/test_sources_list_with_module_worker.html")
        self.assert_sources_list(
            set(
                [
                    tuple(
                        [
                            Source("inlineScript", f"{self.base_urls[0]}/test_sources_list_with_module_worker.html"),
                        ]
                    ),
                    tuple(
                        [
                            Source("Worker", f"{self.base_urls[0]}/module_worker.js"),
                        ]
                    ),
                ]
            ),
        )

    # Sources list for `introductionType` set to values that require `displayURL` (`//# sourceURL`)

    def test_sources_list_with_injected_script_write_and_display_url(self):
        self.run_servoshell(
            url='data:text/html,<script>document.write("<script>//%23 sourceURL=http://test</scr"+"ipt>")</script>'
        )
        self.assert_sources_list(
            set(
                [
                    tuple(
                        [
                            Source(
                                "inlineScript",
                                'data:text/html,<script>document.write("<script>//%23 sourceURL=http://test</scr"+"ipt>")</script>',
                            ),
                            Source("injectedScript", "http://test/"),
                        ]
                    )
                ]
            )
        )

    def test_sources_list_with_injected_script_write_but_no_display_url(self):
        self.run_servoshell(url='data:text/html,<script>document.write("<script>1</scr"+"ipt>")</script>')
        self.assert_sources_list(
            set(
                [
                    tuple(
                        [
                            Source(
                                "inlineScript",
                                'data:text/html,<script>document.write("<script>1</scr"+"ipt>")</script>',
                            ),
                        ]
                    )
                ]
            )
        )

    def test_sources_list_with_injected_script_append_and_display_url(self):
        script = 's=document.createElement("script");s.append("//%23 sourceURL=http://test");document.body.append(s)'
        self.run_servoshell(url=f"data:text/html,<body><script>{script}</script>")
        self.assert_sources_list(
            set(
                [
                    tuple(
                        [
                            Source(
                                "inlineScript",
                                f"data:text/html,<body><script>{script}</script>",
                            ),
                            Source("injectedScript", "http://test/"),
                        ]
                    )
                ]
            )
        )

    def test_sources_list_with_injected_script_append_but_no_display_url(self):
        script = 's=document.createElement("script");s.append("1");document.body.append(s)'
        self.run_servoshell(url=f"data:text/html,<body><script>{script}</script>")
        self.assert_sources_list(
            set(
                [
                    tuple(
                        [
                            Source(
                                "inlineScript",
                                f"data:text/html,<body><script>{script}</script>",
                            ),
                        ]
                    )
                ]
            )
        )

    def test_sources_list_with_eval_and_display_url(self):
        self.run_servoshell(url='data:text/html,<script>eval("//%23 sourceURL=http://test")</script>')
        self.assert_sources_list(
            set(
                [
                    tuple(
                        [
                            Source(
                                "inlineScript", 'data:text/html,<script>eval("//%23 sourceURL=http://test")</script>'
                            ),
                            Source("eval", "http://test/"),
                        ]
                    )
                ]
            )
        )

    def test_sources_list_with_eval_but_no_display_url(self):
        self.run_servoshell(url='data:text/html,<script>eval("1")</script>')
        self.assert_sources_list(set([tuple([Source("inlineScript", 'data:text/html,<script>eval("1")</script>')])]))

    def test_sources_list_with_debugger_eval_and_display_url(self):
        pass

    def test_sources_list_with_debugger_eval_but_no_display_url(self):
        pass

    def test_sources_list_with_function_and_display_url(self):
        self.run_servoshell(url='data:text/html,<script>new Function("//%23 sourceURL=http://test")</script>')
        self.assert_sources_list(
            set(
                [
                    tuple(
                        [
                            Source(
                                "inlineScript",
                                'data:text/html,<script>new Function("//%23 sourceURL=http://test")</script>',
                            ),
                            Source("Function", "http://test/"),
                        ]
                    )
                ]
            )
        )

    def test_sources_list_with_function_but_no_display_url(self):
        self.run_servoshell(url='data:text/html,<script>new Function("1")</script>')
        self.assert_sources_list(
            set(
                [
                    tuple(
                        [
                            Source("inlineScript", 'data:text/html,<script>new Function("1")</script>'),
                        ]
                    )
                ]
            )
        )

    def test_sources_list_with_javascript_url_and_display_url(self):
        # “1” prefix is a workaround for <https://github.com/servo/servo/issues/38547>
        self.run_servoshell(
            url='data:text/html,<a href="javascript:1//%23 sourceURL=http://test"></a><script>document.querySelector("a").click()</script>'
        )
        self.assert_sources_list(
            set(
                [
                    tuple(
                        [
                            Source(
                                "inlineScript",
                                'data:text/html,<a href="javascript:1//%23 sourceURL=http://test"></a><script>document.querySelector("a").click()</script>',
                            ),
                            Source("javascriptURL", "http://test/"),
                        ]
                    )
                ]
            )
        )

    def test_sources_list_with_javascript_url_but_no_display_url(self):
        self.run_servoshell(url='data:text/html,<a href="javascript:1"></a>')
        self.assert_sources_list(set([tuple([])]))

    @unittest.expectedFailure
    def test_sources_list_with_event_handler_and_display_url(self):
        self.run_servoshell(url='data:text/html,<a onclick="//%23 sourceURL=http://test"></a>')
        self.assert_sources_list(
            set(
                [
                    tuple(
                        [
                            Source("eventHandler", "http://test/"),
                        ]
                    )
                ]
            )
        )

    def test_sources_list_with_event_handler_but_no_display_url(self):
        self.run_servoshell(url='data:text/html,<a onclick="1"></a>')
        self.assert_sources_list(set([tuple([])]))

    @unittest.expectedFailure
    def test_sources_list_with_dom_timer_and_display_url(self):
        self.run_servoshell(url='data:text/html,<script>setTimeout("//%23 sourceURL=http://test",0)</script>')
        self.assert_sources_list(
            set(
                [
                    tuple(
                        [
                            Source("domTimer", "http://test/"),
                        ]
                    )
                ]
            )
        )

    @unittest.expectedFailure
    def test_sources_list_with_dom_timer_but_no_display_url(self):
        self.run_servoshell(url='data:text/html,<script>setTimeout("1",0)</script>')
        self.assert_sources_list(set([tuple([])]))

    # Sources list for scripts with `displayURL` (`//# sourceURL`), despite not being required by `introductionType`

    def test_sources_list_with_inline_script_and_display_url(self):
        self.run_servoshell(url="data:text/html,<script>//%23 sourceURL=http://test</script>")
        self.assert_sources_list(
            set(
                [
                    tuple(
                        [
                            Source("inlineScript", "http://test/"),
                        ]
                    )
                ]
            )
        )

    # Extra test case for situation where `//# sourceURL` can’t be parsed with page url as base.
    def test_sources_list_with_inline_script_but_invalid_display_url(self):
        self.run_servoshell(url="data:text/html,<script>//%23 sourceURL=test</script>")
        self.assert_sources_list(
            set(
                [
                    tuple(
                        [
                            Source("inlineScript", "data:text/html,<script>//%23 sourceURL=test</script>"),
                        ]
                    )
                ]
            )
        )

    def test_sources_list_with_inline_script_but_no_display_url(self):
        self.run_servoshell(url="data:text/html,<script>1</script>")
        self.assert_sources_list(
            set(
                [
                    tuple(
                        [
                            Source("inlineScript", "data:text/html,<script>1</script>"),
                        ]
                    )
                ]
            )
        )

    # Sources list for inline scripts in `<iframe srcdoc>`

    @unittest.expectedFailure
    def test_sources_list_with_iframe_srcdoc_and_display_url(self):
        self.run_servoshell(url='data:text/html,<iframe srcdoc="<script>//%23 sourceURL=http://test</script>">')
        self.assert_sources_list(
            set(
                [
                    tuple(
                        [
                            Source("inlineScript", "http://test/"),
                        ]
                    )
                ]
            )
        )

    @unittest.expectedFailure
    def test_sources_list_with_iframe_srcdoc_but_no_display_url(self):
        self.run_servoshell(url='data:text/html,<iframe srcdoc="<script>1</script>">')
        self.assert_sources_list(
            set(
                [
                    tuple(
                        [
                            # FIXME: it’s not really gonna be 0
                            Source("inlineScript", "about:srcdoc#0"),
                        ]
                    )
                ]
            )
        )

    @unittest.expectedFailure
    def test_sources_list_with_iframe_srcdoc_multiple_inline_scripts(self):
        self.run_servoshell(
            url='data:text/html,<iframe srcdoc="<script>//%23 sourceURL=http://test</script><script>2</script>">'
        )
        self.assert_sources_list(
            set(
                [
                    tuple(
                        [
                            Source("inlineScript", "http://test/"),
                            # FIXME: it’s not really gonna be 0
                            Source("inlineScript", "about:srcdoc#0"),
                        ]
                    )
                ]
            )
        )

    # Source contents

    def test_source_content_inline_script(self):
        script_tag = "<script>console.log('Hello, world!')</script>"
        self.run_servoshell(url=f"data:text/html,{script_tag}")
        self.assert_source_content(Source("inlineScript", f"data:text/html,{script_tag}"), script_tag)

    def test_source_content_external_script(self):
        self.start_web_server(test_dir=os.path.join(DevtoolsTests.script_path, "devtools_tests/sources"))
        self.run_servoshell(url=f'data:text/html,<script src="{self.base_urls[0]}/classic.js"></script>')
        expected_content = 'console.log("external classic");\n'
        self.assert_source_content(Source("srcScript", f"{self.base_urls[0]}/classic.js"), expected_content)

    def test_source_content_html_file(self):
        self.start_web_server(test_dir=self.get_test_path("sources"))
        self.run_servoshell()
        expected_content = open(self.get_test_path("sources/test.html")).read()
        self.assert_source_content(Source("inlineScript", f"{self.base_urls[0]}/test.html"), expected_content)

    def test_source_content_with_inline_module_import_external(self):
        self.start_web_server(test_dir=self.get_test_path("sources_content_with_inline_module_import_external"))
        self.run_servoshell()
        expected_content = open(
            self.get_test_path("sources_content_with_inline_module_import_external/test.html")
        ).read()
        self.assert_source_content(Source("inlineScript", f"{self.base_urls[0]}/test.html"), expected_content)

    # Test case that uses innerHTML and would actually need the HTML parser
    # (innerHTML has a fast path for values that don’t contain b'&' | b'\0' | b'<' | b'\r')
    def test_source_content_inline_script_with_inner_html(self):
        script_tag = '<div id="el"></div><script>el.innerHTML="<p>test"</script>'
        self.run_servoshell(url=f"data:text/html,{script_tag}")
        self.assert_source_content(Source("inlineScript", f"data:text/html,{script_tag}"), script_tag)

    # Test case that uses outerHTML and would actually need the HTML parser
    # (innerHTML has a fast path for values that don’t contain b'&' | b'\0' | b'<' | b'\r')
    def test_source_content_inline_script_with_outer_html(self):
        script_tag = '<div id="el"></div><script>el.outerHTML="<p>test"</script>'
        self.run_servoshell(url=f"data:text/html,{script_tag}")
        self.assert_source_content(Source("inlineScript", f"data:text/html,{script_tag}"), script_tag)

    # Test case that uses DOMParser and would actually need the HTML parser
    # (innerHTML has a fast path for values that don’t contain b'&' | b'\0' | b'<' | b'\r')
    def test_source_content_inline_script_with_domparser(self):
        script_tag = '<script>(new DOMParser).parseFromString("<p>test","text/html")</script>'
        self.run_servoshell(url=f"data:text/html,{script_tag}")
        self.assert_source_content(Source("inlineScript", f"data:text/html,{script_tag}"), script_tag)

    # Test case that uses XMLHttpRequest#responseXML and would actually need the HTML parser
    # (innerHTML has a fast path for values that don’t contain b'&' | b'\0' | b'<' | b'\r')
    def test_source_content_inline_script_with_responsexml(self):
        self.start_web_server(test_dir=self.get_test_path("sources_content_with_responsexml"))
        self.run_servoshell()
        expected_content = open(self.get_test_path("sources_content_with_responsexml/test.html")).read()
        self.assert_source_content(Source("inlineScript", f"{self.base_urls[0]}/test.html"), expected_content)

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

    def assert_sources_list(self, expected_sources_by_target: set[tuple[Source]]):
        expected_targets = len(expected_sources_by_target)
        client, watcher, targets = self._setup_devtools_client(expected_targets)
        done = Future()
        # NOTE: breaks if two targets have the same list of source urls.
        # This should really be a multiset, but Python does not have multisets.
        actual_sources_by_target: set[tuple[Source]] = set()

        def on_source_resource(data):
            for [resource_type, sources] in data["array"]:
                try:
                    self.assertEqual(resource_type, "source")
                    source_urls = tuple([Source(source["introductionType"], source["url"]) for source in sources])
                    self.assertFalse(source_urls in actual_sources_by_target)  # See NOTE above
                    actual_sources_by_target.add(source_urls)
                    if len(actual_sources_by_target) == expected_targets:
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
        self.assertEqual(actual_sources_by_target, expected_sources_by_target)
        client.disconnect()

    def assert_source_content(self, expected_source: Source, expected_content: str):
        client, watcher, targets = self._setup_devtools_client()

        done = Future()
        source_actors = {}

        def on_source_resource(data):
            for [resource_type, sources] in data["array"]:
                try:
                    self.assertEqual(resource_type, "source")
                    for source in sources:
                        if Source(source["introductionType"], source["url"]) == expected_source:
                            source_actors[expected_source] = source["actor"]
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
        self.assertIn(expected_source, source_actors)
        source_actor = source_actors[expected_source]

        response = client.send_receive({"to": source_actor, "type": "source"})

        self.assertEqual(response["source"], expected_content)

        client.disconnect()

    def get_test_path(self, path: str) -> str:
        return os.path.join(DevtoolsTests.script_path, os.path.join("devtools_tests", path))


def run_tests(script_path, build_type: BuildType, test_names: list[str]):
    DevtoolsTests.script_path = script_path
    DevtoolsTests.build_type = build_type
    verbosity = 1 if logging.getLogger().level >= logging.WARN else 2
    loader = unittest.TestLoader()
    if test_names:
        patterns = []
        # unittest.main() `-k` treats any `pattern` not containing `*` like `*pattern*`
        for pattern in test_names:
            if "*" in pattern:
                patterns.append(pattern)
            else:
                patterns.append(f"*{pattern}*")
        loader.testNamePatterns = patterns
    suite = loader.loadTestsFromTestCase(DevtoolsTests)
    print(f"Running {suite.countTestCases()} tests:")
    for test in suite:
        print(f"- {test}")
    print()
    return unittest.TextTestRunner(verbosity=verbosity).run(suite).wasSuccessful()
