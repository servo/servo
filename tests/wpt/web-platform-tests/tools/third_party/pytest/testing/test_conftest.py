import argparse
import os
import textwrap
from pathlib import Path
from typing import cast
from typing import Dict
from typing import Generator
from typing import List
from typing import Optional

import pytest
from _pytest.config import ExitCode
from _pytest.config import PytestPluginManager
from _pytest.monkeypatch import MonkeyPatch
from _pytest.pathlib import symlink_or_skip
from _pytest.pytester import Pytester
from _pytest.tmpdir import TempPathFactory


def ConftestWithSetinitial(path) -> PytestPluginManager:
    conftest = PytestPluginManager()
    conftest_setinitial(conftest, [path])
    return conftest


def conftest_setinitial(
    conftest: PytestPluginManager, args, confcutdir: Optional["os.PathLike[str]"] = None
) -> None:
    class Namespace:
        def __init__(self) -> None:
            self.file_or_dir = args
            self.confcutdir = os.fspath(confcutdir) if confcutdir is not None else None
            self.noconftest = False
            self.pyargs = False
            self.importmode = "prepend"

    namespace = cast(argparse.Namespace, Namespace())
    conftest._set_initial_conftests(namespace, rootpath=Path(args[0]))


@pytest.mark.usefixtures("_sys_snapshot")
class TestConftestValueAccessGlobal:
    @pytest.fixture(scope="module", params=["global", "inpackage"])
    def basedir(
        self, request, tmp_path_factory: TempPathFactory
    ) -> Generator[Path, None, None]:
        tmp_path = tmp_path_factory.mktemp("basedir", numbered=True)
        tmp_path.joinpath("adir/b").mkdir(parents=True)
        tmp_path.joinpath("adir/conftest.py").write_text("a=1 ; Directory = 3")
        tmp_path.joinpath("adir/b/conftest.py").write_text("b=2 ; a = 1.5")
        if request.param == "inpackage":
            tmp_path.joinpath("adir/__init__.py").touch()
            tmp_path.joinpath("adir/b/__init__.py").touch()

        yield tmp_path

    def test_basic_init(self, basedir: Path) -> None:
        conftest = PytestPluginManager()
        p = basedir / "adir"
        assert (
            conftest._rget_with_confmod("a", p, importmode="prepend", rootpath=basedir)[
                1
            ]
            == 1
        )

    def test_immediate_initialiation_and_incremental_are_the_same(
        self, basedir: Path
    ) -> None:
        conftest = PytestPluginManager()
        assert not len(conftest._dirpath2confmods)
        conftest._getconftestmodules(
            basedir, importmode="prepend", rootpath=Path(basedir)
        )
        snap1 = len(conftest._dirpath2confmods)
        assert snap1 == 1
        conftest._getconftestmodules(
            basedir / "adir", importmode="prepend", rootpath=basedir
        )
        assert len(conftest._dirpath2confmods) == snap1 + 1
        conftest._getconftestmodules(
            basedir / "b", importmode="prepend", rootpath=basedir
        )
        assert len(conftest._dirpath2confmods) == snap1 + 2

    def test_value_access_not_existing(self, basedir: Path) -> None:
        conftest = ConftestWithSetinitial(basedir)
        with pytest.raises(KeyError):
            conftest._rget_with_confmod(
                "a", basedir, importmode="prepend", rootpath=Path(basedir)
            )

    def test_value_access_by_path(self, basedir: Path) -> None:
        conftest = ConftestWithSetinitial(basedir)
        adir = basedir / "adir"
        assert (
            conftest._rget_with_confmod(
                "a", adir, importmode="prepend", rootpath=basedir
            )[1]
            == 1
        )
        assert (
            conftest._rget_with_confmod(
                "a", adir / "b", importmode="prepend", rootpath=basedir
            )[1]
            == 1.5
        )

    def test_value_access_with_confmod(self, basedir: Path) -> None:
        startdir = basedir / "adir" / "b"
        startdir.joinpath("xx").mkdir()
        conftest = ConftestWithSetinitial(startdir)
        mod, value = conftest._rget_with_confmod(
            "a", startdir, importmode="prepend", rootpath=Path(basedir)
        )
        assert value == 1.5
        path = Path(mod.__file__)
        assert path.parent == basedir / "adir" / "b"
        assert path.stem == "conftest"


