import json
import os
import threading
import traceback
import urlparse
import uuid

errors = None
marionette = None
pytestrunner = None

here = os.path.join(os.path.split(__file__)[0])

from .base import (CallbackHandler,
                   RefTestExecutor,
                   RefTestImplementation,
                   TestharnessExecutor,
                   WdspecExecutor,
                   WebDriverProtocol,
                   extra_timeout,
                   strip_server)
from .protocol import (ActionSequenceProtocolPart,
                       AssertsProtocolPart,
                       BaseProtocolPart,
                       TestharnessProtocolPart,
                       PrefsProtocolPart,
                       Protocol,
                       StorageProtocolPart,
                       SelectorProtocolPart,
                       ClickProtocolPart,
                       SendKeysProtocolPart,
                       TestDriverProtocolPart,
                       CoverageProtocolPart)
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


class MarionetteBaseProtocolPart(BaseProtocolPart):
    def __init__(self, parent):
        super(MarionetteBaseProtocolPart, self).__init__(parent)
        self.timeout = None

    def setup(self):
        self.marionette = self.parent.marionette

    def execute_script(self, script, async=False):
        method = self.marionette.execute_async_script if async else self.marionette.execute_script
        return method(script, new_sandbox=False, sandbox=None)

    def set_timeout(self, timeout):
        """Set the Marionette script timeout.

        :param timeout: Script timeout in seconds

        """
        if timeout != self.timeout:
            self.marionette.timeout.script = timeout
            self.timeout = timeout

    @property
    def current_window(self):
        return self.marionette.current_window_handle

    def set_window(self, handle):
        self.marionette.switch_to_window(handle)

    def wait(self):
        try:
            socket_timeout = self.marionette.client.socket_timeout
        except AttributeError:
            # This can happen if there was a crash
            return
        if socket_timeout:
            try:
                self.marionette.timeout.script = socket_timeout / 2
            except IOError:
                self.logger.debug("Socket closed")
                return

        while True:
            try:
                self.marionette.execute_async_script("")
            except errors.NoSuchWindowException:
                # The window closed
                break
            except errors.ScriptTimeoutException:
                self.logger.debug("Script timed out")
                pass
            except IOError:
                self.logger.debug("Socket closed")
                break
            except Exception as e:
                self.logger.warning(traceback.format_exc(e))
                break


class MarionetteTestharnessProtocolPart(TestharnessProtocolPart):
    def __init__(self, parent):
        super(MarionetteTestharnessProtocolPart, self).__init__(parent)
        self.runner_handle = None
        with open(os.path.join(here, "runner.js")) as f:
            self.runner_script = f.read()

    def setup(self):
        self.marionette = self.parent.marionette

    def load_runner(self, url_protocol):
        # Check if we previously had a test window open, and if we did make sure it's closed
        if self.runner_handle:
            self._close_windows()
        url = urlparse.urljoin(self.parent.executor.server_url(url_protocol),
                               "/testharness_runner.html")
        self.logger.debug("Loading %s" % url)
        try:
            self.dismiss_alert(lambda: self.marionette.navigate(url))
        except Exception as e:
            self.logger.critical(
                "Loading initial page %s failed. Ensure that the "
                "there are no other programs bound to this port and "
                "that your firewall rules or network setup does not "
                "prevent access.\e%s" % (url, traceback.format_exc(e)))
            raise
        self.runner_handle = self.marionette.current_window_handle
        format_map = {"title": threading.current_thread().name.replace("'", '"')}
        self.parent.base.execute_script(self.runner_script % format_map)

    def _close_windows(self):
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
            self.logger.info("Changing harness_window to %s" % runner_handle)

        for handle in handles:
            try:
                self.dismiss_alert(lambda: self.marionette.switch_to_window(handle))
                self.marionette.switch_to_window(handle)
                self.logger.info("Closing window %s" % handle)
                self.marionette.close()
            except errors.NoSuchWindowException:
                # We might have raced with the previous test to close this
                # window, skip it.
                pass
        self.marionette.switch_to_window(runner_handle)
        return runner_handle

    def close_old_windows(self, url_protocol):
        runner_handle = self._close_windows()
        if runner_handle != self.runner_handle:
            self.load_runner(url_protocol)
        return self.runner_handle

    def dismiss_alert(self, f):
        while True:
            try:
                f()
            except errors.UnexpectedAlertOpen:
                alert = self.marionette.switch_to_alert()
                try:
                    alert.dismiss()
                except errors.NoAlertPresentException:
                    pass
            else:
                break

    def get_test_window(self, window_id, parent):
        test_window = None
        if window_id:
            try:
                # Try this, it's in Level 1 but nothing supports it yet
                win_s = self.parent.base.execute_script("return window['%s'];" % self.window_id)
                win_obj = json.loads(win_s)
                test_window = win_obj["window-fcc6-11e5-b4f8-330a88ab9d7f"]
            except Exception:
                pass

        if test_window is None:
            after = self.marionette.window_handles
            if len(after) == 2:
                test_window = next(iter(set(after) - set([parent])))
            elif after[0] == parent and len(after) > 2:
                # Hope the first one here is the test window
                test_window = after[1]
            else:
                raise Exception("unable to find test window")

        assert test_window != parent
        return test_window


