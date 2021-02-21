import os
import shutil
import stat
import sys

import py

import pytest
from _pytest.config import ExitCode
from _pytest.pytester import Testdir

pytest_plugins = ("pytester",)


class TestNewAPI:
    def test_config_cache_makedir(self, testdir):
        testdir.makeini("[pytest]")
        config = testdir.parseconfigure()
        with pytest.raises(ValueError):
            config.cache.makedir("key/name")

        p = config.cache.makedir("name")
        assert p.check()

    def test_config_cache_dataerror(self, testdir):
        testdir.makeini("[pytest]")
        config = testdir.parseconfigure()
        cache = config.cache
        pytest.raises(TypeError, lambda: cache.set("key/name", cache))
        config.cache.set("key/name", 0)
        config.cache._getvaluepath("key/name").write_bytes(b"123invalid")
        val = config.cache.get("key/name", -2)
        assert val == -2

    @pytest.mark.filterwarnings("ignore:could not create cache path")
    def test_cache_writefail_cachfile_silent(self, testdir):
        testdir.makeini("[pytest]")
        testdir.tmpdir.join(".pytest_cache").write("gone wrong")
        config = testdir.parseconfigure()
        cache = config.cache
        cache.set("test/broken", [])

    @pytest.mark.skipif(sys.platform.startswith("win"), reason="no chmod on windows")
    @pytest.mark.filterwarnings(
        "ignore:could not create cache path:pytest.PytestWarning"
    )
    def test_cache_writefail_permissions(self, testdir):
        testdir.makeini("[pytest]")
        cache_dir = str(testdir.tmpdir.ensure_dir(".pytest_cache"))
        mode = os.stat(cache_dir)[stat.ST_MODE]
        testdir.tmpdir.ensure_dir(".pytest_cache").chmod(0)
        try:
            config = testdir.parseconfigure()
            cache = config.cache
            cache.set("test/broken", [])
        finally:
            testdir.tmpdir.ensure_dir(".pytest_cache").chmod(mode)

    @pytest.mark.skipif(sys.platform.startswith("win"), reason="no chmod on windows")
    @pytest.mark.filterwarnings("default")
    def test_cache_failure_warns(self, testdir, monkeypatch):
        monkeypatch.setenv("PYTEST_DISABLE_PLUGIN_AUTOLOAD", "1")
        cache_dir = str(testdir.tmpdir.ensure_dir(".pytest_cache"))
        mode = os.stat(cache_dir)[stat.ST_MODE]
        testdir.tmpdir.ensure_dir(".pytest_cache").chmod(0)
        try:
            testdir.makepyfile("def test_error(): raise Exception")
            result = testdir.runpytest()
            assert result.ret == 1
            # warnings from nodeids, lastfailed, and stepwise
            result.stdout.fnmatch_lines(
                [
                    # Validate location/stacklevel of warning from cacheprovider.
                    "*= warnings summary =*",
                    "*/cacheprovider.py:*",
                    "  */cacheprovider.py:*: PytestCacheWarning: could not create cache path "
                    "{}/v/cache/nodeids".format(cache_dir),
                    '    config.cache.set("cache/nodeids", sorted(self.cached_nodeids))',
                    "*1 failed, 3 warnings in*",
                ]
            )
        finally:
            testdir.tmpdir.ensure_dir(".pytest_cache").chmod(mode)

    def test_config_cache(self, testdir):
        testdir.makeconftest(
            """
            def pytest_configure(config):
                # see that we get cache information early on
                assert hasattr(config, "cache")
        """
        )
        testdir.makepyfile(
            """
            def test_session(pytestconfig):
                assert hasattr(pytestconfig, "cache")
        """
        )
        result = testdir.runpytest()
        assert result.ret == 0
        result.stdout.fnmatch_lines(["*1 passed*"])

    def test_cachefuncarg(self, testdir):
        testdir.makepyfile(
            """
            import pytest
            def test_cachefuncarg(cache):
                val = cache.get("some/thing", None)
                assert val is None
                cache.set("some/thing", [1])
                pytest.raises(TypeError, lambda: cache.get("some/thing"))
                val = cache.get("some/thing", [])
                assert val == [1]
        """
        )
        result = testdir.runpytest()
        assert result.ret == 0
        result.stdout.fnmatch_lines(["*1 passed*"])

    def test_custom_rel_cache_dir(self, testdir):
        rel_cache_dir = os.path.join("custom_cache_dir", "subdir")
        testdir.makeini(
            """
            [pytest]
            cache_dir = {cache_dir}
        """.format(
                cache_dir=rel_cache_dir
            )
        )
        testdir.makepyfile(test_errored="def test_error():\n    assert False")
        testdir.runpytest()
        assert testdir.tmpdir.join(rel_cache_dir).isdir()

    def test_custom_abs_cache_dir(self, testdir, tmpdir_factory):
        tmp = str(tmpdir_factory.mktemp("tmp"))
        abs_cache_dir = os.path.join(tmp, "custom_cache_dir")
        testdir.makeini(
            """
            [pytest]
            cache_dir = {cache_dir}
        """.format(
                cache_dir=abs_cache_dir
            )
        )
        testdir.makepyfile(test_errored="def test_error():\n    assert False")
        testdir.runpytest()
        assert py.path.local(abs_cache_dir).isdir()

    def test_custom_cache_dir_with_env_var(self, testdir, monkeypatch):
        monkeypatch.setenv("env_var", "custom_cache_dir")
        testdir.makeini(
            """
            [pytest]
            cache_dir = {cache_dir}
        """.format(
                cache_dir="$env_var"
            )
        )
        testdir.makepyfile(test_errored="def test_error():\n    assert False")
        testdir.runpytest()
        assert testdir.tmpdir.join("custom_cache_dir").isdir()


