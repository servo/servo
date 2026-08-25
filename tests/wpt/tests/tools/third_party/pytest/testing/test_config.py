# mypy: allow-untyped-defs
import dataclasses
import importlib.metadata
import os
from pathlib import Path
import re
import sys
import textwrap
from typing import Any
from typing import Dict
from typing import List
from typing import Sequence
from typing import Tuple
from typing import Type
from typing import Union

import _pytest._code
from _pytest.config import _get_plugin_specs_as_list
from _pytest.config import _iter_rewritable_modules
from _pytest.config import _strtobool
from _pytest.config import Config
from _pytest.config import ConftestImportFailure
from _pytest.config import ExitCode
from _pytest.config import parse_warning_filter
from _pytest.config.argparsing import get_ini_default_for_type
from _pytest.config.argparsing import Parser
from _pytest.config.exceptions import UsageError
from _pytest.config.findpaths import determine_setup
from _pytest.config.findpaths import get_common_ancestor
from _pytest.config.findpaths import locate_config
from _pytest.monkeypatch import MonkeyPatch
from _pytest.pathlib import absolutepath
from _pytest.pytester import Pytester
import pytest


class TestParseIni:
    @pytest.mark.parametrize(
        "section, filename", [("pytest", "pytest.ini"), ("tool:pytest", "setup.cfg")]
    )
    def test_getcfg_and_config(
        self,
        pytester: Pytester,
        tmp_path: Path,
        section: str,
        filename: str,
        monkeypatch: MonkeyPatch,
    ) -> None:
        sub = tmp_path / "sub"
        sub.mkdir()
        monkeypatch.chdir(sub)
        (tmp_path / filename).write_text(
            textwrap.dedent(
                f"""\
                [{section}]
                name = value
                """
            ),
            encoding="utf-8",
        )
        _, _, cfg = locate_config(Path.cwd(), [sub])
        assert cfg["name"] == "value"
        config = pytester.parseconfigure(str(sub))
        assert config.inicfg["name"] == "value"

    def test_setupcfg_uses_toolpytest_with_pytest(self, pytester: Pytester) -> None:
        p1 = pytester.makepyfile("def test(): pass")
        pytester.makefile(
            ".cfg",
            setup="""
                [tool:pytest]
                testpaths=%s
                [pytest]
                testpaths=ignored
        """
            % p1.name,
        )
        result = pytester.runpytest()
        result.stdout.fnmatch_lines(["configfile: setup.cfg", "* 1 passed in *"])
        assert result.ret == 0

    def test_append_parse_args(
        self, pytester: Pytester, tmp_path: Path, monkeypatch: MonkeyPatch
    ) -> None:
        monkeypatch.setenv("PYTEST_ADDOPTS", '--color no -rs --tb="short"')
        tmp_path.joinpath("pytest.ini").write_text(
            textwrap.dedent(
                """\
                [pytest]
                addopts = --verbose
                """
            ),
            encoding="utf-8",
        )
        config = pytester.parseconfig(tmp_path)
        assert config.option.color == "no"
        assert config.option.reportchars == "s"
        assert config.option.tbstyle == "short"
        assert config.option.verbose

    def test_tox_ini_wrong_version(self, pytester: Pytester) -> None:
        pytester.makefile(
            ".ini",
            tox="""
            [pytest]
            minversion=999.0
        """,
        )
        result = pytester.runpytest()
        assert result.ret != 0
        result.stderr.fnmatch_lines(
            ["*tox.ini: 'minversion' requires pytest-999.0, actual pytest-*"]
        )

    @pytest.mark.parametrize(
        "section, name",
        [
            ("tool:pytest", "setup.cfg"),
            ("pytest", "tox.ini"),
            ("pytest", "pytest.ini"),
            ("pytest", ".pytest.ini"),
        ],
    )
    def test_ini_names(self, pytester: Pytester, name, section) -> None:
        pytester.path.joinpath(name).write_text(
            textwrap.dedent(
                f"""
            [{section}]
            minversion = 3.36
        """
            ),
            encoding="utf-8",
        )
        config = pytester.parseconfig()
        assert config.getini("minversion") == "3.36"

    def test_pyproject_toml(self, pytester: Pytester) -> None:
        pyproject_toml = pytester.makepyprojecttoml(
            """
            [tool.pytest.ini_options]
            minversion = "1.0"
        """
        )
        config = pytester.parseconfig()
        assert config.inipath == pyproject_toml
        assert config.getini("minversion") == "1.0"

    def test_empty_pyproject_toml(self, pytester: Pytester) -> None:
        """An empty pyproject.toml is considered as config if no other option is found."""
        pyproject_toml = pytester.makepyprojecttoml("")
        config = pytester.parseconfig()
        assert config.inipath == pyproject_toml

    def test_empty_pyproject_toml_found_many(self, pytester: Pytester) -> None:
        """
        In case we find multiple pyproject.toml files in our search, without a [tool.pytest.ini_options]
        table and without finding other candidates, the closest to where we started wins.
        """
        pytester.makefile(
            ".toml",
            **{
                "pyproject": "",
                "foo/pyproject": "",
                "foo/bar/pyproject": "",
            },
        )
        config = pytester.parseconfig(pytester.path / "foo/bar")
        assert config.inipath == pytester.path / "foo/bar/pyproject.toml"

    def test_pytest_ini_trumps_pyproject_toml(self, pytester: Pytester) -> None:
        """A pytest.ini always take precedence over a pyproject.toml file."""
        pytester.makepyprojecttoml("[tool.pytest.ini_options]")
        pytest_ini = pytester.makefile(".ini", pytest="")
        config = pytester.parseconfig()
        assert config.inipath == pytest_ini

    def test_toxini_before_lower_pytestini(self, pytester: Pytester) -> None:
        sub = pytester.mkdir("sub")
        sub.joinpath("tox.ini").write_text(
            textwrap.dedent(
                """
            [pytest]
            minversion = 2.0
        """
            ),
            encoding="utf-8",
        )
        pytester.path.joinpath("pytest.ini").write_text(
            textwrap.dedent(
                """
            [pytest]
            minversion = 1.5
        """
            ),
            encoding="utf-8",
        )
        config = pytester.parseconfigure(sub)
        assert config.getini("minversion") == "2.0"

    def test_ini_parse_error(self, pytester: Pytester) -> None:
        pytester.path.joinpath("pytest.ini").write_text(
            "addopts = -x", encoding="utf-8"
        )
        result = pytester.runpytest()
        assert result.ret != 0
        result.stderr.fnmatch_lines("ERROR: *pytest.ini:1: no section header defined")

    def test_toml_parse_error(self, pytester: Pytester) -> None:
        pytester.makepyprojecttoml(
            """
            \\"
            """
        )
        result = pytester.runpytest()
        assert result.ret != 0
        result.stderr.fnmatch_lines("ERROR: *pyproject.toml: Invalid statement*")

    def test_confcutdir_default_without_configfile(self, pytester: Pytester) -> None:
        # If --confcutdir is not specified, and there is no configfile, default
        # to the rootpath.
        sub = pytester.mkdir("sub")
        os.chdir(sub)
        config = pytester.parseconfigure()
        assert config.pluginmanager._confcutdir == sub

    def test_confcutdir_default_with_configfile(self, pytester: Pytester) -> None:
        # If --confcutdir is not specified, and there is a configfile, default
        # to the configfile's directory.
        pytester.makeini("[pytest]")
        sub = pytester.mkdir("sub")
        os.chdir(sub)
        config = pytester.parseconfigure()
        assert config.pluginmanager._confcutdir == pytester.path

    @pytest.mark.xfail(reason="probably not needed")
    def test_confcutdir(self, pytester: Pytester) -> None:
        sub = pytester.mkdir("sub")
        os.chdir(sub)
        pytester.makeini(
            """
            [pytest]
            addopts = --qwe
        """
        )
        result = pytester.inline_run("--confcutdir=.")
        assert result.ret == 0

    @pytest.mark.parametrize(
        "ini_file_text, invalid_keys, warning_output, exception_text",
        [
            pytest.param(
                """
                [pytest]
                unknown_ini = value1
                another_unknown_ini = value2
                """,
                ["unknown_ini", "another_unknown_ini"],
                [
                    "=*= warnings summary =*=",
                    "*PytestConfigWarning:*Unknown config option: another_unknown_ini",
                    "*PytestConfigWarning:*Unknown config option: unknown_ini",
                ],
                "Unknown config option: another_unknown_ini",
                id="2-unknowns",
            ),
            pytest.param(
                """
                [pytest]
                unknown_ini = value1
                minversion = 5.0.0
                """,
                ["unknown_ini"],
                [
                    "=*= warnings summary =*=",
                    "*PytestConfigWarning:*Unknown config option: unknown_ini",
                ],
                "Unknown config option: unknown_ini",
                id="1-unknown",
            ),
            pytest.param(
                """
                [some_other_header]
                unknown_ini = value1
                [pytest]
                minversion = 5.0.0
                """,
                [],
                [],
                "",
                id="unknown-in-other-header",
            ),
            pytest.param(
                """
                [pytest]
                minversion = 5.0.0
                """,
                [],
                [],
                "",
                id="no-unknowns",
            ),
            pytest.param(
                """
                [pytest]
                conftest_ini_key = 1
                """,
                [],
                [],
                "",
                id="1-known",
            ),
        ],
    )
    @pytest.mark.filterwarnings("default")
    def test_invalid_config_options(
        self,
        pytester: Pytester,
        ini_file_text,
        invalid_keys,
        warning_output,
        exception_text,
    ) -> None:
        pytester.makeconftest(
            """
            def pytest_addoption(parser):
                parser.addini("conftest_ini_key", "")
            """
        )
        pytester.makepyfile("def test(): pass")
        pytester.makeini(ini_file_text)

        config = pytester.parseconfig()
        assert sorted(config._get_unknown_ini_keys()) == sorted(invalid_keys)

        result = pytester.runpytest()
        result.stdout.fnmatch_lines(warning_output)

        result = pytester.runpytest("--strict-config")
        if exception_text:
            result.stderr.fnmatch_lines("ERROR: " + exception_text)
            assert result.ret == pytest.ExitCode.USAGE_ERROR
        else:
            result.stderr.no_fnmatch_line(exception_text)
            assert result.ret == pytest.ExitCode.OK

    @pytest.mark.filterwarnings("default")
    def test_silence_unknown_key_warning(self, pytester: Pytester) -> None:
        """Unknown config key warnings can be silenced using filterwarnings (#7620)"""
        pytester.makeini(
            """
            [pytest]
            filterwarnings =
                ignore:Unknown config option:pytest.PytestConfigWarning
            foobar=1
        """
        )
        result = pytester.runpytest()
        result.stdout.no_fnmatch_line("*PytestConfigWarning*")

    @pytest.mark.filterwarnings("default::pytest.PytestConfigWarning")
    def test_disable_warnings_plugin_disables_config_warnings(
        self, pytester: Pytester
    ) -> None:
        """Disabling 'warnings' plugin also disables config time warnings"""
        pytester.makeconftest(
            """
            import pytest
            def pytest_configure(config):
                config.issue_config_time_warning(
                    pytest.PytestConfigWarning("custom config warning"),
                    stacklevel=2,
                )
        """
        )
        result = pytester.runpytest("-pno:warnings")
        result.stdout.no_fnmatch_line("*PytestConfigWarning*")

    @pytest.mark.parametrize(
        "ini_file_text, plugin_version, exception_text",
        [
            pytest.param(
                """
                [pytest]
                required_plugins = a z
                """,
                "1.5",
                "Missing required plugins: a, z",
                id="2-missing",
            ),
            pytest.param(
                """
                [pytest]
                required_plugins = a z myplugin
                """,
                "1.5",
                "Missing required plugins: a, z",
                id="2-missing-1-ok",
            ),
            pytest.param(
                """
                [pytest]
                required_plugins = myplugin
                """,
                "1.5",
                None,
                id="1-ok",
            ),
            pytest.param(
                """
                [pytest]
                required_plugins = myplugin==1.5
                """,
                "1.5",
                None,
                id="1-ok-pin-exact",
            ),
            pytest.param(
                """
                [pytest]
                required_plugins = myplugin>1.0,<2.0
                """,
                "1.5",
                None,
                id="1-ok-pin-loose",
            ),
            pytest.param(
                """
                [pytest]
                required_plugins = myplugin
                """,
                "1.5a1",
                None,
                id="1-ok-prerelease",
            ),
            pytest.param(
                """
                [pytest]
                required_plugins = myplugin==1.6
                """,
                "1.5",
                "Missing required plugins: myplugin==1.6",
                id="missing-version",
            ),
            pytest.param(
                """
                [pytest]
                required_plugins = myplugin==1.6 other==1.0
                """,
                "1.5",
                "Missing required plugins: myplugin==1.6, other==1.0",
                id="missing-versions",
            ),
            pytest.param(
                """
                [some_other_header]
                required_plugins = won't be triggered
                [pytest]
                """,
                "1.5",
                None,
                id="invalid-header",
            ),
        ],
    )
    def test_missing_required_plugins(
        self,
        pytester: Pytester,
        monkeypatch: MonkeyPatch,
        ini_file_text: str,
        plugin_version: str,
        exception_text: str,
    ) -> None:
        """Check 'required_plugins' option with various settings.

        This test installs a mock "myplugin-1.5" which is used in the parametrized test cases.
        """

        @dataclasses.dataclass
        class DummyEntryPoint:
            name: str
            module: str
            group: str = "pytest11"

            def load(self):
                __import__(self.module)
                return sys.modules[self.module]

        entry_points = [
            DummyEntryPoint("myplugin1", "myplugin1_module"),
        ]

        @dataclasses.dataclass
        class DummyDist:
            entry_points: object
            files: object = ()
            version: str = plugin_version

            @property
            def metadata(self):
                return {"name": "myplugin"}

        def my_dists():
            return [DummyDist(entry_points)]

        pytester.makepyfile(myplugin1_module="# my plugin module")
        pytester.syspathinsert()

        monkeypatch.setattr(importlib.metadata, "distributions", my_dists)
        monkeypatch.delenv("PYTEST_DISABLE_PLUGIN_AUTOLOAD", raising=False)

        pytester.makeini(ini_file_text)

        if exception_text:
            with pytest.raises(pytest.UsageError, match=exception_text):
                pytester.parseconfig()
        else:
            pytester.parseconfig()

    def test_early_config_cmdline(
        self, pytester: Pytester, monkeypatch: MonkeyPatch
    ) -> None:
        """early_config contains options registered by third-party plugins.

        This is a regression involving pytest-cov (and possibly others) introduced in #7700.
        """
        pytester.makepyfile(
            myplugin="""
            def pytest_addoption(parser):
                parser.addoption('--foo', default=None, dest='foo')

            def pytest_load_initial_conftests(early_config, parser, args):
                assert early_config.known_args_namespace.foo == "1"
            """
        )
        monkeypatch.setenv("PYTEST_PLUGINS", "myplugin")
        pytester.syspathinsert()
        result = pytester.runpytest("--foo=1")
        result.stdout.fnmatch_lines("* no tests ran in *")

    def test_args_source_args(self, pytester: Pytester):
        config = pytester.parseconfig("--", "test_filename.py")
        assert config.args_source == Config.ArgsSource.ARGS

    def test_args_source_invocation_dir(self, pytester: Pytester):
        config = pytester.parseconfig()
        assert config.args_source == Config.ArgsSource.INVOCATION_DIR

    def test_args_source_testpaths(self, pytester: Pytester):
        pytester.makeini(
            """
            [pytest]
            testpaths=*
        """
        )
        config = pytester.parseconfig()
        assert config.args_source == Config.ArgsSource.TESTPATHS


