# -*- coding: utf-8 -*-
from __future__ import absolute_import, division, print_function
import sys
import textwrap

import _pytest.assertion as plugin
import py
import pytest
from _pytest.assertion import util
from _pytest.assertion import truncate

PY3 = sys.version_info >= (3, 0)


@pytest.fixture
def mock_config():

    class Config(object):
        verbose = False

        def getoption(self, name):
            if name == "verbose":
                return self.verbose
            raise KeyError("Not mocked out: %s" % name)

    return Config()


class TestImportHookInstallation(object):

    @pytest.mark.parametrize("initial_conftest", [True, False])
    @pytest.mark.parametrize("mode", ["plain", "rewrite"])
    def test_conftest_assertion_rewrite(self, testdir, initial_conftest, mode):
        """Test that conftest files are using assertion rewrite on import.
        (#1619)
        """
        testdir.tmpdir.join("foo/tests").ensure(dir=1)
        conftest_path = "conftest.py" if initial_conftest else "foo/conftest.py"
        contents = {
            conftest_path: """
                import pytest
                @pytest.fixture
                def check_first():
                    def check(values, value):
                        assert values.pop(0) == value
                    return check
            """,
            "foo/tests/test_foo.py": """
                def test(check_first):
                    check_first([10, 30], 30)
            """,
        }
        testdir.makepyfile(**contents)
        result = testdir.runpytest_subprocess("--assert=%s" % mode)
        if mode == "plain":
            expected = "E       AssertionError"
        elif mode == "rewrite":
            expected = "*assert 10 == 30*"
        else:
            assert 0
        result.stdout.fnmatch_lines([expected])

    def test_rewrite_assertions_pytester_plugin(self, testdir):
        """
        Assertions in the pytester plugin must also benefit from assertion
        rewriting (#1920).
        """
        testdir.makepyfile(
            """
            pytest_plugins = ['pytester']
            def test_dummy_failure(testdir):  # how meta!
                testdir.makepyfile('def test(): assert 0')
                r = testdir.inline_run()
                r.assertoutcome(passed=1)
        """
        )
        result = testdir.runpytest_subprocess()
        result.stdout.fnmatch_lines(["*assert 1 == 0*"])

    @pytest.mark.parametrize("mode", ["plain", "rewrite"])
    def test_pytest_plugins_rewrite(self, testdir, mode):
        contents = {
            "conftest.py": """
                pytest_plugins = ['ham']
            """,
            "ham.py": """
                import pytest
                @pytest.fixture
                def check_first():
                    def check(values, value):
                        assert values.pop(0) == value
                    return check
            """,
            "test_foo.py": """
                def test_foo(check_first):
                    check_first([10, 30], 30)
            """,
        }
        testdir.makepyfile(**contents)
        result = testdir.runpytest_subprocess("--assert=%s" % mode)
        if mode == "plain":
            expected = "E       AssertionError"
        elif mode == "rewrite":
            expected = "*assert 10 == 30*"
        else:
            assert 0
        result.stdout.fnmatch_lines([expected])

    @pytest.mark.parametrize("mode", ["str", "list"])
    def test_pytest_plugins_rewrite_module_names(self, testdir, mode):
        """Test that pluginmanager correct marks pytest_plugins variables
        for assertion rewriting if they are defined as plain strings or
        list of strings (#1888).
        """
        plugins = '"ham"' if mode == "str" else '["ham"]'
        contents = {
            "conftest.py": """
                pytest_plugins = {plugins}
            """.format(
                plugins=plugins
            ),
            "ham.py": """
                import pytest
            """,
            "test_foo.py": """
                def test_foo(pytestconfig):
                    assert 'ham' in pytestconfig.pluginmanager.rewrite_hook._must_rewrite
            """,
        }
        testdir.makepyfile(**contents)
        result = testdir.runpytest_subprocess("--assert=rewrite")
        assert result.ret == 0

    def test_pytest_plugins_rewrite_module_names_correctly(self, testdir):
        """Test that we match files correctly when they are marked for rewriting (#2939)."""
        contents = {
            "conftest.py": """
                pytest_plugins = "ham"
            """,
            "ham.py": "",
            "hamster.py": "",
            "test_foo.py": """
                def test_foo(pytestconfig):
                    assert pytestconfig.pluginmanager.rewrite_hook.find_module('ham') is not None
                    assert pytestconfig.pluginmanager.rewrite_hook.find_module('hamster') is None
            """,
        }
        testdir.makepyfile(**contents)
        result = testdir.runpytest_subprocess("--assert=rewrite")
        assert result.ret == 0

    @pytest.mark.parametrize("mode", ["plain", "rewrite"])
    @pytest.mark.parametrize("plugin_state", ["development", "installed"])
    def test_installed_plugin_rewrite(self, testdir, mode, plugin_state):
        # Make sure the hook is installed early enough so that plugins
        # installed via setuptools are rewritten.
        testdir.tmpdir.join("hampkg").ensure(dir=1)
        contents = {
            "hampkg/__init__.py": """
                import pytest

                @pytest.fixture
                def check_first2():
                    def check(values, value):
                        assert values.pop(0) == value
                    return check
            """,
            "spamplugin.py": """
            import pytest
            from hampkg import check_first2

            @pytest.fixture
            def check_first():
                def check(values, value):
                    assert values.pop(0) == value
                return check
            """,
            "mainwrapper.py": """
            import pytest, pkg_resources

            plugin_state = "{plugin_state}"

            class DummyDistInfo(object):
                project_name = 'spam'
                version = '1.0'

                def _get_metadata(self, name):
                    # 'RECORD' meta-data only available in installed plugins
                    if name == 'RECORD' and plugin_state == "installed":
                        return ['spamplugin.py,sha256=abc,123',
                                'hampkg/__init__.py,sha256=abc,123']
                    # 'SOURCES.txt' meta-data only available for plugins in development mode
                    elif name == 'SOURCES.txt' and plugin_state == "development":
                        return ['spamplugin.py',
                                'hampkg/__init__.py']
                    return []

            class DummyEntryPoint(object):
                name = 'spam'
                module_name = 'spam.py'
                attrs = ()
                extras = None
                dist = DummyDistInfo()

                def load(self, require=True, *args, **kwargs):
                    import spamplugin
                    return spamplugin

            def iter_entry_points(name):
                yield DummyEntryPoint()

            pkg_resources.iter_entry_points = iter_entry_points
            pytest.main()
            """.format(
                plugin_state=plugin_state
            ),
            "test_foo.py": """
            def test(check_first):
                check_first([10, 30], 30)

            def test2(check_first2):
                check_first([10, 30], 30)
            """,
        }
        testdir.makepyfile(**contents)
        result = testdir.run(
            sys.executable, "mainwrapper.py", "-s", "--assert=%s" % mode
        )
        if mode == "plain":
            expected = "E       AssertionError"
        elif mode == "rewrite":
            expected = "*assert 10 == 30*"
        else:
            assert 0
        result.stdout.fnmatch_lines([expected])

    def test_rewrite_ast(self, testdir):
        testdir.tmpdir.join("pkg").ensure(dir=1)
        contents = {
            "pkg/__init__.py": """
                import pytest
                pytest.register_assert_rewrite('pkg.helper')
            """,
            "pkg/helper.py": """
                def tool():
                    a, b = 2, 3
                    assert a == b
            """,
            "pkg/plugin.py": """
                import pytest, pkg.helper
                @pytest.fixture
                def tool():
                    return pkg.helper.tool
            """,
            "pkg/other.py": """
                values = [3, 2]
                def tool():
                    assert values.pop() == 3
            """,
            "conftest.py": """
                pytest_plugins = ['pkg.plugin']
            """,
            "test_pkg.py": """
                import pkg.other
                def test_tool(tool):
                    tool()
                def test_other():
                    pkg.other.tool()
            """,
        }
        testdir.makepyfile(**contents)
        result = testdir.runpytest_subprocess("--assert=rewrite")
        result.stdout.fnmatch_lines(
            [
                ">*assert a == b*",
                "E*assert 2 == 3*",
                ">*assert values.pop() == 3*",
                "E*AssertionError",
            ]
        )

    def test_register_assert_rewrite_checks_types(self):
        with pytest.raises(TypeError):
            pytest.register_assert_rewrite(["pytest_tests_internal_non_existing"])
        pytest.register_assert_rewrite(
            "pytest_tests_internal_non_existing", "pytest_tests_internal_non_existing2"
        )


