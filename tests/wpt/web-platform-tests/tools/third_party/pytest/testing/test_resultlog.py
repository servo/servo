# -*- coding: utf-8 -*-
from __future__ import absolute_import
from __future__ import division
from __future__ import print_function

import os

import py

import _pytest._code
import pytest
from _pytest.resultlog import pytest_configure
from _pytest.resultlog import pytest_unconfigure
from _pytest.resultlog import ResultLog

pytestmark = pytest.mark.filterwarnings("ignore:--result-log is deprecated")


def test_write_log_entry():
    reslog = ResultLog(None, None)
    reslog.logfile = py.io.TextIO()
    reslog.write_log_entry("name", ".", "")
    entry = reslog.logfile.getvalue()
    assert entry[-1] == "\n"
    entry_lines = entry.splitlines()
    assert len(entry_lines) == 1
    assert entry_lines[0] == ". name"

    reslog.logfile = py.io.TextIO()
    reslog.write_log_entry("name", "s", "Skipped")
    entry = reslog.logfile.getvalue()
    assert entry[-1] == "\n"
    entry_lines = entry.splitlines()
    assert len(entry_lines) == 2
    assert entry_lines[0] == "s name"
    assert entry_lines[1] == " Skipped"

    reslog.logfile = py.io.TextIO()
    reslog.write_log_entry("name", "s", "Skipped\n")
    entry = reslog.logfile.getvalue()
    assert entry[-1] == "\n"
    entry_lines = entry.splitlines()
    assert len(entry_lines) == 2
    assert entry_lines[0] == "s name"
    assert entry_lines[1] == " Skipped"

    reslog.logfile = py.io.TextIO()
    longrepr = " tb1\n tb 2\nE tb3\nSome Error"
    reslog.write_log_entry("name", "F", longrepr)
    entry = reslog.logfile.getvalue()
    assert entry[-1] == "\n"
    entry_lines = entry.splitlines()
    assert len(entry_lines) == 5
    assert entry_lines[0] == "F name"
    assert entry_lines[1:] == [" " + line for line in longrepr.splitlines()]


class TestWithFunctionIntegration(object):
    # XXX (hpk) i think that the resultlog plugin should
    # provide a Parser object so that one can remain
    # ignorant regarding formatting details.
    def getresultlog(self, testdir, arg):
        resultlog = testdir.tmpdir.join("resultlog")
        testdir.plugins.append("resultlog")
        args = ["--resultlog=%s" % resultlog] + [arg]
        testdir.runpytest(*args)
        return [x for x in resultlog.readlines(cr=0) if x]

    def test_collection_report(self, testdir):
        ok = testdir.makepyfile(test_collection_ok="")
        fail = testdir.makepyfile(test_collection_fail="XXX")
        lines = self.getresultlog(testdir, ok)
        assert not lines

        lines = self.getresultlog(testdir, fail)
        assert lines
        assert lines[0].startswith("F ")
        assert lines[0].endswith("test_collection_fail.py"), lines[0]
        for x in lines[1:]:
            assert x.startswith(" ")
        assert "XXX" in "".join(lines[1:])

    def test_log_test_outcomes(self, testdir):
        mod = testdir.makepyfile(
            test_mod="""
            import pytest
            def test_pass(): pass
            def test_skip(): pytest.skip("hello")
            def test_fail(): raise ValueError("FAIL")

            @pytest.mark.xfail
            def test_xfail(): raise ValueError("XFAIL")
            @pytest.mark.xfail
            def test_xpass(): pass

        """
        )
        lines = self.getresultlog(testdir, mod)
        assert len(lines) >= 3
        assert lines[0].startswith(". ")
        assert lines[0].endswith("test_pass")
        assert lines[1].startswith("s "), lines[1]
        assert lines[1].endswith("test_skip")
        assert lines[2].find("hello") != -1

        assert lines[3].startswith("F ")
        assert lines[3].endswith("test_fail")
        tb = "".join(lines[4:8])
        assert tb.find('raise ValueError("FAIL")') != -1

        assert lines[8].startswith("x ")
        tb = "".join(lines[8:14])
        assert tb.find('raise ValueError("XFAIL")') != -1

        assert lines[14].startswith("X ")
        assert len(lines) == 15

    @pytest.mark.parametrize("style", ("native", "long", "short"))
    def test_internal_exception(self, style):
        # they are produced for example by a teardown failing
        # at the end of the run or a failing hook invocation
        try:
            raise ValueError
        except ValueError:
            excinfo = _pytest._code.ExceptionInfo.from_current()
        reslog = ResultLog(None, py.io.TextIO())
        reslog.pytest_internalerror(excinfo.getrepr(style=style))
        entry = reslog.logfile.getvalue()
        entry_lines = entry.splitlines()

        assert entry_lines[0].startswith("! ")
        if style != "native":
            assert os.path.basename(__file__)[:-9] in entry_lines[0]  # .pyc/class
        assert entry_lines[-1][0] == " "
        assert "ValueError" in entry


def test_generic(testdir, LineMatcher):
    testdir.plugins.append("resultlog")
    testdir.makepyfile(
        """
        import pytest
        def test_pass():
            pass
        def test_fail():
            assert 0
        def test_skip():
            pytest.skip("")
        @pytest.mark.xfail
        def test_xfail():
            assert 0
        @pytest.mark.xfail(run=False)
        def test_xfail_norun():
            assert 0
    """
    )
    testdir.runpytest("--resultlog=result.log")
    lines = testdir.tmpdir.join("result.log").readlines(cr=0)
    LineMatcher(lines).fnmatch_lines(
        [
            ". *:test_pass",
            "F *:test_fail",
            "s *:test_skip",
            "x *:test_xfail",
            "x *:test_xfail_norun",
        ]
    )


def test_makedir_for_resultlog(testdir, LineMatcher):
    """--resultlog should automatically create directories for the log file"""
    testdir.plugins.append("resultlog")
    testdir.makepyfile(
        """
        import pytest
        def test_pass():
            pass
    """
    )
    testdir.runpytest("--resultlog=path/to/result.log")
    lines = testdir.tmpdir.join("path/to/result.log").readlines(cr=0)
    LineMatcher(lines).fnmatch_lines([". *:test_pass"])


def test_no_resultlog_on_slaves(testdir):
    config = testdir.parseconfig("-p", "resultlog", "--resultlog=resultlog")

    assert not hasattr(config, "_resultlog")
    pytest_configure(config)
    assert hasattr(config, "_resultlog")
    pytest_unconfigure(config)
    assert not hasattr(config, "_resultlog")

    config.slaveinput = {}
    pytest_configure(config)
    assert not hasattr(config, "_resultlog")
    pytest_unconfigure(config)
    assert not hasattr(config, "_resultlog")


def test_failure_issue380(testdir):
    testdir.makeconftest(
        """
        import pytest
        class MyCollector(pytest.File):
            def collect(self):
                raise ValueError()
            def repr_failure(self, excinfo):
                return "somestring"
        def pytest_collect_file(path, parent):
            return MyCollector(parent=parent, fspath=path)
    """
    )
    testdir.makepyfile(
        """
        def test_func():
            pass
    """
    )
    result = testdir.runpytest("--resultlog=log")
    assert result.ret == 2