class TestConfigCmdlineParsing:
    def test_parsing_again_fails(self, pytester: Pytester) -> None:
        config = pytester.parseconfig()
        pytest.raises(AssertionError, lambda: config.parse([]))

    def test_explicitly_specified_config_file_is_loaded(
        self, pytester: Pytester
    ) -> None:
        pytester.makeconftest(
            """
            def pytest_addoption(parser):
                parser.addini("custom", "")
        """
        )
        pytester.makeini(
            """
            [pytest]
            custom = 0
        """
        )
        pytester.makefile(
            ".ini",
            custom="""
            [pytest]
            custom = 1
        """,
        )
        config = pytester.parseconfig("-c", "custom.ini")
        assert config.getini("custom") == "1"
        config = pytester.parseconfig("--config-file", "custom.ini")
        assert config.getini("custom") == "1"

        pytester.makefile(
            ".cfg",
            custom_tool_pytest_section="""
            [tool:pytest]
            custom = 1
        """,
        )
        config = pytester.parseconfig("-c", "custom_tool_pytest_section.cfg")
        assert config.getini("custom") == "1"
        config = pytester.parseconfig("--config-file", "custom_tool_pytest_section.cfg")
        assert config.getini("custom") == "1"

        pytester.makefile(
            ".toml",
            custom="""
                [tool.pytest.ini_options]
                custom = 1
                value = [
                ]  # this is here on purpose, as it makes this an invalid '.ini' file
            """,
        )
        config = pytester.parseconfig("-c", "custom.toml")
        assert config.getini("custom") == "1"
        config = pytester.parseconfig("--config-file", "custom.toml")
        assert config.getini("custom") == "1"

    def test_absolute_win32_path(self, pytester: Pytester) -> None:
        temp_ini_file = pytester.makefile(
            ".ini",
            custom="""
            [pytest]
            addopts = --version
        """,
        )
        from os.path import normpath

        temp_ini_file_norm = normpath(str(temp_ini_file))
        ret = pytest.main(["-c", temp_ini_file_norm])
        assert ret == ExitCode.OK
        ret = pytest.main(["--config-file", temp_ini_file_norm])
        assert ret == ExitCode.OK


