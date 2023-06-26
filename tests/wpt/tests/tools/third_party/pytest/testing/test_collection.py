import os
import pprint
import shutil
import sys
import textwrap
from pathlib import Path
from typing import List

import pytest
from _pytest.config import ExitCode
from _pytest.fixtures import FixtureRequest
from _pytest.main import _in_venv
from _pytest.main import Session
from _pytest.monkeypatch import MonkeyPatch
from _pytest.nodes import Item
from _pytest.pathlib import symlink_or_skip
from _pytest.pytester import HookRecorder
from _pytest.pytester import Pytester


def ensure_file(file_path: Path) -> Path:
    """Ensure that file exists"""
    file_path.parent.mkdir(parents=True, exist_ok=True)
    file_path.touch(exist_ok=True)
    return file_path


class TestCollector:
    def test_collect_versus_item(self) -> None:
        from pytest import Collector
        from pytest import Item

        assert not issubclass(Collector, Item)
        assert not issubclass(Item, Collector)

    def test_check_equality(self, pytester: Pytester) -> None:
        modcol = pytester.getmodulecol(
            """
            def test_pass(): pass
            def test_fail(): assert 0
        """
        )
        fn1 = pytester.collect_by_name(modcol, "test_pass")
        assert isinstance(fn1, pytest.Function)
        fn2 = pytester.collect_by_name(modcol, "test_pass")
        assert isinstance(fn2, pytest.Function)

        assert fn1 == fn2
        assert fn1 != modcol
        assert hash(fn1) == hash(fn2)

        fn3 = pytester.collect_by_name(modcol, "test_fail")
        assert isinstance(fn3, pytest.Function)
        assert not (fn1 == fn3)
        assert fn1 != fn3

        for fn in fn1, fn2, fn3:
            assert isinstance(fn, pytest.Function)
            assert fn != 3  # type: ignore[comparison-overlap]
            assert fn != modcol
            assert fn != [1, 2, 3]  # type: ignore[comparison-overlap]
            assert [1, 2, 3] != fn  # type: ignore[comparison-overlap]
            assert modcol != fn

        assert pytester.collect_by_name(modcol, "doesnotexist") is None

    def test_getparent_and_accessors(self, pytester: Pytester) -> None:
        modcol = pytester.getmodulecol(
            """
            class TestClass:
                 def test_foo(self):
                     pass
        """
        )
        cls = pytester.collect_by_name(modcol, "TestClass")
        assert isinstance(cls, pytest.Class)
        fn = pytester.collect_by_name(cls, "test_foo")
        assert isinstance(fn, pytest.Function)

        assert fn.getparent(pytest.Module) is modcol
        assert modcol.module is not None
        assert modcol.cls is None
        assert modcol.instance is None

        assert fn.getparent(pytest.Class) is cls
        assert cls.module is not None
        assert cls.cls is not None
        assert cls.instance is None

        assert fn.getparent(pytest.Function) is fn
        assert fn.module is not None
        assert fn.cls is not None
        assert fn.instance is not None
        assert fn.function is not None

    def test_getcustomfile_roundtrip(self, pytester: Pytester) -> None:
        hello = pytester.makefile(".xxx", hello="world")
        pytester.makepyfile(
            conftest="""
            import pytest
            class CustomFile(pytest.File):
                pass
            def pytest_collect_file(file_path, parent):
                if file_path.suffix == ".xxx":
                    return CustomFile.from_parent(path=file_path, parent=parent)
        """
        )
        node = pytester.getpathnode(hello)
        assert isinstance(node, pytest.File)
        assert node.name == "hello.xxx"
        nodes = node.session.perform_collect([node.nodeid], genitems=False)
        assert len(nodes) == 1
        assert isinstance(nodes[0], pytest.File)

    def test_can_skip_class_with_test_attr(self, pytester: Pytester) -> None:
        """Assure test class is skipped when using `__test__=False` (See #2007)."""
        pytester.makepyfile(
            """
            class TestFoo(object):
                __test__ = False
                def __init__(self):
                    pass
                def test_foo():
                    assert True
        """
        )
        result = pytester.runpytest()
        result.stdout.fnmatch_lines(["collected 0 items", "*no tests ran in*"])


