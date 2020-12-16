# -*- coding: utf-8 -*-
from __future__ import absolute_import
from __future__ import division
from __future__ import print_function

import contextlib
import io
import os
import pickle
import subprocess
import sys
import textwrap
from io import UnsupportedOperation

import py
from six import text_type

import pytest
from _pytest import capture
from _pytest.capture import CaptureManager
from _pytest.compat import _PY3
from _pytest.main import EXIT_NOTESTSCOLLECTED

# note: py.io capture tests where copied from
# pylib 1.4.20.dev2 (rev 13d9af95547e)


needsosdup = pytest.mark.skipif(
    not hasattr(os, "dup"), reason="test needs os.dup, not available on this platform"
)


def StdCaptureFD(out=True, err=True, in_=True):
    return capture.MultiCapture(out, err, in_, Capture=capture.FDCapture)


def StdCapture(out=True, err=True, in_=True):
    return capture.MultiCapture(out, err, in_, Capture=capture.SysCapture)


class TestCaptureManager(object):
    def test_getmethod_default_no_fd(self, monkeypatch):
        from _pytest.capture import pytest_addoption
        from _pytest.config.argparsing import Parser

        parser = Parser()
        pytest_addoption(parser)
        default = parser._groups[0].options[0].default
        assert default == "fd" if hasattr(os, "dup") else "sys"
        parser = Parser()
        monkeypatch.delattr(os, "dup", raising=False)
        pytest_addoption(parser)
        assert parser._groups[0].options[0].default == "sys"

    @pytest.mark.parametrize(
        "method", ["no", "sys", pytest.param("fd", marks=needsosdup)]
    )
    def test_capturing_basic_api(self, method):
        capouter = StdCaptureFD()
        old = sys.stdout, sys.stderr, sys.stdin
        try:
            capman = CaptureManager(method)
            capman.start_global_capturing()
            capman.suspend_global_capture()
            outerr = capman.read_global_capture()
            assert outerr == ("", "")
            capman.suspend_global_capture()
            outerr = capman.read_global_capture()
            assert outerr == ("", "")
            print("hello")
            capman.suspend_global_capture()
            out, err = capman.read_global_capture()
            if method == "no":
                assert old == (sys.stdout, sys.stderr, sys.stdin)
            else:
                assert not out
            capman.resume_global_capture()
            print("hello")
            capman.suspend_global_capture()
            out, err = capman.read_global_capture()
            if method != "no":
                assert out == "hello\n"
            capman.stop_global_capturing()
        finally:
            capouter.stop_capturing()

    @needsosdup
    def test_init_capturing(self):
        capouter = StdCaptureFD()
        try:
            capman = CaptureManager("fd")
            capman.start_global_capturing()
            pytest.raises(AssertionError, capman.start_global_capturing)
            capman.stop_global_capturing()
        finally:
            capouter.stop_capturing()


@pytest.mark.parametrize("method", ["fd", "sys"])
def test_capturing_unicode(testdir, method):
    if hasattr(sys, "pypy_version_info") and sys.pypy_version_info < (2, 2):
        pytest.xfail("does not work on pypy < 2.2")
    if sys.version_info >= (3, 0):
        obj = "'b\u00f6y'"
    else:
        obj = "u'\u00f6y'"
    testdir.makepyfile(
        """
        # -*- coding: utf-8 -*-
        # taken from issue 227 from nosetests
        def test_unicode():
            import sys
            print(sys.stdout)
            print(%s)
    """
        % obj
    )
    result = testdir.runpytest("--capture=%s" % method)
    result.stdout.fnmatch_lines(["*1 passed*"])


@pytest.mark.parametrize("method", ["fd", "sys"])
def test_capturing_bytes_in_utf8_encoding(testdir, method):
    testdir.makepyfile(
        """
        def test_unicode():
            print('b\\u00f6y')
    """
    )
    result = testdir.runpytest("--capture=%s" % method)
    result.stdout.fnmatch_lines(["*1 passed*"])


def test_collect_capturing(testdir):
    p = testdir.makepyfile(
        """
        import sys

        print("collect %s failure" % 13)
        sys.stderr.write("collect %s_stderr failure" % 13)
        import xyz42123
    """
    )
    result = testdir.runpytest(p)
    result.stdout.fnmatch_lines(
        [
            "*Captured stdout*",
            "collect 13 failure",
            "*Captured stderr*",
            "collect 13_stderr failure",
        ]
    )


