import os
import sys
from io import BytesIO

import pytest

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "..", ".."))

from wptrunner import manifestexpected


@pytest.mark.parametrize("fuzzy, expected", [
    (b"ref.html:1;200", [("ref.html", ((1, 1), (200, 200)))]),
    (b"ref.html:0-1;100-200", [("ref.html", ((0, 1), (100, 200)))]),
    (b"0-1;100-200", [(None, ((0, 1), (100, 200)))]),
    (b"maxDifference=1;totalPixels=200", [(None, ((1, 1), (200, 200)))]),
    (b"totalPixels=200;maxDifference=1", [(None, ((1, 1), (200, 200)))]),
    (b"totalPixels=200;1", [(None, ((1, 1), (200, 200)))]),
    (b"maxDifference=1;200", [(None, ((1, 1), (200, 200)))]),
    (b"test.html==ref.html:maxDifference=1;totalPixels=200",
     [((u"test.html", u"ref.html", "=="), ((1, 1), (200, 200)))]),
    (b"test.html!=ref.html:maxDifference=1;totalPixels=200",
     [((u"test.html", u"ref.html", "!="), ((1, 1), (200, 200)))]),
    (b"[test.html!=ref.html:maxDifference=1;totalPixels=200, test.html==ref1.html:maxDifference=5-10;100]",
     [((u"test.html", u"ref.html", "!="), ((1, 1), (200, 200))),
      ((u"test.html", u"ref1.html", "=="), ((5,10), (100, 100)))]),
])
def test_fuzzy(fuzzy, expected):
    data = """
[test.html]
  fuzzy: %s""" % fuzzy
    f = BytesIO(data)
    manifest = manifestexpected.static.compile(f,
                                               {},
                                               data_cls_getter=manifestexpected.data_cls_getter,
                                               test_path="test/test.html",
                                               url_base="/")
    assert manifest.get_test("/test/test.html").fuzzy == expected