@pytest.mark.parametrize("env", ((), ("TOX_ENV_DIR", "/tox_env_dir")))
def test_cache_reportheader(env, testdir, monkeypatch):
    testdir.makepyfile("""def test_foo(): pass""")
    if env:
        monkeypatch.setenv(*env)
        expected = os.path.join(env[1], ".pytest_cache")
    else:
        monkeypatch.delenv("TOX_ENV_DIR", raising=False)
        expected = ".pytest_cache"
    result = testdir.runpytest("-v")
    result.stdout.fnmatch_lines(["cachedir: %s" % expected])


def test_cache_reportheader_external_abspath(testdir, tmpdir_factory):
    external_cache = tmpdir_factory.mktemp(
        "test_cache_reportheader_external_abspath_abs"
    )

    testdir.makepyfile("def test_hello(): pass")
    testdir.makeini(
        """
    [pytest]
    cache_dir = {abscache}
    """.format(
            abscache=external_cache
        )
    )
    result = testdir.runpytest("-v")
    result.stdout.fnmatch_lines(
        ["cachedir: {abscache}".format(abscache=external_cache)]
    )


def test_cache_show(testdir):
    result = testdir.runpytest("--cache-show")
    assert result.ret == 0
    result.stdout.fnmatch_lines(["*cache is empty*"])
    testdir.makeconftest(
        """
        def pytest_configure(config):
            config.cache.set("my/name", [1,2,3])
            config.cache.set("my/hello", "world")
            config.cache.set("other/some", {1:2})
            dp = config.cache.makedir("mydb")
            dp.ensure("hello")
            dp.ensure("world")
    """
    )
    result = testdir.runpytest()
    assert result.ret == 5  # no tests executed

    result = testdir.runpytest("--cache-show")
    result.stdout.fnmatch_lines(
        [
            "*cachedir:*",
            "*- cache values for '[*]' -*",
            "cache/nodeids contains:",
            "my/name contains:",
            "  [1, 2, 3]",
            "other/some contains:",
            "  {*'1': 2}",
            "*- cache directories for '[*]' -*",
            "*mydb/hello*length 0*",
            "*mydb/world*length 0*",
        ]
    )
    assert result.ret == 0

    result = testdir.runpytest("--cache-show", "*/hello")
    result.stdout.fnmatch_lines(
        [
            "*cachedir:*",
            "*- cache values for '[*]/hello' -*",
            "my/hello contains:",
            "  *'world'",
            "*- cache directories for '[*]/hello' -*",
            "d/mydb/hello*length 0*",
        ]
    )
    stdout = result.stdout.str()
    assert "other/some" not in stdout
    assert "d/mydb/world" not in stdout
    assert result.ret == 0


