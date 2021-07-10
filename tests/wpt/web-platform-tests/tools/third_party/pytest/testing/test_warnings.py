import os
import warnings
from typing import List
from typing import Optional
from typing import Tuple

import pytest
from _pytest.fixtures import FixtureRequest
from _pytest.pytester import Testdir

WARNINGS_SUMMARY_HEADER = "warnings summary"


@pytest.fixture
def pyfile_with_warnings(testdir: Testdir, request: FixtureRequest) -> str:
    """Create a test file which calls a function in a module which generates warnings."""
    testdir.syspathinsert()
    test_name = request.function.__name__
    module_name = test_name.lstrip("test_") + "_module"
    test_file = testdir.makepyfile(
        """
        import {module_name}
        def test_func():
            assert {module_name}.foo() == 1
        """.format(
            module_name=module_name
        ),
        **{
            module_name: """
            import warnings
            def foo():
                warnings.warn(UserWarning("user warning"))
                warnings.warn(RuntimeWarning("runtime warning"))
                return 1
            """,
        },
    )
    return str(test_file)


@pytest.mark.filterwarnings("default")
def test_normal_flow(testdir, pyfile_with_warnings):
    """Check that the warnings section is displayed."""
    result = testdir.runpytest(pyfile_with_warnings)
    result.stdout.fnmatch_lines(
        [
            "*== %s ==*" % WARNINGS_SUMMARY_HEADER,
            "test_normal_flow.py::test_func",
            "*normal_flow_module.py:3: UserWarning: user warning",
            '*  warnings.warn(UserWarning("user warning"))',
            "*normal_flow_module.py:4: RuntimeWarning: runtime warning",
            '*  warnings.warn(RuntimeWarning("runtime warning"))',
            "* 1 passed, 2 warnings*",
        ]
    )


@pytest.mark.filterwarnings("always")
def test_setup_teardown_warnings(testdir):
    testdir.makepyfile(
        """
        import warnings
        import pytest

        @pytest.fixture
        def fix():
            warnings.warn(UserWarning("warning during setup"))
            yield
            warnings.warn(UserWarning("warning during teardown"))

        def test_func(fix):
            pass
    """
    )
    result = testdir.runpytest()
    result.stdout.fnmatch_lines(
        [
            "*== %s ==*" % WARNINGS_SUMMARY_HEADER,
            "*test_setup_teardown_warnings.py:6: UserWarning: warning during setup",
            '*warnings.warn(UserWarning("warning during setup"))',
            "*test_setup_teardown_warnings.py:8: UserWarning: warning during teardown",
            '*warnings.warn(UserWarning("warning during teardown"))',
            "* 1 passed, 2 warnings*",
        ]
    )


@pytest.mark.parametrize("method", ["cmdline", "ini"])
def test_as_errors(testdir, pyfile_with_warnings, method):
    args = ("-W", "error") if method == "cmdline" else ()
    if method == "ini":
        testdir.makeini(
            """
            [pytest]
            filterwarnings=error
            """
        )
    # Use a subprocess, since changing logging level affects other threads
    # (xdist).
    result = testdir.runpytest_subprocess(*args, pyfile_with_warnings)
    result.stdout.fnmatch_lines(
        [
            "E       UserWarning: user warning",
            "as_errors_module.py:3: UserWarning",
            "* 1 failed in *",
        ]
    )


@pytest.mark.parametrize("method", ["cmdline", "ini"])
def test_ignore(testdir, pyfile_with_warnings, method):
    args = ("-W", "ignore") if method == "cmdline" else ()
    if method == "ini":
        testdir.makeini(
            """
        [pytest]
        filterwarnings= ignore
        """
        )

    result = testdir.runpytest(*args, pyfile_with_warnings)
    result.stdout.fnmatch_lines(["* 1 passed in *"])
    assert WARNINGS_SUMMARY_HEADER not in result.stdout.str()