class TestBinReprIntegration(object):

    def test_pytest_assertrepr_compare_called(self, testdir):
        testdir.makeconftest(
            """
            import pytest
            values = []
            def pytest_assertrepr_compare(op, left, right):
                values.append((op, left, right))

            @pytest.fixture
            def list(request):
                return values
        """
        )
        testdir.makepyfile(
            """
            def test_hello():
                assert 0 == 1
            def test_check(list):
                assert list == [("==", 0, 1)]
        """
        )
        result = testdir.runpytest("-v")
        result.stdout.fnmatch_lines(["*test_hello*FAIL*", "*test_check*PASS*"])


def callequal(left, right, verbose=False):
    config = mock_config()
    config.verbose = verbose
    return plugin.pytest_assertrepr_compare(config, "==", left, right)


class TestAssert_reprcompare(object):

    def test_different_types(self):
        assert callequal([0, 1], "foo") is None

    def test_summary(self):
        summary = callequal([0, 1], [0, 2])[0]
        assert len(summary) < 65

    def test_text_diff(self):
        diff = callequal("spam", "eggs")[1:]
        assert "- spam" in diff
        assert "+ eggs" in diff

    def test_text_skipping(self):
        lines = callequal("a" * 50 + "spam", "a" * 50 + "eggs")
        assert "Skipping" in lines[1]
        for line in lines:
            assert "a" * 50 not in line

    def test_text_skipping_verbose(self):
        lines = callequal("a" * 50 + "spam", "a" * 50 + "eggs", verbose=True)
        assert "- " + "a" * 50 + "spam" in lines
        assert "+ " + "a" * 50 + "eggs" in lines

    def test_multiline_text_diff(self):
        left = "foo\nspam\nbar"
        right = "foo\neggs\nbar"
        diff = callequal(left, right)
        assert "- spam" in diff
        assert "+ eggs" in diff

    def test_list(self):
        expl = callequal([0, 1], [0, 2])
        assert len(expl) > 1

    @pytest.mark.parametrize(
        ["left", "right", "expected"],
        [
            (
                [0, 1],
                [0, 2],
                """
                Full diff:
                - [0, 1]
                ?     ^
                + [0, 2]
                ?     ^
            """,
            ),
            (
                {0: 1},
                {0: 2},
                """
                Full diff:
                - {0: 1}
                ?     ^
                + {0: 2}
                ?     ^
            """,
            ),
            (
                {0, 1},
                {0, 2},
                """
                Full diff:
                - set([0, 1])
                ?         ^
                + set([0, 2])
                ?         ^
            """
                if not PY3
                else """
                Full diff:
                - {0, 1}
                ?     ^
                + {0, 2}
                ?     ^
            """,
            ),
        ],
    )
    def test_iterable_full_diff(self, left, right, expected):
        """Test the full diff assertion failure explanation.

        When verbose is False, then just a -v notice to get the diff is rendered,
        when verbose is True, then ndiff of the pprint is returned.
        """
        expl = callequal(left, right, verbose=False)
        assert expl[-1] == "Use -v to get the full diff"
        expl = "\n".join(callequal(left, right, verbose=True))
        assert expl.endswith(textwrap.dedent(expected).strip())

    def test_list_different_lengths(self):
        expl = callequal([0, 1], [0, 1, 2])
        assert len(expl) > 1
        expl = callequal([0, 1, 2], [0, 1])
        assert len(expl) > 1

    def test_dict(self):
        expl = callequal({"a": 0}, {"a": 1})
        assert len(expl) > 1

    def test_dict_omitting(self):
        lines = callequal({"a": 0, "b": 1}, {"a": 1, "b": 1})
        assert lines[1].startswith("Omitting 1 identical item")
        assert "Common items" not in lines
        for line in lines[1:]:
            assert "b" not in line

    def test_dict_omitting_with_verbosity_1(self):
        """ Ensure differing items are visible for verbosity=1 (#1512) """
        lines = callequal({"a": 0, "b": 1}, {"a": 1, "b": 1}, verbose=1)
        assert lines[1].startswith("Omitting 1 identical item")
        assert lines[2].startswith("Differing items")
        assert lines[3] == "{'a': 0} != {'a': 1}"
        assert "Common items" not in lines

    def test_dict_omitting_with_verbosity_2(self):
        lines = callequal({"a": 0, "b": 1}, {"a": 1, "b": 1}, verbose=2)
        assert lines[1].startswith("Common items:")
        assert "Omitting" not in lines[1]
        assert lines[2] == "{'b': 1}"

    def test_set(self):
        expl = callequal({0, 1}, {0, 2})
        assert len(expl) > 1

    def test_frozenzet(self):
        expl = callequal(frozenset([0, 1]), {0, 2})
        assert len(expl) > 1

    def test_Sequence(self):
        col = py.builtin._tryimport("collections.abc", "collections", "sys")
        if not hasattr(col, "MutableSequence"):
            pytest.skip("cannot import MutableSequence")
        MutableSequence = col.MutableSequence

        class TestSequence(MutableSequence):  # works with a Sequence subclass

            def __init__(self, iterable):
                self.elements = list(iterable)

            def __getitem__(self, item):
                return self.elements[item]

            def __len__(self):
                return len(self.elements)

            def __setitem__(self, item, value):
                pass

            def __delitem__(self, item):
                pass

            def insert(self, item, index):
                pass

        expl = callequal(TestSequence([0, 1]), list([0, 2]))
        assert len(expl) > 1

    def test_list_tuples(self):
        expl = callequal([], [(1, 2)])
        assert len(expl) > 1
        expl = callequal([(1, 2)], [])
        assert len(expl) > 1

    def test_list_bad_repr(self):

        class A(object):

            def __repr__(self):
                raise ValueError(42)

        expl = callequal([], [A()])
        assert "ValueError" in "".join(expl)
        expl = callequal({}, {"1": A()})
        assert "faulty" in "".join(expl)

    def test_one_repr_empty(self):
        """
        the faulty empty string repr did trigger
        an unbound local error in _diff_text
        """

        class A(str):

            def __repr__(self):
                return ""

        expl = callequal(A(), "")
        assert not expl

    def test_repr_no_exc(self):
        expl = " ".join(callequal("foo", "bar"))
        assert "raised in repr()" not in expl

    def test_unicode(self):
        left = py.builtin._totext("£€", "utf-8")
        right = py.builtin._totext("£", "utf-8")
        expl = callequal(left, right)
        assert expl[0] == py.builtin._totext("'£€' == '£'", "utf-8")
        assert expl[1] == py.builtin._totext("- £€", "utf-8")
        assert expl[2] == py.builtin._totext("+ £", "utf-8")

    def test_nonascii_text(self):
        """
        :issue: 877
        non ascii python2 str caused a UnicodeDecodeError
        """

        class A(str):

            def __repr__(self):
                return "\xff"

        expl = callequal(A(), "1")
        assert expl

    def test_format_nonascii_explanation(self):
        assert util.format_explanation("λ")

    def test_mojibake(self):
        # issue 429
        left = "e"
        right = "\xc3\xa9"
        if not isinstance(left, bytes):
            left = bytes(left, "utf-8")
            right = bytes(right, "utf-8")
        expl = callequal(left, right)
        for line in expl:
            assert isinstance(line, py.builtin.text)
        msg = py.builtin._totext("\n").join(expl)
        assert msg


