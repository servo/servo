# Copyright 2013 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

from __future__ import annotations
from concurrent.futures import Future
from dataclasses import dataclass
import logging
import socket
import sys
from geckordp.actors.root import RootActor
from geckordp.actors.descriptors.tab import TabActor
from geckordp.actors.watcher import WatcherActor
from geckordp.actors.web_console import WebConsoleActor
from geckordp.actors.resources import Resources
from geckordp.actors.events import Events
from geckordp.rdp_client import RDPClient
import http.server
import os.path
import socketserver
import subprocess
import time
from threading import Thread
from typing import Any, Iterable, Optional, TypeVar
import unittest

from collections import Counter

# Set this to true to log requests in the internal web servers.
LOG_REQUESTS = False


@dataclass(frozen=True, order=True)
class Source:
    introduction_type: str
    url: str


@dataclass
class Devtools:
    client: RDPClient
    watcher: WatcherActor
    targets: list
    exited: bool = False

    def connect(*, expected_targets: int = 1) -> Devtools:
        """
        Connect to the Servo devtools server.
        You should use a `with` statement to ensure we disconnect unconditionally.
        """
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

        return Devtools(client, watcher, targets)

    def __getattribute__(self, name: str) -> Any:
        """
        Access a property, raising a ValueError if the instance was previously marked as exited.
        """
        if name != "exited" and object.__getattribute__(self, "exited"):
            raise ValueError("Devtools instance must not be used after __exit__()")
        return object.__getattribute__(self, name)

    def __enter__(self) -> Devtools:
        """
        Enter the `with` context for this instance, raising a ValueError if it was previously marked as exited.
        """
        if self.exited:
            raise ValueError("Devtools instance must not be used after __exit__()")
        return self

    def __exit__(self, exc_type, exc_value, traceback) -> None:
        """
        Exit the `with` context for this instance, disconnecting the client and marking it as exited.
        Does not raise a ValueError if it was previously marked as exited, so you can nest `with` statements.
        """
        if not self.exited:
            # Ignore any return value; we never want to return True to suppress exceptions
            self.client.__exit__(exc_type, exc_value, traceback)
        self.exited = True


# TODO: Use new syntax in python 3.12
# <https://docs.python.org/3/reference/compound_stmts.html#generic-functions>
# <https://docs.python.org/3/library/typing.html#user-defined-generic-types>
T = TypeVar("T")
FrozenMultiset = tuple[tuple[T, int], ...]


def frozen_multiset(items: Iterable[T] = []) -> FrozenMultiset[T]:
    """
    Simulate a frozen multiset using a tuple of tuples.
    Python does not have one yet:
    <https://bugs.python.org/issue40411>
    <https://peps.python.org/pep-0603/>
    """
    # First make a mutable multiset
    result = Counter(items)
    # then convert it to a tuple with a stable order
    return tuple(sorted(result.items()))