@pytest.mark.filterwarnings("always")
def test_unicode(testdir):
    testdir.makepyfile(
        """
        import warnings
        import pytest


        @pytest.fixture
        def fix():
            warnings.warn("测试")
            yield

        def test_func(fix):
            pass
        """
    )
    result = testdir.runpytest()
    result.stdout.fnmatch_lines(
        [
            "*== %s ==*" % WARNINGS_SUMMARY_HEADER,
            "*test_unicode.py:7: UserWarning: \u6d4b\u8bd5*",
            "* 1 passed, 1 warning*",
        ]
    )


def test_works_with_filterwarnings(testdir):
    """Ensure our warnings capture does not mess with pre-installed filters (#2430)."""
    testdir.makepyfile(
        """
        import warnings

        class MyWarning(Warning):
            pass

        warnings.filterwarnings("error", category=MyWarning)

        class TestWarnings(object):
            def test_my_warning(self):
                try:
                    warnings.warn(MyWarning("warn!"))
                    assert False
                except MyWarning:
                    assert True
    """
    )
    result = testdir.runpytest()
    result.stdout.fnmatch_lines(["*== 1 passed in *"])


@pytest.mark.parametrize("default_config", ["ini", "cmdline"])
def test_filterwarnings_mark(testdir, default_config):
    """Test ``filterwarnings`` mark works and takes precedence over command
    line and ini options."""
    if default_config == "ini":
        testdir.makeini(
            """
            [pytest]
            filterwarnings = always
        """
        )
    testdir.makepyfile(
        """
        import warnings
        import pytest

        @pytest.mark.filterwarnings('ignore::RuntimeWarning')
        def test_ignore_runtime_warning():
            warnings.warn(RuntimeWarning())

        @pytest.mark.filterwarnings('error')
        def test_warning_error():
            warnings.warn(RuntimeWarning())

        def test_show_warning():
            warnings.warn(RuntimeWarning())
    """
    )
    result = testdir.runpytest("-W always" if default_config == "cmdline" else "")
    result.stdout.fnmatch_lines(["*= 1 failed, 2 passed, 1 warning in *"])


def test_non_string_warning_argument(testdir):
    """Non-str argument passed to warning breaks pytest (#2956)"""
    testdir.makepyfile(
        """\
        import warnings
        import pytest

        def test():
            warnings.warn(UserWarning(1, 'foo'))
        """
    )
    result = testdir.runpytest("-W", "always")
    result.stdout.fnmatch_lines(["*= 1 passed, 1 warning in *"])


def test_filterwarnings_mark_registration(testdir):
    """Ensure filterwarnings mark is registered"""
    testdir.makepyfile(
        """
        import pytest

        @pytest.mark.filterwarnings('error')
        def test_func():
            pass
    """
    )
    result = testdir.runpytest("--strict-markers")
    assert result.ret == 0


@pytest.mark.filterwarnings("always")
def test_warning_captured_hook(testdir):
    testdir.makeconftest(
        """
        def pytest_configure(config):
            config.issue_config_time_warning(UserWarning("config warning"), stacklevel=2)
    """
    )
    testdir.makepyfile(
        """
        import pytest, warnings

        warnings.warn(UserWarning("collect warning"))

        @pytest.fixture
        def fix():
            warnings.warn(UserWarning("setup warning"))
            yield 1
            warnings.warn(UserWarning("teardown warning"))

        def test_func(fix):
            warnings.warn(UserWarning("call warning"))
            assert fix == 1
        """
    )

    collected = []

    class WarningCollector:
        def pytest_warning_recorded(self, warning_message, when, nodeid, location):
            collected.append((str(warning_message.message), when, nodeid, location))

    result = testdir.runpytest(plugins=[WarningCollector()])
    result.stdout.fnmatch_lines(["*1 passed*"])

    expected = [
        ("config warning", "config", ""),
        ("collect warning", "collect", ""),
        ("setup warning", "runtest", "test_warning_captured_hook.py::test_func"),
        ("call warning", "runtest", "test_warning_captured_hook.py::test_func"),
        ("teardown warning", "runtest", "test_warning_captured_hook.py::test_func"),
    ]
    for index in range(len(expected)):
        collected_result = collected[index]
        expected_result = expected[index]

        assert collected_result[0] == expected_result[0], str(collected)
        assert collected_result[1] == expected_result[1], str(collected)
        assert collected_result[2] == expected_result[2], str(collected)

        # NOTE: collected_result[3] is location, which differs based on the platform you are on
        #       thus, the best we can do here is assert the types of the paremeters match what we expect
        #       and not try and preload it in the expected array
        if collected_result[3] is not None:
            assert type(collected_result[3][0]) is str, str(collected)
            assert type(collected_result[3][1]) is int, str(collected)
            assert type(collected_result[3][2]) is str, str(collected)
        else:
            assert collected_result[3] is None, str(collected)


