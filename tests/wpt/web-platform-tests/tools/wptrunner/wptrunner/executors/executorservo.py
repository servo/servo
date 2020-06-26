from __future__ import print_function
import base64
import json
import os
import subprocess
import tempfile
import threading
import traceback
import uuid
from six import ensure_str, iteritems

from mozprocess import ProcessHandler

from tools.serve.serve import make_hosts_file

from .base import (ConnectionlessProtocol,
                   RefTestImplementation,
                   crashtest_result_converter,
                   testharness_result_converter,
                   reftest_result_converter,
                   TimedRunner,
                   WdspecExecutor,
                   WdspecProtocol)
from .process import ProcessTestExecutor
from ..browsers.base import browser_command
from ..process import cast_env
from ..webdriver_server import ServoDriverServer


pytestrunner = None
webdriver = None


def write_hosts_file(config):
    hosts_fd, hosts_path = tempfile.mkstemp()
    with os.fdopen(hosts_fd, "w") as f:
        f.write(make_hosts_file(config, "127.0.0.1"))
    return hosts_path


def build_servo_command(test, test_url_func, browser, binary, pause_after_test, debug_info,
                        extra_args=None, debug_opts="replace-surrogates"):
    args = [
        "--hard-fail", "-u", "Servo/wptrunner",
        "-z", test_url_func(test),
    ]
    if debug_opts:
        args += ["-Z", debug_opts]
    for stylesheet in browser.user_stylesheets:
        args += ["--user-stylesheet", stylesheet]
    for pref, value in iteritems(test.environment.get('prefs', {})):
        args += ["--pref", "%s=%s" % (pref, value)]
    if browser.ca_certificate_path:
        args += ["--certificate-path", browser.ca_certificate_path]
    if extra_args:
        args += extra_args
    args += browser.binary_args
    debug_args, command = browser_command(binary, args, debug_info)
    if pause_after_test:
        command.remove("-z")
    return debug_args + command



class ServoTestharnessExecutor(ProcessTestExecutor):
    convert_result = testharness_result_converter

    def __init__(self, logger, browser, server_config, timeout_multiplier=1, debug_info=None,
                 pause_after_test=False, **kwargs):
        ProcessTestExecutor.__init__(self, logger, browser, server_config,
                                     timeout_multiplier=timeout_multiplier,
                                     debug_info=debug_info)
        self.pause_after_test = pause_after_test
        self.result_data = None
        self.result_flag = None
        self.protocol = ConnectionlessProtocol(self, browser)
        self.hosts_path = write_hosts_file(server_config)

    def teardown(self):
        try:
            os.unlink(self.hosts_path)
        except OSError:
            pass
        ProcessTestExecutor.teardown(self)

    def do_test(self, test):
        self.result_data = None
        self.result_flag = threading.Event()

        self.command = build_servo_command(test,
                                           self.test_url,
                                           self.browser,
                                           self.binary,
                                           self.pause_after_test,
                                           self.debug_info)

        env = os.environ.copy()
        env["HOST_FILE"] = self.hosts_path
        env["RUST_BACKTRACE"] = "1"


        if not self.interactive:
            self.proc = ProcessHandler(self.command,
                                       processOutputLine=[self.on_output],
                                       onFinish=self.on_finish,
                                       env=cast_env(env),
                                       storeOutput=False)
            self.proc.run()
        else:
            self.proc = subprocess.Popen(self.command, env=cast_env(env))

        try:
            timeout = test.timeout * self.timeout_multiplier

            # Now wait to get the output we expect, or until we reach the timeout
            if not self.interactive and not self.pause_after_test:
                wait_timeout = timeout + 5
                self.result_flag.wait(wait_timeout)
            else:
                wait_timeout = None
                self.proc.wait()

            proc_is_running = True

            if self.result_flag.is_set():
                if self.result_data is not None:
                    result = self.convert_result(test, self.result_data)
                else:
                    self.proc.wait()
                    result = (test.result_cls("CRASH", None), [])
                    proc_is_running = False
            else:
                result = (test.result_cls("TIMEOUT", None), [])


            if proc_is_running:
                if self.pause_after_test:
                    self.logger.info("Pausing until the browser exits")
                    self.proc.wait()
                else:
                    self.proc.kill()
        except:  # noqa
            self.proc.kill()
            raise

        return result

    def on_output(self, line):
        prefix = "ALERT: RESULT: "
        line = line.decode("utf8", "replace")
        if line.startswith(prefix):
            self.result_data = json.loads(line[len(prefix):])
            self.result_flag.set()
        else:
            if self.interactive:
                print(line)
            else:
                self.logger.process_output(self.proc.pid,
                                           line,
                                           " ".join(self.command))

    def on_finish(self):
        self.result_flag.set()


class TempFilename(object):
    def __init__(self, directory):
        self.directory = directory
        self.path = None

    def __enter__(self):
        self.path = os.path.join(self.directory, str(uuid.uuid4()))
        return self.path

    def __exit__(self, *args, **kwargs):
        try:
            os.unlink(self.path)
        except OSError:
            pass


