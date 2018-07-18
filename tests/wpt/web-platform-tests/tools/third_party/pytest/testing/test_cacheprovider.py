from __future__ import absolute_import, division, print_function
import sys
import py
import _pytest
import pytest
import os
import shutil

pytest_plugins = "pytester",


class TestNewAPI(object):

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
        config.cache._getvaluepath("key/name").write("123invalid")
        val = config.cache.get("key/name", -2)
        assert val == -2

    def test_cache_writefail_cachfile_silent(self, testdir):
        testdir.makeini("[pytest]")
        testdir.tmpdir.join(".pytest_cache").write("gone wrong")
        config = testdir.parseconfigure()
        cache = config.cache
        cache.set("test/broken", [])

    @pytest.mark.skipif(sys.platform.startswith("win"), reason="no chmod on windows")
    def test_cache_writefail_permissions(self, testdir):
        testdir.makeini("[pytest]")
        testdir.tmpdir.ensure_dir(".pytest_cache").chmod(0)
        config = testdir.parseconfigure()
        cache = config.cache
        cache.set("test/broken", [])

    @pytest.mark.skipif(sys.platform.startswith("win"), reason="no chmod on windows")
    def test_cache_failure_warns(self, testdir):
        testdir.tmpdir.ensure_dir(".pytest_cache").chmod(0)
        testdir.makepyfile(
            """
            def test_error():
                raise Exception

        """
        )
        result = testdir.runpytest("-rw")
        assert result.ret == 1
        result.stdout.fnmatch_lines(["*could not create cache path*", "*2 warnings*"])

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


def test_cache_reportheader(testdir):
    testdir.makepyfile(
        """
        def test_hello():
            pass
    """
    )
    result = testdir.runpytest("-v")
    result.stdout.fnmatch_lines(["cachedir: .pytest_cache"])


def test_cache_show(testdir):
    result = testdir.runpytest("--cache-show")
    assert result.ret == 0
    result.stdout.fnmatch_lines(["*cache is empty*"])
    testdir.makeconftest(
        """
        def pytest_configure(config):
            config.cache.set("my/name", [1,2,3])
            config.cache.set("other/some", {1:2})
            dp = config.cache.makedir("mydb")
            dp.ensure("hello")
            dp.ensure("world")
    """
    )
    result = testdir.runpytest()
    assert result.ret == 5  # no tests executed
    result = testdir.runpytest("--cache-show")
    result.stdout.fnmatch_lines_random(
        [
            "*cachedir:*",
            "-*cache values*-",
            "*my/name contains:",
            "  [1, 2, 3]",
            "*other/some contains*",
            "  {*1*: 2}",
            "-*cache directories*-",
            "*mydb/hello*length 0*",
            "*mydb/world*length 0*",
        ]
    )


