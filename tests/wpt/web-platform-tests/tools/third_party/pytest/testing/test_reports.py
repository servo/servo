import sys
from typing import Sequence
from typing import Union

import pytest
from _pytest._code.code import ExceptionChainRepr
from _pytest._code.code import ExceptionRepr
from _pytest.config import Config
from _pytest.pathlib import Path
from _pytest.pytester import Testdir
from _pytest.reports import CollectReport
from _pytest.reports import TestReport


class TestReportSerialization:
    def test_xdist_longrepr_to_str_issue_241(self, testdir: Testdir) -> None:
        """Regarding issue pytest-xdist#241.

        This test came originally from test_remote.py in xdist (ca03269).
        """
        testdir.makepyfile(
            """
            def test_a(): assert False
            def test_b(): pass
        """
        )
        reprec = testdir.inline_run()
        reports = reprec.getreports("pytest_runtest_logreport")
        assert len(reports) == 6
        test_a_call = reports[1]
        assert test_a_call.when == "call"
        assert test_a_call.outcome == "failed"
        assert test_a_call._to_json()["longrepr"]["reprtraceback"]["style"] == "long"
        test_b_call = reports[4]
        assert test_b_call.when == "call"
        assert test_b_call.outcome == "passed"
        assert test_b_call._to_json()["longrepr"] is None

    def test_xdist_report_longrepr_reprcrash_130(self, testdir: Testdir) -> None:
        """Regarding issue pytest-xdist#130

        This test came originally from test_remote.py in xdist (ca03269).
        """
        reprec = testdir.inline_runsource(
            """
                    def test_fail():
                        assert False, 'Expected Message'
                """
        )
        reports = reprec.getreports("pytest_runtest_logreport")
        assert len(reports) == 3
        rep = reports[1]
        added_section = ("Failure Metadata", "metadata metadata", "*")
        assert isinstance(rep.longrepr, ExceptionRepr)
        rep.longrepr.sections.append(added_section)
        d = rep._to_json()
        a = TestReport._from_json(d)
        assert isinstance(a.longrepr, ExceptionRepr)
        # Check assembled == rep
        assert a.__dict__.keys() == rep.__dict__.keys()
        for key in rep.__dict__.keys():
            if key != "longrepr":
                assert getattr(a, key) == getattr(rep, key)
        assert rep.longrepr.reprcrash is not None
        assert a.longrepr.reprcrash is not None
        assert rep.longrepr.reprcrash.lineno == a.longrepr.reprcrash.lineno
        assert rep.longrepr.reprcrash.message == a.longrepr.reprcrash.message
        assert rep.longrepr.reprcrash.path == a.longrepr.reprcrash.path
        assert rep.longrepr.reprtraceback.entrysep == a.longrepr.reprtraceback.entrysep
        assert (
            rep.longrepr.reprtraceback.extraline == a.longrepr.reprtraceback.extraline
        )
        assert rep.longrepr.reprtraceback.style == a.longrepr.reprtraceback.style
        assert rep.longrepr.sections == a.longrepr.sections
        # Missing section attribute PR171
        assert added_section in a.longrepr.sections

    def test_reprentries_serialization_170(self, testdir: Testdir) -> None:
        """Regarding issue pytest-xdist#170

        This test came originally from test_remote.py in xdist (ca03269).
        """
        from _pytest._code.code import ReprEntry

        reprec = testdir.inline_runsource(
            """
                            def test_repr_entry():
                                x = 0
                                assert x
                        """,
            "--showlocals",
        )
        reports = reprec.getreports("pytest_runtest_logreport")
        assert len(reports) == 3
        rep = reports[1]
        assert isinstance(rep.longrepr, ExceptionRepr)
        d = rep._to_json()
        a = TestReport._from_json(d)
        assert isinstance(a.longrepr, ExceptionRepr)

        rep_entries = rep.longrepr.reprtraceback.reprentries
        a_entries = a.longrepr.reprtraceback.reprentries
        for i in range(len(a_entries)):
            rep_entry = rep_entries[i]
            assert isinstance(rep_entry, ReprEntry)
            assert rep_entry.reprfileloc is not None
            assert rep_entry.reprfuncargs is not None
            assert rep_entry.reprlocals is not None

            a_entry = a_entries[i]
            assert isinstance(a_entry, ReprEntry)
            assert a_entry.reprfileloc is not None
            assert a_entry.reprfuncargs is not None
            assert a_entry.reprlocals is not None

            assert rep_entry.lines == a_entry.lines
            assert rep_entry.reprfileloc.lineno == a_entry.reprfileloc.lineno
            assert rep_entry.reprfileloc.message == a_entry.reprfileloc.message
            assert rep_entry.reprfileloc.path == a_entry.reprfileloc.path
            assert rep_entry.reprfuncargs.args == a_entry.reprfuncargs.args
            assert rep_entry.reprlocals.lines == a_entry.reprlocals.lines
            assert rep_entry.style == a_entry.style

    def test_reprentries_serialization_196(self, testdir: Testdir) -> None:
        """Regarding issue pytest-xdist#196

        This test came originally from test_remote.py in xdist (ca03269).
        """
        from _pytest._code.code import ReprEntryNative

        reprec = testdir.inline_runsource(
            """
                            def test_repr_entry_native():
                                x = 0
                                assert x
                        """,
            "--tb=native",
        )
        reports = reprec.getreports("pytest_runtest_logreport")
        assert len(reports) == 3
        rep = reports[1]
        assert isinstance(rep.longrepr, ExceptionRepr)
        d = rep._to_json()
        a = TestReport._from_json(d)
        assert isinstance(a.longrepr, ExceptionRepr)

        rep_entries = rep.longrepr.reprtraceback.reprentries
        a_entries = a.longrepr.reprtraceback.reprentries
        for i in range(len(a_entries)):
            assert isinstance(rep_entries[i], ReprEntryNative)
            assert rep_entries[i].lines == a_entries[i].lines

    def test_itemreport_outcomes(self, testdir: Testdir) -> None:
        # This test came originally from test_remote.py in xdist (ca03269).
        reprec = testdir.inline_runsource(
            """
            import pytest
            def test_pass(): pass
            def test_fail(): 0/0
            @pytest.mark.skipif("True")
            def test_skip(): pass
            def test_skip_imperative():
                pytest.skip("hello")
            @pytest.mark.xfail("True")
            def test_xfail(): 0/0
            def test_xfail_imperative():
                pytest.xfail("hello")
        """
        )
        reports = reprec.getreports("pytest_runtest_logreport")
        assert len(reports) == 17  # with setup/teardown "passed" reports
        for rep in reports:
            d = rep._to_json()
            newrep = TestReport._from_json(d)
            assert newrep.passed == rep.passed
            assert newrep.failed == rep.failed
            assert newrep.skipped == rep.skipped
            if newrep.skipped and not hasattr(newrep, "wasxfail"):
                assert isinstance(newrep.longrepr, tuple)
                assert len(newrep.longrepr) == 3
            assert newrep.outcome == rep.outcome
            assert newrep.when == rep.when
            assert newrep.keywords == rep.keywords
            if rep.failed:
                assert newrep.longreprtext == rep.longreprtext

    def test_collectreport_passed(self, testdir: Testdir) -> None:
        """This test came originally from test_remote.py in xdist (ca03269)."""
        reprec = testdir.inline_runsource("def test_func(): pass")
        reports = reprec.getreports("pytest_collectreport")
        for rep in reports:
            d = rep._to_json()
            newrep = CollectReport._from_json(d)
            assert newrep.passed == rep.passed
            assert newrep.failed == rep.failed
            assert newrep.skipped == rep.skipped

    def test_collectreport_fail(self, testdir: Testdir) -> None:
        """This test came originally from test_remote.py in xdist (ca03269)."""
        reprec = testdir.inline_runsource("qwe abc")
        reports = reprec.getreports("pytest_collectreport")
        assert reports
        for rep in reports:
            d = rep._to_json()
            newrep = CollectReport._from_json(d)
            assert newrep.passed == rep.passed
            assert newrep.failed == rep.failed
            assert newrep.skipped == rep.skipped
            if rep.failed:
                assert newrep.longrepr == str(rep.longrepr)

    def test_extended_report_deserialization(self, testdir: Testdir) -> None:
        """This test came originally from test_remote.py in xdist (ca03269)."""
        reprec = testdir.inline_runsource("qwe abc")
        reports = reprec.getreports("pytest_collectreport")
        assert reports
        for rep in reports:
            rep.extra = True  # type: ignore[attr-defined]
            d = rep._to_json()
            newrep = CollectReport._from_json(d)
            assert newrep.extra
            assert newrep.passed == rep.passed
            assert newrep.failed == rep.failed
            assert newrep.skipped == rep.skipped
            if rep.failed:
                assert newrep.longrepr == str(rep.longrepr)

    def test_paths_support(self, testdir: Testdir) -> None:
        """Report attributes which are py.path or pathlib objects should become strings."""
        testdir.makepyfile(
            """
            def test_a():
                assert False
        """
        )
        reprec = testdir.inline_run()
        reports = reprec.getreports("pytest_runtest_logreport")
        assert len(reports) == 3
        test_a_call = reports[1]
        test_a_call.path1 = testdir.tmpdir  # type: ignore[attr-defined]
        test_a_call.path2 = Path(testdir.tmpdir)  # type: ignore[attr-defined]
        data = test_a_call._to_json()
        assert data["path1"] == str(testdir.tmpdir)
        assert data["path2"] == str(testdir.tmpdir)

    def test_deserialization_failure(self, testdir: Testdir) -> None:
        """Check handling of failure during deserialization of report types."""
        testdir.makepyfile(
            """
            def test_a():
                assert False
        """
        )
        reprec = testdir.inline_run()
        reports = reprec.getreports("pytest_runtest_logreport")
        assert len(reports) == 3
        test_a_call = reports[1]
        data = test_a_call._to_json()
        entry = data["longrepr"]["reprtraceback"]["reprentries"][0]
        assert entry["type"] == "ReprEntry"

        entry["type"] = "Unknown"
        with pytest.raises(
            RuntimeError, match="INTERNALERROR: Unknown entry type returned: Unknown"
        ):
            TestReport._from_json(data)

    @pytest.mark.parametrize("report_class", [TestReport, CollectReport])
    def test_chained_exceptions(self, testdir: Testdir, tw_mock, report_class) -> None:
        """Check serialization/deserialization of report objects containing chained exceptions (#5786)"""
        testdir.makepyfile(
            """
            def foo():
                raise ValueError('value error')
            def test_a():
                try:
                    foo()
                except ValueError as e:
                    raise RuntimeError('runtime error') from e
            if {error_during_import}:
                test_a()
        """.format(
                error_during_import=report_class is CollectReport
            )
        )

        reprec = testdir.inline_run()
        if report_class is TestReport:
            reports = reprec.getreports(
                "pytest_runtest_logreport"
            )  # type: Union[Sequence[TestReport], Sequence[CollectReport]]
            # we have 3 reports: setup/call/teardown
            assert len(reports) == 3
            # get the call report
            report = reports[1]
        else:
            assert report_class is CollectReport
            # two collection reports: session and test file
            reports = reprec.getreports("pytest_collectreport")
            assert len(reports) == 2
            report = reports[1]

        def check_longrepr(longrepr: ExceptionChainRepr) -> None:
            """Check the attributes of the given longrepr object according to the test file.

            We can get away with testing both CollectReport and TestReport with this function because
            the longrepr objects are very similar.
            """
            assert isinstance(longrepr, ExceptionChainRepr)
            assert longrepr.sections == [("title", "contents", "=")]
            assert len(longrepr.chain) == 2
            entry1, entry2 = longrepr.chain
            tb1, fileloc1, desc1 = entry1
            tb2, fileloc2, desc2 = entry2

            assert "ValueError('value error')" in str(tb1)
            assert "RuntimeError('runtime error')" in str(tb2)

            assert (
                desc1
                == "The above exception was the direct cause of the following exception:"
            )
            assert desc2 is None

        assert report.failed
        assert len(report.sections) == 0
        assert isinstance(report.longrepr, ExceptionChainRepr)
        report.longrepr.addsection("title", "contents", "=")
        check_longrepr(report.longrepr)

        data = report._to_json()
        loaded_report = report_class._from_json(data)

        assert loaded_report.failed
        check_longrepr(loaded_report.longrepr)

        # make sure we don't blow up on ``toterminal`` call; we don't test the actual output because it is very
        # brittle and hard to maintain, but we can assume it is correct because ``toterminal`` is already tested
        # elsewhere and we do check the contents of the longrepr object after loading it.
        loaded_report.longrepr.toterminal(tw_mock)

    def test_chained_exceptions_no_reprcrash(self, testdir: Testdir, tw_mock) -> None:
        """Regression test for tracebacks without a reprcrash (#5971)

        This happens notably on exceptions raised by multiprocess.pool: the exception transfer
        from subprocess to main process creates an artificial exception, which ExceptionInfo
        can't obtain the ReprFileLocation from.
        """
        # somehow in Python 3.5 on Windows this test fails with:
        #   File "c:\...\3.5.4\x64\Lib\multiprocessing\connection.py", line 302, in _recv_bytes
        #     overlapped=True)
        # OSError: [WinError 6] The handle is invalid
        #
        # so in this platform we opted to use a mock traceback which is identical to the
        # one produced by the multiprocessing module
        if sys.version_info[:2] <= (3, 5) and sys.platform.startswith("win"):
            testdir.makepyfile(
                """
                # equivalent of multiprocessing.pool.RemoteTraceback
                class RemoteTraceback(Exception):
                    def __init__(self, tb):
                        self.tb = tb
                    def __str__(self):
                        return self.tb
                def test_a():
                    try:
                        raise ValueError('value error')
                    except ValueError as e:
                        # equivalent to how multiprocessing.pool.rebuild_exc does it
                        e.__cause__ = RemoteTraceback('runtime error')
                        raise e
            """
            )
        else:
            testdir.makepyfile(
                """
                from concurrent.futures import ProcessPoolExecutor

                def func():
                    raise ValueError('value error')

                def test_a():
                    with ProcessPoolExecutor() as p:
                        p.submit(func).result()
            """
            )

        testdir.syspathinsert()
        reprec = testdir.inline_run()

        reports = reprec.getreports("pytest_runtest_logreport")

        def check_longrepr(longrepr: object) -> None:
            assert isinstance(longrepr, ExceptionChainRepr)
            assert len(longrepr.chain) == 2
            entry1, entry2 = longrepr.chain
            tb1, fileloc1, desc1 = entry1
            tb2, fileloc2, desc2 = entry2

            assert "RemoteTraceback" in str(tb1)
            assert "ValueError: value error" in str(tb2)

            assert fileloc1 is None
            assert fileloc2 is not None
            assert fileloc2.message == "ValueError: value error"

        # 3 reports: setup/call/teardown: get the call report
        assert len(reports) == 3
        report = reports[1]

        assert report.failed
        check_longrepr(report.longrepr)

        data = report._to_json()
        loaded_report = TestReport._from_json(data)

        assert loaded_report.failed
        check_longrepr(loaded_report.longrepr)

        # for same reasons as previous test, ensure we don't blow up here
        assert loaded_report.longrepr is not None
        assert isinstance(loaded_report.longrepr, ExceptionChainRepr)
        loaded_report.longrepr.toterminal(tw_mock)

    def test_report_prevent_ConftestImportFailure_hiding_exception(
        self, testdir: Testdir
    ) -> None:
        sub_dir = testdir.tmpdir.join("ns").ensure_dir()
        sub_dir.join("conftest").new(ext=".py").write("import unknown")

        result = testdir.runpytest_subprocess(".")
        result.stdout.fnmatch_lines(["E   *Error: No module named 'unknown'"])
        result.stdout.no_fnmatch_line("ERROR  - *ConftestImportFailure*")


