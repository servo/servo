# -*- coding: utf-8 -*-
from __future__ import absolute_import
from __future__ import division
from __future__ import print_function

import os
import subprocess
import sys
import time

import py.path

import _pytest.pytester as pytester
import pytest
from _pytest.config import PytestPluginManager
from _pytest.main import EXIT_NOTESTSCOLLECTED
from _pytest.main import EXIT_OK
from _pytest.main import EXIT_TESTSFAILED
from _pytest.pytester import CwdSnapshot
from _pytest.pytester import HookRecorder
from _pytest.pytester import LineMatcher
from _pytest.pytester import SysModulesSnapshot
from _pytest.pytester import SysPathsSnapshot


def test_make_hook_recorder(testdir):
    item = testdir.getitem("def test_func(): pass")
    recorder = testdir.make_hook_recorder(item.config.pluginmanager)
    assert not recorder.getfailures()

    pytest.xfail("internal reportrecorder tests need refactoring")

    class rep(object):
        excinfo = None
        passed = False
        failed = True
        skipped = False
        when = "call"

    recorder.hook.pytest_runtest_logreport(report=rep)
    failures = recorder.getfailures()
    assert failures == [rep]
    failures = recorder.getfailures()
    assert failures == [rep]

    class rep(object):
        excinfo = None
        passed = False
        failed = False
        skipped = True
        when = "call"

    rep.passed = False
    rep.skipped = True
    recorder.hook.pytest_runtest_logreport(report=rep)

    modcol = testdir.getmodulecol("")
    rep = modcol.config.hook.pytest_make_collect_report(collector=modcol)
    rep.passed = False
    rep.failed = True
    rep.skipped = False
    recorder.hook.pytest_collectreport(report=rep)

    passed, skipped, failed = recorder.listoutcomes()
    assert not passed and skipped and failed

    numpassed, numskipped, numfailed = recorder.countoutcomes()
    assert numpassed == 0
    assert numskipped == 1
    assert numfailed == 1
    assert len(recorder.getfailedcollections()) == 1

    recorder.unregister()
    recorder.clear()
    recorder.hook.pytest_runtest_logreport(report=rep)
    pytest.raises(ValueError, recorder.getfailures)


def test_parseconfig(testdir):
    config1 = testdir.parseconfig()
    config2 = testdir.parseconfig()
    assert config2 != config1
    assert config1 != pytest.config


def test_testdir_runs_with_plugin(testdir):
    testdir.makepyfile(
        """
        pytest_plugins = "pytester"
        def test_hello(testdir):
            assert 1
    """
    )
    result = testdir.runpytest()
    result.assert_outcomes(passed=1)


def test_runresult_assertion_on_xfail(testdir):
    testdir.makepyfile(
        """
        import pytest

        pytest_plugins = "pytester"

        @pytest.mark.xfail
        def test_potato():
            assert False
    """
    )
    result = testdir.runpytest()
    result.assert_outcomes(xfailed=1)
    assert result.ret == 0


def test_runresult_assertion_on_xpassed(testdir):
    testdir.makepyfile(
        """
        import pytest

        pytest_plugins = "pytester"

        @pytest.mark.xfail
        def test_potato():
            assert True
    """
    )
    result = testdir.runpytest()
    result.assert_outcomes(xpassed=1)
    assert result.ret == 0


def test_runresult_repr():
    from _pytest.pytester import RunResult

    assert (
        repr(
            RunResult(ret="ret", outlines=[""], errlines=["some", "errors"], duration=1)
        )
        == "<RunResult ret='ret' len(stdout.lines)=1 len(stderr.lines)=2 duration=1.00s>"
    )


def test_xpassed_with_strict_is_considered_a_failure(testdir):
    testdir.makepyfile(
        """
        import pytest

        pytest_plugins = "pytester"

        @pytest.mark.xfail(strict=True)
        def test_potato():
            assert True
    """
    )
    result = testdir.runpytest()
    result.assert_outcomes(failed=1)
    assert result.ret != 0


def make_holder():
    class apiclass(object):
        def pytest_xyz(self, arg):
            "x"

        def pytest_xyz_noarg(self):
            "x"

    apimod = type(os)("api")

    def pytest_xyz(arg):
        "x"

    def pytest_xyz_noarg():
        "x"

    apimod.pytest_xyz = pytest_xyz
    apimod.pytest_xyz_noarg = pytest_xyz_noarg
    return apiclass, apimod


