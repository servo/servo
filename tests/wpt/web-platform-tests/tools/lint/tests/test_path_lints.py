from __future__ import unicode_literals

from ..lint import check_path
from .base import check_errors
import pytest
import six

def test_allowed_path_length():
    basename = 29 * "test/"

    for idx in range(5):
        filename = basename + idx * "a"

        errors = check_path("/foo/", filename)
        check_errors(errors)
        assert errors == []


def test_forbidden_path_length():
    basename = 29 * "test/"

    for idx in range(5, 10):
        filename = basename + idx * "a"
        message = "/%s longer than maximum path length (%s > 150)" % (filename, 146 + idx)

        errors = check_path("/foo/", filename)
        check_errors(errors)
        assert errors == [("PATH LENGTH", message, filename, None)]
