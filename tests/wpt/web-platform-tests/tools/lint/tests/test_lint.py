# mypy: allow-untyped-defs

import io
import os
import sys
from unittest import mock

from ...localpaths import repo_root
from .. import lint as lint_mod
from ..lint import filter_ignorelist_errors, parse_ignorelist, lint, create_parser

_dummy_repo = os.path.join(os.path.dirname(__file__), "dummy")

def _mock_lint(name, **kwargs):
    wrapped = getattr(lint_mod, name)
    return mock.patch(lint_mod.__name__ + "." + name, wraps=wrapped, **kwargs)


def test_filter_ignorelist_errors():
    ignorelist = {
        'CONSOLE': {
            'svg/*': {12}
        },
        'INDENT TABS': {
            'svg/*': {None}
        }
    }
    # parse_ignorelist normalises the case/path of the match string so need to do the same
    ignorelist = {e: {os.path.normcase(k): v for k, v in p.items()}
                 for e, p in ignorelist.items()}
    # paths passed into filter_ignorelist_errors are always Unix style
    filteredfile = 'svg/test.html'
    unfilteredfile = 'html/test.html'
    # Tests for passing no errors
    filtered = filter_ignorelist_errors(ignorelist, [])
    assert filtered == []
    filtered = filter_ignorelist_errors(ignorelist, [])
    assert filtered == []
    # Tests for filtering on file and line number
    filtered = filter_ignorelist_errors(ignorelist, [['CONSOLE', '', filteredfile, 12]])
    assert filtered == []
    filtered = filter_ignorelist_errors(ignorelist, [['CONSOLE', '', unfilteredfile, 12]])
    assert filtered == [['CONSOLE', '', unfilteredfile, 12]]
    filtered = filter_ignorelist_errors(ignorelist, [['CONSOLE', '', filteredfile, 11]])
    assert filtered == [['CONSOLE', '', filteredfile, 11]]
    # Tests for filtering on just file
    filtered = filter_ignorelist_errors(ignorelist, [['INDENT TABS', '', filteredfile, 12]])
    assert filtered == []
    filtered = filter_ignorelist_errors(ignorelist, [['INDENT TABS', '', filteredfile, 11]])
    assert filtered == []
    filtered = filter_ignorelist_errors(ignorelist, [['INDENT TABS', '', unfilteredfile, 11]])
    assert filtered == [['INDENT TABS', '', unfilteredfile, 11]]