class TestConfigAPI:
    def test_config_trace(self, pytester: Pytester) -> None:
        config = pytester.parseconfig()
        values: List[str] = []
        config.trace.root.setwriter(values.append)
        config.trace("hello")
        assert len(values) == 1
        assert values[0] == "hello [config]\n"

    def test_config_getoption(self, pytester: Pytester) -> None:
        pytester.makeconftest(
            """
            def pytest_addoption(parser):
                parser.addoption("--hello", "-X", dest="hello")
        """
        )
        config = pytester.parseconfig("--hello=this")
        for x in ("hello", "--hello", "-X"):
            assert config.getoption(x) == "this"
        pytest.raises(ValueError, config.getoption, "qweqwe")

    def test_config_getoption_unicode(self, pytester: Pytester) -> None:
        pytester.makeconftest(
            """
            def pytest_addoption(parser):
                parser.addoption('--hello', type=str)
        """
        )
        config = pytester.parseconfig("--hello=this")
        assert config.getoption("hello") == "this"

    def test_config_getvalueorskip(self, pytester: Pytester) -> None:
        config = pytester.parseconfig()
        pytest.raises(pytest.skip.Exception, config.getvalueorskip, "hello")
        verbose = config.getvalueorskip("verbose")
        assert verbose == config.option.verbose

    def test_config_getvalueorskip_None(self, pytester: Pytester) -> None:
        pytester.makeconftest(
            """
            def pytest_addoption(parser):
                parser.addoption("--hello")
        """
        )
        config = pytester.parseconfig()
        with pytest.raises(pytest.skip.Exception):
            config.getvalueorskip("hello")

    def test_getoption(self, pytester: Pytester) -> None:
        config = pytester.parseconfig()
        with pytest.raises(ValueError):
            config.getvalue("x")
        assert config.getoption("x", 1) == 1

    def test_getconftest_pathlist(self, pytester: Pytester, tmp_path: Path) -> None:
        somepath = tmp_path.joinpath("x", "y", "z")
        p = tmp_path.joinpath("conftest.py")
        p.write_text(f"mylist = {['.', str(somepath)]}", encoding="utf-8")
        config = pytester.parseconfigure(p)
        assert config._getconftest_pathlist("notexist", path=tmp_path) is None
        assert config._getconftest_pathlist("mylist", path=tmp_path) == [
            tmp_path,
            somepath,
        ]

    @pytest.mark.parametrize("maybe_type", ["not passed", "None", '"string"'])
    def test_addini(self, pytester: Pytester, maybe_type: str) -> None:
        if maybe_type == "not passed":
            type_string = ""
        else:
            type_string = f", {maybe_type}"

        pytester.makeconftest(
            f"""
            def pytest_addoption(parser):
                parser.addini("myname", "my new ini value"{type_string})
        """
        )
        pytester.makeini(
            """
            [pytest]
            myname=hello
        """
        )
        config = pytester.parseconfig()
        val = config.getini("myname")
        assert val == "hello"
        pytest.raises(ValueError, config.getini, "other")

    @pytest.mark.parametrize("config_type", ["ini", "pyproject"])
    def test_addini_paths(self, pytester: Pytester, config_type: str) -> None:
        pytester.makeconftest(
            """
            def pytest_addoption(parser):
                parser.addini("paths", "my new ini value", type="paths")
                parser.addini("abc", "abc value")
        """
        )
        if config_type == "ini":
            inipath = pytester.makeini(
                """
                [pytest]
                paths=hello world/sub.py
            """
            )
        elif config_type == "pyproject":
            inipath = pytester.makepyprojecttoml(
                """
                [tool.pytest.ini_options]
                paths=["hello", "world/sub.py"]
            """
            )
        config = pytester.parseconfig()
        values = config.getini("paths")
        assert len(values) == 2
        assert values[0] == inipath.parent.joinpath("hello")
        assert values[1] == inipath.parent.joinpath("world/sub.py")
        pytest.raises(ValueError, config.getini, "other")

    def make_conftest_for_args(self, pytester: Pytester) -> None:
        pytester.makeconftest(
            """
            def pytest_addoption(parser):
                parser.addini("args", "new args", type="args")
                parser.addini("a2", "", "args", default="1 2 3".split())
        """
        )

    def test_addini_args_ini_files(self, pytester: Pytester) -> None:
        self.make_conftest_for_args(pytester)
        pytester.makeini(
            """
            [pytest]
            args=123 "123 hello" "this"
            """
        )
        self.check_config_args(pytester)

    def test_addini_args_pyproject_toml(self, pytester: Pytester) -> None:
        self.make_conftest_for_args(pytester)
        pytester.makepyprojecttoml(
            """
            [tool.pytest.ini_options]
            args = ["123", "123 hello", "this"]
            """
        )
        self.check_config_args(pytester)

    def check_config_args(self, pytester: Pytester) -> None:
        config = pytester.parseconfig()
        values = config.getini("args")
        assert values == ["123", "123 hello", "this"]
        values = config.getini("a2")
        assert values == list("123")

    def make_conftest_for_linelist(self, pytester: Pytester) -> None:
        pytester.makeconftest(
            """
            def pytest_addoption(parser):
                parser.addini("xy", "", type="linelist")
                parser.addini("a2", "", "linelist")
        """
        )

    def test_addini_linelist_ini_files(self, pytester: Pytester) -> None:
        self.make_conftest_for_linelist(pytester)
        pytester.makeini(
            """
            [pytest]
            xy= 123 345
                second line
        """
        )
        self.check_config_linelist(pytester)

    def test_addini_linelist_pprojecttoml(self, pytester: Pytester) -> None:
        self.make_conftest_for_linelist(pytester)
        pytester.makepyprojecttoml(
            """
            [tool.pytest.ini_options]
            xy = ["123 345", "second line"]
        """
        )
        self.check_config_linelist(pytester)

    def check_config_linelist(self, pytester: Pytester) -> None:
        config = pytester.parseconfig()
        values = config.getini("xy")
        assert len(values) == 2
        assert values == ["123 345", "second line"]
        values = config.getini("a2")
        assert values == []

    @pytest.mark.parametrize(
        "str_val, bool_val", [("True", True), ("no", False), ("no-ini", True)]
    )
    def test_addini_bool(
        self, pytester: Pytester, str_val: str, bool_val: bool
    ) -> None:
        pytester.makeconftest(
            """
            def pytest_addoption(parser):
                parser.addini("strip", "", type="bool", default=True)
        """
        )
        if str_val != "no-ini":
            pytester.makeini(
                """
                [pytest]
                strip=%s
            """
                % str_val
            )
        config = pytester.parseconfig()
        assert config.getini("strip") is bool_val

    def test_addinivalue_line_existing(self, pytester: Pytester) -> None:
        pytester.makeconftest(
            """
            def pytest_addoption(parser):
                parser.addini("xy", "", type="linelist")
        """
        )
        pytester.makeini(
            """
            [pytest]
            xy= 123
        """
        )
        config = pytester.parseconfig()
        values = config.getini("xy")
        assert len(values) == 1
        assert values == ["123"]
        config.addinivalue_line("xy", "456")
        values = config.getini("xy")
        assert len(values) == 2
        assert values == ["123", "456"]

    def test_addinivalue_line_new(self, pytester: Pytester) -> None:
        pytester.makeconftest(
            """
            def pytest_addoption(parser):
                parser.addini("xy", "", type="linelist")
        """
        )
        config = pytester.parseconfig()
        assert not config.getini("xy")
        config.addinivalue_line("xy", "456")
        values = config.getini("xy")
        assert len(values) == 1
        assert values == ["456"]
        config.addinivalue_line("xy", "123")
        values = config.getini("xy")
        assert len(values) == 2
        assert values == ["456", "123"]

    def test_addini_default_values(self, pytester: Pytester) -> None:
        """Tests the default values for configuration based on
        config type
        """
        pytester.makeconftest(
            """
            def pytest_addoption(parser):
                parser.addini("linelist1", "", type="linelist")
                parser.addini("paths1", "", type="paths")
                parser.addini("pathlist1", "", type="pathlist")
                parser.addini("args1", "", type="args")
                parser.addini("bool1", "", type="bool")
                parser.addini("string1", "", type="string")
                parser.addini("none_1", "", type="linelist", default=None)
                parser.addini("none_2", "", default=None)
                parser.addini("no_type", "")
        """
        )

        config = pytester.parseconfig()
        # default for linelist, paths, pathlist and args is []
        value = config.getini("linelist1")
        assert value == []
        value = config.getini("paths1")
        assert value == []
        value = config.getini("pathlist1")
        assert value == []
        value = config.getini("args1")
        assert value == []
        # default for bool is False
        value = config.getini("bool1")
        assert value is False
        # default for string is ""
        value = config.getini("string1")
        assert value == ""
        # should return None if None is explicity set as default value
        # irrespective of the type argument
        value = config.getini("none_1")
        assert value is None
        value = config.getini("none_2")
        assert value is None
        # in case no type is provided and no default set
        # treat it as string and default value will be ""
        value = config.getini("no_type")
        assert value == ""

    @pytest.mark.parametrize(
        "type, expected",
        [
            pytest.param(None, "", id="None"),
            pytest.param("string", "", id="string"),
            pytest.param("paths", [], id="paths"),
            pytest.param("pathlist", [], id="pathlist"),
            pytest.param("args", [], id="args"),
            pytest.param("linelist", [], id="linelist"),
            pytest.param("bool", False, id="bool"),
        ],
    )
    def test_get_ini_default_for_type(self, type: Any, expected: Any) -> None:
        assert get_ini_default_for_type(type) == expected

    def test_confcutdir_check_isdir(self, pytester: Pytester) -> None:
        """Give an error if --confcutdir is not a valid directory (#2078)"""
        exp_match = r"^--confcutdir must be a directory, given: "
        with pytest.raises(pytest.UsageError, match=exp_match):
            pytester.parseconfig("--confcutdir", pytester.path.joinpath("file"))
        with pytest.raises(pytest.UsageError, match=exp_match):
            pytester.parseconfig("--confcutdir", pytester.path.joinpath("nonexistent"))

        p = pytester.mkdir("dir")
        config = pytester.parseconfig("--confcutdir", p)
        assert config.getoption("confcutdir") == str(p)

    @pytest.mark.parametrize(
        "names, expected",
        [
            # dist-info based distributions root are files as will be put in PYTHONPATH
            (["bar.py"], ["bar"]),
            (["foo/bar.py"], ["bar"]),
            (["foo/bar.pyc"], []),
            (["foo/__init__.py"], ["foo"]),
            (["bar/__init__.py", "xz.py"], ["bar", "xz"]),
            (["setup.py"], []),
            # egg based distributions root contain the files from the dist root
            (["src/bar/__init__.py"], ["bar"]),
            (["src/bar/__init__.py", "setup.py"], ["bar"]),
            (["source/python/bar/__init__.py", "setup.py"], ["bar"]),
            # editable installation finder modules
            (["__editable___xyz_finder.py"], []),
            (["bar/__init__.py", "__editable___xyz_finder.py"], ["bar"]),
        ],
    )
    def test_iter_rewritable_modules(self, names, expected) -> None:
        assert list(_iter_rewritable_modules(names)) == expected


