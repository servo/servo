from __future__ import absolute_import, division, print_function
import sys
import platform
import os

import _pytest._code
from _pytest.debugging import SUPPORTS_BREAKPOINT_BUILTIN
import pytest


_ENVIRON_PYTHONBREAKPOINT = os.environ.get("PYTHONBREAKPOINT", "")


def runpdb_and_get_report(testdir, source):
    p = testdir.makepyfile(source)
    result = testdir.runpytest_inprocess("--pdb", p)
    reports = result.reprec.getreports("pytest_runtest_logreport")
    assert len(reports) == 3, reports  # setup/call/teardown
    return reports[1]


@pytest.fixture
def custom_pdb_calls():
    called = []

    # install dummy debugger class and track which methods were called on it
    class _CustomPdb(object):

        def __init__(self, *args, **kwargs):
            called.append("init")

        def reset(self):
            called.append("reset")

        def interaction(self, *args):
            called.append("interaction")

    _pytest._CustomPdb = _CustomPdb
    return called


@pytest.fixture
def custom_debugger_hook():
    called = []

    # install dummy debugger class and track which methods were called on it
    class _CustomDebugger(object):

        def __init__(self, *args, **kwargs):
            called.append("init")

        def reset(self):
            called.append("reset")

        def interaction(self, *args):
            called.append("interaction")

        def set_trace(self, frame):
            print("**CustomDebugger**")
            called.append("set_trace")

    _pytest._CustomDebugger = _CustomDebugger
    yield called
    del _pytest._CustomDebugger