def test_conftest_in_nonpkg_with_init(tmp_path: Path, _sys_snapshot) -> None:
    tmp_path.joinpath("adir-1.0/b").mkdir(parents=True)
    tmp_path.joinpath("adir-1.0/conftest.py").write_text("a=1 ; Directory = 3")
    tmp_path.joinpath("adir-1.0/b/conftest.py").write_text("b=2 ; a = 1.5")
    tmp_path.joinpath("adir-1.0/b/__init__.py").touch()
    tmp_path.joinpath("adir-1.0/__init__.py").touch()
    ConftestWithSetinitial(tmp_path.joinpath("adir-1.0", "b"))


def test_doubledash_considered(pytester: Pytester) -> None:
    conf = pytester.mkdir("--option")
    conf.joinpath("conftest.py").touch()
    conftest = PytestPluginManager()
    conftest_setinitial(conftest, [conf.name, conf.name])
    values = conftest._getconftestmodules(
        conf, importmode="prepend", rootpath=pytester.path
    )
    assert len(values) == 1


def test_issue151_load_all_conftests(pytester: Pytester) -> None:
    names = "code proj src".split()
    for name in names:
        p = pytester.mkdir(name)
        p.joinpath("conftest.py").touch()

    conftest = PytestPluginManager()
    conftest_setinitial(conftest, names)
    d = list(conftest._conftestpath2mod.values())
    assert len(d) == len(names)


def test_conftest_global_import(pytester: Pytester) -> None:
    pytester.makeconftest("x=3")
    p = pytester.makepyfile(
        """
        from pathlib import Path
        import pytest
        from _pytest.config import PytestPluginManager
        conf = PytestPluginManager()
        mod = conf._importconftest(Path("conftest.py"), importmode="prepend", rootpath=Path.cwd())
        assert mod.x == 3
        import conftest
        assert conftest is mod, (conftest, mod)
        sub = Path("sub")
        sub.mkdir()
        subconf = sub / "conftest.py"
        subconf.write_text("y=4")
        mod2 = conf._importconftest(subconf, importmode="prepend", rootpath=Path.cwd())
        assert mod != mod2
        assert mod2.y == 4
        import conftest
        assert conftest is mod2, (conftest, mod)
    """
    )
    res = pytester.runpython(p)
    assert res.ret == 0


def test_conftestcutdir(pytester: Pytester) -> None:
    conf = pytester.makeconftest("")
    p = pytester.mkdir("x")
    conftest = PytestPluginManager()
    conftest_setinitial(conftest, [pytester.path], confcutdir=p)
    values = conftest._getconftestmodules(
        p, importmode="prepend", rootpath=pytester.path
    )
    assert len(values) == 0
    values = conftest._getconftestmodules(
        conf.parent, importmode="prepend", rootpath=pytester.path
    )
    assert len(values) == 0
    assert Path(conf) not in conftest._conftestpath2mod
    # but we can still import a conftest directly
    conftest._importconftest(conf, importmode="prepend", rootpath=pytester.path)
    values = conftest._getconftestmodules(
        conf.parent, importmode="prepend", rootpath=pytester.path
    )
    assert values[0].__file__.startswith(str(conf))
    # and all sub paths get updated properly
    values = conftest._getconftestmodules(
        p, importmode="prepend", rootpath=pytester.path
    )
    assert len(values) == 1
    assert values[0].__file__.startswith(str(conf))


def test_conftestcutdir_inplace_considered(pytester: Pytester) -> None:
    conf = pytester.makeconftest("")
    conftest = PytestPluginManager()
    conftest_setinitial(conftest, [conf.parent], confcutdir=conf.parent)
    values = conftest._getconftestmodules(
        conf.parent, importmode="prepend", rootpath=pytester.path
    )
    assert len(values) == 1
    assert values[0].__file__.startswith(str(conf))


@pytest.mark.parametrize("name", "test tests whatever .dotdir".split())
def test_setinitial_conftest_subdirs(pytester: Pytester, name: str) -> None:
    sub = pytester.mkdir(name)
    subconftest = sub.joinpath("conftest.py")
    subconftest.touch()
    conftest = PytestPluginManager()
    conftest_setinitial(conftest, [sub.parent], confcutdir=pytester.path)
    key = subconftest.resolve()
    if name not in ("whatever", ".dotdir"):
        assert key in conftest._conftestpath2mod
        assert len(conftest._conftestpath2mod) == 1
    else:
        assert key not in conftest._conftestpath2mod
        assert len(conftest._conftestpath2mod) == 0


def test_conftest_confcutdir(pytester: Pytester) -> None:
    pytester.makeconftest("assert 0")
    x = pytester.mkdir("x")
    x.joinpath("conftest.py").write_text(
        textwrap.dedent(
            """\
            def pytest_addoption(parser):
                parser.addoption("--xyz", action="store_true")
            """
        )
    )
    result = pytester.runpytest("-h", "--confcutdir=%s" % x, x)
    result.stdout.fnmatch_lines(["*--xyz*"])
    result.stdout.no_fnmatch_line("*warning: could not load initial*")


