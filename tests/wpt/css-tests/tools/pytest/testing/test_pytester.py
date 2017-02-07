import pytest
import os
from _pytest.pytester import HookRecorder
from _pytest.config import PytestPluginManager
from _pytest.main import EXIT_OK, EXIT_TESTSFAILED


def test_make_hook_recorder(testdir):
    item = testdir.getitem("def test_func(): pass")
    recorder = testdir.make_hook_recorder(item.config.pluginmanager)
    assert not recorder.getfailures()

    pytest.xfail("internal reportrecorder tests need refactoring")
    class rep:
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

    class rep:
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
    testdir.makepyfile("""
        pytest_plugins = "pytester"
        def test_hello(testdir):
            assert 1
    """)
    result = testdir.runpytest()
    result.assert_outcomes(passed=1)


def make_holder():
    class apiclass:
        def pytest_xyz(self, arg):
            "x"
        def pytest_xyz_noarg(self):
            "x"

    apimod = type(os)('api')
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

def test_inline_run_clean_modules(testdir):
    test_mod = testdir.makepyfile("def test_foo(): assert True")
    result = testdir.inline_run(str(test_mod))
    assert result.ret == EXIT_OK
    # rewrite module, now test should fail if module was re-imported
    test_mod.write("def test_foo(): assert False")
    result2 = testdir.inline_run(str(test_mod))
    assert result2.ret == EXIT_TESTSFAILED