class TestFormatExplanation(object):

    def test_special_chars_full(self, testdir):
        # Issue 453, for the bug this would raise IndexError
        testdir.makepyfile(
            """
            def test_foo():
                assert '\\n}' == ''
        """
        )
        result = testdir.runpytest()
        assert result.ret == 1
        result.stdout.fnmatch_lines(["*AssertionError*"])

    def test_fmt_simple(self):
        expl = "assert foo"
        assert util.format_explanation(expl) == "assert foo"

    def test_fmt_where(self):
        expl = "\n".join(["assert 1", "{1 = foo", "} == 2"])
        res = "\n".join(["assert 1 == 2", " +  where 1 = foo"])
        assert util.format_explanation(expl) == res

    def test_fmt_and(self):
        expl = "\n".join(["assert 1", "{1 = foo", "} == 2", "{2 = bar", "}"])
        res = "\n".join(["assert 1 == 2", " +  where 1 = foo", " +  and   2 = bar"])
        assert util.format_explanation(expl) == res

    def test_fmt_where_nested(self):
        expl = "\n".join(["assert 1", "{1 = foo", "{foo = bar", "}", "} == 2"])
        res = "\n".join(["assert 1 == 2", " +  where 1 = foo", " +    where foo = bar"])
        assert util.format_explanation(expl) == res

    def test_fmt_newline(self):
        expl = "\n".join(['assert "foo" == "bar"', "~- foo", "~+ bar"])
        res = "\n".join(['assert "foo" == "bar"', "  - foo", "  + bar"])
        assert util.format_explanation(expl) == res

    def test_fmt_newline_escaped(self):
        expl = "\n".join(["assert foo == bar", "baz"])
        res = "assert foo == bar\\nbaz"
        assert util.format_explanation(expl) == res

    def test_fmt_newline_before_where(self):
        expl = "\n".join(
            [
                "the assertion message here",
                ">assert 1",
                "{1 = foo",
                "} == 2",
                "{2 = bar",
                "}",
            ]
        )
        res = "\n".join(
            [
                "the assertion message here",
                "assert 1 == 2",
                " +  where 1 = foo",
                " +  and   2 = bar",
            ]
        )
        assert util.format_explanation(expl) == res

    def test_fmt_multi_newline_before_where(self):
        expl = "\n".join(
            [
                "the assertion",
                "~message here",
                ">assert 1",
                "{1 = foo",
                "} == 2",
                "{2 = bar",
                "}",
            ]
        )
        res = "\n".join(
            [
                "the assertion",
                "  message here",
                "assert 1 == 2",
                " +  where 1 = foo",
                " +  and   2 = bar",
            ]
        )
        assert util.format_explanation(expl) == res