def test_conftest_symlink(pytester: Pytester) -> None:
    """`conftest.py` discovery follows normal path resolution and does not resolve symlinks."""
    # Structure:
    # /real
    # /real/conftest.py
    # /real/app
    # /real/app/tests
    # /real/app/tests/test_foo.py

    # Links:
    # /symlinktests -> /real/app/tests (running at symlinktests should fail)
    # /symlink -> /real (running at /symlink should work)

    real = pytester.mkdir("real")
    realtests = real.joinpath("app/tests")
    realtests.mkdir(parents=True)
    symlink_or_skip(realtests, pytester.path.joinpath("symlinktests"))
    symlink_or_skip(real, pytester.path.joinpath("symlink"))
    pytester.makepyfile(
        **{
            "real/app/tests/test_foo.py": "def test1(fixture): pass",
            "real/conftest.py": textwrap.dedent(
                """
                import pytest

                print("conftest_loaded")

                @pytest.fixture
                def fixture():
                    print("fixture_used")
                """
            ),
        }
    )

    # Should fail because conftest cannot be found from the link structure.
    result = pytester.runpytest("-vs", "symlinktests")
    result.stdout.fnmatch_lines(["*fixture 'fixture' not found*"])
    assert result.ret == ExitCode.TESTS_FAILED

    # Should not cause "ValueError: Plugin already registered" (#4174).
    result = pytester.runpytest("-vs", "symlink")
    assert result.ret == ExitCode.OK


def test_conftest_symlink_files(pytester: Pytester) -> None:
    """Symlinked conftest.py are found when pytest is executed in a directory with symlinked
    files."""
    real = pytester.mkdir("real")
    source = {
        "app/test_foo.py": "def test1(fixture): pass",
        "app/__init__.py": "",
        "app/conftest.py": textwrap.dedent(
            """
            import pytest

            print("conftest_loaded")

            @pytest.fixture
            def fixture():
                print("fixture_used")
            """
        ),
    }
    pytester.makepyfile(**{"real/%s" % k: v for k, v in source.items()})

    # Create a build directory that contains symlinks to actual files
    # but doesn't symlink actual directories.
    build = pytester.mkdir("build")
    build.joinpath("app").mkdir()
    for f in source:
        symlink_or_skip(real.joinpath(f), build.joinpath(f))
    os.chdir(build)
    result = pytester.runpytest("-vs", "app/test_foo.py")
    result.stdout.fnmatch_lines(["*conftest_loaded*", "PASSED"])
    assert result.ret == ExitCode.OK


@pytest.mark.skipif(
    os.path.normcase("x") != os.path.normcase("X"),
    reason="only relevant for case insensitive file systems",
)
def test_conftest_badcase(pytester: Pytester) -> None:
    """Check conftest.py loading when directory casing is wrong (#5792)."""
    pytester.path.joinpath("JenkinsRoot/test").mkdir(parents=True)
    source = {"setup.py": "", "test/__init__.py": "", "test/conftest.py": ""}
    pytester.makepyfile(**{"JenkinsRoot/%s" % k: v for k, v in source.items()})

    os.chdir(pytester.path.joinpath("jenkinsroot/test"))
    result = pytester.runpytest()
    assert result.ret == ExitCode.NO_TESTS_COLLECTED


def test_conftest_uppercase(pytester: Pytester) -> None:
    """Check conftest.py whose qualified name contains uppercase characters (#5819)"""
    source = {"__init__.py": "", "Foo/conftest.py": "", "Foo/__init__.py": ""}
    pytester.makepyfile(**source)

    os.chdir(pytester.path)
    result = pytester.runpytest()
    assert result.ret == ExitCode.NO_TESTS_COLLECTED


def test_no_conftest(pytester: Pytester) -> None:
    pytester.makeconftest("assert 0")
    result = pytester.runpytest("--noconftest")
    assert result.ret == ExitCode.NO_TESTS_COLLECTED

    result = pytester.runpytest()
    assert result.ret == ExitCode.USAGE_ERROR


def test_conftest_existing_junitxml(pytester: Pytester) -> None:
    x = pytester.mkdir("tests")
    x.joinpath("conftest.py").write_text(
        textwrap.dedent(
            """\
            def pytest_addoption(parser):
                parser.addoption("--xyz", action="store_true")
            """
        )
    )
    pytester.makefile(ext=".xml", junit="")  # Writes junit.xml
    result = pytester.runpytest("-h", "--junitxml", "junit.xml")
    result.stdout.fnmatch_lines(["*--xyz*"])