class ServoRefTestExecutor(ProcessTestExecutor):
    convert_result = reftest_result_converter

    def __init__(self, logger, browser, server_config, binary=None, timeout_multiplier=1,
                 screenshot_cache=None, debug_info=None, pause_after_test=False,
                 **kwargs):
        ProcessTestExecutor.__init__(self,
                                     logger,
                                     browser,
                                     server_config,
                                     timeout_multiplier=timeout_multiplier,
                                     debug_info=debug_info)

        self.protocol = ConnectionlessProtocol(self, browser)
        self.screenshot_cache = screenshot_cache
        self.implementation = RefTestImplementation(self)
        self.tempdir = tempfile.mkdtemp()
        self.hosts_path = write_hosts_file(server_config)

    def reset(self):
        self.implementation.reset()

    def teardown(self):
        try:
            os.unlink(self.hosts_path)
        except OSError:
            pass
        os.rmdir(self.tempdir)
        ProcessTestExecutor.teardown(self)

    def screenshot(self, test, viewport_size, dpi, page_ranges):
        with TempFilename(self.tempdir) as output_path:
            extra_args = ["--exit",
                          "--output=%s" % output_path,
                          "--resolution", viewport_size or "800x600"]
            debug_opts = "disable-text-aa,load-webfonts-synchronously,replace-surrogates"

            if dpi:
                extra_args += ["--device-pixel-ratio", dpi]

            self.command = build_servo_command(test,
                                               self.test_url,
                                               self.browser,
                                               self.binary,
                                               False,
                                               self.debug_info,
                                               extra_args,
                                               debug_opts)

            env = os.environ.copy()
            env["HOST_FILE"] = self.hosts_path
            env["RUST_BACKTRACE"] = "1"

            if not self.interactive:
                self.proc = ProcessHandler(self.command,
                                           processOutputLine=[self.on_output],
                                           env=cast_env(env))


                try:
                    self.proc.run()
                    timeout = test.timeout * self.timeout_multiplier + 5
                    rv = self.proc.wait(timeout=timeout)
                except KeyboardInterrupt:
                    self.proc.kill()
                    raise
            else:
                self.proc = subprocess.Popen(self.command,
                                             env=cast_env(env))
                try:
                    rv = self.proc.wait()
                except KeyboardInterrupt:
                    self.proc.kill()
                    raise

            if rv is None:
                self.proc.kill()
                return False, ("EXTERNAL-TIMEOUT", None)

            if rv != 0 or not os.path.exists(output_path):
                return False, ("CRASH", None)

            with open(output_path, "rb") as f:
                # Might need to strip variable headers or something here
                data = f.read()
                return True, [ensure_str(base64.b64encode(data))]

    def do_test(self, test):
        result = self.implementation.run_test(test)

        return self.convert_result(test, result)

    def on_output(self, line):
        line = line.decode("utf8", "replace")
        if self.interactive:
            print(line)
        else:
            self.logger.process_output(self.proc.pid,
                                       line,
                                       " ".join(self.command))


class ServoDriverProtocol(WdspecProtocol):
    server_cls = ServoDriverServer


class ServoWdspecExecutor(WdspecExecutor):
    protocol_cls = ServoDriverProtocol


class ServoTimedRunner(TimedRunner):
    def run_func(self):
        try:
            self.result = True, self.func(self.protocol, self.url, self.timeout)
        except Exception as e:
            message = getattr(e, "message", "")
            if message:
                message += "\n"
            message += traceback.format_exc(e)
            self.result = False, ("INTERNAL-ERROR", message)
        finally:
            self.result_flag.set()

    def set_timeout(self):
        pass


class ServoCrashtestExecutor(ProcessTestExecutor):
    convert_result = crashtest_result_converter

    def __init__(self, logger, browser, server_config, binary=None, timeout_multiplier=1,
                 screenshot_cache=None, debug_info=None, pause_after_test=False,
                 **kwargs):
        ProcessTestExecutor.__init__(self,
                                     logger,
                                     browser,
                                     server_config,
                                     timeout_multiplier=timeout_multiplier,
                                     debug_info=debug_info)

        self.pause_after_test = pause_after_test
        self.protocol = ConnectionlessProtocol(self, browser)
        self.tempdir = tempfile.mkdtemp()
        self.hosts_path = write_hosts_file(server_config)

    def do_test(self, test):
        timeout = (test.timeout * self.timeout_multiplier if self.debug_info is None
                   else None)

        test_url = self.test_url(test)
        # We want to pass the full test object into build_servo_command,
        # so stash it in the class
        self.test = test
        success, data = ServoTimedRunner(self.logger, self.do_crashtest, self.protocol,
                                         test_url, timeout, self.extra_timeout).run()
        # Ensure that no processes hang around if they timeout.
        self.proc.kill()

        if success:
            return self.convert_result(test, data)

        return (test.result_cls(*data), [])

    def do_crashtest(self, protocol, url, timeout):
        env = os.environ.copy()
        env["HOST_FILE"] = self.hosts_path
        env["RUST_BACKTRACE"] = "1"

        command = build_servo_command(self.test,
                                      self.test_url,
                                      self.browser,
                                      self.binary,
                                      False,
                                      self.debug_info,
                                      extra_args=["-x"])

        if not self.interactive:
            self.proc = ProcessHandler(command,
                                       env=cast_env(env),
                                       storeOutput=False)
            self.proc.run()
        else:
            self.proc = subprocess.Popen(command, env=cast_env(env))

        self.proc.wait()

        if self.proc.poll() >= 0:
            return {"status": "PASS", "message": None}

        return {"status": "CRASH", "message": None}
