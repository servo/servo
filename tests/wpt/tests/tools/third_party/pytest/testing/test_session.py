import pytest
from _pytest.config import ExitCode
from _pytest.monkeypatch import MonkeyPatch
from _pytest.pytester import Pytester


class SessionTests:
    def test_basic_testitem_events(self, pytester: Pytester) -> None:
        tfile = pytester.makepyfile(
            """
            def test_one():
                pass
            def test_one_one():
                assert 0
            def test_other():
                raise ValueError(23)
            class TestClass(object):
                def test_two(self, someargs):
                    pass
        """
        )
        reprec = pytester.inline_run(tfile)
        passed, skipped, failed = reprec.listoutcomes()
        assert len(skipped) == 0
        assert len(passed) == 1
        assert len(failed) == 3

        def end(x):
            return x.nodeid.split("::")[-1]

        assert end(failed[0]) == "test_one_one"
        assert end(failed[1]) == "test_other"
        itemstarted = reprec.getcalls("pytest_itemcollected")
        assert len(itemstarted) == 4
        # XXX check for failing funcarg setup
        # colreports = reprec.getcalls("pytest_collectreport")
        # assert len(colreports) == 4
        # assert colreports[1].report.failed

    def test_nested_import_error(self, pytester: Pytester) -> None:
        tfile = pytester.makepyfile(
            """
            import import_fails
            def test_this():
                assert import_fails.a == 1
        """,
            import_fails="""
            import does_not_work
            a = 1
        """,
        )
        reprec = pytester.inline_run(tfile)
        values = reprec.getfailedcollections()
        assert len(values) == 1
        out = str(values[0].longrepr)
        assert out.find("does_not_work") != -1

    def test_raises_output(self, pytester: Pytester) -> None:
        reprec = pytester.inline_runsource(
            """
            import pytest
            def test_raises_doesnt():
                pytest.raises(ValueError, int, "3")
        """
        )
        passed, skipped, failed = reprec.listoutcomes()
        assert len(failed) == 1
        out = failed[0].longrepr.reprcrash.message  # type: ignore[union-attr]
        assert "DID NOT RAISE" in out

    def test_syntax_error_module(self, pytester: Pytester) -> None:
        reprec = pytester.inline_runsource("this is really not python")
        values = reprec.getfailedcollections()
        assert len(values) == 1
        out = str(values[0].longrepr)
        assert out.find("not python") != -1

    def test_exit_first_problem(self, pytester: Pytester) -> None:
        reprec = pytester.inline_runsource(
            """
            def test_one(): assert 0
            def test_two(): assert 0
        """,
            "--exitfirst",
        )
        passed, skipped, failed = reprec.countoutcomes()
        assert failed == 1
        assert passed == skipped == 0

    def test_maxfail(self, pytester: Pytester) -> None:
        reprec = pytester.inline_runsource(
            """
            def test_one(): assert 0
            def test_two(): assert 0
            def test_three(): assert 0
        """,
            "--maxfail=2",
        )
        passed, skipped, failed = reprec.countoutcomes()
        assert failed == 2
        assert passed == skipped == 0

    def test_broken_repr(self, pytester: Pytester) -> None:
        p = pytester.makepyfile(
            """
            import pytest

            class reprexc(BaseException):
                def __str__(self):
                    return "Ha Ha fooled you, I'm a broken repr()."

            class BrokenRepr1(object):
                foo=0
                def __repr__(self):
                    raise reprexc

            class TestBrokenClass(object):
                def test_explicit_bad_repr(self):
                    t = BrokenRepr1()
                    with pytest.raises(BaseException, match="broken repr"):
                        repr(t)

                def test_implicit_bad_repr1(self):
                    t = BrokenRepr1()
                    assert t.foo == 1

        """
        )
        reprec = pytester.inline_run(p)
        passed, skipped, failed = reprec.listoutcomes()
        assert (len(passed), len(skipped), len(failed)) == (1, 0, 1)
        out = failed[0].longrepr.reprcrash.message  # type: ignore[union-attr]
        assert out.find("<[reprexc() raised in repr()] BrokenRepr1") != -1

    def test_broken_repr_with_showlocals_verbose(self, pytester: Pytester) -> None:
        p = pytester.makepyfile(
            """
            class ObjWithErrorInRepr:
                def __repr__(self):
                    raise NotImplementedError

            def test_repr_error():
                x = ObjWithErrorInRepr()
                assert x == "value"
        """
        )
        reprec = pytester.inline_run("--showlocals", "-vv", p)
        passed, skipped, failed = reprec.listoutcomes()
        assert (len(passed), len(skipped), len(failed)) == (0, 0, 1)
        entries = failed[0].longrepr.reprtraceback.reprentries  # type: ignore[union-attr]
        assert len(entries) == 1
        repr_locals = entries[0].reprlocals
        assert repr_locals.lines
        assert len(repr_locals.lines) == 1
        assert repr_locals.lines[0].startswith(
            "x          = <[NotImplementedError() raised in repr()] ObjWithErrorInRepr"
        )

    def test_skip_file_by_conftest(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            conftest="""
            import pytest
            def pytest_collect_file():
                pytest.skip("intentional")
        """,
            test_file="""
            def test_one(): pass
        """,
        )
        try:
            reprec = pytester.inline_run(pytester.path)
        except pytest.skip.Exception:  # pragma: no cover
            pytest.fail("wrong skipped caught")
        reports = reprec.getreports("pytest_collectreport")
        assert len(reports) == 1
        assert reports[0].skipped