class TestPDB(object):

    @pytest.fixture
    def pdblist(self, request):
        monkeypatch = request.getfixturevalue("monkeypatch")
        pdblist = []

        def mypdb(*args):
            pdblist.append(args)

        plugin = request.config.pluginmanager.getplugin("debugging")
        monkeypatch.setattr(plugin, "post_mortem", mypdb)
        return pdblist

    def test_pdb_on_fail(self, testdir, pdblist):
        rep = runpdb_and_get_report(
            testdir,
            """
            def test_func():
                assert 0
        """,
        )
        assert rep.failed
        assert len(pdblist) == 1
        tb = _pytest._code.Traceback(pdblist[0][0])
        assert tb[-1].name == "test_func"

    def test_pdb_on_xfail(self, testdir, pdblist):
        rep = runpdb_and_get_report(
            testdir,
            """
            import pytest
            @pytest.mark.xfail
            def test_func():
                assert 0
        """,
        )
        assert "xfail" in rep.keywords
        assert not pdblist

    def test_pdb_on_skip(self, testdir, pdblist):
        rep = runpdb_and_get_report(
            testdir,
            """
            import pytest
            def test_func():
                pytest.skip("hello")
        """,
        )
        assert rep.skipped
        assert len(pdblist) == 0

    def test_pdb_on_BdbQuit(self, testdir, pdblist):
        rep = runpdb_and_get_report(
            testdir,
            """
            import bdb
            def test_func():
                raise bdb.BdbQuit
        """,
        )
        assert rep.failed
        assert len(pdblist) == 0

    def test_pdb_on_KeyboardInterrupt(self, testdir, pdblist):
        rep = runpdb_and_get_report(
            testdir,
            """
            def test_func():
                raise KeyboardInterrupt
        """,
        )
        assert rep.failed
        assert len(pdblist) == 1

    def test_pdb_interaction(self, testdir):
        p1 = testdir.makepyfile(
            """
            def test_1():
                i = 0
                assert i == 1
        """
        )
        child = testdir.spawn_pytest("--pdb %s" % p1)
        child.expect(".*def test_1")
        child.expect(".*i = 0")
        child.expect("(Pdb)")
        child.sendeof()
        rest = child.read().decode("utf8")
        assert "1 failed" in rest
        assert "def test_1" not in rest
        self.flush(child)

    @staticmethod
    def flush(child):
        if platform.system() == "Darwin":
            return
        if child.isalive():
            child.wait()

    def test_pdb_unittest_postmortem(self, testdir):
        p1 = testdir.makepyfile(
            """
            import unittest
            class Blub(unittest.TestCase):
                def tearDown(self):
                    self.filename = None
                def test_false(self):
                    self.filename = 'debug' + '.me'
                    assert 0
        """
        )
        child = testdir.spawn_pytest("--pdb %s" % p1)
        child.expect("(Pdb)")
        child.sendline("p self.filename")
        child.sendeof()
        rest = child.read().decode("utf8")
        assert "debug.me" in rest
        self.flush(child)

    def test_pdb_unittest_skip(self, testdir):
        """Test for issue #2137"""
        p1 = testdir.makepyfile(
            """
            import unittest
            @unittest.skipIf(True, 'Skipping also with pdb active')
            class MyTestCase(unittest.TestCase):
                def test_one(self):
                    assert 0
        """
        )
        child = testdir.spawn_pytest("-rs --pdb %s" % p1)
        child.expect("Skipping also with pdb active")
        child.expect("1 skipped in")
        child.sendeof()
        self.flush(child)

    def test_pdb_print_captured_stdout(self, testdir):
        p1 = testdir.makepyfile(
            """
            def test_1():
                print("get\\x20rekt")
                assert False
        """
        )
        child = testdir.spawn_pytest("--pdb %s" % p1)
        child.expect("captured stdout")
        child.expect("get rekt")
        child.expect("(Pdb)")
        child.sendeof()
        rest = child.read().decode("utf8")
        assert "1 failed" in rest
        assert "get rekt" not in rest
        self.flush(child)

    def test_pdb_print_captured_stderr(self, testdir):
        p1 = testdir.makepyfile(
            """
            def test_1():
                import sys
                sys.stderr.write("get\\x20rekt")
                assert False
        """
        )
        child = testdir.spawn_pytest("--pdb %s" % p1)
        child.expect("captured stderr")
        child.expect("get rekt")
        child.expect("(Pdb)")
        child.sendeof()
        rest = child.read().decode("utf8")
        assert "1 failed" in rest
        assert "get rekt" not in rest
        self.flush(child)

    def test_pdb_dont_print_empty_captured_stdout_and_stderr(self, testdir):
        p1 = testdir.makepyfile(
            """
            def test_1():
                assert False
        """
        )
        child = testdir.spawn_pytest("--pdb %s" % p1)
        child.expect("(Pdb)")
        output = child.before.decode("utf8")
        child.sendeof()
        assert "captured stdout" not in output
        assert "captured stderr" not in output
        self.flush(child)

    @pytest.mark.parametrize("showcapture", ["all", "no", "log"])
    def test_pdb_print_captured_logs(self, testdir, showcapture):
        p1 = testdir.makepyfile(
            """
            def test_1():
                import logging
                logging.warn("get " + "rekt")
                assert False
        """
        )
        child = testdir.spawn_pytest("--show-capture=%s --pdb %s" % (showcapture, p1))
        if showcapture in ("all", "log"):
            child.expect("captured log")
            child.expect("get rekt")
        child.expect("(Pdb)")
        child.sendeof()
        rest = child.read().decode("utf8")
        assert "1 failed" in rest
        self.flush(child)

    def test_pdb_print_captured_logs_nologging(self, testdir):
        p1 = testdir.makepyfile(
            """
            def test_1():
                import logging
                logging.warn("get " + "rekt")
                assert False
        """
        )
        child = testdir.spawn_pytest(
            "--show-capture=all --pdb " "-p no:logging %s" % p1
        )
        child.expect("get rekt")
        output = child.before.decode("utf8")
        assert "captured log" not in output
        child.expect("(Pdb)")
        child.sendeof()
        rest = child.read().decode("utf8")
        assert "1 failed" in rest
        self.flush(child)

    def test_pdb_interaction_exception(self, testdir):
        p1 = testdir.makepyfile(
            """
            import pytest
            def globalfunc():
                pass
            def test_1():
                pytest.raises(ValueError, globalfunc)
        """
        )
        child = testdir.spawn_pytest("--pdb %s" % p1)
        child.expect(".*def test_1")
        child.expect(".*pytest.raises.*globalfunc")
        child.expect("(Pdb)")
        child.sendline("globalfunc")
        child.expect(".*function")
        child.sendeof()
        child.expect("1 failed")
        self.flush(child)

    def test_pdb_interaction_on_collection_issue181(self, testdir):
        p1 = testdir.makepyfile(
            """
            import pytest
            xxx
        """
        )
        child = testdir.spawn_pytest("--pdb %s" % p1)
        # child.expect(".*import pytest.*")
        child.expect("(Pdb)")
        child.sendeof()
        child.expect("1 error")
        self.flush(child)

    def test_pdb_interaction_on_internal_error(self, testdir):
        testdir.makeconftest(
            """
            def pytest_runtest_protocol():
                0/0
        """
        )
        p1 = testdir.makepyfile("def test_func(): pass")
        child = testdir.spawn_pytest("--pdb %s" % p1)
        # child.expect(".*import pytest.*")
        child.expect("(Pdb)")
        child.sendeof()
        self.flush(child)

    def test_pdb_interaction_capturing_simple(self, testdir):
        p1 = testdir.makepyfile(
            """
            import pytest
            def test_1():
                i = 0
                print ("hello17")
                pytest.set_trace()
                x = 3
        """
        )
        child = testdir.spawn_pytest(str(p1))
        child.expect("test_1")
        child.expect("x = 3")
        child.expect("(Pdb)")
        child.sendeof()
        rest = child.read().decode("utf-8")
        assert "1 failed" in rest
        assert "def test_1" in rest
        assert "hello17" in rest  # out is captured
        self.flush(child)

    def test_pdb_set_trace_interception(self, testdir):
        p1 = testdir.makepyfile(
            """
            import pdb
            def test_1():
                pdb.set_trace()
        """
        )
        child = testdir.spawn_pytest(str(p1))
        child.expect("test_1")
        child.expect("(Pdb)")
        child.sendeof()
        rest = child.read().decode("utf8")
        assert "1 failed" in rest
        assert "reading from stdin while output" not in rest
        self.flush(child)

    def test_pdb_and_capsys(self, testdir):
        p1 = testdir.makepyfile(
            """
            import pytest
            def test_1(capsys):
                print ("hello1")
                pytest.set_trace()
        """
        )
        child = testdir.spawn_pytest(str(p1))
        child.expect("test_1")
        child.send("capsys.readouterr()\n")
        child.expect("hello1")
        child.sendeof()
        child.read()
        self.flush(child)

    def test_set_trace_capturing_afterwards(self, testdir):
        p1 = testdir.makepyfile(
            """
            import pdb
            def test_1():
                pdb.set_trace()
            def test_2():
                print ("hello")
                assert 0
        """
        )
        child = testdir.spawn_pytest(str(p1))
        child.expect("test_1")
        child.send("c\n")
        child.expect("test_2")
        child.expect("Captured")
        child.expect("hello")
        child.sendeof()
        child.read()
        self.flush(child)

    def test_pdb_interaction_doctest(self, testdir):
        p1 = testdir.makepyfile(
            """
            import pytest
            def function_1():
                '''
                >>> i = 0
                >>> assert i == 1
                '''
        """
        )
        child = testdir.spawn_pytest("--doctest-modules --pdb %s" % p1)
        child.expect("(Pdb)")
        child.sendline("i")
        child.expect("0")
        child.expect("(Pdb)")
        child.sendeof()
        rest = child.read().decode("utf8")
        assert "1 failed" in rest
        self.flush(child)

    def test_pdb_interaction_capturing_twice(self, testdir):
        p1 = testdir.makepyfile(
            """
            import pytest
            def test_1():
                i = 0
                print ("hello17")
                pytest.set_trace()
                x = 3
                print ("hello18")
                pytest.set_trace()
                x = 4
        """
        )
        child = testdir.spawn_pytest(str(p1))
        child.expect("test_1")
        child.expect("x = 3")
        child.expect("(Pdb)")
        child.sendline("c")
        child.expect("x = 4")
        child.sendeof()
        rest = child.read().decode("utf8")
        assert "1 failed" in rest
        assert "def test_1" in rest
        assert "hello17" in rest  # out is captured
        assert "hello18" in rest  # out is captured
        self.flush(child)

    def test_pdb_used_outside_test(self, testdir):
        p1 = testdir.makepyfile(
            """
            import pytest
            pytest.set_trace()
            x = 5
        """
        )
        child = testdir.spawn("%s %s" % (sys.executable, p1))
        child.expect("x = 5")
        child.sendeof()
        self.flush(child)

    def test_pdb_used_in_generate_tests(self, testdir):
        p1 = testdir.makepyfile(
            """
            import pytest
            def pytest_generate_tests(metafunc):
                pytest.set_trace()
                x = 5
            def test_foo(a):
                pass
        """
        )
        child = testdir.spawn_pytest(str(p1))
        child.expect("x = 5")
        child.sendeof()
        self.flush(child)

    def test_pdb_collection_failure_is_shown(self, testdir):
        p1 = testdir.makepyfile("xxx")
        result = testdir.runpytest_subprocess("--pdb", p1)
        result.stdout.fnmatch_lines(["*NameError*xxx*", "*1 error*"])

    def test_enter_pdb_hook_is_called(self, testdir):
        testdir.makeconftest(
            """
            def pytest_enter_pdb(config):
                assert config.testing_verification == 'configured'
                print 'enter_pdb_hook'

            def pytest_configure(config):
                config.testing_verification = 'configured'
        """
        )
        p1 = testdir.makepyfile(
            """
            import pytest

            def test_foo():
                pytest.set_trace()
        """
        )
        child = testdir.spawn_pytest(str(p1))
        child.expect("enter_pdb_hook")
        child.send("c\n")
        child.sendeof()
        self.flush(child)

    def test_pdb_custom_cls(self, testdir, custom_pdb_calls):
        p1 = testdir.makepyfile("""xxx """)
        result = testdir.runpytest_inprocess("--pdb", "--pdbcls=_pytest:_CustomPdb", p1)
        result.stdout.fnmatch_lines(["*NameError*xxx*", "*1 error*"])
        assert custom_pdb_calls == ["init", "reset", "interaction"]

    def test_pdb_custom_cls_without_pdb(self, testdir, custom_pdb_calls):
        p1 = testdir.makepyfile("""xxx """)
        result = testdir.runpytest_inprocess("--pdbcls=_pytest:_CustomPdb", p1)
        result.stdout.fnmatch_lines(["*NameError*xxx*", "*1 error*"])
        assert custom_pdb_calls == []

    def test_pdb_custom_cls_with_settrace(self, testdir, monkeypatch):
        testdir.makepyfile(
            custom_pdb="""
            class CustomPdb(object):
                def set_trace(*args, **kwargs):
                    print 'custom set_trace>'
         """
        )
        p1 = testdir.makepyfile(
            """
            import pytest

            def test_foo():
                pytest.set_trace()
        """
        )
        monkeypatch.setenv("PYTHONPATH", str(testdir.tmpdir))
        child = testdir.spawn_pytest("--pdbcls=custom_pdb:CustomPdb %s" % str(p1))

        child.expect("custom set_trace>")
        self.flush(child)