class TestPerTestCapturing(object):
    def test_capture_and_fixtures(self, testdir):
        p = testdir.makepyfile(
            """
            def setup_module(mod):
                print("setup module")
            def setup_function(function):
                print("setup " + function.__name__)
            def test_func1():
                print("in func1")
                assert 0
            def test_func2():
                print("in func2")
                assert 0
        """
        )
        result = testdir.runpytest(p)
        result.stdout.fnmatch_lines(
            [
                "setup module*",
                "setup test_func1*",
                "in func1*",
                "setup test_func2*",
                "in func2*",
            ]
        )

    @pytest.mark.xfail(reason="unimplemented feature")
    def test_capture_scope_cache(self, testdir):
        p = testdir.makepyfile(
            """
            import sys
            def setup_module(func):
                print("module-setup")
            def setup_function(func):
                print("function-setup")
            def test_func():
                print("in function")
                assert 0
            def teardown_function(func):
                print("in teardown")
        """
        )
        result = testdir.runpytest(p)
        result.stdout.fnmatch_lines(
            [
                "*test_func():*",
                "*Captured stdout during setup*",
                "module-setup*",
                "function-setup*",
                "*Captured stdout*",
                "in teardown*",
            ]
        )

    def test_no_carry_over(self, testdir):
        p = testdir.makepyfile(
            """
            def test_func1():
                print("in func1")
            def test_func2():
                print("in func2")
                assert 0
        """
        )
        result = testdir.runpytest(p)
        s = result.stdout.str()
        assert "in func1" not in s
        assert "in func2" in s

    def test_teardown_capturing(self, testdir):
        p = testdir.makepyfile(
            """
            def setup_function(function):
                print("setup func1")
            def teardown_function(function):
                print("teardown func1")
                assert 0
            def test_func1():
                print("in func1")
                pass
        """
        )
        result = testdir.runpytest(p)
        result.stdout.fnmatch_lines(
            [
                "*teardown_function*",
                "*Captured stdout*",
                "setup func1*",
                "in func1*",
                "teardown func1*",
                # "*1 fixture failure*"
            ]
        )

    def test_teardown_capturing_final(self, testdir):
        p = testdir.makepyfile(
            """
            def teardown_module(mod):
                print("teardown module")
                assert 0
            def test_func():
                pass
        """
        )
        result = testdir.runpytest(p)
        result.stdout.fnmatch_lines(
            [
                "*def teardown_module(mod):*",
                "*Captured stdout*",
                "*teardown module*",
                "*1 error*",
            ]
        )

    def test_capturing_outerr(self, testdir):
        p1 = testdir.makepyfile(
            """\
            import sys
            def test_capturing():
                print(42)
                sys.stderr.write(str(23))
            def test_capturing_error():
                print(1)
                sys.stderr.write(str(2))
                raise ValueError
            """
        )
        result = testdir.runpytest(p1)
        result.stdout.fnmatch_lines(
            [
                "*test_capturing_outerr.py .F*",
                "====* FAILURES *====",
                "____*____",
                "*test_capturing_outerr.py:8: ValueError",
                "*--- Captured stdout *call*",
                "1",
                "*--- Captured stderr *call*",
                "2",
            ]
        )


class TestLoggingInteraction(object):
    def test_logging_stream_ownership(self, testdir):
        p = testdir.makepyfile(
            """\
            def test_logging():
                import logging
                import pytest
                stream = capture.CaptureIO()
                logging.basicConfig(stream=stream)
                stream.close() # to free memory/release resources
            """
        )
        result = testdir.runpytest_subprocess(p)
        assert result.stderr.str().find("atexit") == -1

    def test_logging_and_immediate_setupteardown(self, testdir):
        p = testdir.makepyfile(
            """\
            import logging
            def setup_function(function):
                logging.warning("hello1")

            def test_logging():
                logging.warning("hello2")
                assert 0

            def teardown_function(function):
                logging.warning("hello3")
                assert 0
            """
        )
        for optargs in (("--capture=sys",), ("--capture=fd",)):
            print(optargs)
            result = testdir.runpytest_subprocess(p, *optargs)
            s = result.stdout.str()
            result.stdout.fnmatch_lines(
                ["*WARN*hello3", "*WARN*hello1", "*WARN*hello2"]  # errors show first!
            )
            # verify proper termination
            assert "closed" not in s

    def test_logging_and_crossscope_fixtures(self, testdir):
        p = testdir.makepyfile(
            """\
            import logging
            def setup_module(function):
                logging.warning("hello1")

            def test_logging():
                logging.warning("hello2")
                assert 0

            def teardown_module(function):
                logging.warning("hello3")
                assert 0
            """
        )
        for optargs in (("--capture=sys",), ("--capture=fd",)):
            print(optargs)
            result = testdir.runpytest_subprocess(p, *optargs)
            s = result.stdout.str()
            result.stdout.fnmatch_lines(
                ["*WARN*hello3", "*WARN*hello1", "*WARN*hello2"]  # errors come first
            )
            # verify proper termination
            assert "closed" not in s

    def test_conftestlogging_is_shown(self, testdir):
        testdir.makeconftest(
            """\
                import logging
                logging.basicConfig()
                logging.warning("hello435")
            """
        )
        # make sure that logging is still captured in tests
        result = testdir.runpytest_subprocess("-s", "-p", "no:capturelog")
        assert result.ret == EXIT_NOTESTSCOLLECTED
        result.stderr.fnmatch_lines(["WARNING*hello435*"])
        assert "operation on closed file" not in result.stderr.str()

    def test_conftestlogging_and_test_logging(self, testdir):
        testdir.makeconftest(
            """\
                import logging
                logging.basicConfig()
            """
        )
        # make sure that logging is still captured in tests
        p = testdir.makepyfile(
            """\
            def test_hello():
                import logging
                logging.warning("hello433")
                assert 0
            """
        )
        result = testdir.runpytest_subprocess(p, "-p", "no:capturelog")
        assert result.ret != 0
        result.stdout.fnmatch_lines(["WARNING*hello433*"])
        assert "something" not in result.stderr.str()
        assert "operation on closed file" not in result.stderr.str()

    def test_logging_after_cap_stopped(self, testdir):
        testdir.makeconftest(
            """\
                import pytest
                import logging

                log = logging.getLogger(__name__)

                @pytest.fixture
                def log_on_teardown():
                    yield
                    log.warning('Logging on teardown')
            """
        )
        # make sure that logging is still captured in tests
        p = testdir.makepyfile(
            """\
            def test_hello(log_on_teardown):
                import logging
                logging.warning("hello433")
                assert 1
                raise KeyboardInterrupt()
            """
        )
        result = testdir.runpytest_subprocess(p, "--log-cli-level", "info")
        assert result.ret != 0
        result.stdout.fnmatch_lines(
            ["*WARNING*hello433*", "*WARNING*Logging on teardown*"]
        )
        assert (
            "AttributeError: 'NoneType' object has no attribute 'resume_capturing'"
            not in result.stderr.str()
        )