class TestLastFailed(object):

    def test_lastfailed_usecase(self, testdir, monkeypatch):
        monkeypatch.setenv("PYTHONDONTWRITEBYTECODE", 1)
        p = testdir.makepyfile(
            """
            def test_1():
                assert 0
            def test_2():
                assert 0
            def test_3():
                assert 1
        """
        )
        result = testdir.runpytest()
        result.stdout.fnmatch_lines(["*2 failed*"])
        p.write(
            _pytest._code.Source(
                """
            def test_1():
                assert 1

            def test_2():
                assert 1

            def test_3():
                assert 0
        """
            )
        )
        result = testdir.runpytest("--lf")
        result.stdout.fnmatch_lines(["*2 passed*1 desel*"])
        result = testdir.runpytest("--lf")
        result.stdout.fnmatch_lines(["*1 failed*2 passed*"])
        result = testdir.runpytest("--lf", "--cache-clear")
        result.stdout.fnmatch_lines(["*1 failed*2 passed*"])

        # Run this again to make sure clear-cache is robust
        if os.path.isdir(".pytest_cache"):
            shutil.rmtree(".pytest_cache")
        result = testdir.runpytest("--lf", "--cache-clear")
        result.stdout.fnmatch_lines(["*1 failed*2 passed*"])

    def test_failedfirst_order(self, testdir):
        testdir.tmpdir.join("test_a.py").write(
            _pytest._code.Source(
                """
            def test_always_passes():
                assert 1
        """
            )
        )
        testdir.tmpdir.join("test_b.py").write(
            _pytest._code.Source(
                """
            def test_always_fails():
                assert 0
        """
            )
        )
        result = testdir.runpytest()
        # Test order will be collection order; alphabetical
        result.stdout.fnmatch_lines(["test_a.py*", "test_b.py*"])
        result = testdir.runpytest("--ff")
        # Test order will be failing tests firs
        result.stdout.fnmatch_lines(["test_b.py*", "test_a.py*"])

    def test_lastfailed_failedfirst_order(self, testdir):
        testdir.makepyfile(
            **{
                "test_a.py": """
                def test_always_passes():
                    assert 1
            """,
                "test_b.py": """
                def test_always_fails():
                    assert 0
            """,
            }
        )
        result = testdir.runpytest()
        # Test order will be collection order; alphabetical
        result.stdout.fnmatch_lines(["test_a.py*", "test_b.py*"])
        result = testdir.runpytest("--lf", "--ff")
        # Test order will be failing tests firs
        result.stdout.fnmatch_lines(["test_b.py*"])
        assert "test_a.py" not in result.stdout.str()

    def test_lastfailed_difference_invocations(self, testdir, monkeypatch):
        monkeypatch.setenv("PYTHONDONTWRITEBYTECODE", 1)
        testdir.makepyfile(
            test_a="""
            def test_a1():
                assert 0
            def test_a2():
                assert 1
        """,
            test_b="""
            def test_b1():
                assert 0
        """,
        )
        p = testdir.tmpdir.join("test_a.py")
        p2 = testdir.tmpdir.join("test_b.py")

        result = testdir.runpytest()
        result.stdout.fnmatch_lines(["*2 failed*"])
        result = testdir.runpytest("--lf", p2)
        result.stdout.fnmatch_lines(["*1 failed*"])
        p2.write(
            _pytest._code.Source(
                """
            def test_b1():
                assert 1
        """
            )
        )
        result = testdir.runpytest("--lf", p2)
        result.stdout.fnmatch_lines(["*1 passed*"])
        result = testdir.runpytest("--lf", p)
        result.stdout.fnmatch_lines(["*1 failed*1 desel*"])

    def test_lastfailed_usecase_splice(self, testdir, monkeypatch):
        monkeypatch.setenv("PYTHONDONTWRITEBYTECODE", 1)
        testdir.makepyfile(
            """
            def test_1():
                assert 0
        """
        )
        p2 = testdir.tmpdir.join("test_something.py")
        p2.write(
            _pytest._code.Source(
                """
            def test_2():
                assert 0
        """
            )
        )
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
        result.stdout.fnmatch_lines("*1 failed in*")

    def test_terminal_report_lastfailed(self, testdir):
        test_a = testdir.makepyfile(
            test_a="""
            def test_a1():
                pass
            def test_a2():
                pass
        """
        )
        test_b = testdir.makepyfile(
            test_b="""
            def test_b1():
                assert 0
            def test_b2():
                assert 0
        """
        )
        result = testdir.runpytest()
        result.stdout.fnmatch_lines(["collected 4 items", "*2 failed, 2 passed in*"])

        result = testdir.runpytest("--lf")
        result.stdout.fnmatch_lines(
            [
                "collected 4 items / 2 deselected",
                "run-last-failure: rerun previous 2 failures",
                "*2 failed, 2 deselected in*",
            ]
        )

        result = testdir.runpytest(test_a, "--lf")
        result.stdout.fnmatch_lines(
            [
                "collected 2 items",
                "run-last-failure: run all (no recorded failures)",
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
            def test_a1():
                assert 0
            def test_a2():
                pass
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
            monkeypatch.setenv("FAILIMPORT", fail_import)
            monkeypatch.setenv("FAILTEST", fail_run)

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
            monkeypatch.setenv("FAILIMPORT", fail_import)
            monkeypatch.setenv("FAILTEST", fail_run)

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
            def test():
                assert 0
        """
        )
        result = testdir.runpytest()
        result.stdout.fnmatch_lines("*1 xfailed*")
        assert self.get_cached_last_failed(testdir) == []

    def test_xfail_strict_considered_failure(self, testdir):
        testdir.makepyfile(
            """
            import pytest
            @pytest.mark.xfail(strict=True)
            def test():
                pass
        """
        )
        result = testdir.runpytest()
        result.stdout.fnmatch_lines("*1 failed*")
        assert (
            self.get_cached_last_failed(testdir)
            == ["test_xfail_strict_considered_failure.py::test"]
        )

    @pytest.mark.parametrize("mark", ["mark.xfail", "mark.skip"])
    def test_failed_changed_to_xfail_or_skip(self, testdir, mark):
        testdir.makepyfile(
            """
            import pytest
            def test():
                assert 0
        """
        )
        result = testdir.runpytest()
        assert (
            self.get_cached_last_failed(testdir)
            == ["test_failed_changed_to_xfail_or_skip.py::test"]
        )
        assert result.ret == 1

        testdir.makepyfile(
            """
            import pytest
            @pytest.{mark}
            def test():
                assert 0
        """.format(
                mark=mark
            )
        )
        result = testdir.runpytest()
        assert result.ret == 0
        assert self.get_cached_last_failed(testdir) == []
        assert result.ret == 0

    def get_cached_last_failed(self, testdir):
        config = testdir.parseconfigure()
        return sorted(config.cache.get("cache/lastfailed", {}))

    def test_cache_cumulative(self, testdir):
        """
        Test workflow where user fixes errors gradually file by file using --lf.
        """
        # 1. initial run
        test_bar = testdir.makepyfile(
            test_bar="""
            def test_bar_1():
                pass
            def test_bar_2():
                assert 0
        """
        )
        test_foo = testdir.makepyfile(
            test_foo="""
            def test_foo_3():
                pass
            def test_foo_4():
                assert 0
        """
        )
        testdir.runpytest()
        assert (
            self.get_cached_last_failed(testdir)
            == ["test_bar.py::test_bar_2", "test_foo.py::test_foo_4"]
        )

        # 2. fix test_bar_2, run only test_bar.py
        testdir.makepyfile(
            test_bar="""
            def test_bar_1():
                pass
            def test_bar_2():
                pass
        """
        )
        result = testdir.runpytest(test_bar)
        result.stdout.fnmatch_lines("*2 passed*")
        # ensure cache does not forget that test_foo_4 failed once before
        assert self.get_cached_last_failed(testdir) == ["test_foo.py::test_foo_4"]

        result = testdir.runpytest("--last-failed")
        result.stdout.fnmatch_lines("*1 failed, 3 deselected*")
        assert self.get_cached_last_failed(testdir) == ["test_foo.py::test_foo_4"]

        # 3. fix test_foo_4, run only test_foo.py
        test_foo = testdir.makepyfile(
            test_foo="""
            def test_foo_3():
                pass
            def test_foo_4():
                pass
        """
        )
        result = testdir.runpytest(test_foo, "--last-failed")
        result.stdout.fnmatch_lines("*1 passed, 1 deselected*")
        assert self.get_cached_last_failed(testdir) == []

        result = testdir.runpytest("--last-failed")
        result.stdout.fnmatch_lines("*4 passed*")
        assert self.get_cached_last_failed(testdir) == []

    def test_lastfailed_no_failures_behavior_all_passed(self, testdir):
        testdir.makepyfile(
            """
            def test_1():
                assert True
            def test_2():
                assert True
        """
        )
        result = testdir.runpytest()
        result.stdout.fnmatch_lines(["*2 passed*"])
        result = testdir.runpytest("--lf")
        result.stdout.fnmatch_lines(["*2 passed*"])
        result = testdir.runpytest("--lf", "--lfnf", "all")
        result.stdout.fnmatch_lines(["*2 passed*"])
        result = testdir.runpytest("--lf", "--lfnf", "none")
        result.stdout.fnmatch_lines(["*2 desel*"])

    def test_lastfailed_no_failures_behavior_empty_cache(self, testdir):
        testdir.makepyfile(
            """
            def test_1():
                assert True
            def test_2():
                assert False
        """
        )
        result = testdir.runpytest("--lf", "--cache-clear")
        result.stdout.fnmatch_lines(["*1 failed*1 passed*"])
        result = testdir.runpytest("--lf", "--cache-clear", "--lfnf", "all")
        result.stdout.fnmatch_lines(["*1 failed*1 passed*"])
        result = testdir.runpytest("--lf", "--cache-clear", "--lfnf", "none")
        result.stdout.fnmatch_lines(["*2 desel*"])


class TestNewFirst(object):

    def test_newfirst_usecase(self, testdir):
        testdir.makepyfile(
            **{
                "test_1/test_1.py": """
                def test_1(): assert 1
                def test_2(): assert 1
                def test_3(): assert 1
            """,
                "test_2/test_2.py": """
                def test_1(): assert 1
                def test_2(): assert 1
                def test_3(): assert 1
            """,
            }
        )

        testdir.tmpdir.join("test_1/test_1.py").setmtime(1)

        result = testdir.runpytest("-v")
        result.stdout.fnmatch_lines(
            [
                "*test_1/test_1.py::test_1 PASSED*",
                "*test_1/test_1.py::test_2 PASSED*",
                "*test_1/test_1.py::test_3 PASSED*",
                "*test_2/test_2.py::test_1 PASSED*",
                "*test_2/test_2.py::test_2 PASSED*",
                "*test_2/test_2.py::test_3 PASSED*",
            ]
        )

        result = testdir.runpytest("-v", "--nf")

        result.stdout.fnmatch_lines(
            [
                "*test_2/test_2.py::test_1 PASSED*",
                "*test_2/test_2.py::test_2 PASSED*",
                "*test_2/test_2.py::test_3 PASSED*",
                "*test_1/test_1.py::test_1 PASSED*",
                "*test_1/test_1.py::test_2 PASSED*",
                "*test_1/test_1.py::test_3 PASSED*",
            ]
        )

        testdir.tmpdir.join("test_1/test_1.py").write(
            "def test_1(): assert 1\n"
            "def test_2(): assert 1\n"
            "def test_3(): assert 1\n"
            "def test_4(): assert 1\n"
        )
        testdir.tmpdir.join("test_1/test_1.py").setmtime(1)

        result = testdir.runpytest("-v", "--nf")

        result.stdout.fnmatch_lines(
            [
                "*test_1/test_1.py::test_4 PASSED*",
                "*test_2/test_2.py::test_1 PASSED*",
                "*test_2/test_2.py::test_2 PASSED*",
                "*test_2/test_2.py::test_3 PASSED*",
                "*test_1/test_1.py::test_1 PASSED*",
                "*test_1/test_1.py::test_2 PASSED*",
                "*test_1/test_1.py::test_3 PASSED*",
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