class DevtoolsTests(unittest.IsolatedAsyncioTestCase):
    # /path/to/servo/python/servo
    script_path = None
    servo_binary: Optional[str] = None
    base_urls = None
    web_servers = None
    web_server_threads = None

    def __init__(self, methodName="runTest"):
        super().__init__(methodName)
        self.servoshell = None

    # Watcher tests

    def test_watcher_returns_same_breakpoint_list_actor_every_time(self):
        self.run_servoshell(url="data:text/html,")
        with Devtools.connect() as devtools:
            response1 = devtools.watcher.get_breakpoint_list_actor()
            response2 = devtools.watcher.get_breakpoint_list_actor()
            self.assertEqual(response1["breakpointList"]["actor"], response2["breakpointList"]["actor"])

    # Sources list
    # Classic script vs module script:
    # - <https://html.spec.whatwg.org/multipage/#classic-script>
    # - <https://html.spec.whatwg.org/multipage/#module-script>
    # Worker scripts can be classic or module:
    # - <https://html.spec.whatwg.org/multipage/#fetch-a-classic-worker-script>
    # - <https://html.spec.whatwg.org/multipage/#fetch-a-module-worker-script-tree>
    # Non-worker(?) script sources can be inline, external, or blob.
    # Worker script sources can be external or blob.

    def test_sources_list(self):
        self.run_servoshell(url=f"{self.base_urls[0]}/sources/test.html")
        self.assert_sources_list(
            Counter(
                [
                    frozen_multiset(
                        [
                            Source("srcScript", f"{self.base_urls[0]}/sources/classic.js"),
                            Source("inlineScript", f"{self.base_urls[0]}/sources/test.html"),
                            Source("inlineScript", f"{self.base_urls[0]}/sources/test.html"),
                            Source("srcScript", f"{self.base_urls[1]}/sources/classic.js"),
                            Source("importedModule", f"{self.base_urls[0]}/sources/module.js"),
                        ]
                    ),
                    frozen_multiset([Source("Worker", f"{self.base_urls[0]}/sources/classic_worker.js")]),
                ]
            ),
        )

    def test_sources_list_with_data_no_scripts(self):
        self.run_servoshell(url="data:text/html,")
        self.assert_sources_list(Counter([frozen_multiset()]))

    # Sources list for `introductionType` = `inlineScript` and `srcScript`

    def test_sources_list_with_data_empty_inline_classic_script(self):
        self.run_servoshell(url="data:text/html,<script></script>")
        self.assert_sources_list(Counter([frozen_multiset()]))

    def test_sources_list_with_data_inline_classic_script(self):
        self.run_servoshell(url="data:text/html,<script>;</script>")
        self.assert_sources_list(
            Counter([frozen_multiset([Source("inlineScript", "data:text/html,<script>;</script>")])])
        )

    def test_sources_list_with_data_external_classic_script(self):
        self.run_servoshell(url=f'data:text/html,<script src="{self.base_urls[0]}/sources/classic.js"></script>')
        self.assert_sources_list(
            Counter([frozen_multiset([Source("srcScript", f"{self.base_urls[0]}/sources/classic.js")])])
        )

    def test_sources_list_with_data_empty_inline_module_script(self):
        self.run_servoshell(url="data:text/html,<script type=module></script>")
        self.assert_sources_list(Counter([frozen_multiset()]))

    def test_sources_list_with_data_inline_module_script(self):
        self.run_servoshell(url="data:text/html,<script type=module>;</script>")
        self.assert_sources_list(
            Counter([frozen_multiset([Source("inlineScript", "data:text/html,<script type=module>;</script>")])])
        )

    def test_sources_list_with_data_external_module_script(self):
        self.run_servoshell(url=f"{self.base_urls[0]}/sources/test_sources_list_with_data_external_module_script.html")
        self.assert_sources_list(
            Counter([frozen_multiset([Source("srcScript", f"{self.base_urls[0]}/sources/module.js")])])
        )

    # Sources list for `introductionType` = `importedModule`

    def test_sources_list_with_static_import_module(self):
        self.run_servoshell(url=f"{self.base_urls[0]}/sources/test_sources_list_with_static_import_module.html")
        self.assert_sources_list(
            Counter(
                [
                    frozen_multiset(
                        [
                            Source(
                                "inlineScript",
                                f"{self.base_urls[0]}/sources/test_sources_list_with_static_import_module.html",
                            ),
                            Source("importedModule", f"{self.base_urls[0]}/sources/module.js"),
                        ]
                    )
                ]
            ),
        )

    def test_sources_list_with_dynamic_import_module(self):
        self.run_servoshell(url=f"{self.base_urls[0]}/sources/test_sources_list_with_dynamic_import_module.html")
        self.assert_sources_list(
            Counter(
                [
                    frozen_multiset(
                        [
                            Source(
                                "inlineScript",
                                f"{self.base_urls[0]}/sources/test_sources_list_with_dynamic_import_module.html",
                            ),
                            Source("importedModule", f"{self.base_urls[0]}/sources/module.js"),
                        ]
                    )
                ]
            ),
        )

    # Sources list for `introductionType` = `Worker`

    def test_sources_list_with_classic_worker(self):
        self.run_servoshell(url=f"{self.base_urls[0]}/sources/test_sources_list_with_classic_worker.html")
        self.assert_sources_list(
            Counter(
                [
                    frozen_multiset(
                        [
                            Source(
                                "inlineScript",
                                f"{self.base_urls[0]}/sources/test_sources_list_with_classic_worker.html",
                            ),
                        ]
                    ),
                    frozen_multiset(
                        [
                            Source("Worker", f"{self.base_urls[0]}/sources/classic_worker.js"),
                        ]
                    ),
                ]
            ),
        )

    def test_sources_list_with_module_worker(self):
        self.run_servoshell(url=f"{self.base_urls[0]}/sources/test_sources_list_with_module_worker.html")
        self.assert_sources_list(
            Counter(
                [
                    frozen_multiset(
                        [
                            Source(
                                "inlineScript", f"{self.base_urls[0]}/sources/test_sources_list_with_module_worker.html"
                            ),
                        ]
                    ),
                    frozen_multiset(
                        [
                            Source("Worker", f"{self.base_urls[0]}/sources/module_worker.js"),
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
            Counter(
                [
                    frozen_multiset(
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
            Counter(
                [
                    frozen_multiset(
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
            Counter(
                [
                    frozen_multiset(
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
            Counter(
                [
                    frozen_multiset(
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
            Counter(
                [
                    frozen_multiset(
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
        self.assert_sources_list(
            Counter([frozen_multiset([Source("inlineScript", 'data:text/html,<script>eval("1")</script>')])])
        )

    def test_sources_list_with_debugger_eval_and_display_url(self):
        self.run_servoshell(url="data:text/html,")
        with Devtools.connect() as devtools:
            console = WebConsoleActor(devtools.client, devtools.targets[0]["consoleActor"])
            evaluation_result = Future()

            async def on_evaluation_result(data: dict):
                evaluation_result.set_result(data)

            devtools.client.add_event_listener(
                console.actor_id, Events.WebConsole.EVALUATION_RESULT, on_evaluation_result
            )
            console.evaluate_js_async("//# sourceURL=http://test")
            evaluation_result.result(1)
            self.assert_sources_list(
                Counter([frozen_multiset([Source("debugger eval", "http://test/")])]), devtools=devtools
            )

    def test_sources_list_with_debugger_eval_but_no_display_url(self):
        self.run_servoshell(url="data:text/html,")
        with Devtools.connect() as devtools:
            console = WebConsoleActor(devtools.client, devtools.targets[0]["consoleActor"])
            evaluation_result = Future()

            async def on_evaluation_result(data: dict):
                evaluation_result.set_result(data)

            devtools.client.add_event_listener(
                console.actor_id, Events.WebConsole.EVALUATION_RESULT, on_evaluation_result
            )
            console.evaluate_js_async("1")
            evaluation_result.result(1)
            self.assert_sources_list(Counter([frozen_multiset([])]), devtools=devtools)

    def test_sources_list_with_function_and_display_url(self):
        self.run_servoshell(url='data:text/html,<script>new Function("//%23 sourceURL=http://test")</script>')
        self.assert_sources_list(
            Counter(
                [
                    frozen_multiset(
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
            Counter(
                [
                    frozen_multiset(
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
            Counter(
                [
                    frozen_multiset(
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
        self.assert_sources_list(Counter([frozen_multiset([])]))

    @unittest.expectedFailure
    def test_sources_list_with_event_handler_and_display_url(self):
        self.run_servoshell(url='data:text/html,<a onclick="//%23 sourceURL=http://test"></a>')
        self.assert_sources_list(
            Counter(
                [
                    frozen_multiset(
                        [
                            Source("eventHandler", "http://test/"),
                        ]
                    )
                ]
            )
        )

    def test_sources_list_with_event_handler_but_no_display_url(self):
        self.run_servoshell(url='data:text/html,<a onclick="1"></a>')
        self.assert_sources_list(Counter([frozen_multiset([])]))

    @unittest.expectedFailure
    def test_sources_list_with_dom_timer_and_display_url(self):
        self.run_servoshell(url='data:text/html,<script>setTimeout("//%23 sourceURL=http://test",0)</script>')
        self.assert_sources_list(
            Counter(
                [
                    frozen_multiset(
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
        self.assert_sources_list(Counter([frozen_multiset([])]))

    # Sources list for scripts with `displayURL` (`//# sourceURL`), despite not being required by `introductionType`

    def test_sources_list_with_inline_script_and_display_url(self):
        self.run_servoshell(url="data:text/html,<script>//%23 sourceURL=http://test</script>")
        self.assert_sources_list(
            Counter(
                [
                    frozen_multiset(
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
            Counter(
                [
                    frozen_multiset(
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
            Counter(
                [
                    frozen_multiset(
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
            Counter(
                [
                    frozen_multiset(
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
            Counter(
                [
                    frozen_multiset(
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
            Counter(
                [
                    frozen_multiset(
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
        self.run_servoshell(url=f'data:text/html,<script src="{self.base_urls[0]}/sources/classic.js"></script>')
        expected_content = 'console.log("external classic");\n'
        self.assert_source_content(Source("srcScript", f"{self.base_urls[0]}/sources/classic.js"), expected_content)

    def test_source_content_html_file(self):
        self.run_servoshell(url=f"{self.base_urls[0]}/sources/test.html")
        expected_content = open(self.get_test_path("sources/test.html")).read()
        self.assert_source_content(Source("inlineScript", f"{self.base_urls[0]}/sources/test.html"), expected_content)

    def test_source_content_with_inline_module_import_external(self):
        self.run_servoshell(url=f"{self.base_urls[0]}/sources_content_with_inline_module_import_external/test.html")
        path = "sources_content_with_inline_module_import_external/test.html"
        expected_content = open(self.get_test_path(path)).read()
        self.assert_source_content(Source("inlineScript", f"{self.base_urls[0]}/{path}"), expected_content)

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
        self.run_servoshell(url=f"{self.base_urls[0]}/sources_content_with_responsexml/test.html")
        expected_content = open(self.get_test_path("sources_content_with_responsexml/test.html")).read()
        self.assert_source_content(
            Source("inlineScript", f"{self.base_urls[0]}/sources_content_with_responsexml/test.html"), expected_content
        )

    def test_source_breakable_lines_and_positions(self):
        self.run_servoshell(url=f"{self.base_urls[0]}/sources_breakable_lines_and_positions/test.html")
        self.assert_source_breakable_lines_and_positions(
            Source("inlineScript", f"{self.base_urls[0]}/sources_breakable_lines_and_positions/test.html"),
            [4, 5, 6, 7],
            {
                "4": [4, 12, 20, 28],
                "5": [15, 23, 31, 39],  # includes 3 surrogate pairs
                "6": [15, 23, 31, 39],  # includes 1 surrogate pair
                "7": [0],
            },
        )

    def test_source_breakable_lines_and_positions_with_functions(self):
        self.run_servoshell(url=f"{self.base_urls[0]}/sources_breakable_lines_and_positions/test_with_functions.html")
        self.assert_source_breakable_lines_and_positions(
            Source(
                "inlineScript", f"{self.base_urls[0]}/sources_breakable_lines_and_positions/test_with_functions.html"
            ),
            [5, 6, 7, 8, 9, 10],
            {
                "5": [8, 18],
                "6": [12],
                "7": [8],
                "8": [4],
                "9": [4],
                "10": [0],
            },
        )

    def test_console_log_object_with_object_preview(self):
        self.run_servoshell(url=f"{self.base_urls[0]}/console/log_object.html")
        with Devtools.connect() as devtools:
            devtools.watcher.watch_resources([Resources.CONSOLE_MESSAGE])

            console = WebConsoleActor(devtools.client, devtools.targets[0]["consoleActor"])
            evaluation_result = Future()

            async def on_resource_available(data: dict):
                for resource in data["array"]:
                    if resource[0] != "console-message":
                        continue
                    evaluation_result.set_result(resource[1][0])
                    return

            # Listen for console messages
            devtools.client.add_event_listener(
                devtools.watcher.actor_id, Events.Watcher.RESOURCES_AVAILABLE_ARRAY, on_resource_available
            )

            devtools.client.add_event_listener(
                devtools.targets[0]["actor"], Events.Watcher.RESOURCES_AVAILABLE_ARRAY, on_resource_available
            )

            # Run the console.log statements
            console.evaluate_js_async("log_object();")
            result = evaluation_result.result(1)["arguments"][0]

            # Run assertions on the result
            self.assertEquals(result["ownPropertyLength"], 3)

            preview = result["preview"]
            self.assertEquals(preview["kind"], "Object")
            self.assertEquals(preview["ownPropertiesLength"], 3)

            def assert_property_descriptor_equals(actual_descriptor, expected_descriptor):
                for key, value in expected_descriptor.items():
                    self.assertEquals(
                        actual_descriptor[key],
                        value,
                        f"Incorrect value for {key}, expected {value} got {actual_descriptor[key]}",
                    )

            assert_property_descriptor_equals(
                preview["ownProperties"]["foo"],
                {"configurable": True, "enumerable": True, "value": 1, "writable": True},
            )
            assert_property_descriptor_equals(
                preview["ownProperties"]["bar"],
                {"configurable": True, "enumerable": False, "value": "servo", "writable": True},
            )
            assert_property_descriptor_equals(
                preview["ownProperties"]["baz"],
                # TODO: The boolean value here should not be a string! That's a bug in our
                # devtools implementation.
                {"configurable": False, "enumerable": True, "value": "true", "writable": True},
            )

    # Sets `base_url` and `web_server` and `web_server_thread`.
    @classmethod
    def setUpClass(cls):
        assert cls.base_urls is None and cls.web_servers is None and cls.web_server_threads is None
        test_dir = os.path.join(DevtoolsTests.script_path, "devtools_tests")
        num_servers = 2
        base_urls = [Future() for i in range(num_servers)]
        cls.web_servers = [None for i in range(num_servers)]
        cls.web_server_threads = [None for i in range(num_servers)]

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
            cls.web_servers[index] = web_server
            web_server.serve_forever()

        # Start a web server for the test.
        for index in range(num_servers):
            thread = Thread(target=server_thread, args=[index])
            cls.web_server_threads[index] = thread
            thread.start()
        cls.base_urls = [base_url.result(1) for base_url in base_urls]

    # Sets `servoshell`.
    def run_servoshell(self, *, url):
        # Change this setting if you want to debug Servo.
        os.environ["RUST_LOG"] = "error,devtools=warn"

        # Run servoshell.
        self.servoshell = subprocess.Popen([f"{DevtoolsTests.servo_binary}", "--headless", "--devtools=6080", url])

        sleep_per_try = 1 / 8  # seconds
        remaining_tries = 5 / sleep_per_try  # 5 seconds
        while True:
            print(".", end="", file=sys.stderr)
            stream = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
            try:
                stream.connect(("127.0.0.1", 6080))
                stream.recv(4096)  # FIXME: without this, geckordp RDPClient.connect() may fail
                stream.shutdown(socket.SHUT_RDWR)
                print("+", end="", file=sys.stderr)
                break
            except Exception:
                time.sleep(sleep_per_try)
                self.assertGreater(remaining_tries, 0)
                remaining_tries -= 1
                continue

    def tearDown(self):
        # Terminate servoshell, but do not stop the web servers.
        if self.servoshell is not None:
            self.servoshell.terminate()
            try:
                self.servoshell.wait(timeout=3)
            except subprocess.TimeoutExpired:
                print("Warning: servoshell did not terminate", file=sys.stderr)
                self.servoshell.kill()
            self.servoshell = None

    @classmethod
    def tearDownClass(cls):
        # Stop the web servers.
        if cls.web_servers is not None:
            for web_server in cls.web_servers:
                web_server.shutdown()
                web_server.server_close()
            cls.web_servers = None
        if cls.web_server_threads is not None:
            for web_server_thread in cls.web_server_threads:
                web_server_thread.join()
            cls.web_server_threads = None
        if cls.base_urls is not None:
            cls.base_urls = None

    def assert_sources_list(
        self, expected_sources_by_target: Counter[FrozenMultiset[Source]], *, devtools: Optional[Devtools] = None
    ):
        expected_targets = len(expected_sources_by_target)
        if devtools is None:
            devtools = Devtools.connect(expected_targets=expected_targets)
        with devtools:
            done = Future()
            actual_sources_by_target: Counter[FrozenMultiset[Source]] = Counter()

            def on_source_resource(data):
                for [resource_type, sources] in data["array"]:
                    try:
                        self.assertEqual(resource_type, "source")
                        source_urls = frozen_multiset(
                            [Source(source["introductionType"], source["url"]) for source in sources]
                        )
                        self.assertFalse(source_urls in actual_sources_by_target)  # See NOTE above
                        actual_sources_by_target.update([source_urls])
                        if len(actual_sources_by_target) == expected_targets:
                            done.set_result(None)
                    except Exception as e:
                        # Raising here does nothing, for some reason.
                        # Send the exception back so it can be raised.
                        done.set_result(e)

            for target in devtools.targets:
                devtools.client.add_event_listener(
                    target["actor"],
                    Events.Watcher.RESOURCES_AVAILABLE_ARRAY,
                    on_source_resource,
                )
            devtools.watcher.watch_resources([Resources.SOURCE])

            result: Optional[Exception] = done.result(1)
            if result:
                raise result
            self.assertEqual(actual_sources_by_target, expected_sources_by_target)

    def assert_source_content(
        self, expected_source: Source, expected_content: str, *, devtools: Optional[Devtools] = None
    ):
        if devtools is None:
            devtools = Devtools.connect()
        with devtools:
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

            for target in devtools.targets:
                devtools.client.add_event_listener(
                    target["actor"],
                    Events.Watcher.RESOURCES_AVAILABLE_ARRAY,
                    on_source_resource,
                )
            devtools.watcher.watch_resources([Resources.SOURCE])

            result: Optional[Exception] = done.result(1)
            if result:
                raise result

            # We found at least one source with the given url.
            self.assertIn(expected_source, source_actors)
            source_actor = source_actors[expected_source]

            response = devtools.client.send_receive({"to": source_actor, "type": "source"})

            self.assertEqual(response["source"], expected_content)

    def assert_source_breakable_lines_and_positions(
        self,
        expected_source: Source,
        expected_breakable_lines: list[int],
        expected_positions: dict[int, list[int]],
        *,
        devtools: Optional[Devtools] = None,
    ):
        if devtools is None:
            devtools = Devtools.connect()
        with devtools:
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

            for target in devtools.targets:
                devtools.client.add_event_listener(
                    target["actor"],
                    Events.Watcher.RESOURCES_AVAILABLE_ARRAY,
                    on_source_resource,
                )
            devtools.watcher.watch_resources([Resources.SOURCE])

            result: Optional[Exception] = done.result(1)
            if result:
                raise result

            # We found at least one source with the given url.
            self.assertIn(expected_source, source_actors)
            source_actor = source_actors[expected_source]

            response = devtools.client.send_receive({"to": source_actor, "type": "getBreakableLines"})
            self.assertEqual(response["lines"], expected_breakable_lines)

            response = devtools.client.send_receive({"to": source_actor, "type": "getBreakpointPositionsCompressed"})
            self.assertEqual(response["positions"], expected_positions)

    def get_test_path(self, path: str) -> str:
        return os.path.join(DevtoolsTests.script_path, os.path.join("devtools_tests", path))


def run_tests(script_path, servo_binary: str, test_names: list[str]):
    DevtoolsTests.script_path = script_path
    DevtoolsTests.servo_binary = servo_binary
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
    print(f"Running {suite.countTestCases()} tests:", file=sys.stderr)
    for test in suite:
        print(f"- {test}", file=sys.stderr)
    print(file=sys.stderr)
    return unittest.TextTestRunner(verbosity=verbosity).run(suite).wasSuccessful()
