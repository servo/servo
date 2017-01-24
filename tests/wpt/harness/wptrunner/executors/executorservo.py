# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this file,
# You can obtain one at http://mozilla.org/MPL/2.0/.

import base64
import hashlib
import httplib
import json
import os
import subprocess
import tempfile
import threading
import traceback
import urlparse
import uuid
from collections import defaultdict

from mozprocess import ProcessHandler

from .base import (ExecutorException,
                   Protocol,
                   RefTestImplementation,
                   testharness_result_converter,
                   reftest_result_converter,
                   WdspecExecutor)
from .process import ProcessTestExecutor
from ..browsers.base import browser_command
from ..wpttest import WdspecResult, WdspecSubtestResult
from ..webdriver_server import ServoDriverServer
from .executormarionette import WdspecRun

pytestrunner = None
render_arg = None
webdriver = None

extra_timeout = 5 # seconds

def do_delayed_imports():
    global render_arg
    from ..browsers.servo import render_arg

hosts_text = """127.0.0.1 web-platform.test
127.0.0.1 www.web-platform.test
127.0.0.1 www1.web-platform.test
127.0.0.1 www2.web-platform.test
127.0.0.1 xn--n8j6ds53lwwkrqhv28a.web-platform.test
127.0.0.1 xn--lve-6lad.web-platform.test
"""

def make_hosts_file():
    hosts_fd, hosts_path = tempfile.mkstemp()
    with os.fdopen(hosts_fd, "w") as f:
        f.write(hosts_text)
    return hosts_path


class ServoTestharnessExecutor(ProcessTestExecutor):
    convert_result = testharness_result_converter

    def __init__(self, browser, server_config, timeout_multiplier=1, debug_info=None,
                 pause_after_test=False):
        do_delayed_imports()
        ProcessTestExecutor.__init__(self, browser, server_config,
                                     timeout_multiplier=timeout_multiplier,
                                     debug_info=debug_info)
        self.pause_after_test = pause_after_test
        self.result_data = None
        self.result_flag = None
        self.protocol = Protocol(self, browser)
        self.hosts_path = make_hosts_file()

    def teardown(self):
        try:
            os.unlink(self.hosts_path)
        except OSError:
            pass
        ProcessTestExecutor.teardown(self)

    def do_test(self, test):
        self.result_data = None
        self.result_flag = threading.Event()

        args = [render_arg(self.browser.render_backend), "--hard-fail", "-u", "Servo/wptrunner",
                "-Z", "replace-surrogates", "-z", self.test_url(test)]
        for stylesheet in self.browser.user_stylesheets:
            args += ["--user-stylesheet", stylesheet]
        for pref, value in test.environment.get('prefs', {}).iteritems():
            args += ["--pref", "%s=%s" % (pref, value)]
        args += self.browser.binary_args
        debug_args, command = browser_command(self.binary, args, self.debug_info)

        self.command = command

        if self.pause_after_test:
            self.command.remove("-z")

        self.command = debug_args + self.command

        env = os.environ.copy()
        env["HOST_FILE"] = self.hosts_path
        env["RUST_BACKTRACE"] = "1"


        if not self.interactive:
            self.proc = ProcessHandler(self.command,
                                       processOutputLine=[self.on_output],
                                       onFinish=self.on_finish,
                                       env=env,
                                       storeOutput=False)
            self.proc.run()
        else:
            self.proc = subprocess.Popen(self.command, env=env)

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
        except KeyboardInterrupt:
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
                print line
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

    def __init__(self, browser, server_config, binary=None, timeout_multiplier=1,
                 screenshot_cache=None, debug_info=None, pause_after_test=False):
        do_delayed_imports()
        ProcessTestExecutor.__init__(self,
                                     browser,
                                     server_config,
                                     timeout_multiplier=timeout_multiplier,
                                     debug_info=debug_info)

        self.protocol = Protocol(self, browser)
        self.screenshot_cache = screenshot_cache
        self.implementation = RefTestImplementation(self)
        self.tempdir = tempfile.mkdtemp()
        self.hosts_path = make_hosts_file()

    def teardown(self):
        try:
            os.unlink(self.hosts_path)
        except OSError:
            pass
        os.rmdir(self.tempdir)
        ProcessTestExecutor.teardown(self)

    def screenshot(self, test, viewport_size, dpi):
        full_url = self.test_url(test)

        with TempFilename(self.tempdir) as output_path:
            debug_args, command = browser_command(
                self.binary,
                [render_arg(self.browser.render_backend), "--hard-fail", "--exit",
                 "-u", "Servo/wptrunner", "-Z", "disable-text-aa,load-webfonts-synchronously,replace-surrogates",
                 "--output=%s" % output_path, full_url] + self.browser.binary_args,
                self.debug_info)

            for stylesheet in self.browser.user_stylesheets:
                command += ["--user-stylesheet", stylesheet]

            for pref, value in test.environment.get('prefs', {}).iteritems():
                command += ["--pref", "%s=%s" % (pref, value)]

            command += ["--resolution", viewport_size or "800x600"]

            if dpi:
                command += ["--device-pixel-ratio", dpi]

            # Run ref tests in headless mode
            command += ["-z"]

            self.command = debug_args + command

            env = os.environ.copy()
            env["HOST_FILE"] = self.hosts_path
            env["RUST_BACKTRACE"] = "1"

            if not self.interactive:
                self.proc = ProcessHandler(self.command,
                                           processOutputLine=[self.on_output],
                                           env=env)


                try:
                    self.proc.run()
                    timeout = test.timeout * self.timeout_multiplier + 5
                    rv = self.proc.wait(timeout=timeout)
                except KeyboardInterrupt:
                    self.proc.kill()
                    raise
            else:
                self.proc = subprocess.Popen(self.command,
                                             env=env)
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

            with open(output_path) as f:
                # Might need to strip variable headers or something here
                data = f.read()
                return True, base64.b64encode(data)

    def do_test(self, test):
        result = self.implementation.run_test(test)

        return self.convert_result(test, result)

    def on_output(self, line):
        line = line.decode("utf8", "replace")
        if self.interactive:
            print line
        else:
            self.logger.process_output(self.proc.pid,
                                       line,
                                       " ".join(self.command))