def test_conftest_import_order(pytester: Pytester, monkeypatch: MonkeyPatch) -> None:
    ct1 = pytester.makeconftest("")
    sub = pytester.mkdir("sub")
    ct2 = sub / "conftest.py"
    ct2.write_text("")

    def impct(p, importmode, root):
        return p

    conftest = PytestPluginManager()
    conftest._confcutdir = pytester.path
    monkeypatch.setattr(conftest, "_importconftest", impct)
    mods = cast(
        List[Path],
        conftest._getconftestmodules(sub, importmode="prepend", rootpath=pytester.path),
    )
    expected = [ct1, ct2]
    assert mods == expected


def test_fixture_dependency(pytester: Pytester) -> None:
    pytester.makeconftest("")
    pytester.path.joinpath("__init__.py").touch()
    sub = pytester.mkdir("sub")
    sub.joinpath("__init__.py").touch()
    sub.joinpath("conftest.py").write_text(
        textwrap.dedent(
            """\
            import pytest

            @pytest.fixture
            def not_needed():
                assert False, "Should not be called!"

            @pytest.fixture
            def foo():
                assert False, "Should not be called!"

            @pytest.fixture
            def bar(foo):
                return 'bar'
            """
        )
    )
    subsub = sub.joinpath("subsub")
    subsub.mkdir()
    subsub.joinpath("__init__.py").touch()
    subsub.joinpath("test_bar.py").write_text(
        textwrap.dedent(
            """\
            import pytest

            @pytest.fixture
            def bar():
                return 'sub bar'

            def test_event_fixture(bar):
                assert bar == 'sub bar'
            """
        )
    )
    result = pytester.runpytest("sub")
    result.stdout.fnmatch_lines(["*1 passed*"])


def test_conftest_found_with_double_dash(pytester: Pytester) -> None:
    sub = pytester.mkdir("sub")
    sub.joinpath("conftest.py").write_text(
        textwrap.dedent(
            """\
            def pytest_addoption(parser):
                parser.addoption("--hello-world", action="store_true")
            """
        )
    )
    p = sub.joinpath("test_hello.py")
    p.write_text("def test_hello(): pass")
    result = pytester.runpytest(str(p) + "::test_hello", "-h")
    result.stdout.fnmatch_lines(
        """
        *--hello-world*
    """
    )


class TestConftestVisibility:
    def _setup_tree(self, pytester: Pytester) -> Dict[str, Path]:  # for issue616
        # example mostly taken from:
        # https://mail.python.org/pipermail/pytest-dev/2014-September/002617.html
        runner = pytester.mkdir("empty")
        package = pytester.mkdir("package")

        package.joinpath("conftest.py").write_text(
            textwrap.dedent(
                """\
                import pytest
                @pytest.fixture
                def fxtr():
                    return "from-package"
                """
            )
        )
        package.joinpath("test_pkgroot.py").write_text(
            textwrap.dedent(
                """\
                def test_pkgroot(fxtr):
                    assert fxtr == "from-package"
                """
            )
        )

        swc = package.joinpath("swc")
        swc.mkdir()
        swc.joinpath("__init__.py").touch()
        swc.joinpath("conftest.py").write_text(
            textwrap.dedent(
                """\
                import pytest
                @pytest.fixture
                def fxtr():
                    return "from-swc"
                """
            )
        )
        swc.joinpath("test_with_conftest.py").write_text(
            textwrap.dedent(
                """\
                def test_with_conftest(fxtr):
                    assert fxtr == "from-swc"
                """
            )
        )

        snc = package.joinpath("snc")
        snc.mkdir()
        snc.joinpath("__init__.py").touch()
        snc.joinpath("test_no_conftest.py").write_text(
            textwrap.dedent(
                """\
                def test_no_conftest(fxtr):
                    assert fxtr == "from-package"   # No local conftest.py, so should
                                                    # use value from parent dir's
                """
            )
        )
        print("created directory structure:")
        for x in pytester.path.rglob(""):
            print("   " + str(x.relative_to(pytester.path)))

        return {"runner": runner, "package": package, "swc": swc, "snc": snc}

    # N.B.: "swc" stands for "subdir with conftest.py"
    #       "snc" stands for "subdir no [i.e. without] conftest.py"
    @pytest.mark.parametrize(
        "chdir,testarg,expect_ntests_passed",
        [
            # Effective target: package/..
            ("runner", "..", 3),
            ("package", "..", 3),
            ("swc", "../..", 3),
            ("snc", "../..", 3),
            # Effective target: package
            ("runner", "../package", 3),
            ("package", ".", 3),
            ("swc", "..", 3),
            ("snc", "..", 3),
            # Effective target: package/swc
            ("runner", "../package/swc", 1),
            ("package", "./swc", 1),
            ("swc", ".", 1),
            ("snc", "../swc", 1),
            # Effective target: package/snc
            ("runner", "../package/snc", 1),
            ("package", "./snc", 1),
            ("swc", "../snc", 1),
            ("snc", ".", 1),
        ],
    )
    def test_parsefactories_relative_node_ids(
        self, pytester: Pytester, chdir: str, testarg: str, expect_ntests_passed: int
    ) -> None:
        """#616"""
        dirs = self._setup_tree(pytester)
        print("pytest run in cwd: %s" % (dirs[chdir].relative_to(pytester.path)))
        print("pytestarg        : %s" % testarg)
        print("expected pass    : %s" % expect_ntests_passed)
        os.chdir(dirs[chdir])
        reprec = pytester.inline_run(testarg, "-q", "--traceconfig")
        reprec.assertoutcome(passed=expect_ntests_passed)