class TestTruncateExplanation(object):

    """ Confirm assertion output is truncated as expected """

    # The number of lines in the truncation explanation message. Used
    # to calculate that results have the expected length.
    LINES_IN_TRUNCATION_MSG = 2

    def test_doesnt_truncate_when_input_is_empty_list(self):
        expl = []
        result = truncate._truncate_explanation(expl, max_lines=8, max_chars=100)
        assert result == expl

    def test_doesnt_truncate_at_when_input_is_5_lines_and_LT_max_chars(self):
        expl = ["a" * 100 for x in range(5)]
        result = truncate._truncate_explanation(expl, max_lines=8, max_chars=8 * 80)
        assert result == expl

    def test_truncates_at_8_lines_when_given_list_of_empty_strings(self):
        expl = ["" for x in range(50)]
        result = truncate._truncate_explanation(expl, max_lines=8, max_chars=100)
        assert result != expl
        assert len(result) == 8 + self.LINES_IN_TRUNCATION_MSG
        assert "Full output truncated" in result[-1]
        assert "43 lines hidden" in result[-1]
        last_line_before_trunc_msg = result[-self.LINES_IN_TRUNCATION_MSG - 1]
        assert last_line_before_trunc_msg.endswith("...")

    def test_truncates_at_8_lines_when_first_8_lines_are_LT_max_chars(self):
        expl = ["a" for x in range(100)]
        result = truncate._truncate_explanation(expl, max_lines=8, max_chars=8 * 80)
        assert result != expl
        assert len(result) == 8 + self.LINES_IN_TRUNCATION_MSG
        assert "Full output truncated" in result[-1]
        assert "93 lines hidden" in result[-1]
        last_line_before_trunc_msg = result[-self.LINES_IN_TRUNCATION_MSG - 1]
        assert last_line_before_trunc_msg.endswith("...")

    def test_truncates_at_8_lines_when_first_8_lines_are_EQ_max_chars(self):
        expl = ["a" * 80 for x in range(16)]
        result = truncate._truncate_explanation(expl, max_lines=8, max_chars=8 * 80)
        assert result != expl
        assert len(result) == 8 + self.LINES_IN_TRUNCATION_MSG
        assert "Full output truncated" in result[-1]
        assert "9 lines hidden" in result[-1]
        last_line_before_trunc_msg = result[-self.LINES_IN_TRUNCATION_MSG - 1]
        assert last_line_before_trunc_msg.endswith("...")

    def test_truncates_at_4_lines_when_first_4_lines_are_GT_max_chars(self):
        expl = ["a" * 250 for x in range(10)]
        result = truncate._truncate_explanation(expl, max_lines=8, max_chars=999)
        assert result != expl
        assert len(result) == 4 + self.LINES_IN_TRUNCATION_MSG
        assert "Full output truncated" in result[-1]
        assert "7 lines hidden" in result[-1]
        last_line_before_trunc_msg = result[-self.LINES_IN_TRUNCATION_MSG - 1]
        assert last_line_before_trunc_msg.endswith("...")

    def test_truncates_at_1_line_when_first_line_is_GT_max_chars(self):
        expl = ["a" * 250 for x in range(1000)]
        result = truncate._truncate_explanation(expl, max_lines=8, max_chars=100)
        assert result != expl
        assert len(result) == 1 + self.LINES_IN_TRUNCATION_MSG
        assert "Full output truncated" in result[-1]
        assert "1000 lines hidden" in result[-1]
        last_line_before_trunc_msg = result[-self.LINES_IN_TRUNCATION_MSG - 1]
        assert last_line_before_trunc_msg.endswith("...")

    def test_full_output_truncated(self, monkeypatch, testdir):
        """ Test against full runpytest() output. """

        line_count = 7
        line_len = 100
        expected_truncated_lines = 2
        testdir.makepyfile(
            r"""
            def test_many_lines():
                a = list([str(i)[0] * %d for i in range(%d)])
                b = a[::2]
                a = '\n'.join(map(str, a))
                b = '\n'.join(map(str, b))
                assert a == b
        """
            % (line_len, line_count)
        )
        monkeypatch.delenv("CI", raising=False)

        result = testdir.runpytest()
        # without -vv, truncate the message showing a few diff lines only
        result.stdout.fnmatch_lines(
            [
                "*- 1*",
                "*- 3*",
                "*- 5*",
                "*truncated (%d lines hidden)*use*-vv*" % expected_truncated_lines,
            ]
        )

        result = testdir.runpytest("-vv")
        result.stdout.fnmatch_lines(["* 6*"])

        monkeypatch.setenv("CI", "1")
        result = testdir.runpytest()
        result.stdout.fnmatch_lines(["* 6*"])