class TestNewSession(SessionTests):
    def test_order_of_execution(self, pytester: Pytester) -> None:
        reprec = pytester.inline_runsource(
            """
            values = []
            def test_1():
                values.append(1)
            def test_2():
                values.append(2)
            def test_3():
                assert values == [1,2]
            class Testmygroup(object):
                reslist = values
                def test_1(self):
                    self.reslist.append(1)
                def test_2(self):
                    self.reslist.append(2)
                def test_3(self):
                    self.reslist.append(3)
                def test_4(self):
                    assert self.reslist == [1,2,1,2,3]
        """
        )
        passed, skipped, failed = reprec.countoutcomes()
        assert failed == skipped == 0
        assert passed == 7

    def test_collect_only_with_various_situations(self, pytester: Pytester) -> None:
        p = pytester.makepyfile(
            test_one="""
                def test_one():
                    raise ValueError()

                class TestX(object):
                    def test_method_one(self):
                        pass

                class TestY(TestX):
                    pass
            """,
            test_three="xxxdsadsadsadsa",
            __init__="",
        )
        reprec = pytester.inline_run("--collect-only", p.parent)

        itemstarted = reprec.getcalls("pytest_itemcollected")
        assert len(itemstarted) == 3
        assert not reprec.getreports("pytest_runtest_logreport")
        started = reprec.getcalls("pytest_collectstart")
        finished = reprec.getreports("pytest_collectreport")
        assert len(started) == len(finished)
        assert len(started) == 6
        colfail = [x for x in finished if x.failed]
        assert len(colfail) == 1

    def test_minus_x_import_error(self, pytester: Pytester) -> None:
        pytester.makepyfile(__init__="")
        pytester.makepyfile(test_one="xxxx", test_two="yyyy")
        reprec = pytester.inline_run("-x", pytester.path)
        finished = reprec.getreports("pytest_collectreport")
        colfail = [x for x in finished if x.failed]
        assert len(colfail) == 1

    def test_minus_x_overridden_by_maxfail(self, pytester: Pytester) -> None:
        pytester.makepyfile(__init__="")
        pytester.makepyfile(test_one="xxxx", test_two="yyyy", test_third="zzz")
        reprec = pytester.inline_run("-x", "--maxfail=2", pytester.path)
        finished = reprec.getreports("pytest_collectreport")
        colfail = [x for x in finished if x.failed]
        assert len(colfail) == 2


def test_plugin_specify(pytester: Pytester) -> None:
    with pytest.raises(ImportError):
        pytester.parseconfig("-p", "nqweotexistent")
    # pytest.raises(ImportError,
    #    "config.do_configure(config)"
    # )