@pytest.mark.filterwarnings("always")
def test_collection_warnings(testdir):
    """Check that we also capture warnings issued during test collection (#3251)."""
    testdir.makepyfile(
        """
        import warnings

        warnings.warn(UserWarning("collection warning"))

        def test_foo():
            pass
    """
    )
    result = testdir.runpytest()
    result.stdout.fnmatch_lines(
        [
            "*== %s ==*" % WARNINGS_SUMMARY_HEADER,
            "  *collection_warnings.py:3: UserWarning: collection warning",
            '    warnings.warn(UserWarning("collection warning"))',
            "* 1 passed, 1 warning*",
        ]
    )


@pytest.mark.filterwarnings("always")
def test_mark_regex_escape(testdir):
    """@pytest.mark.filterwarnings should not try to escape regex characters (#3936)"""
    testdir.makepyfile(
        r"""
        import pytest, warnings

        @pytest.mark.filterwarnings(r"ignore:some \(warning\)")
        def test_foo():
            warnings.warn(UserWarning("some (warning)"))
    """
    )
    result = testdir.runpytest()
    assert WARNINGS_SUMMARY_HEADER not in result.stdout.str()


@pytest.mark.filterwarnings("default")
@pytest.mark.parametrize("ignore_pytest_warnings", ["no", "ini", "cmdline"])
def test_hide_pytest_internal_warnings(testdir, ignore_pytest_warnings):
    """Make sure we can ignore internal pytest warnings using a warnings filter."""
    testdir.makepyfile(
        """
        import pytest
        import warnings

        warnings.warn(pytest.PytestWarning("some internal warning"))

        def test_bar():
            pass
    """
    )
    if ignore_pytest_warnings == "ini":
        testdir.makeini(
            """
            [pytest]
            filterwarnings = ignore::pytest.PytestWarning
        """
        )
    args = (
        ["-W", "ignore::pytest.PytestWarning"]
        if ignore_pytest_warnings == "cmdline"
        else []
    )
    result = testdir.runpytest(*args)
    if ignore_pytest_warnings != "no":
        assert WARNINGS_SUMMARY_HEADER not in result.stdout.str()
    else:
        result.stdout.fnmatch_lines(
            [
                "*== %s ==*" % WARNINGS_SUMMARY_HEADER,
                "*test_hide_pytest_internal_warnings.py:4: PytestWarning: some internal warning",
                "* 1 passed, 1 warning *",
            ]
        )


@pytest.mark.parametrize("ignore_on_cmdline", [True, False])
def test_option_precedence_cmdline_over_ini(testdir, ignore_on_cmdline):
    """Filters defined in the command-line should take precedence over filters in ini files (#3946)."""
    testdir.makeini(
        """
        [pytest]
        filterwarnings = error
    """
    )
    testdir.makepyfile(
        """
        import warnings
        def test():
            warnings.warn(UserWarning('hello'))
    """
    )
    args = ["-W", "ignore"] if ignore_on_cmdline else []
    result = testdir.runpytest(*args)
    if ignore_on_cmdline:
        result.stdout.fnmatch_lines(["* 1 passed in*"])
    else:
        result.stdout.fnmatch_lines(["* 1 failed in*"])