class TestConfigFromdictargs:
    def test_basic_behavior(self, _sys_snapshot) -> None:
        option_dict = {"verbose": 444, "foo": "bar", "capture": "no"}
        args = ["a", "b"]

        config = Config.fromdictargs(option_dict, args)
        with pytest.raises(AssertionError):
            config.parse(["should refuse to parse again"])
        assert config.option.verbose == 444
        assert config.option.foo == "bar"
        assert config.option.capture == "no"
        assert config.args == args

    def test_invocation_params_args(self, _sys_snapshot) -> None:
        """Show that fromdictargs can handle args in their "orig" format"""
        option_dict: Dict[str, object] = {}
        args = ["-vvvv", "-s", "a", "b"]

        config = Config.fromdictargs(option_dict, args)
        assert config.args == ["a", "b"]
        assert config.invocation_params.args == tuple(args)
        assert config.option.verbose == 4
        assert config.option.capture == "no"

    def test_inifilename(self, tmp_path: Path) -> None:
        d1 = tmp_path.joinpath("foo")
        d1.mkdir()
        p1 = d1.joinpath("bar.ini")
        p1.touch()
        p1.write_text(
            textwrap.dedent(
                """\
                [pytest]
                name = value
                """
            ),
            encoding="utf-8",
        )

        inifilename = "../../foo/bar.ini"
        option_dict = {"inifilename": inifilename, "capture": "no"}

        cwd = tmp_path.joinpath("a/b")
        cwd.mkdir(parents=True)
        p2 = cwd.joinpath("pytest.ini")
        p2.touch()
        p2.write_text(
            textwrap.dedent(
                """\
                [pytest]
                name = wrong-value
                should_not_be_set = true
                """
            ),
            encoding="utf-8",
        )
        with MonkeyPatch.context() as mp:
            mp.chdir(cwd)
            config = Config.fromdictargs(option_dict, ())
            inipath = absolutepath(inifilename)

        assert config.args == [str(cwd)]
        assert config.option.inifilename == inifilename
        assert config.option.capture == "no"

        # this indicates this is the file used for getting configuration values
        assert config.inipath == inipath
        assert config.inicfg.get("name") == "value"
        assert config.inicfg.get("should_not_be_set") is None


def test_options_on_small_file_do_not_blow_up(pytester: Pytester) -> None:
    def runfiletest(opts: Sequence[str]) -> None:
        reprec = pytester.inline_run(*opts)
        passed, skipped, failed = reprec.countoutcomes()
        assert failed == 2
        assert skipped == passed == 0

    path = str(
        pytester.makepyfile(
            """
        def test_f1(): assert 0
        def test_f2(): assert 0
    """
        )
    )

    runfiletest([path])
    runfiletest(["-l", path])
    runfiletest(["-s", path])
    runfiletest(["--tb=no", path])
    runfiletest(["--tb=short", path])
    runfiletest(["--tb=long", path])
    runfiletest(["--fulltrace", path])
    runfiletest(["--traceconfig", path])
    runfiletest(["-v", path])
    runfiletest(["-v", "-v", path])


def test_preparse_ordering_with_setuptools(
    pytester: Pytester, monkeypatch: MonkeyPatch
) -> None:
    monkeypatch.delenv("PYTEST_DISABLE_PLUGIN_AUTOLOAD", raising=False)

    class EntryPoint:
        name = "mytestplugin"
        group = "pytest11"

        def load(self):
            class PseudoPlugin:
                x = 42

            return PseudoPlugin()

    class Dist:
        files = ()
        metadata = {"name": "foo"}
        entry_points = (EntryPoint(),)

    def my_dists():
        return (Dist,)

    monkeypatch.setattr(importlib.metadata, "distributions", my_dists)
    pytester.makeconftest(
        """
        pytest_plugins = "mytestplugin",
    """
    )
    monkeypatch.setenv("PYTEST_PLUGINS", "mytestplugin")
    config = pytester.parseconfig()
    plugin = config.pluginmanager.getplugin("mytestplugin")
    assert plugin.x == 42


def test_setuptools_importerror_issue1479(
    pytester: Pytester, monkeypatch: MonkeyPatch
) -> None:
    monkeypatch.delenv("PYTEST_DISABLE_PLUGIN_AUTOLOAD", raising=False)

    class DummyEntryPoint:
        name = "mytestplugin"
        group = "pytest11"

        def load(self):
            raise ImportError("Don't hide me!")

    class Distribution:
        version = "1.0"
        files = ("foo.txt",)
        metadata = {"name": "foo"}
        entry_points = (DummyEntryPoint(),)

    def distributions():
        return (Distribution(),)

    monkeypatch.setattr(importlib.metadata, "distributions", distributions)
    with pytest.raises(ImportError):
        pytester.parseconfig()


def test_importlib_metadata_broken_distribution(
    pytester: Pytester, monkeypatch: MonkeyPatch
) -> None:
    """Integration test for broken distributions with 'files' metadata being None (#5389)"""
    monkeypatch.delenv("PYTEST_DISABLE_PLUGIN_AUTOLOAD", raising=False)

    class DummyEntryPoint:
        name = "mytestplugin"
        group = "pytest11"

        def load(self):
            return object()

    class Distribution:
        version = "1.0"
        files = None
        metadata = {"name": "foo"}
        entry_points = (DummyEntryPoint(),)

    def distributions():
        return (Distribution(),)

    monkeypatch.setattr(importlib.metadata, "distributions", distributions)
    pytester.parseconfig()


@pytest.mark.parametrize("block_it", [True, False])
def test_plugin_preparse_prevents_setuptools_loading(
    pytester: Pytester, monkeypatch: MonkeyPatch, block_it: bool
) -> None:
    monkeypatch.delenv("PYTEST_DISABLE_PLUGIN_AUTOLOAD", raising=False)

    plugin_module_placeholder = object()

    class DummyEntryPoint:
        name = "mytestplugin"
        group = "pytest11"

        def load(self):
            return plugin_module_placeholder

    class Distribution:
        version = "1.0"
        files = ("foo.txt",)
        metadata = {"name": "foo"}
        entry_points = (DummyEntryPoint(),)

    def distributions():
        return (Distribution(),)

    monkeypatch.setattr(importlib.metadata, "distributions", distributions)
    args = ("-p", "no:mytestplugin") if block_it else ()
    config = pytester.parseconfig(*args)
    config.pluginmanager.import_plugin("mytestplugin")
    if block_it:
        assert "mytestplugin" not in sys.modules
        assert config.pluginmanager.get_plugin("mytestplugin") is None
    else:
        assert (
            config.pluginmanager.get_plugin("mytestplugin") is plugin_module_placeholder
        )


@pytest.mark.parametrize(
    "parse_args,should_load", [(("-p", "mytestplugin"), True), ((), False)]
)
def test_disable_plugin_autoload(
    pytester: Pytester,
    monkeypatch: MonkeyPatch,
    parse_args: Union[Tuple[str, str], Tuple[()]],
    should_load: bool,
) -> None:
    class DummyEntryPoint:
        project_name = name = "mytestplugin"
        group = "pytest11"
        version = "1.0"

        def load(self):
            return sys.modules[self.name]

    class Distribution:
        metadata = {"name": "foo"}
        entry_points = (DummyEntryPoint(),)
        files = ()

    class PseudoPlugin:
        x = 42

        attrs_used = []

        def __getattr__(self, name):
            assert name == "__loader__"
            self.attrs_used.append(name)
            return object()

    def distributions():
        return (Distribution(),)

    monkeypatch.setenv("PYTEST_DISABLE_PLUGIN_AUTOLOAD", "1")
    monkeypatch.setattr(importlib.metadata, "distributions", distributions)
    monkeypatch.setitem(sys.modules, "mytestplugin", PseudoPlugin())
    config = pytester.parseconfig(*parse_args)
    has_loaded = config.pluginmanager.get_plugin("mytestplugin") is not None
    assert has_loaded == should_load
    if should_load:
        assert PseudoPlugin.attrs_used == ["__loader__"]
    else:
        assert PseudoPlugin.attrs_used == []


def test_plugin_loading_order(pytester: Pytester) -> None:
    """Test order of plugin loading with `-p`."""
    p1 = pytester.makepyfile(
        """
        def test_terminal_plugin(request):
            import myplugin
            assert myplugin.terminal_plugin == [False, True]
        """,
        myplugin="""
            terminal_plugin = []

            def pytest_configure(config):
                terminal_plugin.append(bool(config.pluginmanager.get_plugin("terminalreporter")))

            def pytest_sessionstart(session):
                config = session.config
                terminal_plugin.append(bool(config.pluginmanager.get_plugin("terminalreporter")))
            """,
    )
    pytester.syspathinsert()
    result = pytester.runpytest("-p", "myplugin", str(p1))
    assert result.ret == 0


