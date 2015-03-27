# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this file,
# You can obtain one at http://mozilla.org/MPL/2.0/.

import base64
import hashlib
import json
import os
import subprocess
import tempfile
import threading
import urlparse
import uuid
from collections import defaultdict

from mozprocess import ProcessHandler

from .base import (ExecutorException,
                   Protocol,
                   RefTestImplementation,
                   testharness_result_converter,
                   reftest_result_converter)
from .process import ProcessTestExecutor

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

    def __init__(self, browser, server_config, timeout_multiplier=1, debug_args=None,
                 pause_after_test=False):
        ProcessTestExecutor.__init__(self, browser, server_config,
                                     timeout_multiplier=timeout_multiplier,
                                     debug_args=debug_args)
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

        self.command = [self.binary, "--cpu", "--hard-fail", "-z", self.test_url(test)]

        if self.pause_after_test:
            self.command.remove("-z")

        if self.debug_args:
            self.command = list(self.debug_args) + self.command

        env = os.environ.copy()
        env["HOST_FILE"] = self.hosts_path

        self.proc = ProcessHandler(self.command,
                                   processOutputLine=[self.on_output],
                                   onFinish=self.on_finish,
                                   env=env)

        try:
            self.proc.run()

            timeout = test.timeout * self.timeout_multiplier

            # Now wait to get the output we expect, or until we reach the timeout
            if self.debug_args is None and not self.pause_after_test:
                wait_timeout = timeout + 5
            else:
                wait_timeout = None
            self.result_flag.wait(wait_timeout)

            proc_is_running = True
            if self.result_flag.is_set() and self.result_data is not None:
                self.result_data["test"] = test.url
                result = self.convert_result(test, self.result_data)
            else:
                if self.proc.proc.poll() is not None:
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
                 screenshot_cache=None, debug_args=None, pause_after_test=False):

        ProcessTestExecutor.__init__(self,
                                     browser,
                                     server_config,
                                     timeout_multiplier=timeout_multiplier,
                                     debug_args=debug_args)

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

    def screenshot(self, test):
        full_url = self.test_url(test)

        with TempFilename(self.tempdir) as output_path:
            self.command = [self.binary, "--cpu", "--hard-fail", "--exit",
                            "--output=%s" % output_path, full_url]

            env = os.environ.copy()
            env["HOST_FILE"] = self.hosts_path

            self.proc = ProcessHandler(self.command,
                                       processOutputLine=[self.on_output],
                                       env=env)

            try:
                self.proc.run()
                rv = self.proc.wait(timeout=test.timeout)
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