def test_plugin_already_exists(pytester: Pytester) -> None:
    config = pytester.parseconfig("-p", "terminal")
    assert config.option.plugins == ["terminal"]
    config._do_configure()
    config._ensure_unconfigure()


def test_exclude(pytester: Pytester) -> None:
    hellodir = pytester.mkdir("hello")
    hellodir.joinpath("test_hello.py").write_text("x y syntaxerror")
    hello2dir = pytester.mkdir("hello2")
    hello2dir.joinpath("test_hello2.py").write_text("x y syntaxerror")
    pytester.makepyfile(test_ok="def test_pass(): pass")
    result = pytester.runpytest("--ignore=hello", "--ignore=hello2")
    assert result.ret == 0
    result.stdout.fnmatch_lines(["*1 passed*"])


def test_exclude_glob(pytester: Pytester) -> None:
    hellodir = pytester.mkdir("hello")
    hellodir.joinpath("test_hello.py").write_text("x y syntaxerror")
    hello2dir = pytester.mkdir("hello2")
    hello2dir.joinpath("test_hello2.py").write_text("x y syntaxerror")
    hello3dir = pytester.mkdir("hallo3")
    hello3dir.joinpath("test_hello3.py").write_text("x y syntaxerror")
    subdir = pytester.mkdir("sub")
    subdir.joinpath("test_hello4.py").write_text("x y syntaxerror")
    pytester.makepyfile(test_ok="def test_pass(): pass")
    result = pytester.runpytest("--ignore-glob=*h[ea]llo*")
    assert result.ret == 0
    result.stdout.fnmatch_lines(["*1 passed*"])


def test_deselect(pytester: Pytester) -> None:
    pytester.makepyfile(
        test_a="""
        import pytest

        def test_a1(): pass

        @pytest.mark.parametrize('b', range(3))
        def test_a2(b): pass

        class TestClass:
            def test_c1(self): pass

            def test_c2(self): pass
    """
    )
    result = pytester.runpytest(
        "-v",
        "--deselect=test_a.py::test_a2[1]",
        "--deselect=test_a.py::test_a2[2]",
        "--deselect=test_a.py::TestClass::test_c1",
    )
    assert result.ret == 0
    result.stdout.fnmatch_lines(["*3 passed, 3 deselected*"])
    for line in result.stdout.lines:
        assert not line.startswith(("test_a.py::test_a2[1]", "test_a.py::test_a2[2]"))


def test_sessionfinish_with_start(pytester: Pytester) -> None:
    pytester.makeconftest(
        """
        import os
        values = []
        def pytest_sessionstart():
            values.append(os.getcwd())
            os.chdir("..")

        def pytest_sessionfinish():
            assert values[0] == os.getcwd()

    """
    )
    res = pytester.runpytest("--collect-only")
    assert res.ret == ExitCode.NO_TESTS_COLLECTED


@pytest.mark.parametrize("path", ["root", "{relative}/root", "{environment}/root"])
def test_rootdir_option_arg(
    pytester: Pytester, monkeypatch: MonkeyPatch, path: str
) -> None:
    monkeypatch.setenv("PY_ROOTDIR_PATH", str(pytester.path))
    path = path.format(relative=str(pytester.path), environment="$PY_ROOTDIR_PATH")

    rootdir = pytester.path / "root" / "tests"
    rootdir.mkdir(parents=True)
    pytester.makepyfile(
        """
        import os
        def test_one():
            assert 1
    """
    )

    result = pytester.runpytest(f"--rootdir={path}")
    result.stdout.fnmatch_lines(
        [
            f"*rootdir: {pytester.path}/root",
            "root/test_rootdir_option_arg.py *",
            "*1 passed*",
        ]
    )


def test_rootdir_wrong_option_arg(pytester: Pytester) -> None:
    result = pytester.runpytest("--rootdir=wrong_dir")
    result.stderr.fnmatch_lines(
        ["*Directory *wrong_dir* not found. Check your '--rootdir' option.*"]
    )
