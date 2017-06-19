import unittest

from cStringIO import StringIO

from ..backends import static

# There aren't many tests here because it turns out to be way more convenient to
# use test_serializer for the majority of cases


class TestStatic(unittest.TestCase):
    def compile(self, input_text, input_data):
        return static.compile(input_text, input_data)

    def test_get_0(self):
        data = """
key: value

[Heading 1]
  other_key:
    if a == 1: value_1
    if a == 2: value_2
    value_3
"""

        manifest = self.compile(data, {"a": 2})

        self.assertEquals(manifest.get("key"), "value")
        children = list(item for item in manifest.iterchildren())
        self.assertEquals(len(children), 1)
        section = children[0]
        self.assertEquals(section.name, "Heading 1")

        self.assertEquals(section.get("other_key"), "value_2")
        self.assertEquals(section.get("key"), "value")

    def test_get_1(self):
        data = """
key: value

[Heading 1]
  other_key:
    if a == 1: value_1
    if a == 2: value_2
    value_3
"""
        manifest = self.compile(data, {"a": 3})

        children = list(item for item in manifest.iterchildren())
        section = children[0]
        self.assertEquals(section.get("other_key"), "value_3")

    def test_get_3(self):
        data = """key:
  if a == "1": value_1
  if a[0] == "ab"[0]: value_2
"""
        manifest = self.compile(data, {"a": "1"})
        self.assertEquals(manifest.get("key"), "value_1")

        manifest = self.compile(data, {"a": "ac"})
        self.assertEquals(manifest.get("key"), "value_2")

    def test_get_4(self):
        data = """key:
  if not a: value_1
  value_2
"""
        manifest = self.compile(data, {"a": True})
        self.assertEquals(manifest.get("key"), "value_2")

        manifest = self.compile(data, {"a": False})
        self.assertEquals(manifest.get("key"), "value_1")

    def test_api(self):
        data = """key:
  if a == 1.5: value_1
  value_2
key_1: other_value
"""
        manifest = self.compile(data, {"a": 1.5})

        self.assertFalse(manifest.is_empty)
        self.assertEquals(manifest.root, manifest)
        self.assertTrue(manifest.has_key("key_1"))
        self.assertFalse(manifest.has_key("key_2"))

        self.assertEquals(set(manifest.iterkeys()), set(["key", "key_1"]))
        self.assertEquals(set(manifest.itervalues()), set(["value_1", "other_value"]))

    def test_is_empty_1(self):
        data = """
[Section]
  [Subsection]
"""
        manifest = self.compile(data, {})

        self.assertTrue(manifest.is_empty)