def test_python25_compile_issue257(testdir):
    testdir.makepyfile(
        """
        def test_rewritten():
            assert 1 == 2
        # some comment
    """
    )
    result = testdir.runpytest()
    assert result.ret == 1
    result.stdout.fnmatch_lines(
        """
            *E*assert 1 == 2*
            *1 failed*
    """
    )


def test_rewritten(testdir):
    testdir.makepyfile(
        """
        def test_rewritten():
            assert "@py_builtins" in globals()
    """
    )
    assert testdir.runpytest().ret == 0


def test_reprcompare_notin(mock_config):
    detail = plugin.pytest_assertrepr_compare(
        mock_config, "not in", "foo", "aaafoobbb"
    )[
        1:
    ]
    assert detail == ["'foo' is contained here:", "  aaafoobbb", "?    +++"]


def test_reprcompare_whitespaces(mock_config):
    detail = plugin.pytest_assertrepr_compare(mock_config, "==", "\r\n", "\n")
    assert (
        detail
        == [
            r"'\r\n' == '\n'",
            r"Strings contain only whitespace, escaping them using repr()",
            r"- '\r\n'",
            r"?  --",
            r"+ '\n'",
        ]
    )


def test_pytest_assertrepr_compare_integration(testdir):
    testdir.makepyfile(
        """
        def test_hello():
            x = set(range(100))
            y = x.copy()
            y.remove(50)
            assert x == y
    """
    )
    result = testdir.runpytest()
    result.stdout.fnmatch_lines(
        ["*def test_hello():*", "*assert x == y*", "*E*Extra items*left*", "*E*50*"]
    )


