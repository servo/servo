from __future__ import unicode_literals

import os

import mock
import pytest
import six

from .. import lint as lint_mod
from ..lint import filter_whitelist_errors, parse_whitelist, lint

_dummy_repo = os.path.join(os.path.dirname(__file__), "dummy")


def _mock_lint(name):
    wrapped = getattr(lint_mod, name)
    return mock.patch(lint_mod.__name__ + "." + name, wraps=wrapped)


def test_filter_whitelist_errors():
    filtered = filter_whitelist_errors({}, '', [])
    assert filtered == []


def test_parse_whitelist():
    input_buffer = six.StringIO("""
# Comment
CR AT EOL: svg/import/*
CR AT EOL: streams/resources/test-utils.js

INDENT TABS: .gitmodules
INDENT TABS: app-uri/*
INDENT TABS: svg/*

TRAILING WHITESPACE: app-uri/*

CONSOLE:streams/resources/test-utils.js: 12

*:*.pdf
*:resources/*
""")

    expected_data = {
        '.gitmodules': {
            'INDENT TABS': {None},
        },
        'app-uri/*': {
            'TRAILING WHITESPACE': {None},
            'INDENT TABS': {None},
        },
        'streams/resources/test-utils.js': {
            'CONSOLE': {12},
            'CR AT EOL': {None},
        },
        'svg/*': {
            'INDENT TABS': {None},
        },
        'svg/import/*': {
            'CR AT EOL': {None},
        },
    }
    expected_data = {os.path.normcase(p): e for p, e in expected_data.items()}
    expected_ignored = {os.path.normcase(x) for x in {"*.pdf", "resources/*"}}
    data, ignored = parse_whitelist(input_buffer)
    assert data == expected_data
    assert ignored == expected_ignored


def test_lint_no_files(capsys):
    rv = lint(_dummy_repo, [], False)
    assert rv == 0
    out, err = capsys.readouterr()
    assert out == ""
    assert err == ""


def test_lint_ignored_file(capsys):
    with _mock_lint("check_path") as mocked_check_path:
        with _mock_lint("check_file_contents") as mocked_check_file_contents:
            rv = lint(_dummy_repo, ["broken_ignored.html"], False)
            assert rv == 0
            assert not mocked_check_path.called
            assert not mocked_check_file_contents.called
    out, err = capsys.readouterr()
    assert out == ""
    assert err == ""


def test_lint_not_existing_file(capsys):
    with _mock_lint("check_path") as mocked_check_path:
        with _mock_lint("check_file_contents") as mocked_check_file_contents:
            # really long path-linted filename
            name = "a" * 256 + ".html"
            rv = lint(_dummy_repo, [name], False)
            assert rv == 0
            assert not mocked_check_path.called
            assert not mocked_check_file_contents.called
    out, err = capsys.readouterr()
    assert out == ""
    assert err == ""


def test_lint_passing(capsys):
    with _mock_lint("check_path") as mocked_check_path:
        with _mock_lint("check_file_contents") as mocked_check_file_contents:
            rv = lint(_dummy_repo, ["okay.html"], False)
            assert rv == 0
            assert mocked_check_path.call_count == 1
            assert mocked_check_file_contents.call_count == 1
    out, err = capsys.readouterr()
    assert out == ""
    assert err == ""


def test_lint_failing(capsys):
    with _mock_lint("check_path") as mocked_check_path:
        with _mock_lint("check_file_contents") as mocked_check_file_contents:
            rv = lint(_dummy_repo, ["broken.html"], False)
            assert rv == 1
            assert mocked_check_path.call_count == 1
            assert mocked_check_file_contents.call_count == 1
    out, err = capsys.readouterr()
    assert "TRAILING WHITESPACE" in out
    assert "broken.html 1 " in out
    assert err == ""


def test_lint_passing_and_failing(capsys):
    with _mock_lint("check_path") as mocked_check_path:
        with _mock_lint("check_file_contents") as mocked_check_file_contents:
            rv = lint(_dummy_repo, ["broken.html", "okay.html"], False)
            assert rv == 1
            assert mocked_check_path.call_count == 2
            assert mocked_check_file_contents.call_count == 2
    out, err = capsys.readouterr()
    assert "TRAILING WHITESPACE" in out
    assert "broken.html 1 " in out
    assert "okay.html" not in out
    assert err == ""