class TestCollectFS:
    def test_ignored_certain_directories(self, pytester: Pytester) -> None:
        tmp_path = pytester.path
        ensure_file(tmp_path / "build" / "test_notfound.py")
        ensure_file(tmp_path / "dist" / "test_notfound.py")
        ensure_file(tmp_path / "_darcs" / "test_notfound.py")
        ensure_file(tmp_path / "CVS" / "test_notfound.py")
        ensure_file(tmp_path / "{arch}" / "test_notfound.py")
        ensure_file(tmp_path / ".whatever" / "test_notfound.py")
        ensure_file(tmp_path / ".bzr" / "test_notfound.py")
        ensure_file(tmp_path / "normal" / "test_found.py")
        for x in tmp_path.rglob("test_*.py"):
            x.write_text("def test_hello(): pass", "utf-8")

        result = pytester.runpytest("--collect-only")
        s = result.stdout.str()
        assert "test_notfound" not in s
        assert "test_found" in s

    @pytest.mark.parametrize(
        "fname",
        (
            "activate",
            "activate.csh",
            "activate.fish",
            "Activate",
            "Activate.bat",
            "Activate.ps1",
        ),
    )
    def test_ignored_virtualenvs(self, pytester: Pytester, fname: str) -> None:
        bindir = "Scripts" if sys.platform.startswith("win") else "bin"
        ensure_file(pytester.path / "virtual" / bindir / fname)
        testfile = ensure_file(pytester.path / "virtual" / "test_invenv.py")
        testfile.write_text("def test_hello(): pass")

        # by default, ignore tests inside a virtualenv
        result = pytester.runpytest()
        result.stdout.no_fnmatch_line("*test_invenv*")
        # allow test collection if user insists
        result = pytester.runpytest("--collect-in-virtualenv")
        assert "test_invenv" in result.stdout.str()
        # allow test collection if user directly passes in the directory
        result = pytester.runpytest("virtual")
        assert "test_invenv" in result.stdout.str()

    @pytest.mark.parametrize(
        "fname",
        (
            "activate",
            "activate.csh",
            "activate.fish",
            "Activate",
            "Activate.bat",
            "Activate.ps1",
        ),
    )
    def test_ignored_virtualenvs_norecursedirs_precedence(
        self, pytester: Pytester, fname: str
    ) -> None:
        bindir = "Scripts" if sys.platform.startswith("win") else "bin"
        # norecursedirs takes priority
        ensure_file(pytester.path / ".virtual" / bindir / fname)
        testfile = ensure_file(pytester.path / ".virtual" / "test_invenv.py")
        testfile.write_text("def test_hello(): pass")
        result = pytester.runpytest("--collect-in-virtualenv")
        result.stdout.no_fnmatch_line("*test_invenv*")
        # ...unless the virtualenv is explicitly given on the CLI
        result = pytester.runpytest("--collect-in-virtualenv", ".virtual")
        assert "test_invenv" in result.stdout.str()

    @pytest.mark.parametrize(
        "fname",
        (
            "activate",
            "activate.csh",
            "activate.fish",
            "Activate",
            "Activate.bat",
            "Activate.ps1",
        ),
    )
    def test__in_venv(self, pytester: Pytester, fname: str) -> None:
        """Directly test the virtual env detection function"""
        bindir = "Scripts" if sys.platform.startswith("win") else "bin"
        # no bin/activate, not a virtualenv
        base_path = pytester.mkdir("venv")
        assert _in_venv(base_path) is False
        # with bin/activate, totally a virtualenv
        bin_path = base_path.joinpath(bindir)
        bin_path.mkdir()
        bin_path.joinpath(fname).touch()
        assert _in_venv(base_path) is True

    def test_custom_norecursedirs(self, pytester: Pytester) -> None:
        pytester.makeini(
            """
            [pytest]
            norecursedirs = mydir xyz*
        """
        )
        tmp_path = pytester.path
        ensure_file(tmp_path / "mydir" / "test_hello.py").write_text(
            "def test_1(): pass"
        )
        ensure_file(tmp_path / "xyz123" / "test_2.py").write_text("def test_2(): 0/0")
        ensure_file(tmp_path / "xy" / "test_ok.py").write_text("def test_3(): pass")
        rec = pytester.inline_run()
        rec.assertoutcome(passed=1)
        rec = pytester.inline_run("xyz123/test_2.py")
        rec.assertoutcome(failed=1)

    def test_testpaths_ini(self, pytester: Pytester, monkeypatch: MonkeyPatch) -> None:
        pytester.makeini(
            """
            [pytest]
            testpaths = gui uts
        """
        )
        tmp_path = pytester.path
        ensure_file(tmp_path / "env" / "test_1.py").write_text("def test_env(): pass")
        ensure_file(tmp_path / "gui" / "test_2.py").write_text("def test_gui(): pass")
        ensure_file(tmp_path / "uts" / "test_3.py").write_text("def test_uts(): pass")

        # executing from rootdir only tests from `testpaths` directories
        # are collected
        items, reprec = pytester.inline_genitems("-v")
        assert [x.name for x in items] == ["test_gui", "test_uts"]

        # check that explicitly passing directories in the command-line
        # collects the tests
        for dirname in ("env", "gui", "uts"):
            items, reprec = pytester.inline_genitems(tmp_path.joinpath(dirname))
            assert [x.name for x in items] == ["test_%s" % dirname]

        # changing cwd to each subdirectory and running pytest without
        # arguments collects the tests in that directory normally
        for dirname in ("env", "gui", "uts"):
            monkeypatch.chdir(pytester.path.joinpath(dirname))
            items, reprec = pytester.inline_genitems()
            assert [x.name for x in items] == ["test_%s" % dirname]


class TestCollectPluginHookRelay:
    def test_pytest_collect_file(self, pytester: Pytester) -> None:
        wascalled = []

        class Plugin:
            def pytest_collect_file(self, file_path: Path) -> None:
                if not file_path.name.startswith("."):
                    # Ignore hidden files, e.g. .testmondata.
                    wascalled.append(file_path)

        pytester.makefile(".abc", "xyz")
        pytest.main(pytester.path, plugins=[Plugin()])
        assert len(wascalled) == 1
        assert wascalled[0].suffix == ".abc"


class TestPrunetraceback:
    def test_custom_repr_failure(self, pytester: Pytester) -> None:
        p = pytester.makepyfile(
            """
            import not_exists
        """
        )
        pytester.makeconftest(
            """
            import pytest
            def pytest_collect_file(file_path, parent):
                return MyFile.from_parent(path=file_path, parent=parent)
            class MyError(Exception):
                pass
            class MyFile(pytest.File):
                def collect(self):
                    raise MyError()
                def repr_failure(self, excinfo):
                    if isinstance(excinfo.value, MyError):
                        return "hello world"
                    return pytest.File.repr_failure(self, excinfo)
        """
        )

        result = pytester.runpytest(p)
        result.stdout.fnmatch_lines(["*ERROR collecting*", "*hello world*"])

    @pytest.mark.xfail(reason="other mechanism for adding to reporting needed")
    def test_collect_report_postprocessing(self, pytester: Pytester) -> None:
        p = pytester.makepyfile(
            """
            import not_exists
        """
        )
        pytester.makeconftest(
            """
            import pytest
            @pytest.hookimpl(hookwrapper=True)
            def pytest_make_collect_report():
                outcome = yield
                rep = outcome.get_result()
                rep.headerlines += ["header1"]
                outcome.force_result(rep)
        """
        )
        result = pytester.runpytest(p)
        result.stdout.fnmatch_lines(["*ERROR collecting*", "*header1*"])


