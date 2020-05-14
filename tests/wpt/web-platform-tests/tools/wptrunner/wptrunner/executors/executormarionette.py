import json
import os
import threading
import time
import traceback
import uuid

from six import iteritems, iterkeys
from six.moves.urllib.parse import urljoin

errors = None
marionette = None
pytestrunner = None

here = os.path.join(os.path.split(__file__)[0])

from .base import (CallbackHandler,
                   CrashtestExecutor,
                   RefTestExecutor,
                   RefTestImplementation,
                   TestharnessExecutor,
                   TimedRunner,
                   WdspecExecutor,
                   WebDriverProtocol,
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
                       CoverageProtocolPart,
                       GenerateTestReportProtocolPart,
                       VirtualAuthenticatorProtocolPart,
                       SetPermissionProtocolPart)
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

    def execute_script(self, script, asynchronous=False):
        method = self.marionette.execute_async_script if asynchronous else self.marionette.execute_script
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

    def load(self, url):
        self.marionette.navigate(url)

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
            except errors.JavascriptException as e:
                # This can happen if we navigate, but just keep going
                self.logger.debug(e.message)
                pass
            except IOError:
                self.logger.debug("Socket closed")
                break
            except Exception:
                self.logger.warning(traceback.format_exc())
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
        url = urljoin(self.parent.executor.server_url(url_protocol), "/testharness_runner.html")
        self.logger.debug("Loading %s" % url)
        try:
            self.dismiss_alert(lambda: self.marionette.navigate(url))
        except Exception:
            self.logger.critical(
                "Loading initial page %s failed. Ensure that the "
                "there are no other programs bound to this port and "
                "that your firewall rules or network setup does not "
                r"prevent access.\e%s" % (url, traceback.format_exc()))
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
                self.logger.info("Closing window %s" % handle)
                self.marionette.switch_to_window(handle)
                self.dismiss_alert(lambda: self.marionette.close())
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

    def get_test_window(self, window_id, parent, timeout=5):
        """Find the test window amongst all the open windows.
        This is assumed to be either the named window or the one after the parent in the list of
        window handles

        :param window_id: The DOM name of the Window
        :param parent: The handle of the runner window
        :param timeout: The time in seconds to wait for the window to appear. This is because in
                        some implementations there's a race between calling window.open and the
                        window being added to the list of WebDriver accessible windows."""
        test_window = None
        end_time = time.time() + timeout
        while time.time() < end_time:
            if window_id:
                try:
                    # Try this, it's in Level 1 but nothing supports it yet
                    win_s = self.parent.base.execute_script("return window['%s'];" % self.window_id)
                    win_obj = json.loads(win_s)
                    test_window = win_obj["window-fcc6-11e5-b4f8-330a88ab9d7f"]
                except Exception:
                    pass

            if test_window is None:
                handles = self.marionette.window_handles
                if len(handles) == 2:
                    test_window = next(iter(set(handles) - {parent}))
                elif len(handles) > 2 and handles[0] == parent:
                    # Hope the first one here is the test window
                    test_window = handles[1]

            if test_window is not None:
                assert test_window != parent
                return test_window

            time.sleep(0.1)

        raise Exception("unable to find test window")


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
                case prefInterface.PREF_INVALID:
                    // Pref doesn't seem to be defined already; guess at the
                    // right way to set it based on the type of value we have.
                    switch (typeof value) {
                        case "boolean":
                            prefInterface.setBoolPref(pref, value);
                            break;
                        case "string":
                            prefInterface.setCharPref(pref, value);
                            break;
                        case "number":
                            prefInterface.setIntPref(pref, value);
                            break;
                        default:
                            throw new Error("Unknown pref value type: " + (typeof value));
                    }
                    break;
                default:
                    throw new Error("Unknown pref type " + type);
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
            let principal = ssm.createContentPrincipal(uri, {});
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

    def elements_by_selector_and_frame(self, element_selector, frame):
        return self.marionette.find_elements("css selector", element_selector)


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
            const {PerTestCoverageUtils} = ChromeUtils.import("chrome://marionette/content/PerTestCoverageUtils.jsm");
            return PerTestCoverageUtils.enabled;
            """
        with self.marionette.using_context(self.marionette.CONTEXT_CHROME):
            self.is_enabled = self.marionette.execute_script(script)

    def reset(self):
        script = """
            var callback = arguments[arguments.length - 1];

            const {PerTestCoverageUtils} = ChromeUtils.import("chrome://marionette/content/PerTestCoverageUtils.jsm");
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

            const {PerTestCoverageUtils} = ChromeUtils.import("chrome://marionette/content/PerTestCoverageUtils.jsm");
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

