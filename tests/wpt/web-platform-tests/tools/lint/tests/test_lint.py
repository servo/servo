from __future__ import unicode_literals

import os
import sys

import mock
import pytest
import six

from ...localpaths import repo_root
from .. import lint as lint_mod
from ..lint import filter_whitelist_errors, parse_whitelist, lint, parse_args

_dummy_repo = os.path.join(os.path.dirname(__file__), "dummy")

def _mock_lint(name, **kwargs):
    wrapped = getattr(lint_mod, name)
    return mock.patch(lint_mod.__name__ + "." + name, wraps=wrapped, **kwargs)


def test_filter_whitelist_errors():
    whitelist = {
        'CONSOLE': {
            'svg/*': {12}
        },
        'INDENT TABS': {
            'svg/*': {None}
        }
    }
    # parse_whitelist normalises the case/path of the match string so need to do the same
    whitelist = {e: {os.path.normcase(k): v for k, v in p.items()}
                 for e, p in whitelist.items()}
    # paths passed into filter_whitelist_errors are always Unix style
    filteredfile = 'svg/test.html'
    unfilteredfile = 'html/test.html'
    # Tests for passing no errors
    filtered = filter_whitelist_errors(whitelist, [])
    assert filtered == []
    filtered = filter_whitelist_errors(whitelist, [])
    assert filtered == []
    # Tests for filtering on file and line number
    filtered = filter_whitelist_errors(whitelist, [['CONSOLE', '', filteredfile, 12]])
    assert filtered == []
    filtered = filter_whitelist_errors(whitelist, [['CONSOLE', '', unfilteredfile, 12]])
    assert filtered == [['CONSOLE', '', unfilteredfile, 12]]
    filtered = filter_whitelist_errors(whitelist, [['CONSOLE', '', filteredfile, 11]])
    assert filtered == [['CONSOLE', '', filteredfile, 11]]
    # Tests for filtering on just file
    filtered = filter_whitelist_errors(whitelist, [['INDENT TABS', '', filteredfile, 12]])
    assert filtered == []
    filtered = filter_whitelist_errors(whitelist, [['INDENT TABS', '', filteredfile, 11]])
    assert filtered == []
    filtered = filter_whitelist_errors(whitelist, [['INDENT TABS', '', unfilteredfile, 11]])
    assert filtered == [['INDENT TABS', '', unfilteredfile, 11]]


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

CR AT EOL, INDENT TABS: html/test.js

CR AT EOL, INDENT TABS: html/test2.js: 42

