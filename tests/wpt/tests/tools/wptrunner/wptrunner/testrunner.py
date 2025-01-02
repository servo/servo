# mypy: allow-untyped-defs

import threading
import traceback
from queue import Empty
from collections import namedtuple, defaultdict
from typing import Any, Mapping, Optional

from mozlog import structuredlog, capture

from . import mpcontext, testloader

# Special value used as a sentinal in various commands
Stop = object()


def release_mozlog_lock():
    try:
        from mozlog.structuredlog import StructuredLogger
        try:
            StructuredLogger._lock.release()
        except threading.ThreadError:
            pass
    except ImportError:
        pass


TestImplementation = namedtuple('TestImplementation',
                                ['executor_cls', 'executor_kwargs',
                                 'browser_cls', 'browser_kwargs'])


ExecutorImplementation = namedtuple('ExecutorImplementation',
                                ['executor_cls', 'executor_kwargs',
                                 'executor_browser_cls', 'executor_browser_kwargs'])


class StopFlag:
    """Synchronization for coordinating a graceful exit."""

    def __init__(self, size: int):
        # Flag that is polled by threads so that they can gracefully exit in the
        # face of SIGINT.
        self._should_stop = threading.Event()
        # A barrier that each `TestRunnerManager` thread waits on when exiting
        # its run loop. This provides a reliable way for the `ManagerGroup` to
        # tell when all threads have cleaned up their resources.
        #
        # The barrier's extra waiter is the main thread (`ManagerGroup`).
        self._all_managers_done = threading.Barrier(1 + size)

    def stop(self) -> None:
        self._should_stop.set()

    def should_stop(self) -> bool:
        return self._should_stop.is_set()

    def wait_for_all_managers_done(self, timeout: Optional[float] = None) -> None:
        self._all_managers_done.wait(timeout)


class LogMessageHandler:
    def __init__(self, send_message):
        self.send_message = send_message

    def __call__(self, data):
        self.send_message("log", data)