def test_option_precedence_mark(testdir):
    """Filters defined by marks should always take precedence (#3946)."""
    testdir.makeini(
        """
        [pytest]
        filterwarnings = ignore
    """
    )
    testdir.makepyfile(
        """
        import pytest, warnings
        @pytest.mark.filterwarnings('error')
        def test():
            warnings.warn(UserWarning('hello'))
    """
    )
    result = testdir.runpytest("-W", "ignore")
    result.stdout.fnmatch_lines(["* 1 failed in*"])


class TestDeprecationWarningsByDefault:
    """
    Note: all pytest runs are executed in a subprocess so we don't inherit warning filters
    from pytest's own test suite
    """

    def create_file(self, testdir, mark=""):
        testdir.makepyfile(
            """
            import pytest, warnings

            warnings.warn(DeprecationWarning("collection"))

            {mark}
            def test_foo():
                warnings.warn(PendingDeprecationWarning("test run"))
        """.format(
                mark=mark
            )
        )

    @pytest.mark.parametrize("customize_filters", [True, False])
    def test_shown_by_default(self, testdir, customize_filters):
        """Show deprecation warnings by default, even if user has customized the warnings filters (#4013)."""
        self.create_file(testdir)
        if customize_filters:
            testdir.makeini(
                """
                [pytest]
                filterwarnings =
                    once::UserWarning
            """
            )
        result = testdir.runpytest_subprocess()
        result.stdout.fnmatch_lines(
            [
                "*== %s ==*" % WARNINGS_SUMMARY_HEADER,
                "*test_shown_by_default.py:3: DeprecationWarning: collection",
                "*test_shown_by_default.py:7: PendingDeprecationWarning: test run",
                "* 1 passed, 2 warnings*",
            ]
        )

    def test_hidden_by_ini(self, testdir):
        self.create_file(testdir)
        testdir.makeini(
            """
            [pytest]
            filterwarnings =
                ignore::DeprecationWarning
                ignore::PendingDeprecationWarning
        """
        )
        result = testdir.runpytest_subprocess()
        assert WARNINGS_SUMMARY_HEADER not in result.stdout.str()

    def test_hidden_by_mark(self, testdir):
        """Should hide the deprecation warning from the function, but the warning during collection should
        be displayed normally.
        """
        self.create_file(
            testdir,
            mark='@pytest.mark.filterwarnings("ignore::PendingDeprecationWarning")',
        )
        result = testdir.runpytest_subprocess()
        result.stdout.fnmatch_lines(
            [
                "*== %s ==*" % WARNINGS_SUMMARY_HEADER,
                "*test_hidden_by_mark.py:3: DeprecationWarning: collection",
                "* 1 passed, 1 warning*",
            ]
        )

    def test_hidden_by_cmdline(self, testdir):
        self.create_file(testdir)
        result = testdir.runpytest_subprocess(
            "-W",
            "ignore::DeprecationWarning",
            "-W",
            "ignore::PendingDeprecationWarning",
        )
        assert WARNINGS_SUMMARY_HEADER not in result.stdout.str()

    def test_hidden_by_system(self, testdir, monkeypatch):
        self.create_file(testdir)
        monkeypatch.setenv("PYTHONWARNINGS", "once::UserWarning")
        result = testdir.runpytest_subprocess()
        assert WARNINGS_SUMMARY_HEADER not in result.stdout.str()


@pytest.mark.parametrize("change_default", [None, "ini", "cmdline"])
@pytest.mark.skip(
    reason="This test should be enabled again before pytest 7.0 is released"
)
def test_deprecation_warning_as_error(testdir, change_default):
    """This ensures that PytestDeprecationWarnings raised by pytest are turned into errors.

    This test should be enabled as part of each major release, and skipped again afterwards
    to ensure our deprecations are turning into warnings as expected.
    """
    testdir.makepyfile(
        """
        import warnings, pytest
        def test():
            warnings.warn(pytest.PytestDeprecationWarning("some warning"))
    """
    )
    if change_default == "ini":
        testdir.makeini(
            """
            [pytest]
            filterwarnings =
                ignore::pytest.PytestDeprecationWarning
        """
        )

    args = (
        ("-Wignore::pytest.PytestDeprecationWarning",)
        if change_default == "cmdline"
        else ()
    )
    result = testdir.runpytest(*args)
    if change_default is None:
        result.stdout.fnmatch_lines(["* 1 failed in *"])
    else:
        assert change_default in ("ini", "cmdline")
        result.stdout.fnmatch_lines(["* 1 passed in *"])


