# Copyright 2013 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

import unittest

from .utils import DevtoolsTestCase


class ConsoleTests(DevtoolsTestCase):
    def test_console_log_object_with_object_preview(self):
        self.run_servoshell(url=f"{self.base_urls[0]}/console/log_object.html")

        result = self.evaluate_and_capture_console_log_output("log_object();")["arguments"][0]

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
                    f"Incorrect value for {key}, expected {value}, got {actual_descriptor[key]}",
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
            {"configurable": False, "enumerable": True, "value": True, "writable": True},
        )

    def test_console_log_booleans(self):
        script_tag = "<script>let log_booleans = () => console.log(true, false, !false, !true);</script>"
        self.run_servoshell(url=f"data:text/html,{script_tag}")

        result = self.evaluate_and_capture_console_log_output("log_booleans();")
        self.assertEquals(result["arguments"], [True, False, True, False])

    def test_console_log_numbers(self):
        script_tag = "<script>let log_numbers = () => console.log(1/0, -1/0, 0/0, -0, 1);</script>"
        self.run_servoshell(url=f"data:text/html,{script_tag}")

        result = self.evaluate_and_capture_console_log_output("log_numbers();")

        self.assertEquals(
            result["arguments"], [{"type": "Infinity"}, {"type": "-Infinity"}, {"type": "NaN"}, {"type": "-0"}, 1.0]
        )

    def test_console_log_array(self):
        script_tag = "<script>let log_array = () => console.log([1, 2, 3]);</script>"
        self.run_servoshell(url=f"data:text/html,{script_tag}")

        result = self.evaluate_and_capture_console_log_output("log_array();")
        object = result["arguments"][0]
        self.assertEquals(object["class"], "Array")
        preview = object["preview"]
        self.assertEquals(preview["kind"], "ArrayLike")
        self.assertEquals(preview["length"], 3)
        self.assertEquals(preview["items"], [1, 2, 3])

    def test_console_log_function(self):
        script_tag = "<script>function test_function() { }let log_function = () => console.log(test_function);</script>"
        self.run_servoshell(url=f"data:text/html,{script_tag}")

        result = self.evaluate_and_capture_console_log_output("log_function();")
        function = result["arguments"][0]
        self.assertEquals(function["class"], "Function")
        self.assertEquals(function["name"], "test_function")
        self.assertEquals(function["displayName"], "test_function")
        preview = function["preview"]
        self.assertEquals(preview["kind"], "Object")

    @unittest.expectedFailure
    def test_console_log_function_arguments(self):
        script_tag = (
            "<script>function test_arguments(a, b) { return a + b; }"
            "let log_arguments = () => console.log(test_arguments);"
            "</script>"
        )
        self.run_servoshell(url=f"data:text/html,{script_tag}")

        result = self.evaluate_and_capture_console_log_output("log_arguments();")
        self.assertEquals(result["arguments"][0]["parameterNames"], ["a", "b"])

    def test_console_log_sprintf_substitutions(self):
        script_tag = (
            "<script>let log_sprintf = () => "
            "console.log('String %s Int %d Int %i Float %f', 'string', 32, 46, Math.PI);"
            "</script>"
        )
        self.run_servoshell(url=f"data:text/html,{script_tag}")

        result = self.evaluate_and_capture_console_log_output("log_sprintf();")
        self.assertEquals(result["arguments"], ["String string Int 32 Int 46 Float 3.141592653589793"])

    def test_console_actor_can_handle_self_referential_objects(self):
        self.run_servoshell(url="data:text/html,")

        js = open(self.get_test_path("console/log_object_containing_itself.js")).read()
        self.evaluate_and_capture_console_log_output(js)

        # We don't run any assertions on the result because we don't implement these circular references
        # properly yet. The important part is that we didn't crash and didn't time out waiting for
        # a console notification (meaning we got *something*).

    def test_console_actor_log_window_object(self):
        self.run_servoshell(url="data:text/html,")

        self.evaluate_and_capture_console_log_output("console.log(window);")

        # We don't run any assertions on the result because we don't implement previews for the window object
        # yet. The important part is that we didn't crash and didn't time out waiting for
        # a console notification (meaning we got *something*).