class TestCaptureFixture(object):
    @pytest.mark.parametrize("opt", [[], ["-s"]])
    def test_std_functional(self, testdir, opt):
        reprec = testdir.inline_runsource(
            """\
            def test_hello(capsys):
                print(42)
                out, err = capsys.readouterr()
                assert out.startswith("42")
            """,
            *opt
        )
        reprec.assertoutcome(passed=1)

    def test_capsyscapfd(self, testdir):
        p = testdir.makepyfile(
            """\
            def test_one(capsys, capfd):
                pass
            def test_two(capfd, capsys):
                pass
            """
        )
        result = testdir.runpytest(p)
        result.stdout.fnmatch_lines(
            [
                "*ERROR*setup*test_one*",
                "E*capfd*capsys*same*time*",
                "*ERROR*setup*test_two*",
                "E*capsys*capfd*same*time*",
                "*2 error*",
            ]
        )

    def test_capturing_getfixturevalue(self, testdir):
        """Test that asking for "capfd" and "capsys" using request.getfixturevalue
        in the same test is an error.
        """
        testdir.makepyfile(
            """\
            def test_one(capsys, request):
                request.getfixturevalue("capfd")
            def test_two(capfd, request):
                request.getfixturevalue("capsys")
            """
        )
        result = testdir.runpytest()
        result.stdout.fnmatch_lines(
            [
                "*test_one*",
                "*capsys*capfd*same*time*",
                "*test_two*",
                "*capfd*capsys*same*time*",
                "*2 failed in*",
            ]
        )

    def test_capsyscapfdbinary(self, testdir):
        p = testdir.makepyfile(
            """\
            def test_one(capsys, capfdbinary):
                pass
            """
        )
        result = testdir.runpytest(p)
        result.stdout.fnmatch_lines(
            ["*ERROR*setup*test_one*", "E*capfdbinary*capsys*same*time*", "*1 error*"]
        )

    @pytest.mark.parametrize("method", ["sys", "fd"])
    def test_capture_is_represented_on_failure_issue128(self, testdir, method):
        p = testdir.makepyfile(
            """\
            def test_hello(cap{}):
                print("xxx42xxx")
                assert 0
            """.format(
                method
            )
        )
        result = testdir.runpytest(p)
        result.stdout.fnmatch_lines(["xxx42xxx"])

    @needsosdup
    def test_stdfd_functional(self, testdir):
        reprec = testdir.inline_runsource(
            """\
            def test_hello(capfd):
                import os
                os.write(1, "42".encode('ascii'))
                out, err = capfd.readouterr()
                assert out.startswith("42")
                capfd.close()
            """
        )
        reprec.assertoutcome(passed=1)

    @needsosdup
    def test_capfdbinary(self, testdir):
        reprec = testdir.inline_runsource(
            """\
            def test_hello(capfdbinary):
                import os
                # some likely un-decodable bytes
                os.write(1, b'\\xfe\\x98\\x20')
                out, err = capfdbinary.readouterr()
                assert out == b'\\xfe\\x98\\x20'
                assert err == b''
            """
        )
        reprec.assertoutcome(passed=1)

    @pytest.mark.skipif(
        sys.version_info < (3,), reason="only have capsysbinary in python 3"
    )
    def test_capsysbinary(self, testdir):
        reprec = testdir.inline_runsource(
            """\
            def test_hello(capsysbinary):
                import sys
                # some likely un-decodable bytes
                sys.stdout.buffer.write(b'\\xfe\\x98\\x20')
                out, err = capsysbinary.readouterr()
                assert out == b'\\xfe\\x98\\x20'
                assert err == b''
            """
        )
        reprec.assertoutcome(passed=1)

    @pytest.mark.skipif(
        sys.version_info >= (3,), reason="only have capsysbinary in python 3"
    )
    def test_capsysbinary_forbidden_in_python2(self, testdir):
        testdir.makepyfile(
            """\
            def test_hello(capsysbinary):
                pass
            """
        )
        result = testdir.runpytest()
        result.stdout.fnmatch_lines(
            [
                "*test_hello*",
                "*capsysbinary is only supported on Python 3*",
                "*1 error in*",
            ]
        )

    def test_partial_setup_failure(self, testdir):
        p = testdir.makepyfile(
            """\
            def test_hello(capsys, missingarg):
                pass
            """
        )
        result = testdir.runpytest(p)
        result.stdout.fnmatch_lines(["*test_partial_setup_failure*", "*1 error*"])

    @needsosdup
    def test_keyboardinterrupt_disables_capturing(self, testdir):
        p = testdir.makepyfile(
            """\
            def test_hello(capfd):
                import os
                os.write(1, str(42).encode('ascii'))
                raise KeyboardInterrupt()
            """
        )
        result = testdir.runpytest_subprocess(p)
        result.stdout.fnmatch_lines(["*KeyboardInterrupt*"])
        assert result.ret == 2

    def test_capture_and_logging(self, testdir):
        """#14"""
        p = testdir.makepyfile(
            """\
            import logging
            def test_log(capsys):
                logging.error('x')
            """
        )
        result = testdir.runpytest_subprocess(p)
        assert "closed" not in result.stderr.str()

    @pytest.mark.parametrize("fixture", ["capsys", "capfd"])
    @pytest.mark.parametrize("no_capture", [True, False])
    def test_disabled_capture_fixture(self, testdir, fixture, no_capture):
        testdir.makepyfile(
            """\
            def test_disabled({fixture}):
                print('captured before')
                with {fixture}.disabled():
                    print('while capture is disabled')
                print('captured after')
                assert {fixture}.readouterr() == ('captured before\\ncaptured after\\n', '')

            def test_normal():
                print('test_normal executed')
        """.format(
                fixture=fixture
            )
        )
        args = ("-s",) if no_capture else ()
        result = testdir.runpytest_subprocess(*args)
        result.stdout.fnmatch_lines(
            """
            *while capture is disabled*
        """
        )
        assert "captured before" not in result.stdout.str()
        assert "captured after" not in result.stdout.str()
        if no_capture:
            assert "test_normal executed" in result.stdout.str()
        else:
            assert "test_normal executed" not in result.stdout.str()

    @pytest.mark.parametrize("fixture", ["capsys", "capfd"])
    def test_fixture_use_by_other_fixtures(self, testdir, fixture):
        """
        Ensure that capsys and capfd can be used by other fixtures during setup and teardown.
        """
        testdir.makepyfile(
            """\
            from __future__ import print_function
            import sys
            import pytest

            @pytest.fixture
            def captured_print({fixture}):
                print('stdout contents begin')
                print('stderr contents begin', file=sys.stderr)
                out, err = {fixture}.readouterr()

                yield out, err

                print('stdout contents end')
                print('stderr contents end', file=sys.stderr)
                out, err = {fixture}.readouterr()
                assert out == 'stdout contents end\\n'
                assert err == 'stderr contents end\\n'

            def test_captured_print(captured_print):
                out, err = captured_print
                assert out == 'stdout contents begin\\n'
                assert err == 'stderr contents begin\\n'
        """.format(
                fixture=fixture
            )
        )
        result = testdir.runpytest_subprocess()
        result.stdout.fnmatch_lines(["*1 passed*"])
        assert "stdout contents begin" not in result.stdout.str()
        assert "stderr contents begin" not in result.stdout.str()

    @pytest.mark.parametrize("cap", ["capsys", "capfd"])
    def test_fixture_use_by_other_fixtures_teardown(self, testdir, cap):
        """Ensure we can access setup and teardown buffers from teardown when using capsys/capfd (##3033)"""
        testdir.makepyfile(
            """\
            import sys
            import pytest
            import os

            @pytest.fixture()
            def fix({cap}):
                print("setup out")
                sys.stderr.write("setup err\\n")
                yield
                out, err = {cap}.readouterr()
                assert out == 'setup out\\ncall out\\n'
                assert err == 'setup err\\ncall err\\n'

            def test_a(fix):
                print("call out")
                sys.stderr.write("call err\\n")
        """.format(
                cap=cap
            )
        )
        reprec = testdir.inline_run()
        reprec.assertoutcome(passed=1)