class TestCustomConftests:
    def test_ignore_collect_path(self, pytester: Pytester) -> None:
        pytester.makeconftest(
            """
            def pytest_ignore_collect(collection_path, config):
                return collection_path.name.startswith("x") or collection_path.name == "test_one.py"
        """
        )
        sub = pytester.mkdir("xy123")
        ensure_file(sub / "test_hello.py").write_text("syntax error")
        sub.joinpath("conftest.py").write_text("syntax error")
        pytester.makepyfile("def test_hello(): pass")
        pytester.makepyfile(test_one="syntax error")
        result = pytester.runpytest("--fulltrace")
        assert result.ret == 0
        result.stdout.fnmatch_lines(["*1 passed*"])

    def test_ignore_collect_not_called_on_argument(self, pytester: Pytester) -> None:
        pytester.makeconftest(
            """
            def pytest_ignore_collect(collection_path, config):
                return True
        """
        )
        p = pytester.makepyfile("def test_hello(): pass")
        result = pytester.runpytest(p)
        assert result.ret == 0
        result.stdout.fnmatch_lines(["*1 passed*"])
        result = pytester.runpytest()
        assert result.ret == ExitCode.NO_TESTS_COLLECTED
        result.stdout.fnmatch_lines(["*collected 0 items*"])

    def test_collectignore_exclude_on_option(self, pytester: Pytester) -> None:
        pytester.makeconftest(
            """
            from pathlib import Path

            class MyPathLike:
                def __init__(self, path):
                    self.path = path
                def __fspath__(self):
                    return "path"

            collect_ignore = [MyPathLike('hello'), 'test_world.py', Path('bye')]

            def pytest_addoption(parser):
                parser.addoption("--XX", action="store_true", default=False)

            def pytest_configure(config):
                if config.getvalue("XX"):
                    collect_ignore[:] = []
        """
        )
        pytester.mkdir("hello")
        pytester.makepyfile(test_world="def test_hello(): pass")
        result = pytester.runpytest()
        assert result.ret == ExitCode.NO_TESTS_COLLECTED
        result.stdout.no_fnmatch_line("*passed*")
        result = pytester.runpytest("--XX")
        assert result.ret == 0
        assert "passed" in result.stdout.str()

    def test_collectignoreglob_exclude_on_option(self, pytester: Pytester) -> None:
        pytester.makeconftest(
            """
            collect_ignore_glob = ['*w*l[dt]*']
            def pytest_addoption(parser):
                parser.addoption("--XX", action="store_true", default=False)
            def pytest_configure(config):
                if config.getvalue("XX"):
                    collect_ignore_glob[:] = []
        """
        )
        pytester.makepyfile(test_world="def test_hello(): pass")
        pytester.makepyfile(test_welt="def test_hallo(): pass")
        result = pytester.runpytest()
        assert result.ret == ExitCode.NO_TESTS_COLLECTED
        result.stdout.fnmatch_lines(["*collected 0 items*"])
        result = pytester.runpytest("--XX")
        assert result.ret == 0
        result.stdout.fnmatch_lines(["*2 passed*"])

    def test_pytest_fs_collect_hooks_are_seen(self, pytester: Pytester) -> None:
        pytester.makeconftest(
            """
            import pytest
            class MyModule(pytest.Module):
                pass
            def pytest_collect_file(file_path, parent):
                if file_path.suffix == ".py":
                    return MyModule.from_parent(path=file_path, parent=parent)
        """
        )
        pytester.mkdir("sub")
        pytester.makepyfile("def test_x(): pass")
        result = pytester.runpytest("--co")
        result.stdout.fnmatch_lines(["*MyModule*", "*test_x*"])

    def test_pytest_collect_file_from_sister_dir(self, pytester: Pytester) -> None:
        sub1 = pytester.mkpydir("sub1")
        sub2 = pytester.mkpydir("sub2")
        conf1 = pytester.makeconftest(
            """
            import pytest
            class MyModule1(pytest.Module):
                pass
            def pytest_collect_file(file_path, parent):
                if file_path.suffix == ".py":
                    return MyModule1.from_parent(path=file_path, parent=parent)
        """
        )
        conf1.replace(sub1.joinpath(conf1.name))
        conf2 = pytester.makeconftest(
            """
            import pytest
            class MyModule2(pytest.Module):
                pass
            def pytest_collect_file(file_path, parent):
                if file_path.suffix == ".py":
                    return MyModule2.from_parent(path=file_path, parent=parent)
        """
        )
        conf2.replace(sub2.joinpath(conf2.name))
        p = pytester.makepyfile("def test_x(): pass")
        shutil.copy(p, sub1.joinpath(p.name))
        shutil.copy(p, sub2.joinpath(p.name))
        result = pytester.runpytest("--co")
        result.stdout.fnmatch_lines(["*MyModule1*", "*MyModule2*", "*test_x*"])