def test_invalid_options_show_extra_information(pytester: Pytester) -> None:
    """Display extra information when pytest exits due to unrecognized
    options in the command-line."""
    pytester.makeini(
        """
        [pytest]
        addopts = --invalid-option
    """
    )
    result = pytester.runpytest()
    result.stderr.fnmatch_lines(
        [
            "*error: unrecognized arguments: --invalid-option*",
            "*  inifile: %s*" % pytester.path.joinpath("tox.ini"),
            "*  rootdir: %s*" % pytester.path,
        ]
    )


@pytest.mark.parametrize(
    "args",
    [
        ["dir1", "dir2", "-v"],
        ["dir1", "-v", "dir2"],
        ["dir2", "-v", "dir1"],
        ["-v", "dir2", "dir1"],
    ],
)
def test_consider_args_after_options_for_rootdir(
    pytester: Pytester, args: List[str]
) -> None:
    """
    Consider all arguments in the command-line for rootdir
    discovery, even if they happen to occur after an option. #949
    """
    # replace "dir1" and "dir2" from "args" into their real directory
    root = pytester.mkdir("myroot")
    d1 = root.joinpath("dir1")
    d1.mkdir()
    d2 = root.joinpath("dir2")
    d2.mkdir()
    for i, arg in enumerate(args):
        if arg == "dir1":
            args[i] = str(d1)
        elif arg == "dir2":
            args[i] = str(d2)
    with MonkeyPatch.context() as mp:
        mp.chdir(root)
        result = pytester.runpytest(*args)
    result.stdout.fnmatch_lines(["*rootdir: *myroot"])


def test_toolongargs_issue224(pytester: Pytester) -> None:
    result = pytester.runpytest("-m", "hello" * 500)
    assert result.ret == ExitCode.NO_TESTS_COLLECTED


def test_config_in_subdirectory_colon_command_line_issue2148(
    pytester: Pytester,
) -> None:
    conftest_source = """
        def pytest_addoption(parser):
            parser.addini('foo', 'foo')
    """

    pytester.makefile(
        ".ini",
        **{"pytest": "[pytest]\nfoo = root", "subdir/pytest": "[pytest]\nfoo = subdir"},
    )

    pytester.makepyfile(
        **{
            "conftest": conftest_source,
            "subdir/conftest": conftest_source,
            "subdir/test_foo": """\
            def test_foo(pytestconfig):
                assert pytestconfig.getini('foo') == 'subdir'
            """,
        }
    )

    result = pytester.runpytest("subdir/test_foo.py::test_foo")
    assert result.ret == 0


def test_notify_exception(pytester: Pytester, capfd) -> None:
    config = pytester.parseconfig()
    with pytest.raises(ValueError) as excinfo:
        raise ValueError(1)
    config.notify_exception(excinfo, config.option)
    _, err = capfd.readouterr()
    assert "ValueError" in err

    class A:
        def pytest_internalerror(self):
            return True

    config.pluginmanager.register(A())
    config.notify_exception(excinfo, config.option)
    _, err = capfd.readouterr()
    assert not err

    config = pytester.parseconfig("-p", "no:terminal")
    with pytest.raises(ValueError) as excinfo:
        raise ValueError(1)
    config.notify_exception(excinfo, config.option)
    _, err = capfd.readouterr()
    assert "ValueError" in err


def test_no_terminal_discovery_error(pytester: Pytester) -> None:
    pytester.makepyfile("raise TypeError('oops!')")
    result = pytester.runpytest("-p", "no:terminal", "--collect-only")
    assert result.ret == ExitCode.INTERRUPTED


def test_load_initial_conftest_last_ordering(_config_for_test):
    pm = _config_for_test.pluginmanager

    class My:
        def pytest_load_initial_conftests(self):
            pass

    m = My()
    pm.register(m)
    hc = pm.hook.pytest_load_initial_conftests
    hookimpls = [
        (
            hookimpl.function.__module__,
            "wrapper" if (hookimpl.wrapper or hookimpl.hookwrapper) else "nonwrapper",
        )
        for hookimpl in hc.get_hookimpls()
    ]
    assert hookimpls == [
        ("_pytest.config", "nonwrapper"),
        (m.__module__, "nonwrapper"),
        ("_pytest.legacypath", "nonwrapper"),
        ("_pytest.python_path", "nonwrapper"),
        ("_pytest.capture", "wrapper"),
        ("_pytest.warnings", "wrapper"),
    ]


def test_get_plugin_specs_as_list() -> None:
    def exp_match(val: object) -> str:
        return (
            "Plugins may be specified as a sequence or a ','-separated string of plugin names. Got: %s"
            % re.escape(repr(val))
        )

    with pytest.raises(pytest.UsageError, match=exp_match({"foo"})):
        _get_plugin_specs_as_list({"foo"})  # type: ignore[arg-type]
    with pytest.raises(pytest.UsageError, match=exp_match({})):
        _get_plugin_specs_as_list(dict())  # type: ignore[arg-type]

    assert _get_plugin_specs_as_list(None) == []
    assert _get_plugin_specs_as_list("") == []
    assert _get_plugin_specs_as_list("foo") == ["foo"]
    assert _get_plugin_specs_as_list("foo,bar") == ["foo", "bar"]
    assert _get_plugin_specs_as_list(["foo", "bar"]) == ["foo", "bar"]
    assert _get_plugin_specs_as_list(("foo", "bar")) == ["foo", "bar"]


def test_collect_pytest_prefix_bug_integration(pytester: Pytester) -> None:
    """Integration test for issue #3775"""
    p = pytester.copy_example("config/collect_pytest_prefix")
    result = pytester.runpytest(p)
    result.stdout.fnmatch_lines(["* 1 passed *"])


def test_collect_pytest_prefix_bug(pytestconfig):
    """Ensure we collect only actual functions from conftest files (#3775)"""

    class Dummy:
        class pytest_something:
            pass

    pm = pytestconfig.pluginmanager
    assert pm.parse_hookimpl_opts(Dummy(), "pytest_something") is None