class TestAssertionWarnings:
    @staticmethod
    def assert_result_warns(result, msg):
        result.stdout.fnmatch_lines(["*PytestAssertRewriteWarning: %s*" % msg])

    def test_tuple_warning(self, testdir):
        testdir.makepyfile(
            """\
            def test_foo():
                assert (1,2)
            """
        )
        result = testdir.runpytest()
        self.assert_result_warns(
            result, "assertion is always true, perhaps remove parentheses?"
        )


def test_warnings_checker_twice():
    """Issue #4617"""
    expectation = pytest.warns(UserWarning)
    with expectation:
        warnings.warn("Message A", UserWarning)
    with expectation:
        warnings.warn("Message B", UserWarning)


@pytest.mark.filterwarnings("ignore::pytest.PytestExperimentalApiWarning")
@pytest.mark.filterwarnings("always")
def test_group_warnings_by_message(testdir):
    testdir.copy_example("warnings/test_group_warnings_by_message.py")
    result = testdir.runpytest()
    result.stdout.fnmatch_lines(
        [
            "*== %s ==*" % WARNINGS_SUMMARY_HEADER,
            "test_group_warnings_by_message.py::test_foo[[]0[]]",
            "test_group_warnings_by_message.py::test_foo[[]1[]]",
            "test_group_warnings_by_message.py::test_foo[[]2[]]",
            "test_group_warnings_by_message.py::test_foo[[]3[]]",
            "test_group_warnings_by_message.py::test_foo[[]4[]]",
            "test_group_warnings_by_message.py::test_foo_1",
            "  */test_group_warnings_by_message.py:*: UserWarning: foo",
            "    warnings.warn(UserWarning(msg))",
            "",
            "test_group_warnings_by_message.py::test_bar[[]0[]]",
            "test_group_warnings_by_message.py::test_bar[[]1[]]",
            "test_group_warnings_by_message.py::test_bar[[]2[]]",
            "test_group_warnings_by_message.py::test_bar[[]3[]]",
            "test_group_warnings_by_message.py::test_bar[[]4[]]",
            "  */test_group_warnings_by_message.py:*: UserWarning: bar",
            "    warnings.warn(UserWarning(msg))",
            "",
            "-- Docs: *",
            "*= 11 passed, 11 warnings *",
        ],
        consecutive=True,
    )


@pytest.mark.filterwarnings("ignore::pytest.PytestExperimentalApiWarning")
@pytest.mark.filterwarnings("always")
def test_group_warnings_by_message_summary(testdir):
    testdir.copy_example("warnings/test_group_warnings_by_message_summary")
    testdir.syspathinsert()
    result = testdir.runpytest()
    result.stdout.fnmatch_lines(
        [
            "*== %s ==*" % WARNINGS_SUMMARY_HEADER,
            "test_1.py: 21 warnings",
            "test_2.py: 1 warning",
            "  */test_1.py:7: UserWarning: foo",
            "    warnings.warn(UserWarning(msg))",
            "",
            "test_1.py: 20 warnings",
            "  */test_1.py:7: UserWarning: bar",
            "    warnings.warn(UserWarning(msg))",
            "",
            "-- Docs: *",
            "*= 42 passed, 42 warnings *",
        ],
        consecutive=True,
    )


def test_pytest_configure_warning(testdir, recwarn):
    """Issue 5115."""
    testdir.makeconftest(
        """
        def pytest_configure():
            import warnings

            warnings.warn("from pytest_configure")
        """
    )

    result = testdir.runpytest()
    assert result.ret == 5
    assert "INTERNALERROR" not in result.stderr.str()
    warning = recwarn.pop()
    assert str(warning.message) == "from pytest_configure"


