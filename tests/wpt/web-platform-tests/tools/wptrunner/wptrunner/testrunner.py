# mypy: allow-untyped-defs

import threading
import traceback
from queue import Empty
from collections import namedtuple

from mozlog import structuredlog, capture

from . import mpcontext

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
    :param command_queue: subprocess.Queue used to send commands to the
                          process
    :param result_queue: subprocess.Queue used to send results to the
                         parent TestRunnerManager process
    :param executor: TestExecutor object that will actually run a test.
    """
    def __init__(self, logger, command_queue, result_queue, executor, recording):
        self.command_queue = command_queue
        self.result_queue = result_queue

        self.executor = executor
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
        self.send_message("runner_teardown")
        self.result_queue = None
        self.command_queue = None
        self.browser = None

    def run(self):
        """Main loop accepting commands over the pipe and triggering
        the associated methods"""
        try:
            self.setup()
        except Exception:
            self.logger.warning("An error occured during executor setup:\n%s" %
                                traceback.format_exc())
            raise
        commands = {"run_test": self.run_test,
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
            self.logger.error(traceback.format_exc())
            raise

    def wait(self):
        rerun = self.executor.wait()
        self.send_message("wait_finished", rerun)

    def send_message(self, command, *args):
        self.result_queue.put((command, args))


def start_runner(runner_command_queue, runner_result_queue,
                 executor_cls, executor_kwargs,
                 executor_browser_cls, executor_browser_kwargs,
                 capture_stdio, stop_flag, recording):
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
            browser = executor_browser_cls(**executor_browser_kwargs)
            executor = executor_cls(logger, browser, **executor_kwargs)
            with TestRunner(logger, runner_command_queue, runner_result_queue, executor, recording) as runner:
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
            self.logger.warning("Failure during init %s" % traceback.format_exc())
            if self.init_timer is not None:
                self.init_timer.cancel()
            self.logger.error(traceback.format_exc())
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


class _RunnerManagerState:
    before_init = namedtuple("before_init", [])
    initializing = namedtuple("initializing",
                              ["test", "test_group", "group_metadata", "failure_count"])
    running = namedtuple("running", ["test", "test_group", "group_metadata"])
    restarting = namedtuple("restarting", ["test", "test_group", "group_metadata", "force_stop"])
    error = namedtuple("error", [])
    stop = namedtuple("stop", ["force_stop"])


RunnerManagerState = _RunnerManagerState()


class TestRunnerManager(threading.Thread):
    def __init__(self, suite_name, index, test_type, test_queue, test_source_cls, browser_cls,
                 browser_kwargs, executor_cls, executor_kwargs, stop_flag, rerun=1,
                 pause_after_test=False, pause_on_unexpected=False, restart_on_unexpected=True,
                 debug_info=None, capture_stdio=True, recording=None):
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

        self.test_source = test_source_cls(test_queue)

        self.manager_number = index
        self.test_type = test_type
        self.browser_cls = browser_cls
        self.browser_kwargs = browser_kwargs.copy()
        if self.browser_kwargs.get("device_serial"):
            # Assign Android device to runner according to current manager index
            self.browser_kwargs["device_serial"] = (
                self.browser_kwargs["device_serial"][index])

        self.executor_cls = executor_cls
        self.executor_kwargs = executor_kwargs

        mp = mpcontext.get_context()

        # Flags used to shut down this thread if we get a sigint
        self.parent_stop_flag = stop_flag
        self.child_stop_flag = mp.Event()

        self.rerun = rerun
        self.run_count = 0
        self.pause_after_test = pause_after_test
        self.pause_on_unexpected = pause_on_unexpected
        self.restart_on_unexpected = restart_on_unexpected
        self.debug_info = debug_info

        assert recording is not None
        self.recording = recording

        self.command_queue = mp.Queue()
        self.remote_queue = mp.Queue()

        self.test_runner_proc = None

        threading.Thread.__init__(self, name="TestRunnerManager-%s-%i" % (test_type, index))
        # This is started in the actual new thread
        self.logger = None

        self.test_count = 0
        self.unexpected_count = 0
        self.unexpected_pass_count = 0

        # This may not really be what we want
        self.daemon = True

        self.timer = None

        self.max_restarts = 5

        self.browser = None

        self.capture_stdio = capture_stdio

    def run(self):
        """Main loop for the TestRunnerManager.

        TestRunnerManagers generally receive commands from their
        TestRunner updating them on the status of a test. They
        may also have a stop flag set by the main thread indicating
        that the manager should shut down the next time the event loop
        spins."""
        self.recording.set(["testrunner", "startup"])
        self.logger = structuredlog.StructuredLogger(self.suite_name)
        with self.browser_cls(self.logger, remote_queue=self.command_queue,
                              **self.browser_kwargs) as browser:
            self.browser = BrowserManager(self.logger,
                                          browser,
                                          self.command_queue,
                                          no_timeout=self.debug_info is not None)
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
                        self.logger.debug("Dispatch %s" % f.__name__)
                        if self.should_stop():
                            return
                        new_state = f()
                        if new_state is None:
                            break
                        self.state = new_state
                        self.logger.debug("new state: %s" % self.state.__class__.__name__)
                        if isinstance(self.state, end_states):
                            return
                        f = dispatch.get(self.state.__class__)

                    new_state = None
                    while new_state is None:
                        new_state = self.wait_event()
                        if self.should_stop():
                            return
                    self.state = new_state
                    self.logger.debug("new state: %s" % self.state.__class__.__name__)
            except Exception:
                self.logger.error(traceback.format_exc())
                raise
            finally:
                self.logger.debug("TestRunnerManager main loop terminating, starting cleanup")
                force_stop = not isinstance(self.state, RunnerManagerState.stop) or self.state.force_stop
                self.stop_runner(force=force_stop)
                self.teardown()
        self.logger.debug("TestRunnerManager main loop terminated")

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
            RunnerManagerState.restarting: {},
            RunnerManagerState.error: {},
            RunnerManagerState.stop: {},
            None: {
                "runner_teardown": self.runner_teardown,
                "log": self.log,
                "error": self.error
            }
        }
        try:
            command, data = self.command_queue.get(True, 1)
            self.logger.debug("Got command: %r" % command)
        except OSError:
            self.logger.error("Got IOError from poll")
            return RunnerManagerState.restarting(self.state.test,
                                                 self.state.test_group,
                                                 self.state.group_metadata,
                                                 False)
        except Empty:
            if (self.debug_info and self.debug_info.interactive and
                self.browser.started and not self.browser.is_alive()):
                self.logger.debug("Debugger exited")
                return RunnerManagerState.stop(False)

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
                return RunnerManagerState.restarting(self.state.test,
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
        return self.child_stop_flag.is_set() or self.parent_stop_flag.is_set()

    def start_init(self):
        test, test_group, group_metadata = self.get_next_test()
        self.recording.set(["testrunner", "init"])
        if test is None:
            return RunnerManagerState.stop(True)
        else:
            return RunnerManagerState.initializing(test, test_group, group_metadata, 0)

    def init(self):
        assert isinstance(self.state, RunnerManagerState.initializing)
        if self.state.failure_count > self.max_restarts:
            self.logger.critical("Max restarts exceeded")
            return RunnerManagerState.error()

        self.browser.update_settings(self.state.test)

        result = self.browser.init(self.state.group_metadata)
        if result is Stop:
            return RunnerManagerState.error()
        elif not result:
            return RunnerManagerState.initializing(self.state.test,
                                                   self.state.test_group,
                                                   self.state.group_metadata,
                                                   self.state.failure_count + 1)
        else:
            self.executor_kwargs["group_metadata"] = self.state.group_metadata
            self.executor_kwargs["browser_settings"] = self.browser.browser_settings
            self.start_test_runner()

    def start_test_runner(self):
        # Note that we need to be careful to start the browser before the
        # test runner to ensure that any state set when the browser is started
        # can be passed in to the test runner.
        assert isinstance(self.state, RunnerManagerState.initializing)
        assert self.command_queue is not None
        assert self.remote_queue is not None
        self.logger.info("Starting runner")
        executor_browser_cls, executor_browser_kwargs = self.browser.browser.executor_browser()

        args = (self.remote_queue,
                self.command_queue,
                self.executor_cls,
                self.executor_kwargs,
                executor_browser_cls,
                executor_browser_kwargs,
                self.capture_stdio,
                self.child_stop_flag,
                self.recording)

        mp = mpcontext.get_context()
        self.test_runner_proc = mp.Process(target=start_runner,
                                           args=args,
                                           name="TestRunner-%s-%i" % (
                                               self.test_type, self.manager_number))
        self.test_runner_proc.start()
        self.logger.debug("Test runner started")
        # Now we wait for either an init_succeeded event or an init_failed event

    def init_succeeded(self):
        assert isinstance(self.state, RunnerManagerState.initializing)
        self.browser.after_init()
        return RunnerManagerState.running(self.state.test,
                                          self.state.test_group,
                                          self.state.group_metadata)

    def init_failed(self):
        assert isinstance(self.state, RunnerManagerState.initializing)
        self.browser.check_crash(None)
        self.browser.after_init()
        self.stop_runner(force=True)
        return RunnerManagerState.initializing(self.state.test,
                                               self.state.test_group,
                                               self.state.group_metadata,
                                               self.state.failure_count + 1)

    def get_next_test(self, test_group=None):
        test = None
        while test is None:
            while test_group is None or len(test_group) == 0:
                test_group, group_metadata = self.test_source.group()
                if test_group is None:
                    self.logger.info("No more tests")
                    return None, None, None
            test = test_group.popleft()
        self.run_count = 0
        return test, test_group, group_metadata

    def run_test(self):
        assert isinstance(self.state, RunnerManagerState.running)
        assert self.state.test is not None

        if self.browser.update_settings(self.state.test):
            self.logger.info("Restarting browser for new test environment")
            return RunnerManagerState.restarting(self.state.test,
                                                 self.state.test_group,
                                                 self.state.group_metadata,
                                                 False)

        self.recording.set(["testrunner", "test"] + self.state.test.id.split("/")[1:])
        self.logger.test_start(self.state.test.id)
        if self.rerun > 1:
            self.logger.info("Run %d/%d" % (self.run_count, self.rerun))
            self.send_message("reset")
        self.run_count += 1
        if self.debug_info is None:
            # Factor of 3 on the extra timeout here is based on allowing the executor
            # at least test.timeout + 2 * extra_timeout to complete,
            # which in turn is based on having several layers of timeout inside the executor
            wait_timeout = (self.state.test.timeout * self.executor_kwargs['timeout_multiplier'] +
                            3 * self.executor_cls.extra_timeout)
            self.timer = threading.Timer(wait_timeout, self._timeout)

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
            (test.result_cls("EXTERNAL-TIMEOUT",
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
            self.logger.error("Received unexpected test_ended for %s" % test)
            return
        if self.timer is not None:
            self.timer.cancel()

        # Write the result of each subtest
        file_result, test_results = results
        subtest_unexpected = False
        expect_any_subtest_status = test.expect_any_subtest_status()
        if expect_any_subtest_status:
            self.logger.debug("Ignoring subtest statuses for test %s" % test.id)
        for result in test_results:
            if test.disabled(result.name):
                continue
            if expect_any_subtest_status:
                expected = result.status
            else:
                expected = test.expected(result.name)
            known_intermittent = test.known_intermittent(result.name)
            is_unexpected = expected != result.status and result.status not in known_intermittent

            if is_unexpected:
                self.unexpected_count += 1
                self.logger.debug("Unexpected count in this thread %i" % self.unexpected_count)
                subtest_unexpected = True

            is_unexpected_pass = is_unexpected and result.status == "PASS"
            if is_unexpected_pass:
                self.unexpected_pass_count += 1

            self.logger.test_status(test.id,
                                    result.name,
                                    result.status,
                                    message=result.message,
                                    expected=expected,
                                    known_intermittent=known_intermittent,
                                    stack=result.stack)

        expected = test.expected()
        known_intermittent = test.known_intermittent()
        status = file_result.status

        if self.browser.check_crash(test.id) and status != "CRASH":
            if test.test_type == "crashtest" or status == "EXTERNAL-TIMEOUT":
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
        if is_unexpected:
            self.unexpected_count += 1
            self.logger.debug("Unexpected count in this thread %i" % self.unexpected_count)

        is_unexpected_pass = is_unexpected and status == "OK"
        if is_unexpected_pass:
            self.unexpected_pass_count += 1

        if "assertion_count" in file_result.extra:
            assertion_count = file_result.extra["assertion_count"]
            if assertion_count is not None and assertion_count > 0:
                self.logger.assertion_count(test.id,
                                            int(assertion_count),
                                            test.min_assertion_count,
                                            test.max_assertion_count)

        file_result.extra["test_timeout"] = test.timeout * self.executor_kwargs['timeout_multiplier']

        self.logger.test_end(test.id,
                             status,
                             message=file_result.message,
                             expected=expected,
                             known_intermittent=known_intermittent,
                             extra=file_result.extra,
                             stack=file_result.stack)

        restart_before_next = (test.restart_after or
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

    def after_test_end(self, test, restart, force_rerun=False, force_stop=False):
        assert isinstance(self.state, RunnerManagerState.running)
        # Mixing manual reruns and automatic reruns is confusing; we currently assume
        # that as long as we've done at least the automatic run count in total we can
        # continue with the next test.
        if not force_rerun and self.run_count >= self.rerun:
            test, test_group, group_metadata = self.get_next_test()
            if test is None:
                return RunnerManagerState.stop(force_stop)
            if test_group is not self.state.test_group:
                # We are starting a new group of tests, so force a restart
                self.logger.info("Restarting browser for new test group")
                restart = True
        else:
            test_group = self.state.test_group
            group_metadata = self.state.group_metadata
        if restart:
            return RunnerManagerState.restarting(test, test_group, group_metadata, force_stop)
        else:
            return RunnerManagerState.running(test, test_group, group_metadata)

    def restart_runner(self):
        """Stop and restart the TestRunner"""
        assert isinstance(self.state, RunnerManagerState.restarting)
        self.stop_runner(force=self.state.force_stop)
        return RunnerManagerState.initializing(self.state.test, self.state.test_group, self.state.group_metadata, 0)

    def log(self, data):
        self.logger.log_raw(data)

    def error(self, message):
        self.logger.error(message)
        self.restart_runner()

    def stop_runner(self, force=False):
        """Stop the TestRunner and the browser binary."""
        self.recording.set(["testrunner", "stop_runner"])
        if self.test_runner_proc is None:
            return

        if self.test_runner_proc.is_alive():
            self.send_message("stop")
        try:
            self.browser.stop(force=force)
            self.ensure_runner_stopped()
        finally:
            self.cleanup()

    def teardown(self):
        self.logger.debug("TestRunnerManager teardown")
        self.test_runner_proc = None
        self.command_queue.close()
        self.remote_queue.close()
        self.command_queue = None
        self.remote_queue = None
        self.recording.pause()

    def ensure_runner_stopped(self):
        self.logger.debug("ensure_runner_stopped")
        if self.test_runner_proc is None:
            return

        self.browser.stop(force=True)
        self.logger.debug("waiting for runner process to end")
        self.test_runner_proc.join(10)
        self.logger.debug("After join")
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

    def runner_teardown(self):
        self.ensure_runner_stopped()
        return RunnerManagerState.stop(False)

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
        while True:
            try:
                cmd, data = self.command_queue.get_nowait()
            except Empty:
                break
            else:
                if cmd == "log":
                    self.log(*data)
                elif cmd == "runner_teardown":
                    # It's OK for the "runner_teardown" message to be left in
                    # the queue during cleanup, as we will already have tried
                    # to stop the TestRunner in `stop_runner`.
                    pass
                else:
                    self.logger.warning(f"Command left in command_queue during cleanup: {cmd!r}, {data!r}")
        while True:
            try:
                cmd, data = self.remote_queue.get_nowait()
                self.logger.warning(f"Command left in remote_queue during cleanup: {cmd!r}, {data!r}")
            except Empty:
                break


def make_test_queue(tests, test_source_cls, **test_source_kwargs):
    queue = test_source_cls.make_queue(tests, **test_source_kwargs)

    # There is a race condition that means sometimes we continue
    # before the tests have been written to the underlying pipe.
    # Polling the pipe for data here avoids that
    queue._reader.poll(10)
    assert not queue.empty()
    return queue


class ManagerGroup:
    """Main thread object that owns all the TestRunnerManager threads."""
    def __init__(self, suite_name, size, test_source_cls, test_source_kwargs,
                 browser_cls, browser_kwargs,
                 executor_cls, executor_kwargs,
                 rerun=1,
                 pause_after_test=False,
                 pause_on_unexpected=False,
                 restart_on_unexpected=True,
                 debug_info=None,
                 capture_stdio=True,
                 recording=None):
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
        self.restart_on_unexpected = restart_on_unexpected
        self.debug_info = debug_info
        self.rerun = rerun
        self.capture_stdio = capture_stdio
        self.recording = recording
        assert recording is not None

        self.pool = set()
        # Event that is polled by threads so that they can gracefully exit in the face
        # of sigint
        self.stop_flag = threading.Event()
        self.logger = structuredlog.StructuredLogger(suite_name)

    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        self.stop()

    def run(self, test_type, tests):
        """Start all managers in the group"""
        self.logger.debug("Using %i processes" % self.size)
        type_tests = tests[test_type]
        if not type_tests:
            self.logger.info("No %s tests to run" % test_type)
            return

        test_queue = make_test_queue(type_tests, self.test_source_cls, **self.test_source_kwargs)

        for idx in range(self.size):
            manager = TestRunnerManager(self.suite_name,
                                        idx,
                                        test_type,
                                        test_queue,
                                        self.test_source_cls,
                                        self.browser_cls,
                                        self.browser_kwargs,
                                        self.executor_cls,
                                        self.executor_kwargs,
                                        self.stop_flag,
                                        self.rerun,
                                        self.pause_after_test,
                                        self.pause_on_unexpected,
                                        self.restart_on_unexpected,
                                        self.debug_info,
                                        self.capture_stdio,
                                        recording=self.recording)
            manager.start()
            self.pool.add(manager)
        self.wait()

    def wait(self):
        """Wait for all the managers in the group to finish"""
        for manager in self.pool:
            manager.join()

    def stop(self):
        """Set the stop flag so that all managers in the group stop as soon
        as possible"""
        self.stop_flag.set()
        self.logger.debug("Stop flag set in ManagerGroup")

    def test_count(self):
        return sum(manager.test_count for manager in self.pool)

    def unexpected_count(self):
        return sum(manager.unexpected_count for manager in self.pool)

    def unexpected_pass_count(self):
        return sum(manager.unexpected_pass_count for manager in self.pool)