class TestSession:
    def test_collect_topdir(self, pytester: Pytester) -> None:
        p = pytester.makepyfile("def test_func(): pass")
        id = "::".join([p.name, "test_func"])
        # XXX migrate to collectonly? (see below)
        config = pytester.parseconfig(id)
        topdir = pytester.path
        rcol = Session.from_config(config)
        assert topdir == rcol.path
        # rootid = rcol.nodeid
        # root2 = rcol.perform_collect([rcol.nodeid], genitems=False)[0]
        # assert root2 == rcol, rootid
        colitems = rcol.perform_collect([rcol.nodeid], genitems=False)
        assert len(colitems) == 1
        assert colitems[0].path == p

    def get_reported_items(self, hookrec: HookRecorder) -> List[Item]:
        """Return pytest.Item instances reported by the pytest_collectreport hook"""
        calls = hookrec.getcalls("pytest_collectreport")
        return [
            x
            for call in calls
            for x in call.report.result
            if isinstance(x, pytest.Item)
        ]

    def test_collect_protocol_single_function(self, pytester: Pytester) -> None:
        p = pytester.makepyfile("def test_func(): pass")
        id = "::".join([p.name, "test_func"])
        items, hookrec = pytester.inline_genitems(id)
        (item,) = items
        assert item.name == "test_func"
        newid = item.nodeid
        assert newid == id
        pprint.pprint(hookrec.calls)
        topdir = pytester.path  # noqa
        hookrec.assert_contains(
            [
                ("pytest_collectstart", "collector.path == topdir"),
                ("pytest_make_collect_report", "collector.path == topdir"),
                ("pytest_collectstart", "collector.path == p"),
                ("pytest_make_collect_report", "collector.path == p"),
                ("pytest_pycollect_makeitem", "name == 'test_func'"),
                ("pytest_collectreport", "report.result[0].name == 'test_func'"),
            ]
        )
        # ensure we are reporting the collection of the single test item (#2464)
        assert [x.name for x in self.get_reported_items(hookrec)] == ["test_func"]

    def test_collect_protocol_method(self, pytester: Pytester) -> None:
        p = pytester.makepyfile(
            """
            class TestClass(object):
                def test_method(self):
                    pass
        """
        )
        normid = p.name + "::TestClass::test_method"
        for id in [p.name, p.name + "::TestClass", normid]:
            items, hookrec = pytester.inline_genitems(id)
            assert len(items) == 1
            assert items[0].name == "test_method"
            newid = items[0].nodeid
            assert newid == normid
            # ensure we are reporting the collection of the single test item (#2464)
            assert [x.name for x in self.get_reported_items(hookrec)] == ["test_method"]

    def test_collect_custom_nodes_multi_id(self, pytester: Pytester) -> None:
        p = pytester.makepyfile("def test_func(): pass")
        pytester.makeconftest(
            """
            import pytest
            class SpecialItem(pytest.Item):
                def runtest(self):
                    return # ok
            class SpecialFile(pytest.File):
                def collect(self):
                    return [SpecialItem.from_parent(name="check", parent=self)]
            def pytest_collect_file(file_path, parent):
                if file_path.name == %r:
                    return SpecialFile.from_parent(path=file_path, parent=parent)
        """
            % p.name
        )
        id = p.name

        items, hookrec = pytester.inline_genitems(id)
        pprint.pprint(hookrec.calls)
        assert len(items) == 2
        hookrec.assert_contains(
            [
                ("pytest_collectstart", "collector.path == collector.session.path"),
                (
                    "pytest_collectstart",
                    "collector.__class__.__name__ == 'SpecialFile'",
                ),
                ("pytest_collectstart", "collector.__class__.__name__ == 'Module'"),
                ("pytest_pycollect_makeitem", "name == 'test_func'"),
                ("pytest_collectreport", "report.nodeid.startswith(p.name)"),
            ]
        )
        assert len(self.get_reported_items(hookrec)) == 2

    def test_collect_subdir_event_ordering(self, pytester: Pytester) -> None:
        p = pytester.makepyfile("def test_func(): pass")
        aaa = pytester.mkpydir("aaa")
        test_aaa = aaa.joinpath("test_aaa.py")
        p.replace(test_aaa)

        items, hookrec = pytester.inline_genitems()
        assert len(items) == 1
        pprint.pprint(hookrec.calls)
        hookrec.assert_contains(
            [
                ("pytest_collectstart", "collector.path == test_aaa"),
                ("pytest_pycollect_makeitem", "name == 'test_func'"),
                ("pytest_collectreport", "report.nodeid.startswith('aaa/test_aaa.py')"),
            ]
        )

    def test_collect_two_commandline_args(self, pytester: Pytester) -> None:
        p = pytester.makepyfile("def test_func(): pass")
        aaa = pytester.mkpydir("aaa")
        bbb = pytester.mkpydir("bbb")
        test_aaa = aaa.joinpath("test_aaa.py")
        shutil.copy(p, test_aaa)
        test_bbb = bbb.joinpath("test_bbb.py")
        p.replace(test_bbb)

        id = "."

        items, hookrec = pytester.inline_genitems(id)
        assert len(items) == 2
        pprint.pprint(hookrec.calls)
        hookrec.assert_contains(
            [
                ("pytest_collectstart", "collector.path == test_aaa"),
                ("pytest_pycollect_makeitem", "name == 'test_func'"),
                ("pytest_collectreport", "report.nodeid == 'aaa/test_aaa.py'"),
                ("pytest_collectstart", "collector.path == test_bbb"),
                ("pytest_pycollect_makeitem", "name == 'test_func'"),
                ("pytest_collectreport", "report.nodeid == 'bbb/test_bbb.py'"),
            ]
        )

    def test_serialization_byid(self, pytester: Pytester) -> None:
        pytester.makepyfile("def test_func(): pass")
        items, hookrec = pytester.inline_genitems()
        assert len(items) == 1
        (item,) = items
        items2, hookrec = pytester.inline_genitems(item.nodeid)
        (item2,) = items2
        assert item2.name == item.name
        assert item2.path == item.path

    def test_find_byid_without_instance_parents(self, pytester: Pytester) -> None:
        p = pytester.makepyfile(
            """
            class TestClass(object):
                def test_method(self):
                    pass
        """
        )
        arg = p.name + "::TestClass::test_method"
        items, hookrec = pytester.inline_genitems(arg)
        assert len(items) == 1
        (item,) = items
        assert item.nodeid.endswith("TestClass::test_method")
        # ensure we are reporting the collection of the single test item (#2464)
        assert [x.name for x in self.get_reported_items(hookrec)] == ["test_method"]


class Test_getinitialnodes:
    def test_global_file(self, pytester: Pytester) -> None:
        tmp_path = pytester.path
        x = ensure_file(tmp_path / "x.py")
        config = pytester.parseconfigure(x)
        col = pytester.getnode(config, x)
        assert isinstance(col, pytest.Module)
        assert col.name == "x.py"
        assert col.parent is not None
        assert col.parent.parent is None
        for parent in col.listchain():
            assert parent.config is config

    def test_pkgfile(self, pytester: Pytester) -> None:
        """Verify nesting when a module is within a package.
        The parent chain should match: Module<x.py> -> Package<subdir> -> Session.
            Session's parent should always be None.
        """
        tmp_path = pytester.path
        subdir = tmp_path.joinpath("subdir")
        x = ensure_file(subdir / "x.py")
        ensure_file(subdir / "__init__.py")
        with subdir.cwd():
            config = pytester.parseconfigure(x)
        col = pytester.getnode(config, x)
        assert col is not None
        assert col.name == "x.py"
        assert isinstance(col, pytest.Module)
        assert isinstance(col.parent, pytest.Package)
        assert isinstance(col.parent.parent, pytest.Session)
        # session is batman (has no parents)
        assert col.parent.parent.parent is None
        for parent in col.listchain():
            assert parent.config is config


