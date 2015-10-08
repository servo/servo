# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

from __future__ import unicode_literals

import multiprocessing
import sys
import threading
import traceback
from Queue import Empty
from multiprocessing import Process, current_process, Queue

from mozlog import structuredlog

# Special value used as a sentinal in various commands
Stop = object()


class MessageLogger(object):
    def __init__(self, message_func):
        self.send_message = message_func

    def _log_data(self, action, **kwargs):
        self.send_message("log", action, kwargs)

    def process_output(self, process, data, command):
        self._log_data("process_output", process=process, data=data, command=command)


def _log_func(level_name):
    def log(self, message):
        self._log_data(level_name.lower(), message=message)
    log.__doc__ = """Log a message with level %s

:param message: The string message to log
""" % level_name
    log.__name__ = str(level_name).lower()
    return log

# Create all the methods on StructuredLog for debug levels
for level_name in structuredlog.log_levels:
    setattr(MessageLogger, level_name.lower(), _log_func(level_name))


class TestRunner(object):
    def __init__(self, test_queue, command_queue, result_queue, executor):
        """Class implementing the main loop for running tests.

        This class delegates the job of actually running a test to the executor
        that is passed in.

        :param test_queue: subprocess.Queue containing the tests to run
        :param command_queue: subprocess.Queue used to send commands to the
                              process
        :param result_queue: subprocess.Queue used to send results to the
                             parent TestManager process
        :param executor: TestExecutor object that will actually run a test.
        """
        self.test_queue = test_queue
        self.command_queue = command_queue
        self.result_queue = result_queue

        self.executor = executor
        self.name = current_process().name
        self.logger = MessageLogger(self.send_message)

    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc_value, traceback):
        self.teardown()

    def setup(self):
        self.executor.setup(self)

    def teardown(self):
        self.executor.teardown()
        self.send_message("runner_teardown")
        self.result_queue = None
        self.command_queue = None
        self.browser = None

    def run(self):
        """Main loop accepting commands over the pipe and triggering
        the associated methods"""
        self.setup()
        commands = {"run_test": self.run_test,
                    "stop": self.stop,
                    "wait": self.wait}
        while True:
            command, args = self.command_queue.get()
            try:
                rv = commands[command](*args)
            except Exception:
                self.send_message("error",
                                  "Error running command %s with arguments %r:\n%s" %
                                  (command, args, traceback.format_exc()))
            else:
                if rv is Stop:
                    break

    def stop(self):
        return Stop

    def run_test(self):
        if not self.executor.is_alive():
            self.send_message("restart_runner")
            return
        try:
            # Need to block here just to allow for contention with other processes
            test = self.test_queue.get(block=True, timeout=1)
        except Empty:
            # If we are running tests in groups (e.g. by-dir) then this queue might be
            # empty but there could be other test queues. restart_runner won't actually
            # start the runner if there aren't any more tests to run
            self.send_message("restart_runner")
            return
        else:
            self.send_message("test_start", test)
        try:
            return self.executor.run_test(test)
        except Exception:
            self.logger.critical(traceback.format_exc())
            raise

    def wait(self):
        self.executor.protocol.wait()
        self.send_message("after_test_ended", True)

    def send_message(self, command, *args):
        self.result_queue.put((command, args))


def start_runner(test_queue, runner_command_queue, runner_result_queue,
                 executor_cls, executor_kwargs,
                 executor_browser_cls, executor_browser_kwargs,
                 stop_flag):
    """Launch a TestRunner in a new process"""
    try:
        browser = executor_browser_cls(**executor_browser_kwargs)
        executor = executor_cls(browser, **executor_kwargs)
        with TestRunner(test_queue, runner_command_queue, runner_result_queue, executor) as runner:
            try:
                runner.run()
            except KeyboardInterrupt:
                stop_flag.set()
    except Exception:
        runner_result_queue.put(("log", ("critical", {"message": traceback.format_exc()})))
        print >> sys.stderr, traceback.format_exc()
        stop_flag.set()
    finally:
        runner_command_queue = None
        runner_result_queue = None


manager_count = 0


def next_manager_number():
    global manager_count
    local = manager_count = manager_count + 1
    return local


