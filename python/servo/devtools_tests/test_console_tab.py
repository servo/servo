# Copyright 2013 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

from concurrent.futures import Future

import pytest
from geckordp.actors.events import Events
from geckordp.actors.resources import Resources
from geckordp.actors.web_console import WebConsoleActor

from .utils import Devtools, attach_thread, set_breakpoint, wait_for_pause, wait_for_source


def evaluate_and_capture_console_log_output(js: str, timeout: float = 1) -> dict:
    with Devtools.connect() as devtools:
        devtools.watcher.watch_resources([Resources.CONSOLE_MESSAGE])

        console = WebConsoleActor(devtools.client, devtools.targets[0]["consoleActor"])
        evaluation_result = Future()

        async def on_resource_available(data):
            for resource in data["array"]:
                if resource[0] != "console-message":
                    continue
                evaluation_result.set_result(resource[1][0])
                return

        devtools.client.add_event_listener(
            devtools.targets[0]["actor"], Events.Watcher.RESOURCES_AVAILABLE_ARRAY, on_resource_available
        )

        console.evaluate_js_async(js)
        return evaluation_result.result(timeout)


class TestConsoleTab:
    def test_console_log_object_with_object_preview(self, run_servoshell, web_server_urls):
        run_servoshell(url=f"{web_server_urls[0]}/console/log_object.html")

        result = evaluate_and_capture_console_log_output("log_object();")["arguments"][0]

        # Run assertions on the result
        assert result["ownPropertyLength"] == 3

        preview = result["preview"]
        assert preview["kind"] == "Object"
        assert preview["ownPropertiesLength"] == 3

        assert preview["ownProperties"]["foo"] == {
            "configurable": True,
            "enumerable": True,
            "isAccessor": False,
            "value": 1,
            "writable": True,
        }
        assert preview["ownProperties"]["bar"] == {
            "configurable": True,
            "enumerable": False,
            "isAccessor": False,
            "value": "servo",
            "writable": True,
        }
        assert preview["ownProperties"]["baz"] == {
            "configurable": False,
            "enumerable": True,
            "isAccessor": False,
            "value": True,
            "writable": True,
        }

    def test_console_log_booleans(self, run_servoshell):
        script_tag = "<script>let log_booleans = () => console.log(true, false, !false, !true);</script>"
        run_servoshell(url=f"data:text/html,{script_tag}")

        result = evaluate_and_capture_console_log_output("log_booleans();")
        assert result["arguments"] == [True, False, True, False]

    def test_console_log_numbers(self, run_servoshell):
        script_tag = "<script>let log_numbers = () => console.log(1/0, -1/0, 0/0, -0, 1);</script>"
        run_servoshell(url=f"data:text/html,{script_tag}")

        result = evaluate_and_capture_console_log_output("log_numbers();")

        assert result["arguments"] == [
            {"type": "Infinity"},
            {"type": "-Infinity"},
            {"type": "NaN"},
            {"type": "-0"},
            1.0,
        ]

    def test_console_log_array(self, run_servoshell):
        script_tag = "<script>let log_array = () => console.log([1, 2, 3]);</script>"
        run_servoshell(url=f"data:text/html,{script_tag}")

        result = evaluate_and_capture_console_log_output("log_array();")
        object = result["arguments"][0]
        assert object["class"] == "Array"
        preview = object["preview"]
        assert preview["kind"] == "ArrayLike"
        assert preview["length"] == 3
        assert preview["items"] == [1, 2, 3]

    def test_console_log_map(self, run_servoshell):
        script_tag = "<script>let log_map = () => console.log(new Map([['a', 1], ['b', true]]));</script>"
        run_servoshell(url=f"data:text/html,{script_tag}")

        result = evaluate_and_capture_console_log_output("log_map();")
        object = result["arguments"][0]
        assert object["class"] == "Map"
        preview = object["preview"]
        assert preview["kind"] == "MapLike"
        assert preview["size"] == 2
        assert preview["entries"] == [["a", 1], ["b", True]]

    def test_console_log_function(self, run_servoshell):
        script_tag = "<script>function test_function() { }let log_function = () => console.log(test_function);</script>"
        run_servoshell(url=f"data:text/html,{script_tag}")

        result = evaluate_and_capture_console_log_output("log_function();")
        function = result["arguments"][0]
        assert function["class"] == "Function"
        assert function["name"] == "test_function"
        assert function["displayName"] == "test_function"
        preview = function["preview"]
        assert preview["kind"] == "Object"

    @pytest.mark.xfail
    def test_console_log_function_arguments(self, run_servoshell):
        script_tag = (
            "<script>function test_arguments(a, b) { return a + b; }"
            "let log_arguments = () => console.log(test_arguments);"
            "</script>"
        )
        run_servoshell(url=f"data:text/html,{script_tag}")

        result = evaluate_and_capture_console_log_output("log_arguments();")
        assert result["arguments"][0]["parameterNames"] == ["a", "b"]

    def test_console_log_sprintf_substitutions(self, run_servoshell):
        script_tag = (
            "<script>let log_sprintf = () => "
            "console.log('String %s Int %d Int %i Float %f', 'string', 32, 46, Math.PI);"
            "</script>"
        )
        run_servoshell(url=f"data:text/html,{script_tag}")

        result = evaluate_and_capture_console_log_output("log_sprintf();")
        assert result["arguments"] == ["String string Int 32 Int 46 Float 3.141592653589793"]

    def test_console_actor_can_handle_self_referential_objects(self, run_servoshell, web_server_urls):
        run_servoshell(url="data:text/html,")

        evaluate_and_capture_console_log_output("x = {}; x.foo = x; console.log(x);")

        # We don't run any assertions on the result because we don't implement these circular references
        # properly yet. The important part is that we didn't crash and didn't time out waiting for
        # a console notification (meaning we got *something*).

    def test_console_actor_log_window_object(self, run_servoshell):
        run_servoshell(url="data:text/html,")

        evaluate_and_capture_console_log_output("console.log(window);")

        # We don't run any assertions on the result because we don't implement previews for the window object
        # yet. The important part is that we didn't crash and didn't time out waiting for
        # a console notification (meaning we got *something*).

    def test_console_throw_exception(self, run_servoshell):
        run_servoshell(url="data:text/html,")

        with Devtools.connect() as devtools:
            console = WebConsoleActor(devtools.client, devtools.targets[0]["consoleActor"])
            evaluation_result = Future()

            def on_evaluation(data):
                evaluation_result.set_result(data)

            devtools.client.add_event_listener(console.actor_id, Events.WebConsole.EVALUATION_RESULT, on_evaluation)

            console.evaluate_js_async("document.head.insertBefore(document.documentElement);")
            result = evaluation_result.result(1)

            assert not result["result"]
            assert result["exception"]
            assert "Not enough arguments" in result["exceptionMessage"]

    def test_global_autocomplete(self, run_servoshell):
        script_tag = "<script>console_test_value = 5;</script>"
        run_servoshell(url=f"data:text/html,{script_tag}")

        with Devtools.connect() as devtools:
            console = WebConsoleActor(devtools.client, devtools.targets[0]["consoleActor"])
            prompt = "con"
            result = console.autocomplete(text=prompt, frame_actor=None)

            # TODO: These are not supported yet
            # assert "console" in result["matches"]
            # assert "const" in result["matches"]
            # assert "continue" in result["matches"]

            assert "console_test_value" in result["matches"]
            assert result["matchProp"] == prompt

    def test_frame_autocomplete(self, run_servoshell, web_server_urls):
        run_servoshell(url=f"{web_server_urls[0]}/console/autocomplete_scoped.html")
        with Devtools.connect() as devtools:
            thread_actor = attach_thread(devtools)
            source_actor = wait_for_source(devtools, "console/autocomplete_scoped.html")

            # Get valid breakpoint position
            positions = devtools.client.send_receive(
                {"to": source_actor, "type": "getBreakpointPositionsCompressed"}
            ).get("positions", {})

            line_str = str(6)
            line, column = int(line_str), positions[line_str][0]

            def trigger():
                set_breakpoint(devtools, f"{web_server_urls[0]}/console/autocomplete_scoped.html", line, column)

            pause_data = wait_for_pause(devtools.client, thread_actor, trigger)
            frame_actor = pause_data["frame"]["actor"]

            prompt = "val"
            console = WebConsoleActor(devtools.client, devtools.targets[0]["consoleActor"])
            result = console.autocomplete(text=prompt, frame_actor=frame_actor)

            assert "valInner" in result["matches"]
            assert "valOuter" in result["matches"]
            assert result["matchProp"] == prompt