class TestRootdir:
    def test_simple_noini(self, tmp_path: Path, monkeypatch: MonkeyPatch) -> None:
        assert get_common_ancestor(Path.cwd(), [tmp_path]) == tmp_path
        a = tmp_path / "a"
        a.mkdir()
        assert get_common_ancestor(Path.cwd(), [a, tmp_path]) == tmp_path
        assert get_common_ancestor(Path.cwd(), [tmp_path, a]) == tmp_path
        monkeypatch.chdir(tmp_path)
        assert get_common_ancestor(Path.cwd(), []) == tmp_path
        no_path = tmp_path / "does-not-exist"
        assert get_common_ancestor(Path.cwd(), [no_path]) == tmp_path
        assert get_common_ancestor(Path.cwd(), [no_path / "a"]) == tmp_path

    @pytest.mark.parametrize(
        "name, contents",
        [
            pytest.param("pytest.ini", "[pytest]\nx=10", id="pytest.ini"),
            pytest.param(
                "pyproject.toml", "[tool.pytest.ini_options]\nx=10", id="pyproject.toml"
            ),
            pytest.param("tox.ini", "[pytest]\nx=10", id="tox.ini"),
            pytest.param("setup.cfg", "[tool:pytest]\nx=10", id="setup.cfg"),
        ],
    )
    def test_with_ini(self, tmp_path: Path, name: str, contents: str) -> None:
        inipath = tmp_path / name
        inipath.write_text(contents, encoding="utf-8")

        a = tmp_path / "a"
        a.mkdir()
        b = a / "b"
        b.mkdir()
        for args in ([str(tmp_path)], [str(a)], [str(b)]):
            rootpath, parsed_inipath, _ = determine_setup(
                inifile=None,
                args=args,
                rootdir_cmd_arg=None,
                invocation_dir=Path.cwd(),
            )
            assert rootpath == tmp_path
            assert parsed_inipath == inipath
        rootpath, parsed_inipath, ini_config = determine_setup(
            inifile=None,
            args=[str(b), str(a)],
            rootdir_cmd_arg=None,
            invocation_dir=Path.cwd(),
        )
        assert rootpath == tmp_path
        assert parsed_inipath == inipath
        assert ini_config == {"x": "10"}

    @pytest.mark.parametrize("name", ["setup.cfg", "tox.ini"])
    def test_pytestini_overrides_empty_other(self, tmp_path: Path, name: str) -> None:
        inipath = tmp_path / "pytest.ini"
        inipath.touch()
        a = tmp_path / "a"
        a.mkdir()
        (a / name).touch()
        rootpath, parsed_inipath, _ = determine_setup(
            inifile=None,
            args=[str(a)],
            rootdir_cmd_arg=None,
            invocation_dir=Path.cwd(),
        )
        assert rootpath == tmp_path
        assert parsed_inipath == inipath

    def test_setuppy_fallback(self, tmp_path: Path) -> None:
        a = tmp_path / "a"
        a.mkdir()
        (a / "setup.cfg").touch()
        (tmp_path / "setup.py").touch()
        rootpath, inipath, inicfg = determine_setup(
            inifile=None,
            args=[str(a)],
            rootdir_cmd_arg=None,
            invocation_dir=Path.cwd(),
        )
        assert rootpath == tmp_path
        assert inipath is None
        assert inicfg == {}

    def test_nothing(self, tmp_path: Path, monkeypatch: MonkeyPatch) -> None:
        monkeypatch.chdir(tmp_path)
        rootpath, inipath, inicfg = determine_setup(
            inifile=None,
            args=[str(tmp_path)],
            rootdir_cmd_arg=None,
            invocation_dir=Path.cwd(),
        )
        assert rootpath == tmp_path
        assert inipath is None
        assert inicfg == {}

    @pytest.mark.parametrize(
        "name, contents",
        [
            # pytest.param("pytest.ini", "[pytest]\nx=10", id="pytest.ini"),
            pytest.param(
                "pyproject.toml", "[tool.pytest.ini_options]\nx=10", id="pyproject.toml"
            ),
            # pytest.param("tox.ini", "[pytest]\nx=10", id="tox.ini"),
            # pytest.param("setup.cfg", "[tool:pytest]\nx=10", id="setup.cfg"),
        ],
    )
    def test_with_specific_inifile(
        self, tmp_path: Path, name: str, contents: str
    ) -> None:
        p = tmp_path / name
        p.touch()
        p.write_text(contents, encoding="utf-8")
        rootpath, inipath, ini_config = determine_setup(
            inifile=str(p),
            args=[str(tmp_path)],
            rootdir_cmd_arg=None,
            invocation_dir=Path.cwd(),
        )
        assert rootpath == tmp_path
        assert inipath == p
        assert ini_config == {"x": "10"}

    def test_explicit_config_file_sets_rootdir(
        self, tmp_path: Path, monkeypatch: pytest.MonkeyPatch
    ) -> None:
        tests_dir = tmp_path / "tests"
        tests_dir.mkdir()

        monkeypatch.chdir(tmp_path)

        # No config file is explicitly given: rootdir is determined to be cwd.
        rootpath, found_inipath, *_ = determine_setup(
            inifile=None,
            args=[str(tests_dir)],
            rootdir_cmd_arg=None,
            invocation_dir=Path.cwd(),
        )
        assert rootpath == tmp_path
        assert found_inipath is None

        # Config file is explicitly given: rootdir is determined to be inifile's directory.
        inipath = tmp_path / "pytest.ini"
        inipath.touch()
        rootpath, found_inipath, *_ = determine_setup(
            inifile=str(inipath),
            args=[str(tests_dir)],
            rootdir_cmd_arg=None,
            invocation_dir=Path.cwd(),
        )
        assert rootpath == tmp_path
        assert found_inipath == inipath

    def test_with_arg_outside_cwd_without_inifile(
        self, tmp_path: Path, monkeypatch: MonkeyPatch
    ) -> None:
        monkeypatch.chdir(tmp_path)
        a = tmp_path / "a"
        a.mkdir()
        b = tmp_path / "b"
        b.mkdir()
        rootpath, inifile, _ = determine_setup(
            inifile=None,
            args=[str(a), str(b)],
            rootdir_cmd_arg=None,
            invocation_dir=Path.cwd(),
        )
        assert rootpath == tmp_path
        assert inifile is None

    def test_with_arg_outside_cwd_with_inifile(self, tmp_path: Path) -> None:
        a = tmp_path / "a"
        a.mkdir()
        b = tmp_path / "b"
        b.mkdir()
        inipath = a / "pytest.ini"
        inipath.touch()
        rootpath, parsed_inipath, _ = determine_setup(
            inifile=None,
            args=[str(a), str(b)],
            rootdir_cmd_arg=None,
            invocation_dir=Path.cwd(),
        )
        assert rootpath == a
        assert inipath == parsed_inipath

    @pytest.mark.parametrize("dirs", ([], ["does-not-exist"], ["a/does-not-exist"]))
    def test_with_non_dir_arg(
        self, dirs: Sequence[str], tmp_path: Path, monkeypatch: MonkeyPatch
    ) -> None:
        monkeypatch.chdir(tmp_path)
        rootpath, inipath, _ = determine_setup(
            inifile=None,
            args=dirs,
            rootdir_cmd_arg=None,
            invocation_dir=Path.cwd(),
        )
        assert rootpath == tmp_path
        assert inipath is None

    def test_with_existing_file_in_subdir(
        self, tmp_path: Path, monkeypatch: MonkeyPatch
    ) -> None:
        a = tmp_path / "a"
        a.mkdir()
        (a / "exists").touch()
        monkeypatch.chdir(tmp_path)
        rootpath, inipath, _ = determine_setup(
            inifile=None,
            args=["a/exist"],
            rootdir_cmd_arg=None,
            invocation_dir=Path.cwd(),
        )
        assert rootpath == tmp_path
        assert inipath is None

    def test_with_config_also_in_parent_directory(
        self, tmp_path: Path, monkeypatch: MonkeyPatch
    ) -> None:
        """Regression test for #7807."""
        (tmp_path / "setup.cfg").write_text("[tool:pytest]\n", "utf-8")
        (tmp_path / "myproject").mkdir()
        (tmp_path / "myproject" / "setup.cfg").write_text("[tool:pytest]\n", "utf-8")
        (tmp_path / "myproject" / "tests").mkdir()
        monkeypatch.chdir(tmp_path / "myproject")

        rootpath, inipath, _ = determine_setup(
            inifile=None,
            args=["tests/"],
            rootdir_cmd_arg=None,
            invocation_dir=Path.cwd(),
        )

        assert rootpath == tmp_path / "myproject"
        assert inipath == tmp_path / "myproject" / "setup.cfg"


class TestOverrideIniArgs:
    @pytest.mark.parametrize("name", "setup.cfg tox.ini pytest.ini".split())
    def test_override_ini_names(self, pytester: Pytester, name: str) -> None:
        section = "[pytest]" if name != "setup.cfg" else "[tool:pytest]"
        pytester.path.joinpath(name).write_text(
            textwrap.dedent(
                f"""
            {section}
            custom = 1.0"""
            ),
            encoding="utf-8",
        )
        pytester.makeconftest(
            """
            def pytest_addoption(parser):
                parser.addini("custom", "")"""
        )
        pytester.makepyfile(
            """
            def test_pass(pytestconfig):
                ini_val = pytestconfig.getini("custom")
                print('\\ncustom_option:%s\\n' % ini_val)"""
        )

        result = pytester.runpytest("--override-ini", "custom=2.0", "-s")
        assert result.ret == 0
        result.stdout.fnmatch_lines(["custom_option:2.0"])

        result = pytester.runpytest(
            "--override-ini", "custom=2.0", "--override-ini=custom=3.0", "-s"
        )
        assert result.ret == 0
        result.stdout.fnmatch_lines(["custom_option:3.0"])

    def test_override_ini_paths(self, pytester: Pytester) -> None:
        pytester.makeconftest(
            """
            def pytest_addoption(parser):
                parser.addini("paths", "my new ini value", type="paths")"""
        )
        pytester.makeini(
            """
            [pytest]
            paths=blah.py"""
        )
        pytester.makepyfile(
            r"""
            def test_overridden(pytestconfig):
                config_paths = pytestconfig.getini("paths")
                print(config_paths)
                for cpf in config_paths:
                    print('\nuser_path:%s' % cpf.name)
            """
        )
        result = pytester.runpytest(
            "--override-ini", "paths=foo/bar1.py foo/bar2.py", "-s"
        )
        result.stdout.fnmatch_lines(["user_path:bar1.py", "user_path:bar2.py"])

    def test_override_multiple_and_default(self, pytester: Pytester) -> None:
        pytester.makeconftest(
            """
            def pytest_addoption(parser):
                addini = parser.addini
                addini("custom_option_1", "", default="o1")
                addini("custom_option_2", "", default="o2")
                addini("custom_option_3", "", default=False, type="bool")
                addini("custom_option_4", "", default=True, type="bool")"""
        )
        pytester.makeini(
            """
            [pytest]
            custom_option_1=custom_option_1
            custom_option_2=custom_option_2
        """
        )
        pytester.makepyfile(
            """
            def test_multiple_options(pytestconfig):
                prefix = "custom_option"
                for x in range(1, 5):
                    ini_value=pytestconfig.getini("%s_%d" % (prefix, x))
                    print('\\nini%d:%s' % (x, ini_value))
        """
        )
        result = pytester.runpytest(
            "--override-ini",
            "custom_option_1=fulldir=/tmp/user1",
            "-o",
            "custom_option_2=url=/tmp/user2?a=b&d=e",
            "-o",
            "custom_option_3=True",
            "-o",
            "custom_option_4=no",
            "-s",
        )
        result.stdout.fnmatch_lines(
            [
                "ini1:fulldir=/tmp/user1",
                "ini2:url=/tmp/user2?a=b&d=e",
                "ini3:True",
                "ini4:False",
            ]
        )

    def test_override_ini_usage_error_bad_style(self, pytester: Pytester) -> None:
        pytester.makeini(
            """
            [pytest]
            xdist_strict=False
        """
        )
        result = pytester.runpytest("--override-ini", "xdist_strict", "True")
        result.stderr.fnmatch_lines(
            [
                "ERROR: -o/--override-ini expects option=value style (got: 'xdist_strict').",
            ]
        )

    @pytest.mark.parametrize("with_ini", [True, False])
    def test_override_ini_handled_asap(
        self, pytester: Pytester, with_ini: bool
    ) -> None:
        """-o should be handled as soon as possible and always override what's in ini files (#2238)"""
        if with_ini:
            pytester.makeini(
                """
                [pytest]
                python_files=test_*.py
            """
            )
        pytester.makepyfile(
            unittest_ini_handle="""
            def test():
                pass
        """
        )
        result = pytester.runpytest("--override-ini", "python_files=unittest_*.py")
        result.stdout.fnmatch_lines(["*1 passed in*"])

    def test_addopts_before_initini(
        self, monkeypatch: MonkeyPatch, _config_for_test, _sys_snapshot
    ) -> None:
        cache_dir = ".custom_cache"
        monkeypatch.setenv("PYTEST_ADDOPTS", "-o cache_dir=%s" % cache_dir)
        config = _config_for_test
        config._preparse([], addopts=True)
        assert config._override_ini == ["cache_dir=%s" % cache_dir]

    def test_addopts_from_env_not_concatenated(
        self, monkeypatch: MonkeyPatch, _config_for_test
    ) -> None:
        """PYTEST_ADDOPTS should not take values from normal args (#4265)."""
        monkeypatch.setenv("PYTEST_ADDOPTS", "-o")
        config = _config_for_test
        with pytest.raises(UsageError) as excinfo:
            config._preparse(["cache_dir=ignored"], addopts=True)
        assert (
            "error: argument -o/--override-ini: expected one argument (via PYTEST_ADDOPTS)"
            in excinfo.value.args[0]
        )

    def test_addopts_from_ini_not_concatenated(self, pytester: Pytester) -> None:
        """`addopts` from ini should not take values from normal args (#4265)."""
        pytester.makeini(
            """
            [pytest]
            addopts=-o
        """
        )
        result = pytester.runpytest("cache_dir=ignored")
        result.stderr.fnmatch_lines(
            [
                f"{pytester._request.config._parser.optparser.prog}: error: "
                f"argument -o/--override-ini: expected one argument (via addopts config)"
            ]
        )
        assert result.ret == _pytest.config.ExitCode.USAGE_ERROR

    def test_override_ini_does_not_contain_paths(
        self, _config_for_test, _sys_snapshot
    ) -> None:
        """Check that -o no longer swallows all options after it (#3103)"""
        config = _config_for_test
        config._preparse(["-o", "cache_dir=/cache", "/some/test/path"])
        assert config._override_ini == ["cache_dir=/cache"]

    def test_multiple_override_ini_options(self, pytester: Pytester) -> None:
        """Ensure a file path following a '-o' option does not generate an error (#3103)"""
        pytester.makepyfile(
            **{
                "conftest.py": """
                def pytest_addoption(parser):
                    parser.addini('foo', default=None, help='some option')
                    parser.addini('bar', default=None, help='some option')
            """,
                "test_foo.py": """
                def test(pytestconfig):
                    assert pytestconfig.getini('foo') == '1'
                    assert pytestconfig.getini('bar') == '0'
            """,
                "test_bar.py": """
                def test():
                    assert False
            """,
            }
        )
        result = pytester.runpytest("-o", "foo=1", "-o", "bar=0", "test_foo.py")
        assert "ERROR:" not in result.stderr.str()
        result.stdout.fnmatch_lines(["collected 1 item", "*= 1 passed in *="])

    def test_override_ini_without_config_file(self, pytester: Pytester) -> None:
        pytester.makepyfile(**{"src/override_ini_without_config_file.py": ""})
        pytester.makepyfile(
            **{
                "tests/test_override_ini_without_config_file.py": (
                    "import override_ini_without_config_file\ndef test(): pass"
                ),
            }
        )
        result = pytester.runpytest("--override-ini", "pythonpath=src")
        assert result.parseoutcomes() == {"passed": 1}