def test_setup_failure_does_not_kill_capturing(testdir):
    sub1 = testdir.mkpydir("sub1")
    sub1.join("conftest.py").write(
        textwrap.dedent(
            """\
            def pytest_runtest_setup(item):
                raise ValueError(42)
            """
        )
    )
    sub1.join("test_mod.py").write("def test_func1(): pass")
    result = testdir.runpytest(testdir.tmpdir, "--traceconfig")
    result.stdout.fnmatch_lines(["*ValueError(42)*", "*1 error*"])


def test_fdfuncarg_skips_on_no_osdup(testdir):
    testdir.makepyfile(
        """
        import os
        if hasattr(os, 'dup'):
            del os.dup
        def test_hello(capfd):
            pass
    """
    )
    result = testdir.runpytest_subprocess("--capture=no")
    result.stdout.fnmatch_lines(["*1 skipped*"])


def test_capture_conftest_runtest_setup(testdir):
    testdir.makeconftest(
        """
        def pytest_runtest_setup():
            print("hello19")
    """
    )
    testdir.makepyfile("def test_func(): pass")
    result = testdir.runpytest()
    assert result.ret == 0
    assert "hello19" not in result.stdout.str()


def test_capture_badoutput_issue412(testdir):
    testdir.makepyfile(
        """
        import os

        def test_func():
            omg = bytearray([1,129,1])
            os.write(1, omg)
            assert 0
        """
    )
    result = testdir.runpytest("--cap=fd")
    result.stdout.fnmatch_lines(
        """
        *def test_func*
        *assert 0*
        *Captured*
        *1 failed*
    """
    )


def test_capture_early_option_parsing(testdir):
    testdir.makeconftest(
        """
        def pytest_runtest_setup():
            print("hello19")
    """
    )
    testdir.makepyfile("def test_func(): pass")
    result = testdir.runpytest("-vs")
    assert result.ret == 0
    assert "hello19" in result.stdout.str()


