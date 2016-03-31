import sys

import _pytest
import pytest
import os
import shutil

pytest_plugins = "pytester",

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
        config.cache._getvaluepath("key/name").write("123invalid")
        val = config.cache.get("key/name", -2)
        assert val == -2

    def test_cache_writefail_cachfile_silent(self, testdir):
        testdir.makeini("[pytest]")
        testdir.tmpdir.join('.cache').write('gone wrong')
        config = testdir.parseconfigure()
        cache = config.cache
        cache.set('test/broken', [])

    @pytest.mark.skipif(sys.platform.startswith('win'), reason='no chmod on windows')
    def test_cache_writefail_permissions(self, testdir):
        testdir.makeini("[pytest]")
        testdir.tmpdir.ensure_dir('.cache').chmod(0)
        config = testdir.parseconfigure()
        cache = config.cache
        cache.set('test/broken', [])

    @pytest.mark.skipif(sys.platform.startswith('win'), reason='no chmod on windows')
    def test_cache_failure_warns(self, testdir):
        testdir.tmpdir.ensure_dir('.cache').chmod(0)
        testdir.makepyfile("""
            def test_error():
                raise Exception

        """)
        result = testdir.runpytest('-rw')
        assert result.ret == 1
        result.stdout.fnmatch_lines([
            "*could not create cache path*",
            "*1 pytest-warnings*",
        ])

    def test_config_cache(self, testdir):
        testdir.makeconftest("""
            def pytest_configure(config):
                # see that we get cache information early on
                assert hasattr(config, "cache")
        """)
        testdir.makepyfile("""
            def test_session(pytestconfig):
                assert hasattr(pytestconfig, "cache")
        """)
        result = testdir.runpytest()
        assert result.ret == 0
        result.stdout.fnmatch_lines(["*1 passed*"])

    def test_cachefuncarg(self, testdir):
        testdir.makepyfile("""
            import pytest
            def test_cachefuncarg(cache):
                val = cache.get("some/thing", None)
                assert val is None
                cache.set("some/thing", [1])
                pytest.raises(TypeError, lambda: cache.get("some/thing"))
                val = cache.get("some/thing", [])
                assert val == [1]
        """)
        result = testdir.runpytest()
        assert result.ret == 0
        result.stdout.fnmatch_lines(["*1 passed*"])



def test_cache_reportheader(testdir):
    testdir.makepyfile("""
        def test_hello():
            pass
    """)
    result = testdir.runpytest("-v")
    result.stdout.fnmatch_lines([
        "cachedir: .cache"
    ])


def test_cache_show(testdir):
    result = testdir.runpytest("--cache-show")
    assert result.ret == 0
    result.stdout.fnmatch_lines([
        "*cache is empty*"
    ])
    testdir.makeconftest("""
        def pytest_configure(config):
            config.cache.set("my/name", [1,2,3])
            config.cache.set("other/some", {1:2})
            dp = config.cache.makedir("mydb")
            dp.ensure("hello")
            dp.ensure("world")
    """)
    result = testdir.runpytest()
    assert result.ret == 5  # no tests executed
    result = testdir.runpytest("--cache-show")
    result.stdout.fnmatch_lines_random([
        "*cachedir:*",
        "-*cache values*-",
        "*my/name contains:",
        "  [1, 2, 3]",
        "*other/some contains*",
        "  {*1*: 2}",
        "-*cache directories*-",
        "*mydb/hello*length 0*",
        "*mydb/world*length 0*",
    ])


