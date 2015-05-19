# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this file,
# You can obtain one at http://mozilla.org/MPL/2.0/.

import hashlib
import os
import socket
import sys
import threading
import time
import traceback
import urlparse
import uuid
from collections import defaultdict

marionette = None

here = os.path.join(os.path.split(__file__)[0])

from .base import (ExecutorException,
                   Protocol,
                   RefTestExecutor,
                   RefTestImplementation,
                   TestExecutor,
                   TestharnessExecutor,
                   testharness_result_converter,
                   reftest_result_converter,
                   strip_server)
from ..testrunner import Stop

# Extra timeout to use after internal test timeout at which the harness
# should force a timeout
extra_timeout = 5 # seconds

def do_delayed_imports():
    global marionette
    global errors
    try:
        import marionette
        from marionette import errors
    except ImportError:
        from marionette_driver import marionette, errors


class MarionetteProtocol(Protocol):
    def __init__(self, executor, browser):
        do_delayed_imports()

        Protocol.__init__(self, executor, browser)
        self.marionette = None
        self.marionette_port = browser.marionette_port

    def setup(self, runner):
        """Connect to browser via Marionette."""
        Protocol.setup(self, runner)

        self.logger.debug("Connecting to marionette on port %i" % self.marionette_port)
        self.marionette = marionette.Marionette(host='localhost', port=self.marionette_port)

        # XXX Move this timeout somewhere
        self.logger.debug("Waiting for Marionette connection")
        while True:
            success = self.marionette.wait_for_port(60)
            #When running in a debugger wait indefinitely for firefox to start
            if success or self.executor.debug_info is None:
                break

        session_started = False
        if success:
            try:
                self.logger.debug("Starting Marionette session")
                self.marionette.start_session()
            except Exception as e:
                self.logger.warning("Starting marionette session failed: %s" % e)
            else:
                self.logger.debug("Marionette session started")
                session_started = True

        if not success or not session_started:
            self.logger.warning("Failed to connect to Marionette")
            self.executor.runner.send_message("init_failed")
        else:
            try:
                self.after_connect()
            except Exception:
                self.logger.warning("Post-connection steps failed")
                self.logger.error(traceback.format_exc())
                self.executor.runner.send_message("init_failed")
            else:
                self.executor.runner.send_message("init_succeeded")

    def teardown(self):
        try:
            self.marionette.delete_session()
        except Exception:
            # This is typically because the session never started
            pass
        del self.marionette

    def is_alive(self):
        """Check if the marionette connection is still active"""
        try:
            # Get a simple property over the connection
            self.marionette.current_window_handle
        except Exception:
            return False
        return True

    def after_connect(self):
        self.load_runner("http")

    def load_runner(self, protocol):
        # Check if we previously had a test window open, and if we did make sure it's closed
        self.marionette.execute_script("if (window.wrappedJSObject.win) {window.wrappedJSObject.win.close()}")
        url = urlparse.urljoin(self.executor.server_url(protocol), "/testharness_runner.html")
        self.logger.debug("Loading %s" % url)
        try:
            self.marionette.navigate(url)
        except Exception as e:
            self.logger.critical(
                "Loading initial page %s failed. Ensure that the "
                "there are no other programs bound to this port and "
                "that your firewall rules or network setup does not "
                "prevent access.\e%s" % (url, traceback.format_exc(e)))
        self.marionette.execute_script(
            "document.title = '%s'" % threading.current_thread().name.replace("'", '"'))

    def wait(self):
        while True:
            try:
                self.marionette.execute_async_script("");
            except errors.ScriptTimeoutException:
                pass
            except (socket.timeout, IOError):
                break
            except Exception as e:
                self.logger.error(traceback.format_exc(e))
                break

    def on_environment_change(self, old_environment, new_environment):
        #Unset all the old prefs
        for name, _ in old_environment.get("prefs", []):
            value = self.executor.original_pref_values[name]
            if value is None:
                self.clear_user_pref(name)
            else:
                self.set_pref(name, value)

        for name, value in new_environment.get("prefs", []):
            self.executor.original_pref_values[name] = self.get_pref(name)
            self.set_pref(name, value)

    def set_pref(self, name, value):
        self.logger.info("Setting pref %s (%s)" % (name, value))
        self.marionette.set_context(self.marionette.CONTEXT_CHROME)
        script = """
            let prefInterface = Components.classes["@mozilla.org/preferences-service;1"]
                                          .getService(Components.interfaces.nsIPrefBranch);
            let pref = '%s';
            let type = prefInterface.getPrefType(pref);
            switch(type) {
                case prefInterface.PREF_STRING:
                    prefInterface.setCharPref(pref, '%s');
                    break;
                case prefInterface.PREF_BOOL:
                    prefInterface.setBoolPref(pref, %s);
                    break;
                case prefInterface.PREF_INT:
                    prefInterface.setIntPref(pref, %s);
                    break;
            }
            """ % (name, value, value, value)
        self.marionette.execute_script(script)
        self.marionette.set_context(self.marionette.CONTEXT_CONTENT)

    def clear_user_pref(self, name):
        self.logger.info("Clearing pref %s" % (name))
        self.marionette.set_context(self.marionette.CONTEXT_CHROME)
        script = """
            let prefInterface = Components.classes["@mozilla.org/preferences-service;1"]
                                          .getService(Components.interfaces.nsIPrefBranch);
            let pref = '%s';
            prefInterface.clearUserPref(pref);
            """ % name
        self.marionette.execute_script(script)
        self.marionette.set_context(self.marionette.CONTEXT_CONTENT)

    def get_pref(self, name):
        self.marionette.set_context(self.marionette.CONTEXT_CHROME)
        self.marionette.execute_script("""
            let prefInterface = Components.classes["@mozilla.org/preferences-service;1"]
                                          .getService(Components.interfaces.nsIPrefBranch);
            let pref = '%s';
            let type = prefInterface.getPrefType(pref);
            switch(type) {
                case prefInterface.PREF_STRING:
                    return prefInterface.getCharPref(pref);
                case prefInterface.PREF_BOOL:
                    return prefInterface.getBoolPref(pref);
                case prefInterface.PREF_INT:
                    return prefInterface.getIntPref(pref);
                case prefInterface.PREF_INVALID:
                    return null;
            }
            """ % (name))
        self.marionette.set_context(self.marionette.CONTEXT_CONTENT)