class MarionetteGenerateTestReportProtocolPart(GenerateTestReportProtocolPart):
    def setup(self):
        self.marionette = self.parent.marionette

    def generate_test_report(self, config):
        raise NotImplementedError("generate_test_report not yet implemented")

class MarionetteVirtualAuthenticatorProtocolPart(VirtualAuthenticatorProtocolPart):
    def setup(self):
        self.marionette = self.parent.marionette

    def add_virtual_authenticator(self, config):
        raise NotImplementedError("add_virtual_authenticator not yet implemented")

    def remove_virtual_authenticator(self, authenticator_id):
        raise NotImplementedError("remove_virtual_authenticator not yet implemented")

    def add_credential(self, authenticator_id, credential):
        raise NotImplementedError("add_credential not yet implemented")

    def get_credentials(self, authenticator_id):
        raise NotImplementedError("get_credentials not yet implemented")

    def remove_credential(self, authenticator_id, credential_id):
        raise NotImplementedError("remove_credential not yet implemented")

    def remove_all_credentials(self, authenticator_id):
        raise NotImplementedError("remove_all_credentials not yet implemented")

    def set_user_verified(self, authenticator_id, uv):
        raise NotImplementedError("set_user_verified not yet implemented")

class MarionetteSetPermissionProtocolPart(SetPermissionProtocolPart):
    def setup(self):
        self.marionette = self.parent.marionette

    def set_permission(self, name, state, one_realm):
        raise NotImplementedError("set_permission not yet implemented")

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
                  MarionetteCoverageProtocolPart,
                  MarionetteGenerateTestReportProtocolPart,
                  MarionetteVirtualAuthenticatorProtocolPart,
                  MarionetteSetPermissionProtocolPart]

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
        if self.marionette and self.marionette.session_id:
            try:
                self.marionette._request_in_app_shutdown()
                self.marionette.delete_session(send_request=False)
            except Exception:
                # This is typically because the session never started
                pass
        if self.marionette is not None:
            del self.marionette
        super(MarionetteProtocol, self).teardown()

    def is_alive(self):
        try:
            self.marionette.current_window_handle
        except Exception:
            return False
        return True

    def on_environment_change(self, old_environment, new_environment):
        #Unset all the old prefs
        for name in iterkeys(old_environment.get("prefs", {})):
            value = self.executor.original_pref_values[name]
            if value is None:
                self.prefs.clear(name)
            else:
                self.prefs.set(name, value)

        for name, value in iteritems(new_environment.get("prefs", {})):
            self.executor.original_pref_values[name] = self.prefs.get(name)
            self.prefs.set(name, value)


class ExecuteAsyncScriptRun(TimedRunner):

    def set_timeout(self):
        timeout = self.timeout

        try:
            if timeout is not None:
                self.protocol.base.set_timeout(timeout + self.extra_timeout)
            else:
                # We just want it to never time out, really, but marionette doesn't
                # make that possible. It also seems to time out immediately if the
                # timeout is set too high. This works at least.
                self.protocol.base.set_timeout(2**28 - 1)
        except IOError:
            self.logger.error("Lost marionette connection before starting test")
            return Stop

    def before_run(self):
        index = self.url.rfind("/storage/")
        if index != -1:
            # Clear storage
            self.protocol.storage.clear_origin(self.url)

    def run_func(self):
        try:
            self.result = True, self.func(self.protocol, self.url, self.timeout)
        except errors.ScriptTimeoutException:
            self.logger.debug("Got a marionette timeout")
            self.result = False, ("EXTERNAL-TIMEOUT", None)
        except IOError:
            # This can happen on a crash
            # Also, should check after the test if the firefox process is still running
            # and otherwise ignore any other result and set it to crash
            self.logger.info("IOError on command, setting status to CRASH")
            self.result = False, ("CRASH", None)
        except errors.NoSuchWindowException:
            self.logger.info("NoSuchWindowException on command, setting status to CRASH")
            self.result = False, ("CRASH", None)
        except Exception as e:
            if isinstance(e, errors.JavascriptException) and e.message.startswith("Document was unloaded"):
                message = "Document unloaded; maybe test navigated the top-level-browsing context?"
            else:
                message = getattr(e, "message", "")
                if message:
                    message += "\n"
                message += traceback.format_exc()
                self.logger.warning(traceback.format_exc())
            self.result = False, ("INTERNAL-ERROR", message)
        finally:
            self.result_flag.set()