class TestStackLevel:
    @pytest.fixture
    def capwarn(self, testdir):
        class CapturedWarnings:
            captured = (
                []
            )  # type: List[Tuple[warnings.WarningMessage, Optional[Tuple[str, int, str]]]]

            @classmethod
            def pytest_warning_recorded(cls, warning_message, when, nodeid, location):
                cls.captured.append((warning_message, location))

        testdir.plugins = [CapturedWarnings()]

        return CapturedWarnings

    def test_issue4445_rewrite(self, testdir, capwarn):
        """#4445: Make sure the warning points to a reasonable location
        See origin of _issue_warning_captured at: _pytest.assertion.rewrite.py:241
        """
        testdir.makepyfile(some_mod="")
        conftest = testdir.makeconftest(
            """
                import some_mod
                import pytest

                pytest.register_assert_rewrite("some_mod")
            """
        )
        testdir.parseconfig()

        # with stacklevel=5 the warning originates from register_assert_rewrite
        # function in the created conftest.py
        assert len(capwarn.captured) == 1
        warning, location = capwarn.captured.pop()
        file, lineno, func = location

        assert "Module already imported" in str(warning.message)
        assert file == str(conftest)
        assert func == "<module>"  # the above conftest.py
        assert lineno == 4

    def test_issue4445_preparse(self, testdir, capwarn):
        """#4445: Make sure the warning points to a reasonable location
        See origin of _issue_warning_captured at: _pytest.config.__init__.py:910
        """
        testdir.makeconftest(
            """
            import nothing
            """
        )
        testdir.parseconfig("--help")

        # with stacklevel=2 the warning should originate from config._preparse and is
        # thrown by an errorneous conftest.py
        assert len(capwarn.captured) == 1
        warning, location = capwarn.captured.pop()
        file, _, func = location

        assert "could not load initial conftests" in str(warning.message)
        assert "config{sep}__init__.py".format(sep=os.sep) in file
        assert func == "_preparse"

    @pytest.mark.filterwarnings("default")
    def test_conftest_warning_captured(self, testdir: Testdir) -> None:
        """Warnings raised during importing of conftest.py files is captured (#2891)."""
        testdir.makeconftest(
            """
            import warnings
            warnings.warn(UserWarning("my custom warning"))
            """
        )
        result = testdir.runpytest()
        result.stdout.fnmatch_lines(
            ["conftest.py:2", "*UserWarning: my custom warning*"]
        )

    def test_issue4445_import_plugin(self, testdir, capwarn):
        """#4445: Make sure the warning points to a reasonable location"""
        testdir.makepyfile(
            some_plugin="""
            import pytest
            pytest.skip("thing", allow_module_level=True)
            """
        )
        testdir.syspathinsert()
        testdir.parseconfig("-p", "some_plugin")

        # with stacklevel=2 the warning should originate from
        # config.PytestPluginManager.import_plugin is thrown by a skipped plugin

        assert len(capwarn.captured) == 1
        warning, location = capwarn.captured.pop()
        file, _, func = location

        assert "skipped plugin 'some_plugin': thing" in str(warning.message)
        assert "config{sep}__init__.py".format(sep=os.sep) in file
        assert func == "_warn_about_skipped_plugins"

    def test_issue4445_issue5928_mark_generator(self, testdir):
        """#4445 and #5928: Make sure the warning from an unknown mark points to
        the test file where this mark is used.
        """
        testfile = testdir.makepyfile(
            """
            import pytest

            @pytest.mark.unknown
            def test_it():
                pass
            """
        )
        result = testdir.runpytest_subprocess()
        # with stacklevel=2 the warning should originate from the above created test file
        result.stdout.fnmatch_lines_random(
            [
                "*{testfile}:3*".format(testfile=str(testfile)),
                "*Unknown pytest.mark.unknown*",
            ]
        )