class ServoWdspecProtocol(Protocol):
    def __init__(self, executor, browser):
        self.do_delayed_imports()
        Protocol.__init__(self, executor, browser)
        self.session = None
        self.server = None

    def setup(self, runner):
        try:
            self.server = ServoDriverServer(self.logger, binary=self.browser.binary, binary_args=self.browser.binary_args, render_backend=self.browser.render_backend)
            self.server.start(block=False)
            self.logger.info(
                "WebDriver HTTP server listening at %s" % self.server.url)

            self.logger.info(
                "Establishing new WebDriver session with %s" % self.server.url)
            self.session = webdriver.Session(
                self.server.host, self.server.port, self.server.base_path)
        except Exception:
            self.logger.error(traceback.format_exc())
            self.executor.runner.send_message("init_failed")
        else:
            self.executor.runner.send_message("init_succeeded")

    def teardown(self):
        if self.server is not None:
            try:
                if self.session.session_id is not None:
                    self.session.end()
            except Exception:
                pass
            if self.server.is_alive:
                self.server.stop()

    @property
    def is_alive(self):
        conn = httplib.HTTPConnection(self.server.host, self.server.port)
        conn.request("HEAD", self.server.base_path + "invalid")
        res = conn.getresponse()
        return res.status == 404

    def do_delayed_imports(self):
        global pytestrunner, webdriver
        from . import pytestrunner
        import webdriver


class ServoWdspecExecutor(WdspecExecutor):
    def __init__(self, browser, server_config,
                 timeout_multiplier=1, close_after_done=True, debug_info=None,
                 **kwargs):
        WdspecExecutor.__init__(self, browser, server_config,
                                timeout_multiplier=timeout_multiplier,
                                debug_info=debug_info)
        self.protocol = ServoWdspecProtocol(self, browser)

    def is_alive(self):
        return self.protocol.is_alive

    def on_environment_change(self, new_environment):
        pass

    def do_test(self, test):
        timeout = test.timeout * self.timeout_multiplier + extra_timeout

        success, data = WdspecRun(self.do_wdspec,
                                  self.protocol.session,
                                  test.path,
                                  timeout).run()

        if success:
            return self.convert_result(test, data)

        return (test.result_cls(*data), [])

    def do_wdspec(self, session, path, timeout):
        harness_result = ("OK", None)
        subtest_results = pytestrunner.run(path, session, timeout=timeout)
        return (harness_result, subtest_results)