class TestLastFailed:
    def test_lastfailed_usecase(self, testdir, monkeypatch):
        monkeypatch.setattr("sys.dont_write_bytecode", True)
        p = testdir.makepyfile(
            """
            def test_1(): assert 0
            def test_2(): assert 0
            def test_3(): assert 1
            """
        )
        result = testdir.runpytest(str(p))
        result.stdout.fnmatch_lines(["*2 failed*"])
        p = testdir.makepyfile(
            """
            def test_1(): assert 1
            def test_2(): assert 1
            def test_3(): assert 0
            """
        )
        result = testdir.runpytest(str(p), "--lf")
        result.stdout.fnmatch_lines(
            [
                "collected 3 items / 1 deselected / 2 selected",
                "run-last-failure: rerun previous 2 failures",
                "*= 2 passed, 1 deselected in *",
            ]
        )
        result = testdir.runpytest(str(p), "--lf")
        result.stdout.fnmatch_lines(
            [
                "collected 3 items",
                "run-last-failure: no previously failed tests, not deselecting items.",
                "*1 failed*2 passed*",
            ]
        )
        testdir.tmpdir.join(".pytest_cache").mkdir(".git")
        result = testdir.runpytest(str(p), "--lf", "--cache-clear")
        result.stdout.fnmatch_lines(["*1 failed*2 passed*"])
        assert testdir.tmpdir.join(".pytest_cache", "README.md").isfile()
        assert testdir.tmpdir.join(".pytest_cache", ".git").isdir()

        # Run this again to make sure clear-cache is robust
        if os.path.isdir(".pytest_cache"):
            shutil.rmtree(".pytest_cache")
        result = testdir.runpytest("--lf", "--cache-clear")
        result.stdout.fnmatch_lines(["*1 failed*2 passed*"])

    def test_failedfirst_order(self, testdir):
        testdir.makepyfile(
            test_a="def test_always_passes(): pass",
            test_b="def test_always_fails(): assert 0",
        )
        result = testdir.runpytest()
        # Test order will be collection order; alphabetical
        result.stdout.fnmatch_lines(["test_a.py*", "test_b.py*"])
        result = testdir.runpytest("--ff")
        # Test order will be failing tests first
        result.stdout.fnmatch_lines(
            [
                "collected 2 items",
                "run-last-failure: rerun previous 1 failure first",
                "test_b.py*",
                "test_a.py*",
            ]
        )

    def test_lastfailed_failedfirst_order(self, testdir):
        testdir.makepyfile(
            test_a="def test_always_passes(): assert 1",
            test_b="def test_always_fails(): assert 0",
        )
        result = testdir.runpytest()
        # Test order will be collection order; alphabetical
        result.stdout.fnmatch_lines(["test_a.py*", "test_b.py*"])
        result = testdir.runpytest("--lf", "--ff")
        # Test order will be failing tests first
        result.stdout.fnmatch_lines(["test_b.py*"])
        result.stdout.no_fnmatch_line("*test_a.py*")

    def test_lastfailed_difference_invocations(self, testdir, monkeypatch):
        monkeypatch.setattr("sys.dont_write_bytecode", True)
        testdir.makepyfile(
            test_a="""
                def test_a1(): assert 0
                def test_a2(): assert 1
            """,
            test_b="def test_b1(): assert 0",
        )
        p = testdir.tmpdir.join("test_a.py")
        p2 = testdir.tmpdir.join("test_b.py")

        result = testdir.runpytest()
        result.stdout.fnmatch_lines(["*2 failed*"])
        result = testdir.runpytest("--lf", p2)
        result.stdout.fnmatch_lines(["*1 failed*"])

        testdir.makepyfile(test_b="def test_b1(): assert 1")
        result = testdir.runpytest("--lf", p2)
        result.stdout.fnmatch_lines(["*1 passed*"])
        result = testdir.runpytest("--lf", p)
        result.stdout.fnmatch_lines(
            [
                "collected 2 items / 1 deselected / 1 selected",
                "run-last-failure: rerun previous 1 failure",
                "*= 1 failed, 1 deselected in *",
            ]
        )

    def test_lastfailed_usecase_splice(self, testdir, monkeypatch):
        monkeypatch.setattr("sys.dont_write_bytecode", True)
        testdir.makepyfile(
            "def test_1(): assert 0", test_something="def test_2(): assert 0"
        )
        p2 = testdir.tmpdir.join("test_something.py")
        result = testdir.runpytest()
        result.stdout.fnmatch_lines(["*2 failed*"])
        result = testdir.runpytest("--lf", p2)
        result.stdout.fnmatch_lines(["*1 failed*"])
        result = testdir.runpytest("--lf")
        result.stdout.fnmatch_lines(["*2 failed*"])

    def test_lastfailed_xpass(self, testdir):
        testdir.inline_runsource(
            """
            import pytest
            @pytest.mark.xfail
            def test_hello():
                assert 1
        """
        )
        config = testdir.parseconfigure()
        lastfailed = config.cache.get("cache/lastfailed", -1)
        assert lastfailed == -1

    def test_non_serializable_parametrize(self, testdir):
        """Test that failed parametrized tests with unmarshable parameters
        don't break pytest-cache.
        """
        testdir.makepyfile(
            r"""
            import pytest

            @pytest.mark.parametrize('val', [
                b'\xac\x10\x02G',
            ])
            def test_fail(val):
                assert False
        """
        )
        result = testdir.runpytest()
        result.stdout.fnmatch_lines(["*1 failed in*"])

    def test_terminal_report_lastfailed(self, testdir):
        test_a = testdir.makepyfile(
            test_a="""
            def test_a1(): pass
            def test_a2(): pass
        """
        )
        test_b = testdir.makepyfile(
            test_b="""
            def test_b1(): assert 0
            def test_b2(): assert 0
        """
        )
        result = testdir.runpytest()
        result.stdout.fnmatch_lines(["collected 4 items", "*2 failed, 2 passed in*"])

        result = testdir.runpytest("--lf")
        result.stdout.fnmatch_lines(
            [
                "collected 2 items",
                "run-last-failure: rerun previous 2 failures (skipped 1 file)",
                "*2 failed in*",
            ]
        )

        result = testdir.runpytest(test_a, "--lf")
        result.stdout.fnmatch_lines(
            [
                "collected 2 items",
                "run-last-failure: 2 known failures not in selected tests",
                "*2 passed in*",
            ]
        )

        result = testdir.runpytest(test_b, "--lf")
        result.stdout.fnmatch_lines(
            [
                "collected 2 items",
                "run-last-failure: rerun previous 2 failures",
                "*2 failed in*",
            ]
        )

        result = testdir.runpytest("test_b.py::test_b1", "--lf")
        result.stdout.fnmatch_lines(
            [
                "collected 1 item",
                "run-last-failure: rerun previous 1 failure",
                "*1 failed in*",
            ]
        )

    def test_terminal_report_failedfirst(self, testdir):
        testdir.makepyfile(
            test_a="""
            def test_a1(): assert 0
            def test_a2(): pass
        """
        )
        result = testdir.runpytest()
        result.stdout.fnmatch_lines(["collected 2 items", "*1 failed, 1 passed in*"])

        result = testdir.runpytest("--ff")
        result.stdout.fnmatch_lines(
            [
                "collected 2 items",
                "run-last-failure: rerun previous 1 failure first",
                "*1 failed, 1 passed in*",
            ]
        )

    def test_lastfailed_collectfailure(self, testdir, monkeypatch):

        testdir.makepyfile(
            test_maybe="""
            import os
            env = os.environ
            if '1' == env['FAILIMPORT']:
                raise ImportError('fail')
            def test_hello():
                assert '0' == env['FAILTEST']
        """
        )

        def rlf(fail_import, fail_run):
            monkeypatch.setenv("FAILIMPORT", str(fail_import))
            monkeypatch.setenv("FAILTEST", str(fail_run))

            testdir.runpytest("-q")
            config = testdir.parseconfigure()
            lastfailed = config.cache.get("cache/lastfailed", -1)
            return lastfailed

        lastfailed = rlf(fail_import=0, fail_run=0)
        assert lastfailed == -1

        lastfailed = rlf(fail_import=1, fail_run=0)
        assert list(lastfailed) == ["test_maybe.py"]

        lastfailed = rlf(fail_import=0, fail_run=1)
        assert list(lastfailed) == ["test_maybe.py::test_hello"]

    def test_lastfailed_failure_subset(self, testdir, monkeypatch):
        testdir.makepyfile(
            test_maybe="""
            import os
            env = os.environ
            if '1' == env['FAILIMPORT']:
                raise ImportError('fail')
            def test_hello():
                assert '0' == env['FAILTEST']
        """
        )

        testdir.makepyfile(
            test_maybe2="""
            import os
            env = os.environ
            if '1' == env['FAILIMPORT']:
                raise ImportError('fail')

            def test_hello():
                assert '0' == env['FAILTEST']

            def test_pass():
                pass
        """
        )

        def rlf(fail_import, fail_run, args=()):
            monkeypatch.setenv("FAILIMPORT", str(fail_import))
            monkeypatch.setenv("FAILTEST", str(fail_run))

            result = testdir.runpytest("-q", "--lf", *args)
            config = testdir.parseconfigure()
            lastfailed = config.cache.get("cache/lastfailed", -1)
            return result, lastfailed

        result, lastfailed = rlf(fail_import=0, fail_run=0)
        assert lastfailed == -1
        result.stdout.fnmatch_lines(["*3 passed*"])

        result, lastfailed = rlf(fail_import=1, fail_run=0)
        assert sorted(list(lastfailed)) == ["test_maybe.py", "test_maybe2.py"]

        result, lastfailed = rlf(fail_import=0, fail_run=0, args=("test_maybe2.py",))
        assert list(lastfailed) == ["test_maybe.py"]

        # edge case of test selection - even if we remember failures
        # from other tests we still need to run all tests if no test
        # matches the failures
        result, lastfailed = rlf(fail_import=0, fail_run=0, args=("test_maybe2.py",))
        assert list(lastfailed) == ["test_maybe.py"]
        result.stdout.fnmatch_lines(["*2 passed*"])

    def test_lastfailed_creates_cache_when_needed(self, testdir):
        # Issue #1342
        testdir.makepyfile(test_empty="")
        testdir.runpytest("-q", "--lf")
        assert not os.path.exists(".pytest_cache/v/cache/lastfailed")

        testdir.makepyfile(test_successful="def test_success():\n    assert True")
        testdir.runpytest("-q", "--lf")
        assert not os.path.exists(".pytest_cache/v/cache/lastfailed")

        testdir.makepyfile(test_errored="def test_error():\n    assert False")
        testdir.runpytest("-q", "--lf")
        assert os.path.exists(".pytest_cache/v/cache/lastfailed")

    def test_xfail_not_considered_failure(self, testdir):
        testdir.makepyfile(
            """
            import pytest
            @pytest.mark.xfail
            def test(): assert 0
        """
        )
        result = testdir.runpytest()
        result.stdout.fnmatch_lines(["*1 xfailed*"])
        assert self.get_cached_last_failed(testdir) == []

    def test_xfail_strict_considered_failure(self, testdir):
        testdir.makepyfile(
            """
            import pytest
            @pytest.mark.xfail(strict=True)
            def test(): pass
        """
        )
        result = testdir.runpytest()
        result.stdout.fnmatch_lines(["*1 failed*"])
        assert self.get_cached_last_failed(testdir) == [
            "test_xfail_strict_considered_failure.py::test"
        ]

    @pytest.mark.parametrize("mark", ["mark.xfail", "mark.skip"])
    def test_failed_changed_to_xfail_or_skip(self, testdir, mark):
        testdir.makepyfile(
            """
            import pytest
            def test(): assert 0
        """
        )
        result = testdir.runpytest()
        assert self.get_cached_last_failed(testdir) == [
            "test_failed_changed_to_xfail_or_skip.py::test"
        ]
        assert result.ret == 1

        testdir.makepyfile(
            """
            import pytest
            @pytest.{mark}
            def test(): assert 0
        """.format(
                mark=mark
            )
        )
        result = testdir.runpytest()
        assert result.ret == 0
        assert self.get_cached_last_failed(testdir) == []
        assert result.ret == 0

    @pytest.mark.parametrize("quiet", [True, False])
    @pytest.mark.parametrize("opt", ["--ff", "--lf"])
    def test_lf_and_ff_prints_no_needless_message(self, quiet, opt, testdir):
        # Issue 3853
        testdir.makepyfile("def test(): assert 0")
        args = [opt]
        if quiet:
            args.append("-q")
        result = testdir.runpytest(*args)
        result.stdout.no_fnmatch_line("*run all*")

        result = testdir.runpytest(*args)
        if quiet:
            result.stdout.no_fnmatch_line("*run all*")
        else:
            assert "rerun previous" in result.stdout.str()

    def get_cached_last_failed(self, testdir):
        config = testdir.parseconfigure()
        return sorted(config.cache.get("cache/lastfailed", {}))

    def test_cache_cumulative(self, testdir):
        """Test workflow where user fixes errors gradually file by file using --lf."""
        # 1. initial run
        test_bar = testdir.makepyfile(
            test_bar="""
            def test_bar_1(): pass
            def test_bar_2(): assert 0
        """
        )
        test_foo = testdir.makepyfile(
            test_foo="""
            def test_foo_3(): pass
            def test_foo_4(): assert 0
        """
        )
        testdir.runpytest()
        assert self.get_cached_last_failed(testdir) == [
            "test_bar.py::test_bar_2",
            "test_foo.py::test_foo_4",
        ]

        # 2. fix test_bar_2, run only test_bar.py
        testdir.makepyfile(
            test_bar="""
            def test_bar_1(): pass
            def test_bar_2(): pass
        """
        )
        result = testdir.runpytest(test_bar)
        result.stdout.fnmatch_lines(["*2 passed*"])
        # ensure cache does not forget that test_foo_4 failed once before
        assert self.get_cached_last_failed(testdir) == ["test_foo.py::test_foo_4"]

        result = testdir.runpytest("--last-failed")
        result.stdout.fnmatch_lines(
            [
                "collected 1 item",
                "run-last-failure: rerun previous 1 failure (skipped 1 file)",
                "*= 1 failed in *",
            ]
        )
        assert self.get_cached_last_failed(testdir) == ["test_foo.py::test_foo_4"]

        # 3. fix test_foo_4, run only test_foo.py
        test_foo = testdir.makepyfile(
            test_foo="""
            def test_foo_3(): pass
            def test_foo_4(): pass
        """
        )
        result = testdir.runpytest(test_foo, "--last-failed")
        result.stdout.fnmatch_lines(
            [
                "collected 2 items / 1 deselected / 1 selected",
                "run-last-failure: rerun previous 1 failure",
                "*= 1 passed, 1 deselected in *",
            ]
        )
        assert self.get_cached_last_failed(testdir) == []

        result = testdir.runpytest("--last-failed")
        result.stdout.fnmatch_lines(["*4 passed*"])
        assert self.get_cached_last_failed(testdir) == []

    def test_lastfailed_no_failures_behavior_all_passed(self, testdir):
        testdir.makepyfile(
            """
            def test_1(): pass
            def test_2(): pass
        """
        )
        result = testdir.runpytest()
        result.stdout.fnmatch_lines(["*2 passed*"])
        result = testdir.runpytest("--lf")
        result.stdout.fnmatch_lines(["*2 passed*"])
        result = testdir.runpytest("--lf", "--lfnf", "all")
        result.stdout.fnmatch_lines(["*2 passed*"])

        # Ensure the list passed to pytest_deselected is a copy,
        # and not a reference which is cleared right after.
        testdir.makeconftest(
            """
            deselected = []

            def pytest_deselected(items):
                global deselected
                deselected = items

            def pytest_sessionfinish():
                print("\\ndeselected={}".format(len(deselected)))
        """
        )

        result = testdir.runpytest("--lf", "--lfnf", "none")
        result.stdout.fnmatch_lines(
            [
                "collected 2 items / 2 deselected",
                "run-last-failure: no previously failed tests, deselecting all items.",
                "deselected=2",
                "* 2 deselected in *",
            ]
        )
        assert result.ret == ExitCode.NO_TESTS_COLLECTED

    def test_lastfailed_no_failures_behavior_empty_cache(self, testdir):
        testdir.makepyfile(
            """
            def test_1(): pass
            def test_2(): assert 0
        """
        )
        result = testdir.runpytest("--lf", "--cache-clear")
        result.stdout.fnmatch_lines(["*1 failed*1 passed*"])
        result = testdir.runpytest("--lf", "--cache-clear", "--lfnf", "all")
        result.stdout.fnmatch_lines(["*1 failed*1 passed*"])
        result = testdir.runpytest("--lf", "--cache-clear", "--lfnf", "none")
        result.stdout.fnmatch_lines(["*2 desel*"])

    def test_lastfailed_skip_collection(self, testdir):
        """
        Test --lf behavior regarding skipping collection of files that are not marked as
        failed in the cache (#5172).
        """
        testdir.makepyfile(
            **{
                "pkg1/test_1.py": """
                import pytest

                @pytest.mark.parametrize('i', range(3))
                def test_1(i): pass
            """,
                "pkg2/test_2.py": """
                import pytest

                @pytest.mark.parametrize('i', range(5))
                def test_1(i):
                    assert i not in (1, 3)
            """,
            }
        )
        # first run: collects 8 items (test_1: 3, test_2: 5)
        result = testdir.runpytest()
        result.stdout.fnmatch_lines(["collected 8 items", "*2 failed*6 passed*"])
        # second run: collects only 5 items from test_2, because all tests from test_1 have passed
        result = testdir.runpytest("--lf")
        result.stdout.fnmatch_lines(
            [
                "collected 2 items",
                "run-last-failure: rerun previous 2 failures (skipped 1 file)",
                "*= 2 failed in *",
            ]
        )

        # add another file and check if message is correct when skipping more than 1 file
        testdir.makepyfile(
            **{
                "pkg1/test_3.py": """
                def test_3(): pass
            """
            }
        )
        result = testdir.runpytest("--lf")
        result.stdout.fnmatch_lines(
            [
                "collected 2 items",
                "run-last-failure: rerun previous 2 failures (skipped 2 files)",
                "*= 2 failed in *",
            ]
        )

    def test_lastfailed_with_known_failures_not_being_selected(self, testdir):
        testdir.makepyfile(
            **{
                "pkg1/test_1.py": """def test_1(): assert 0""",
                "pkg1/test_2.py": """def test_2(): pass""",
            }
        )
        result = testdir.runpytest()
        result.stdout.fnmatch_lines(["collected 2 items", "* 1 failed, 1 passed in *"])

        py.path.local("pkg1/test_1.py").remove()
        result = testdir.runpytest("--lf")
        result.stdout.fnmatch_lines(
            [
                "collected 1 item",
                "run-last-failure: 1 known failures not in selected tests",
                "* 1 passed in *",
            ]
        )

        # Recreate file with known failure.
        testdir.makepyfile(**{"pkg1/test_1.py": """def test_1(): assert 0"""})
        result = testdir.runpytest("--lf")
        result.stdout.fnmatch_lines(
            [
                "collected 1 item",
                "run-last-failure: rerun previous 1 failure (skipped 1 file)",
                "* 1 failed in *",
            ]
        )

        # Remove/rename test: collects the file again.
        testdir.makepyfile(**{"pkg1/test_1.py": """def test_renamed(): assert 0"""})
        result = testdir.runpytest("--lf", "-rf")
        result.stdout.fnmatch_lines(
            [
                "collected 2 items",
                "run-last-failure: 1 known failures not in selected tests",
                "pkg1/test_1.py F *",
                "pkg1/test_2.py . *",
                "FAILED pkg1/test_1.py::test_renamed - assert 0",
                "* 1 failed, 1 passed in *",
            ]
        )

        result = testdir.runpytest("--lf", "--co")
        result.stdout.fnmatch_lines(
            [
                "collected 1 item",
                "run-last-failure: rerun previous 1 failure (skipped 1 file)",
                "",
                "<Module pkg1/test_1.py>",
                "  <Function test_renamed>",
            ]
        )

    def test_lastfailed_args_with_deselected(self, testdir: Testdir) -> None:
        """Test regression with --lf running into NoMatch error.

        This was caused by it not collecting (non-failed) nodes given as
        arguments.
        """
        testdir.makepyfile(
            **{
                "pkg1/test_1.py": """
                    def test_pass(): pass
                    def test_fail(): assert 0
                """,
            }
        )
        result = testdir.runpytest()
        result.stdout.fnmatch_lines(["collected 2 items", "* 1 failed, 1 passed in *"])
        assert result.ret == 1

        result = testdir.runpytest("pkg1/test_1.py::test_pass", "--lf", "--co")
        assert result.ret == 0
        result.stdout.fnmatch_lines(
            [
                "*collected 1 item",
                "run-last-failure: 1 known failures not in selected tests",
                "",
                "<Module pkg1/test_1.py>",
                "  <Function test_pass>",
            ],
            consecutive=True,
        )

        result = testdir.runpytest(
            "pkg1/test_1.py::test_pass", "pkg1/test_1.py::test_fail", "--lf", "--co"
        )
        assert result.ret == 0
        result.stdout.fnmatch_lines(
            [
                "collected 2 items / 1 deselected / 1 selected",
                "run-last-failure: rerun previous 1 failure",
                "",
                "<Module pkg1/test_1.py>",
                "  <Function test_fail>",
                "*= 1 deselected in *",
            ],
        )

    def test_lastfailed_with_class_items(self, testdir: Testdir) -> None:
        """Test regression with --lf deselecting whole classes."""
        testdir.makepyfile(
            **{
                "pkg1/test_1.py": """
                    class TestFoo:
                        def test_pass(self): pass
                        def test_fail(self): assert 0

                    def test_other(): assert 0
                """,
            }
        )
        result = testdir.runpytest()
        result.stdout.fnmatch_lines(["collected 3 items", "* 2 failed, 1 passed in *"])
        assert result.ret == 1

        result = testdir.runpytest("--lf", "--co")
        assert result.ret == 0
        result.stdout.fnmatch_lines(
            [
                "collected 3 items / 1 deselected / 2 selected",
                "run-last-failure: rerun previous 2 failures",
                "",
                "<Module pkg1/test_1.py>",
                "  <Class TestFoo>",
                "      <Function test_fail>",
                "  <Function test_other>",
                "",
                "*= 1 deselected in *",
            ],
            consecutive=True,
        )

    def test_lastfailed_with_all_filtered(self, testdir: Testdir) -> None:
        testdir.makepyfile(
            **{
                "pkg1/test_1.py": """
                    def test_fail(): assert 0
                    def test_pass(): pass
                """,
            }
        )
        result = testdir.runpytest()
        result.stdout.fnmatch_lines(["collected 2 items", "* 1 failed, 1 passed in *"])
        assert result.ret == 1

        # Remove known failure.
        testdir.makepyfile(
            **{
                "pkg1/test_1.py": """
                    def test_pass(): pass
                """,
            }
        )
        result = testdir.runpytest("--lf", "--co")
        result.stdout.fnmatch_lines(
            [
                "collected 1 item",
                "run-last-failure: 1 known failures not in selected tests",
                "",
                "<Module pkg1/test_1.py>",
                "  <Function test_pass>",
                "",
                "*= no tests ran in*",
            ],
            consecutive=True,
        )
        assert result.ret == 0


