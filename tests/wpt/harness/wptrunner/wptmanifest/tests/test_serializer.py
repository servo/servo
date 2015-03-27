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
        return self.serializer.serialize(self.parser.parse(StringIO(input_str)))

    def compare(self, input_str, expected=None):
        if expected is None:
            expected = input_str

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
"""
                     )

    def test_11(self):
        self.compare("""key:
  if not a and b and c and d: true
"""
                     )
