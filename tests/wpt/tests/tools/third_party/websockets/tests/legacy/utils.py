import asyncio
import contextlib
import functools
import logging
import unittest


class AsyncioTestCase(unittest.TestCase):
    """
    Base class for tests that sets up an isolated event loop for each test.

    IsolatedAsyncioTestCase was introduced in Python 3.8 for similar purposes
    but isn't a drop-in replacement.

    """

    def __init_subclass__(cls, **kwargs):
        """
        Convert test coroutines to test functions.

        This supports asynchronous tests transparently.

        """
        super().__init_subclass__(**kwargs)
        for name in unittest.defaultTestLoader.getTestCaseNames(cls):
            test = getattr(cls, name)
            if asyncio.iscoroutinefunction(test):
                setattr(cls, name, cls.convert_async_to_sync(test))

    @staticmethod
    def convert_async_to_sync(test):
        """
        Convert a test coroutine to a test function.

        """

        @functools.wraps(test)
        def test_func(self, *args, **kwargs):
            return self.loop.run_until_complete(test(self, *args, **kwargs))

        return test_func

    def setUp(self):
        super().setUp()
        self.loop = asyncio.new_event_loop()
        asyncio.set_event_loop(self.loop)

    def tearDown(self):
        self.loop.close()
        super().tearDown()

    def run_loop_once(self):
        # Process callbacks scheduled with call_soon by appending a callback
        # to stop the event loop then running it until it hits that callback.
        self.loop.call_soon(self.loop.stop)
        self.loop.run_forever()

    # Remove when dropping Python < 3.10
    @contextlib.contextmanager
    def assertNoLogs(self, logger="websockets", level=logging.ERROR):
        """
        No message is logged on the given logger with at least the given level.

        """
        with self.assertLogs(logger, level) as logs:
            # We want to test that no log message is emitted
            # but assertLogs expects at least one log message.
            logging.getLogger(logger).log(level, "dummy")
            yield

        level_name = logging.getLevelName(level)
        self.assertEqual(logs.output, [f"{level_name}:{logger}:dummy"])

    def assertDeprecationWarnings(self, recorded_warnings, expected_warnings):
        """
        Check recorded deprecation warnings match a list of expected messages.

        """
        for recorded in recorded_warnings:
            self.assertEqual(type(recorded.message), DeprecationWarning)
        self.assertEqual(
            set(str(recorded.message) for recorded in recorded_warnings),
            set(expected_warnings),
        )