def test_capture_binary_output(testdir):
    testdir.makepyfile(
        r"""
        import pytest

        def test_a():
            import sys
            import subprocess
            subprocess.call([sys.executable, __file__])

        def test_foo():
            import os;os.write(1, b'\xc3')

        if __name__ == '__main__':
            test_foo()
        """
    )
    result = testdir.runpytest("--assert=plain")
    result.assert_outcomes(passed=2)


def test_error_during_readouterr(testdir):
    """Make sure we suspend capturing if errors occur during readouterr"""
    testdir.makepyfile(
        pytest_xyz="""
        from _pytest.capture import FDCapture

        def bad_snap(self):
            raise Exception('boom')

        assert FDCapture.snap
        FDCapture.snap = bad_snap
    """
    )
    result = testdir.runpytest_subprocess("-p", "pytest_xyz", "--version")
    result.stderr.fnmatch_lines(
        ["*in bad_snap", "    raise Exception('boom')", "Exception: boom"]
    )


class TestCaptureIO(object):
    def test_text(self):
        f = capture.CaptureIO()
        f.write("hello")
        s = f.getvalue()
        assert s == "hello"
        f.close()

    def test_unicode_and_str_mixture(self):
        f = capture.CaptureIO()
        if sys.version_info >= (3, 0):
            f.write("\u00f6")
            pytest.raises(TypeError, f.write, b"hello")
        else:
            f.write(u"\u00f6")
            f.write(b"hello")
            s = f.getvalue()
            f.close()
            assert isinstance(s, text_type)

    @pytest.mark.skipif(sys.version_info[0] == 2, reason="python 3 only behaviour")
    def test_write_bytes_to_buffer(self):
        """In python3, stdout / stderr are text io wrappers (exposing a buffer
        property of the underlying bytestream).  See issue #1407
        """
        f = capture.CaptureIO()
        f.buffer.write(b"foo\r\n")
        assert f.getvalue() == "foo\r\n"


def test_dontreadfrominput():
    from _pytest.capture import DontReadFromInput

    f = DontReadFromInput()
    assert not f.isatty()
    pytest.raises(IOError, f.read)
    pytest.raises(IOError, f.readlines)
    iter_f = iter(f)
    pytest.raises(IOError, next, iter_f)
    pytest.raises(UnsupportedOperation, f.fileno)
    f.close()  # just for completeness


@pytest.mark.skipif("sys.version_info < (3,)", reason="python2 has no buffer")
def test_dontreadfrominput_buffer_python3():
    from _pytest.capture import DontReadFromInput

    f = DontReadFromInput()
    fb = f.buffer
    assert not fb.isatty()
    pytest.raises(IOError, fb.read)
    pytest.raises(IOError, fb.readlines)
    iter_f = iter(f)
    pytest.raises(IOError, next, iter_f)
    pytest.raises(ValueError, fb.fileno)
    f.close()  # just for completeness


@pytest.mark.skipif("sys.version_info >= (3,)", reason="python2 has no buffer")
def test_dontreadfrominput_buffer_python2():
    from _pytest.capture import DontReadFromInput

    f = DontReadFromInput()
    with pytest.raises(AttributeError):
        f.buffer
    f.close()  # just for completeness


@pytest.yield_fixture
def tmpfile(testdir):
    f = testdir.makepyfile("").open("wb+")
    yield f
    if not f.closed:
        f.close()


@needsosdup
def test_dupfile(tmpfile):
    flist = []
    for i in range(5):
        nf = capture.safe_text_dupfile(tmpfile, "wb")
        assert nf != tmpfile
        assert nf.fileno() != tmpfile.fileno()
        assert nf not in flist
        print(i, end="", file=nf)
        flist.append(nf)

    fname_open = flist[0].name
    assert fname_open == repr(flist[0].buffer)

    for i in range(5):
        f = flist[i]
        f.close()
    fname_closed = flist[0].name
    assert fname_closed == repr(flist[0].buffer)
    assert fname_closed != fname_open
    tmpfile.seek(0)
    s = tmpfile.read()
    assert "01234" in repr(s)
    tmpfile.close()
    assert fname_closed == repr(flist[0].buffer)


def test_dupfile_on_bytesio():
    bio = io.BytesIO()
    f = capture.safe_text_dupfile(bio, "wb")
    f.write("hello")
    assert bio.getvalue() == b"hello"
    assert "BytesIO object" in f.name


def test_dupfile_on_textio():
    tio = py.io.TextIO()
    f = capture.safe_text_dupfile(tio, "wb")
    f.write("hello")
    assert tio.getvalue() == "hello"
    assert not hasattr(f, "name")


@contextlib.contextmanager
def lsof_check():
    pid = os.getpid()
    try:
        out = subprocess.check_output(("lsof", "-p", str(pid))).decode()
    except (OSError, subprocess.CalledProcessError, UnicodeDecodeError):
        # about UnicodeDecodeError, see note on pytester
        pytest.skip("could not run 'lsof'")
    yield
    out2 = subprocess.check_output(("lsof", "-p", str(pid))).decode()
    len1 = len([x for x in out.split("\n") if "REG" in x])
    len2 = len([x for x in out2.split("\n") if "REG" in x])
    assert len2 < len1 + 3, out2


