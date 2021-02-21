import collections.abc
import sys
import textwrap
from typing import Any
from typing import List
from typing import Optional

import attr

import _pytest.assertion as plugin
import pytest
from _pytest import outcomes
from _pytest.assertion import truncate
from _pytest.assertion import util
from _pytest.compat import ATTRS_EQ_FIELD


def mock_config(verbose=0):
    class Config:
        def getoption(self, name):
            if name == "verbose":
                return verbose
            raise KeyError("Not mocked out: %s" % name)

    return Config()


class TestImportHookInstallation:
    @pytest.mark.parametrize("initial_conftest", [True, False])
    @pytest.mark.parametrize("mode", ["plain", "rewrite"])
    def test_conftest_assertion_rewrite(self, testdir, initial_conftest, mode):
        """Test that conftest files are using assertion rewrite on import (#1619)."""
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
        result.stdout.fnmatch_lines(
            [
                ">       r.assertoutcome(passed=1)",
                "E       AssertionError: ([[][]], [[][]], [[]<TestReport *>[]])*",
                "E       assert {'failed': 1,... 'skipped': 0} == {'failed': 0,... 'skipped': 0}",
                "E         Omitting 1 identical items, use -vv to show",
                "E         Differing items:",
                "E         Use -v to get the full diff",
            ]
        )
        # XXX: unstable output.
        result.stdout.fnmatch_lines_random(
            [
                "E         {'failed': 1} != {'failed': 0}",
                "E         {'passed': 0} != {'passed': 1}",
            ]
        )

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
            "conftest.py": """\
                pytest_plugins = "ham"
            """,
            "ham.py": "",
            "hamster.py": "",
            "test_foo.py": """\
                def test_foo(pytestconfig):
                    assert pytestconfig.pluginmanager.rewrite_hook.find_spec('ham') is not None
                    assert pytestconfig.pluginmanager.rewrite_hook.find_spec('hamster') is None
            """,
        }
        testdir.makepyfile(**contents)
        result = testdir.runpytest_subprocess("--assert=rewrite")
        assert result.ret == 0

    @pytest.mark.parametrize("mode", ["plain", "rewrite"])
    def test_installed_plugin_rewrite(self, testdir, mode, monkeypatch):
        monkeypatch.delenv("PYTEST_DISABLE_PLUGIN_AUTOLOAD", raising=False)
        # Make sure the hook is installed early enough so that plugins
        # installed via setuptools are rewritten.
        testdir.tmpdir.join("hampkg").ensure(dir=1)
        contents = {
            "hampkg/__init__.py": """\
                import pytest

                @pytest.fixture
                def check_first2():
                    def check(values, value):
                        assert values.pop(0) == value
                    return check
            """,
            "spamplugin.py": """\
            import pytest
            from hampkg import check_first2

            @pytest.fixture
            def check_first():
                def check(values, value):
                    assert values.pop(0) == value
                return check
            """,
            "mainwrapper.py": """\
            import pytest
            from _pytest.compat import importlib_metadata

            class DummyEntryPoint(object):
                name = 'spam'
                module_name = 'spam.py'
                group = 'pytest11'

                def load(self):
                    import spamplugin
                    return spamplugin

            class DummyDistInfo(object):
                version = '1.0'
                files = ('spamplugin.py', 'hampkg/__init__.py')
                entry_points = (DummyEntryPoint(),)
                metadata = {'name': 'foo'}

            def distributions():
                return (DummyDistInfo(),)

            importlib_metadata.distributions = distributions
            pytest.main()
            """,
            "test_foo.py": """\
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

    def test_register_assert_rewrite_checks_types(self) -> None:
        with pytest.raises(TypeError):
            pytest.register_assert_rewrite(["pytest_tests_internal_non_existing"])  # type: ignore
        pytest.register_assert_rewrite(
            "pytest_tests_internal_non_existing", "pytest_tests_internal_non_existing2"
        )


class TestBinReprIntegration:
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


def callop(op: str, left: Any, right: Any, verbose: int = 0) -> Optional[List[str]]:
    config = mock_config(verbose=verbose)
    return plugin.pytest_assertrepr_compare(config, op, left, right)


def callequal(left: Any, right: Any, verbose: int = 0) -> Optional[List[str]]:
    return callop("==", left, right, verbose)


class TestAssert_reprcompare:
    def test_different_types(self):
        assert callequal([0, 1], "foo") is None

    def test_summary(self) -> None:
        lines = callequal([0, 1], [0, 2])
        assert lines is not None
        summary = lines[0]
        assert len(summary) < 65

    def test_text_diff(self):
        assert callequal("spam", "eggs") == [
            "'spam' == 'eggs'",
            "- eggs",
            "+ spam",
        ]

    def test_text_skipping(self) -> None:
        lines = callequal("a" * 50 + "spam", "a" * 50 + "eggs")
        assert lines is not None
        assert "Skipping" in lines[1]
        for line in lines:
            assert "a" * 50 not in line

    def test_text_skipping_verbose(self) -> None:
        lines = callequal("a" * 50 + "spam", "a" * 50 + "eggs", verbose=1)
        assert lines is not None
        assert "- " + "a" * 50 + "eggs" in lines
        assert "+ " + "a" * 50 + "spam" in lines

    def test_multiline_text_diff(self) -> None:
        left = "foo\nspam\nbar"
        right = "foo\neggs\nbar"
        diff = callequal(left, right)
        assert diff is not None
        assert "- eggs" in diff
        assert "+ spam" in diff

    def test_bytes_diff_normal(self):
        """Check special handling for bytes diff (#5260)"""
        diff = callequal(b"spam", b"eggs")

        assert diff == [
            "b'spam' == b'eggs'",
            "At index 0 diff: b's' != b'e'",
            "Use -v to get the full diff",
        ]

    def test_bytes_diff_verbose(self):
        """Check special handling for bytes diff (#5260)"""
        diff = callequal(b"spam", b"eggs", verbose=1)
        assert diff == [
            "b'spam' == b'eggs'",
            "At index 0 diff: b's' != b'e'",
            "Full diff:",
            "- b'eggs'",
            "+ b'spam'",
        ]

    def test_list(self) -> None:
        expl = callequal([0, 1], [0, 2])
        assert expl is not None
        assert len(expl) > 1

    @pytest.mark.parametrize(
        ["left", "right", "expected"],
        [
            pytest.param(
                [0, 1],
                [0, 2],
                """
                Full diff:
                - [0, 2]
                ?     ^
                + [0, 1]
                ?     ^
            """,
                id="lists",
            ),
            pytest.param(
                {0: 1},
                {0: 2},
                """
                Full diff:
                - {0: 2}
                ?     ^
                + {0: 1}
                ?     ^
            """,
                id="dicts",
            ),
            pytest.param(
                {0, 1},
                {0, 2},
                """
                Full diff:
                - {0, 2}
                ?     ^
                + {0, 1}
                ?     ^
            """,
                id="sets",
            ),
        ],
    )
    def test_iterable_full_diff(self, left, right, expected) -> None:
        """Test the full diff assertion failure explanation.

        When verbose is False, then just a -v notice to get the diff is rendered,
        when verbose is True, then ndiff of the pprint is returned.
        """
        expl = callequal(left, right, verbose=0)
        assert expl is not None
        assert expl[-1] == "Use -v to get the full diff"
        verbose_expl = callequal(left, right, verbose=1)
        assert verbose_expl is not None
        assert "\n".join(verbose_expl).endswith(textwrap.dedent(expected).strip())

    def test_list_different_lengths(self) -> None:
        expl = callequal([0, 1], [0, 1, 2])
        assert expl is not None
        assert len(expl) > 1
        expl = callequal([0, 1, 2], [0, 1])
        assert expl is not None
        assert len(expl) > 1

    def test_list_wrap_for_multiple_lines(self):
        long_d = "d" * 80
        l1 = ["a", "b", "c"]
        l2 = ["a", "b", "c", long_d]
        diff = callequal(l1, l2, verbose=True)
        assert diff == [
            "['a', 'b', 'c'] == ['a', 'b', 'c...dddddddddddd']",
            "Right contains one more item: '" + long_d + "'",
            "Full diff:",
            "  [",
            "   'a',",
            "   'b',",
            "   'c',",
            "-  '" + long_d + "',",
            "  ]",
        ]

        diff = callequal(l2, l1, verbose=True)
        assert diff == [
            "['a', 'b', 'c...dddddddddddd'] == ['a', 'b', 'c']",
            "Left contains one more item: '" + long_d + "'",
            "Full diff:",
            "  [",
            "   'a',",
            "   'b',",
            "   'c',",
            "+  '" + long_d + "',",
            "  ]",
        ]

    def test_list_wrap_for_width_rewrap_same_length(self):
        long_a = "a" * 30
        long_b = "b" * 30
        long_c = "c" * 30
        l1 = [long_a, long_b, long_c]
        l2 = [long_b, long_c, long_a]
        diff = callequal(l1, l2, verbose=True)
        assert diff == [
            "['aaaaaaaaaaa...cccccccccccc'] == ['bbbbbbbbbbb...aaaaaaaaaaaa']",
            "At index 0 diff: 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa' != 'bbbbbbbbbbbbbbbbbbbbbbbbbbbbbb'",
            "Full diff:",
            "  [",
            "+  'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',",
            "   'bbbbbbbbbbbbbbbbbbbbbbbbbbbbbb',",
            "   'cccccccccccccccccccccccccccccc',",
            "-  'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',",
            "  ]",
        ]

    def test_list_dont_wrap_strings(self):
        long_a = "a" * 10
        l1 = ["a"] + [long_a for _ in range(0, 7)]
        l2 = ["should not get wrapped"]
        diff = callequal(l1, l2, verbose=True)
        assert diff == [
            "['a', 'aaaaaa...aaaaaaa', ...] == ['should not get wrapped']",
            "At index 0 diff: 'a' != 'should not get wrapped'",
            "Left contains 7 more items, first extra item: 'aaaaaaaaaa'",
            "Full diff:",
            "  [",
            "-  'should not get wrapped',",
            "+  'a',",
            "+  'aaaaaaaaaa',",
            "+  'aaaaaaaaaa',",
            "+  'aaaaaaaaaa',",
            "+  'aaaaaaaaaa',",
            "+  'aaaaaaaaaa',",
            "+  'aaaaaaaaaa',",
            "+  'aaaaaaaaaa',",
            "  ]",
        ]

    def test_dict_wrap(self):
        d1 = {"common": 1, "env": {"env1": 1, "env2": 2}}
        d2 = {"common": 1, "env": {"env1": 1}}

        diff = callequal(d1, d2, verbose=True)
        assert diff == [
            "{'common': 1,...1, 'env2': 2}} == {'common': 1,...: {'env1': 1}}",
            "Omitting 1 identical items, use -vv to show",
            "Differing items:",
            "{'env': {'env1': 1, 'env2': 2}} != {'env': {'env1': 1}}",
            "Full diff:",
            "- {'common': 1, 'env': {'env1': 1}}",
            "+ {'common': 1, 'env': {'env1': 1, 'env2': 2}}",
            "?                                +++++++++++",
        ]

        long_a = "a" * 80
        sub = {"long_a": long_a, "sub1": {"long_a": "substring that gets wrapped " * 2}}
        d1 = {"env": {"sub": sub}}
        d2 = {"env": {"sub": sub}, "new": 1}
        diff = callequal(d1, d2, verbose=True)
        assert diff == [
            "{'env': {'sub... wrapped '}}}} == {'env': {'sub...}}}, 'new': 1}",
            "Omitting 1 identical items, use -vv to show",
            "Right contains 1 more item:",
            "{'new': 1}",
            "Full diff:",
            "  {",
            "   'env': {'sub': {'long_a': '" + long_a + "',",
            "                   'sub1': {'long_a': 'substring that gets wrapped substring '",
            "                                      'that gets wrapped '}}},",
            "-  'new': 1,",
            "  }",
        ]

    def test_dict(self) -> None:
        expl = callequal({"a": 0}, {"a": 1})
        assert expl is not None
        assert len(expl) > 1

    def test_dict_omitting(self) -> None:
        lines = callequal({"a": 0, "b": 1}, {"a": 1, "b": 1})
        assert lines is not None
        assert lines[1].startswith("Omitting 1 identical item")
        assert "Common items" not in lines
        for line in lines[1:]:
            assert "b" not in line

    def test_dict_omitting_with_verbosity_1(self) -> None:
        """Ensure differing items are visible for verbosity=1 (#1512)."""
        lines = callequal({"a": 0, "b": 1}, {"a": 1, "b": 1}, verbose=1)
        assert lines is not None
        assert lines[1].startswith("Omitting 1 identical item")
        assert lines[2].startswith("Differing items")
        assert lines[3] == "{'a': 0} != {'a': 1}"
        assert "Common items" not in lines

    def test_dict_omitting_with_verbosity_2(self) -> None:
        lines = callequal({"a": 0, "b": 1}, {"a": 1, "b": 1}, verbose=2)
        assert lines is not None
        assert lines[1].startswith("Common items:")
        assert "Omitting" not in lines[1]
        assert lines[2] == "{'b': 1}"

    def test_dict_different_items(self):
        lines = callequal({"a": 0}, {"b": 1, "c": 2}, verbose=2)
        assert lines == [
            "{'a': 0} == {'b': 1, 'c': 2}",
            "Left contains 1 more item:",
            "{'a': 0}",
            "Right contains 2 more items:",
            "{'b': 1, 'c': 2}",
            "Full diff:",
            "- {'b': 1, 'c': 2}",
            "+ {'a': 0}",
        ]
        lines = callequal({"b": 1, "c": 2}, {"a": 0}, verbose=2)
        assert lines == [
            "{'b': 1, 'c': 2} == {'a': 0}",
            "Left contains 2 more items:",
            "{'b': 1, 'c': 2}",
            "Right contains 1 more item:",
            "{'a': 0}",
            "Full diff:",
            "- {'a': 0}",
            "+ {'b': 1, 'c': 2}",
        ]

    def test_sequence_different_items(self):
        lines = callequal((1, 2), (3, 4, 5), verbose=2)
        assert lines == [
            "(1, 2) == (3, 4, 5)",
            "At index 0 diff: 1 != 3",
            "Right contains one more item: 5",
            "Full diff:",
            "- (3, 4, 5)",
            "+ (1, 2)",
        ]
        lines = callequal((1, 2, 3), (4,), verbose=2)
        assert lines == [
            "(1, 2, 3) == (4,)",
            "At index 0 diff: 1 != 4",
            "Left contains 2 more items, first extra item: 2",
            "Full diff:",
            "- (4,)",
            "+ (1, 2, 3)",
        ]

    def test_set(self) -> None:
        expl = callequal({0, 1}, {0, 2})
        assert expl is not None
        assert len(expl) > 1

    def test_frozenzet(self) -> None:
        expl = callequal(frozenset([0, 1]), {0, 2})
        assert expl is not None
        assert len(expl) > 1

    def test_Sequence(self) -> None:
        # Test comparing with a Sequence subclass.
        # TODO(py36): Inherit from typing.MutableSequence[int].
        class TestSequence(collections.abc.MutableSequence):  # type: ignore[type-arg]
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
        assert expl is not None
        assert len(expl) > 1

    def test_list_tuples(self) -> None:
        expl = callequal([], [(1, 2)])
        assert expl is not None
        assert len(expl) > 1
        expl = callequal([(1, 2)], [])
        assert expl is not None
        assert len(expl) > 1

    def test_repr_verbose(self) -> None:
        class Nums:
            def __init__(self, nums):
                self.nums = nums

            def __repr__(self):
                return str(self.nums)

        list_x = list(range(5000))
        list_y = list(range(5000))
        list_y[len(list_y) // 2] = 3
        nums_x = Nums(list_x)
        nums_y = Nums(list_y)

        assert callequal(nums_x, nums_y) is None

        expl = callequal(nums_x, nums_y, verbose=1)
        assert expl is not None
        assert "+" + repr(nums_x) in expl
        assert "-" + repr(nums_y) in expl

        expl = callequal(nums_x, nums_y, verbose=2)
        assert expl is not None
        assert "+" + repr(nums_x) in expl
        assert "-" + repr(nums_y) in expl

    def test_list_bad_repr(self) -> None:
        class A:
            def __repr__(self):
                raise ValueError(42)

        expl = callequal([], [A()])
        assert expl is not None
        assert "ValueError" in "".join(expl)
        expl = callequal({}, {"1": A()}, verbose=2)
        assert expl is not None
        assert expl[0].startswith("{} == <[ValueError")
        assert "raised in repr" in expl[0]
        assert expl[1:] == [
            "(pytest_assertion plugin: representation of details failed:"
            " {}:{}: ValueError: 42.".format(
                __file__, A.__repr__.__code__.co_firstlineno + 1
            ),
            " Probably an object has a faulty __repr__.)",
        ]

    def test_one_repr_empty(self):
        """The faulty empty string repr did trigger an unbound local error in _diff_text."""

        class A(str):
            def __repr__(self):
                return ""

        expl = callequal(A(), "")
        assert not expl

    def test_repr_no_exc(self) -> None:
        expl = callequal("foo", "bar")
        assert expl is not None
        assert "raised in repr()" not in " ".join(expl)

    def test_unicode(self):
        assert callequal("£€", "£") == [
            "'£€' == '£'",
            "- £",
            "+ £€",
        ]

    def test_nonascii_text(self):
        """
        :issue: 877
        non ascii python2 str caused a UnicodeDecodeError
        """

        class A(str):
            def __repr__(self):
                return "\xff"

        expl = callequal(A(), "1")
        assert expl == ["ÿ == '1'", "- 1"]

    def test_format_nonascii_explanation(self):
        assert util.format_explanation("λ")

    def test_mojibake(self) -> None:
        # issue 429
        left = b"e"
        right = b"\xc3\xa9"
        expl = callequal(left, right)
        assert expl is not None
        for line in expl:
            assert isinstance(line, str)
        msg = "\n".join(expl)
        assert msg


class TestAssert_reprcompare_dataclass:
    @pytest.mark.skipif(sys.version_info < (3, 7), reason="Dataclasses in Python3.7+")
    def test_dataclasses(self, testdir):
        p = testdir.copy_example("dataclasses/test_compare_dataclasses.py")
        result = testdir.runpytest(p)
        result.assert_outcomes(failed=1, passed=0)
        result.stdout.fnmatch_lines(
            [
                "E         Omitting 1 identical items, use -vv to show",
                "E         Differing attributes:",
                "E         ['field_b']",
                "E         ",
                "E         Drill down into differing attribute field_b:",
                "E           field_b: 'b' != 'c'...",
                "E         ",
                "E         ...Full output truncated (3 lines hidden), use '-vv' to show",
            ],
            consecutive=True,
        )

    @pytest.mark.skipif(sys.version_info < (3, 7), reason="Dataclasses in Python3.7+")
    def test_recursive_dataclasses(self, testdir):
        p = testdir.copy_example("dataclasses/test_compare_recursive_dataclasses.py")
        result = testdir.runpytest(p)
        result.assert_outcomes(failed=1, passed=0)
        result.stdout.fnmatch_lines(
            [
                "E         Omitting 1 identical items, use -vv to show",
                "E         Differing attributes:",
                "E         ['g', 'h', 'j']",
                "E         ",
                "E         Drill down into differing attribute g:",
                "E           g: S(a=10, b='ten') != S(a=20, b='xxx')...",
                "E         ",
                "E         ...Full output truncated (52 lines hidden), use '-vv' to show",
            ],
            consecutive=True,
        )

    @pytest.mark.skipif(sys.version_info < (3, 7), reason="Dataclasses in Python3.7+")
    def test_recursive_dataclasses_verbose(self, testdir):
        p = testdir.copy_example("dataclasses/test_compare_recursive_dataclasses.py")
        result = testdir.runpytest(p, "-vv")
        result.assert_outcomes(failed=1, passed=0)
        result.stdout.fnmatch_lines(
            [
                "E         Matching attributes:",
                "E         ['i']",
                "E         Differing attributes:",
                "E         ['g', 'h', 'j']",
                "E         ",
                "E         Drill down into differing attribute g:",
                "E           g: S(a=10, b='ten') != S(a=20, b='xxx')",
                "E           ",
                "E           Differing attributes:",
                "E           ['a', 'b']",
                "E           ",
                "E           Drill down into differing attribute a:",
                "E             a: 10 != 20",
                "E             +10",
                "E             -20",
                "E           ",
                "E           Drill down into differing attribute b:",
                "E             b: 'ten' != 'xxx'",
                "E             - xxx",
                "E             + ten",
                "E         ",
                "E         Drill down into differing attribute h:",
            ],
            consecutive=True,
        )

    @pytest.mark.skipif(sys.version_info < (3, 7), reason="Dataclasses in Python3.7+")
    def test_dataclasses_verbose(self, testdir):
        p = testdir.copy_example("dataclasses/test_compare_dataclasses_verbose.py")
        result = testdir.runpytest(p, "-vv")
        result.assert_outcomes(failed=1, passed=0)
        result.stdout.fnmatch_lines(
            [
                "*Matching attributes:*",
                "*['field_a']*",
                "*Differing attributes:*",
                "*field_b: 'b' != 'c'*",
            ]
        )

    @pytest.mark.skipif(sys.version_info < (3, 7), reason="Dataclasses in Python3.7+")
    def test_dataclasses_with_attribute_comparison_off(self, testdir):
        p = testdir.copy_example(
            "dataclasses/test_compare_dataclasses_field_comparison_off.py"
        )
        result = testdir.runpytest(p, "-vv")
        result.assert_outcomes(failed=0, passed=1)

    @pytest.mark.skipif(sys.version_info < (3, 7), reason="Dataclasses in Python3.7+")
    def test_comparing_two_different_data_classes(self, testdir):
        p = testdir.copy_example(
            "dataclasses/test_compare_two_different_dataclasses.py"
        )
        result = testdir.runpytest(p, "-vv")
        result.assert_outcomes(failed=0, passed=1)


class TestAssert_reprcompare_attrsclass:
    def test_attrs(self) -> None:
        @attr.s
        class SimpleDataObject:
            field_a = attr.ib()
            field_b = attr.ib()

        left = SimpleDataObject(1, "b")
        right = SimpleDataObject(1, "c")

        lines = callequal(left, right)
        assert lines is not None
        assert lines[2].startswith("Omitting 1 identical item")
        assert "Matching attributes" not in lines
        for line in lines[2:]:
            assert "field_a" not in line

    def test_attrs_recursive(self) -> None:
        @attr.s
        class OtherDataObject:
            field_c = attr.ib()
            field_d = attr.ib()

        @attr.s
        class SimpleDataObject:
            field_a = attr.ib()
            field_b = attr.ib()

        left = SimpleDataObject(OtherDataObject(1, "a"), "b")
        right = SimpleDataObject(OtherDataObject(1, "b"), "b")

        lines = callequal(left, right)
        assert lines is not None
        assert "Matching attributes" not in lines
        for line in lines[1:]:
            assert "field_b:" not in line
            assert "field_c:" not in line

    def test_attrs_recursive_verbose(self) -> None:
        @attr.s
        class OtherDataObject:
            field_c = attr.ib()
            field_d = attr.ib()

        @attr.s
        class SimpleDataObject:
            field_a = attr.ib()
            field_b = attr.ib()

        left = SimpleDataObject(OtherDataObject(1, "a"), "b")
        right = SimpleDataObject(OtherDataObject(1, "b"), "b")

        lines = callequal(left, right)
        assert lines is not None
        # indentation in output because of nested object structure
        assert "    field_d: 'a' != 'b'" in lines

    def test_attrs_verbose(self) -> None:
        @attr.s
        class SimpleDataObject:
            field_a = attr.ib()
            field_b = attr.ib()

        left = SimpleDataObject(1, "b")
        right = SimpleDataObject(1, "c")

        lines = callequal(left, right, verbose=2)
        assert lines is not None
        assert lines[2].startswith("Matching attributes:")
        assert "Omitting" not in lines[2]
        assert lines[3] == "['field_a']"

    def test_attrs_with_attribute_comparison_off(self):
        @attr.s
        class SimpleDataObject:
            field_a = attr.ib()
            field_b = attr.ib(**{ATTRS_EQ_FIELD: False})  # type: ignore

        left = SimpleDataObject(1, "b")
        right = SimpleDataObject(1, "b")

        lines = callequal(left, right, verbose=2)
        print(lines)
        assert lines is not None
        assert lines[2].startswith("Matching attributes:")
        assert "Omitting" not in lines[1]
        assert lines[3] == "['field_a']"
        for line in lines[3:]:
            assert "field_b" not in line

    def test_comparing_two_different_attrs_classes(self):
        @attr.s
        class SimpleDataObjectOne:
            field_a = attr.ib()
            field_b = attr.ib()

        @attr.s
        class SimpleDataObjectTwo:
            field_a = attr.ib()
            field_b = attr.ib()

        left = SimpleDataObjectOne(1, "b")
        right = SimpleDataObjectTwo(1, "c")

        lines = callequal(left, right)
        assert lines is None


class TestFormatExplanation:
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


class TestTruncateExplanation:
    # The number of lines in the truncation explanation message. Used
    # to calculate that results have the expected length.
    LINES_IN_TRUNCATION_MSG = 2

    def test_doesnt_truncate_when_input_is_empty_list(self) -> None:
        expl = []  # type: List[str]
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
        """Test against full runpytest() output."""

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
                "*+ 1*",
                "*+ 3*",
                "*+ 5*",
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


def test_reprcompare_notin() -> None:
    assert callop("not in", "foo", "aaafoobbb") == [
        "'foo' not in 'aaafoobbb'",
        "'foo' is contained here:",
        "  aaafoobbb",
        "?    +++",
    ]


def test_reprcompare_whitespaces():
    assert callequal("\r\n", "\n") == [
        r"'\r\n' == '\n'",
        r"Strings contain only whitespace, escaping them using repr()",
        r"- '\n'",
        r"+ '\r\n'",
        r"?  ++",
    ]


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
        [
            "*def test_hello():*",
            "*assert x == y*",
            "*E*Extra items*left*",
            "*E*50*",
            "*= 1 failed in*",
        ]
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
    result.stdout.no_fnmatch_line("*3 == 4*")


def test_triple_quoted_string_issue113(testdir):
    testdir.makepyfile(
        """
        def test_hello():
            assert "" == '''
    '''"""
    )
    result = testdir.runpytest("--fulltrace")
    result.stdout.fnmatch_lines(["*1 failed*"])
    result.stdout.no_fnmatch_line("*SyntaxError*")


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


def test_exception_handling_no_traceback(testdir):
    """Handle chain exceptions in tasks submitted by the multiprocess module (#1984)."""
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
    testdir.syspathinsert()
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


@pytest.mark.skipif("'__pypy__' in sys.builtin_module_names")
@pytest.mark.parametrize(
    "cmdline_args, warning_output",
    [
        (
            ["-OO", "-m", "pytest", "-h"],
            ["warning :*PytestConfigWarning:*assert statements are not executed*"],
        ),
        (
            ["-OO", "-m", "pytest"],
            [
                "=*= warnings summary =*=",
                "*PytestConfigWarning:*assert statements are not executed*",
            ],
        ),
        (
            ["-OO", "-m", "pytest", "--assert=plain"],
            [
                "=*= warnings summary =*=",
                "*PytestConfigWarning: ASSERTIONS ARE NOT EXECUTED and FAILING TESTS WILL PASS.  "
                "Are you using python -O?",
            ],
        ),
    ],
)
def test_warn_missing(testdir, cmdline_args, warning_output):
    testdir.makepyfile("")

    result = testdir.run(sys.executable, *cmdline_args)
    result.stdout.fnmatch_lines(warning_output)


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


def test_diff_newline_at_end(testdir):
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
        *  ?     -
        *  + asdf
    """
    )


@pytest.mark.filterwarnings("default")
def test_assert_tuple_warning(testdir):
    msg = "assertion is always true"
    testdir.makepyfile(
        """
        def test_tuple():
            assert(False, 'you shall not pass')
    """
    )
    result = testdir.runpytest()
    result.stdout.fnmatch_lines(["*test_assert_tuple_warning.py:2:*{}*".format(msg)])

    # tuples with size != 2 should not trigger the warning
    testdir.makepyfile(
        """
        def test_tuple():
            assert ()
    """
    )
    result = testdir.runpytest()
    assert msg not in result.stdout.str()


def test_assert_indirect_tuple_no_warning(testdir):
    testdir.makepyfile(
        """
        def test_tuple():
            tpl = ('foo', 'bar')
            assert tpl
    """
    )
    result = testdir.runpytest()
    output = "\n".join(result.stdout.lines)
    assert "WR1" not in output


def test_assert_with_unicode(testdir):
    testdir.makepyfile(
        """\
        def test_unicode():
            assert '유니코드' == 'Unicode'
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
        """
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
    assert (
        "AttributeError: 'Module' object has no attribute '_obj'"
        not in result.stdout.str()
    )


def test_exit_from_assertrepr_compare(monkeypatch):
    def raise_exit(obj):
        outcomes.exit("Quitting debugger")

    monkeypatch.setattr(util, "istext", raise_exit)

    with pytest.raises(outcomes.Exit, match="Quitting debugger"):
        callequal(1, 1)


def test_assertion_location_with_coverage(testdir):
    """This used to report the wrong location when run with coverage (#5754)."""
    p = testdir.makepyfile(
        """
        def test():
            assert False, 1
            assert False, 2
        """
    )
    result = testdir.runpytest(str(p))
    result.stdout.fnmatch_lines(
        [
            ">       assert False, 1",
            "E       AssertionError: 1",
            "E       assert False",
            "*= 1 failed in*",
        ]
    )
