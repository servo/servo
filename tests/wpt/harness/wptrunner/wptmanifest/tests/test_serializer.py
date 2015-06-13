# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this file,
# You can obtain one at http://mozilla.org/MPL/2.0/.

import unittest

from cStringIO import StringIO

from .. import parser, serializer


class TokenizerTest(unittest.TestCase):
    def setUp(self):
        self.serializer = serializer.ManifestSerializer()
        self.parser = parser.Parser()

    def serialize(self, input_str):
        return self.serializer.serialize(self.parser.parse(input_str))

    def compare(self, input_str, expected=None):
        if expected is None:
            expected = input_str
        expected = expected.encode("utf8")
        actual = self.serialize(input_str)
        self.assertEquals(actual, expected)

    def test_0(self):
        self.compare("""key: value
[Heading 1]
  other_key: other_value
""")

    def test_1(self):
        self.compare("""key: value
[Heading 1]
  other_key:
    if a or b: other_value
""")

    def test_2(self):
        self.compare("""key: value
[Heading 1]
  other_key:
    if a or b: other_value
    fallback_value
""")

    def test_3(self):
        self.compare("""key: value
[Heading 1]
  other_key:
    if a == 1: other_value
    fallback_value
""")

    def test_4(self):
        self.compare("""key: value
[Heading 1]
  other_key:
    if a == "1": other_value
    fallback_value
""")

    def test_5(self):
        self.compare("""key: value
[Heading 1]
  other_key:
    if a == "abc"[1]: other_value
    fallback_value
""")

    def test_6(self):
        self.compare("""key: value
[Heading 1]
  other_key:
    if a == "abc"[c]: other_value
    fallback_value
""")

    def test_7(self):
        self.compare("""key: value
[Heading 1]
  other_key:
    if (a or b) and c: other_value
    fallback_value
""",
"""key: value
[Heading 1]
  other_key:
    if a or b and c: other_value
    fallback_value
""")

    def test_8(self):
        self.compare("""key: value
[Heading 1]
  other_key:
    if a or (b and c): other_value
    fallback_value
""")

    def test_9(self):
        self.compare("""key: value
[Heading 1]
  other_key:
    if not (a and b): other_value
    fallback_value
""")

    def test_10(self):
        self.compare("""key: value
[Heading 1]
  some_key: some_value

[Heading 2]
  other_key: other_value
""")

    def test_11(self):
        self.compare("""key:
  if not a and b and c and d: true
""")

    def test_12(self):
        self.compare("""[Heading 1]
  key: [a:1, b:2]
""")

    def test_13(self):
        self.compare("""key: [a:1, "b:#"]
""")

    def test_14(self):
        self.compare("""key: [","]
""")

    def test_15(self):
        self.compare("""key: ,
""")

    def test_16(self):
        self.compare("""key: ["]", b]
""")

    def test_17(self):
        self.compare("""key: ]
""")

    def test_18(self):
        self.compare("""key: \]
        """, """key: ]
""")

    def test_escape_0(self):
        self.compare(r"""k\t\:y: \a\b\f\n\r\t\v""",
                     r"""k\t\:y: \x07\x08\x0c\n\r\t\x0b
""")

    def test_escape_1(self):
        self.compare(r"""k\x00: \x12A\x45""",
                     r"""k\x00: \x12AE
""")

    def test_escape_2(self):
        self.compare(r"""k\u0045y: \u1234A\uABc6""",
                     u"""kEy: \u1234A\uabc6
""")

    def test_escape_3(self):
        self.compare(r"""k\u0045y: \u1234A\uABc6""",
                     u"""kEy: \u1234A\uabc6
""")

    def test_escape_4(self):
        self.compare(r"""key: '\u1234A\uABc6'""",
                     u"""key: \u1234A\uabc6
""")

    def test_escape_5(self):
        self.compare(r"""key: [\u1234A\uABc6]""",
                     u"""key: [\u1234A\uabc6]
""")

    def test_escape_6(self):
        self.compare(r"""key: [\u1234A\uABc6\,]""",
                     u"""key: ["\u1234A\uabc6,"]
""")

    def test_escape_7(self):
        self.compare(r"""key: [\,\]\#]""",
                     r"""key: [",]#"]
""")

    def test_escape_8(self):
        self.compare(r"""key: \#""",
                     r"""key: "#"
""")

    def test_escape_9(self):
        self.compare(r"""key: \U10FFFFabc""",
                     u"""key: \U0010FFFFabc
""")

    def test_escape_10(self):
        self.compare(r"""key: \u10FFab""",
                     u"""key: \u10FFab
""")

    def test_escape_11(self):
        self.compare(r"""key: \\ab
""")

    def test_atom_1(self):
            self.compare(r"""key: @True
""")

    def test_atom_2(self):
            self.compare(r"""key: @False
""")

    def test_atom_3(self):
            self.compare(r"""key: @Reset
""")

    def test_atom_4(self):
        self.compare(r"""key: [a, @Reset, b]
""")
