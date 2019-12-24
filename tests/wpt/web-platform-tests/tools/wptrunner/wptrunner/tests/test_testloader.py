from __future__ import unicode_literals

import sys
import tempfile

import pytest

from mozlog import structured
from ..testloader import TestFilter as Filter
from .test_wpttest import make_mock_manifest

structured.set_default_logger(structured.structuredlog.StructuredLogger("TestLoader"))

include_ini = """\
skip: true
[test_\u53F0]
  skip: false
"""


@pytest.mark.xfail(sys.platform == "win32",
                   reason="NamedTemporaryFile cannot be reopened on Win32")
def test_filter_unicode():
    tests = make_mock_manifest(("test", "a", 10), ("test", "a/b", 10),
                               ("test", "c", 10))

    with tempfile.NamedTemporaryFile("wb", suffix=".ini") as f:
        f.write(include_ini.encode('utf-8'))
        f.flush()

        Filter(manifest_path=f.name, test_manifests=tests)