def test_help_via_addopts(pytester: Pytester) -> None:
    pytester.makeini(
        """
        [pytest]
        addopts = --unknown-option-should-allow-for-help --help
    """
    )
    result = pytester.runpytest()
    assert result.ret == 0
    result.stdout.fnmatch_lines(
        [
            "usage: *",
            "positional arguments:",
            # Displays full/default help.
            "to see available markers type: pytest --markers",
        ]
    )


def test_help_and_version_after_argument_error(pytester: Pytester) -> None:
    pytester.makeconftest(
        """
        def validate(arg):
            raise argparse.ArgumentTypeError("argerror")

        def pytest_addoption(parser):
            group = parser.getgroup('cov')
            group.addoption(
                "--invalid-option-should-allow-for-help",
                type=validate,
            )
        """
    )
    pytester.makeini(
        """
        [pytest]
        addopts = --invalid-option-should-allow-for-help
    """
    )
    result = pytester.runpytest("--help")
    result.stdout.fnmatch_lines(
        [
            "usage: *",
            "positional arguments:",
            "NOTE: displaying only minimal help due to UsageError.",
        ]
    )
    result.stderr.fnmatch_lines(
        [
            "ERROR: usage: *",
            f"{pytester._request.config._parser.optparser.prog}: error: "
            f"argument --invalid-option-should-allow-for-help: expected one argument",
        ]
    )
    # Does not display full/default help.
    assert "to see available markers type: pytest --markers" not in result.stdout.lines
    assert result.ret == ExitCode.USAGE_ERROR

    result = pytester.runpytest("--version")
    result.stdout.fnmatch_lines([f"pytest {pytest.__version__}"])
    assert result.ret == ExitCode.USAGE_ERROR


def test_help_formatter_uses_py_get_terminal_width(monkeypatch: MonkeyPatch) -> None:
    from _pytest.config.argparsing import DropShorterLongHelpFormatter

    monkeypatch.setenv("COLUMNS", "90")
    formatter = DropShorterLongHelpFormatter("prog")
    assert formatter._width == 90

    monkeypatch.setattr("_pytest._io.get_terminal_width", lambda: 160)
    formatter = DropShorterLongHelpFormatter("prog")
    assert formatter._width == 160

    formatter = DropShorterLongHelpFormatter("prog", width=42)
    assert formatter._width == 42


def test_config_does_not_load_blocked_plugin_from_args(pytester: Pytester) -> None:
    """This tests that pytest's config setup handles "-p no:X"."""
    p = pytester.makepyfile("def test(capfd): pass")
    result = pytester.runpytest(str(p), "-pno:capture")
    result.stdout.fnmatch_lines(["E       fixture 'capfd' not found"])
    assert result.ret == ExitCode.TESTS_FAILED

    result = pytester.runpytest(str(p), "-pno:capture", "-s")
    result.stderr.fnmatch_lines(["*: error: unrecognized arguments: -s"])
    assert result.ret == ExitCode.USAGE_ERROR

    result = pytester.runpytest(str(p), "-p no:capture", "-s")
    result.stderr.fnmatch_lines(["*: error: unrecognized arguments: -s"])
    assert result.ret == ExitCode.USAGE_ERROR


def test_invocation_args(pytester: Pytester) -> None:
    """Ensure that Config.invocation_* arguments are correctly defined"""

    class DummyPlugin:
        pass

    p = pytester.makepyfile("def test(): pass")
    plugin = DummyPlugin()
    rec = pytester.inline_run(p, "-v", plugins=[plugin])
    calls = rec.getcalls("pytest_runtest_protocol")
    assert len(calls) == 1
    call = calls[0]
    config = call.item.config

    assert config.invocation_params.args == (str(p), "-v")
    assert config.invocation_params.dir == pytester.path

    plugins = config.invocation_params.plugins
    assert len(plugins) == 2
    assert plugins[0] is plugin
    assert type(plugins[1]).__name__ == "Collect"  # installed by pytester.inline_run()

    # args cannot be None
    with pytest.raises(TypeError):
        Config.InvocationParams(args=None, plugins=None, dir=Path())  # type: ignore[arg-type]


@pytest.mark.parametrize(
    "plugin",
    [
        x
        for x in _pytest.config.default_plugins
        if x not in _pytest.config.essential_plugins
    ],
)
def test_config_blocked_default_plugins(pytester: Pytester, plugin: str) -> None:
    p = pytester.makepyfile("def test(): pass")
    result = pytester.runpytest(str(p), "-pno:%s" % plugin)

    if plugin == "python":
        assert result.ret == ExitCode.USAGE_ERROR
        result.stderr.fnmatch_lines(
            [
                "ERROR: not found: */test_config_blocked_default_plugins.py",
                "(no match in any of *<Dir *>*",
            ]
        )
        return

    assert result.ret == ExitCode.OK
    if plugin != "terminal":
        result.stdout.fnmatch_lines(["* 1 passed in *"])

    p = pytester.makepyfile("def test(): assert 0")
    result = pytester.runpytest(str(p), "-pno:%s" % plugin)
    assert result.ret == ExitCode.TESTS_FAILED
    if plugin != "terminal":
        result.stdout.fnmatch_lines(["* 1 failed in *"])
    else:
        assert result.stdout.lines == []


class TestSetupCfg:
    def test_pytest_setup_cfg_unsupported(self, pytester: Pytester) -> None:
        pytester.makefile(
            ".cfg",
            setup="""
            [pytest]
            addopts = --verbose
        """,
        )
        with pytest.raises(pytest.fail.Exception):
            pytester.runpytest()

    def test_pytest_custom_cfg_unsupported(self, pytester: Pytester) -> None:
        pytester.makefile(
            ".cfg",
            custom="""
            [pytest]
            addopts = --verbose
        """,
        )
        with pytest.raises(pytest.fail.Exception):
            pytester.runpytest("-c", "custom.cfg")

        with pytest.raises(pytest.fail.Exception):
            pytester.runpytest("--config-file", "custom.cfg")