class Test_genitems:
    def test_check_collect_hashes(self, pytester: Pytester) -> None:
        p = pytester.makepyfile(
            """
            def test_1():
                pass

            def test_2():
                pass
        """
        )
        shutil.copy(p, p.parent / (p.stem + "2" + ".py"))
        items, reprec = pytester.inline_genitems(p.parent)
        assert len(items) == 4
        for numi, i in enumerate(items):
            for numj, j in enumerate(items):
                if numj != numi:
                    assert hash(i) != hash(j)
                    assert i != j

    def test_example_items1(self, pytester: Pytester) -> None:
        p = pytester.makepyfile(
            """
            import pytest

            def testone():
                pass

            class TestX(object):
                def testmethod_one(self):
                    pass

            class TestY(TestX):
                @pytest.mark.parametrize("arg0", [".["])
                def testmethod_two(self, arg0):
                    pass
        """
        )
        items, reprec = pytester.inline_genitems(p)
        assert len(items) == 4
        assert items[0].name == "testone"
        assert items[1].name == "testmethod_one"
        assert items[2].name == "testmethod_one"
        assert items[3].name == "testmethod_two[.[]"

        # let's also test getmodpath here
        assert items[0].getmodpath() == "testone"  # type: ignore[attr-defined]
        assert items[1].getmodpath() == "TestX.testmethod_one"  # type: ignore[attr-defined]
        assert items[2].getmodpath() == "TestY.testmethod_one"  # type: ignore[attr-defined]
        # PR #6202: Fix incorrect result of getmodpath method. (Resolves issue #6189)
        assert items[3].getmodpath() == "TestY.testmethod_two[.[]"  # type: ignore[attr-defined]

        s = items[0].getmodpath(stopatmodule=False)  # type: ignore[attr-defined]
        assert s.endswith("test_example_items1.testone")
        print(s)

    def test_class_and_functions_discovery_using_glob(self, pytester: Pytester) -> None:
        """Test that Python_classes and Python_functions config options work
        as prefixes and glob-like patterns (#600)."""
        pytester.makeini(
            """
            [pytest]
            python_classes = *Suite Test
            python_functions = *_test test
        """
        )
        p = pytester.makepyfile(
            """
            class MyTestSuite(object):
                def x_test(self):
                    pass

            class TestCase(object):
                def test_y(self):
                    pass
        """
        )
        items, reprec = pytester.inline_genitems(p)
        ids = [x.getmodpath() for x in items]  # type: ignore[attr-defined]
        assert ids == ["MyTestSuite.x_test", "TestCase.test_y"]


def test_matchnodes_two_collections_same_file(pytester: Pytester) -> None:
    pytester.makeconftest(
        """
        import pytest
        def pytest_configure(config):
            config.pluginmanager.register(Plugin2())

        class Plugin2(object):
            def pytest_collect_file(self, file_path, parent):
                if file_path.suffix == ".abc":
                    return MyFile2.from_parent(path=file_path, parent=parent)

        def pytest_collect_file(file_path, parent):
            if file_path.suffix == ".abc":
                return MyFile1.from_parent(path=file_path, parent=parent)

        class MyFile1(pytest.File):
            def collect(self):
                yield Item1.from_parent(name="item1", parent=self)

        class MyFile2(pytest.File):
            def collect(self):
                yield Item2.from_parent(name="item2", parent=self)

        class Item1(pytest.Item):
            def runtest(self):
                pass

        class Item2(pytest.Item):
            def runtest(self):
                pass
    """
    )
    p = pytester.makefile(".abc", "")
    result = pytester.runpytest()
    assert result.ret == 0
    result.stdout.fnmatch_lines(["*2 passed*"])
    res = pytester.runpytest("%s::item2" % p.name)
    res.stdout.fnmatch_lines(["*1 passed*"])


class TestNodeKeywords:
    def test_no_under(self, pytester: Pytester) -> None:
        modcol = pytester.getmodulecol(
            """
            def test_pass(): pass
            def test_fail(): assert 0
        """
        )
        values = list(modcol.keywords)
        assert modcol.name in values
        for x in values:
            assert not x.startswith("_")
        assert modcol.name in repr(modcol.keywords)

    def test_issue345(self, pytester: Pytester) -> None:
        pytester.makepyfile(
            """
            def test_should_not_be_selected():
                assert False, 'I should not have been selected to run'

            def test___repr__():
                pass
        """
        )
        reprec = pytester.inline_run("-k repr")
        reprec.assertoutcome(passed=1, failed=0)

    def test_keyword_matching_is_case_insensitive_by_default(
        self, pytester: Pytester
    ) -> None:
        """Check that selection via -k EXPRESSION is case-insensitive.

        Since markers are also added to the node keywords, they too can
        be matched without having to think about case sensitivity.

        """
        pytester.makepyfile(
            """
            import pytest

            def test_sPeCiFiCToPiC_1():
                assert True

            class TestSpecificTopic_2:
                def test(self):
                    assert True

            @pytest.mark.sPeCiFiCToPic_3
            def test():
                assert True

            @pytest.mark.sPeCiFiCToPic_4
            class Test:
                def test(self):
                    assert True

            def test_failing_5():
                assert False, "This should not match"

        """
        )
        num_matching_tests = 4
        for expression in ("specifictopic", "SPECIFICTOPIC", "SpecificTopic"):
            reprec = pytester.inline_run("-k " + expression)
            reprec.assertoutcome(passed=num_matching_tests, failed=0)

    def test_duplicates_handled_correctly(self, pytester: Pytester) -> None:
        item = pytester.getitem(
            """
            import pytest
            pytestmark = pytest.mark.kw
            class TestClass:
                pytestmark = pytest.mark.kw
                def test_method(self): pass
                test_method.kw = 'method'
        """,
            "test_method",
        )
        assert item.parent is not None and item.parent.parent is not None
        item.parent.parent.keywords["kw"] = "class"

        assert item.keywords["kw"] == "method"
        assert len(item.keywords) == len(set(item.keywords))


COLLECTION_ERROR_PY_FILES = dict(
    test_01_failure="""
        def test_1():
            assert False
        """,
    test_02_import_error="""
        import asdfasdfasdf
        def test_2():
            assert True
        """,
    test_03_import_error="""
        import asdfasdfasdf
        def test_3():
            assert True
    """,
    test_04_success="""
        def test_4():
            assert True
    """,
)


def test_exit_on_collection_error(pytester: Pytester) -> None:
    """Verify that all collection errors are collected and no tests executed"""
    pytester.makepyfile(**COLLECTION_ERROR_PY_FILES)

    res = pytester.runpytest()
    assert res.ret == 2

    res.stdout.fnmatch_lines(
        [
            "collected 2 items / 2 errors",
            "*ERROR collecting test_02_import_error.py*",
            "*No module named *asdfa*",
            "*ERROR collecting test_03_import_error.py*",
            "*No module named *asdfa*",
        ]
    )


def test_exit_on_collection_with_maxfail_smaller_than_n_errors(
    pytester: Pytester,
) -> None:
    """
    Verify collection is aborted once maxfail errors are encountered ignoring
    further modules which would cause more collection errors.
    """
    pytester.makepyfile(**COLLECTION_ERROR_PY_FILES)

    res = pytester.runpytest("--maxfail=1")
    assert res.ret == 1
    res.stdout.fnmatch_lines(
        [
            "collected 1 item / 1 error",
            "*ERROR collecting test_02_import_error.py*",
            "*No module named *asdfa*",
            "*! stopping after 1 failures !*",
            "*= 1 error in *",
        ]
    )
    res.stdout.no_fnmatch_line("*test_03*")