class MarionetteTestharnessExecutor(TestharnessExecutor):
    supports_testdriver = True

    def __init__(self, logger, browser, server_config, timeout_multiplier=1,
                 close_after_done=True, debug_info=None, capabilities=None,
                 debug=False, ccov=False, **kwargs):
        """Marionette-based executor for testharness.js tests"""
        TestharnessExecutor.__init__(self, logger, browser, server_config,
                                     timeout_multiplier=timeout_multiplier,
                                     debug_info=debug_info)
        self.protocol = MarionetteProtocol(self,
                                           browser,
                                           capabilities,
                                           timeout_multiplier,
                                           kwargs["e10s"],
                                           ccov)
        with open(os.path.join(here, "testharness_webdriver_resume.js")) as f:
            self.script_resume = f.read()
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
        return self.protocol.is_alive()

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
                                              timeout,
                                              self.extra_timeout).run()
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

        if self.protocol.coverage.is_enabled:
            self.protocol.coverage.reset()

        format_map = {"url": strip_server(url)}

        protocol.base.execute_script("window.open(undefined, '%s', 'noopener')" % self.window_id)
        test_window = protocol.testharness.get_test_window(self.window_id, parent_window,
                                                           timeout=10*self.timeout_multiplier)
        self.protocol.base.set_window(test_window)
        handler = CallbackHandler(self.logger, protocol, test_window)
        protocol.marionette.navigate(url)
        while True:
            result = protocol.base.execute_script(
                self.script_resume % format_map, asynchronous=True)
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
    def __init__(self, logger, browser, server_config, timeout_multiplier=1,
                 screenshot_cache=None, close_after_done=True,
                 debug_info=None, reftest_internal=False,
                 reftest_screenshot="unexpected", ccov=False,
                 group_metadata=None, capabilities=None, debug=False, **kwargs):
        """Marionette-based executor for reftests"""
        RefTestExecutor.__init__(self,
                                 logger,
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
        with open(os.path.join(here, "test-wait.js")) as f:
            self.wait_script = f.read() % {"classname": "reftest-wait"}

    def setup(self, runner):
        super(MarionetteRefTestExecutor, self).setup(runner)
        self.implementation.setup(**self.implementation_kwargs)

    def teardown(self):
        try:
            self.implementation.teardown()
            if self.protocol.marionette and self.protocol.marionette.session_id:
                handles = self.protocol.marionette.window_handles
                if handles:
                    self.protocol.marionette.switch_to_window(handles[0])
            super(MarionetteRefTestExecutor, self).teardown()
        except Exception:
            # Ignore errors during teardown
            self.logger.warning("Exception during reftest teardown:\n%s" %
                                traceback.format_exc())

    def reset(self):
        self.implementation.reset(**self.implementation_kwargs)

    def is_alive(self):
        return self.protocol.is_alive()

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
        # https://github.com/web-platform-tests/wpt/issues/7135
        assert viewport_size is None
        assert dpi is None

        timeout = self.timeout_multiplier * test.timeout if self.debug_info is None else None

        test_url = self.test_url(test)

        return ExecuteAsyncScriptRun(self.logger,
                                     self._screenshot,
                                     self.protocol,
                                     test_url,
                                     timeout,
                                     self.extra_timeout).run()

    def _screenshot(self, protocol, url, timeout):
        protocol.marionette.navigate(url)

        protocol.base.execute_script(self.wait_script, asynchronous=True)

        screenshot = protocol.marionette.screenshot(full=False)
        # strip off the data:img/png, part of the url
        if screenshot.startswith("data:image/png;base64,"):
            screenshot = screenshot.split(",", 1)[1]

        return screenshot


class InternalRefTestImplementation(RefTestImplementation):
    def __init__(self, executor):
        self.timeout_multiplier = executor.timeout_multiplier
        self.executor = executor

    @property
    def logger(self):
        return self.executor.logger

    def setup(self, screenshot="unexpected"):
        data = {"screenshot": screenshot}
        if self.executor.group_metadata is not None:
            data["urlCount"] = {urljoin(self.executor.server_url(key[0]), key[1]):value
                                for key, value in iteritems(
                                    self.executor.group_metadata.get("url_count", {}))
                                if value > 1}
        self.executor.protocol.marionette.set_context(self.executor.protocol.marionette.CONTEXT_CHROME)
        self.executor.protocol.marionette._send_message("reftest:setup", data)

    def reset(self, screenshot=None):
        # this is obvious wrong; it shouldn't be a no-op
        # see https://github.com/web-platform-tests/wpt/issues/15604
        pass

    def run_test(self, test):
        references = self.get_references(test, test)
        timeout = (test.timeout * 1000) * self.timeout_multiplier
        rv = self.executor.protocol.marionette._send_message("reftest:run",
                                                             {"test": self.executor.test_url(test),
                                                              "references": references,
                                                              "expected": test.expected(),
                                                              "timeout": timeout,
                                                              "width": 800,
                                                              "height": 600})["value"]
        return rv

    def get_references(self, root_test, node):
        rv = []
        for item, relation in node.references:
            rv.append([self.executor.test_url(item), self.get_references(root_test, item), relation,
                       {"fuzzy": self.get_fuzzy(root_test, [node, item], relation)}])
        return rv

    def teardown(self):
        try:
            if self.executor.protocol.marionette and self.executor.protocol.marionette.session_id:
                self.executor.protocol.marionette._send_message("reftest:teardown", {})
                self.executor.protocol.marionette.set_context(self.executor.protocol.marionette.CONTEXT_CONTENT)
                # the reftest runner opens/closes a window with focus, so as
                # with after closing a window we need to give a new window
                # focus
                handles = self.executor.protocol.marionette.window_handles
                if handles:
                    self.executor.protocol.marionette.switch_to_window(handles[0])
        except Exception:
            # Ignore errors during teardown
            self.logger.warning(traceback.format_exc())


class GeckoDriverProtocol(WebDriverProtocol):
    server_cls = GeckoDriverServer


class MarionetteWdspecExecutor(WdspecExecutor):
    protocol_cls = GeckoDriverProtocol


class MarionetteCrashtestExecutor(CrashtestExecutor):
    def __init__(self, logger, browser, server_config, timeout_multiplier=1,
                 debug_info=None, capabilities=None, debug=False,
                 ccov=False, **kwargs):
        """Marionette-based executor for testharness.js tests"""
        CrashtestExecutor.__init__(self, logger, browser, server_config,
                                   timeout_multiplier=timeout_multiplier,
                                   debug_info=debug_info)
        self.protocol = MarionetteProtocol(self,
                                           browser,
                                           capabilities,
                                           timeout_multiplier,
                                           kwargs["e10s"],
                                           ccov)

        self.original_pref_values = {}
        self.debug = debug

        with open(os.path.join(here, "test-wait.js")) as f:
            self.wait_script = f.read() % {"classname": "test-wait"}

        if marionette is None:
            do_delayed_imports()

    def is_alive(self):
        return self.protocol.is_alive()

    def on_environment_change(self, new_environment):
        self.protocol.on_environment_change(self.last_environment, new_environment)

    def do_test(self, test):
        timeout = (test.timeout * self.timeout_multiplier if self.debug_info is None
                   else None)

        success, data = ExecuteAsyncScriptRun(self.logger,
                                              self.do_crashtest,
                                              self.protocol,
                                              self.test_url(test),
                                              timeout,
                                              self.extra_timeout).run()
        status = None
        if not success:
            status = data[0]

        extra = None
        if self.debug and (success or status not in ("CRASH", "INTERNAL-ERROR")):
            assertion_count = self.protocol.asserts.get()
            if assertion_count is not None:
                extra = {"assertion_count": assertion_count}

        if success:
            return self.convert_result(test, data)

        return (test.result_cls(extra=extra, *data), [])

    def do_crashtest(self, protocol, url, timeout):
        if self.protocol.coverage.is_enabled:
            self.protocol.coverage.reset()

        protocol.base.load(url)
        protocol.base.execute_script(self.wait_script, asynchronous=True)

        if self.protocol.coverage.is_enabled:
            self.protocol.coverage.dump()

        return {"status": "PASS",
                "message": None}