class MarionetteRun(object):
    def __init__(self, logger, func, marionette, url, timeout):
        self.logger = logger
        self.result = None
        self.marionette = marionette
        self.func = func
        self.url = url
        self.timeout = timeout
        self.result_flag = threading.Event()

    def run(self):
        timeout = self.timeout

        try:
            if timeout is not None:
                self.marionette.set_script_timeout((timeout + extra_timeout) * 1000)
            else:
                # We just want it to never time out, really, but marionette doesn't
                # make that possible. It also seems to time out immediately if the
                # timeout is set too high. This works at least.
                self.marionette.set_script_timeout(2**31 - 1)
        except IOError:
            self.logger.error("Lost marionette connection before starting test")
            return Stop

        executor = threading.Thread(target = self._run)
        executor.start()

        if timeout is not None:
            wait_timeout = timeout + 2 * extra_timeout
        else:
            wait_timeout = None

        flag = self.result_flag.wait(wait_timeout)
        if self.result is None:
            self.logger.debug("Timed out waiting for a result")
            assert not flag
            self.result = False, ("EXTERNAL-TIMEOUT", None)

        return self.result

    def _run(self):
        try:
            self.result = True, self.func(self.marionette, self.url, self.timeout)
        except errors.ScriptTimeoutException:
            self.logger.debug("Got a marionette timeout")
            self.result = False, ("EXTERNAL-TIMEOUT", None)
        except (socket.timeout, IOError):
            # This can happen on a crash
            # Also, should check after the test if the firefox process is still running
            # and otherwise ignore any other result and set it to crash
            self.result = False, ("CRASH", None)
        except Exception as e:
            message = getattr(e, "message", "")
            if message:
                message += "\n"
            message += traceback.format_exc(e)
            self.result = False, ("ERROR", e)

        finally:
            self.result_flag.set()


