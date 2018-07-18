# -*- coding: utf-8 -*-
from __future__ import absolute_import, division, print_function
import os
import py.path
import pytest
import sys
import _pytest.pytester as pytester
from _pytest.pytester import HookRecorder
from _pytest.pytester import CwdSnapshot, SysModulesSnapshot, SysPathsSnapshot
from _pytest.config import PytestPluginManager
from _pytest.main import EXIT_OK, EXIT_TESTSFAILED


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
    pytest.raises(ValueError, "recorder.getfailures()")


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
    pm.addhooks(holder)
    rec = HookRecorder(pm)
    pm.hook.pytest_xyz(arg=123)
    call = rec.popcall("pytest_xyz")
    assert call.arg == 123
    assert call._name == "pytest_xyz"
    pytest.raises(pytest.fail.Exception, "rec.popcall('abc')")
    pm.hook.pytest_xyz_noarg()
    call = rec.popcall("pytest_xyz_noarg")
    assert call._name == "pytest_xyz_noarg"


def test_makepyfile_unicode(testdir):
    global unichr
    try:
        unichr(65)
    except NameError:
        unichr = chr
    testdir.makepyfile(unichr(0xfffd))


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
        original = dict(sys.modules)
        testdir.syspathinsert()
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


def test_inline_run_clean_sys_paths(testdir):

    def test_sys_path_change_cleanup(self, testdir):
        test_path1 = testdir.tmpdir.join("boink1").strpath
        test_path2 = testdir.tmpdir.join("boink2").strpath
        test_path3 = testdir.tmpdir.join("boink3").strpath
        sys.path.append(test_path1)
        sys.meta_path.append(test_path1)
        original_path = list(sys.path)
        original_meta_path = list(sys.meta_path)
        test_mod = testdir.makepyfile(
            """
            import sys
            sys.path.append({:test_path2})
            sys.meta_path.append({:test_path2})
            def test_foo():
                sys.path.append({:test_path3})
                sys.meta_path.append({:test_path3})""".format(
                locals()
            )
        )
        testdir.inline_run(str(test_mod))
        assert sys.path == original_path
        assert sys.meta_path == original_meta_path

    def spy_factory(self):

        class SysPathsSnapshotSpy(object):
            instances = []

            def __init__(self):
                SysPathsSnapshotSpy.instances.append(self)
                self._spy_restore_count = 0
                self.__snapshot = SysPathsSnapshot()

            def restore(self):
                self._spy_restore_count += 1
                return self.__snapshot.restore()

        return SysPathsSnapshotSpy

    def test_inline_run_taking_and_restoring_a_sys_paths_snapshot(
        self, testdir, monkeypatch
    ):
        spy_factory = self.spy_factory()
        monkeypatch.setattr(pytester, "SysPathsSnapshot", spy_factory)
        test_mod = testdir.makepyfile("def test_foo(): pass")
        testdir.inline_run(str(test_mod))
        assert len(spy_factory.instances) == 1
        spy = spy_factory.instances[0]
        assert spy._spy_restore_count == 1


def test_assert_outcomes_after_pytest_error(testdir):
    testdir.makepyfile("def test_foo(): assert True")

    result = testdir.runpytest("--unexpected-argument")
    with pytest.raises(ValueError, message="Pytest terminal report not found"):
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
        transformation = {
            "source": (0, 1, 2, 3, 4, 5), "target": (6, 2, 9, 7, 5, 8)
        }  # noqa: E201
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
