# Copyright 2013 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

from __future__ import annotations

import os.path
import time
from collections import Counter
from concurrent.futures import Future

import pytest
from geckordp.actors.events import Events
from geckordp.actors.web_console import WebConsoleActor

from .utils import (
    Devtools,
    Source,
    assert_source_breakable_lines_and_positions,
    assert_source_content,
    assert_sources_list,
    attach_thread,
    frozen_multiset,
    resume_and_wait,
    set_breakpoint,
    step,
    wait_and_assert_no_pause,
    wait_for_pause,
    wait_for_source,
)


class TestDebuggerTab:
    def test_breakpoint_pause(self, run_servoshell, web_server_urls):
        run_servoshell(url=f"{web_server_urls[0]}/debugger/loop.html")
        with Devtools.connect() as devtools:
            thread_actor = attach_thread(devtools)
            source_actor = wait_for_source(devtools, "debugger/loop.html")

            # Get valid breakpoint position
            positions = devtools.client.send_receive(
                {"to": source_actor, "type": "getBreakpointPositionsCompressed"}
            ).get("positions", {})
            line_str = min(positions.keys(), key=int)
            line, column = int(line_str), positions[line_str][0]

            def trigger():
                set_breakpoint(devtools, f"{web_server_urls[0]}/debugger/loop.html", line, column)

            paused_data = wait_for_pause(devtools.client, thread_actor, trigger)
            assert paused_data.get("type") == "paused"
            assert paused_data.get("why", {}).get("type") == "breakpoint"

    def test_breakpoint_at_invalid_entry_point_does_not_crash(self, run_servoshell, web_server_urls):
        run_servoshell(url=f"{web_server_urls[0]}/debugger/loop.html")
        with Devtools.connect() as devtools:
            breakpoint_list = devtools.watcher.get_breakpoint_list_actor()
            response = devtools.client.send_receive(
                {
                    "to": breakpoint_list["breakpointList"]["actor"],
                    "type": "setBreakpoint",
                    "location": {
                        "sourceUrl": f"{web_server_urls[0]}/debugger/loop.html",
                        "line": 1,
                        "column": 0,
                    },
                }
            )
            assert "from" in response

    def test_console_eval_does_not_pause_again_while_already_paused(self, run_servoshell, web_server_urls):
        run_servoshell(url=f"{web_server_urls[0]}/debugger/stepping.html")
        with Devtools.connect() as devtools:
            thread_actor = attach_thread(devtools)
            console_actor = devtools.targets[0]["consoleActor"]

            # Stop at the breakpoint in end()
            def trigger():
                set_breakpoint(devtools, f"{web_server_urls[0]}/debugger/stepping.html", 5, 16)

            paused_data = wait_for_pause(devtools.client, thread_actor, trigger)

            eval_result = Future()
            nested_pause = Future()

            devtools.client.add_event_listener(
                console_actor, Events.WebConsole.EVALUATION_RESULT, eval_result.set_result
            )
            devtools.client.add_event_listener(thread_actor, "paused", nested_pause.set_result)
            devtools.client.send_receive(
                {
                    "to": console_actor,
                    "type": "evaluateJSAsync",
                    "text": "end()",
                    "frameActor": paused_data["frame"]["actor"],
                }
            )

            # The console evaluation should not pause again
            time.sleep(0.5)
            assert not nested_pause.done()
            eval_result.result(2)

            # Clean up by resuming from the original pause
            devtools.client.send_receive({"to": thread_actor, "type": "resume"})

    def test_frame_scoped_eval(self, run_servoshell, web_server_urls):
        run_servoshell(url=f"{web_server_urls[0]}/debugger/frame_scoped.html")
        with Devtools.connect() as devtools:
            thread_actor = attach_thread(devtools)
            console_actor = devtools.targets[0]["consoleActor"]

            def trigger():
                devtools.client.send_receive({"to": thread_actor, "type": "interrupt", "when": "onNext"})

            paused_data = wait_for_pause(devtools.client, thread_actor, trigger)
            frame_actor = paused_data.get("frame", {}).get("actor")
            assert frame_actor is not None

            eval_future = Future()

            def on_eval_result(data):
                eval_future.set_result(data)

            devtools.client.add_event_listener(console_actor, Events.WebConsole.EVALUATION_RESULT, on_eval_result)
            devtools.client.send_receive(
                {
                    "to": console_actor,
                    "type": "evaluateJSAsync",
                    "text": "i",
                    "frameActor": frame_actor,
                }
            )

            eval_result = eval_future.result(2)
            assert not eval_result.get("hasException", True)
            assert eval_result.get("result") == 42

    def test_manual_pause(self, run_servoshell, web_server_urls):
        run_servoshell(url=f"{web_server_urls[0]}/debugger/loop.html")
        with Devtools.connect() as devtools:
            thread_actor = attach_thread(devtools)

            def trigger():
                devtools.client.send_receive({"to": thread_actor, "type": "interrupt", "when": "onNext"})

            paused_data = wait_for_pause(devtools.client, thread_actor, trigger)
            assert paused_data.get("type") == "paused"
            why = paused_data.get("why", {})
            assert why.get("type") == "interrupted"
            assert why.get("onNext") is True

    def test_stepping_hooks(self, run_servoshell, web_server_urls):
        run_servoshell(url=f"{web_server_urls[0]}/debugger/stepping.html")
        with Devtools.connect() as devtools:
            thread_actor = attach_thread(devtools)
            source_actor = wait_for_source(devtools, "debugger/stepping.html")

            # Get the breakpoint positions and find the line with `end()` call(should be line 10)
            positions = devtools.client.send_receive(
                {"to": source_actor, "type": "getBreakpointPositionsCompressed"}
            ).get("positions", {})

            # Line 10 - should be the `end()` call
            assert "10" in positions
            line, column = 10, positions["10"][0]

            # Set breakpoint at the end() call
            def trigger():
                set_breakpoint(devtools, f"{web_server_urls[0]}/debugger/stepping.html", line, column)

            # Pause and breakpoint hit, this is necessary for stepping hooks
            paused_data = wait_for_pause(devtools.client, thread_actor, trigger)
            assert paused_data.get("type") == "paused"
            assert paused_data.get("why", {}).get("type") == "breakpoint"

            # Did we pause at line 10?
            assert paused_data.get("frame", {}).get("where", {}).get("line") == 10

            # Step over! This should execute end() and pause at line 11
            step_data = step(devtools.client, thread_actor, "next")
            assert step_data.get("type") == "paused"
            assert step_data.get("why", {}).get("type") == "resumeLimit"
            assert step_data.get("frame", {}).get("where", {}).get("line") == 11

            # Step over
            step_data = step(devtools.client, thread_actor, "next")
            assert step_data.get("frame", {}).get("where", {}).get("line") == 12

            # Step over to line 13
            step_data = step(devtools.client, thread_actor, "next")
            assert step_data.get("frame", {}).get("where", {}).get("line") == 13

            # We should let the loop continue and hit breakpoint again at line 10
            paused_data = resume_and_wait(devtools.client, thread_actor)
            assert paused_data.get("why", {}).get("type") == "breakpoint"
            assert paused_data.get("frame", {}).get("where", {}).get("line") == 10

            # STEP IN to end() function
            step_data = step(devtools.client, thread_actor, "step")
            assert step_data.get("why", {}).get("type") == "resumeLimit"
            assert step_data.get("frame", {}).get("where", {}).get("line") == 4
            assert step_data.get("frame", {}).get("displayName") == "end"

            # Step over inside end() to line 5
            step_data = step(devtools.client, thread_actor, "next")
            assert step_data.get("frame", {}).get("where", {}).get("line") == 5

            # Step over inside end() to line 6
            step_data = step(devtools.client, thread_actor, "next")
            assert step_data.get("frame", {}).get("where", {}).get("line") == 6

            # Step out of end() back to loop() at line 11
            step_data = step(devtools.client, thread_actor, "finish")
            assert step_data.get("why", {}).get("type") == "resumeLimit"
            assert step_data.get("frame", {}).get("where", {}).get("line") == 11
            assert step_data.get("frame", {}).get("displayName") == "loop"

    # Sources list
    # Classic script vs module script:
    # - <https://html.spec.whatwg.org/multipage/#classic-script>
    # - <https://html.spec.whatwg.org/multipage/#module-script>
    # Worker scripts can be classic or module:
    # - <https://html.spec.whatwg.org/multipage/#fetch-a-classic-worker-script>
    # - <https://html.spec.whatwg.org/multipage/#fetch-a-module-worker-script-tree>
    # Non-worker(?) script sources can be inline, external, or blob.
    # Worker script sources can be external or blob.

    def test_sources_list(self, run_servoshell, web_server_urls):
        run_servoshell(url=f"{web_server_urls[0]}/sources/test.html")
        assert_sources_list(
            Counter(
                [
                    frozen_multiset(
                        [
                            Source("srcScript", f"{web_server_urls[0]}/sources/classic.js"),
                            Source("inlineScript", f"{web_server_urls[0]}/sources/test.html"),
                            Source("inlineScript", f"{web_server_urls[0]}/sources/test.html"),
                            Source("srcScript", f"{web_server_urls[1]}/sources/classic.js"),
                            Source("importedModule", f"{web_server_urls[0]}/sources/module.js"),
                        ]
                    ),
                    frozen_multiset([Source("Worker", f"{web_server_urls[0]}/sources/classic_worker.js")]),
                ]
            ),
        )

    def test_sources_list_with_data_no_scripts(self, run_servoshell):
        run_servoshell(url="data:text/html,")
        assert_sources_list(Counter([frozen_multiset()]))

    # Sources list for `introductionType` = `inlineScript` and `srcScript`

    def test_sources_list_with_data_empty_inline_classic_script(self, run_servoshell):
        run_servoshell(url="data:text/html,<script></script>")
        assert_sources_list(Counter([frozen_multiset()]))

    def test_sources_list_with_data_inline_classic_script(self, run_servoshell):
        run_servoshell(url="data:text/html,<script>;</script>")
        assert_sources_list(Counter([frozen_multiset([Source("inlineScript", "data:text/html,<script>;</script>")])]))

    def test_sources_list_with_data_external_classic_script(self, run_servoshell, web_server_urls):
        run_servoshell(url=f'data:text/html,<script src="{web_server_urls[0]}/sources/classic.js"></script>')
        assert_sources_list(
            Counter([frozen_multiset([Source("srcScript", f"{web_server_urls[0]}/sources/classic.js")])])
        )

    def test_sources_list_with_data_empty_inline_module_script(self, run_servoshell):
        run_servoshell(url="data:text/html,<script type=module></script>")
        assert_sources_list(Counter([frozen_multiset()]))

    def test_sources_list_with_data_inline_module_script(self, run_servoshell):
        run_servoshell(url="data:text/html,<script type=module>;</script>")
        assert_sources_list(
            Counter([frozen_multiset([Source("inlineScript", "data:text/html,<script type=module>;</script>")])])
        )

    def test_sources_list_with_data_external_module_script(self, run_servoshell, web_server_urls):
        run_servoshell(url=f"{web_server_urls[0]}/sources/test_sources_list_with_data_external_module_script.html")
        assert_sources_list(
            Counter([frozen_multiset([Source("srcScript", f"{web_server_urls[0]}/sources/module.js")])])
        )

    # Sources list for `introductionType` = `importedModule`

    def test_sources_list_with_static_import_module(self, run_servoshell, web_server_urls):
        run_servoshell(url=f"{web_server_urls[0]}/sources/test_sources_list_with_static_import_module.html")
        assert_sources_list(
            Counter(
                [
                    frozen_multiset(
                        [
                            Source(
                                "inlineScript",
                                f"{web_server_urls[0]}/sources/test_sources_list_with_static_import_module.html",
                            ),
                            Source("importedModule", f"{web_server_urls[0]}/sources/module.js"),
                        ]
                    )
                ]
            ),
        )

    def test_sources_list_with_dynamic_import_module(self, run_servoshell, web_server_urls):
        run_servoshell(url=f"{web_server_urls[0]}/sources/test_sources_list_with_dynamic_import_module.html")
        assert_sources_list(
            Counter(
                [
                    frozen_multiset(
                        [
                            Source(
                                "inlineScript",
                                f"{web_server_urls[0]}/sources/test_sources_list_with_dynamic_import_module.html",
                            ),
                            Source("importedModule", f"{web_server_urls[0]}/sources/module.js"),
                        ]
                    )
                ]
            ),
        )

    # Sources list for `introductionType` = `Worker`

    def test_sources_list_with_classic_worker(self, run_servoshell, web_server_urls):
        run_servoshell(url=f"{web_server_urls[0]}/sources/test_sources_list_with_classic_worker.html")
        assert_sources_list(
            Counter(
                [
                    frozen_multiset(
                        [
                            Source(
                                "inlineScript",
                                f"{web_server_urls[0]}/sources/test_sources_list_with_classic_worker.html",
                            ),
                        ]
                    ),
                    frozen_multiset(
                        [
                            Source("Worker", f"{web_server_urls[0]}/sources/classic_worker.js"),
                        ]
                    ),
                ]
            ),
        )

    def test_sources_list_with_module_worker(self, run_servoshell, web_server_urls):
        run_servoshell(url=f"{web_server_urls[0]}/sources/test_sources_list_with_module_worker.html")
        assert_sources_list(
            Counter(
                [
                    frozen_multiset(
                        [
                            Source(
                                "inlineScript",
                                f"{web_server_urls[0]}/sources/test_sources_list_with_module_worker.html",
                            ),
                        ]
                    ),
                    frozen_multiset(
                        [
                            Source("Worker", f"{web_server_urls[0]}/sources/module_worker.js"),
                        ]
                    ),
                ]
            ),
        )

    # Sources list for `introductionType` set to values that require `displayURL` (`//# sourceURL`)

    def test_sources_list_with_injected_script_write_and_display_url(self, run_servoshell):
        run_servoshell(
            url='data:text/html,<script>document.write("<script>//%23 sourceURL=http://test</scr"+"ipt>")</script>'
        )
        assert_sources_list(
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

    def test_sources_list_with_injected_script_write_but_no_display_url(self, run_servoshell):
        run_servoshell(url='data:text/html,<script>document.write("<script>1</scr"+"ipt>")</script>')
        assert_sources_list(
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

    def test_sources_list_with_injected_script_append_and_display_url(self, run_servoshell):
        script = 's=document.createElement("script");s.append("//%23 sourceURL=http://test");document.body.append(s)'
        run_servoshell(url=f"data:text/html,<body><script>{script}</script>")
        assert_sources_list(
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

    def test_sources_list_with_injected_script_append_but_no_display_url(self, run_servoshell):
        script = 's=document.createElement("script");s.append("1");document.body.append(s)'
        run_servoshell(url=f"data:text/html,<body><script>{script}</script>")
        assert_sources_list(
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

    def test_sources_list_with_eval_and_display_url(self, run_servoshell):
        run_servoshell(url='data:text/html,<script>eval("//%23 sourceURL=http://test")</script>')
        assert_sources_list(
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

    def test_sources_list_with_eval_but_no_display_url(self, run_servoshell):
        run_servoshell(url='data:text/html,<script>eval("1")</script>')
        assert_sources_list(
            Counter([frozen_multiset([Source("inlineScript", 'data:text/html,<script>eval("1")</script>')])])
        )

    def test_sources_list_with_debugger_eval_and_display_url(self, run_servoshell):
        run_servoshell(url="data:text/html,")
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
            assert_sources_list(
                Counter([frozen_multiset([Source("debugger eval", "http://test/")])]), devtools=devtools
            )

    def test_sources_list_with_debugger_eval_but_no_display_url(self, run_servoshell):
        run_servoshell(url="data:text/html,")
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
            assert_sources_list(Counter([frozen_multiset([])]), devtools=devtools)

    def test_sources_list_with_function_and_display_url(self, run_servoshell):
        run_servoshell(url='data:text/html,<script>new Function("//%23 sourceURL=http://test")</script>')
        assert_sources_list(
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

    def test_sources_list_with_function_but_no_display_url(self, run_servoshell):
        run_servoshell(url='data:text/html,<script>new Function("1")</script>')
        assert_sources_list(
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

    def test_sources_list_with_javascript_url_and_display_url(self, run_servoshell):
        # “1” prefix is a workaround for <https://github.com/servo/servo/issues/38547>
        run_servoshell(
            url='data:text/html,<a href="javascript:1//%23 sourceURL=http://test"></a><script>document.querySelector("a").click()</script>'
        )
        assert_sources_list(
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

    def test_sources_list_with_javascript_url_but_no_display_url(self, run_servoshell):
        run_servoshell(url='data:text/html,<a href="javascript:1"></a>')
        assert_sources_list(Counter([frozen_multiset([])]))

    @pytest.mark.xfail(strict=False)
    def test_sources_list_with_event_handler_and_display_url(self, run_servoshell):
        run_servoshell(url='data:text/html,<a onclick="//%23 sourceURL=http://test"></a>')
        assert_sources_list(
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

    def test_sources_list_with_event_handler_but_no_display_url(self, run_servoshell):
        run_servoshell(url='data:text/html,<a onclick="1"></a>')
        assert_sources_list(Counter([frozen_multiset([])]))

    @pytest.mark.xfail(strict=False)
    def test_sources_list_with_dom_timer_and_display_url(self, run_servoshell):
        run_servoshell(url='data:text/html,<script>setTimeout("//%23 sourceURL=http://test",0)</script>')
        assert_sources_list(
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

    @pytest.mark.xfail(strict=False)
    def test_sources_list_with_dom_timer_but_no_display_url(self, run_servoshell):
        run_servoshell(url='data:text/html,<script>setTimeout("1",0)</script>')
        assert_sources_list(Counter([frozen_multiset([])]))

    # Sources list for scripts with `displayURL` (`//# sourceURL`), despite not being required by `introductionType`

    def test_sources_list_with_inline_script_and_display_url(self, run_servoshell):
        run_servoshell(url="data:text/html,<script>//%23 sourceURL=http://test</script>")
        assert_sources_list(
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
    def test_sources_list_with_inline_script_but_invalid_display_url(self, run_servoshell):
        run_servoshell(url="data:text/html,<script>//%23 sourceURL=test</script>")
        assert_sources_list(
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

    def test_sources_list_with_inline_script_but_no_display_url(self, run_servoshell):
        run_servoshell(url="data:text/html,<script>1</script>")
        assert_sources_list(
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

    @pytest.mark.xfail(strict=False)
    def test_sources_list_with_iframe_srcdoc_and_display_url(self, run_servoshell):
        run_servoshell(url='data:text/html,<iframe srcdoc="<script>//%23 sourceURL=http://test</script>">')
        assert_sources_list(
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

    @pytest.mark.xfail(strict=False)
    def test_sources_list_with_iframe_srcdoc_but_no_display_url(self, run_servoshell):
        run_servoshell(url='data:text/html,<iframe srcdoc="<script>1</script>">')
        assert_sources_list(
            Counter(
                [
                    frozen_multiset(
                        [
                            # FIXME: it's not really gonna be 0
                            Source("inlineScript", "about:srcdoc#0"),
                        ]
                    )
                ]
            )
        )

    @pytest.mark.xfail(strict=False)
    def test_sources_list_with_iframe_srcdoc_multiple_inline_scripts(self, run_servoshell):
        run_servoshell(
            url='data:text/html,<iframe srcdoc="<script>//%23 sourceURL=http://test</script><script>2</script>">'
        )
        assert_sources_list(
            Counter(
                [
                    frozen_multiset(
                        [
                            Source("inlineScript", "http://test/"),
                            # FIXME: it's not really gonna be 0
                            Source("inlineScript", "about:srcdoc#0"),
                        ]
                    )
                ]
            )
        )

    # Source contents

    def test_source_content_inline_script(self, run_servoshell):
        script_tag = "<script>console.log('Hello, world!')</script>"
        run_servoshell(url=f"data:text/html,{script_tag}")
        assert_source_content(Source("inlineScript", f"data:text/html,{script_tag}"), script_tag)

    def test_source_content_external_script(self, run_servoshell, web_server_urls):
        run_servoshell(url=f'data:text/html,<script src="{web_server_urls[0]}/sources/classic.js"></script>')
        expected_content = 'console.log("external classic");\n'
        assert_source_content(Source("srcScript", f"{web_server_urls[0]}/sources/classic.js"), expected_content)

    def test_source_content_html_file(self, run_servoshell, web_server_urls, script_path):
        run_servoshell(url=f"{web_server_urls[0]}/sources/test.html")
        expected_content = open(os.path.join(script_path, "devtools_tests/sources/test.html")).read()
        assert_source_content(Source("inlineScript", f"{web_server_urls[0]}/sources/test.html"), expected_content)

    def test_source_content_with_inline_module_import_external(self, run_servoshell, web_server_urls, script_path):
        run_servoshell(url=f"{web_server_urls[0]}/sources_content_with_inline_module_import_external/test.html")
        expected_content = open(
            os.path.join(script_path, "devtools_tests", "sources_content_with_inline_module_import_external/test.html")
        ).read()
        assert_source_content(
            Source(
                "inlineScript", f"{web_server_urls[0]}/sources_content_with_inline_module_import_external/test.html"
            ),
            expected_content,
        )

    # Test case that uses innerHTML and would actually need the HTML parser
    # (innerHTML has a fast path for values that don't contain b'&' | b'\0' | b'<' | b'\r')
    def test_source_content_inline_script_with_inner_html(self, run_servoshell):
        script_tag = '<div id="el"></div><script>el.innerHTML="<p>test"</script>'
        run_servoshell(url=f"data:text/html,{script_tag}")
        assert_source_content(Source("inlineScript", f"data:text/html,{script_tag}"), script_tag)

    # Test case that uses outerHTML and would actually need the HTML parser
    # (innerHTML has a fast path for values that don't contain b'&' | b'\0' | b'<' | b'\r')
    def test_source_content_inline_script_with_outer_html(self, run_servoshell):
        script_tag = '<div id="el"></div><script>el.outerHTML="<p>test"</script>'
        run_servoshell(url=f"data:text/html,{script_tag}")
        assert_source_content(Source("inlineScript", f"data:text/html,{script_tag}"), script_tag)

    # Test case that uses DOMParser and would actually need the HTML parser
    # (innerHTML has a fast path for values that don't contain b'&' | b'\0' | b'<' | b'\r')
    def test_source_content_inline_script_with_domparser(self, run_servoshell):
        script_tag = '<script>(new DOMParser).parseFromString("<p>test","text/html")</script>'
        run_servoshell(url=f"data:text/html,{script_tag}")
        assert_source_content(Source("inlineScript", f"data:text/html,{script_tag}"), script_tag)

    # Test case that uses XMLHttpRequest#responseXML and would actually need the HTML parser
    # (innerHTML has a fast path for values that don't contain b'&' | b'\0' | b'<' | b'\r')
    def test_source_content_inline_script_with_responsexml(self, run_servoshell, web_server_urls, script_path):
        run_servoshell(url=f"{web_server_urls[0]}/sources_content_with_responsexml/test.html")
        expected_content = open(
            os.path.join(script_path, "devtools_tests/sources_content_with_responsexml/test.html")
        ).read()
        assert_source_content(
            Source("inlineScript", f"{web_server_urls[0]}/sources_content_with_responsexml/test.html"),
            expected_content,
        )

    def test_source_breakable_lines_and_positions(self, run_servoshell, web_server_urls):
        run_servoshell(url=f"{web_server_urls[0]}/sources_breakable_lines_and_positions/test.html")
        assert_source_breakable_lines_and_positions(
            Source("inlineScript", f"{web_server_urls[0]}/sources_breakable_lines_and_positions/test.html"),
            [4, 5, 6, 7],
            {
                "4": [4, 12, 20, 28],
                "5": [15, 23, 31, 39],  # includes 3 surrogate pairs
                "6": [15, 23, 31, 39],  # includes 1 surrogate pair
                "7": [0],
            },
        )

    def test_source_breakable_lines_and_positions_with_functions(self, run_servoshell, web_server_urls):
        run_servoshell(url=f"{web_server_urls[0]}/sources_breakable_lines_and_positions/test_with_functions.html")
        assert_source_breakable_lines_and_positions(
            Source(
                "inlineScript",
                f"{web_server_urls[0]}/sources_breakable_lines_and_positions/test_with_functions.html",
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

    def test_blackboxing_prevents_breakpoint_pause(self, run_servoshell, web_server_urls):
        url = f"{web_server_urls[0]}/debugger/loop.html"
        run_servoshell(url=url)
        with Devtools.connect() as devtools:
            thread_actor = attach_thread(devtools)
            source_actor = wait_for_source(devtools, "debugger/loop.html")

            # Get valid breakpoint position
            positions = devtools.client.send_receive(
                {"to": source_actor, "type": "getBreakpointPositionsCompressed"}
            ).get("positions", {})
            line_str = min(positions.keys(), key=int)
            line, column = int(line_str), positions[line_str][0]

            # Blackbox the entire source
            blackboxing_actor = devtools.watcher.get_blackboxing_actor()["blackboxing"]["actor"]
            devtools.client.send_receive({"to": blackboxing_actor, "type": "blackbox", "range": [], "url": url})

            # Set a breakpoint and confirm that we will not pause
            def trigger():
                set_breakpoint(devtools, url, line, column)

            wait_and_assert_no_pause(devtools.client, thread_actor, trigger, duration=1)

    def test_blackboxing_prevents_breakpoint_pause_single_line(self, run_servoshell, web_server_urls):
        url = f"{web_server_urls[0]}/debugger/loop.html"
        run_servoshell(url=url)
        with Devtools.connect() as devtools:
            thread_actor = attach_thread(devtools)
            source_actor = wait_for_source(devtools, "debugger/loop.html")

            # Get valid breakpoint position
            positions = devtools.client.send_receive(
                {"to": source_actor, "type": "getBreakpointPositionsCompressed"}
            ).get("positions", {})
            line_str = min(positions.keys(), key=int)
            line, column = int(line_str), positions[line_str][0]

            # Blackbox line 4
            blackboxing_actor = devtools.watcher.get_blackboxing_actor()["blackboxing"]["actor"]
            devtools.client.send_receive(
                {
                    "to": blackboxing_actor,
                    "type": "blackbox",
                    "range": [{"start": {"line": 4, "column": 0}, "end": {"line": 4, "column": 15}}],
                    "url": url,
                }
            )

            # Set a breakpoint and confirm that we will not pause
            def trigger():
                set_breakpoint(devtools, url, line, column)

            wait_and_assert_no_pause(devtools.client, thread_actor, trigger, duration=1)

    def test_blackboxing_prevents_breakpoint_pause_multiline(self, run_servoshell, web_server_urls):
        url = f"{web_server_urls[0]}/debugger/loop.html"
        run_servoshell(url=url)
        with Devtools.connect() as devtools:
            thread_actor = attach_thread(devtools)
            source_actor = wait_for_source(devtools, "debugger/loop.html")

            # Get valid breakpoint position
            positions = devtools.client.send_receive(
                {"to": source_actor, "type": "getBreakpointPositionsCompressed"}
            ).get("positions", {})
            line_str = min(positions.keys(), key=int)
            line, column = int(line_str), positions[line_str][0]

            # Blackbox line 4
            blackboxing_actor = devtools.watcher.get_blackboxing_actor()["blackboxing"]["actor"]
            devtools.client.send_receive(
                {
                    "to": blackboxing_actor,
                    "type": "blackbox",
                    "range": [{"start": {"line": 4, "column": 8}, "end": {"line": 5, "column": 30}}],
                    "url": url,
                }
            )

            # Set a breakpoint and confirm that we will not pause
            def trigger():
                set_breakpoint(devtools, url, line, column)

            wait_and_assert_no_pause(devtools.client, thread_actor, trigger, duration=1)

    def test_unblackboxing_partial(self, run_servoshell, web_server_urls):
        url = f"{web_server_urls[0]}/debugger/loop.html"
        run_servoshell(url=url)
        with Devtools.connect() as devtools:
            thread_actor = attach_thread(devtools)
            source_actor = wait_for_source(devtools, "debugger/loop.html")

            # Get valid breakpoint position
            positions = devtools.client.send_receive(
                {"to": source_actor, "type": "getBreakpointPositionsCompressed"}
            ).get("positions", {})
            line_str = min(positions.keys(), key=int)
            line, column = int(line_str), positions[line_str][0]

            # Blackbox line 4
            blackboxing_actor = devtools.watcher.get_blackboxing_actor()["blackboxing"]["actor"]
            range = [{"start": {"line": 4, "column": 0}, "end": {"line": 5, "column": 15}}]
            devtools.client.send_receive({"to": blackboxing_actor, "type": "blackbox", "range": range, "url": url})

            # Then unblackbox it
            devtools.client.send_receive({"to": blackboxing_actor, "type": "unblackbox", "range": range, "url": url})

            # Set a breakpoint and confirm that we still pause
            def trigger():
                set_breakpoint(devtools, f"{web_server_urls[0]}/debugger/loop.html", line, column)

            wait_for_pause(devtools.client, thread_actor, trigger)

    def test_blackboxing_prevents_stepping(self, run_servoshell, web_server_urls):
        url = f"{web_server_urls[0]}/debugger/loop.html"
        run_servoshell(url=url)
        with Devtools.connect() as devtools:
            thread_actor = attach_thread(devtools)
            source_actor = wait_for_source(devtools, "debugger/loop.html")

            # Get valid breakpoint position
            positions = devtools.client.send_receive(
                {"to": source_actor, "type": "getBreakpointPositionsCompressed"}
            ).get("positions", {})
            print(positions)
            line1 = 4
            line2 = 5
            line1_col = min(positions[str(line1)])
            line2_col = min(positions[str(line2)])

            # Set breakpoint on line 4 and wait for it to be reached
            def trigger():
                set_breakpoint(devtools, f"{web_server_urls[0]}/debugger/loop.html", line1, line1_col)

            wait_for_pause(devtools.client, thread_actor, trigger)

            # Set breakpoint on line 5
            set_breakpoint(devtools, f"{web_server_urls[0]}/debugger/loop.html", line2, line2_col)

            # Blackbox line 5
            blackboxing_actor = devtools.watcher.get_blackboxing_actor()["blackboxing"]["actor"]
            devtools.client.send_receive(
                {
                    "to": blackboxing_actor,
                    "type": "blackbox",
                    "range": [{"start": {"line": 5, "column": 0}, "end": {"line": 5, "column": 30}}],
                    "url": url,
                }
            )

            # Step forward! We should skip over line 5 and end up on line 4 again
            step_data = step(devtools.client, thread_actor, "next")
            assert step_data.get("frame", {}).get("where", {}).get("line") == 4