class TestNewFirst:
    def test_newfirst_usecase(self, testdir):
        testdir.makepyfile(
            **{
                "test_1/test_1.py": """
                def test_1(): assert 1
            """,
                "test_2/test_2.py": """
                def test_1(): assert 1
            """,
            }
        )
        testdir.tmpdir.join("test_1/test_1.py").setmtime(1)

        result = testdir.runpytest("-v")
        result.stdout.fnmatch_lines(
            ["*test_1/test_1.py::test_1 PASSED*", "*test_2/test_2.py::test_1 PASSED*"]
        )

        result = testdir.runpytest("-v", "--nf")
        result.stdout.fnmatch_lines(
            ["*test_2/test_2.py::test_1 PASSED*", "*test_1/test_1.py::test_1 PASSED*"]
        )

        testdir.tmpdir.join("test_1/test_1.py").write(
            "def test_1(): assert 1\n" "def test_2(): assert 1\n"
        )
        testdir.tmpdir.join("test_1/test_1.py").setmtime(1)

        result = testdir.runpytest("--nf", "--collect-only", "-q")
        result.stdout.fnmatch_lines(
            [
                "test_1/test_1.py::test_2",
                "test_2/test_2.py::test_1",
                "test_1/test_1.py::test_1",
            ]
        )

        # Newest first with (plugin) pytest_collection_modifyitems hook.
        testdir.makepyfile(
            myplugin="""
            def pytest_collection_modifyitems(items):
                items[:] = sorted(items, key=lambda item: item.nodeid)
                print("new_items:", [x.nodeid for x in items])
            """
        )
        testdir.syspathinsert()
        result = testdir.runpytest("--nf", "-p", "myplugin", "--collect-only", "-q")
        result.stdout.fnmatch_lines(
            [
                "new_items: *test_1.py*test_1.py*test_2.py*",
                "test_1/test_1.py::test_2",
                "test_2/test_2.py::test_1",
                "test_1/test_1.py::test_1",
            ]
        )

    def test_newfirst_parametrize(self, testdir):
        testdir.makepyfile(
            **{
                "test_1/test_1.py": """
                import pytest
                @pytest.mark.parametrize('num', [1, 2])
                def test_1(num): assert num
            """,
                "test_2/test_2.py": """
                import pytest
                @pytest.mark.parametrize('num', [1, 2])
                def test_1(num): assert num
            """,
            }
        )

        testdir.tmpdir.join("test_1/test_1.py").setmtime(1)

        result = testdir.runpytest("-v")
        result.stdout.fnmatch_lines(
            [
                "*test_1/test_1.py::test_1[1*",
                "*test_1/test_1.py::test_1[2*",
                "*test_2/test_2.py::test_1[1*",
                "*test_2/test_2.py::test_1[2*",
            ]
        )

        result = testdir.runpytest("-v", "--nf")
        result.stdout.fnmatch_lines(
            [
                "*test_2/test_2.py::test_1[1*",
                "*test_2/test_2.py::test_1[2*",
                "*test_1/test_1.py::test_1[1*",
                "*test_1/test_1.py::test_1[2*",
            ]
        )

        testdir.tmpdir.join("test_1/test_1.py").write(
            "import pytest\n"
            "@pytest.mark.parametrize('num', [1, 2, 3])\n"
            "def test_1(num): assert num\n"
        )
        testdir.tmpdir.join("test_1/test_1.py").setmtime(1)

        # Running only a subset does not forget about existing ones.
        result = testdir.runpytest("-v", "--nf", "test_2/test_2.py")
        result.stdout.fnmatch_lines(
            ["*test_2/test_2.py::test_1[1*", "*test_2/test_2.py::test_1[2*"]
        )

        result = testdir.runpytest("-v", "--nf")
        result.stdout.fnmatch_lines(
            [
                "*test_1/test_1.py::test_1[3*",
                "*test_2/test_2.py::test_1[1*",
                "*test_2/test_2.py::test_1[2*",
                "*test_1/test_1.py::test_1[1*",
                "*test_1/test_1.py::test_1[2*",
            ]
        )