def test_parse_ignorelist():
    input_buffer = io.StringIO("""
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
    expected_skipped = {os.path.normcase(x) for x in {"*.pdf", "resources/*", "*.png"}}
    data, skipped_files = parse_ignorelist(input_buffer)
    assert data == expected_data
    assert skipped_files == expected_skipped


def test_lint_no_files(caplog):
    rv = lint(_dummy_repo, [], "normal")
    assert rv == 0
    assert caplog.text == ""


def test_lint_ignored_file(caplog):
    with _mock_lint("check_path") as mocked_check_path:
        with _mock_lint("check_file_contents") as mocked_check_file_contents:
            rv = lint(_dummy_repo, ["broken_ignored.html"], "normal")
            assert rv == 0
            assert not mocked_check_path.called
            assert not mocked_check_file_contents.called
    assert caplog.text == ""


def test_lint_not_existing_file(caplog):
    with _mock_lint("check_path") as mocked_check_path:
        with _mock_lint("check_file_contents") as mocked_check_file_contents:
            # really long path-linted filename
            name = "a" * 256 + ".html"
            rv = lint(_dummy_repo, [name], "normal")
            assert rv == 0
            assert not mocked_check_path.called
            assert not mocked_check_file_contents.called
    assert caplog.text == ""


def test_lint_passing(caplog):
    with _mock_lint("check_path") as mocked_check_path:
        with _mock_lint("check_file_contents") as mocked_check_file_contents:
            rv = lint(_dummy_repo, ["okay.html"], "normal")
            assert rv == 0
            assert mocked_check_path.call_count == 1
            assert mocked_check_file_contents.call_count == 1
    assert caplog.text == ""


def test_lint_failing(caplog):
    with _mock_lint("check_path") as mocked_check_path:
        with _mock_lint("check_file_contents") as mocked_check_file_contents:
            rv = lint(_dummy_repo, ["broken.html"], "normal")
            assert rv == 1
            assert mocked_check_path.call_count == 1
            assert mocked_check_file_contents.call_count == 1
    assert "TRAILING WHITESPACE" in caplog.text
    assert "broken.html:1" in caplog.text


def test_ref_existent_relative(caplog):
    with _mock_lint("check_path") as mocked_check_path:
        with _mock_lint("check_file_contents") as mocked_check_file_contents:
            rv = lint(_dummy_repo, ["ref/existent_relative.html"], "normal")
            assert rv == 0
            assert mocked_check_path.call_count == 1
            assert mocked_check_file_contents.call_count == 1
    assert caplog.text == ""


def test_ref_existent_root_relative(caplog):
    with _mock_lint("check_path") as mocked_check_path:
        with _mock_lint("check_file_contents") as mocked_check_file_contents:
            rv = lint(_dummy_repo, ["ref/existent_root_relative.html"], "normal")
            assert rv == 0
            assert mocked_check_path.call_count == 1
            assert mocked_check_file_contents.call_count == 1
    assert caplog.text == ""


def test_ref_non_existent_relative(caplog):
    with _mock_lint("check_path") as mocked_check_path:
        with _mock_lint("check_file_contents") as mocked_check_file_contents:
            rv = lint(_dummy_repo, ["ref/non_existent_relative.html"], "normal")
            assert rv == 1
            assert mocked_check_path.call_count == 1
            assert mocked_check_file_contents.call_count == 1
    assert "NON-EXISTENT-REF" in caplog.text
    assert "ref/non_existent_relative.html" in caplog.text
    assert "non_existent_file.html" in caplog.text


def test_ref_non_existent_root_relative(caplog):
    with _mock_lint("check_path") as mocked_check_path:
        with _mock_lint("check_file_contents") as mocked_check_file_contents:
            rv = lint(_dummy_repo, ["ref/non_existent_root_relative.html"], "normal")
            assert rv == 1
            assert mocked_check_path.call_count == 1
            assert mocked_check_file_contents.call_count == 1
    assert "NON-EXISTENT-REF" in caplog.text
    assert "ref/non_existent_root_relative.html" in caplog.text
    assert "/non_existent_file.html" in caplog.text



def test_ref_absolute_url(caplog):
    with _mock_lint("check_path") as mocked_check_path:
        with _mock_lint("check_file_contents") as mocked_check_file_contents:
            rv = lint(_dummy_repo, ["ref/absolute.html"], "normal")
            assert rv == 1
            assert mocked_check_path.call_count == 1
            assert mocked_check_file_contents.call_count == 1
    assert "ABSOLUTE-URL-REF" in caplog.text
    assert "http://example.com/reference.html" in caplog.text
    assert "ref/absolute.html" in caplog.text


def test_about_blank_as_ref(caplog):
    with _mock_lint("check_path"):
        with _mock_lint("check_file_contents") as mocked_check_file_contents:
            rv = lint(_dummy_repo, ["about_blank.html"], "normal")
            assert rv == 0
            assert mocked_check_file_contents.call_count == 1
    assert caplog.text == ""


def test_ref_same_file_empty(caplog):
    with _mock_lint("check_path") as mocked_check_path:
        with _mock_lint("check_file_contents") as mocked_check_file_contents:
            rv = lint(_dummy_repo, ["ref/same_file_empty.html"], "normal")
            assert rv == 1
            assert mocked_check_path.call_count == 1
            assert mocked_check_file_contents.call_count == 1
    assert "SAME-FILE-REF" in caplog.text
    assert "same_file_empty.html" in caplog.text


def test_ref_same_file_path(caplog):
    with _mock_lint("check_path") as mocked_check_path:
        with _mock_lint("check_file_contents") as mocked_check_file_contents:
            rv = lint(_dummy_repo, ["ref/same_file_path.html"], "normal")
            assert rv == 1
            assert mocked_check_path.call_count == 1
            assert mocked_check_file_contents.call_count == 1
    assert "SAME-FILE-REF" in caplog.text
    assert "same_file_path.html" in caplog.text


def test_manual_path_testharness(caplog):
    rv = lint(_dummy_repo, ["tests/relative-testharness-manual.html"], "normal")
    assert rv == 2
    assert "TESTHARNESS-PATH" in caplog.text
    assert "TESTHARNESSREPORT-PATH" in caplog.text


def test_css_visual_path_testharness(caplog):
    rv = lint(_dummy_repo, ["css/css-unique/relative-testharness.html"], "normal")
    assert rv == 3
    assert "CONTENT-VISUAL" in caplog.text
    assert "TESTHARNESS-PATH" in caplog.text
    assert "TESTHARNESSREPORT-PATH" in caplog.text


def test_css_manual_path_testharness(caplog):
    rv = lint(_dummy_repo, ["css/css-unique/relative-testharness-interact.html"], "normal")
    assert rv == 3
    assert "CONTENT-MANUAL" in caplog.text
    assert "TESTHARNESS-PATH" in caplog.text
    assert "TESTHARNESSREPORT-PATH" in caplog.text


def test_lint_passing_and_failing(caplog):
    with _mock_lint("check_path") as mocked_check_path:
        with _mock_lint("check_file_contents") as mocked_check_file_contents:
            rv = lint(_dummy_repo, ["broken.html", "okay.html"], "normal")
            assert rv == 1
            assert mocked_check_path.call_count == 2
            assert mocked_check_file_contents.call_count == 2
    assert "TRAILING WHITESPACE" in caplog.text
    assert "broken.html:1" in caplog.text
    assert "okay.html" not in caplog.text


def test_check_css_globally_unique_identical_test(caplog):
    with _mock_lint("check_path") as mocked_check_path:
        with _mock_lint("check_file_contents") as mocked_check_file_contents:
            rv = lint(_dummy_repo, ["css/css-unique/match/a.html", "css/css-unique/a.html"], "normal")
            assert rv == 0
            assert mocked_check_path.call_count == 2
            assert mocked_check_file_contents.call_count == 2
    assert caplog.text == ""


def test_check_css_globally_unique_different_test(caplog):
    with _mock_lint("check_path") as mocked_check_path:
        with _mock_lint("check_file_contents") as mocked_check_file_contents:
            rv = lint(_dummy_repo, ["css/css-unique/not-match/a.html", "css/css-unique/a.html"], "normal")
            assert rv == 2
            assert mocked_check_path.call_count == 2
            assert mocked_check_file_contents.call_count == 2
    assert "CSS-COLLIDING-TEST-NAME" in caplog.text


def test_check_css_globally_unique_different_spec_test(caplog):
    with _mock_lint("check_path") as mocked_check_path:
        with _mock_lint("check_file_contents") as mocked_check_file_contents:
            rv = lint(_dummy_repo, ["css/css-unique/selectors/a.html", "css/css-unique/a.html"], "normal")
            assert rv == 0
            assert mocked_check_path.call_count == 2
            assert mocked_check_file_contents.call_count == 2
    assert caplog.text == ""


def test_check_css_globally_unique_support_ignored(caplog):
    with _mock_lint("check_path") as mocked_check_path:
        with _mock_lint("check_file_contents") as mocked_check_file_contents:
            rv = lint(_dummy_repo, ["css/css-unique/support/a.html", "css/css-unique/support/tools/a.html"], "normal")
            assert rv == 0
            assert mocked_check_path.call_count == 2
            assert mocked_check_file_contents.call_count == 2
    assert caplog.text == ""


def test_check_css_globally_unique_support_identical(caplog):
    with _mock_lint("check_path") as mocked_check_path:
        with _mock_lint("check_file_contents") as mocked_check_file_contents:
            rv = lint(_dummy_repo, ["css/css-unique/support/a.html", "css/css-unique/match/support/a.html"], "normal")
            assert rv == 0
            assert mocked_check_path.call_count == 2
            assert mocked_check_file_contents.call_count == 2
    assert caplog.text == ""


def test_check_css_globally_unique_support_different(caplog):
    with _mock_lint("check_path") as mocked_check_path:
        with _mock_lint("check_file_contents") as mocked_check_file_contents:
            rv = lint(_dummy_repo, ["css/css-unique/not-match/support/a.html", "css/css-unique/support/a.html"], "normal")
            assert rv == 2
            assert mocked_check_path.call_count == 2
            assert mocked_check_file_contents.call_count == 2
    assert "CSS-COLLIDING-SUPPORT-NAME" in caplog.text


def test_check_css_globally_unique_test_support(caplog):
    with _mock_lint("check_path") as mocked_check_path:
        with _mock_lint("check_file_contents") as mocked_check_file_contents:
            rv = lint(_dummy_repo, ["css/css-unique/support/a.html", "css/css-unique/a.html"], "normal")
            assert rv == 0
            assert mocked_check_path.call_count == 2
            assert mocked_check_file_contents.call_count == 2
    assert caplog.text == ""


def test_check_css_globally_unique_ref_identical(caplog):
    with _mock_lint("check_path") as mocked_check_path:
        with _mock_lint("check_file_contents") as mocked_check_file_contents:
            rv = lint(_dummy_repo, ["css/css-unique/a-ref.html", "css/css-unique/match/a-ref.html"], "normal")
            assert rv == 0
            assert mocked_check_path.call_count == 2
            assert mocked_check_file_contents.call_count == 2
    assert caplog.text == ""


def test_check_css_globally_unique_ref_different(caplog):
    with _mock_lint("check_path") as mocked_check_path:
        with _mock_lint("check_file_contents") as mocked_check_file_contents:
            rv = lint(_dummy_repo, ["css/css-unique/not-match/a-ref.html", "css/css-unique/a-ref.html"], "normal")
            assert rv == 2
            assert mocked_check_path.call_count == 2
            assert mocked_check_file_contents.call_count == 2
    assert "CSS-COLLIDING-REF-NAME" in caplog.text


def test_check_css_globally_unique_test_ref(caplog):
    with _mock_lint("check_path") as mocked_check_path:
        with _mock_lint("check_file_contents") as mocked_check_file_contents:
            rv = lint(_dummy_repo, ["css/css-unique/a-ref.html", "css/css-unique/a.html"], "normal")
            assert rv == 0
            assert mocked_check_path.call_count == 2
            assert mocked_check_file_contents.call_count == 2
    assert caplog.text == ""


def test_check_css_globally_unique_ignored(caplog):
    with _mock_lint("check_path") as mocked_check_path:
        with _mock_lint("check_file_contents") as mocked_check_file_contents:
            rv = lint(_dummy_repo, ["css/css-unique/tools/a.html", "css/css-unique/not-match/tools/a.html"], "normal")
            assert rv == 0
            assert mocked_check_path.call_count == 2
            assert mocked_check_file_contents.call_count == 2
    assert caplog.text == ""


def test_check_css_globally_unique_ignored_dir(caplog):
    with _mock_lint("check_path") as mocked_check_path:
        with _mock_lint("check_file_contents") as mocked_check_file_contents:
            rv = lint(_dummy_repo, ["css/css-unique/support/a.html"], "normal")
            assert rv == 0
            assert mocked_check_path.call_count == 1
            assert mocked_check_file_contents.call_count == 1
    assert caplog.text == ""


def test_check_unique_testharness_basename_same_basename(caplog):
    # Precondition: There are testharness files with conflicting basename paths.
    assert os.path.exists(os.path.join(_dummy_repo, 'tests', 'dir1', 'a.html'))
    assert os.path.exists(os.path.join(_dummy_repo, 'tests', 'dir1', 'a.xhtml'))

    with _mock_lint("check_path") as mocked_check_path:
        with _mock_lint("check_file_contents") as mocked_check_file_contents:
            rv = lint(_dummy_repo, ["tests/dir1/a.html", "tests/dir1/a.xhtml"], "normal")
            # There will be one failure for each file.
            assert rv == 2
            assert mocked_check_path.call_count == 2
            assert mocked_check_file_contents.call_count == 2
    assert "DUPLICATE-BASENAME-PATH" in caplog.text


def test_check_unique_testharness_basename_different_name(caplog):
    # Precondition: There are two testharness files in the same directory with
    # different names.
    assert os.path.exists(os.path.join(_dummy_repo, 'tests', 'dir1', 'a.html'))
    assert os.path.exists(os.path.join(_dummy_repo, 'tests', 'dir1', 'b.html'))

    with _mock_lint("check_path") as mocked_check_path:
        with _mock_lint("check_file_contents") as mocked_check_file_contents:
            rv = lint(_dummy_repo, ["tests/dir1/a.html", "tests/dir1/b.html"], "normal")
            assert rv == 0
            assert mocked_check_path.call_count == 2
            assert mocked_check_file_contents.call_count == 2
    assert caplog.text == ""


def test_check_unique_testharness_basename_different_dir(caplog):
    # Precondition: There are two testharness files in different directories
    # with the same basename.
    assert os.path.exists(os.path.join(_dummy_repo, 'tests', 'dir1', 'a.html'))
    assert os.path.exists(os.path.join(_dummy_repo, 'tests', 'dir2', 'a.xhtml'))

    with _mock_lint("check_path") as mocked_check_path:
        with _mock_lint("check_file_contents") as mocked_check_file_contents:
            rv = lint(_dummy_repo, ["tests/dir1/a.html", "tests/dir2/a.xhtml"], "normal")
            assert rv == 0
            assert mocked_check_path.call_count == 2
            assert mocked_check_file_contents.call_count == 2
    assert caplog.text == ""


def test_check_unique_testharness_basename_not_testharness(caplog):
    # Precondition: There are non-testharness files with conflicting basename paths.
    assert os.path.exists(os.path.join(_dummy_repo, 'tests', 'dir1', 'a.html'))
    assert os.path.exists(os.path.join(_dummy_repo, 'tests', 'dir1', 'a.js'))

    with _mock_lint("check_path") as mocked_check_path:
        with _mock_lint("check_file_contents") as mocked_check_file_contents:
            rv = lint(_dummy_repo, ["tests/dir1/a.html", "tests/dir1/a.js"], "normal")
            assert rv == 0
            assert mocked_check_path.call_count == 2
            assert mocked_check_file_contents.call_count == 2
    assert caplog.text == ""


def test_ignore_glob(caplog):
    # Lint two files in the ref/ directory, and pass in ignore_glob to omit one
    # of them.
    # When we omit absolute.html, no lint errors appear since the other file is
    # clean.
    with _mock_lint("check_path") as mocked_check_path:
        with _mock_lint("check_file_contents") as mocked_check_file_contents:
            rv = lint(_dummy_repo,
                      ["broken.html", "ref/absolute.html", "ref/existent_relative.html"],
                      "normal",
                      ["broken*", "*solu*"])
            assert rv == 0
            # Also confirm that only one file is checked
            assert mocked_check_path.call_count == 1
            assert mocked_check_file_contents.call_count == 1
            assert caplog.text == ""
    # However, linting the same two files without ignore_glob yields lint errors.
    with _mock_lint("check_path") as mocked_check_path:
        with _mock_lint("check_file_contents") as mocked_check_file_contents:
            rv = lint(_dummy_repo, ["broken.html", "ref/absolute.html", "ref/existent_relative.html"], "normal")
            assert rv == 2
            assert mocked_check_path.call_count == 3
            assert mocked_check_file_contents.call_count == 3
            assert "TRAILING WHITESPACE" in caplog.text
            assert "ABSOLUTE-URL-REF" in caplog.text


def test_all_filesystem_paths():
    with mock.patch(
            'tools.lint.lint.walk',
            return_value=[(b'',
                           [(b'dir_a', None), (b'dir_b', None)],
                           [(b'file_a', None), (b'file_b', None)]),
                          (b'dir_a',
                           [],
                           [(b'file_c', None), (b'file_d', None)])]
    ):
        got = list(lint_mod.all_filesystem_paths('.'))
        assert got == ['file_a',
                       'file_b',
                       os.path.join('dir_a', 'file_c'),
                       os.path.join('dir_a', 'file_d')]


def test_filesystem_paths_subdir():
    with mock.patch(
            'tools.lint.lint.walk',
            return_value=[(b'',
                           [(b'dir_a', None), (b'dir_b', None)],
                           [(b'file_a', None), (b'file_b', None)]),
                          (b'dir_a',
                           [],
                           [(b'file_c', None), (b'file_d', None)])]
    ):
        got = list(lint_mod.all_filesystem_paths('.', 'dir'))
        assert got == [os.path.join('dir', 'file_a'),
                       os.path.join('dir', 'file_b'),
                       os.path.join('dir', 'dir_a', 'file_c'),
                       os.path.join('dir', 'dir_a', 'file_d')]


def test_main_with_args():
    orig_argv = sys.argv
    try:
        sys.argv = ['./lint', 'a', 'b', 'c']
        with mock.patch(lint_mod.__name__ + ".os.path.isfile") as mock_isfile:
            mock_isfile.return_value = True
            with _mock_lint('lint', return_value=True) as m:
                lint_mod.main(**vars(create_parser().parse_args()))
                m.assert_called_once_with(repo_root,
                                          [os.path.relpath(os.path.join(os.getcwd(), x), repo_root)
                                           for x in ['a', 'b', 'c']],
                                          "normal",
                                          None,
                                          None,
                                          0)
    finally:
        sys.argv = orig_argv


def test_main_no_args():
    orig_argv = sys.argv
    try:
        sys.argv = ['./lint']
        with _mock_lint('lint', return_value=True) as m:
            with _mock_lint('changed_files', return_value=['foo', 'bar']):
                lint_mod.main(**vars(create_parser().parse_args()))
                m.assert_called_once_with(repo_root, ['foo', 'bar'], "normal", None, None, 0)
    finally:
        sys.argv = orig_argv


def test_main_all():
    orig_argv = sys.argv
    try:
        sys.argv = ['./lint', '--all']
        with _mock_lint('lint', return_value=True) as m:
            with _mock_lint('all_filesystem_paths', return_value=['foo', 'bar']):
                lint_mod.main(**vars(create_parser().parse_args()))
                m.assert_called_once_with(repo_root, ['foo', 'bar'], "normal", None, None, 0)
    finally:
        sys.argv = orig_argv
