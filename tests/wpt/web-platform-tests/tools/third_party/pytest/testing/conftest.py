import re
import sys
from typing import List

import pytest
from _pytest.pytester import RunResult
from _pytest.pytester import Testdir

if sys.gettrace():

    @pytest.fixture(autouse=True)
    def restore_tracing():
        """Restore tracing function (when run with Coverage.py).

        https://bugs.python.org/issue37011
        """
        orig_trace = sys.gettrace()
        yield
        if sys.gettrace() != orig_trace:
            sys.settrace(orig_trace)


@pytest.hookimpl(hookwrapper=True, tryfirst=True)
def pytest_collection_modifyitems(items):
    """Prefer faster tests.

    Use a hookwrapper to do this in the beginning, so e.g. --ff still works
    correctly.
    """
    fast_items = []
    slow_items = []
    slowest_items = []
    neutral_items = []

    spawn_names = {"spawn_pytest", "spawn"}

    for item in items:
        try:
            fixtures = item.fixturenames
        except AttributeError:
            # doctest at least
            # (https://github.com/pytest-dev/pytest/issues/5070)
            neutral_items.append(item)
        else:
            if "testdir" in fixtures:
                co_names = item.function.__code__.co_names
                if spawn_names.intersection(co_names):
                    item.add_marker(pytest.mark.uses_pexpect)
                    slowest_items.append(item)
                elif "runpytest_subprocess" in co_names:
                    slowest_items.append(item)
                else:
                    slow_items.append(item)
                item.add_marker(pytest.mark.slow)
            else:
                marker = item.get_closest_marker("slow")
                if marker:
                    slowest_items.append(item)
                else:
                    fast_items.append(item)

    items[:] = fast_items + neutral_items + slow_items + slowest_items

    yield


@pytest.fixture
def tw_mock():
    """Returns a mock terminal writer"""

    class TWMock:
        WRITE = object()

        def __init__(self):
            self.lines = []
            self.is_writing = False

        def sep(self, sep, line=None):
            self.lines.append((sep, line))

        def write(self, msg, **kw):
            self.lines.append((TWMock.WRITE, msg))

        def _write_source(self, lines, indents=()):
            if not indents:
                indents = [""] * len(lines)
            for indent, line in zip(indents, lines):
                self.line(indent + line)

        def line(self, line, **kw):
            self.lines.append(line)

        def markup(self, text, **kw):
            return text

        def get_write_msg(self, idx):
            flag, msg = self.lines[idx]
            assert flag == TWMock.WRITE
            return msg

        fullwidth = 80

    return TWMock()


@pytest.fixture
def dummy_yaml_custom_test(testdir):
    """Writes a conftest file that collects and executes a dummy yaml test.

    Taken from the docs, but stripped down to the bare minimum, useful for
    tests which needs custom items collected.
    """
    testdir.makeconftest(
        """
        import pytest

        def pytest_collect_file(parent, path):
            if path.ext == ".yaml" and path.basename.startswith("test"):
                return YamlFile.from_parent(fspath=path, parent=parent)

        class YamlFile(pytest.File):
            def collect(self):
                yield YamlItem.from_parent(name=self.fspath.basename, parent=self)

        class YamlItem(pytest.Item):
            def runtest(self):
                pass
    """
    )
    testdir.makefile(".yaml", test1="")


@pytest.fixture
def testdir(testdir: Testdir) -> Testdir:
    testdir.monkeypatch.setenv("PYTEST_DISABLE_PLUGIN_AUTOLOAD", "1")
    return testdir


@pytest.fixture(scope="session")
def color_mapping():
    """Returns a utility class which can replace keys in strings in the form "{NAME}"
    by their equivalent ASCII codes in the terminal.

    Used by tests which check the actual colors output by pytest.
    """

    class ColorMapping:
        COLORS = {
            "red": "\x1b[31m",
            "green": "\x1b[32m",
            "yellow": "\x1b[33m",
            "bold": "\x1b[1m",
            "reset": "\x1b[0m",
            "kw": "\x1b[94m",
            "hl-reset": "\x1b[39;49;00m",
            "function": "\x1b[92m",
            "number": "\x1b[94m",
            "str": "\x1b[33m",
            "print": "\x1b[96m",
        }
        RE_COLORS = {k: re.escape(v) for k, v in COLORS.items()}

        @classmethod
        def format(cls, lines: List[str]) -> List[str]:
            """Straightforward replacement of color names to their ASCII codes."""
            return [line.format(**cls.COLORS) for line in lines]

        @classmethod
        def format_for_fnmatch(cls, lines: List[str]) -> List[str]:
            """Replace color names for use with LineMatcher.fnmatch_lines"""
            return [line.format(**cls.COLORS).replace("[", "[[]") for line in lines]

        @classmethod
        def format_for_rematch(cls, lines: List[str]) -> List[str]:
            """Replace color names for use with LineMatcher.re_match_lines"""
            return [line.format(**cls.RE_COLORS) for line in lines]

        @classmethod
        def requires_ordered_markup(cls, result: RunResult):
            """Should be called if a test expects markup to appear in the output
            in the order they were passed, for example:

                tw.write(line, bold=True, red=True)

            In Python 3.5 there's no guarantee that the generated markup will appear
            in the order called, so we do some limited color testing and skip the rest of
            the test.
            """
            if sys.version_info < (3, 6):
                # terminal writer.write accepts keyword arguments, so
                # py36+ is required so the markup appears in the expected order
                output = result.stdout.str()
                assert "test session starts" in output
                assert "\x1b[1m" in output
                pytest.skip(
                    "doing limited testing because lacking ordered markup on py35"
                )

    return ColorMapping


@pytest.fixture
def mock_timing(monkeypatch):
    """Mocks _pytest.timing with a known object that can be used to control timing in tests
    deterministically.

    pytest itself should always use functions from `_pytest.timing` instead of `time` directly.

    This then allows us more control over time during testing, if testing code also
    uses `_pytest.timing` functions.

    Time is static, and only advances through `sleep` calls, thus tests might sleep over large
    numbers and obtain accurate time() calls at the end, making tests reliable and instant.
    """
    import attr

    @attr.s
    class MockTiming:

        _current_time = attr.ib(default=1590150050.0)

        def sleep(self, seconds):
            self._current_time += seconds

        def time(self):
            return self._current_time

        def patch(self):
            from _pytest import timing

            monkeypatch.setattr(timing, "sleep", self.sleep)
            monkeypatch.setattr(timing, "time", self.time)
            monkeypatch.setattr(timing, "perf_counter", self.time)

    result = MockTiming()
    result.patch()
    return result
