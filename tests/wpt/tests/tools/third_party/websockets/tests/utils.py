import contextlib
import email.utils
import os
import pathlib
import platform
import tempfile
import time
import unittest
import warnings


# Generate TLS certificate with:
# $ openssl req -x509 -config test_localhost.cnf -days 15340 -newkey rsa:2048 \
#       -out test_localhost.crt -keyout test_localhost.key
# $ cat test_localhost.key test_localhost.crt > test_localhost.pem
# $ rm test_localhost.key test_localhost.crt

CERTIFICATE = bytes(pathlib.Path(__file__).with_name("test_localhost.pem"))


DATE = email.utils.formatdate(usegmt=True)


# Unit for timeouts. May be increased on slow machines by setting the
# WEBSOCKETS_TESTS_TIMEOUT_FACTOR environment variable.
MS = 0.001 * float(os.environ.get("WEBSOCKETS_TESTS_TIMEOUT_FACTOR", "1"))

# PyPy has a performance penalty for this test suite.
if platform.python_implementation() == "PyPy":  # pragma: no cover
    MS *= 5

# asyncio's debug mode has a 10x performance penalty for this test suite.
if os.environ.get("PYTHONASYNCIODEBUG"):  # pragma: no cover
    MS *= 10

# Ensure that timeouts are larger than the clock's resolution (for Windows).
MS = max(MS, 2.5 * time.get_clock_info("monotonic").resolution)


class GeneratorTestCase(unittest.TestCase):
    """
    Base class for testing generator-based coroutines.

    """

    def assertGeneratorRunning(self, gen):
        """
        Check that a generator-based coroutine hasn't completed yet.

        """
        next(gen)

    def assertGeneratorReturns(self, gen):
        """
        Check that a generator-based coroutine completes and return its value.

        """
        with self.assertRaises(StopIteration) as raised:
            next(gen)
        return raised.exception.value


class DeprecationTestCase(unittest.TestCase):
    """
    Base class for testing deprecations.

    """

    @contextlib.contextmanager
    def assertDeprecationWarning(self, message):
        """
        Check that a deprecation warning was raised with the given message.

        """
        with warnings.catch_warnings(record=True) as recorded_warnings:
            warnings.simplefilter("always")
            yield

        self.assertEqual(len(recorded_warnings), 1)
        warning = recorded_warnings[0]
        self.assertEqual(warning.category, DeprecationWarning)
        self.assertEqual(str(warning.message), message)


@contextlib.contextmanager
def temp_unix_socket_path():
    with tempfile.TemporaryDirectory() as temp_dir:
        yield str(pathlib.Path(temp_dir) / "websockets")
