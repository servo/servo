from __future__ import unicode_literals

from ..lint import check_path
from .base import check_errors
import pytest

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

@pytest.mark.parametrize("path_ending,generated", [(".worker.html", ".worker.js"),
                                                   (".any.worker.html", ".any.js"),
                                                   (".any.html", ".any.js")])
def test_forbidden_path_endings(path_ending, generated):
    path = "/test/test" + path_ending

    message = ("path ends with %s which collides with generated tests from %s files" %
               (path_ending, generated))

    errors = check_path("/foo/", path)
    check_errors(errors)
    assert errors == [("WORKER COLLISION", message, path, None)]