class TestRunner:
    """Class implementing the main loop for running tests.

    This class delegates the job of actually running a test to the executor
    that is passed in.

    :param logger: Structured logger
    :param command_queue: multiprocessing.Queue used to send commands to the
                          process
    :param result_queue: multiprocessing.Queue used to send results to the
                         parent TestRunnerManager process
    :param executor: TestExecutor object that will actually run a test.
    """
    def __init__(self, logger, command_queue, result_queue, executor_implementation, recording):
        self.command_queue = command_queue
        self.result_queue = result_queue
        browser = executor_implementation.executor_browser_cls(
            **executor_implementation.executor_browser_kwargs)
        self.executor = executor_implementation.executor_cls(
            logger, browser, **executor_implementation.executor_kwargs)
        self.name = mpcontext.get_context().current_process().name
        self.logger = logger
        self.recording = recording

    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc_value, traceback):
        self.teardown()

    def setup(self):
        self.logger.debug("Executor setup")
        try:
            self.executor.setup(self)
        except Exception:
            # The caller is responsible for logging the exception if required
            self.send_message("init_failed")
        else:
            self.send_message("init_succeeded")
        self.logger.debug("Executor setup done")

    def teardown(self):
        self.executor.teardown()
        self.result_queue = None
        self.command_queue = None
        self.browser = None

    def switch_executor(self, executor_implementation):
        assert self.executor is not None
        # reuse the current protocol connection
        protocol = self.executor.protocol
        self.executor.protocol = None
        self.executor.teardown()
        browser = executor_implementation.executor_browser_cls(
            **executor_implementation.executor_browser_kwargs)
        self.executor = executor_implementation.executor_cls(
            self.logger, browser, **executor_implementation.executor_kwargs)
        if type(self.executor.protocol) is not type(protocol):
            self.send_message("switch_executor_failed")
            self.logger.error("Protocol type does not match, switch executor failed.")
            return
        try:
            self.executor.setup(self, protocol)
        except Exception:
            self.send_message("switch_executor_failed")
        else:
            self.send_message("switch_executor_succeeded")
        self.logger.debug("Switch Executor done")

    def run(self):
        """Main loop accepting commands over the pipe and triggering
        the associated methods"""
        self.setup()
        commands = {"run_test": self.run_test,
                    "switch_executor": self.switch_executor,
                    "reset": self.reset,
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

    def reset(self):
        self.executor.reset()

    def run_test(self, test):
        try:
            return self.executor.run_test(test)
        except Exception:
            message = "TestRunner.run_test caught an exception:\n"
            message += traceback.format_exc()
            self.logger.error(message)
            raise

    def wait(self):
        rerun = self.executor.wait()
        self.send_message("wait_finished", rerun)

    def send_message(self, command, *args):
        self.result_queue.put((command, args))


def start_runner(runner_command_queue, runner_result_queue,
                 executor_implementation, capture_stdio,
                 stop_flag, recording):
    """Launch a TestRunner in a new process"""

    def send_message(command, *args):
        runner_result_queue.put((command, args))

    def handle_error(e):
        logger.critical(traceback.format_exc())
        stop_flag.set()

    # Ensure that when we start this in a new process we have the global lock
    # in the logging module unlocked
    release_mozlog_lock()

    proc_name = mpcontext.get_context().current_process().name
    logger = structuredlog.StructuredLogger(proc_name)
    logger.add_handler(LogMessageHandler(send_message))

    with capture.CaptureIO(logger, capture_stdio):
        try:
            with TestRunner(logger,
                            runner_command_queue,
                            runner_result_queue,
                            executor_implementation,
                            recording) as runner:
                try:
                    runner.run()
                except KeyboardInterrupt:
                    stop_flag.set()
                except Exception as e:
                    handle_error(e)
        except Exception as e:
            handle_error(e)


class BrowserManager:
    def __init__(self, logger, browser, command_queue, no_timeout=False):
        self.logger = logger
        self.browser = browser
        self.no_timeout = no_timeout
        self.browser_settings = None
        self.last_test = None

        self.started = False

        self.browser_pid = None
        self.init_timer = None
        self.command_queue = command_queue

    def update_settings(self, test):
        browser_settings = self.browser.settings(test)
        restart_required = ((self.browser_settings is not None and
                             self.browser_settings != browser_settings) or
                            (self.last_test != test and test.expected() == "CRASH"))
        self.browser_settings = browser_settings
        self.last_test = test
        return restart_required

    def init(self, group_metadata):
        """Launch the browser that is being tested,
        and the TestRunner process that will run the tests."""
        # It seems that this lock is helpful to prevent some race that otherwise
        # sometimes stops the spawned processes initialising correctly, and
        # leaves this thread hung
        if self.init_timer is not None:
            self.init_timer.cancel()

        self.logger.debug("Init called, starting browser and runner")

        if not self.no_timeout:
            self.init_timer = threading.Timer(self.browser.init_timeout,
                                              self.init_timeout)
        try:
            if self.init_timer is not None:
                self.init_timer.start()
            self.logger.debug("Starting browser with settings %r" % self.browser_settings)
            self.browser.start(group_metadata=group_metadata, **self.browser_settings)
            self.browser_pid = self.browser.pid
        except Exception:
            self.logger.error(f"Failure during init:\n{traceback.format_exc()}")
            if self.init_timer is not None:
                self.init_timer.cancel()
            succeeded = False
        else:
            succeeded = True
            self.started = True

        return succeeded

    def send_message(self, command, *args):
        self.command_queue.put((command, args))

    def init_timeout(self):
        # This is called from a separate thread, so we send a message to the
        # main loop so we get back onto the manager thread
        self.logger.debug("init_failed called from timer")
        self.send_message("init_failed")

    def after_init(self):
        """Callback when we have started the browser, started the remote
        control connection, and we are ready to start testing."""
        if self.init_timer is not None:
            self.init_timer.cancel()

    def stop(self, force=False):
        self.browser.stop(force=force)
        self.started = False

    def cleanup(self):
        if self.init_timer is not None:
            self.init_timer.cancel()

    def check_crash(self, test_id):
        return self.browser.check_crash(process=self.browser_pid, test=test_id)

    def is_alive(self):
        return self.browser.is_alive()


class TestSource:
    def __init__(self, logger: structuredlog.StructuredLogger, test_queue: testloader.ReadQueue):
        self.logger = logger
        self.test_queue = test_queue
        self.current_group = testloader.TestGroup(None, None, None, None)

    def group(self) -> testloader.TestGroup:
        if not self.current_group.group or len(self.current_group.group) == 0:
            try:
                self.current_group = self.test_queue.get()
                self.logger.debug(f"Got new test group subsuite:{self.current_group[1]} "
                                  f"test_type:{self.current_group[2]}")
            except Empty:
                return testloader.TestGroup(None, None, None, None)
        return self.current_group


class _RunnerManagerState:
    before_init = namedtuple("before_init", [])
    initializing = namedtuple("initializing",
                              ["subsuite", "test_type", "test", "test_group",
                               "group_metadata", "failure_count"])
    running = namedtuple("running", ["subsuite", "test_type", "test", "test_group", "group_metadata"])
    restarting = namedtuple("restarting", ["subsuite", "test_type", "test", "test_group",
                                           "group_metadata", "force_stop"])
    switching_executor = namedtuple("switching_executor",
                                    ["subsuite", "test_type", "test", "test_group", "group_metadata"])
    error = namedtuple("error", [])
    stop = namedtuple("stop", ["force_stop"])


RunnerManagerState = _RunnerManagerState()


class TestRunnerManager(threading.Thread):
    def __init__(self, suite_name, index, test_queue,
                 test_implementations, stop_flag, retry_index=0, rerun=1,
                 pause_after_test=False, pause_on_unexpected=False,
                 restart_on_unexpected=True, debug_info=None,
                 capture_stdio=True, restart_on_new_group=True, recording=None, max_restarts=5):
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
        self.manager_number = index
        self.test_implementation_key = None

        self.test_implementations = {}
        for key, test_implementation in test_implementations.items():
            browser_kwargs = test_implementation.browser_kwargs
            if browser_kwargs.get("device_serial"):
                browser_kwargs = browser_kwargs.copy()
                # Assign Android device to runner according to current manager index
                browser_kwargs["device_serial"] = browser_kwargs["device_serial"][index]
                self.test_implementations[key] = TestImplementation(
                    test_implementation.executor_cls,
                    test_implementation.executor_kwargs,
                    test_implementation.browser_cls,
                    browser_kwargs)
            else:
                self.test_implementations[key] = test_implementation

        # Flags used to shut down this thread if we get a sigint
        self.parent_stop_flag = stop_flag
        self.child_stop_flag = mpcontext.get_context().Event()

        # Keep track of the current retry index. The retries are meant to handle
        # flakiness, so at retry round we should restart the browser after each test.
        self.retry_index = retry_index
        self.rerun = rerun
        self.run_count = 0
        self.pause_after_test = pause_after_test
        self.pause_on_unexpected = pause_on_unexpected
        self.restart_on_unexpected = restart_on_unexpected
        self.debug_info = debug_info
        self.capture_stdio = capture_stdio
        self.restart_on_new_group = restart_on_new_group
        self.max_restarts = max_restarts

        assert recording is not None
        self.recording = recording

        self.test_count = 0
        self.unexpected_fail_tests = defaultdict(list)
        self.unexpected_pass_tests = defaultdict(list)

        # Properties we initialize right after the thread is started
        self.logger = None
        self.test_source = None
        self.command_queue = None
        self.remote_queue = None

        # Properties we initalize later in the lifecycle
        self.timer = None
        self.test_runner_proc = None
        self.browser = None

        super().__init__(name=f"TestRunnerManager-{index}", target=self.run_loop, args=[test_queue], daemon=True)

    def run_loop(self, test_queue):
        """Main loop for the TestRunnerManager.

        TestRunnerManagers generally receive commands from their
        TestRunner updating them on the status of a test. They
        may also have a stop flag set by the main thread indicating
        that the manager should shut down the next time the event loop
        spins."""
        self.recording.set(["testrunner", "startup"])
        self.logger = structuredlog.StructuredLogger(self.suite_name)

        self.test_source = TestSource(self.logger, test_queue)

        mp = mpcontext.get_context()
        self.command_queue = mp.Queue()
        self.remote_queue = mp.Queue()

        dispatch = {
            RunnerManagerState.before_init: self.start_init,
            RunnerManagerState.initializing: self.init,
            RunnerManagerState.running: self.run_test,
            RunnerManagerState.restarting: self.restart_runner,
        }

        self.state = RunnerManagerState.before_init()
        end_states = (RunnerManagerState.stop,
                      RunnerManagerState.error)

        try:
            while not isinstance(self.state, end_states):
                f = dispatch.get(self.state.__class__)
                while f:
                    self.logger.debug(f"Dispatch {f.__name__}")
                    if self.should_stop():
                        return
                    new_state = f()
                    if new_state is None:
                        break
                    self.state = new_state
                    self.logger.debug(f"new state: {self.state.__class__.__name__}")
                    if isinstance(self.state, end_states):
                        return
                    f = dispatch.get(self.state.__class__)

                new_state = None
                while new_state is None:
                    new_state = self.wait_event()
                    if self.should_stop():
                        return
                self.state = new_state
                self.logger.debug(f"new state: {self.state.__class__.__name__}")
        except Exception:
            message = "Uncaught exception in TestRunnerManager.run:\n"
            message += traceback.format_exc()
            self.logger.critical(message)
            raise
        finally:
            self._cleanup_run_loop()

    def _cleanup_run_loop(self):
        try:
            self.logger.debug("TestRunnerManager main loop terminating, starting cleanup")

            skipped_tests = []
            test_group, subsuite, _, _ = self.test_source.current_group
            while test_group is not None and len(test_group) > 0:
                test = test_group.popleft()
                skipped_tests.append(test)

            if skipped_tests:
                self.logger.critical(
                    f"Tests left in the queue: {subsuite}:{skipped_tests[0].id!r} "
                    f"and {len(skipped_tests) - 1} others"
                )
                for test in skipped_tests[1:]:
                    self.logger.debug(f"Test left in the queue: {subsuite}:{test.id!r}")

            force_stop = (not isinstance(self.state, RunnerManagerState.stop) or
                          self.state.force_stop)
            self.stop_runner(force=force_stop)
            self.teardown()
            if self.browser is not None:
                assert self.browser.browser is not None
                self.browser.browser.cleanup()
            self.logger.debug("TestRunnerManager main loop terminated")
        finally:
            # Even if the cleanup fails, signal that this thread is ready to
            # exit. Otherwise, the barrier backing `parent_stop_flag` will never
            # get enough watiers, causing wptrunner to hang.
            self.parent_stop_flag.wait_for_all_managers_done()

    def wait_event(self):
        dispatch = {
            RunnerManagerState.before_init: {},
            RunnerManagerState.initializing:
            {
                "init_succeeded": self.init_succeeded,
                "init_failed": self.init_failed,
            },
            RunnerManagerState.running:
            {
                "test_ended": self.test_ended,
                "wait_finished": self.wait_finished,
            },
            RunnerManagerState.switching_executor:
            {
                "switch_executor_succeeded": self.switch_executor_succeeded,
                "switch_executor_failed": self.switch_executor_failed,
            },
            RunnerManagerState.restarting: {},
            RunnerManagerState.error: {},
            RunnerManagerState.stop: {},
            None: {
                "log": self.log,
                "error": self.error,
            }
        }
        try:
            command, data = self.command_queue.get(True, 1)
            self.logger.debug("Got command: %r" % command)
        except OSError:
            self.logger.error("Got IOError from poll")
            return RunnerManagerState.restarting(self.state.subsuite,
                                                 self.state.test_type,
                                                 self.state.test,
                                                 self.state.test_group,
                                                 self.state.group_metadata,
                                                 False)
        except Empty:
            if (self.debug_info and self.debug_info.interactive and
                self.browser.started and not self.browser.is_alive()):
                self.logger.debug("Debugger exited")
                return RunnerManagerState.stop(False)

            # `test_runner_proc` must be nonnull in the manager's `running` state.
            if (isinstance(self.state, RunnerManagerState.running) and
                not self.test_runner_proc.is_alive()):
                if not self.command_queue.empty():
                    # We got a new message so process that
                    return

                # If we got to here the runner presumably shut down
                # unexpectedly
                self.logger.info("Test runner process shut down")

                if self.state.test is not None:
                    # This could happen if the test runner crashed for some other
                    # reason
                    # Need to consider the unlikely case where one test causes the
                    # runner process to repeatedly die
                    self.logger.critical("Last test did not complete")
                    return RunnerManagerState.error()
                self.logger.warning("More tests found, but runner process died, restarting")
                return RunnerManagerState.restarting(self.state.test_type,
                                                     self.state.test,
                                                     self.state.test_group,
                                                     self.state.group_metadata,
                                                     False)
        else:
            f = (dispatch.get(self.state.__class__, {}).get(command) or
                 dispatch.get(None, {}).get(command))
            if not f:
                self.logger.warning("Got command %s in state %s" %
                                    (command, self.state.__class__.__name__))
                return
            return f(*data)

    def should_stop(self):
        return self.child_stop_flag.is_set() or self.parent_stop_flag.should_stop()

    def start_init(self):
        subsuite, test_type, test, test_group, group_metadata = self.get_next_test()
        self.recording.set(["testrunner", "init"])
        if test is None:
            return RunnerManagerState.stop(True)
        else:
            return RunnerManagerState.initializing(subsuite, test_type, test, test_group, group_metadata, 0)

    def init(self):
        assert isinstance(self.state, RunnerManagerState.initializing)
        if self.state.failure_count > self.max_restarts:
            self.logger.critical("Max restarts exceeded")
            return RunnerManagerState.error()

        if (self.state.subsuite, self.state.test_type) != self.test_implementation_key:
            if self.browser is not None:
                assert self.browser.browser is not None
                self.browser.browser.cleanup()
            impl = self.test_implementations[(self.state.subsuite, self.state.test_type)]
            browser = impl.browser_cls(self.logger, remote_queue=self.command_queue,
                                       **impl.browser_kwargs)
            browser.setup()
            self.browser = BrowserManager(self.logger,
                                          browser,
                                          self.command_queue,
                                          no_timeout=self.debug_info is not None)
            self.test_implementation_key = (self.state.subsuite, self.state.test_type)

        assert self.browser is not None
        self.browser.update_settings(self.state.test)

        result = self.browser.init(self.state.group_metadata)
        if not result:
            return self.init_failed()

        self.start_test_runner()

    def start_test_runner(self):
        # Note that we need to be careful to start the browser before the
        # test runner to ensure that any state set when the browser is started
        # can be passed in to the test runner.
        assert isinstance(self.state, RunnerManagerState.initializing)
        assert self.command_queue is not None
        assert self.remote_queue is not None
        self.logger.info("Starting runner")
        impl = self.test_implementations[(self.state.subsuite, self.state.test_type)]
        self.executor_implementation = self.get_executor_implementation(impl)

        args = (self.remote_queue,
                self.command_queue,
                self.executor_implementation,
                self.capture_stdio,
                self.child_stop_flag,
                self.recording)

        mp = mpcontext.get_context()
        self.test_runner_proc = mp.Process(target=start_runner,
                                           args=args,
                                           name="TestRunner-%i" % self.manager_number)
        self.test_runner_proc.start()
        self.logger.debug("Test runner started")
        # Now we wait for either an init_succeeded event or an init_failed event

    def get_executor_implementation(self, impl):
        executor_kwargs = impl.executor_kwargs
        executor_kwargs["group_metadata"] = self.state.group_metadata
        executor_kwargs["browser_settings"] = self.browser.browser_settings
        executor_browser_cls, executor_browser_kwargs = self.browser.browser.executor_browser()
        return ExecutorImplementation(impl.executor_cls,
                                      executor_kwargs,
                                      executor_browser_cls,
                                      executor_browser_kwargs)

    def init_succeeded(self):
        assert isinstance(self.state, RunnerManagerState.initializing)
        self.browser.after_init()
        return RunnerManagerState.running(self.state.subsuite,
                                          self.state.test_type,
                                          self.state.test,
                                          self.state.test_group,
                                          self.state.group_metadata)

    def init_failed(self):
        assert isinstance(self.state, RunnerManagerState.initializing)
        self.browser.check_crash(None)
        self.browser.after_init()
        self.stop_runner(force=True)
        return RunnerManagerState.initializing(self.state.subsuite,
                                               self.state.test_type,
                                               self.state.test,
                                               self.state.test_group,
                                               self.state.group_metadata,
                                               self.state.failure_count + 1)

    def get_next_test(self):
        # returns test_type, test, test_group, group_metadata
        test = None
        test_group = None
        while test is None:
            while test_group is None or len(test_group) == 0:
                test_group, subsuite, test_type, group_metadata = self.test_source.group()
                if test_group is None:
                    self.logger.info("No more tests")
                    return None, None, None, None, None
            test = test_group.popleft()
        self.run_count = 0
        return subsuite, test_type, test, test_group, group_metadata

    def run_test(self):
        assert isinstance(self.state, RunnerManagerState.running)
        assert self.state.test is not None

        if self.browser.update_settings(self.state.test):
            self.logger.info("Restarting browser for new test environment")
            return RunnerManagerState.restarting(self.state.subsuite,
                                                 self.state.test_type,
                                                 self.state.test,
                                                 self.state.test_group,
                                                 self.state.group_metadata,
                                                 False)

        self.recording.set(["testrunner", "test"] + self.state.test.id.split("/")[1:])
        self.logger.test_start(self.state.test.id, subsuite=self.state.subsuite)
        if self.rerun > 1:
            self.logger.info(f"Run {self.run_count + 1}/{self.rerun}")
            self.send_message("reset")
        self.run_count += 1
        if self.debug_info is None:
            # Factor of 3 on the extra timeout here is based on allowing the executor
            # at least test.timeout + 2 * extra_timeout to complete,
            # which in turn is based on having several layers of timeout inside the executor
            timeout_multiplier = self.executor_implementation.executor_kwargs['timeout_multiplier']
            wait_timeout = (self.state.test.timeout * timeout_multiplier +
                            3 * self.executor_implementation.executor_cls.extra_timeout)
            self.timer = threading.Timer(wait_timeout, self._timeout)
            self.timer.name = f"{self.name}-timeout"

        self.send_message("run_test", self.state.test)
        if self.timer:
            self.timer.start()

    def _timeout(self):
        # This is executed in a different thread (threading.Timer).
        self.logger.info("Got timeout in harness")
        test = self.state.test
        self.inject_message(
            "test_ended",
            test,
            (test.make_result("EXTERNAL-TIMEOUT",
                              "TestRunner hit external timeout "
                              "(this may indicate a hang)"), []),
        )

    def test_ended(self, test, results):
        """Handle the end of a test.

        Output the result of each subtest, and the result of the overall
        harness to the logs.
        """
        if ((not isinstance(self.state, RunnerManagerState.running)) or
            (test != self.state.test)):
            # Due to inherent race conditions in EXTERNAL-TIMEOUT, we might
            # receive multiple test_ended for a test (e.g. from both Executor
            # and TestRunner), in which case we ignore the duplicate message.
            self.logger.warning("Received unexpected test_ended for %s" % test)
            return
        if self.timer is not None:
            self.timer.cancel()

        # Write the result of each subtest
        file_result, test_results = results
        subtest_unexpected = False
        subtest_all_pass_or_expected = True
        for result in test_results:
            if test.disabled(result.name):
                continue
            expected = result.expected
            known_intermittent = result.known_intermittent
            is_unexpected = expected != result.status and result.status not in known_intermittent
            is_expected_notrun = (expected == "NOTRUN" or "NOTRUN" in known_intermittent)

            if not is_unexpected and result.status in ["FAIL", "PRECONDITION_FAILED"]:
                # subtest is expected FAIL or expected PRECONDITION_FAILED,
                # change result to unexpected if expected_fail_message does not
                # match
                expected_fail_message = test.expected_fail_message(result.name)
                if expected_fail_message is not None and result.message.strip() != expected_fail_message:
                    is_unexpected = True
                    if result.status in known_intermittent:
                        known_intermittent.remove(result.status)
                    elif len(known_intermittent) > 0:
                        expected = known_intermittent[0]
                        known_intermittent = known_intermittent[1:]
                    else:
                        expected = "PASS"

            if is_unexpected:
                subtest_unexpected = True

                if result.status != "PASS" and not is_expected_notrun:
                    # Any result against an expected "NOTRUN" should be treated
                    # as unexpected pass.
                    subtest_all_pass_or_expected = False

            self.logger.test_status(test.id,
                                    result.name,
                                    result.status,
                                    message=result.message,
                                    expected=expected,
                                    known_intermittent=known_intermittent,
                                    stack=result.stack,
                                    subsuite=self.state.subsuite)

        expected = file_result.expected
        known_intermittent = file_result.known_intermittent
        status = file_result.status

        if self.browser.check_crash(test.id) and status != "CRASH":
            if test.test_type in ["crashtest", "wdspec"] or status == "EXTERNAL-TIMEOUT":
                self.logger.info("Found a crash dump file; changing status to CRASH")
                status = "CRASH"
            else:
                self.logger.warning(f"Found a crash dump; should change status from {status} to CRASH but this causes instability")

        # We have a couple of status codes that are used internally, but not exposed to the
        # user. These are used to indicate that some possibly-broken state was reached
        # and we should restart the runner before the next test.
        # INTERNAL-ERROR indicates a Python exception was caught in the harness
        # EXTERNAL-TIMEOUT indicates we had to forcibly kill the browser from the harness
        # because the test didn't return a result after reaching the test-internal timeout
        status_subns = {"INTERNAL-ERROR": "ERROR",
                        "EXTERNAL-TIMEOUT": "TIMEOUT"}
        status = status_subns.get(status, status)

        self.test_count += 1
        is_unexpected = expected != status and status not in known_intermittent
        is_pass_or_expected = status in ["OK", "PASS"] or (not is_unexpected)

        # A result is unexpected pass if the test or any subtest run
        # unexpectedly, and the overall status is expected or passing (OK for test
        # harness test, or PASS for reftest), and all unexpected results for
        # subtests (if any) are unexpected pass.
        is_unexpected_pass = ((is_unexpected or subtest_unexpected) and
                              is_pass_or_expected and subtest_all_pass_or_expected)
        if is_unexpected_pass:
            self.unexpected_pass_tests[self.state.subsuite, test.test_type].append(test)
        elif is_unexpected or subtest_unexpected:
            self.unexpected_fail_tests[self.state.subsuite, test.test_type].append(test)

        if "assertion_count" in file_result.extra:
            assertion_count = file_result.extra["assertion_count"]
            if assertion_count is not None and assertion_count > 0:
                self.logger.assertion_count(test.id,
                                            int(assertion_count),
                                            test.min_assertion_count,
                                            test.max_assertion_count)

        timeout_multiplier = self.executor_implementation.executor_kwargs['timeout_multiplier']
        file_result.extra["test_timeout"] = test.timeout * timeout_multiplier
        if self.browser.browser_pid:
            file_result.extra["browser_pid"] = self.browser.browser_pid

        self.logger.test_end(test.id,
                             status,
                             message=file_result.message,
                             expected=expected,
                             known_intermittent=known_intermittent,
                             extra=file_result.extra,
                             stack=file_result.stack,
                             subsuite=self.state.subsuite)

        restart_before_next = (self.retry_index > 0 or test.restart_after or
                               file_result.status in ("CRASH", "EXTERNAL-TIMEOUT", "INTERNAL-ERROR") or
                               ((subtest_unexpected or is_unexpected) and
                                self.restart_on_unexpected))
        force_stop = test.test_type == "wdspec" and file_result.status == "EXTERNAL-TIMEOUT"

        self.recording.set(["testrunner", "after-test"])
        if (not file_result.status == "CRASH" and
            self.pause_after_test or
            (self.pause_on_unexpected and (subtest_unexpected or is_unexpected))):
            self.logger.info("Pausing until the browser exits")
            self.send_message("wait")
        else:
            return self.after_test_end(test, restart_before_next, force_stop=force_stop)

    def wait_finished(self, rerun=False):
        assert isinstance(self.state, RunnerManagerState.running)
        self.logger.debug("Wait finished")

        # The browser should be stopped already, but this ensures we do any
        # post-stop processing
        return self.after_test_end(self.state.test, not rerun, force_rerun=rerun)

    def switch_executor_succeeded(self):
        assert isinstance(self.state, RunnerManagerState.switching_executor)
        return RunnerManagerState.running(self.state.subsuite,
                                          self.state.test_type,
                                          self.state.test,
                                          self.state.test_group,
                                          self.state.group_metadata)

    def switch_executor_failed(self):
        assert isinstance(self.state, RunnerManagerState.switching_executor)
        return RunnerManagerState.restarting(self.state.subsuite,
                                             self.state.test_type,
                                             self.state.test,
                                             self.state.test_group,
                                             self.state.group_metadata,
                                             False)

    def after_test_end(self, test, restart, force_rerun=False, force_stop=False):
        assert isinstance(self.state, RunnerManagerState.running)
        # Mixing manual reruns and automatic reruns is confusing; we currently assume
        # that as long as we've done at least the automatic run count in total we can
        # continue with the next test.
        if not force_rerun and self.run_count >= self.rerun:
            subsuite, test_type, test, test_group, group_metadata = self.get_next_test()
            if test is None:
                return RunnerManagerState.stop(force_stop)
            if subsuite != self.state.subsuite:
                self.logger.info(f"Restarting browser for new subsuite:{subsuite}")
                restart = True
            elif self.restart_on_new_group and test_group is not self.state.test_group:
                self.logger.info("Restarting browser for new test group")
                restart = True
            elif test_type != self.state.test_type:
                if self.browser.browser.restart_on_test_type_change(test_type, self.state.test_type):
                    self.logger.info(f"Restarting browser for new test type:{test_type}")
                    restart = True
                else:
                    self.logger.info(f"Switching executor for new test type: {self.state.test_type} => {test_type}")
                    impl = self.test_implementations[subsuite, test_type]
                    self.executor_implementation = self.get_executor_implementation(impl)
                    self.send_message("switch_executor", self.executor_implementation)
                    return RunnerManagerState.switching_executor(
                        subsuite, test_type, test, test_group, group_metadata)
        else:
            subsuite = self.state.subsuite
            test_type = self.state.test_type
            test_group = self.state.test_group
            group_metadata = self.state.group_metadata

        if restart:
            return RunnerManagerState.restarting(
                subsuite, test_type, test, test_group, group_metadata, force_stop)
        else:
            return RunnerManagerState.running(
                subsuite, test_type, test, test_group, group_metadata)

    def restart_runner(self):
        """Stop and restart the TestRunner"""
        assert isinstance(self.state, RunnerManagerState.restarting)
        self.stop_runner(force=self.state.force_stop)
        return RunnerManagerState.initializing(
            self.state.subsuite, self.state.test_type, self.state.test,
            self.state.test_group, self.state.group_metadata, 0)

    def log(self, data: Mapping[str, Any]) -> None:
        self.logger.log_raw(data)

    def error(self, message):
        self.logger.error(message)
        self.restart_runner()

    def stop_runner(self, force=False):
        """Stop the TestRunner and the browser binary."""
        self.recording.set(["testrunner", "stop_runner"])
        try:
            # Stop the runner process before the browser process so that the
            # former can gracefully tear down the protocol (e.g., closing an
            # active WebDriver session).
            self._ensure_runner_stopped()
            # TODO(web-platform-tests/wpt#48030): Consider removing the
            # `stop(force=...)` argument.
            if self.browser:
                self.browser.stop(force=True)
        except (OSError, PermissionError):
            self.logger.error("Failed to stop either the runner or the browser process",
                              exc_info=True)
        finally:
            self.cleanup()

    def teardown(self):
        self.logger.debug("TestRunnerManager teardown")
        self.command_queue.close()
        self.remote_queue.close()
        self.command_queue = None
        self.remote_queue = None
        self.recording.pause()

    def _ensure_runner_stopped(self):
        if self.test_runner_proc is None:
            return
        self.logger.debug("Stopping runner process")
        self.send_message("stop")
        self.test_runner_proc.join(10)
        mp = mpcontext.get_context()
        if self.test_runner_proc.is_alive():
            # This might leak a file handle from the queue
            self.logger.warning("Forcibly terminating runner process")
            self.test_runner_proc.terminate()
            self.logger.debug("After terminating runner process")

            # Multiprocessing queues are backed by operating system pipes. If
            # the pipe in the child process had buffered data at the time of
            # forced termination, the queue is no longer in a usable state
            # (subsequent attempts to retrieve items may block indefinitely).
            # Discard the potentially-corrupted queue and create a new one.
            self.logger.debug("Recreating command queue")
            self.command_queue.cancel_join_thread()
            self.command_queue.close()
            self.command_queue = mp.Queue()
            self.logger.debug("Recreating remote queue")
            self.remote_queue.cancel_join_thread()
            self.remote_queue.close()
            self.remote_queue = mp.Queue()
        else:
            self.logger.debug("Runner process exited with code %i" % self.test_runner_proc.exitcode)
        self.test_runner_proc = None

    def send_message(self, command, *args):
        """Send a message to the remote queue (to Executor)."""
        self.remote_queue.put((command, args))

    def inject_message(self, command, *args):
        """Inject a message to the command queue (from Executor)."""
        self.command_queue.put((command, args))

    def cleanup(self):
        self.logger.debug("TestRunnerManager cleanup")
        if self.browser:
            self.browser.cleanup()
        if self.timer:
            self.timer.cancel()
        while True:
            try:
                cmd, data = self.command_queue.get_nowait()
            except Empty:
                break
            else:
                if cmd == "log":
                    self.log(*data)
                else:
                    self.logger.warning(f"Command left in command_queue during cleanup: {cmd!r}, {data!r}")
        while True:
            try:
                cmd, data = self.remote_queue.get_nowait()
                self.logger.warning(f"Command left in remote_queue during cleanup: {cmd!r}, {data!r}")
            except Empty:
                break


class ManagerGroup:
    """Main thread object that owns all the TestRunnerManager threads."""
    def __init__(self, suite_name, test_queue_builder, test_implementations,
                 retry_index=0,
                 rerun=1,
                 pause_after_test=False,
                 pause_on_unexpected=False,
                 restart_on_unexpected=True,
                 debug_info=None,
                 capture_stdio=True,
                 restart_on_new_group=True,
                 recording=None,
                 max_restarts=5):
        self.suite_name = suite_name
        self.test_queue_builder = test_queue_builder
        self.test_implementations = test_implementations
        self.pause_after_test = pause_after_test
        self.pause_on_unexpected = pause_on_unexpected
        self.restart_on_unexpected = restart_on_unexpected
        self.debug_info = debug_info
        self.retry_index = retry_index
        self.rerun = rerun
        self.capture_stdio = capture_stdio
        self.restart_on_new_group = restart_on_new_group
        self.recording = recording
        assert recording is not None
        self.max_restarts = max_restarts

        self.pool = set()
        self.stop_flag = None
        self.logger = structuredlog.StructuredLogger(suite_name)

    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        self.stop()

    def run(self, tests):
        """Start all managers in the group"""
        test_queue, size = self.test_queue_builder.make_queue(tests)
        self.logger.info("Using %i child processes" % size)
        self.stop_flag = StopFlag(size)

        for idx in range(size):
            manager = TestRunnerManager(self.suite_name,
                                        idx,
                                        test_queue,
                                        self.test_implementations,
                                        self.stop_flag,
                                        self.retry_index,
                                        self.rerun,
                                        self.pause_after_test,
                                        self.pause_on_unexpected,
                                        self.restart_on_unexpected,
                                        self.debug_info,
                                        self.capture_stdio,
                                        self.restart_on_new_group,
                                        recording=self.recording,
                                        max_restarts=self.max_restarts)
            manager.start()
            self.pool.add(manager)
        self.wait()

    def wait(self, timeout: Optional[float] = None) -> None:
        """Wait for all the managers in the group to finish.

        Arguments:
            timeout: Overall timeout (in seconds) for all threads to join. The
                default value indicates an indefinite timeout.
        """
        # Here, the main thread cannot simply `join()` the threads in
        # `self.pool` sequentially because a keyboard interrupt raised during a
        # `Thread.join()` may incorrectly mark that thread as "stopped" when it
        # is not [0, 1]. Subsequent `join()`s for the affected thread won't
        # block anymore, so a subsequent `ManagerGroup.wait()` may return with
        # that thread still alive.
        #
        # To the extent the timeout allows, it's important that
        # `ManagerGroup.wait()` returns with all `TestRunnerManager` threads
        # actually stopped. Otherwise, a live thread may log after `mozlog`
        # shutdown (not allowed) or worse, leak browser processes that the
        # thread should have stopped when exiting its run loop [2].
        #
        # [0]: https://github.com/python/cpython/issues/90882
        # [1]: https://github.com/python/cpython/blob/558b517b/Lib/threading.py#L1146-L1178
        # [2]: https://crbug.com/330236796
        assert self.stop_flag, "ManagerGroup hasn't been started yet"
        self.stop_flag.wait_for_all_managers_done(timeout)

    def stop(self):
        """Set the stop flag so that all managers in the group stop as soon
        as possible"""
        if self.stop_flag:
            self.stop_flag.stop()
            self.logger.debug("Stop flag set in ManagerGroup")

    def test_count(self):
        return sum(manager.test_count for manager in self.pool)

    def unexpected_fail_tests(self):
        rv = defaultdict(list)
        for manager in self.pool:
            for (subsuite, test_type), tests in manager.unexpected_fail_tests.items():
                rv[(subsuite, test_type)].extend(tests)
        return rv

    def unexpected_pass_tests(self):
        rv = defaultdict(list)
        for manager in self.pool:
            for (subsuite, test_type), tests in manager.unexpected_pass_tests.items():
                rv[(subsuite, test_type)].extend(tests)
        return rv