def test_exit_on_collection_with_maxfail_bigger_than_n_errors(
    pytester: Pytester,
) -> None:
    """
    Verify the test run aborts due to collection errors even if maxfail count of
    errors was not reached.
    """
    pytester.makepyfile(**COLLECTION_ERROR_PY_FILES)

    res = pytester.runpytest("--maxfail=4")
    assert res.ret == 2
    res.stdout.fnmatch_lines(
        [
            "collected 2 items / 2 errors",
            "*ERROR collecting test_02_import_error.py*",
            "*No module named *asdfa*",
            "*ERROR collecting test_03_import_error.py*",
            "*No module named *asdfa*",
            "*! Interrupted: 2 errors during collection !*",
            "*= 2 errors in *",
        ]
    )


def test_continue_on_collection_errors(pytester: Pytester) -> None:
    """
    Verify tests are executed even when collection errors occur when the
    --continue-on-collection-errors flag is set
    """
    pytester.makepyfile(**COLLECTION_ERROR_PY_FILES)

    res = pytester.runpytest("--continue-on-collection-errors")
    assert res.ret == 1

    res.stdout.fnmatch_lines(
        ["collected 2 items / 2 errors", "*1 failed, 1 passed, 2 errors*"]
    )


def test_continue_on_collection_errors_maxfail(pytester: Pytester) -> None:
    """
    Verify tests are executed even when collection errors occur and that maxfail
    is honoured (including the collection error count).
    4 tests: 2 collection errors + 1 failure + 1 success
    test_4 is never executed because the test run is with --maxfail=3 which
    means it is interrupted after the 2 collection errors + 1 failure.
    """
    pytester.makepyfile(**COLLECTION_ERROR_PY_FILES)

    res = pytester.runpytest("--continue-on-collection-errors", "--maxfail=3")
    assert res.ret == 1

    res.stdout.fnmatch_lines(["collected 2 items / 2 errors", "*1 failed, 2 errors*"])


def test_fixture_scope_sibling_conftests(pytester: Pytester) -> None:
    """Regression test case for https://github.com/pytest-dev/pytest/issues/2836"""
    foo_path = pytester.mkdir("foo")
    foo_path.joinpath("conftest.py").write_text(
        textwrap.dedent(
            """\
            import pytest
            @pytest.fixture
            def fix():
                return 1
            """
        )
    )
    foo_path.joinpath("test_foo.py").write_text("def test_foo(fix): assert fix == 1")

    # Tests in `food/` should not see the conftest fixture from `foo/`
    food_path = pytester.mkpydir("food")
    food_path.joinpath("test_food.py").write_text("def test_food(fix): assert fix == 1")

    res = pytester.runpytest()
    assert res.ret == 1

    res.stdout.fnmatch_lines(
        [
            "*ERROR at setup of test_food*",
            "E*fixture 'fix' not found",
            "*1 passed, 1 error*",
        ]
    )


def test_collect_init_tests(pytester: Pytester) -> None:
    """Check that we collect files from __init__.py files when they patch the 'python_files' (#3773)"""
    p = pytester.copy_example("collect/collect_init_tests")
    result = pytester.runpytest(p, "--collect-only")
    result.stdout.fnmatch_lines(
        [
            "collected 2 items",
            "<Package tests>",
            "  <Module __init__.py>",
            "    <Function test_init>",
            "  <Module test_foo.py>",
            "    <Function test_foo>",
        ]
    )
    result = pytester.runpytest("./tests", "--collect-only")
    result.stdout.fnmatch_lines(
        [
            "collected 2 items",
            "<Package tests>",
            "  <Module __init__.py>",
            "    <Function test_init>",
            "  <Module test_foo.py>",
            "    <Function test_foo>",
        ]
    )
    # Ignores duplicates with "." and pkginit (#4310).
    result = pytester.runpytest("./tests", ".", "--collect-only")
    result.stdout.fnmatch_lines(
        [
            "collected 2 items",
            "<Package tests>",
            "  <Module __init__.py>",
            "    <Function test_init>",
            "  <Module test_foo.py>",
            "    <Function test_foo>",
        ]
    )
    # Same as before, but different order.
    result = pytester.runpytest(".", "tests", "--collect-only")
    result.stdout.fnmatch_lines(
        [
            "collected 2 items",
            "<Package tests>",
            "  <Module __init__.py>",
            "    <Function test_init>",
            "  <Module test_foo.py>",
            "    <Function test_foo>",
        ]
    )
    result = pytester.runpytest("./tests/test_foo.py", "--collect-only")
    result.stdout.fnmatch_lines(
        ["<Package tests>", "  <Module test_foo.py>", "    <Function test_foo>"]
    )
    result.stdout.no_fnmatch_line("*test_init*")
    result = pytester.runpytest("./tests/__init__.py", "--collect-only")
    result.stdout.fnmatch_lines(
        ["<Package tests>", "  <Module __init__.py>", "    <Function test_init>"]
    )
    result.stdout.no_fnmatch_line("*test_foo*")


def test_collect_invalid_signature_message(pytester: Pytester) -> None:
    """Check that we issue a proper message when we can't determine the signature of a test
    function (#4026).
    """
    pytester.makepyfile(
        """
        import pytest

        class TestCase:
            @pytest.fixture
            def fix():
                pass
    """
    )
    result = pytester.runpytest()
    result.stdout.fnmatch_lines(
        ["Could not determine arguments of *.fix *: invalid method signature"]
    )


def test_collect_handles_raising_on_dunder_class(pytester: Pytester) -> None:
    """Handle proxy classes like Django's LazySettings that might raise on
    ``isinstance`` (#4266).
    """
    pytester.makepyfile(
        """
        class ImproperlyConfigured(Exception):
            pass

        class RaisesOnGetAttr(object):
            def raises(self):
                raise ImproperlyConfigured

            __class__ = property(raises)

        raises = RaisesOnGetAttr()


        def test_1():
            pass
    """
    )
    result = pytester.runpytest()
    result.stdout.fnmatch_lines(["*1 passed in*"])
    assert result.ret == 0