class TestHooks:
    """Test that the hooks are working correctly for plugins"""

    def test_test_report(self, testdir: Testdir, pytestconfig: Config) -> None:
        testdir.makepyfile(
            """
            def test_a(): assert False
            def test_b(): pass
        """
        )
        reprec = testdir.inline_run()
        reports = reprec.getreports("pytest_runtest_logreport")
        assert len(reports) == 6
        for rep in reports:
            data = pytestconfig.hook.pytest_report_to_serializable(
                config=pytestconfig, report=rep
            )
            assert data["$report_type"] == "TestReport"
            new_rep = pytestconfig.hook.pytest_report_from_serializable(
                config=pytestconfig, data=data
            )
            assert new_rep.nodeid == rep.nodeid
            assert new_rep.when == rep.when
            assert new_rep.outcome == rep.outcome

    def test_collect_report(self, testdir: Testdir, pytestconfig: Config) -> None:
        testdir.makepyfile(
            """
            def test_a(): assert False
            def test_b(): pass
        """
        )
        reprec = testdir.inline_run()
        reports = reprec.getreports("pytest_collectreport")
        assert len(reports) == 2
        for rep in reports:
            data = pytestconfig.hook.pytest_report_to_serializable(
                config=pytestconfig, report=rep
            )
            assert data["$report_type"] == "CollectReport"
            new_rep = pytestconfig.hook.pytest_report_from_serializable(
                config=pytestconfig, data=data
            )
            assert new_rep.nodeid == rep.nodeid
            assert new_rep.when == "collect"
            assert new_rep.outcome == rep.outcome

    @pytest.mark.parametrize(
        "hook_name", ["pytest_runtest_logreport", "pytest_collectreport"]
    )
    def test_invalid_report_types(
        self, testdir: Testdir, pytestconfig: Config, hook_name: str
    ) -> None:
        testdir.makepyfile(
            """
            def test_a(): pass
            """
        )
        reprec = testdir.inline_run()
        reports = reprec.getreports(hook_name)
        assert reports
        rep = reports[0]
        data = pytestconfig.hook.pytest_report_to_serializable(
            config=pytestconfig, report=rep
        )
        data["$report_type"] = "Unknown"
        with pytest.raises(AssertionError):
            _ = pytestconfig.hook.pytest_report_from_serializable(
                config=pytestconfig, data=data
            )