def test_sequence_comparison_uses_repr(testdir):
    testdir.makepyfile(
        """
        def test_hello():
            x = set("hello x")
            y = set("hello y")
            assert x == y
    """
    )
    result = testdir.runpytest()
    result.stdout.fnmatch_lines(
        [
            "*def test_hello():*",
            "*assert x == y*",
            "*E*Extra items*left*",
            "*E*'x'*",
            "*E*Extra items*right*",
            "*E*'y'*",
        ]
    )


def test_assertrepr_loaded_per_dir(testdir):
    testdir.makepyfile(test_base=["def test_base(): assert 1 == 2"])
    a = testdir.mkdir("a")
    a_test = a.join("test_a.py")
    a_test.write("def test_a(): assert 1 == 2")
    a_conftest = a.join("conftest.py")
    a_conftest.write('def pytest_assertrepr_compare(): return ["summary a"]')
    b = testdir.mkdir("b")
    b_test = b.join("test_b.py")
    b_test.write("def test_b(): assert 1 == 2")
    b_conftest = b.join("conftest.py")
    b_conftest.write('def pytest_assertrepr_compare(): return ["summary b"]')
    result = testdir.runpytest()
    result.stdout.fnmatch_lines(
        [
            "*def test_base():*",
            "*E*assert 1 == 2*",
            "*def test_a():*",
            "*E*assert summary a*",
            "*def test_b():*",
            "*E*assert summary b*",
        ]
    )