@pytest.mark.parametrize("holder", make_holder())
def test_hookrecorder_basic(holder):
    pm = PytestPluginManager()
    pm.add_hookspecs(holder)
    rec = HookRecorder(pm)
    pm.hook.pytest_xyz(arg=123)
    call = rec.popcall("pytest_xyz")
    assert call.arg == 123
    assert call._name == "pytest_xyz"
    pytest.raises(pytest.fail.Exception, rec.popcall, "abc")
    pm.hook.pytest_xyz_noarg()
    call = rec.popcall("pytest_xyz_noarg")
    assert call._name == "pytest_xyz_noarg"


def test_makepyfile_unicode(testdir):
    global unichr
    try:
        unichr(65)
    except NameError:
        unichr = chr
    testdir.makepyfile(unichr(0xFFFD))


def test_makepyfile_utf8(testdir):
    """Ensure makepyfile accepts utf-8 bytes as input (#2738)"""
    utf8_contents = u"""
        def setup_function(function):
            mixed_encoding = u'São Paulo'
    """.encode(
        "utf-8"
    )
    p = testdir.makepyfile(utf8_contents)
    assert u"mixed_encoding = u'São Paulo'".encode("utf-8") in p.read("rb")


class TestInlineRunModulesCleanup(object):
    def test_inline_run_test_module_not_cleaned_up(self, testdir):
        test_mod = testdir.makepyfile("def test_foo(): assert True")
        result = testdir.inline_run(str(test_mod))
        assert result.ret == EXIT_OK
        # rewrite module, now test should fail if module was re-imported
        test_mod.write("def test_foo(): assert False")
        result2 = testdir.inline_run(str(test_mod))
        assert result2.ret == EXIT_TESTSFAILED

    def spy_factory(self):
        class SysModulesSnapshotSpy(object):
            instances = []

            def __init__(self, preserve=None):
                SysModulesSnapshotSpy.instances.append(self)
                self._spy_restore_count = 0
                self._spy_preserve = preserve
                self.__snapshot = SysModulesSnapshot(preserve=preserve)

            def restore(self):
                self._spy_restore_count += 1
                return self.__snapshot.restore()

        return SysModulesSnapshotSpy

    def test_inline_run_taking_and_restoring_a_sys_modules_snapshot(
        self, testdir, monkeypatch
    ):
        spy_factory = self.spy_factory()
        monkeypatch.setattr(pytester, "SysModulesSnapshot", spy_factory)
        testdir.syspathinsert()
        original = dict(sys.modules)
        testdir.makepyfile(import1="# you son of a silly person")
        testdir.makepyfile(import2="# my hovercraft is full of eels")
        test_mod = testdir.makepyfile(
            """
            import import1
            def test_foo(): import import2"""
        )
        testdir.inline_run(str(test_mod))
        assert len(spy_factory.instances) == 1
        spy = spy_factory.instances[0]
        assert spy._spy_restore_count == 1
        assert sys.modules == original
        assert all(sys.modules[x] is original[x] for x in sys.modules)

    def test_inline_run_sys_modules_snapshot_restore_preserving_modules(
        self, testdir, monkeypatch
    ):
        spy_factory = self.spy_factory()
        monkeypatch.setattr(pytester, "SysModulesSnapshot", spy_factory)
        test_mod = testdir.makepyfile("def test_foo(): pass")
        testdir.inline_run(str(test_mod))
        spy = spy_factory.instances[0]
        assert not spy._spy_preserve("black_knight")
        assert spy._spy_preserve("zope")
        assert spy._spy_preserve("zope.interface")
        assert spy._spy_preserve("zopelicious")

    def test_external_test_module_imports_not_cleaned_up(self, testdir):
        testdir.syspathinsert()
        testdir.makepyfile(imported="data = 'you son of a silly person'")
        import imported

        test_mod = testdir.makepyfile(
            """
            def test_foo():
                import imported
                imported.data = 42"""
        )
        testdir.inline_run(str(test_mod))
        assert imported.data == 42


def test_assert_outcomes_after_pytest_error(testdir):
    testdir.makepyfile("def test_foo(): assert True")

    result = testdir.runpytest("--unexpected-argument")
    with pytest.raises(ValueError, match="Pytest terminal report not found"):
        result.assert_outcomes(passed=0)


def test_cwd_snapshot(tmpdir):
    foo = tmpdir.ensure("foo", dir=1)
    bar = tmpdir.ensure("bar", dir=1)
    foo.chdir()
    snapshot = CwdSnapshot()
    bar.chdir()
    assert py.path.local() == bar
    snapshot.restore()
    assert py.path.local() == foo