class TestLastFailed:

    def test_lastfailed_usecase(self, testdir, monkeypatch):
        monkeypatch.setenv("PYTHONDONTWRITEBYTECODE", 1)
        p = testdir.makepyfile("""
            def test_1():
                assert 0
            def test_2():
                assert 0
            def test_3():
                assert 1
        """)
        result = testdir.runpytest()
        result.stdout.fnmatch_lines([
            "*2 failed*",
        ])
        p.write(_pytest._code.Source("""
            def test_1():
                assert 1

            def test_2():
                assert 1

            def test_3():
                assert 0
        """))
        result = testdir.runpytest("--lf")
        result.stdout.fnmatch_lines([
            "*2 passed*1 desel*",
        ])
        result = testdir.runpytest("--lf")
        result.stdout.fnmatch_lines([
            "*1 failed*2 passed*",
        ])
        result = testdir.runpytest("--lf", "--cache-clear")
        result.stdout.fnmatch_lines([
            "*1 failed*2 passed*",
        ])

        # Run this again to make sure clear-cache is robust
        if os.path.isdir('.cache'):
            shutil.rmtree('.cache')
        result = testdir.runpytest("--lf", "--cache-clear")
        result.stdout.fnmatch_lines([
            "*1 failed*2 passed*",
        ])

    def test_failedfirst_order(self, testdir):
        testdir.tmpdir.join('test_a.py').write(_pytest._code.Source("""
            def test_always_passes():
                assert 1
        """))
        testdir.tmpdir.join('test_b.py').write(_pytest._code.Source("""
            def test_always_fails():
                assert 0
        """))
        result = testdir.runpytest()
        # Test order will be collection order; alphabetical
        result.stdout.fnmatch_lines([
            "test_a.py*",
            "test_b.py*",
        ])
        result = testdir.runpytest("--lf", "--ff")
        # Test order will be failing tests firs
        result.stdout.fnmatch_lines([
            "test_b.py*",
            "test_a.py*",
        ])

    def test_lastfailed_difference_invocations(self, testdir, monkeypatch):
        monkeypatch.setenv("PYTHONDONTWRITEBYTECODE", 1)
        testdir.makepyfile(test_a="""
            def test_a1():
                assert 0
            def test_a2():
                assert 1
        """, test_b="""
            def test_b1():
                assert 0
        """)
        p = testdir.tmpdir.join("test_a.py")
        p2 = testdir.tmpdir.join("test_b.py")

        result = testdir.runpytest()
        result.stdout.fnmatch_lines([
            "*2 failed*",
        ])
        result = testdir.runpytest("--lf", p2)
        result.stdout.fnmatch_lines([
            "*1 failed*",
        ])
        p2.write(_pytest._code.Source("""
            def test_b1():
                assert 1
        """))
        result = testdir.runpytest("--lf", p2)
        result.stdout.fnmatch_lines([
            "*1 passed*",
        ])
        result = testdir.runpytest("--lf", p)
        result.stdout.fnmatch_lines([
            "*1 failed*1 desel*",
        ])

    def test_lastfailed_usecase_splice(self, testdir, monkeypatch):
        monkeypatch.setenv("PYTHONDONTWRITEBYTECODE", 1)
        testdir.makepyfile("""
            def test_1():
                assert 0
        """)
        p2 = testdir.tmpdir.join("test_something.py")
        p2.write(_pytest._code.Source("""
            def test_2():
                assert 0
        """))
        result = testdir.runpytest()
        result.stdout.fnmatch_lines([
            "*2 failed*",
        ])
        result = testdir.runpytest("--lf", p2)
        result.stdout.fnmatch_lines([
            "*1 failed*",
        ])
        result = testdir.runpytest("--lf")
        result.stdout.fnmatch_lines([
            "*2 failed*",
        ])

    def test_lastfailed_xpass(self, testdir):
        testdir.inline_runsource("""
            import pytest
            @pytest.mark.xfail
            def test_hello():
                assert 1
        """)
        config = testdir.parseconfigure()
        lastfailed = config.cache.get("cache/lastfailed", -1)
        assert lastfailed == -1

    def test_non_serializable_parametrize(self, testdir):
        """Test that failed parametrized tests with unmarshable parameters
        don't break pytest-cache.
        """
        testdir.makepyfile(r"""
            import pytest

            @pytest.mark.parametrize('val', [
                b'\xac\x10\x02G',
            ])
            def test_fail(val):
                assert False
        """)
        result = testdir.runpytest()
        result.stdout.fnmatch_lines('*1 failed in*')

    def test_lastfailed_collectfailure(self, testdir, monkeypatch):

        testdir.makepyfile(test_maybe="""
            import py
            env = py.std.os.environ
            if '1' == env['FAILIMPORT']:
                raise ImportError('fail')
            def test_hello():
                assert '0' == env['FAILTEST']
        """)

        def rlf(fail_import, fail_run):
            monkeypatch.setenv('FAILIMPORT', fail_import)
            monkeypatch.setenv('FAILTEST', fail_run)

            testdir.runpytest('-q')
            config = testdir.parseconfigure()
            lastfailed = config.cache.get("cache/lastfailed", -1)
            return lastfailed

        lastfailed = rlf(fail_import=0, fail_run=0)
        assert lastfailed == -1

        lastfailed = rlf(fail_import=1, fail_run=0)
        assert list(lastfailed) == ['test_maybe.py']

        lastfailed = rlf(fail_import=0, fail_run=1)
        assert list(lastfailed) == ['test_maybe.py::test_hello']


    def test_lastfailed_failure_subset(self, testdir, monkeypatch):

        testdir.makepyfile(test_maybe="""
            import py
            env = py.std.os.environ
            if '1' == env['FAILIMPORT']:
                raise ImportError('fail')
            def test_hello():
                assert '0' == env['FAILTEST']
        """)

        testdir.makepyfile(test_maybe2="""
            import py
            env = py.std.os.environ
            if '1' == env['FAILIMPORT']:
                raise ImportError('fail')
            def test_hello():
                assert '0' == env['FAILTEST']

            def test_pass():
                pass
        """)

        def rlf(fail_import, fail_run, args=()):
            monkeypatch.setenv('FAILIMPORT', fail_import)
            monkeypatch.setenv('FAILTEST', fail_run)

            result = testdir.runpytest('-q', '--lf', *args)
            config = testdir.parseconfigure()
            lastfailed = config.cache.get("cache/lastfailed", -1)
            return result, lastfailed

        result, lastfailed = rlf(fail_import=0, fail_run=0)
        assert lastfailed == -1
        result.stdout.fnmatch_lines([
            '*3 passed*',
        ])

        result, lastfailed = rlf(fail_import=1, fail_run=0)
        assert sorted(list(lastfailed)) == ['test_maybe.py', 'test_maybe2.py']


        result, lastfailed = rlf(fail_import=0, fail_run=0,
                                 args=('test_maybe2.py',))
        assert list(lastfailed) == ['test_maybe.py']


        # edge case of test selection - even if we remember failures
        # from other tests we still need to run all tests if no test
        # matches the failures
        result, lastfailed = rlf(fail_import=0, fail_run=0,
                                 args=('test_maybe2.py',))
        assert list(lastfailed) == ['test_maybe.py']
        result.stdout.fnmatch_lines([
            '*2 passed*',
        ])

    def test_lastfailed_creates_cache_when_needed(self, testdir):
        # Issue #1342
        testdir.makepyfile(test_empty='')
        testdir.runpytest('-q', '--lf')
        assert not os.path.exists('.cache')

        testdir.makepyfile(test_successful='def test_success():\n    assert True')
        testdir.runpytest('-q', '--lf')
        assert not os.path.exists('.cache')

        testdir.makepyfile(test_errored='def test_error():\n    assert False')
        testdir.runpytest('-q', '--lf')
        assert os.path.exists('.cache')
