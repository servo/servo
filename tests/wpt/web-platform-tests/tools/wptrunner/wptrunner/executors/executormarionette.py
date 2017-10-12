import os
import socket
import threading
import traceback
import urlparse
import uuid

errors = None
marionette = None
pytestrunner = None

here = os.path.join(os.path.split(__file__)[0])

from .base import (ExecutorException,
                   Protocol,
                   RefTestExecutor,
                   RefTestImplementation,
                   TestExecutor,
                   TestharnessExecutor,
                   WdspecExecutor,
                   WdspecRun,
                   WebDriverProtocol,
                   extra_timeout,
                   testharness_result_converter,
                   reftest_result_converter,
                   strip_server)

from ..testrunner import Stop
from ..webdriver_server import GeckoDriverServer



def do_delayed_imports():
    global errors, marionette

    # Marionette client used to be called marionette, recently it changed
    # to marionette_driver for unfathomable reasons
    try:
        import marionette
        from marionette import errors
    except ImportError:
        from marionette_driver import marionette, errors


class MarionetteProtocol(Protocol):
    def __init__(self, executor, browser, timeout_multiplier=1):
        do_delayed_imports()

        Protocol.__init__(self, executor, browser)
        self.marionette = None
        self.marionette_port = browser.marionette_port
        self.timeout_multiplier = timeout_multiplier
        self.timeout = None
        self.runner_handle = None

    def setup(self, runner):
        """Connect to browser via Marionette."""
        Protocol.setup(self, runner)

        self.logger.debug("Connecting to Marionette on port %i" % self.marionette_port)
        startup_timeout = marionette.Marionette.DEFAULT_STARTUP_TIMEOUT * self.timeout_multiplier
        self.marionette = marionette.Marionette(host='localhost',
                                                port=self.marionette_port,
                                                socket_timeout=None,
                                                startup_timeout=startup_timeout)

        # XXX Move this timeout somewhere
        self.logger.debug("Waiting for Marionette connection")
        while True:
            success = self.marionette.wait_for_port(60 * self.timeout_multiplier)
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
            self.marionette._request_in_app_shutdown()
            self.marionette.delete_session(send_request=False, reset_session_id=True)
        except Exception:
            # This is typically because the session never started
            pass
        if self.marionette is not None:
            del self.marionette

    @property
    def is_alive(self):
        """Check if the Marionette connection is still active."""
        try:
            self.marionette.current_window_handle
        except Exception:
            return False
        return True

    def after_connect(self):
        self.load_runner(self.executor.last_environment["protocol"])

    def set_timeout(self, timeout):
        """Set the Marionette script timeout.

        :param timeout: Script timeout in seconds

        """
        self.marionette.timeout.script = timeout
        self.timeout = timeout

    def load_runner(self, protocol):
        # Check if we previously had a test window open, and if we did make sure it's closed
        self.marionette.execute_script("if (window.wrappedJSObject.win) {window.wrappedJSObject.win.close()}")
        url = urlparse.urljoin(self.executor.server_url(protocol), "/testharness_runner.html")
        self.logger.debug("Loading %s" % url)
        self.runner_handle = self.marionette.current_window_handle
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

    def close_old_windows(self, protocol):
        handles = self.marionette.window_handles
        runner_handle = None
        try:
            handles.remove(self.runner_handle)
            runner_handle = self.runner_handle
        except ValueError:
            # The runner window probably changed id but we can restore it
            # This isn't supposed to happen, but marionette ids are not yet stable
            # We assume that the first handle returned corresponds to the runner,
            # but it hopefully doesn't matter too much if that assumption is
            # wrong since we reload the runner in that tab anyway.
            runner_handle = handles.pop(0)

        for handle in handles:
            self.marionette.switch_to_window(handle)
            self.marionette.close()

        self.marionette.switch_to_window(runner_handle)
        if runner_handle != self.runner_handle:
            self.load_runner(protocol)

    def wait(self):
        socket_timeout = self.marionette.client.sock.gettimeout()
        if socket_timeout:
            self.marionette.timeout.script = socket_timeout / 2

        self.marionette.switch_to_window(self.runner_handle)
        while True:
            try:
                self.marionette.execute_async_script("")
            except errors.NoSuchWindowException:
                # The window closed
                break
            except errors.ScriptTimeoutException:
                self.logger.debug("Script timed out")
                pass
            except (socket.timeout, IOError):
                self.logger.debug("Socket closed")
                break
            except Exception as e:
                self.logger.warning(traceback.format_exc(e))
                break

    def on_environment_change(self, old_environment, new_environment):
        #Unset all the old prefs
        for name in old_environment.get("prefs", {}).iterkeys():
            value = self.executor.original_pref_values[name]
            if value is None:
                self.clear_user_pref(name)
            else:
                self.set_pref(name, value)

        for name, value in new_environment.get("prefs", {}).iteritems():
            self.executor.original_pref_values[name] = self.get_pref(name)
            self.set_pref(name, value)

    def set_pref(self, name, value):
        if value.lower() not in ("true", "false"):
            try:
                int(value)
            except ValueError:
                value = "'%s'" % value
        else:
            value = value.lower()

        self.logger.info("Setting pref %s (%s)" % (name, value))

        script = """
            let prefInterface = Components.classes["@mozilla.org/preferences-service;1"]
                                          .getService(Components.interfaces.nsIPrefBranch);
            let pref = '%s';
            let type = prefInterface.getPrefType(pref);
            let value = %s;
            switch(type) {
                case prefInterface.PREF_STRING:
                    prefInterface.setCharPref(pref, value);
                    break;
                case prefInterface.PREF_BOOL:
                    prefInterface.setBoolPref(pref, value);
                    break;
                case prefInterface.PREF_INT:
                    prefInterface.setIntPref(pref, value);
                    break;
            }
            """ % (name, value)
        with self.marionette.using_context(self.marionette.CONTEXT_CHROME):
            self.marionette.execute_script(script)

    def clear_user_pref(self, name):
        self.logger.info("Clearing pref %s" % (name))
        script = """
            let prefInterface = Components.classes["@mozilla.org/preferences-service;1"]
                                          .getService(Components.interfaces.nsIPrefBranch);
            let pref = '%s';
            prefInterface.clearUserPref(pref);
            """ % name
        with self.marionette.using_context(self.marionette.CONTEXT_CHROME):
            self.marionette.execute_script(script)

    def get_pref(self, name):
        script = """
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
            """ % name
        with self.marionette.using_context(self.marionette.CONTEXT_CHROME):
            self.marionette.execute_script(script)

    def clear_origin(self, url):
        self.logger.info("Clearing origin %s" % (url))
        script = """
            let url = '%s';
            let uri = Components.classes["@mozilla.org/network/io-service;1"]
                                .getService(Ci.nsIIOService)
                                .newURI(url);
            let ssm = Components.classes["@mozilla.org/scriptsecuritymanager;1"]
                                .getService(Ci.nsIScriptSecurityManager);
            let principal = ssm.createCodebasePrincipal(uri, {});
            let qms = Components.classes["@mozilla.org/dom/quota-manager-service;1"]
                                .getService(Components.interfaces.nsIQuotaManagerService);
            qms.clearStoragesForPrincipal(principal, "default", true);
            """ % url
        with self.marionette.using_context(self.marionette.CONTEXT_CHROME):
            self.marionette.execute_script(script)