class TestSysModulesSnapshot(object):
    key = "my-test-module"

    def test_remove_added(self):
        original = dict(sys.modules)
        assert self.key not in sys.modules
        snapshot = SysModulesSnapshot()
        sys.modules[self.key] = "something"
        assert self.key in sys.modules
        snapshot.restore()
        assert sys.modules == original

    def test_add_removed(self, monkeypatch):
        assert self.key not in sys.modules
        monkeypatch.setitem(sys.modules, self.key, "something")
        assert self.key in sys.modules
        original = dict(sys.modules)
        snapshot = SysModulesSnapshot()
        del sys.modules[self.key]
        assert self.key not in sys.modules
        snapshot.restore()
        assert sys.modules == original

    def test_restore_reloaded(self, monkeypatch):
        assert self.key not in sys.modules
        monkeypatch.setitem(sys.modules, self.key, "something")
        assert self.key in sys.modules
        original = dict(sys.modules)
        snapshot = SysModulesSnapshot()
        sys.modules[self.key] = "something else"
        snapshot.restore()
        assert sys.modules == original

    def test_preserve_modules(self, monkeypatch):
        key = [self.key + str(i) for i in range(3)]
        assert not any(k in sys.modules for k in key)
        for i, k in enumerate(key):
            monkeypatch.setitem(sys.modules, k, "something" + str(i))
        original = dict(sys.modules)

        def preserve(name):
            return name in (key[0], key[1], "some-other-key")

        snapshot = SysModulesSnapshot(preserve=preserve)
        sys.modules[key[0]] = original[key[0]] = "something else0"
        sys.modules[key[1]] = original[key[1]] = "something else1"
        sys.modules[key[2]] = "something else2"
        snapshot.restore()
        assert sys.modules == original

    def test_preserve_container(self, monkeypatch):
        original = dict(sys.modules)
        assert self.key not in original
        replacement = dict(sys.modules)
        replacement[self.key] = "life of brian"
        snapshot = SysModulesSnapshot()
        monkeypatch.setattr(sys, "modules", replacement)
        snapshot.restore()
        assert sys.modules is replacement
        assert sys.modules == original


@pytest.mark.parametrize("path_type", ("path", "meta_path"))
class TestSysPathsSnapshot(object):
    other_path = {"path": "meta_path", "meta_path": "path"}

    @staticmethod
    def path(n):
        return "my-dirty-little-secret-" + str(n)

    def test_restore(self, monkeypatch, path_type):
        other_path_type = self.other_path[path_type]
        for i in range(10):
            assert self.path(i) not in getattr(sys, path_type)
        sys_path = [self.path(i) for i in range(6)]
        monkeypatch.setattr(sys, path_type, sys_path)
        original = list(sys_path)
        original_other = list(getattr(sys, other_path_type))
        snapshot = SysPathsSnapshot()
        transformation = {"source": (0, 1, 2, 3, 4, 5), "target": (6, 2, 9, 7, 5, 8)}
        assert sys_path == [self.path(x) for x in transformation["source"]]
        sys_path[1] = self.path(6)
        sys_path[3] = self.path(7)
        sys_path.append(self.path(8))
        del sys_path[4]
        sys_path[3:3] = [self.path(9)]
        del sys_path[0]
        assert sys_path == [self.path(x) for x in transformation["target"]]
        snapshot.restore()
        assert getattr(sys, path_type) is sys_path
        assert getattr(sys, path_type) == original
        assert getattr(sys, other_path_type) == original_other

    def test_preserve_container(self, monkeypatch, path_type):
        other_path_type = self.other_path[path_type]
        original_data = list(getattr(sys, path_type))
        original_other = getattr(sys, other_path_type)
        original_other_data = list(original_other)
        new = []
        snapshot = SysPathsSnapshot()
        monkeypatch.setattr(sys, path_type, new)
        snapshot.restore()
        assert getattr(sys, path_type) is new
        assert getattr(sys, path_type) == original_data
        assert getattr(sys, other_path_type) is original_other
        assert getattr(sys, other_path_type) == original_other_data


def test_testdir_subprocess(testdir):
    testfile = testdir.makepyfile("def test_one(): pass")
    assert testdir.runpytest_subprocess(testfile).ret == 0


def test_unicode_args(testdir):
    result = testdir.runpytest("-k", u"💩")
    assert result.ret == EXIT_NOTESTSCOLLECTED


def test_testdir_run_no_timeout(testdir):
    testfile = testdir.makepyfile("def test_no_timeout(): pass")
    assert testdir.runpytest_subprocess(testfile).ret == EXIT_OK