def test_assertion_options(testdir):
    testdir.makepyfile(
        """
        def test_hello():
            x = 3
            assert x == 4
    """
    )
    result = testdir.runpytest()
    assert "3 == 4" in result.stdout.str()
    result = testdir.runpytest_subprocess("--assert=plain")
    assert "3 == 4" not in result.stdout.str()


def test_triple_quoted_string_issue113(testdir):
    testdir.makepyfile(
        """
        def test_hello():
            assert "" == '''
    '''"""
    )
    result = testdir.runpytest("--fulltrace")
    result.stdout.fnmatch_lines(["*1 failed*"])
    assert "SyntaxError" not in result.stdout.str()


def test_traceback_failure(testdir):
    p1 = testdir.makepyfile(
        """
        def g():
            return 2
        def f(x):
            assert x == g()
        def test_onefails():
            f(3)
    """
    )
    result = testdir.runpytest(p1, "--tb=long")
    result.stdout.fnmatch_lines(
        [
            "*test_traceback_failure.py F*",
            "====* FAILURES *====",
            "____*____",
            "",
            "    def test_onefails():",
            ">       f(3)",
            "",
            "*test_*.py:6: ",
            "_ _ _ *",
            # "",
            "    def f(x):",
            ">       assert x == g()",
            "E       assert 3 == 2",
            "E        +  where 2 = g()",
            "",
            "*test_traceback_failure.py:4: AssertionError",
        ]
    )

    result = testdir.runpytest(p1)  # "auto"
    result.stdout.fnmatch_lines(
        [
            "*test_traceback_failure.py F*",
            "====* FAILURES *====",
            "____*____",
            "",
            "    def test_onefails():",
            ">       f(3)",
            "",
            "*test_*.py:6: ",
            "",
            "    def f(x):",
            ">       assert x == g()",
            "E       assert 3 == 2",
            "E        +  where 2 = g()",
            "",
            "*test_traceback_failure.py:4: AssertionError",
        ]
    )


@pytest.mark.skipif(
    sys.version_info[:2] <= (3, 3),
    reason="Python 3.4+ shows chained exceptions on multiprocess",
)
def test_exception_handling_no_traceback(testdir):
    """
    Handle chain exceptions in tasks submitted by the multiprocess module (#1984).
    """
    p1 = testdir.makepyfile(
        """
        from multiprocessing import Pool

        def process_task(n):
            assert n == 10

        def multitask_job():
            tasks = [1]
            with Pool(processes=1) as pool:
                pool.map(process_task, tasks)

        def test_multitask_job():
            multitask_job()
    """
    )
    result = testdir.runpytest(p1, "--tb=long")
    result.stdout.fnmatch_lines(
        [
            "====* FAILURES *====",
            "*multiprocessing.pool.RemoteTraceback:*",
            "Traceback (most recent call last):",
            "*assert n == 10",
            "The above exception was the direct cause of the following exception:",
            "> * multitask_job()",
        ]
    )


@pytest.mark.skipif(
    "'__pypy__' in sys.builtin_module_names or sys.platform.startswith('java')"
)
def test_warn_missing(testdir):
    testdir.makepyfile("")
    result = testdir.run(sys.executable, "-OO", "-m", "pytest", "-h")
    result.stderr.fnmatch_lines(["*WARNING*assert statements are not executed*"])
    result = testdir.run(sys.executable, "-OO", "-m", "pytest")
    result.stderr.fnmatch_lines(["*WARNING*assert statements are not executed*"])


