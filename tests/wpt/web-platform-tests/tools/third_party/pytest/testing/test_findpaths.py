from pathlib import Path
from textwrap import dedent

import pytest
from _pytest.config import UsageError
from _pytest.config.findpaths import get_common_ancestor
from _pytest.config.findpaths import get_dirs_from_args
from _pytest.config.findpaths import load_config_dict_from_file


class TestLoadConfigDictFromFile:
    def test_empty_pytest_ini(self, tmp_path: Path) -> None:
        """pytest.ini files are always considered for configuration, even if empty"""
        fn = tmp_path / "pytest.ini"
        fn.write_text("", encoding="utf-8")
        assert load_config_dict_from_file(fn) == {}

    def test_pytest_ini(self, tmp_path: Path) -> None:
        """[pytest] section in pytest.ini files is read correctly"""
        fn = tmp_path / "pytest.ini"
        fn.write_text("[pytest]\nx=1", encoding="utf-8")
        assert load_config_dict_from_file(fn) == {"x": "1"}

    def test_custom_ini(self, tmp_path: Path) -> None:
        """[pytest] section in any .ini file is read correctly"""
        fn = tmp_path / "custom.ini"
        fn.write_text("[pytest]\nx=1", encoding="utf-8")
        assert load_config_dict_from_file(fn) == {"x": "1"}

    def test_custom_ini_without_section(self, tmp_path: Path) -> None:
        """Custom .ini files without [pytest] section are not considered for configuration"""
        fn = tmp_path / "custom.ini"
        fn.write_text("[custom]", encoding="utf-8")
        assert load_config_dict_from_file(fn) is None

    def test_custom_cfg_file(self, tmp_path: Path) -> None:
        """Custom .cfg files without [tool:pytest] section are not considered for configuration"""
        fn = tmp_path / "custom.cfg"
        fn.write_text("[custom]", encoding="utf-8")
        assert load_config_dict_from_file(fn) is None

    def test_valid_cfg_file(self, tmp_path: Path) -> None:
        """Custom .cfg files with [tool:pytest] section are read correctly"""
        fn = tmp_path / "custom.cfg"
        fn.write_text("[tool:pytest]\nx=1", encoding="utf-8")
        assert load_config_dict_from_file(fn) == {"x": "1"}

    def test_unsupported_pytest_section_in_cfg_file(self, tmp_path: Path) -> None:
        """.cfg files with [pytest] section are no longer supported and should fail to alert users"""
        fn = tmp_path / "custom.cfg"
        fn.write_text("[pytest]", encoding="utf-8")
        with pytest.raises(pytest.fail.Exception):
            load_config_dict_from_file(fn)

    def test_invalid_toml_file(self, tmp_path: Path) -> None:
        """Invalid .toml files should raise `UsageError`."""
        fn = tmp_path / "myconfig.toml"
        fn.write_text("]invalid toml[", encoding="utf-8")
        with pytest.raises(UsageError):
            load_config_dict_from_file(fn)

    def test_custom_toml_file(self, tmp_path: Path) -> None:
        """.toml files without [tool.pytest.ini_options] are not considered for configuration."""
        fn = tmp_path / "myconfig.toml"
        fn.write_text(
            dedent(
                """
            [build_system]
            x = 1
            """
            ),
            encoding="utf-8",
        )
        assert load_config_dict_from_file(fn) is None

    def test_valid_toml_file(self, tmp_path: Path) -> None:
        """.toml files with [tool.pytest.ini_options] are read correctly, including changing
        data types to str/list for compatibility with other configuration options."""
        fn = tmp_path / "myconfig.toml"
        fn.write_text(
            dedent(
                """
            [tool.pytest.ini_options]
            x = 1
            y = 20.0
            values = ["tests", "integration"]
            name = "foo"
            heterogeneous_array = [1, "str"]
            """
            ),
            encoding="utf-8",
        )
        assert load_config_dict_from_file(fn) == {
            "x": "1",
            "y": "20.0",
            "values": ["tests", "integration"],
            "name": "foo",
            "heterogeneous_array": [1, "str"],
        }


class TestCommonAncestor:
    def test_has_ancestor(self, tmp_path: Path) -> None:
        fn1 = tmp_path / "foo" / "bar" / "test_1.py"
        fn1.parent.mkdir(parents=True)
        fn1.touch()
        fn2 = tmp_path / "foo" / "zaz" / "test_2.py"
        fn2.parent.mkdir(parents=True)
        fn2.touch()
        assert get_common_ancestor([fn1, fn2]) == tmp_path / "foo"
        assert get_common_ancestor([fn1.parent, fn2]) == tmp_path / "foo"
        assert get_common_ancestor([fn1.parent, fn2.parent]) == tmp_path / "foo"
        assert get_common_ancestor([fn1, fn2.parent]) == tmp_path / "foo"

    def test_single_dir(self, tmp_path: Path) -> None:
        assert get_common_ancestor([tmp_path]) == tmp_path

    def test_single_file(self, tmp_path: Path) -> None:
        fn = tmp_path / "foo.py"
        fn.touch()
        assert get_common_ancestor([fn]) == tmp_path


def test_get_dirs_from_args(tmp_path):
    """get_dirs_from_args() skips over non-existing directories and files"""
    fn = tmp_path / "foo.py"
    fn.touch()
    d = tmp_path / "tests"
    d.mkdir()
    option = "--foobar=/foo.txt"
    # xdist uses options in this format for its rsync feature (#7638)
    xdist_rsync_option = "popen=c:/dest"
    assert get_dirs_from_args(
        [str(fn), str(tmp_path / "does_not_exist"), str(d), option, xdist_rsync_option]
    ) == [fn.parent, d]
