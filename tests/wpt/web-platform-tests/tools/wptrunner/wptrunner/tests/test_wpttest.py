import os
import sys
from io import BytesIO

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", ".."))

from wptrunner import manifestexpected, wpttest
from .test_chunker import make_mock_manifest

dir_ini_0 = """\
prefs: [a:b]
"""

dir_ini_1 = """\
prefs: [@Reset, b:c]
max-asserts: 2
min-asserts: 1
tags: [b, c]
"""

test_0 = """\
[0.html]
  prefs: [c:d]
  max-asserts: 3
  tags: [a, @Reset]
"""


def test_metadata_inherit():
    tests = make_mock_manifest(("test", "a", 10), ("test", "a/b", 10),
                               ("test", "c", 10))

    inherit_metadata = [
        manifestexpected.static.compile(
            BytesIO(item),
            {},
            data_cls_getter=lambda x,y: manifestexpected.DirectoryManifest)
        for item in [dir_ini_0, dir_ini_1]]
    test_metadata = manifestexpected.static.compile(BytesIO(test_0),
                                                    {},
                                                    data_cls_getter=manifestexpected.data_cls_getter,
                                                    test_path="a",
                                                    url_base="")

    test = tests[0][2].pop()
    test_obj = wpttest.from_manifest(test, inherit_metadata, test_metadata.get_test(test.id))
    assert test_obj.max_assertion_count == 3
    assert test_obj.min_assertion_count == 1
    assert test_obj.prefs == {"b": "c", "c": "d"}
    assert test_obj.tags == {"a", "dir:a"}