def test_recursion_source_decode(testdir):
    testdir.makepyfile(
        """
        def test_something():
            pass
    """
    )
    testdir.makeini(
        """
        [pytest]
        python_files = *.py
    """
    )
    result = testdir.runpytest("--collect-only")
    result.stdout.fnmatch_lines(
        """
        <Module*>
    """
    )


def test_AssertionError_message(testdir):
    testdir.makepyfile(
        """
        def test_hello():
            x,y = 1,2
            assert 0, (x,y)
    """
    )
    result = testdir.runpytest()
    result.stdout.fnmatch_lines(
        """
        *def test_hello*
        *assert 0, (x,y)*
        *AssertionError: (1, 2)*
    """
    )


@pytest.mark.skipif(PY3, reason="This bug does not exist on PY3")
def test_set_with_unsortable_elements():
    # issue #718
    class UnsortableKey(object):

        def __init__(self, name):
            self.name = name

        def __lt__(self, other):
            raise RuntimeError()

        def __repr__(self):
            return "repr({})".format(self.name)

        def __eq__(self, other):
            return self.name == other.name

        def __hash__(self):
            return hash(self.name)

    left_set = {UnsortableKey(str(i)) for i in range(1, 3)}
    right_set = {UnsortableKey(str(i)) for i in range(2, 4)}
    expl = callequal(left_set, right_set, verbose=True)
    # skip first line because it contains the "construction" of the set, which does not have a guaranteed order
    expl = expl[1:]
    dedent = textwrap.dedent(
        """
        Extra items in the left set:
        repr(1)
        Extra items in the right set:
        repr(3)
        Full diff (fallback to calling repr on each item):
        - repr(1)
        repr(2)
        + repr(3)
    """
    ).strip()
    assert "\n".join(expl) == dedent


def test_diff_newline_at_end(monkeypatch, testdir):
    testdir.makepyfile(
        r"""
        def test_diff():
            assert 'asdf' == 'asdf\n'
    """
    )

    result = testdir.runpytest()
    result.stdout.fnmatch_lines(
        r"""
        *assert 'asdf' == 'asdf\n'
        *  - asdf
        *  + asdf
        *  ?     +
    """
    )


def test_assert_tuple_warning(testdir):
    testdir.makepyfile(
        """
        def test_tuple():
            assert(False, 'you shall not pass')
    """
    )
    result = testdir.runpytest("-rw")
    result.stdout.fnmatch_lines(
        ["*test_assert_tuple_warning.py:2", "*assertion is always true*"]
    )


def test_assert_indirect_tuple_no_warning(testdir):
    testdir.makepyfile(
        """
        def test_tuple():
            tpl = ('foo', 'bar')
            assert tpl
    """
    )
    result = testdir.runpytest("-rw")
    output = "\n".join(result.stdout.lines)
    assert "WR1" not in output


def test_assert_with_unicode(monkeypatch, testdir):
    testdir.makepyfile(
        u"""
        # -*- coding: utf-8 -*-
        def test_unicode():
            assert u'유니코드' == u'Unicode'
    """
    )
    result = testdir.runpytest()
    result.stdout.fnmatch_lines(["*AssertionError*"])


def test_raise_unprintable_assertion_error(testdir):
    testdir.makepyfile(
        r"""
        def test_raise_assertion_error():
            raise AssertionError('\xff')
    """
    )
    result = testdir.runpytest()
    result.stdout.fnmatch_lines(
        [r">       raise AssertionError('\xff')", "E       AssertionError: *"]
    )


def test_raise_assertion_error_raisin_repr(testdir):
    testdir.makepyfile(
        u"""
        class RaisingRepr(object):
            def __repr__(self):
                raise Exception()
        def test_raising_repr():
            raise AssertionError(RaisingRepr())
    """
    )
    result = testdir.runpytest()
    result.stdout.fnmatch_lines(
        ["E       AssertionError: <unprintable AssertionError object>"]
    )


def test_issue_1944(testdir):
    testdir.makepyfile(
        """
        def f():
            return

        assert f() == 10
    """
    )
    result = testdir.runpytest()
    result.stdout.fnmatch_lines(["*1 error*"])
    assert "AttributeError: 'Module' object has no attribute '_obj'" not in result.stdout.str()