def test_collect_with_chdir_during_import(pytester: Pytester) -> None:
    subdir = pytester.mkdir("sub")
    pytester.path.joinpath("conftest.py").write_text(
        textwrap.dedent(
            """
            import os
            os.chdir(%r)
            """
            % (str(subdir),)
        )
    )
    pytester.makepyfile(
        """
        def test_1():
            import os
            assert os.getcwd() == %r
        """
        % (str(subdir),)
    )
    with pytester.path.cwd():
        result = pytester.runpytest()
    result.stdout.fnmatch_lines(["*1 passed in*"])
    assert result.ret == 0

    # Handles relative testpaths.
    pytester.makeini(
        """
        [pytest]
        testpaths = .
    """
    )
    with pytester.path.cwd():
        result = pytester.runpytest("--collect-only")
    result.stdout.fnmatch_lines(["collected 1 item"])


def test_collect_pyargs_with_testpaths(
    pytester: Pytester, monkeypatch: MonkeyPatch
) -> None:
    testmod = pytester.mkdir("testmod")
    # NOTE: __init__.py is not collected since it does not match python_files.
    testmod.joinpath("__init__.py").write_text("def test_func(): pass")
    testmod.joinpath("test_file.py").write_text("def test_func(): pass")

    root = pytester.mkdir("root")
    root.joinpath("pytest.ini").write_text(
        textwrap.dedent(
            """
        [pytest]
        addopts = --pyargs
        testpaths = testmod
    """
        )
    )
    monkeypatch.setenv("PYTHONPATH", str(pytester.path), prepend=os.pathsep)
    with root.cwd():
        result = pytester.runpytest_subprocess()
    result.stdout.fnmatch_lines(["*1 passed in*"])


def test_collect_symlink_file_arg(pytester: Pytester) -> None:
    """Collect a direct symlink works even if it does not match python_files (#4325)."""
    real = pytester.makepyfile(
        real="""
        def test_nodeid(request):
            assert request.node.nodeid == "symlink.py::test_nodeid"
        """
    )
    symlink = pytester.path.joinpath("symlink.py")
    symlink_or_skip(real, symlink)
    result = pytester.runpytest("-v", symlink)
    result.stdout.fnmatch_lines(["symlink.py::test_nodeid PASSED*", "*1 passed in*"])
    assert result.ret == 0


def test_collect_symlink_out_of_tree(pytester: Pytester) -> None:
    """Test collection of symlink via out-of-tree rootdir."""
    sub = pytester.mkdir("sub")
    real = sub.joinpath("test_real.py")
    real.write_text(
        textwrap.dedent(
            """
        def test_nodeid(request):
            # Should not contain sub/ prefix.
            assert request.node.nodeid == "test_real.py::test_nodeid"
        """
        ),
    )

    out_of_tree = pytester.mkdir("out_of_tree")
    symlink_to_sub = out_of_tree.joinpath("symlink_to_sub")
    symlink_or_skip(sub, symlink_to_sub)
    os.chdir(sub)
    result = pytester.runpytest("-vs", "--rootdir=%s" % sub, symlink_to_sub)
    result.stdout.fnmatch_lines(
        [
            # Should not contain "sub/"!
            "test_real.py::test_nodeid PASSED"
        ]
    )
    assert result.ret == 0


def test_collect_symlink_dir(pytester: Pytester) -> None:
    """A symlinked directory is collected."""
    dir = pytester.mkdir("dir")
    dir.joinpath("test_it.py").write_text("def test_it(): pass", "utf-8")
    symlink_or_skip(pytester.path.joinpath("symlink_dir"), dir)
    result = pytester.runpytest()
    result.assert_outcomes(passed=2)


def test_collectignore_via_conftest(pytester: Pytester) -> None:
    """collect_ignore in parent conftest skips importing child (issue #4592)."""
    tests = pytester.mkpydir("tests")
    tests.joinpath("conftest.py").write_text("collect_ignore = ['ignore_me']")

    ignore_me = tests.joinpath("ignore_me")
    ignore_me.mkdir()
    ignore_me.joinpath("__init__.py").touch()
    ignore_me.joinpath("conftest.py").write_text("assert 0, 'should_not_be_called'")

    result = pytester.runpytest()
    assert result.ret == ExitCode.NO_TESTS_COLLECTED


def test_collect_pkg_init_and_file_in_args(pytester: Pytester) -> None:
    subdir = pytester.mkdir("sub")
    init = subdir.joinpath("__init__.py")
    init.write_text("def test_init(): pass")
    p = subdir.joinpath("test_file.py")
    p.write_text("def test_file(): pass")

    # NOTE: without "-o python_files=*.py" this collects test_file.py twice.
    # This changed/broke with "Add package scoped fixtures #2283" (2b1410895)
    # initially (causing a RecursionError).
    result = pytester.runpytest("-v", str(init), str(p))
    result.stdout.fnmatch_lines(
        [
            "sub/test_file.py::test_file PASSED*",
            "sub/test_file.py::test_file PASSED*",
            "*2 passed in*",
        ]
    )

    result = pytester.runpytest("-v", "-o", "python_files=*.py", str(init), str(p))
    result.stdout.fnmatch_lines(
        [
            "sub/__init__.py::test_init PASSED*",
            "sub/test_file.py::test_file PASSED*",
            "*2 passed in*",
        ]
    )


def test_collect_pkg_init_only(pytester: Pytester) -> None:
    subdir = pytester.mkdir("sub")
    init = subdir.joinpath("__init__.py")
    init.write_text("def test_init(): pass")

    result = pytester.runpytest(str(init))
    result.stdout.fnmatch_lines(["*no tests ran in*"])

    result = pytester.runpytest("-v", "-o", "python_files=*.py", str(init))
    result.stdout.fnmatch_lines(["sub/__init__.py::test_init PASSED*", "*1 passed in*"])


@pytest.mark.parametrize("use_pkg", (True, False))
def test_collect_sub_with_symlinks(use_pkg: bool, pytester: Pytester) -> None:
    """Collection works with symlinked files and broken symlinks"""
    sub = pytester.mkdir("sub")
    if use_pkg:
        sub.joinpath("__init__.py").touch()
    sub.joinpath("test_file.py").write_text("def test_file(): pass")

    # Create a broken symlink.
    symlink_or_skip("test_doesnotexist.py", sub.joinpath("test_broken.py"))

    # Symlink that gets collected.
    symlink_or_skip("test_file.py", sub.joinpath("test_symlink.py"))

    result = pytester.runpytest("-v", str(sub))
    result.stdout.fnmatch_lines(
        [
            "sub/test_file.py::test_file PASSED*",
            "sub/test_symlink.py::test_file PASSED*",
            "*2 passed in*",
        ]
    )