*:*.pdf
*:resources/*

*, CR AT EOL: *.png
""")

    expected_data = {
        'INDENT TABS': {
            '.gitmodules': {None},
            'app-uri/*': {None},
            'svg/*': {None},
            'html/test.js': {None},
            'html/test2.js': {42},
        },
        'TRAILING WHITESPACE': {
            'app-uri/*': {None},
        },
        'CONSOLE': {
            'streams/resources/test-utils.js': {12},
        },
        'CR AT EOL': {
            'streams/resources/test-utils.js': {None},
            'svg/import/*': {None},
            'html/test.js': {None},
            'html/test2.js': {42},
        }
    }
    expected_data = {e: {os.path.normcase(k): v for k, v in p.items()}
                     for e, p in expected_data.items()}
    expected_ignored = {os.path.normcase(x) for x in {"*.pdf", "resources/*", "*.png"}}
    data, ignored = parse_whitelist(input_buffer)
    assert data == expected_data
    assert ignored == expected_ignored


def test_lint_no_files(caplog):
    rv = lint(_dummy_repo, [], "normal", False)
    assert rv == 0
    assert caplog.text == ""


def test_lint_ignored_file(caplog):
    with _mock_lint("check_path") as mocked_check_path:
        with _mock_lint("check_file_contents") as mocked_check_file_contents:
            rv = lint(_dummy_repo, ["broken_ignored.html"], "normal", False)
            assert rv == 0
            assert not mocked_check_path.called
            assert not mocked_check_file_contents.called
    assert caplog.text == ""


def test_lint_not_existing_file(caplog):
    with _mock_lint("check_path") as mocked_check_path:
        with _mock_lint("check_file_contents") as mocked_check_file_contents:
            # really long path-linted filename
            name = "a" * 256 + ".html"
            rv = lint(_dummy_repo, [name], "normal", False)
            assert rv == 0
            assert not mocked_check_path.called
            assert not mocked_check_file_contents.called
    assert caplog.text == ""


def test_lint_passing(caplog):
    with _mock_lint("check_path") as mocked_check_path:
        with _mock_lint("check_file_contents") as mocked_check_file_contents:
            rv = lint(_dummy_repo, ["okay.html"], "normal", False)
            assert rv == 0
            assert mocked_check_path.call_count == 1
            assert mocked_check_file_contents.call_count == 1
    assert caplog.text == ""


def test_lint_failing(caplog):
    with _mock_lint("check_path") as mocked_check_path:
        with _mock_lint("check_file_contents") as mocked_check_file_contents:
            rv = lint(_dummy_repo, ["broken.html"], "normal", False)
            assert rv == 1
            assert mocked_check_path.call_count == 1
            assert mocked_check_file_contents.call_count == 1
    assert "TRAILING WHITESPACE" in caplog.text
    assert "broken.html:1" in caplog.text


def test_ref_existent_relative(caplog):
    with _mock_lint("check_path") as mocked_check_path:
        with _mock_lint("check_file_contents") as mocked_check_file_contents:
            rv = lint(_dummy_repo, ["ref/existent_relative.html"], "normal", False)
            assert rv == 0
            assert mocked_check_path.call_count == 1
            assert mocked_check_file_contents.call_count == 1
    assert caplog.text == ""


def test_ref_existent_root_relative(caplog):
    with _mock_lint("check_path") as mocked_check_path:
        with _mock_lint("check_file_contents") as mocked_check_file_contents:
            rv = lint(_dummy_repo, ["ref/existent_root_relative.html"], "normal", False)
            assert rv == 0
            assert mocked_check_path.call_count == 1
            assert mocked_check_file_contents.call_count == 1
    assert caplog.text == ""


def test_ref_non_existent_relative(caplog):
    with _mock_lint("check_path") as mocked_check_path:
        with _mock_lint("check_file_contents") as mocked_check_file_contents:
            rv = lint(_dummy_repo, ["ref/non_existent_relative.html"], "normal", False)
            assert rv == 1
            assert mocked_check_path.call_count == 1
            assert mocked_check_file_contents.call_count == 1
    assert "NON-EXISTENT-REF" in caplog.text
    assert "ref/non_existent_relative.html" in caplog.text
    assert "non_existent_file.html" in caplog.text


def test_ref_non_existent_root_relative(caplog):
    with _mock_lint("check_path") as mocked_check_path:
        with _mock_lint("check_file_contents") as mocked_check_file_contents:
            rv = lint(_dummy_repo, ["ref/non_existent_root_relative.html"], "normal", False)
            assert rv == 1
            assert mocked_check_path.call_count == 1
            assert mocked_check_file_contents.call_count == 1
    assert "NON-EXISTENT-REF" in caplog.text
    assert "ref/non_existent_root_relative.html" in caplog.text
    assert "/non_existent_file.html" in caplog.text



def test_ref_absolute_url(caplog):
    with _mock_lint("check_path") as mocked_check_path:
        with _mock_lint("check_file_contents") as mocked_check_file_contents:
            rv = lint(_dummy_repo, ["ref/absolute.html"], "normal", False)
            assert rv == 1
            assert mocked_check_path.call_count == 1
            assert mocked_check_file_contents.call_count == 1
    assert "ABSOLUTE-URL-REF" in caplog.text
    assert "http://example.com/reference.html" in caplog.text
    assert "ref/absolute.html" in caplog.text


def test_ref_same_file_empty(caplog):
    with _mock_lint("check_path") as mocked_check_path:
        with _mock_lint("check_file_contents") as mocked_check_file_contents:
            rv = lint(_dummy_repo, ["ref/same_file_empty.html"], "normal", False)
            assert rv == 1
            assert mocked_check_path.call_count == 1
            assert mocked_check_file_contents.call_count == 1
    assert "SAME-FILE-REF" in caplog.text
    assert "same_file_empty.html" in caplog.text


def test_ref_same_file_path(caplog):
    with _mock_lint("check_path") as mocked_check_path:
        with _mock_lint("check_file_contents") as mocked_check_file_contents:
            rv = lint(_dummy_repo, ["ref/same_file_path.html"], "normal", False)
            assert rv == 1
            assert mocked_check_path.call_count == 1
            assert mocked_check_file_contents.call_count == 1
    assert "SAME-FILE-REF" in caplog.text
    assert "same_file_path.html" in caplog.text


def test_lint_passing_and_failing(caplog):
    with _mock_lint("check_path") as mocked_check_path:
        with _mock_lint("check_file_contents") as mocked_check_file_contents:
            rv = lint(_dummy_repo, ["broken.html", "okay.html"], "normal", False)
            assert rv == 1
            assert mocked_check_path.call_count == 2
            assert mocked_check_file_contents.call_count == 2
    assert "TRAILING WHITESPACE" in caplog.text
    assert "broken.html:1" in caplog.text
    assert "okay.html" not in caplog.text


def test_check_css_globally_unique_identical_test(caplog):
    with _mock_lint("check_path") as mocked_check_path:
        with _mock_lint("check_file_contents") as mocked_check_file_contents:
            rv = lint(_dummy_repo, ["css-unique/match/a.html", "css-unique/a.html"], "normal", True)
            assert rv == 0
            assert mocked_check_path.call_count == 2
            assert mocked_check_file_contents.call_count == 2
    assert caplog.text == ""


def test_check_css_globally_unique_different_test(caplog):
    with _mock_lint("check_path") as mocked_check_path:
        with _mock_lint("check_file_contents") as mocked_check_file_contents:
            rv = lint(_dummy_repo, ["css-unique/not-match/a.html", "css-unique/a.html"], "normal", True)
            assert rv == 2
            assert mocked_check_path.call_count == 2
            assert mocked_check_file_contents.call_count == 2
    assert "CSS-COLLIDING-TEST-NAME" in caplog.text


def test_check_css_globally_unique_different_spec_test(caplog):
    with _mock_lint("check_path") as mocked_check_path:
        with _mock_lint("check_file_contents") as mocked_check_file_contents:
            rv = lint(_dummy_repo, ["css-unique/selectors/a.html", "css-unique/a.html"], "normal", True)
            assert rv == 0
            assert mocked_check_path.call_count == 2
            assert mocked_check_file_contents.call_count == 2
    assert caplog.text == ""


def test_check_css_globally_unique_support_ignored(caplog):
    with _mock_lint("check_path") as mocked_check_path:
        with _mock_lint("check_file_contents") as mocked_check_file_contents:
            rv = lint(_dummy_repo, ["css-unique/support/a.html", "css-unique/support/tools/a.html"], "normal", True)
            assert rv == 0
            assert mocked_check_path.call_count == 2
            assert mocked_check_file_contents.call_count == 2
    assert caplog.text == ""


def test_check_css_globally_unique_support_identical(caplog):
    with _mock_lint("check_path") as mocked_check_path:
        with _mock_lint("check_file_contents") as mocked_check_file_contents:
            rv = lint(_dummy_repo, ["css-unique/support/a.html", "css-unique/match/support/a.html"], "normal", True)
            assert rv == 0
            assert mocked_check_path.call_count == 2
            assert mocked_check_file_contents.call_count == 2
    assert caplog.text == ""


def test_check_css_globally_unique_support_different(caplog):
    with _mock_lint("check_path") as mocked_check_path:
        with _mock_lint("check_file_contents") as mocked_check_file_contents:
            rv = lint(_dummy_repo, ["css-unique/not-match/support/a.html", "css-unique/support/a.html"], "normal", True)
            assert rv == 2
            assert mocked_check_path.call_count == 2
            assert mocked_check_file_contents.call_count == 2
    assert "CSS-COLLIDING-SUPPORT-NAME" in caplog.text


def test_check_css_globally_unique_test_support(caplog):
    with _mock_lint("check_path") as mocked_check_path:
        with _mock_lint("check_file_contents") as mocked_check_file_contents:
            rv = lint(_dummy_repo, ["css-unique/support/a.html", "css-unique/a.html"], "normal", True)
            assert rv == 0
            assert mocked_check_path.call_count == 2
            assert mocked_check_file_contents.call_count == 2
    assert caplog.text == ""


def test_check_css_globally_unique_ref_identical(caplog):
    with _mock_lint("check_path") as mocked_check_path:
        with _mock_lint("check_file_contents") as mocked_check_file_contents:
            rv = lint(_dummy_repo, ["css-unique/a-ref.html", "css-unique/match/a-ref.html"], "normal", True)
            assert rv == 0
            assert mocked_check_path.call_count == 2
            assert mocked_check_file_contents.call_count == 2
    assert caplog.text == ""


def test_check_css_globally_unique_ref_different(caplog):
    with _mock_lint("check_path") as mocked_check_path:
        with _mock_lint("check_file_contents") as mocked_check_file_contents:
            rv = lint(_dummy_repo, ["css-unique/not-match/a-ref.html", "css-unique/a-ref.html"], "normal", True)
            assert rv == 2
            assert mocked_check_path.call_count == 2
            assert mocked_check_file_contents.call_count == 2
    assert "CSS-COLLIDING-REF-NAME" in caplog.text


def test_check_css_globally_unique_test_ref(caplog):
    with _mock_lint("check_path") as mocked_check_path:
        with _mock_lint("check_file_contents") as mocked_check_file_contents:
            rv = lint(_dummy_repo, ["css-unique/a-ref.html", "css-unique/a.html"], "normal", True)
            assert rv == 0
            assert mocked_check_path.call_count == 2
            assert mocked_check_file_contents.call_count == 2
    assert caplog.text == ""


def test_check_css_globally_unique_ignored(caplog):
    with _mock_lint("check_path") as mocked_check_path:
        with _mock_lint("check_file_contents") as mocked_check_file_contents:
            rv = lint(_dummy_repo, ["css-unique/tools/a.html", "css-unique/not-match/tools/a.html"], "normal", True)
            assert rv == 0
            assert mocked_check_path.call_count == 2
            assert mocked_check_file_contents.call_count == 2
    assert caplog.text == ""


def test_check_css_globally_unique_ignored_dir(caplog):
    with _mock_lint("check_path") as mocked_check_path:
        with _mock_lint("check_file_contents") as mocked_check_file_contents:
            rv = lint(_dummy_repo, ["css-unique/support/a.html", "css/work-in-progress/foo/support/a.html"], "normal", True)
            assert rv == 0
            assert mocked_check_path.call_count == 2
            assert mocked_check_file_contents.call_count == 2
    assert caplog.text == ""


def test_all_filesystem_paths():
    with mock.patch(
            'os.walk',
            return_value=[('.',
                           ['dir_a', 'dir_b'],
                           ['file_a', 'file_b']),
                          (os.path.join('.', 'dir_a'),
                           [],
                           ['file_c', 'file_d'])]
    ) as m:
        got = list(lint_mod.all_filesystem_paths('.'))
        assert got == ['file_a',
                       'file_b',
                       os.path.join('dir_a', 'file_c'),
                       os.path.join('dir_a', 'file_d')]


def test_main_with_args():
    orig_argv = sys.argv
    try:
        sys.argv = ['./lint', 'a', 'b', 'c']
        with _mock_lint('lint', return_value=True) as m:
            lint_mod.main(**vars(parse_args()))
            m.assert_called_once_with(repo_root, ['a', 'b', 'c'], "normal", False)
    finally:
        sys.argv = orig_argv


def test_main_no_args():
    orig_argv = sys.argv
    try:
        sys.argv = ['./lint']
        with _mock_lint('lint', return_value=True) as m:
            with _mock_lint('all_filesystem_paths', return_value=['foo', 'bar']) as m2:
                lint_mod.main(**vars(parse_args()))
                m.assert_called_once_with(repo_root, ['foo', 'bar'], "normal", False)
    finally:
        sys.argv = orig_argv
