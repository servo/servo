from _pytest.pytester import Pytester
import pytest


@pytest.mark.filterwarnings("default::pytest.PytestUnhandledThreadExceptionWarning")
def test_unhandled_thread_exception(pytester: Pytester) -> None:
    pytester.makepyfile(
        test_it="""
        import threading

        def test_it():
            def oops():
                raise ValueError("Oops")

            t = threading.Thread(target=oops, name="MyThread")
            t.start()
            t.join()

        def test_2(): pass
        """
    )
    result = pytester.runpytest()
    assert result.ret == 0
    assert result.parseoutcomes() == {"passed": 2, "warnings": 1}
    result.stdout.fnmatch_lines(
        [
            "*= warnings summary =*",
            "test_it.py::test_it",
            "  * PytestUnhandledThreadExceptionWarning: Exception in thread MyThread",
            "  ",
            "  Traceback (most recent call last):",
            "  ValueError: Oops",
            "  ",
            "    warnings.warn(pytest.PytestUnhandledThreadExceptionWarning(msg))",
        ]
    )


@pytest.mark.filterwarnings("default::pytest.PytestUnhandledThreadExceptionWarning")
def test_unhandled_thread_exception_in_setup(pytester: Pytester) -> None:
    pytester.makepyfile(
        test_it="""
        import threading
        import pytest

        @pytest.fixture
        def threadexc():
            def oops():
                raise ValueError("Oops")
            t = threading.Thread(target=oops, name="MyThread")
            t.start()
            t.join()

        def test_it(threadexc): pass
        def test_2(): pass
        """
    )
    result = pytester.runpytest()
    assert result.ret == 0
    assert result.parseoutcomes() == {"passed": 2, "warnings": 1}
    result.stdout.fnmatch_lines(
        [
            "*= warnings summary =*",
            "test_it.py::test_it",
            "  * PytestUnhandledThreadExceptionWarning: Exception in thread MyThread",
            "  ",
            "  Traceback (most recent call last):",
            "  ValueError: Oops",
            "  ",
            "    warnings.warn(pytest.PytestUnhandledThreadExceptionWarning(msg))",
        ]
    )


@pytest.mark.filterwarnings("default::pytest.PytestUnhandledThreadExceptionWarning")
def test_unhandled_thread_exception_in_teardown(pytester: Pytester) -> None:
    pytester.makepyfile(
        test_it="""
        import threading
        import pytest

        @pytest.fixture
        def threadexc():
            def oops():
                raise ValueError("Oops")
            yield
            t = threading.Thread(target=oops, name="MyThread")
            t.start()
            t.join()

        def test_it(threadexc): pass
        def test_2(): pass
        """
    )
    result = pytester.runpytest()
    assert result.ret == 0
    assert result.parseoutcomes() == {"passed": 2, "warnings": 1}
    result.stdout.fnmatch_lines(
        [
            "*= warnings summary =*",
            "test_it.py::test_it",
            "  * PytestUnhandledThreadExceptionWarning: Exception in thread MyThread",
            "  ",
            "  Traceback (most recent call last):",
            "  ValueError: Oops",
            "  ",
            "    warnings.warn(pytest.PytestUnhandledThreadExceptionWarning(msg))",
        ]
    )


@pytest.mark.filterwarnings("error::pytest.PytestUnhandledThreadExceptionWarning")
def test_unhandled_thread_exception_warning_error(pytester: Pytester) -> None:
    pytester.makepyfile(
        test_it="""
        import threading
        import pytest

        def test_it():
            def oops():
                raise ValueError("Oops")
            t = threading.Thread(target=oops, name="MyThread")
            t.start()
            t.join()

        def test_2(): pass
        """
    )
    result = pytester.runpytest()
    assert result.ret == pytest.ExitCode.TESTS_FAILED
    assert result.parseoutcomes() == {"passed": 1, "failed": 1}