class MarionetteTestharnessExecutor(TestharnessExecutor):
    def __init__(self, browser, server_config, timeout_multiplier=1, close_after_done=True,
                 debug_info=None):
        """Marionette-based executor for testharness.js tests"""
        TestharnessExecutor.__init__(self, browser, server_config,
                                     timeout_multiplier=timeout_multiplier,
                                     debug_info=debug_info)

        self.protocol = MarionetteProtocol(self, browser)
        self.script = open(os.path.join(here, "testharness_marionette.js")).read()
        self.close_after_done = close_after_done
        self.window_id = str(uuid.uuid4())

        self.original_pref_values = {}

        if marionette is None:
            do_delayed_imports()

    def is_alive(self):
        return self.protocol.is_alive()

    def on_environment_change(self, new_environment):
        self.protocol.on_environment_change(self.last_environment, new_environment)

        if new_environment["protocol"] != self.last_environment["protocol"]:
            self.protocol.load_runner(new_environment["protocol"])

    def do_test(self, test):
        timeout = (test.timeout * self.timeout_multiplier if self.debug_info is None
                   else None)

        success, data = MarionetteRun(self.logger,
                                      self.do_testharness,
                                      self.protocol.marionette,
                                      self.test_url(test),
                                      timeout).run()
        if success:
            return self.convert_result(test, data)

        return (test.result_cls(*data), [])

    def do_testharness(self, marionette, url, timeout):
        if self.close_after_done:
            marionette.execute_script("if (window.wrappedJSObject.win) {window.wrappedJSObject.win.close()}")

        if timeout is not None:
            timeout_ms = str(timeout * 1000)
        else:
            timeout_ms = "null"

        script = self.script % {"abs_url": url,
                                "url": strip_server(url),
                                "window_id": self.window_id,
                                "timeout_multiplier": self.timeout_multiplier,
                                "timeout": timeout_ms,
                                "explicit_timeout": timeout is None}

        return marionette.execute_async_script(script, new_sandbox=False)


class MarionetteRefTestExecutor(RefTestExecutor):
    def __init__(self, browser, server_config, timeout_multiplier=1,
                 screenshot_cache=None, close_after_done=True, debug_info=None):
        """Marionette-based executor for reftests"""
        RefTestExecutor.__init__(self,
                                 browser,
                                 server_config,
                                 screenshot_cache=screenshot_cache,
                                 timeout_multiplier=timeout_multiplier,
                                 debug_info=debug_info)
        self.protocol = MarionetteProtocol(self, browser)
        self.implementation = RefTestImplementation(self)
        self.close_after_done = close_after_done
        self.has_window = False
        self.original_pref_values = {}

        with open(os.path.join(here, "reftest.js")) as f:
            self.script = f.read()
        with open(os.path.join(here, "reftest-wait.js")) as f:
            self.wait_script = f.read()

    def is_alive(self):
        return self.protocol.is_alive()

    def on_environment_change(self, new_environment):
        self.protocol.on_environment_change(self.last_environment, new_environment)

    def do_test(self, test):
        if self.close_after_done and self.has_window:
            self.protocol.marionette.close()
            self.protocol.marionette.switch_to_window(
                self.protocol.marionette.window_handles[-1])
            self.has_window = False

        if not self.has_window:
            self.protocol.marionette.execute_script(self.script)
            self.protocol.marionette.switch_to_window(self.protocol.marionette.window_handles[-1])
            self.has_window = True

        result = self.implementation.run_test(test)

        return self.convert_result(test, result)

    def screenshot(self, test):
        timeout =  self.timeout_multiplier * test.timeout if self.debug_info is None else None

        test_url = self.test_url(test)

        return MarionetteRun(self.logger,
                             self._screenshot,
                             self.protocol.marionette,
                             test_url,
                             timeout).run()

    def _screenshot(self, marionette, url, timeout):
        try:
            marionette.navigate(url)
        except errors.MarionetteException:
            raise ExecutorException("ERROR", "Failed to load url %s" % (url,))

        marionette.execute_async_script(self.wait_script)

        screenshot = marionette.screenshot()
        # strip off the data:img/png, part of the url
        if screenshot.startswith("data:image/png;base64,"):
            screenshot = screenshot.split(",", 1)[1]

        return screenshot