class TestDebuggingBreakpoints(object):

    def test_supports_breakpoint_module_global(self):
        """
        Test that supports breakpoint global marks on Python 3.7+ and not on
        CPython 3.5, 2.7
        """
        if sys.version_info.major == 3 and sys.version_info.minor >= 7:
            assert SUPPORTS_BREAKPOINT_BUILTIN is True
        if sys.version_info.major == 3 and sys.version_info.minor == 5:
            assert SUPPORTS_BREAKPOINT_BUILTIN is False
        if sys.version_info.major == 2 and sys.version_info.minor == 7:
            assert SUPPORTS_BREAKPOINT_BUILTIN is False

    @pytest.mark.skipif(
        not SUPPORTS_BREAKPOINT_BUILTIN, reason="Requires breakpoint() builtin"
    )
    @pytest.mark.parametrize("arg", ["--pdb", ""])
    def test_sys_breakpointhook_configure_and_unconfigure(self, testdir, arg):
        """
        Test that sys.breakpointhook is set to the custom Pdb class once configured, test that
        hook is reset to system value once pytest has been unconfigured
        """
        testdir.makeconftest(
            """
            import sys
            from pytest import hookimpl
            from _pytest.debugging import pytestPDB

            def pytest_configure(config):
                config._cleanup.append(check_restored)

            def check_restored():
                assert sys.breakpointhook == sys.__breakpointhook__

            def test_check():
                assert sys.breakpointhook == pytestPDB.set_trace
        """
        )
        testdir.makepyfile(
            """
            def test_nothing(): pass
        """
        )
        args = (arg,) if arg else ()
        result = testdir.runpytest_subprocess(*args)
        result.stdout.fnmatch_lines(["*1 passed in *"])

    @pytest.mark.skipif(
        not SUPPORTS_BREAKPOINT_BUILTIN, reason="Requires breakpoint() builtin"
    )
    def test_pdb_custom_cls(self, testdir, custom_debugger_hook):
        p1 = testdir.makepyfile(
            """
            def test_nothing():
                breakpoint()
        """
        )
        result = testdir.runpytest_inprocess(
            "--pdb", "--pdbcls=_pytest:_CustomDebugger", p1
        )
        result.stdout.fnmatch_lines(["*CustomDebugger*", "*1 passed*"])
        assert custom_debugger_hook == ["init", "set_trace"]

    @pytest.mark.parametrize("arg", ["--pdb", ""])
    @pytest.mark.skipif(
        not SUPPORTS_BREAKPOINT_BUILTIN, reason="Requires breakpoint() builtin"
    )
    def test_environ_custom_class(self, testdir, custom_debugger_hook, arg):
        testdir.makeconftest(
            """
            import os
            import sys

            os.environ['PYTHONBREAKPOINT'] = '_pytest._CustomDebugger.set_trace'

            def pytest_configure(config):
                config._cleanup.append(check_restored)

            def check_restored():
                assert sys.breakpointhook == sys.__breakpointhook__

            def test_check():
                import _pytest
                assert sys.breakpointhook is _pytest._CustomDebugger.set_trace
        """
        )
        testdir.makepyfile(
            """
            def test_nothing(): pass
        """
        )
        args = (arg,) if arg else ()
        result = testdir.runpytest_subprocess(*args)
        result.stdout.fnmatch_lines(["*1 passed in *"])

    @pytest.mark.skipif(
        not SUPPORTS_BREAKPOINT_BUILTIN, reason="Requires breakpoint() builtin"
    )
    @pytest.mark.skipif(
        not _ENVIRON_PYTHONBREAKPOINT == "",
        reason="Requires breakpoint() default value",
    )
    def test_sys_breakpoint_interception(self, testdir):
        p1 = testdir.makepyfile(
            """
            def test_1():
                breakpoint()
        """
        )
        child = testdir.spawn_pytest(str(p1))
        child.expect("test_1")
        child.expect("(Pdb)")
        child.sendeof()
        rest = child.read().decode("utf8")
        assert "1 failed" in rest
        assert "reading from stdin while output" not in rest
        TestPDB.flush(child)

    @pytest.mark.skipif(
        not SUPPORTS_BREAKPOINT_BUILTIN, reason="Requires breakpoint() builtin"
    )
    def test_pdb_not_altered(self, testdir):
        p1 = testdir.makepyfile(
            """
            import pdb
            def test_1():
                pdb.set_trace()
        """
        )
        child = testdir.spawn_pytest(str(p1))
        child.expect("test_1")
        child.expect("(Pdb)")
        child.sendeof()
        rest = child.read().decode("utf8")
        assert "1 failed" in rest
        assert "reading from stdin while output" not in rest
        TestPDB.flush(child)
