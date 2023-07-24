import io
import sys

import pytest
from _pytest.pytester import Pytester


def test_enabled(pytester: Pytester) -> None:
    """Test single crashing test displays a traceback."""
    pytester.makepyfile(
        """
    import faulthandler
    def test_crash():
        faulthandler._sigabrt()
    """
    )
    result = pytester.runpytest_subprocess()
    result.stderr.fnmatch_lines(["*Fatal Python error*"])
    assert result.ret != 0


def setup_crashing_test(pytester: Pytester) -> None:
    pytester.makepyfile(
        """
        import faulthandler
        import atexit
        def test_ok():
            atexit.register(faulthandler._sigabrt)
        """
    )


def test_crash_during_shutdown_captured(pytester: Pytester) -> None:
    """
    Re-enable faulthandler if pytest encountered it enabled during configure.
    We should be able to then see crashes during interpreter shutdown.
    """
    setup_crashing_test(pytester)
    args = (sys.executable, "-Xfaulthandler", "-mpytest")
    result = pytester.run(*args)
    result.stderr.fnmatch_lines(["*Fatal Python error*"])
    assert result.ret != 0


def test_crash_during_shutdown_not_captured(pytester: Pytester) -> None:
    """
    Check that pytest leaves faulthandler disabled if it was not enabled during configure.
    This prevents us from seeing crashes during interpreter shutdown (see #8260).
    """
    setup_crashing_test(pytester)
    args = (sys.executable, "-mpytest")
    result = pytester.run(*args)
    result.stderr.no_fnmatch_line("*Fatal Python error*")
    assert result.ret != 0


def test_disabled(pytester: Pytester) -> None:
    """Test option to disable fault handler in the command line."""
    pytester.makepyfile(
        """
    import faulthandler
    def test_disabled():
        assert not faulthandler.is_enabled()
    """
    )
    result = pytester.runpytest_subprocess("-p", "no:faulthandler")
    result.stdout.fnmatch_lines(["*1 passed*"])
    assert result.ret == 0


@pytest.mark.parametrize(
    "enabled",
    [
        pytest.param(
            True, marks=pytest.mark.skip(reason="sometimes crashes on CI (#7022)")
        ),
        False,
    ],
)
def test_timeout(pytester: Pytester, enabled: bool) -> None:
    """Test option to dump tracebacks after a certain timeout.

    If faulthandler is disabled, no traceback will be dumped.
    """
    pytester.makepyfile(
        """
    import os, time
    def test_timeout():
        time.sleep(1 if "CI" in os.environ else 0.1)
    """
    )
    pytester.makeini(
        """
        [pytest]
        faulthandler_timeout = 0.01
        """
    )
    args = ["-p", "no:faulthandler"] if not enabled else []

    result = pytester.runpytest_subprocess(*args)
    tb_output = "most recent call first"
    if enabled:
        result.stderr.fnmatch_lines(["*%s*" % tb_output])
    else:
        assert tb_output not in result.stderr.str()
    result.stdout.fnmatch_lines(["*1 passed*"])
    assert result.ret == 0


@pytest.mark.parametrize("hook_name", ["pytest_enter_pdb", "pytest_exception_interact"])
def test_cancel_timeout_on_hook(monkeypatch, hook_name) -> None:
    """Make sure that we are cancelling any scheduled traceback dumping due
    to timeout before entering pdb (pytest-dev/pytest-faulthandler#12) or any
    other interactive exception (pytest-dev/pytest-faulthandler#14)."""
    import faulthandler
    from _pytest import faulthandler as faulthandler_plugin

    called = []

    monkeypatch.setattr(
        faulthandler, "cancel_dump_traceback_later", lambda: called.append(1)
    )

    # call our hook explicitly, we can trust that pytest will call the hook
    # for us at the appropriate moment
    hook_func = getattr(faulthandler_plugin, hook_name)
    hook_func()
    assert called == [1]


def test_already_initialized_crash(pytester: Pytester) -> None:
    """Even if faulthandler is already initialized, we still dump tracebacks on crashes (#8258)."""
    pytester.makepyfile(
        """
        def test():
            import faulthandler
            faulthandler._sigabrt()
    """
    )
    result = pytester.run(
        sys.executable,
        "-X",
        "faulthandler",
        "-mpytest",
        pytester.path,
    )
    result.stderr.fnmatch_lines(["*Fatal Python error*"])
    assert result.ret != 0


def test_get_stderr_fileno_invalid_fd() -> None:
    """Test for faulthandler being able to handle invalid file descriptors for stderr (#8249)."""
    from _pytest.faulthandler import get_stderr_fileno

    class StdErrWrapper(io.StringIO):
        """
        Mimic ``twisted.logger.LoggingFile`` to simulate returning an invalid file descriptor.

        https://github.com/twisted/twisted/blob/twisted-20.3.0/src/twisted/logger/_io.py#L132-L139
        """

        def fileno(self):
            return -1

    wrapper = StdErrWrapper()

    with pytest.MonkeyPatch.context() as mp:
        mp.setattr("sys.stderr", wrapper)

        # Even when the stderr wrapper signals an invalid file descriptor,
        # ``_get_stderr_fileno()`` should return the real one.
        assert get_stderr_fileno() == 2