def test_collector_respects_tbstyle(pytester: Pytester) -> None:
    p1 = pytester.makepyfile("assert 0")
    result = pytester.runpytest(p1, "--tb=native")
    assert result.ret == ExitCode.INTERRUPTED
    result.stdout.fnmatch_lines(
        [
            "*_ ERROR collecting test_collector_respects_tbstyle.py _*",
            "Traceback (most recent call last):",
            '  File "*/test_collector_respects_tbstyle.py", line 1, in <module>',
            "    assert 0",
            "AssertionError: assert 0",
            "*! Interrupted: 1 error during collection !*",
            "*= 1 error in *",
        ]
    )


def test_does_not_eagerly_collect_packages(pytester: Pytester) -> None:
    pytester.makepyfile("def test(): pass")
    pydir = pytester.mkpydir("foopkg")
    pydir.joinpath("__init__.py").write_text("assert False")
    result = pytester.runpytest()
    assert result.ret == ExitCode.OK


def test_does_not_put_src_on_path(pytester: Pytester) -> None:
    # `src` is not on sys.path so it should not be importable
    ensure_file(pytester.path / "src/nope/__init__.py")
    pytester.makepyfile(
        "import pytest\n"
        "def test():\n"
        "    with pytest.raises(ImportError):\n"
        "        import nope\n"
    )
    result = pytester.runpytest()
    assert result.ret == ExitCode.OK


def test_fscollector_from_parent(pytester: Pytester, request: FixtureRequest) -> None:
    """Ensure File.from_parent can forward custom arguments to the constructor.

    Context: https://github.com/pytest-dev/pytest-cpp/pull/47
    """

    class MyCollector(pytest.File):
        def __init__(self, *k, x, **kw):
            super().__init__(*k, **kw)
            self.x = x

    collector = MyCollector.from_parent(
        parent=request.session, path=pytester.path / "foo", x=10
    )
    assert collector.x == 10


def test_class_from_parent(pytester: Pytester, request: FixtureRequest) -> None:
    """Ensure Class.from_parent can forward custom arguments to the constructor."""

    class MyCollector(pytest.Class):
        def __init__(self, name, parent, x):
            super().__init__(name, parent)
            self.x = x

        @classmethod
        def from_parent(cls, parent, *, name, x):
            return super().from_parent(parent=parent, name=name, x=x)

    collector = MyCollector.from_parent(parent=request.session, name="foo", x=10)
    assert collector.x == 10


class TestImportModeImportlib:
    def test_collect_duplicate_names(self, pytester: Pytester) -> None:
        """--import-mode=importlib can import modules with same names that are not in packages."""
        pytester.makepyfile(
            **{
                "tests_a/test_foo.py": "def test_foo1(): pass",
                "tests_b/test_foo.py": "def test_foo2(): pass",
            }
        )
        result = pytester.runpytest("-v", "--import-mode=importlib")
        result.stdout.fnmatch_lines(
            [
                "tests_a/test_foo.py::test_foo1 *",
                "tests_b/test_foo.py::test_foo2 *",
                "* 2 passed in *",
            ]
        )

    def test_conftest(self, pytester: Pytester) -> None:
        """Directory containing conftest modules are not put in sys.path as a side-effect of
        importing them."""
        tests_dir = pytester.path.joinpath("tests")
        pytester.makepyfile(
            **{
                "tests/conftest.py": "",
                "tests/test_foo.py": """
                import sys
                def test_check():
                    assert r"{tests_dir}" not in sys.path
                """.format(
                    tests_dir=tests_dir
                ),
            }
        )
        result = pytester.runpytest("-v", "--import-mode=importlib")
        result.stdout.fnmatch_lines(["* 1 passed in *"])

    def setup_conftest_and_foo(self, pytester: Pytester) -> None:
        """Setup a tests folder to be used to test if modules in that folder can be imported
        due to side-effects of --import-mode or not."""
        pytester.makepyfile(
            **{
                "tests/conftest.py": "",
                "tests/foo.py": """
                    def foo(): return 42
                """,
                "tests/test_foo.py": """
                    def test_check():
                        from foo import foo
                        assert foo() == 42
                """,
            }
        )

    def test_modules_importable_as_side_effect(self, pytester: Pytester) -> None:
        """In import-modes `prepend` and `append`, we are able to import modules from folders
        containing conftest.py files due to the side effect of changing sys.path."""
        self.setup_conftest_and_foo(pytester)
        result = pytester.runpytest("-v", "--import-mode=prepend")
        result.stdout.fnmatch_lines(["* 1 passed in *"])

    def test_modules_not_importable_as_side_effect(self, pytester: Pytester) -> None:
        """In import-mode `importlib`, modules in folders containing conftest.py are not
        importable, as don't change sys.path or sys.modules as side effect of importing
        the conftest.py file.
        """
        self.setup_conftest_and_foo(pytester)
        result = pytester.runpytest("-v", "--import-mode=importlib")
        result.stdout.fnmatch_lines(
            [
                "*ModuleNotFoundError: No module named 'foo'",
                "tests?test_foo.py:2: ModuleNotFoundError",
                "* 1 failed in *",
            ]
        )


def test_does_not_crash_on_error_from_decorated_function(pytester: Pytester) -> None:
    """Regression test for an issue around bad exception formatting due to
    assertion rewriting mangling lineno's (#4984)."""
    pytester.makepyfile(
        """
        @pytest.fixture
        def a(): return 4
        """
    )
    result = pytester.runpytest()
    # Not INTERNAL_ERROR
    assert result.ret == ExitCode.INTERRUPTED


def test_does_not_crash_on_recursive_symlink(pytester: Pytester) -> None:
    """Regression test for an issue around recursive symlinks (#7951)."""
    symlink_or_skip("recursive", pytester.path.joinpath("recursive"))
    pytester.makepyfile(
        """
        def test_foo(): assert True
        """
    )
    result = pytester.runpytest()

    assert result.ret == ExitCode.OK
    assert result.parseoutcomes() == {"passed": 1}
