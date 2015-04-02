# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this file,
# You can obtain one at http://mozilla.org/MPL/2.0/.

import unittest

from cStringIO import StringIO

from .. import parser

# There aren't many tests here because it turns out to be way more convenient to
# use test_serializer for the majority of cases


class TestExpression(unittest.TestCase):
    def setUp(self):
        self.parser = parser.Parser()

    def parse(self, input_str):
        return self.parser.parse(StringIO(input_str))

    def compare(self, input_text, expected):
        actual = self.parse(input_text)
        self.match(expected, actual)

    def match(self, expected_node, actual_node):
        self.assertEquals(expected_node[0], actual_node.__class__.__name__)
        self.assertEquals(expected_node[1], actual_node.data)
        self.assertEquals(len(expected_node[2]), len(actual_node.children))
        for expected_child, actual_child in zip(expected_node[2], actual_node.children):
            self.match(expected_child, actual_child)

    def test_expr_0(self):
        self.compare(
            """
key:
  if x == 1 : value""",
            ["DataNode", None,
             [["KeyValueNode", "key",
               [["ConditionalNode", None,
                 [["BinaryExpressionNode", None,
                   [["BinaryOperatorNode", "==", []],
                    ["VariableNode", "x", []],
                       ["NumberNode", "1", []]
                    ]],
                     ["ValueNode", "value", []],
                  ]]]]]]
        )

    def test_expr_1(self):
        self.compare(
            """
key:
  if not x and y : value""",
            ["DataNode", None,
             [["KeyValueNode", "key",
               [["ConditionalNode", None,
                 [["BinaryExpressionNode", None,
                   [["BinaryOperatorNode", "and", []],
                    ["UnaryExpressionNode", None,
                       [["UnaryOperatorNode", "not", []],
                        ["VariableNode", "x", []]
                        ]],
                       ["VariableNode", "y", []]
                    ]],
                     ["ValueNode", "value", []],
                  ]]]]]]
        )

if __name__ == "__main__":
    unittest.main()
