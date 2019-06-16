from __future__ import unicode_literals

import mock
import os

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
    path = "test/test" + path_ending

    message = ("path ends with %s which collides with generated tests from %s files" %
               (path_ending, generated))

    errors = check_path("/foo/", path)
    check_errors(errors)
    assert errors == [("WORKER COLLISION", message, path, None)]


def test_file_type():
    path = "test/test"

    message = "/%s is an unsupported file type (symlink)" % (path,)

    with mock.patch("os.path.islink", returnvalue=True):
        errors = check_path("/foo/", path)

    assert errors == [("FILE TYPE", message, path, None)]


@pytest.mark.parametrize("path", ["ahem.ttf",
                                  "Ahem.ttf",
                                  "ahem.tTf",
                                  "not-ahem.ttf",
                                  "support/ahem.ttf",
                                  "ahem/other.ttf"])
def test_ahem_copy(path):
    expected_error = ("AHEM COPY",
                      "Don't add extra copies of Ahem, use /fonts/Ahem.ttf",
                      path,
                      None)

    errors = check_path("/foo/", path)

    assert errors == [expected_error]

@pytest.mark.parametrize("path", ["ahem.woff",
                                  "ahem.ttff",
                                  "support/ahem.woff",
                                  "ahem/other.woff"])
def test_ahem_copy_negative(path):
    errors = check_path("/foo/", path)

    assert errors == []

@pytest.mark.parametrize("path", ["elsewhere/.gitignore",
                                  "else/where/.gitignore"
                                  "elsewhere/tools/.gitignore",
                                  "elsewhere/docs/.gitignore",
                                  "elsewhere/resources/webidl2/.gitignore",
                                  "elsewhere/css/tools/apiclient/.gitignore"])
def test_gitignore_file(path):
    path = os.path.join(*path.split("/"))

    expected_error = ("GITIGNORE",
                      ".gitignore found outside the root",
                      path,
                      None)

    errors = check_path("/foo/", path)

    assert errors == [expected_error]

@pytest.mark.parametrize("path", [".gitignore",
                                  "elsewhere/.gitignores",
                                  "elsewhere/name.gitignore",
                                  "tools/.gitignore",
                                  "tools/elsewhere/.gitignore",
                                  "docs/.gitignore"
                                  "docs/elsewhere/.gitignore",
                                  "resources/webidl2/.gitignore",
                                  "resources/webidl2/elsewhere/.gitignore",
                                  "css/tools/apiclient/.gitignore",
                                  "css/tools/apiclient/elsewhere/.gitignore"])
def test_gitignore_negative(path):
    path = os.path.join(*path.split("/"))

    errors = check_path("/foo/", path)

    assert errors == []