class TestReadme:
    def check_readme(self, testdir):
        config = testdir.parseconfigure()
        readme = config.cache._cachedir.joinpath("README.md")
        return readme.is_file()

    def test_readme_passed(self, testdir):
        testdir.makepyfile("def test_always_passes(): pass")
        testdir.runpytest()
        assert self.check_readme(testdir) is True

    def test_readme_failed(self, testdir):
        testdir.makepyfile("def test_always_fails(): assert 0")
        testdir.runpytest()
        assert self.check_readme(testdir) is True


def test_gitignore(testdir):
    """Ensure we automatically create .gitignore file in the pytest_cache directory (#3286)."""
    from _pytest.cacheprovider import Cache

    config = testdir.parseconfig()
    cache = Cache.for_config(config)
    cache.set("foo", "bar")
    msg = "# Created by pytest automatically.\n*\n"
    gitignore_path = cache._cachedir.joinpath(".gitignore")
    assert gitignore_path.read_text(encoding="UTF-8") == msg

    # Does not overwrite existing/custom one.
    gitignore_path.write_text("custom")
    cache.set("something", "else")
    assert gitignore_path.read_text(encoding="UTF-8") == "custom"


def test_does_not_create_boilerplate_in_existing_dirs(testdir):
    from _pytest.cacheprovider import Cache

    testdir.makeini(
        """
        [pytest]
        cache_dir = .
        """
    )
    config = testdir.parseconfig()
    cache = Cache.for_config(config)
    cache.set("foo", "bar")

    assert os.path.isdir("v")  # cache contents
    assert not os.path.exists(".gitignore")
    assert not os.path.exists("README.md")


def test_cachedir_tag(testdir):
    """Ensure we automatically create CACHEDIR.TAG file in the pytest_cache directory (#4278)."""
    from _pytest.cacheprovider import Cache
    from _pytest.cacheprovider import CACHEDIR_TAG_CONTENT

    config = testdir.parseconfig()
    cache = Cache.for_config(config)
    cache.set("foo", "bar")
    cachedir_tag_path = cache._cachedir.joinpath("CACHEDIR.TAG")
    assert cachedir_tag_path.read_bytes() == CACHEDIR_TAG_CONTENT