class ExecuteAsyncScriptRun(object):
    def __init__(self, logger, func, protocol, url, timeout):
        self.logger = logger
        self.result = (None, None)
        self.protocol = protocol
        self.marionette = protocol.marionette
        self.func = func
        self.url = url
        self.timeout = timeout
        self.result_flag = threading.Event()

    def run(self):
        index = self.url.rfind("/storage/");
        if index != -1:
            # Clear storage
            self.protocol.clear_origin(self.url)

        timeout = self.timeout

        try:
            if timeout is not None:
                if timeout + extra_timeout != self.protocol.timeout:
                    self.protocol.set_timeout(timeout + extra_timeout)
            else:
                # We just want it to never time out, really, but marionette doesn't
                # make that possible. It also seems to time out immediately if the
                # timeout is set too high. This works at least.
                self.protocol.set_timeout(2**28 - 1)
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

        if self.result == (None, None):
            self.logger.debug("Timed out waiting for a result")
            self.result = False, ("EXTERNAL-TIMEOUT", None)
        elif self.result[1] is None:
            # We didn't get any data back from the test, so check if the
            # browser is still responsive
            if self.protocol.is_alive:
                self.result = False, ("ERROR", None)
            else:
                self.result = False, ("CRASH", None)
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
    def __init__(self, browser, server_config, timeout_multiplier=1,
                 close_after_done=True, debug_info=None, **kwargs):
        """Marionette-based executor for testharness.js tests"""
        TestharnessExecutor.__init__(self, browser, server_config,
                                     timeout_multiplier=timeout_multiplier,
                                     debug_info=debug_info)

        self.protocol = MarionetteProtocol(self, browser, timeout_multiplier)
        self.script = open(os.path.join(here, "testharness_marionette.js")).read()
        self.close_after_done = close_after_done
        self.window_id = str(uuid.uuid4())

        self.original_pref_values = {}

        if marionette is None:
            do_delayed_imports()

    def is_alive(self):
        return self.protocol.is_alive

    def on_environment_change(self, new_environment):
        self.protocol.on_environment_change(self.last_environment, new_environment)

        if new_environment["protocol"] != self.last_environment["protocol"]:
            self.protocol.load_runner(new_environment["protocol"])

    def do_test(self, test):
        timeout = (test.timeout * self.timeout_multiplier if self.debug_info is None
                   else None)

        success, data = ExecuteAsyncScriptRun(self.logger,
                                              self.do_testharness,
                                              self.protocol,
                                              self.test_url(test),
                                              timeout).run()
        if success:
            return self.convert_result(test, data)

        return (test.result_cls(*data), [])

    def do_testharness(self, marionette, url, timeout):
        if self.close_after_done:
            marionette.execute_script("if (window.wrappedJSObject.win) {window.wrappedJSObject.win.close()}")
            self.protocol.close_old_windows(self.protocol)

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

        rv = marionette.execute_async_script(script, new_sandbox=False)
        return rv