class MarionettePrefsProtocolPart(PrefsProtocolPart):
    def setup(self):
        self.marionette = self.parent.marionette

    def set(self, name, value):
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

    def clear(self, name):
        self.logger.info("Clearing pref %s" % (name))
        script = """
            let prefInterface = Components.classes["@mozilla.org/preferences-service;1"]
                                          .getService(Components.interfaces.nsIPrefBranch);
            let pref = '%s';
            prefInterface.clearUserPref(pref);
            """ % name
        with self.marionette.using_context(self.marionette.CONTEXT_CHROME):
            self.marionette.execute_script(script)

    def get(self, name):
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


class MarionetteStorageProtocolPart(StorageProtocolPart):
    def setup(self):
        self.marionette = self.parent.marionette

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
            qms.clearStoragesForPrincipal(principal, "default", null, true);
            """ % url
        with self.marionette.using_context(self.marionette.CONTEXT_CHROME):
            self.marionette.execute_script(script)


class MarionetteAssertsProtocolPart(AssertsProtocolPart):
    def setup(self):
        self.assert_count = {"chrome": 0, "content": 0}
        self.chrome_assert_count = 0
        self.marionette = self.parent.marionette

    def get(self):
        script = """
        debug = Cc["@mozilla.org/xpcom/debug;1"].getService(Ci.nsIDebug2);
        if (debug.isDebugBuild) {
          return debug.assertionCount;
        }
        return 0;
        """

        def get_count(context, **kwargs):
            try:
                context_count = self.marionette.execute_script(script, **kwargs)
                if context_count:
                    self.parent.logger.info("Got %s assert count %s" % (context, context_count))
                    test_count = context_count - self.assert_count[context]
                    self.assert_count[context] = context_count
                    return test_count
            except errors.NoSuchWindowException:
                # If the window was already closed
                self.parent.logger.warning("Failed to get assertion count; window was closed")
            except (errors.MarionetteException, IOError):
                # This usually happens if the process crashed
                pass

        counts = []
        with self.marionette.using_context(self.marionette.CONTEXT_CHROME):
            counts.append(get_count("chrome"))
        if self.parent.e10s:
            counts.append(get_count("content", sandbox="system"))

        counts = [item for item in counts if item is not None]

        if not counts:
            return None

        return sum(counts)


class MarionetteSelectorProtocolPart(SelectorProtocolPart):
    def setup(self):
        self.marionette = self.parent.marionette

    def elements_by_selector(self, selector):
        return self.marionette.find_elements("css selector", selector)


class MarionetteClickProtocolPart(ClickProtocolPart):
    def setup(self):
        self.marionette = self.parent.marionette

    def element(self, element):
        return element.click()


class MarionetteSendKeysProtocolPart(SendKeysProtocolPart):
    def setup(self):
        self.marionette = self.parent.marionette

    def send_keys(self, element, keys):
        return element.send_keys(keys)


class MarionetteActionSequenceProtocolPart(ActionSequenceProtocolPart):
    def setup(self):
        self.marionette = self.parent.marionette

    def send_actions(self, actions):
        actions = self.marionette._to_json(actions)
        self.logger.info(actions)
        self.marionette._send_message("WebDriver:PerformActions", actions)


class MarionetteTestDriverProtocolPart(TestDriverProtocolPart):
    def setup(self):
        self.marionette = self.parent.marionette

    def send_message(self, message_type, status, message=None):
        obj = {
            "type": "testdriver-%s" % str(message_type),
            "status": str(status)
        }
        if message:
            obj["message"] = str(message)
        self.parent.base.execute_script("window.postMessage(%s, '*')" % json.dumps(obj))


class MarionetteCoverageProtocolPart(CoverageProtocolPart):
    def setup(self):
        self.marionette = self.parent.marionette

        if not self.parent.ccov:
            self.is_enabled = False
            return

        script = """
            ChromeUtils.import("chrome://marionette/content/PerTestCoverageUtils.jsm");
            return PerTestCoverageUtils.enabled;
            """
        with self.marionette.using_context(self.marionette.CONTEXT_CHROME):
            self.is_enabled = self.marionette.execute_script(script)

    def reset(self):
        script = """
            var callback = arguments[arguments.length - 1];

            ChromeUtils.import("chrome://marionette/content/PerTestCoverageUtils.jsm");
            PerTestCoverageUtils.beforeTest().then(callback, callback);
            """
        with self.marionette.using_context(self.marionette.CONTEXT_CHROME):
            try:
                error = self.marionette.execute_async_script(script)
                if error is not None:
                    raise Exception('Failure while resetting counters: %s' % json.dumps(error))
            except (errors.MarionetteException, IOError):
                # This usually happens if the process crashed
                pass

    def dump(self):
        if len(self.marionette.window_handles):
            handle = self.marionette.window_handles[0]
            self.marionette.switch_to_window(handle)

        script = """
            var callback = arguments[arguments.length - 1];

            ChromeUtils.import("chrome://marionette/content/PerTestCoverageUtils.jsm");
            PerTestCoverageUtils.afterTest().then(callback, callback);
            """
        with self.marionette.using_context(self.marionette.CONTEXT_CHROME):
            try:
                error = self.marionette.execute_async_script(script)
                if error is not None:
                    raise Exception('Failure while dumping counters: %s' % json.dumps(error))
            except (errors.MarionetteException, IOError):
                # This usually happens if the process crashed
                pass


class MarionetteProtocol(Protocol):
    implements = [MarionetteBaseProtocolPart,
                  MarionetteTestharnessProtocolPart,
                  MarionettePrefsProtocolPart,
                  MarionetteStorageProtocolPart,
                  MarionetteSelectorProtocolPart,
                  MarionetteClickProtocolPart,
                  MarionetteSendKeysProtocolPart,
                  MarionetteActionSequenceProtocolPart,
                  MarionetteTestDriverProtocolPart,
                  MarionetteAssertsProtocolPart,
                  MarionetteCoverageProtocolPart]

    def __init__(self, executor, browser, capabilities=None, timeout_multiplier=1, e10s=True, ccov=False):
        do_delayed_imports()

        super(MarionetteProtocol, self).__init__(executor, browser)
        self.marionette = None
        self.marionette_port = browser.marionette_port
        self.capabilities = capabilities
        self.timeout_multiplier = timeout_multiplier
        self.runner_handle = None
        self.e10s = e10s
        self.ccov = ccov

    def connect(self):
        self.logger.debug("Connecting to Marionette on port %i" % self.marionette_port)
        startup_timeout = marionette.Marionette.DEFAULT_STARTUP_TIMEOUT * self.timeout_multiplier
        self.marionette = marionette.Marionette(host='127.0.0.1',
                                                port=self.marionette_port,
                                                socket_timeout=None,
                                                startup_timeout=startup_timeout)

        self.logger.debug("Waiting for Marionette connection")
        while True:
            try:
                self.marionette.raise_for_port()
                break
            except IOError:
                # When running in a debugger wait indefinitely for Firefox to start
                if self.executor.debug_info is None:
                    raise

        self.logger.debug("Starting Marionette session")
        self.marionette.start_session(self.capabilities)
        self.logger.debug("Marionette session started")

    def after_connect(self):
        pass

    def teardown(self):
        try:
            self.marionette._request_in_app_shutdown()
            self.marionette.delete_session(send_request=False)
        except Exception:
            # This is typically because the session never started
            pass
        if self.marionette is not None:
            del self.marionette
        super(MarionetteProtocol, self).teardown()

    @property
    def is_alive(self):
        try:
            self.marionette.current_window_handle
        except Exception:
            return False
        return True

    def on_environment_change(self, old_environment, new_environment):
        #Unset all the old prefs
        for name in old_environment.get("prefs", {}).iterkeys():
            value = self.executor.original_pref_values[name]
            if value is None:
                self.prefs.clear(name)
            else:
                self.prefs.set(name, value)

        for name, value in new_environment.get("prefs", {}).iteritems():
            self.executor.original_pref_values[name] = self.prefs.get(name)
            self.prefs.set(name, value)


class ExecuteAsyncScriptRun(object):
    def __init__(self, logger, func, protocol, url, timeout):
        self.logger = logger
        self.result = (None, None)
        self.protocol = protocol
        self.func = func
        self.url = url
        self.timeout = timeout
        self.result_flag = threading.Event()

    def run(self):
        index = self.url.rfind("/storage/")
        if index != -1:
            # Clear storage
            self.protocol.storage.clear_origin(self.url)

        timeout = self.timeout

        try:
            if timeout is not None:
                self.protocol.base.set_timeout(timeout + extra_timeout)
            else:
                # We just want it to never time out, really, but marionette doesn't
                # make that possible. It also seems to time out immediately if the
                # timeout is set too high. This works at least.
                self.protocol.base.set_timeout(2**28 - 1)
        except IOError:
            self.logger.error("Lost marionette connection before starting test")
            return Stop

        if timeout is not None:
            wait_timeout = timeout + 2 * extra_timeout
        else:
            wait_timeout = None

        timer = threading.Timer(wait_timeout, self._timeout)
        timer.start()

        self._run()

        self.result_flag.wait()
        timer.cancel()

        if self.result == (None, None):
            self.logger.debug("Timed out waiting for a result")
            self.result = False, ("EXTERNAL-TIMEOUT", None)
        elif self.result[1] is None:
            # We didn't get any data back from the test, so check if the
            # browser is still responsive
            if self.protocol.is_alive:
                self.result = False, ("INTERNAL-ERROR", None)
            else:
                self.result = False, ("CRASH", None)
        return self.result

    def _run(self):
        try:
            self.result = True, self.func(self.protocol, self.url, self.timeout)
        except errors.ScriptTimeoutException:
            self.logger.debug("Got a marionette timeout")
            self.result = False, ("EXTERNAL-TIMEOUT", None)
        except IOError:
            # This can happen on a crash
            # Also, should check after the test if the firefox process is still running
            # and otherwise ignore any other result and set it to crash
            self.result = False, ("CRASH", None)
        except Exception as e:
            message = getattr(e, "message", "")
            if message:
                message += "\n"
            message += traceback.format_exc(e)
            self.logger.warning(message)
            self.result = False, ("INTERNAL-ERROR", None)

        finally:
            self.result_flag.set()

    def _timeout(self):
        self.result = False, ("EXTERNAL-TIMEOUT", None)
        self.result_flag.set()


class MarionetteTestharnessExecutor(TestharnessExecutor):
    supports_testdriver = True

    def __init__(self, browser, server_config, timeout_multiplier=1,
                 close_after_done=True, debug_info=None, capabilities=None,
                 debug=False, ccov=False, **kwargs):
        """Marionette-based executor for testharness.js tests"""
        TestharnessExecutor.__init__(self, browser, server_config,
                                     timeout_multiplier=timeout_multiplier,
                                     debug_info=debug_info)
        self.protocol = MarionetteProtocol(self,
                                           browser,
                                           capabilities,
                                           timeout_multiplier,
                                           kwargs["e10s"],
                                           ccov)
        self.script = open(os.path.join(here, "testharness_webdriver.js")).read()
        self.script_resume = open(os.path.join(here, "testharness_webdriver_resume.js")).read()
        self.close_after_done = close_after_done
        self.window_id = str(uuid.uuid4())
        self.debug = debug

        self.original_pref_values = {}

        if marionette is None:
            do_delayed_imports()

    def setup(self, runner):
        super(MarionetteTestharnessExecutor, self).setup(runner)
        self.protocol.testharness.load_runner(self.last_environment["protocol"])

    def is_alive(self):
        return self.protocol.is_alive

    def on_environment_change(self, new_environment):
        self.protocol.on_environment_change(self.last_environment, new_environment)

        if new_environment["protocol"] != self.last_environment["protocol"]:
            self.protocol.testharness.load_runner(new_environment["protocol"])

    def do_test(self, test):
        timeout = (test.timeout * self.timeout_multiplier if self.debug_info is None
                   else None)

        success, data = ExecuteAsyncScriptRun(self.logger,
                                              self.do_testharness,
                                              self.protocol,
                                              self.test_url(test),
                                              timeout).run()
        # The format of data depends on whether the test ran to completion or not
        # For asserts we only care about the fact that if it didn't complete, the
        # status is in the first field.
        status = None
        if not success:
            status = data[0]

        extra = None
        if self.debug and (success or status not in ("CRASH", "INTERNAL-ERROR")):
            assertion_count = self.protocol.asserts.get()
            if assertion_count is not None:
                extra = {"assertion_count": assertion_count}

        if success:
            return self.convert_result(test, data, extra=extra)

        return (test.result_cls(extra=extra, *data), [])

    def do_testharness(self, protocol, url, timeout):
        parent_window = protocol.testharness.close_old_windows(protocol)

        if timeout is not None:
            timeout_ms = str(timeout * 1000)
        else:
            timeout_ms = "null"

        if self.protocol.coverage.is_enabled:
            self.protocol.coverage.reset()

        format_map = {"abs_url": url,
                      "url": strip_server(url),
                      "window_id": self.window_id,
                      "timeout_multiplier": self.timeout_multiplier,
                      "timeout": timeout_ms,
                      "explicit_timeout": timeout is None}

        script = self.script % format_map

        protocol.base.execute_script(script, async=True)
        test_window = protocol.testharness.get_test_window(self.window_id, parent_window)

        handler = CallbackHandler(self.logger, protocol, test_window)
        while True:
            self.protocol.base.set_window(test_window)
            result = protocol.base.execute_script(
                self.script_resume % format_map, async=True)
            if result is None:
                # This can happen if we get an content process crash
                return None
            done, rv = handler(result)
            if done:
                break

        if self.protocol.coverage.is_enabled:
            self.protocol.coverage.dump()

        return rv


class MarionetteRefTestExecutor(RefTestExecutor):
    def __init__(self, browser, server_config, timeout_multiplier=1,
                 screenshot_cache=None, close_after_done=True,
                 debug_info=None, reftest_internal=False,
                 reftest_screenshot="unexpected", ccov=False,
                 group_metadata=None, capabilities=None, debug=False, **kwargs):
        """Marionette-based executor for reftests"""
        RefTestExecutor.__init__(self,
                                 browser,
                                 server_config,
                                 screenshot_cache=screenshot_cache,
                                 timeout_multiplier=timeout_multiplier,
                                 debug_info=debug_info)
        self.protocol = MarionetteProtocol(self, browser, capabilities,
                                           timeout_multiplier, kwargs["e10s"],
                                           ccov)
        self.implementation = (InternalRefTestImplementation
                               if reftest_internal
                               else RefTestImplementation)(self)
        self.implementation_kwargs = ({"screenshot": reftest_screenshot} if
                                      reftest_internal else {})

        self.close_after_done = close_after_done
        self.has_window = False
        self.original_pref_values = {}
        self.group_metadata = group_metadata
        self.debug = debug

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
            handles = self.protocol.marionette.window_handles
            if handles:
                self.protocol.marionette.switch_to_window(handles[0])
            super(self.__class__, self).teardown()
        except Exception as e:
            # Ignore errors during teardown
            self.logger.warning("Exception during reftest teardown:\n%s" %
                                traceback.format_exc(e))

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
                self.protocol.base.execute_script(self.script)
                self.protocol.base.set_window(self.protocol.marionette.window_handles[-1])
                self.has_window = True

        if self.protocol.coverage.is_enabled:
            self.protocol.coverage.reset()

        result = self.implementation.run_test(test)

        if self.protocol.coverage.is_enabled:
            self.protocol.coverage.dump()

        if self.debug:
            assertion_count = self.protocol.asserts.get()
            if "extra" not in result:
                result["extra"] = {}
            result["extra"]["assertion_count"] = assertion_count

        return self.convert_result(test, result)

    def screenshot(self, test, viewport_size, dpi):
        # https://github.com/w3c/wptrunner/issues/166
        assert viewport_size is None
        assert dpi is None

        timeout = self.timeout_multiplier * test.timeout if self.debug_info is None else None

        test_url = self.test_url(test)

        return ExecuteAsyncScriptRun(self.logger,
                                     self._screenshot,
                                     self.protocol,
                                     test_url,
                                     timeout).run()

    def _screenshot(self, protocol, url, timeout):
        protocol.marionette.navigate(url)

        protocol.base.execute_script(self.wait_script, async=True)

        screenshot = protocol.marionette.screenshot(full=False)
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
        references = self.get_references(test)
        timeout = (test.timeout * 1000) * self.timeout_multiplier
        rv = self.executor.protocol.marionette._send_message("reftest:run",
                                                             {"test": self.executor.test_url(test),
                                                              "references": references,
                                                              "expected": test.expected(),
                                                              "timeout": timeout})["value"]
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
            self.logger.warning(traceback.format_exc(e))



class GeckoDriverProtocol(WebDriverProtocol):
    server_cls = GeckoDriverServer


class MarionetteWdspecExecutor(WdspecExecutor):
    protocol_cls = GeckoDriverProtocol