class TestPytestPluginsVariable:
    def test_pytest_plugins_in_non_top_level_conftest_unsupported(
        self, pytester: Pytester
    ) -> None:
        pytester.makepyfile(
            **{
                "subdirectory/conftest.py": """
            pytest_plugins=['capture']
        """
            }
        )
        pytester.makepyfile(
            """
            def test_func():
                pass
        """
        )
        res = pytester.runpytest()
        assert res.ret == 2
        msg = "Defining 'pytest_plugins' in a non-top-level conftest is no longer supported"
        res.stdout.fnmatch_lines([f"*{msg}*", f"*subdirectory{os.sep}conftest.py*"])

    @pytest.mark.parametrize("use_pyargs", [True, False])
    def test_pytest_plugins_in_non_top_level_conftest_unsupported_pyargs(
        self, pytester: Pytester, use_pyargs: bool
    ) -> None:
        """When using --pyargs, do not emit the warning about non-top-level conftest warnings (#4039, #4044)"""
        files = {
            "src/pkg/__init__.py": "",
            "src/pkg/conftest.py": "",
            "src/pkg/test_root.py": "def test(): pass",
            "src/pkg/sub/__init__.py": "",
            "src/pkg/sub/conftest.py": "pytest_plugins=['capture']",
            "src/pkg/sub/test_bar.py": "def test(): pass",
        }
        pytester.makepyfile(**files)
        pytester.syspathinsert(pytester.path.joinpath("src"))

        args = ("--pyargs", "pkg") if use_pyargs else ()
        res = pytester.runpytest(*args)
        assert res.ret == (0 if use_pyargs else 2)
        msg = "Defining 'pytest_plugins' in a non-top-level conftest is no longer supported"
        if use_pyargs:
            assert msg not in res.stdout.str()
        else:
            res.stdout.fnmatch_lines([f"*{msg}*"])

    def test_pytest_plugins_in_non_top_level_conftest_unsupported_no_top_level_conftest(
        self, pytester: Pytester
    ) -> None:
        subdirectory = pytester.path.joinpath("subdirectory")
        subdirectory.mkdir()
        pytester.makeconftest(
            """
            pytest_plugins=['capture']
        """
        )
        pytester.path.joinpath("conftest.py").rename(
            subdirectory.joinpath("conftest.py")
        )

        pytester.makepyfile(
            """
            def test_func():
                pass
        """
        )

        res = pytester.runpytest_subprocess()
        assert res.ret == 2
        msg = "Defining 'pytest_plugins' in a non-top-level conftest is no longer supported"
        res.stdout.fnmatch_lines([f"*{msg}*", f"*subdirectory{os.sep}conftest.py*"])

    def test_pytest_plugins_in_non_top_level_conftest_unsupported_no_false_positives(
        self, pytester: Pytester
    ) -> None:
        pytester.makepyfile(
            "def test_func(): pass",
            **{
                "subdirectory/conftest": "pass",
                "conftest": """
                    import warnings
                    warnings.filterwarnings('always', category=DeprecationWarning)
                    pytest_plugins=['capture']
                    """,
            },
        )
        res = pytester.runpytest_subprocess()
        assert res.ret == 0
        msg = "Defining 'pytest_plugins' in a non-top-level conftest is no longer supported"
        assert msg not in res.stdout.str()


def test_conftest_import_error_repr(tmp_path: Path) -> None:
    """`ConftestImportFailure` should use a short error message and readable
    path to the failed conftest.py file."""
    path = tmp_path.joinpath("foo/conftest.py")
    with pytest.raises(
        ConftestImportFailure,
        match=re.escape(f"RuntimeError: some error (from {path})"),
    ):
        try:
            raise RuntimeError("some error")
        except Exception as exc:
            raise ConftestImportFailure(path, cause=exc) from exc


def test_strtobool() -> None:
    assert _strtobool("YES")
    assert not _strtobool("NO")
    with pytest.raises(ValueError):
        _strtobool("unknown")


@pytest.mark.parametrize(
    "arg, escape, expected",
    [
        ("ignore", False, ("ignore", "", Warning, "", 0)),
        (
            "ignore::DeprecationWarning",
            False,
            ("ignore", "", DeprecationWarning, "", 0),
        ),
        (
            "ignore:some msg:DeprecationWarning",
            False,
            ("ignore", "some msg", DeprecationWarning, "", 0),
        ),
        (
            "ignore::DeprecationWarning:mod",
            False,
            ("ignore", "", DeprecationWarning, "mod", 0),
        ),
        (
            "ignore::DeprecationWarning:mod:42",
            False,
            ("ignore", "", DeprecationWarning, "mod", 42),
        ),
        ("error:some\\msg:::", True, ("error", "some\\\\msg", Warning, "", 0)),
        ("error:::mod\\foo:", True, ("error", "", Warning, "mod\\\\foo\\Z", 0)),
    ],
)
def test_parse_warning_filter(
    arg: str, escape: bool, expected: Tuple[str, str, Type[Warning], str, int]
) -> None:
    assert parse_warning_filter(arg, escape=escape) == expected


@pytest.mark.parametrize(
    "arg",
    [
        # Too much parts.
        ":" * 5,
        # Invalid action.
        "FOO::",
        # ImportError when importing the warning class.
        "::test_parse_warning_filter_failure.NonExistentClass::",
        # Class is not a Warning subclass.
        "::list::",
        # Negative line number.
        "::::-1",
        # Not a line number.
        "::::not-a-number",
    ],
)
def test_parse_warning_filter_failure(arg: str) -> None:
    with pytest.raises(pytest.UsageError):
        parse_warning_filter(arg, escape=True)


class TestDebugOptions:
    def test_without_debug_does_not_write_log(self, pytester: Pytester) -> None:
        result = pytester.runpytest()
        result.stderr.no_fnmatch_line(
            "*writing pytest debug information to*pytestdebug.log"
        )
        result.stderr.no_fnmatch_line(
            "*wrote pytest debug information to*pytestdebug.log"
        )
        assert not [f.name for f in pytester.path.glob("**/*.log")]

    def test_with_only_debug_writes_pytestdebug_log(self, pytester: Pytester) -> None:
        result = pytester.runpytest("--debug")
        result.stderr.fnmatch_lines(
            [
                "*writing pytest debug information to*pytestdebug.log",
                "*wrote pytest debug information to*pytestdebug.log",
            ]
        )
        assert "pytestdebug.log" in [f.name for f in pytester.path.glob("**/*.log")]

    def test_multiple_custom_debug_logs(self, pytester: Pytester) -> None:
        result = pytester.runpytest("--debug", "bar.log")
        result.stderr.fnmatch_lines(
            [
                "*writing pytest debug information to*bar.log",
                "*wrote pytest debug information to*bar.log",
            ]
        )
        result = pytester.runpytest("--debug", "foo.log")
        result.stderr.fnmatch_lines(
            [
                "*writing pytest debug information to*foo.log",
                "*wrote pytest debug information to*foo.log",
            ]
        )

        assert {"bar.log", "foo.log"} == {
            f.name for f in pytester.path.glob("**/*.log")
        }

    def test_debug_help(self, pytester: Pytester) -> None:
        result = pytester.runpytest("-h")
        result.stdout.fnmatch_lines(
            [
                "*Store internal tracing debug information in this log*",
                "*file. This file is opened with 'w' and truncated as a*",
                "*Default: pytestdebug.log.",
            ]
        )


class TestVerbosity:
    SOME_OUTPUT_TYPE = Config.VERBOSITY_ASSERTIONS
    SOME_OUTPUT_VERBOSITY_LEVEL = 5

    class VerbosityIni:
        def pytest_addoption(self, parser: Parser) -> None:
            Config._add_verbosity_ini(
                parser, TestVerbosity.SOME_OUTPUT_TYPE, help="some help text"
            )

    def test_level_matches_verbose_when_not_specified(
        self, pytester: Pytester, tmp_path: Path
    ) -> None:
        tmp_path.joinpath("pytest.ini").write_text(
            textwrap.dedent(
                """\
                [pytest]
                addopts = --verbose
                """
            ),
            encoding="utf-8",
        )
        pytester.plugins = [TestVerbosity.VerbosityIni()]

        config = pytester.parseconfig(tmp_path)

        assert (
            config.get_verbosity(TestVerbosity.SOME_OUTPUT_TYPE)
            == config.option.verbose
        )

    def test_level_matches_verbose_when_not_known_type(
        self, pytester: Pytester, tmp_path: Path
    ) -> None:
        tmp_path.joinpath("pytest.ini").write_text(
            textwrap.dedent(
                """\
                [pytest]
                addopts = --verbose
                """
            ),
            encoding="utf-8",
        )
        pytester.plugins = [TestVerbosity.VerbosityIni()]

        config = pytester.parseconfig(tmp_path)

        assert config.get_verbosity("some fake verbosity type") == config.option.verbose

    def test_level_matches_specified_override(
        self, pytester: Pytester, tmp_path: Path
    ) -> None:
        setting_name = f"verbosity_{TestVerbosity.SOME_OUTPUT_TYPE}"
        tmp_path.joinpath("pytest.ini").write_text(
            textwrap.dedent(
                f"""\
                [pytest]
                addopts = --verbose
                {setting_name} = {TestVerbosity.SOME_OUTPUT_VERBOSITY_LEVEL}
                """
            ),
            encoding="utf-8",
        )
        pytester.plugins = [TestVerbosity.VerbosityIni()]

        config = pytester.parseconfig(tmp_path)

        assert (
            config.get_verbosity(TestVerbosity.SOME_OUTPUT_TYPE)
            == TestVerbosity.SOME_OUTPUT_VERBOSITY_LEVEL
        )