class TestRunnerManager(threading.Thread):
    init_lock = threading.Lock()

    def __init__(self, suite_name, test_queue, test_source_cls, browser_cls, browser_kwargs,
                 executor_cls, executor_kwargs, stop_flag, pause_after_test=False,
                 pause_on_unexpected=False, debug_info=None):
        """Thread that owns a single TestRunner process and any processes required
        by the TestRunner (e.g. the Firefox binary).

        TestRunnerManagers are responsible for launching the browser process and the
        runner process, and for logging the test progress. The actual test running
        is done by the TestRunner. In particular they:

        * Start the binary of the program under test
        * Start the TestRunner
        * Tell the TestRunner to start a test, if any
        * Log that the test started
        * Log the test results
        * Take any remedial action required e.g. restart crashed or hung
          processes
        """
        self.suite_name = suite_name

        self.test_queue = test_queue
        self.test_source_cls = test_source_cls

        self.browser_cls = browser_cls
        self.browser_kwargs = browser_kwargs

        self.executor_cls = executor_cls
        self.executor_kwargs = executor_kwargs

        self.test_source = None

        self.browser = None
        self.browser_pid = None
        self.browser_started = False

        # Flags used to shut down this thread if we get a sigint
        self.parent_stop_flag = stop_flag
        self.child_stop_flag = multiprocessing.Event()

        self.pause_after_test = pause_after_test
        self.pause_on_unexpected = pause_on_unexpected
        self.debug_info = debug_info

        self.manager_number = next_manager_number()

        self.command_queue = Queue()
        self.remote_queue = Queue()

        self.test_runner_proc = None

        threading.Thread.__init__(self, name="Thread-TestrunnerManager-%i" % self.manager_number)
        # This is started in the actual new thread
        self.logger = None

        # The test that is currently running
        self.test = None

        self.unexpected_count = 0

        # This may not really be what we want
        self.daemon = True

        self.init_fail_count = 0
        self.max_init_fails = 5
        self.init_timer = None

        self.restart_count = 0
        self.max_restarts = 5

    def run(self):
        """Main loop for the TestManager.

        TestManagers generally receive commands from their
        TestRunner updating them on the status of a test. They
        may also have a stop flag set by the main thread indicating
        that the manager should shut down the next time the event loop
        spins."""
        self.logger = structuredlog.StructuredLogger(self.suite_name)
        with self.browser_cls(self.logger, **self.browser_kwargs) as browser, self.test_source_cls(self.test_queue) as test_source:
            self.browser = browser
            self.test_source = test_source
            try:
                if self.init() is Stop:
                    return
                while True:
                    commands = {"init_succeeded": self.init_succeeded,
                                "init_failed": self.init_failed,
                                "test_start": self.test_start,
                                "test_ended": self.test_ended,
                                "after_test_ended": self.after_test_ended,
                                "restart_runner": self.restart_runner,
                                "runner_teardown": self.runner_teardown,
                                "log": self.log,
                                "error": self.error}
                    try:
                        command, data = self.command_queue.get(True, 1)
                    except IOError:
                        if not self.should_stop():
                            self.logger.error("Got IOError from poll")
                            self.restart_count += 1
                            if self.restart_runner() is Stop:
                                break
                    except Empty:
                        command = None

                    if self.should_stop():
                        self.logger.debug("A flag was set; stopping")
                        break

                    if command is not None:
                        self.restart_count = 0
                        if commands[command](*data) is Stop:
                            break
                    else:
                        if (self.debug_info and self.debug_info.interactive and
                            self.browser_started and not browser.is_alive()):
                            self.logger.debug("Debugger exited")
                            break
                        if not self.test_runner_proc.is_alive():
                            if not self.command_queue.empty():
                                # We got a new message so process that
                                continue

                            # If we got to here the runner presumably shut down
                            # unexpectedly
                            self.logger.info("Test runner process shut down")

                            if self.test is not None:
                                # This could happen if the test runner crashed for some other
                                # reason
                                # Need to consider the unlikely case where one test causes the
                                # runner process to repeatedly die
                                self.logger.critical("Last test did not complete")
                                break
                            self.logger.warning(
                                "More tests found, but runner process died, restarting")
                            self.restart_count += 1
                            if self.restart_runner() is Stop:
                                break
            finally:
                self.logger.debug("TestRunnerManager main loop terminating, starting cleanup")
                self.stop_runner()
                self.teardown()
                self.logger.debug("TestRunnerManager main loop terminated")

    def should_stop(self):
        return self.child_stop_flag.is_set() or self.parent_stop_flag.is_set()

    def init(self):
        """Launch the browser that is being tested,
        and the TestRunner process that will run the tests."""
        # It seems that this lock is helpful to prevent some race that otherwise
        # sometimes stops the spawned processes initalising correctly, and
        # leaves this thread hung
        if self.init_timer is not None:
            self.init_timer.cancel()

        self.logger.debug("Init called, starting browser and runner")

        def init_failed():
            # This is called from a seperate thread, so we send a message to the
            # main loop so we get back onto the manager thread
            self.logger.debug("init_failed called from timer")
            if self.command_queue:
                self.command_queue.put(("init_failed", ()))
            else:
                self.logger.debug("Setting child stop flag in init_failed")
                self.child_stop_flag.set()

        with self.init_lock:
            # Guard against problems initialising the browser or the browser
            # remote control method
            if self.debug_info is None:
                self.init_timer = threading.Timer(self.browser.init_timeout, init_failed)

            test_queue = self.test_source.get_queue()
            if test_queue is None:
                self.logger.info("No more tests")
                return Stop

            try:
                if self.init_timer is not None:
                    self.init_timer.start()
                self.browser.start()
                self.browser_pid = self.browser.pid()
                self.start_test_runner(test_queue)
            except:
                self.logger.warning("Failure during init %s" % traceback.format_exc())
                if self.init_timer is not None:
                    self.init_timer.cancel()
                self.logger.error(traceback.format_exc())
                succeeded = False
            else:
                succeeded = True
                self.browser_started = True

        # This has to happen after the lock is released
        if not succeeded:
            self.init_failed()

    def init_succeeded(self):
        """Callback when we have started the browser, started the remote
        control connection, and we are ready to start testing."""
        self.logger.debug("Init succeeded")
        if self.init_timer is not None:
            self.init_timer.cancel()
        self.init_fail_count = 0
        self.start_next_test()

    def init_failed(self):
        """Callback when starting the browser or the remote control connect
        fails."""
        self.init_fail_count += 1
        self.logger.warning("Init failed %i" % self.init_fail_count)
        if self.init_timer is not None:
            self.init_timer.cancel()
        if self.init_fail_count < self.max_init_fails:
            self.restart_runner()
        else:
            self.logger.critical("Test runner failed to initialise correctly; shutting down")
            return Stop

    def start_test_runner(self, test_queue):
        # Note that we need to be careful to start the browser before the
        # test runner to ensure that any state set when the browser is started
        # can be passed in to the test runner.
        assert self.command_queue is not None
        assert self.remote_queue is not None
        self.logger.info("Starting runner")
        executor_browser_cls, executor_browser_kwargs = self.browser.executor_browser()

        args = (test_queue,
                self.remote_queue,
                self.command_queue,
                self.executor_cls,
                self.executor_kwargs,
                executor_browser_cls,
                executor_browser_kwargs,
                self.child_stop_flag)
        self.test_runner_proc = Process(target=start_runner,
                                        args=args,
                                        name="Thread-TestRunner-%i" % self.manager_number)
        self.test_runner_proc.start()
        self.logger.debug("Test runner started")

    def send_message(self, command, *args):
        self.remote_queue.put((command, args))

    def cleanup(self):
        if self.init_timer is not None:
            self.init_timer.cancel()
        self.logger.debug("TestManager cleanup")

        while True:
            try:
                self.logger.warning(" ".join(map(repr, self.command_queue.get_nowait())))
            except Empty:
                break

        while True:
            try:
                self.logger.warning(" ".join(map(repr, self.remote_queue.get_nowait())))
            except Empty:
                break

    def teardown(self):
        self.logger.debug("teardown in testrunnermanager")
        self.test_runner_proc = None
        self.command_queue.close()
        self.remote_queue.close()
        self.command_queue = None
        self.remote_queue = None

    def ensure_runner_stopped(self):
        if self.test_runner_proc is None:
            return

        self.test_runner_proc.join(10)
        if self.test_runner_proc.is_alive():
            # This might leak a file handle from the queue
            self.logger.warning("Forcibly terminating runner process")
            self.test_runner_proc.terminate()
            self.test_runner_proc.join(10)
        else:
            self.logger.debug("Testrunner exited with code %i" % self.test_runner_proc.exitcode)

    def runner_teardown(self):
        self.ensure_runner_stopped()
        return Stop

    def stop_runner(self):
        """Stop the TestRunner and the Firefox binary."""
        self.logger.debug("Stopping runner")
        if self.test_runner_proc is None:
            return
        try:
            self.browser.stop()
            self.browser_started = False
            if self.test_runner_proc.is_alive():
                self.send_message("stop")
                self.ensure_runner_stopped()
        finally:
            self.cleanup()

    def start_next_test(self):
        self.send_message("run_test")

    def test_start(self, test):
        self.test = test
        self.logger.test_start(test.id)

    def test_ended(self, test, results):
        """Handle the end of a test.

        Output the result of each subtest, and the result of the overall
        harness to the logs.
        """
        assert test == self.test
        # Write the result of each subtest
        file_result, test_results = results
        subtest_unexpected = False
        for result in test_results:
            if test.disabled(result.name):
                continue
            expected = test.expected(result.name)
            is_unexpected = expected != result.status

            if is_unexpected:
                self.unexpected_count += 1
                self.logger.debug("Unexpected count in this thread %i" % self.unexpected_count)
                subtest_unexpected = True
            self.logger.test_status(test.id,
                                    result.name,
                                    result.status,
                                    message=result.message,
                                    expected=expected,
                                    stack=result.stack)

        # TODO: consider changing result if there is a crash dump file

        # Write the result of the test harness
        expected = test.expected()
        status = file_result.status if file_result.status != "EXTERNAL-TIMEOUT" else "TIMEOUT"
        is_unexpected = expected != status
        if is_unexpected:
            self.unexpected_count += 1
            self.logger.debug("Unexpected count in this thread %i" % self.unexpected_count)
        if status == "CRASH":
            self.browser.log_crash(process=self.browser_pid, test=test.id)

        self.logger.test_end(test.id,
                             status,
                             message=file_result.message,
                             expected=expected,
                             extra=file_result.extra)

        self.test = None

        restart_before_next = (file_result.status in ("CRASH", "EXTERNAL-TIMEOUT") or
                               subtest_unexpected or is_unexpected)

        if (self.pause_after_test or
            (self.pause_on_unexpected and (subtest_unexpected or is_unexpected))):
            self.logger.info("Pausing until the browser exits")
            self.send_message("wait")
        else:
            self.after_test_ended(restart_before_next)

    def after_test_ended(self, restart_before_next):
        # Handle starting the next test, with a runner restart if required
        if restart_before_next:
            return self.restart_runner()
        else:
            return self.start_next_test()

    def restart_runner(self):
        """Stop and restart the TestRunner"""
        if self.restart_count >= self.max_restarts:
            return Stop
        self.stop_runner()
        return self.init()

    def log(self, action, kwargs):
        getattr(self.logger, action)(**kwargs)

    def error(self, message):
        self.logger.error(message)
        self.restart_runner()