class TestFDCapture(object):
    pytestmark = needsosdup

    def test_simple(self, tmpfile):
        fd = tmpfile.fileno()
        cap = capture.FDCapture(fd)
        data = b"hello"
        os.write(fd, data)
        s = cap.snap()
        cap.done()
        assert not s
        cap = capture.FDCapture(fd)
        cap.start()
        os.write(fd, data)
        s = cap.snap()
        cap.done()
        assert s == "hello"

    def test_simple_many(self, tmpfile):
        for i in range(10):
            self.test_simple(tmpfile)

    def test_simple_many_check_open_files(self, testdir):
        with lsof_check():
            with testdir.makepyfile("").open("wb+") as tmpfile:
                self.test_simple_many(tmpfile)

    def test_simple_fail_second_start(self, tmpfile):
        fd = tmpfile.fileno()
        cap = capture.FDCapture(fd)
        cap.done()
        pytest.raises(ValueError, cap.start)

    def test_stderr(self):
        cap = capture.FDCapture(2)
        cap.start()
        print("hello", file=sys.stderr)
        s = cap.snap()
        cap.done()
        assert s == "hello\n"

    def test_stdin(self, tmpfile):
        cap = capture.FDCapture(0)
        cap.start()
        x = os.read(0, 100).strip()
        cap.done()
        assert x == b""

    def test_writeorg(self, tmpfile):
        data1, data2 = b"foo", b"bar"
        cap = capture.FDCapture(tmpfile.fileno())
        cap.start()
        tmpfile.write(data1)
        tmpfile.flush()
        cap.writeorg(data2)
        scap = cap.snap()
        cap.done()
        assert scap == data1.decode("ascii")
        with open(tmpfile.name, "rb") as stmp_file:
            stmp = stmp_file.read()
            assert stmp == data2

    def test_simple_resume_suspend(self, tmpfile):
        with saved_fd(1):
            cap = capture.FDCapture(1)
            cap.start()
            data = b"hello"
            os.write(1, data)
            sys.stdout.write("whatever")
            s = cap.snap()
            assert s == "hellowhatever"
            cap.suspend()
            os.write(1, b"world")
            sys.stdout.write("qlwkej")
            assert not cap.snap()
            cap.resume()
            os.write(1, b"but now")
            sys.stdout.write(" yes\n")
            s = cap.snap()
            assert s == "but now yes\n"
            cap.suspend()
            cap.done()
            pytest.raises(AttributeError, cap.suspend)

    def test_capfd_sys_stdout_mode(self, capfd):
        assert "b" not in sys.stdout.mode


@contextlib.contextmanager
def saved_fd(fd):
    new_fd = os.dup(fd)
    try:
        yield
    finally:
        os.dup2(new_fd, fd)
        os.close(new_fd)


class TestStdCapture(object):
    captureclass = staticmethod(StdCapture)

    @contextlib.contextmanager
    def getcapture(self, **kw):
        cap = self.__class__.captureclass(**kw)
        cap.start_capturing()
        try:
            yield cap
        finally:
            cap.stop_capturing()

    def test_capturing_done_simple(self):
        with self.getcapture() as cap:
            sys.stdout.write("hello")
            sys.stderr.write("world")
            out, err = cap.readouterr()
        assert out == "hello"
        assert err == "world"

    def test_capturing_reset_simple(self):
        with self.getcapture() as cap:
            print("hello world")
            sys.stderr.write("hello error\n")
            out, err = cap.readouterr()
        assert out == "hello world\n"
        assert err == "hello error\n"

    def test_capturing_readouterr(self):
        with self.getcapture() as cap:
            print("hello world")
            sys.stderr.write("hello error\n")
            out, err = cap.readouterr()
            assert out == "hello world\n"
            assert err == "hello error\n"
            sys.stderr.write("error2")
            out, err = cap.readouterr()
        assert err == "error2"

    def test_capture_results_accessible_by_attribute(self):
        with self.getcapture() as cap:
            sys.stdout.write("hello")
            sys.stderr.write("world")
            capture_result = cap.readouterr()
        assert capture_result.out == "hello"
        assert capture_result.err == "world"

    def test_capturing_readouterr_unicode(self):
        with self.getcapture() as cap:
            print("hxąć")
            out, err = cap.readouterr()
        assert out == u"hxąć\n"

    @pytest.mark.skipif(
        "sys.version_info >= (3,)", reason="text output different for bytes on python3"
    )
    def test_capturing_readouterr_decode_error_handling(self):
        with self.getcapture() as cap:
            # triggered an internal error in pytest
            print("\xa6")
            out, err = cap.readouterr()
        assert out == u"\ufffd\n"

    def test_reset_twice_error(self):
        with self.getcapture() as cap:
            print("hello")
            out, err = cap.readouterr()
        pytest.raises(ValueError, cap.stop_capturing)
        assert out == "hello\n"
        assert not err

    def test_capturing_modify_sysouterr_in_between(self):
        oldout = sys.stdout
        olderr = sys.stderr
        with self.getcapture() as cap:
            sys.stdout.write("hello")
            sys.stderr.write("world")
            sys.stdout = capture.CaptureIO()
            sys.stderr = capture.CaptureIO()
            print("not seen")
            sys.stderr.write("not seen\n")
            out, err = cap.readouterr()
        assert out == "hello"
        assert err == "world"
        assert sys.stdout == oldout
        assert sys.stderr == olderr

    def test_capturing_error_recursive(self):
        with self.getcapture() as cap1:
            print("cap1")
            with self.getcapture() as cap2:
                print("cap2")
                out2, err2 = cap2.readouterr()
                out1, err1 = cap1.readouterr()
        assert out1 == "cap1\n"
        assert out2 == "cap2\n"

    def test_just_out_capture(self):
        with self.getcapture(out=True, err=False) as cap:
            sys.stdout.write("hello")
            sys.stderr.write("world")
            out, err = cap.readouterr()
        assert out == "hello"
        assert not err

    def test_just_err_capture(self):
        with self.getcapture(out=False, err=True) as cap:
            sys.stdout.write("hello")
            sys.stderr.write("world")
            out, err = cap.readouterr()
        assert err == "world"
        assert not out

    def test_stdin_restored(self):
        old = sys.stdin
        with self.getcapture(in_=True):
            newstdin = sys.stdin
        assert newstdin != sys.stdin
        assert sys.stdin is old

    def test_stdin_nulled_by_default(self):
        print("XXX this test may well hang instead of crashing")
        print("XXX which indicates an error in the underlying capturing")
        print("XXX mechanisms")
        with self.getcapture():
            pytest.raises(IOError, sys.stdin.read)


