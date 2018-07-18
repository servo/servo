from __future__ import absolute_import, division, print_function

import pytest

from _pytest.main import EXIT_NOTESTSCOLLECTED


class SessionTests(object):

    def test_basic_testitem_events(self, testdir):
        tfile = testdir.makepyfile(
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
        reprec = testdir.inline_run(tfile)
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

    def test_nested_import_error(self, testdir):
        tfile = testdir.makepyfile(
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
        reprec = testdir.inline_run(tfile)
        values = reprec.getfailedcollections()
        assert len(values) == 1
        out = str(values[0].longrepr)
        assert out.find("does_not_work") != -1

    def test_raises_output(self, testdir):
        reprec = testdir.inline_runsource(
            """
            import pytest
            def test_raises_doesnt():
                pytest.raises(ValueError, int, "3")
        """
        )
        passed, skipped, failed = reprec.listoutcomes()
        assert len(failed) == 1
        out = failed[0].longrepr.reprcrash.message
        if not out.find("DID NOT RAISE") != -1:
            print(out)
            pytest.fail("incorrect raises() output")

    def test_generator_yields_None(self, testdir):
        reprec = testdir.inline_runsource(
            """
            def test_1():
                yield None
        """
        )
        failures = reprec.getfailedcollections()
        out = failures[0].longrepr.reprcrash.message
        i = out.find("TypeError")
        assert i != -1

    def test_syntax_error_module(self, testdir):
        reprec = testdir.inline_runsource("this is really not python")
        values = reprec.getfailedcollections()
        assert len(values) == 1
        out = str(values[0].longrepr)
        assert out.find(str("not python")) != -1

    def test_exit_first_problem(self, testdir):
        reprec = testdir.inline_runsource(
            """
            def test_one(): assert 0
            def test_two(): assert 0
        """,
            "--exitfirst",
        )
        passed, skipped, failed = reprec.countoutcomes()
        assert failed == 1
        assert passed == skipped == 0

    def test_maxfail(self, testdir):
        reprec = testdir.inline_runsource(
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

    def test_broken_repr(self, testdir):
        p = testdir.makepyfile(
            """
            import pytest
            class BrokenRepr1(object):
                foo=0
                def __repr__(self):
                    raise Exception("Ha Ha fooled you, I'm a broken repr().")

            class TestBrokenClass(object):
                def test_explicit_bad_repr(self):
                    t = BrokenRepr1()
                    pytest.raises(Exception, 'repr(t)')

                def test_implicit_bad_repr1(self):
                    t = BrokenRepr1()
                    assert t.foo == 1

        """
        )
        reprec = testdir.inline_run(p)
        passed, skipped, failed = reprec.listoutcomes()
        assert len(failed) == 1
        out = failed[0].longrepr.reprcrash.message
        assert (
            out.find(
                """[Exception("Ha Ha fooled you, I'm a broken repr().") raised in repr()]"""
            )
            != -1
        )  # '

    def test_skip_file_by_conftest(self, testdir):
        testdir.makepyfile(
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
            reprec = testdir.inline_run(testdir.tmpdir)
        except pytest.skip.Exception:
            pytest.fail("wrong skipped caught")
        reports = reprec.getreports("pytest_collectreport")
        assert len(reports) == 1
        assert reports[0].skipped


class TestNewSession(SessionTests):

    def test_order_of_execution(self, testdir):
        reprec = testdir.inline_runsource(
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
        # also test listnames() here ...

    def test_collect_only_with_various_situations(self, testdir):
        p = testdir.makepyfile(
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
        reprec = testdir.inline_run("--collect-only", p.dirpath())

        itemstarted = reprec.getcalls("pytest_itemcollected")
        assert len(itemstarted) == 3
        assert not reprec.getreports("pytest_runtest_logreport")
        started = reprec.getcalls("pytest_collectstart")
        finished = reprec.getreports("pytest_collectreport")
        assert len(started) == len(finished)
        assert len(started) == 7  # XXX extra TopCollector
        colfail = [x for x in finished if x.failed]
        assert len(colfail) == 1

    def test_minus_x_import_error(self, testdir):
        testdir.makepyfile(__init__="")
        testdir.makepyfile(test_one="xxxx", test_two="yyyy")
        reprec = testdir.inline_run("-x", testdir.tmpdir)
        finished = reprec.getreports("pytest_collectreport")
        colfail = [x for x in finished if x.failed]
        assert len(colfail) == 1

    def test_minus_x_overridden_by_maxfail(self, testdir):
        testdir.makepyfile(__init__="")
        testdir.makepyfile(test_one="xxxx", test_two="yyyy", test_third="zzz")
        reprec = testdir.inline_run("-x", "--maxfail=2", testdir.tmpdir)
        finished = reprec.getreports("pytest_collectreport")
        colfail = [x for x in finished if x.failed]
        assert len(colfail) == 2


def test_plugin_specify(testdir):
    pytest.raises(
        ImportError,
        """
            testdir.parseconfig("-p", "nqweotexistent")
    """,
    )
    # pytest.raises(ImportError,
    #    "config.do_configure(config)"
    # )


def test_plugin_already_exists(testdir):
    config = testdir.parseconfig("-p", "terminal")
    assert config.option.plugins == ["terminal"]
    config._do_configure()
    config._ensure_unconfigure()


def test_exclude(testdir):
    hellodir = testdir.mkdir("hello")
    hellodir.join("test_hello.py").write("x y syntaxerror")
    hello2dir = testdir.mkdir("hello2")
    hello2dir.join("test_hello2.py").write("x y syntaxerror")
    testdir.makepyfile(test_ok="def test_pass(): pass")
    result = testdir.runpytest("--ignore=hello", "--ignore=hello2")
    assert result.ret == 0
    result.stdout.fnmatch_lines(["*1 passed*"])


def test_deselect(testdir):
    testdir.makepyfile(
        test_a="""
        import pytest
        def test_a1(): pass
        @pytest.mark.parametrize('b', range(3))
        def test_a2(b): pass
    """
    )
    result = testdir.runpytest(
        "-v", "--deselect=test_a.py::test_a2[1]", "--deselect=test_a.py::test_a2[2]"
    )
    assert result.ret == 0
    result.stdout.fnmatch_lines(["*2 passed, 2 deselected*"])
    for line in result.stdout.lines:
        assert not line.startswith(("test_a.py::test_a2[1]", "test_a.py::test_a2[2]"))


def test_sessionfinish_with_start(testdir):
    testdir.makeconftest(
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
    res = testdir.runpytest("--collect-only")
    assert res.ret == EXIT_NOTESTSCOLLECTED


@pytest.mark.parametrize("path", ["root", "{relative}/root", "{environment}/root"])
def test_rootdir_option_arg(testdir, monkeypatch, path):
    monkeypatch.setenv("PY_ROOTDIR_PATH", str(testdir.tmpdir))
    path = path.format(relative=str(testdir.tmpdir), environment="$PY_ROOTDIR_PATH")

    rootdir = testdir.mkdir("root")
    rootdir.mkdir("tests")
    testdir.makepyfile(
        """
        import os
        def test_one():
            assert 1
    """
    )

    result = testdir.runpytest("--rootdir={}".format(path))
    result.stdout.fnmatch_lines(
        ["*rootdir: {}/root, inifile:*".format(testdir.tmpdir), "*1 passed*"]
    )


def test_rootdir_wrong_option_arg(testdir):
    testdir.makepyfile(
        """
        import os
        def test_one():
            assert 1
    """
    )

    result = testdir.runpytest("--rootdir=wrong_dir")
    result.stderr.fnmatch_lines(
        ["*Directory *wrong_dir* not found. Check your '--rootdir' option.*"]
    )
