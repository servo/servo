# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this file,
# You can obtain one at http://mozilla.org/MPL/2.0/.

import sys
import os
import unittest

sys.path.insert(0, os.path.abspath(".."))
from cStringIO import StringIO

from .. import parser
from ..parser import token_types


class TokenizerTest(unittest.TestCase):
    def setUp(self):
        self.tokenizer = parser.Tokenizer()

    def tokenize(self, input_str):
        rv = []
        for item in self.tokenizer.tokenize(StringIO(input_str)):
            rv.append(item)
            if item[0] == token_types.eof:
                break
        return rv

    def compare(self, input_text, expected):
        expected = expected + [(token_types.eof, None)]
        actual = self.tokenize(input_text)
        self.assertEquals(actual, expected)

    def test_heading_0(self):
        self.compare("""[Heading text]""",
                     [(token_types.paren, "["),
                      (token_types.string, "Heading text"),
                      (token_types.paren, "]")])

    def test_heading_1(self):
        self.compare("""[Heading [text\]]""",
                     [(token_types.paren, "["),
                      (token_types.string, "Heading [text]"),
                      (token_types.paren, "]")])

    def test_heading_2(self):
        self.compare("""[Heading #text]""",
                     [(token_types.paren, "["),
                      (token_types.string, "Heading #text"),
                      (token_types.paren, "]")])

    def test_heading_3(self):
        self.compare("""[Heading [\]text]""",
                     [(token_types.paren, "["),
                      (token_types.string, "Heading []text"),
                      (token_types.paren, "]")])

    def test_heading_4(self):
        with self.assertRaises(parser.ParseError):
            self.tokenize("[Heading")

    def test_heading_5(self):
        self.compare("""[Heading [\]text] #comment""",
                     [(token_types.paren, "["),
                      (token_types.string, "Heading []text"),
                      (token_types.paren, "]")])

    def test_heading_6(self):
        self.compare(r"""[Heading \ttext]""",
                     [(token_types.paren, "["),
                      (token_types.string, "Heading \ttext"),
                      (token_types.paren, "]")])

    def test_key_0(self):
        self.compare("""key:value""",
                     [(token_types.string, "key"),
                      (token_types.separator, ":"),
                      (token_types.string, "value")])

    def test_key_1(self):
        self.compare("""key  :  value""",
                     [(token_types.string, "key"),
                      (token_types.separator, ":"),
                      (token_types.string, "value")])

    def test_key_2(self):
        self.compare("""key  :  val ue""",
                     [(token_types.string, "key"),
                      (token_types.separator, ":"),
                      (token_types.string, "val ue")])

    def test_key_3(self):
        self.compare("""key: value#comment""",
                     [(token_types.string, "key"),
                      (token_types.separator, ":"),
                      (token_types.string, "value")])

    def test_key_4(self):
        with self.assertRaises(parser.ParseError):
            self.tokenize("""ke y: value""")

    def test_key_5(self):
        with self.assertRaises(parser.ParseError):
            self.tokenize("""key""")

    def test_key_6(self):
        self.compare("""key: "value\"""",
                     [(token_types.string, "key"),
                      (token_types.separator, ":"),
                      (token_types.string, "value")])

    def test_key_7(self):
        self.compare("""key: 'value'""",
                     [(token_types.string, "key"),
                      (token_types.separator, ":"),
                      (token_types.string, "value")])

    def test_key_8(self):
        self.compare("""key: "#value\"""",
                     [(token_types.string, "key"),
                      (token_types.separator, ":"),
                      (token_types.string, "#value")])

    def test_key_9(self):
        self.compare("""key: '#value\'""",
                     [(token_types.string, "key"),
                      (token_types.separator, ":"),
                      (token_types.string, "#value")])

    def test_key_10(self):
        with self.assertRaises(parser.ParseError):
            self.tokenize("""key: "value""")

    def test_key_11(self):
        with self.assertRaises(parser.ParseError):
            self.tokenize("""key: 'value""")

    def test_key_12(self):
        with self.assertRaises(parser.ParseError):
            self.tokenize("""key: 'value""")

    def test_key_13(self):
        with self.assertRaises(parser.ParseError):
            self.tokenize("""key: 'value' abc""")

    def test_key_14(self):
        self.compare(r"""key: \\nb""",
                     [(token_types.string, "key"),
                      (token_types.separator, ":"),
                      (token_types.string, r"\nb")])

    def test_list_0(self):
        self.compare(
"""
key: []""",
            [(token_types.string, "key"),
             (token_types.separator, ":"),
             (token_types.list_start, "["),
             (token_types.list_end, "]")])

    def test_list_1(self):
        self.compare(
"""
key: [a, "b"]""",
            [(token_types.string, "key"),
             (token_types.separator, ":"),
             (token_types.list_start, "["),
             (token_types.string, "a"),
             (token_types.string, "b"),
             (token_types.list_end, "]")])

    def test_list_2(self):
        self.compare(
"""
key: [a,
      b]""",
            [(token_types.string, "key"),
             (token_types.separator, ":"),
             (token_types.list_start, "["),
             (token_types.string, "a"),
             (token_types.string, "b"),
             (token_types.list_end, "]")])

    def test_list_3(self):
        self.compare(
"""
key: [a, #b]
      c]""",
            [(token_types.string, "key"),
             (token_types.separator, ":"),
             (token_types.list_start, "["),
             (token_types.string, "a"),
             (token_types.string, "c"),
             (token_types.list_end, "]")])

    def test_list_4(self):
        with self.assertRaises(parser.ParseError):
            self.tokenize("""key: [a #b]
            c]""")

    def test_list_5(self):
        with self.assertRaises(parser.ParseError):
            self.tokenize("""key: [a \\
            c]""")

    def test_list_6(self):
        self.compare(
"""key: [a , b]""",
        [(token_types.string, "key"),
         (token_types.separator, ":"),
         (token_types.list_start, "["),
         (token_types.string, "a"),
         (token_types.string, "b"),
         (token_types.list_end, "]")])

    def test_expr_0(self):
        self.compare(
"""
key:
  if cond == 1: value""",
            [(token_types.string, "key"),
             (token_types.separator, ":"),
             (token_types.group_start, None),
             (token_types.ident, "if"),
             (token_types.ident, "cond"),
             (token_types.ident, "=="),
             (token_types.number, "1"),
             (token_types.separator, ":"),
             (token_types.string, "value")])

    def test_expr_1(self):
        self.compare(
"""
key:
  if cond == 1: value1
  value2""",
            [(token_types.string, "key"),
             (token_types.separator, ":"),
             (token_types.group_start, None),
             (token_types.ident, "if"),
             (token_types.ident, "cond"),
             (token_types.ident, "=="),
             (token_types.number, "1"),
             (token_types.separator, ":"),
             (token_types.string, "value1"),
             (token_types.string, "value2")])

    def test_expr_2(self):
        self.compare(
"""
key:
  if cond=="1": value""",
            [(token_types.string, "key"),
             (token_types.separator, ":"),
             (token_types.group_start, None),
             (token_types.ident, "if"),
             (token_types.ident, "cond"),
             (token_types.ident, "=="),
             (token_types.string, "1"),
             (token_types.separator, ":"),
             (token_types.string, "value")])

    def test_expr_3(self):
        self.compare(
"""
key:
  if cond==1.1: value""",
            [(token_types.string, "key"),
             (token_types.separator, ":"),
             (token_types.group_start, None),
             (token_types.ident, "if"),
             (token_types.ident, "cond"),
             (token_types.ident, "=="),
             (token_types.number, "1.1"),
             (token_types.separator, ":"),
             (token_types.string, "value")])

    def test_expr_4(self):
        self.compare(
            """
key:
  if cond==1.1 and cond2 == "a": value""",
            [(token_types.string, "key"),
             (token_types.separator, ":"),
             (token_types.group_start, None),
             (token_types.ident, "if"),
             (token_types.ident, "cond"),
             (token_types.ident, "=="),
             (token_types.number, "1.1"),
             (token_types.ident, "and"),
             (token_types.ident, "cond2"),
             (token_types.ident, "=="),
             (token_types.string, "a"),
             (token_types.separator, ":"),
             (token_types.string, "value")])

    def test_expr_5(self):
        self.compare(
"""
key:
  if (cond==1.1 ): value""",
            [(token_types.string, "key"),
             (token_types.separator, ":"),
             (token_types.group_start, None),
             (token_types.ident, "if"),
             (token_types.paren, "("),
             (token_types.ident, "cond"),
             (token_types.ident, "=="),
             (token_types.number, "1.1"),
             (token_types.paren, ")"),
             (token_types.separator, ":"),
             (token_types.string, "value")])

    def test_expr_6(self):
        self.compare(
"""
key:
  if "\\ttest": value""",
            [(token_types.string, "key"),
             (token_types.separator, ":"),
             (token_types.group_start, None),
             (token_types.ident, "if"),
             (token_types.string, "\ttest"),
             (token_types.separator, ":"),
             (token_types.string, "value")])

    def test_expr_7(self):
        with self.assertRaises(parser.ParseError):
            self.tokenize(
"""
key:
  if 1A: value""")

    def test_expr_8(self):
        with self.assertRaises(parser.ParseError):
            self.tokenize(
"""
key:
  if 1a: value""")

    def test_expr_9(self):
        with self.assertRaises(parser.ParseError):
            self.tokenize(
"""
key:
  if 1.1.1: value""")

    def test_expr_10(self):
        self.compare(
"""
key:
  if 1.: value""",
            [(token_types.string, "key"),
             (token_types.separator, ":"),
             (token_types.group_start, None),
             (token_types.ident, "if"),
             (token_types.number, "1."),
             (token_types.separator, ":"),
             (token_types.string, "value")])

if __name__ == "__main__":
    unittest.main()