class TestStdCaptureFD(TestStdCapture):
    pytestmark = needsosdup
    captureclass = staticmethod(StdCaptureFD)

    def test_simple_only_fd(self, testdir):
        testdir.makepyfile(
            """
            import os
            def test_x():
                os.write(1, "hello\\n".encode("ascii"))
                assert 0
        """
        )
        result = testdir.runpytest_subprocess()
        result.stdout.fnmatch_lines(
            """
            *test_x*
            *assert 0*
            *Captured stdout*
        """
        )

    def test_intermingling(self):
        with self.getcapture() as cap:
            os.write(1, b"1")
            sys.stdout.write(str(2))
            sys.stdout.flush()
            os.write(1, b"3")
            os.write(2, b"a")
            sys.stderr.write("b")
            sys.stderr.flush()
            os.write(2, b"c")
            out, err = cap.readouterr()
        assert out == "123"
        assert err == "abc"

    def test_many(self, capfd):
        with lsof_check():
            for i in range(10):
                cap = StdCaptureFD()
                cap.stop_capturing()


class TestStdCaptureFDinvalidFD(object):
    pytestmark = needsosdup

    def test_stdcapture_fd_invalid_fd(self, testdir):
        testdir.makepyfile(
            """
            import os
            from _pytest import capture

            def StdCaptureFD(out=True, err=True, in_=True):
                return capture.MultiCapture(out, err, in_, Capture=capture.FDCapture)

            def test_stdout():
                os.close(1)
                cap = StdCaptureFD(out=True, err=False, in_=False)
                assert repr(cap.out) == "<FDCapture 1 oldfd=None _state=None>"
                cap.stop_capturing()

            def test_stderr():
                os.close(2)
                cap = StdCaptureFD(out=False, err=True, in_=False)
                assert repr(cap.err) == "<FDCapture 2 oldfd=None _state=None>"
                cap.stop_capturing()

            def test_stdin():
                os.close(0)
                cap = StdCaptureFD(out=False, err=False, in_=True)
                assert repr(cap.in_) == "<FDCapture 0 oldfd=None _state=None>"
                cap.stop_capturing()
        """
        )
        result = testdir.runpytest_subprocess("--capture=fd")
        assert result.ret == 0
        assert result.parseoutcomes()["passed"] == 3


def test_capture_not_started_but_reset():
    capsys = StdCapture()
    capsys.stop_capturing()


def test_using_capsys_fixture_works_with_sys_stdout_encoding(capsys):
    test_text = "test text"

    print(test_text.encode(sys.stdout.encoding, "replace"))
    (out, err) = capsys.readouterr()
    assert out
    assert err == ""


def test_capsys_results_accessible_by_attribute(capsys):
    sys.stdout.write("spam")
    sys.stderr.write("eggs")
    capture_result = capsys.readouterr()
    assert capture_result.out == "spam"
    assert capture_result.err == "eggs"


@needsosdup
@pytest.mark.parametrize("use", [True, False])
def test_fdcapture_tmpfile_remains_the_same(tmpfile, use):
    if not use:
        tmpfile = True
    cap = StdCaptureFD(out=False, err=tmpfile)
    try:
        cap.start_capturing()
        capfile = cap.err.tmpfile
        cap.readouterr()
    finally:
        cap.stop_capturing()
    capfile2 = cap.err.tmpfile
    assert capfile2 == capfile


@needsosdup
def test_close_and_capture_again(testdir):
    testdir.makepyfile(
        """
        import os
        def test_close():
            os.close(1)
        def test_capture_again():
            os.write(1, b"hello\\n")
            assert 0
    """
    )
    result = testdir.runpytest_subprocess()
    result.stdout.fnmatch_lines(
        """
        *test_capture_again*
        *assert 0*
        *stdout*
        *hello*
    """
    )


@pytest.mark.parametrize("method", ["SysCapture", "FDCapture"])
def test_capturing_and_logging_fundamentals(testdir, method):
    if method == "StdCaptureFD" and not hasattr(os, "dup"):
        pytest.skip("need os.dup")
    # here we check a fundamental feature
    p = testdir.makepyfile(
        """
        import sys, os
        import py, logging
        from _pytest import capture
        cap = capture.MultiCapture(out=False, in_=False,
                                     Capture=capture.%s)
        cap.start_capturing()

        logging.warning("hello1")
        outerr = cap.readouterr()
        print("suspend, captured %%s" %%(outerr,))
        logging.warning("hello2")

        cap.pop_outerr_to_orig()
        logging.warning("hello3")

        outerr = cap.readouterr()
        print("suspend2, captured %%s" %% (outerr,))
    """
        % (method,)
    )
    result = testdir.runpython(p)
    result.stdout.fnmatch_lines(
        """
        suspend, captured*hello1*
        suspend2, captured*WARNING:root:hello3*
    """
    )
    result.stderr.fnmatch_lines(
        """
        WARNING:root:hello2
    """
    )
    assert "atexit" not in result.stderr.str()