@pytest.mark.parametrize(
    "confcutdir,passed,error", [(".", 2, 0), ("src", 1, 1), (None, 1, 1)]
)
def test_search_conftest_up_to_inifile(
    pytester: Pytester, confcutdir: str, passed: int, error: int
) -> None:
    """Test that conftest files are detected only up to an ini file, unless
    an explicit --confcutdir option is given.
    """
    root = pytester.path
    src = root.joinpath("src")
    src.mkdir()
    src.joinpath("pytest.ini").write_text("[pytest]")
    src.joinpath("conftest.py").write_text(
        textwrap.dedent(
            """\
            import pytest
            @pytest.fixture
            def fix1(): pass
            """
        )
    )
    src.joinpath("test_foo.py").write_text(
        textwrap.dedent(
            """\
            def test_1(fix1):
                pass
            def test_2(out_of_reach):
                pass
            """
        )
    )
    root.joinpath("conftest.py").write_text(
        textwrap.dedent(
            """\
            import pytest
            @pytest.fixture
            def out_of_reach(): pass
            """
        )
    )

    args = [str(src)]
    if confcutdir:
        args = ["--confcutdir=%s" % root.joinpath(confcutdir)]
    result = pytester.runpytest(*args)
    match = ""
    if passed:
        match += "*%d passed*" % passed
    if error:
        match += "*%d error*" % error
    result.stdout.fnmatch_lines(match)


def test_issue1073_conftest_special_objects(pytester: Pytester) -> None:
    pytester.makeconftest(
        """\
        class DontTouchMe(object):
            def __getattr__(self, x):
                raise Exception('cant touch me')

        x = DontTouchMe()
        """
    )
    pytester.makepyfile(
        """\
        def test_some():
            pass
        """
    )
    res = pytester.runpytest()
    assert res.ret == 0


def test_conftest_exception_handling(pytester: Pytester) -> None:
    pytester.makeconftest(
        """\
        raise ValueError()
        """
    )
    pytester.makepyfile(
        """\
        def test_some():
            pass
        """
    )
    res = pytester.runpytest()
    assert res.ret == 4
    assert "raise ValueError()" in [line.strip() for line in res.errlines]


def test_hook_proxy(pytester: Pytester) -> None:
    """Session's gethookproxy() would cache conftests incorrectly (#2016).
    It was decided to remove the cache altogether.
    """
    pytester.makepyfile(
        **{
            "root/demo-0/test_foo1.py": "def test1(): pass",
            "root/demo-a/test_foo2.py": "def test1(): pass",
            "root/demo-a/conftest.py": """\
            def pytest_ignore_collect(collection_path, config):
                return True
            """,
            "root/demo-b/test_foo3.py": "def test1(): pass",
            "root/demo-c/test_foo4.py": "def test1(): pass",
        }
    )
    result = pytester.runpytest()
    result.stdout.fnmatch_lines(
        ["*test_foo1.py*", "*test_foo3.py*", "*test_foo4.py*", "*3 passed*"]
    )


def test_required_option_help(pytester: Pytester) -> None:
    pytester.makeconftest("assert 0")
    x = pytester.mkdir("x")
    x.joinpath("conftest.py").write_text(
        textwrap.dedent(
            """\
            def pytest_addoption(parser):
                parser.addoption("--xyz", action="store_true", required=True)
            """
        )
    )
    result = pytester.runpytest("-h", x)
    result.stdout.no_fnmatch_line("*argument --xyz is required*")
    assert "general:" in result.stdout.str()
