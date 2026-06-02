# mypy: allow-untyped-defs

import textwrap
import unittest

from .. import parser, serializer


class SerializerTest(unittest.TestCase):
    def setUp(self):
        self.serializer = serializer.ManifestSerializer()
        self.parser = parser.Parser()

    def serialize(self, input_str):
        return self.serializer.serialize(self.parser.parse(input_str))

    def compare(self, input_str, expected=None):
        if expected is None:
            expected = input_str.decode("utf-8")
        actual = self.serialize(input_str)
        self.assertEqual(actual, expected)

    def test_0(self):
        self.compare(b"""key: value
[Heading 1]
  other_key: other_value
""")

    def test_1(self):
        self.compare(b"""key: value
[Heading 1]
  other_key:
    if a or b: other_value
""")

    def test_2(self):
        self.compare(b"""key: value
[Heading 1]
  other_key:
    if a or b: other_value
    fallback_value
""")

    def test_3(self):
        self.compare(b"""key: value
[Heading 1]
  other_key:
    if a == 1: other_value
    fallback_value
""")

    def test_4(self):
        self.compare(b"""key: value
[Heading 1]
  other_key:
    if a == "1": other_value
    fallback_value
""")

    def test_5(self):
        self.compare(b"""key: value
[Heading 1]
  other_key:
    if a == "abc"[1]: other_value
    fallback_value
""")

    def test_6(self):
        self.compare(b"""key: value
[Heading 1]
  other_key:
    if a == "abc"[c]: other_value
    fallback_value
""")

    def test_7(self):
        self.compare(b"""key: value
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
        self.compare(b"""key: value
[Heading 1]
  other_key:
    if a or (b and c): other_value
    fallback_value
""")

    def test_9(self):
        self.compare(b"""key: value
[Heading 1]
  other_key:
    if not (a and b): other_value
    fallback_value
""")

    def test_10(self):
        self.compare(b"""key: value
[Heading 1]
  some_key: some_value

[Heading 2]
  other_key: other_value
""")

    def test_11(self):
        self.compare(b"""key:
  if not a and b and c and d: true
""")

    def test_12(self):
        self.compare(b"""[Heading 1]
  key: [a:1, b:2]
""")

    def test_13(self):
        self.compare(b"""key: [a:1, "b:#"]
""")

    def test_14(self):
        self.compare(b"""key: [","]
""")

    def test_15(self):
        self.compare(b"""key: ,
""")

    def test_16(self):
        self.compare(b"""key: ["]", b]
""")

    def test_17(self):
        self.compare(b"""key: ]
""")

    def test_18(self):
        self.compare(br"""key: \]
        """, """key: ]
""")

    def test_atom_as_default(self):
        self.compare(
            textwrap.dedent(
                """\
                key:
                  if a == 1: @True
                  @False
                """).encode())

    def test_escape_0(self):
        self.compare(br"""k\t\:y: \a\b\f\n\r\t\v""",
                     r"""k\t\:y: \x07\x08\x0c\n\r\t\x0b
""")

    def test_escape_1(self):
        self.compare(br"""k\x00: \x12A\x45""",
                     r"""k\x00: \x12AE
""")

    def test_escape_2(self):
        self.compare(br"""k\u0045y: \u1234A\uABc6""",
                     """kEy: \u1234A\uabc6
""")

    def test_escape_3(self):
        self.compare(br"""k\u0045y: \u1234A\uABc6""",
                     """kEy: \u1234A\uabc6
""")

    def test_escape_4(self):
        self.compare(br"""key: '\u1234A\uABc6'""",
                     """key: \u1234A\uabc6
""")

    def test_escape_5(self):
        self.compare(br"""key: [\u1234A\uABc6]""",
                     """key: [\u1234A\uabc6]
""")

    def test_escape_6(self):
        self.compare(br"""key: [\u1234A\uABc6\,]""",
                     """key: ["\u1234A\uabc6,"]
""")

    def test_escape_7(self):
        self.compare(br"""key: [\,\]\#]""",
                     r"""key: [",]#"]
""")

    def test_escape_8(self):
        self.compare(br"""key: \#""",
                     r"""key: "#"
""")

    def test_escape_9(self):
        self.compare(br"""key: \U10FFFFabc""",
                     """key: \U0010FFFFabc
""")

    def test_escape_10(self):
        self.compare(br"""key: \u10FFab""",
                     """key: \u10FFab
""")

    def test_escape_11(self):
        self.compare(br"""key: \\ab
""")

    def test_atom_1(self):
        self.compare(br"""key: @True
""")

    def test_atom_2(self):
        self.compare(br"""key: @False
""")

    def test_atom_3(self):
        self.compare(br"""key: @Reset
""")

    def test_atom_4(self):
        self.compare(br"""key: [a, @Reset, b]
""")

    def test_conditional_1(self):
        self.compare(b"""foo:
  if a or b: [1, 2]
""")

    def test_if_string_0(self):
        self.compare(b"""foo: "if bar"
""")

    def test_non_ascii_1(self):
        self.compare(b"""[\xf0\x9f\x99\x84]
""")

    def test_comments_preceding_kv_pair(self):
        self.compare(
            textwrap.dedent(
                """\
                # These two comments should be attached
                # to the first key-value pair.
                key1: value
                # Attached to the second pair.
                key2: value
                """).encode())

    def test_comments_preceding_headings(self):
        self.compare(
            textwrap.dedent(
                """\
                # Attached to the first heading.
                [test1.html]

                # Attached to the second heading.
                [test2.html]
                # Attached to subheading.
                    # Also attached to subheading.
                  [subheading]  # Also attached to subheading (inline).
                """).encode(),
            textwrap.dedent(
                """\
                # Attached to the first heading.
                [test1.html]

                # Attached to the second heading.
                [test2.html]
                  # Attached to subheading.
                  # Also attached to subheading.
                  [subheading]  # Also attached to subheading (inline).
                """))

    def test_comments_inline(self):
        self.compare(
            textwrap.dedent(
                """\
                key1:           # inline after key
                  value         # inline after string value
                key2:
                  [value]       # inline after list in group
                [test.html]     # inline after heading
                  key1: @True   # inline after atom
                  key2: [       # inline after list start
                    @False,     # inline after atom in list
                    value1,     # inline after value in list
                    value2]     # inline after list end
                """).encode(),
            textwrap.dedent(
                """\
                # inline after key
                key1: value  # inline after string value
                key2: [value]  # inline after list in group
                [test.html]  # inline after heading
                  key1: @True  # inline after atom
                  # inline after atom in list
                  # inline after value in list
                  # inline after list end
                  key2: [@False, value1, value2]  # inline after list start
                """))

    def test_comments_conditions(self):
        self.compare(
            textwrap.dedent(
                """\
                key1:
                # cond 1
                  if cond == 1: value
                  # cond 2
                  if cond == 2: value  # cond 2
                # cond 3
                    # cond 3
                  if cond == 3: value
                  # default 0
                  default  # default 1
                  # default 2
                  # default 3
                key2:
                  if cond == 1: value
                  [value]
                  # list default
                key3:
                  if cond == 1: value
                  # no default
                """).encode(),
            textwrap.dedent(
                """\
                key1:
                  # cond 1
                  if cond == 1: value
                  # cond 2
                  if cond == 2: value  # cond 2
                  # cond 3
                  # cond 3
                  if cond == 3: value
                  # default 0
                  # default 2
                  # default 3
                  default  # default 1
                key2:
                  if cond == 1: value
                  # list default
                  [value]
                # no default
                key3:
                  if cond == 1: value
                """))