def test_error_attribute_issue555(testdir):
    testdir.makepyfile(
        """
        import sys
        def test_capattr():
            assert sys.stdout.errors == "strict"
            assert sys.stderr.errors == "strict"
    """
    )
    reprec = testdir.inline_run()
    reprec.assertoutcome(passed=1)


@pytest.mark.skipif(
    not sys.platform.startswith("win") and sys.version_info[:2] >= (3, 6),
    reason="only py3.6+ on windows",
)
def test_py36_windowsconsoleio_workaround_non_standard_streams():
    """
    Ensure _py36_windowsconsoleio_workaround function works with objects that
    do not implement the full ``io``-based stream protocol, for example execnet channels (#2666).
    """
    from _pytest.capture import _py36_windowsconsoleio_workaround

    class DummyStream(object):
        def write(self, s):
            pass

    stream = DummyStream()
    _py36_windowsconsoleio_workaround(stream)


def test_dontreadfrominput_has_encoding(testdir):
    testdir.makepyfile(
        """
        import sys
        def test_capattr():
            # should not raise AttributeError
            assert sys.stdout.encoding
            assert sys.stderr.encoding
    """
    )
    reprec = testdir.inline_run()
    reprec.assertoutcome(passed=1)


def test_crash_on_closing_tmpfile_py27(testdir):
    p = testdir.makepyfile(
        """
        from __future__ import print_function
        import threading
        import sys

        printing = threading.Event()

        def spam():
            f = sys.stderr
            print('SPAMBEFORE', end='', file=f)
            printing.set()

            while True:
                try:
                    f.flush()
                except (OSError, ValueError):
                    break

        def test_spam_in_thread():
            t = threading.Thread(target=spam)
            t.daemon = True
            t.start()

            printing.wait()
    """
    )
    result = testdir.runpytest_subprocess(str(p))
    assert result.ret == 0
    assert result.stderr.str() == ""
    assert "IOError" not in result.stdout.str()


def test_pickling_and_unpickling_encoded_file():
    # See https://bitbucket.org/pytest-dev/pytest/pull-request/194
    # pickle.loads() raises infinite recursion if
    # EncodedFile.__getattr__ is not implemented properly
    ef = capture.EncodedFile(None, None)
    ef_as_str = pickle.dumps(ef)
    pickle.loads(ef_as_str)


def test_global_capture_with_live_logging(testdir):
    # Issue 3819
    # capture should work with live cli logging

    # Teardown report seems to have the capture for the whole process (setup, capture, teardown)
    testdir.makeconftest(
        """
        def pytest_runtest_logreport(report):
            if "test_global" in report.nodeid:
                if report.when == "teardown":
                    with open("caplog", "w") as f:
                        f.write(report.caplog)
                    with open("capstdout", "w") as f:
                        f.write(report.capstdout)
        """
    )

    testdir.makepyfile(
        """
        import logging
        import sys
        import pytest

        logger = logging.getLogger(__name__)

        @pytest.fixture
        def fix1():
            print("fix setup")
            logging.info("fix setup")
            yield
            logging.info("fix teardown")
            print("fix teardown")

        def test_global(fix1):
            print("begin test")
            logging.info("something in test")
            print("end test")
        """
    )
    result = testdir.runpytest_subprocess("--log-cli-level=INFO")
    assert result.ret == 0

    with open("caplog", "r") as f:
        caplog = f.read()

    assert "fix setup" in caplog
    assert "something in test" in caplog
    assert "fix teardown" in caplog

    with open("capstdout", "r") as f:
        capstdout = f.read()

    assert "fix setup" in capstdout
    assert "begin test" in capstdout
    assert "end test" in capstdout
    assert "fix teardown" in capstdout


@pytest.mark.parametrize("capture_fixture", ["capsys", "capfd"])
def test_capture_with_live_logging(testdir, capture_fixture):
    # Issue 3819
    # capture should work with live cli logging

    testdir.makepyfile(
        """
        import logging
        import sys

        logger = logging.getLogger(__name__)

        def test_capture({0}):
            print("hello")
            sys.stderr.write("world\\n")
            captured = {0}.readouterr()
            assert captured.out == "hello\\n"
            assert captured.err == "world\\n"

            logging.info("something")
            print("next")
            logging.info("something")

            captured = {0}.readouterr()
            assert captured.out == "next\\n"
        """.format(
            capture_fixture
        )
    )

    result = testdir.runpytest_subprocess("--log-cli-level=INFO")
    assert result.ret == 0


def test_typeerror_encodedfile_write(testdir):
    """It should behave the same with and without output capturing (#4861)."""
    p = testdir.makepyfile(
        """
        def test_fails():
            import sys
            sys.stdout.write(b"foo")
    """
    )
    result_without_capture = testdir.runpytest("-s", str(p))

    result_with_capture = testdir.runpytest(str(p))

    assert result_with_capture.ret == result_without_capture.ret

    if _PY3:
        result_with_capture.stdout.fnmatch_lines(
            ["E           TypeError: write() argument must be str, not bytes"]
        )
    else:
        assert result_with_capture.ret == 0