class MarionetteRefTestExecutor(RefTestExecutor):
    def __init__(self, browser, server_config, timeout_multiplier=1,
                 screenshot_cache=None, close_after_done=True,
                 debug_info=None, reftest_internal=False,
                 reftest_screenshot="unexpected",
                 group_metadata=None, **kwargs):
        """Marionette-based executor for reftests"""
        RefTestExecutor.__init__(self,
                                 browser,
                                 server_config,
                                 screenshot_cache=screenshot_cache,
                                 timeout_multiplier=timeout_multiplier,
                                 debug_info=debug_info)
        self.protocol = MarionetteProtocol(self, browser)
        self.implementation = (InternalRefTestImplementation
                               if reftest_internal
                               else RefTestImplementation)(self)
        self.implementation_kwargs = ({"screenshot": reftest_screenshot} if
                                      reftest_internal else {})

        self.close_after_done = close_after_done
        self.has_window = False
        self.original_pref_values = {}
        self.group_metadata = group_metadata

        with open(os.path.join(here, "reftest.js")) as f:
            self.script = f.read()
        with open(os.path.join(here, "reftest-wait_marionette.js")) as f:
            self.wait_script = f.read()

    def setup(self, runner):
        super(self.__class__, self).setup(runner)
        self.implementation.setup(**self.implementation_kwargs)

    def teardown(self):
        try:
            self.implementation.teardown()
            handle = self.protocol.marionette.window_handles[0]
            self.protocol.marionette.switch_to_window(handle)
            super(self.__class__, self).teardown()
        except Exception as e:
            # Ignore errors during teardown
            self.logger.warning(traceback.format_exc(e))

    def is_alive(self):
        return self.protocol.is_alive

    def on_environment_change(self, new_environment):
        self.protocol.on_environment_change(self.last_environment, new_environment)

    def do_test(self, test):
        if not isinstance(self.implementation, InternalRefTestImplementation):
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

    def screenshot(self, test, viewport_size, dpi):
        # https://github.com/w3c/wptrunner/issues/166
        assert viewport_size is None
        assert dpi is None

        timeout =  self.timeout_multiplier * test.timeout if self.debug_info is None else None

        test_url = self.test_url(test)

        return ExecuteAsyncScriptRun(self.logger,
                                     self._screenshot,
                                     self.protocol,
                                     test_url,
                                     timeout).run()

    def _screenshot(self, marionette, url, timeout):
        marionette.navigate(url)

        marionette.execute_async_script(self.wait_script)

        screenshot = marionette.screenshot(full=False)
        # strip off the data:img/png, part of the url
        if screenshot.startswith("data:image/png;base64,"):
            screenshot = screenshot.split(",", 1)[1]

        return screenshot


class InternalRefTestImplementation(object):
    def __init__(self, executor):
        self.timeout_multiplier = executor.timeout_multiplier
        self.executor = executor

    @property
    def logger(self):
        return self.executor.logger

    def setup(self, screenshot="unexpected"):
        data = {"screenshot": screenshot}
        if self.executor.group_metadata is not None:
            data["urlCount"] = {urlparse.urljoin(self.executor.server_url(key[0]), key[1]):value
                                for key, value in self.executor.group_metadata.get("url_count", {}).iteritems()
                                if value > 1}
        self.executor.protocol.marionette.set_context(self.executor.protocol.marionette.CONTEXT_CHROME)
        self.executor.protocol.marionette._send_message("reftest:setup", data)

    def run_test(self, test):
        viewport_size = test.viewport_size
        dpi = test.dpi

        references = self.get_references(test)
        rv = self.executor.protocol.marionette._send_message("reftest:run",
                                                             {"test": self.executor.test_url(test),
                                                              "references": references,
                                                              "expected": test.expected(),
                                                              "timeout": test.timeout * 1000})["value"]
        return rv

    def get_references(self, node):
        rv = []
        for item, relation in node.references:
            rv.append([self.executor.test_url(item), self.get_references(item), relation])
        return rv

    def teardown(self):
        try:
            self.executor.protocol.marionette._send_message("reftest:teardown", {})
            self.executor.protocol.marionette.set_context(self.executor.protocol.marionette.CONTEXT_CONTENT)
        except Exception as e:
            # Ignore errors during teardown
            self.logger.warning(traceback.traceback.format_exc(e))



class GeckoDriverProtocol(WebDriverProtocol):
    server_cls = GeckoDriverServer


class MarionetteWdspecExecutor(WdspecExecutor):
    protocol_cls = GeckoDriverProtocol