def test_testdir_run_with_timeout(testdir):
    testfile = testdir.makepyfile("def test_no_timeout(): pass")

    timeout = 120

    start = time.time()
    result = testdir.runpytest_subprocess(testfile, timeout=timeout)
    end = time.time()
    duration = end - start

    assert result.ret == EXIT_OK
    assert duration < timeout


def test_testdir_run_timeout_expires(testdir):
    testfile = testdir.makepyfile(
        """
        import time

        def test_timeout():
            time.sleep(10)"""
    )
    with pytest.raises(testdir.TimeoutExpired):
        testdir.runpytest_subprocess(testfile, timeout=1)


def test_linematcher_with_nonlist():
    """Test LineMatcher with regard to passing in a set (accidentally)."""
    lm = LineMatcher([])

    with pytest.raises(AssertionError):
        lm.fnmatch_lines(set())
    with pytest.raises(AssertionError):
        lm.fnmatch_lines({})
    lm.fnmatch_lines([])
    lm.fnmatch_lines(())

    assert lm._getlines({}) == {}
    assert lm._getlines(set()) == set()


def test_pytester_addopts(request, monkeypatch):
    monkeypatch.setenv("PYTEST_ADDOPTS", "--orig-unused")

    testdir = request.getfixturevalue("testdir")

    try:
        assert "PYTEST_ADDOPTS" not in os.environ
    finally:
        testdir.finalize()

    assert os.environ["PYTEST_ADDOPTS"] == "--orig-unused"


def test_run_stdin(testdir):
    with pytest.raises(testdir.TimeoutExpired):
        testdir.run(
            sys.executable,
            "-c",
            "import sys, time; time.sleep(1); print(sys.stdin.read())",
            stdin=subprocess.PIPE,
            timeout=0.1,
        )

    with pytest.raises(testdir.TimeoutExpired):
        result = testdir.run(
            sys.executable,
            "-c",
            "import sys, time; time.sleep(1); print(sys.stdin.read())",
            stdin=b"input\n2ndline",
            timeout=0.1,
        )

    result = testdir.run(
        sys.executable,
        "-c",
        "import sys; print(sys.stdin.read())",
        stdin=b"input\n2ndline",
    )
    assert result.stdout.lines == ["input", "2ndline"]
    assert result.stderr.str() == ""
    assert result.ret == 0


def test_popen_stdin_pipe(testdir):
    proc = testdir.popen(
        [sys.executable, "-c", "import sys; print(sys.stdin.read())"],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        stdin=subprocess.PIPE,
    )
    stdin = b"input\n2ndline"
    stdout, stderr = proc.communicate(input=stdin)
    assert stdout.decode("utf8").splitlines() == ["input", "2ndline"]
    assert stderr == b""
    assert proc.returncode == 0


def test_popen_stdin_bytes(testdir):
    proc = testdir.popen(
        [sys.executable, "-c", "import sys; print(sys.stdin.read())"],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        stdin=b"input\n2ndline",
    )
    stdout, stderr = proc.communicate()
    assert stdout.decode("utf8").splitlines() == ["input", "2ndline"]
    assert stderr == b""
    assert proc.returncode == 0


def test_popen_default_stdin_stderr_and_stdin_None(testdir):
    # stdout, stderr default to pipes,
    # stdin can be None to not close the pipe, avoiding
    # "ValueError: flush of closed file" with `communicate()`.
    p1 = testdir.makepyfile(
        """
        import sys
        print(sys.stdin.read())  # empty
        print('stdout')
        sys.stderr.write('stderr')
        """
    )
    proc = testdir.popen([sys.executable, str(p1)], stdin=None)
    stdout, stderr = proc.communicate(b"ignored")
    assert stdout.splitlines() == [b"", b"stdout"]
    assert stderr.splitlines() == [b"stderr"]
    assert proc.returncode == 0


def test_spawn_uses_tmphome(testdir):
    import os

    tmphome = str(testdir.tmpdir)

    # Does use HOME only during run.
    assert os.environ.get("HOME") != tmphome

    testdir._env_run_update["CUSTOMENV"] = "42"

    p1 = testdir.makepyfile(
        """
        import os

        def test():
            assert os.environ["HOME"] == {tmphome!r}
            assert os.environ["CUSTOMENV"] == "42"
        """.format(
            tmphome=tmphome
        )
    )
    child = testdir.spawn_pytest(str(p1))
    out = child.read()
    assert child.wait() == 0, out.decode("utf8")