class TestQueue(object):
    def __init__(self, test_source_cls, test_type, tests, **kwargs):
        self.queue = None
        self.test_source_cls = test_source_cls
        self.test_type = test_type
        self.tests = tests
        self.kwargs = kwargs

    def __enter__(self):
        if not self.tests[self.test_type]:
            return None

        self.queue = Queue()
        has_tests = self.test_source_cls.queue_tests(self.queue,
                                                     self.test_type,
                                                     self.tests,
                                                     **self.kwargs)
        # There is a race condition that means sometimes we continue
        # before the tests have been written to the underlying pipe.
        # Polling the pipe for data here avoids that
        self.queue._reader.poll(10)
        assert not self.queue.empty()
        return self.queue

    def __exit__(self, *args, **kwargs):
        if self.queue is not None:
            self.queue.close()
            self.queue = None


class ManagerGroup(object):
    def __init__(self, suite_name, size, test_source_cls, test_source_kwargs,
                 browser_cls, browser_kwargs,
                 executor_cls, executor_kwargs,
                 pause_after_test=False,
                 pause_on_unexpected=False,
                 debug_info=None):
        """Main thread object that owns all the TestManager threads."""
        self.suite_name = suite_name
        self.size = size
        self.test_source_cls = test_source_cls
        self.test_source_kwargs = test_source_kwargs
        self.browser_cls = browser_cls
        self.browser_kwargs = browser_kwargs
        self.executor_cls = executor_cls
        self.executor_kwargs = executor_kwargs
        self.pause_after_test = pause_after_test
        self.pause_on_unexpected = pause_on_unexpected
        self.debug_info = debug_info

        self.pool = set()
        # Event that is polled by threads so that they can gracefully exit in the face
        # of sigint
        self.stop_flag = threading.Event()
        self.logger = structuredlog.StructuredLogger(suite_name)
        self.test_queue = None

    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        self.stop()

    def run(self, test_type, tests):
        """Start all managers in the group"""
        self.logger.debug("Using %i processes" % self.size)

        self.test_queue = TestQueue(self.test_source_cls,
                                    test_type,
                                    tests,
                                    **self.test_source_kwargs)
        with self.test_queue as test_queue:
            if test_queue is None:
                self.logger.info("No %s tests to run" % test_type)
                return
            for _ in range(self.size):
                manager = TestRunnerManager(self.suite_name,
                                            test_queue,
                                            self.test_source_cls,
                                            self.browser_cls,
                                            self.browser_kwargs,
                                            self.executor_cls,
                                            self.executor_kwargs,
                                            self.stop_flag,
                                            self.pause_after_test,
                                            self.pause_on_unexpected,
                                            self.debug_info)
                manager.start()
                self.pool.add(manager)
            self.wait()

    def is_alive(self):
        """Boolean indicating whether any manager in the group is still alive"""
        return any(manager.is_alive() for manager in self.pool)

    def wait(self):
        """Wait for all the managers in the group to finish"""
        for item in self.pool:
            item.join()

    def stop(self):
        """Set the stop flag so that all managers in the group stop as soon
        as possible"""
        self.stop_flag.set()
        self.logger.debug("Stop flag set in ManagerGroup")

    def unexpected_count(self):
        return sum(item.unexpected_count for item in self.pool)
