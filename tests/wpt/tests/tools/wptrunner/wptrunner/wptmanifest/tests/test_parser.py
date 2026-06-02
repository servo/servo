# mypy: allow-untyped-defs

import unittest

from .. import parser

# There aren't many tests here because it turns out to be way more convenient to
# use test_serializer for the majority of cases


class TestExpression(unittest.TestCase):
    def setUp(self):
        self.parser = parser.Parser()

    def parse(self, input_str):
        return self.parser.parse(input_str)

    def compare(self, input_text, expected):
        actual = self.parse(input_text)
        self.match(expected, actual)

    def match(self, expected_node, actual_node):
        self.assertEqual(expected_node[0], actual_node.__class__.__name__)
        self.assertEqual(expected_node[1], actual_node.data)
        self.assertEqual(len(expected_node[2]), len(actual_node.children))
        for expected_child, actual_child in zip(expected_node[2], actual_node.children):
            self.match(expected_child, actual_child)

    def test_expr_0(self):
        self.compare(
            b"""
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
            b"""
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

    def test_expr_2(self):
        self.compare(
            b"""
key:
  if x == 1 : [value1, value2]""",
            ["DataNode", None,
             [["KeyValueNode", "key",
               [["ConditionalNode", None,
                 [["BinaryExpressionNode", None,
                   [["BinaryOperatorNode", "==", []],
                    ["VariableNode", "x", []],
                       ["NumberNode", "1", []]
                    ]],
                  ["ListNode", None,
                   [["ValueNode", "value1", []],
                    ["ValueNode", "value2", []]]],
                  ]]]]]]
        )

    def test_expr_3(self):
        self.compare(
            b"""
key:
  if x == 1: 'if b: value'""",
            ["DataNode", None,
             [["KeyValueNode", "key",
               [["ConditionalNode", None,
                 [["BinaryExpressionNode", None,
                   [["BinaryOperatorNode", "==", []],
                    ["VariableNode", "x", []],
                       ["NumberNode", "1", []]
                    ]],
                     ["ValueNode", "if b: value", []],
                  ]]]]]]
        )

    def test_atom_0(self):
        with self.assertRaises(parser.ParseError):
            self.parse(b"key: @Unknown")

    def test_atom_1(self):
        with self.assertRaises(parser.ParseError):
            self.parse(b"key: @true")

    def test_list_expr(self):
        self.compare(
            b"""
key:
  if x == 1: [a]
  [b]""",
            ["DataNode", None,
             [["KeyValueNode", "key",
               [["ConditionalNode", None,
                 [["BinaryExpressionNode", None,
                   [["BinaryOperatorNode", "==", []],
                    ["VariableNode", "x", []],
                       ["NumberNode", "1", []]
                    ]],
                     ["ListNode", None,
                      [["ValueNode", "a", []]]],
                  ]],
                ["ListNode", None,
                 [["ValueNode", "b", []]]]]]]])

    def test_list_heading(self):
        self.compare(
            b"""
key:
  if x == 1: [a]
[b]""",
            ["DataNode", None,
             [["KeyValueNode", "key",
               [["ConditionalNode", None,
                 [["BinaryExpressionNode", None,
                   [["BinaryOperatorNode", "==", []],
                    ["VariableNode", "x", []],
                       ["NumberNode", "1", []]
                    ]],
                     ["ListNode", None,
                      [["ValueNode", "a", []]]],
                  ]]]],
              ["DataNode", "b", []]]])

    def test_if_1(self):
        with self.assertRaises(parser.ParseError):
            self.parse(b"key: if foo")


if __name__ == "__main__":
    unittest.main()
