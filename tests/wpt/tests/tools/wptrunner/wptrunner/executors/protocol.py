# mypy: allow-untyped-defs

import traceback
from http.client import HTTPConnection

from abc import ABCMeta, abstractmethod
from typing import ClassVar, List, Type


def merge_dicts(target, source):
    if not (isinstance(target, dict) and isinstance(source, dict)):
        raise TypeError
    for (key, source_value) in source.items():
        if key not in target:
            target[key] = source_value
        else:
            if isinstance(source_value, dict) and isinstance(target[key], dict):
                merge_dicts(target[key], source_value)
            else:
                target[key] = source_value

class Protocol:
    """Backend for a specific browser-control protocol.

    Each Protocol is composed of a set of ProtocolParts that implement
    the APIs required for specific interactions. This reflects the fact
    that not all implementaions will support exactly the same feature set.
    Each ProtocolPart is exposed directly on the protocol through an accessor
    attribute with a name given by its `name` property.

    :param Executor executor: The Executor instance that's using this Protocol
    :param Browser browser: The Browser using this protocol"""
    __metaclass__ = ABCMeta

    implements: ClassVar[List[Type["ProtocolPart"]]] = []

    def __init__(self, executor, browser):
        self.executor = executor
        self.browser = browser

        for cls in self.implements:
            name = cls.name
            assert not hasattr(self, name)
            setattr(self, name, cls(self))

    @property
    def logger(self):
        """:returns: Current logger"""
        return self.executor.logger

    def is_alive(self):
        """Is the browser connection still active

        :returns: A boolean indicating whether the connection is still active."""
        return True

    def setup(self, runner):
        """Handle protocol setup, and send a message to the runner to indicate
        success or failure."""
        msg = None
        try:
            msg = "Failed to start protocol connection"
            self.connect()

            msg = None

            for cls in self.implements:
                getattr(self, cls.name).setup()

            msg = "Post-connection steps failed"
            self.after_connect()
        except Exception:
            message = "Protocol.setup caught an exception:\n"
            message += f"{msg}\n" if msg is not None else ""
            message += traceback.format_exc()
            self.logger.warning(message)
            raise

    @abstractmethod
    def connect(self):
        """Make a connection to the remote browser"""
        pass

    @abstractmethod
    def after_connect(self):
        """Run any post-connection steps. This happens after the ProtocolParts are
        initalized so can depend on a fully-populated object."""
        pass

    def teardown(self):
        """Run cleanup steps after the tests are finished."""
        for cls in self.implements:
            getattr(self, cls.name).teardown()


class ProtocolPart:
    """Base class  for all ProtocolParts.

    :param Protocol parent: The parent protocol"""
    __metaclass__ = ABCMeta

    name: ClassVar[str]

    def __init__(self, parent):
        self.parent = parent

    @property
    def logger(self):
        """:returns: Current logger"""
        return self.parent.logger

    def setup(self):
        """Run any setup steps required for the ProtocolPart."""
        pass

    def teardown(self):
        """Run any teardown steps required for the ProtocolPart."""
        pass


class BaseProtocolPart(ProtocolPart):
    """Generic bits of protocol that are required for multiple test types"""
    __metaclass__ = ABCMeta

    name = "base"

    @abstractmethod
    def execute_script(self, script, asynchronous=False):
        """Execute javascript in the current Window.

        :param str script: The js source to execute. This is implicitly wrapped in a function.
        :param bool asynchronous: Whether the script is asynchronous in the webdriver
                           sense i.e. whether the return value is the result of
                           the initial function call or if it waits for some callback.
        :returns: The result of the script execution.
        """
        pass

    @abstractmethod
    def set_timeout(self, timeout):
        """Set the timeout for script execution.

        :param timeout: Script timeout in seconds"""
        pass

    @abstractmethod
    def wait(self):
        """Wait indefinitely for the browser to close.

        :returns: True to re-run the test, or False to continue with the next test"""
        pass

    @property
    def current_window(self):
        """Return a handle identifying the current top level browsing context

        :returns: A protocol-specific handle"""
        pass

    @abstractmethod
    def set_window(self, handle):
        """Set the top level browsing context to one specified by a given handle.

        :param handle: A protocol-specific handle identifying a top level browsing
                       context."""
        pass

    @abstractmethod
    def window_handles(self):
        """Get a list of handles to top-level browsing contexts"""
        pass

    @abstractmethod
    def load(self, url):
        """Load a url in the current browsing context

        :param url: The url to load"""
        pass


class TestharnessProtocolPart(ProtocolPart):
    """Protocol part required to run testharness tests."""
    __metaclass__ = ABCMeta

    name = "testharness"

    @abstractmethod
    def load_runner(self, url_protocol):
        """Load the initial page used to control the tests.

        :param str url_protocol: "https" or "http" depending on the test metadata.
        """
        pass

    @abstractmethod
    def close_old_windows(self, url_protocol):
        """Close existing windows except for the initial runner window.
        After calling this method there must be exactly one open window that
        contains the initial runner page.

        :param str url_protocol: "https" or "http" depending on the test metadata.
        """
        pass

    @abstractmethod
    def get_test_window(self, window_id, parent):
        """Get the window handle dorresponding to the window containing the
        currently active test.

        :param window_id: A string containing the DOM name of the Window that
        contains the test, or None.
        :param parent: The handle of the runner window.
        :returns: A protocol-specific window handle.
        """
        pass

    @abstractmethod
    def test_window_loaded(self):
        """Wait until the newly opened test window has been loaded."""


class PrefsProtocolPart(ProtocolPart):
    """Protocol part that allows getting and setting browser prefs."""
    __metaclass__ = ABCMeta

    name = "prefs"

    @abstractmethod
    def set(self, name, value):
        """Set the named pref to value.

        :param name: A pref name of browser-specific type
        :param value: A pref value of browser-specific type"""
        pass

    @abstractmethod
    def get(self, name):
        """Get the current value of a named pref

        :param name: A pref name of browser-specific type
        :returns: A pref value of browser-specific type"""
        pass

    @abstractmethod
    def clear(self, name):
        """Reset the value of a named pref back to the default.

        :param name: A pref name of browser-specific type"""
        pass


class StorageProtocolPart(ProtocolPart):
    """Protocol part for manipulating browser storage."""
    __metaclass__ = ABCMeta

    name = "storage"

    @abstractmethod
    def clear_origin(self, url):
        """Clear all the storage for a specified origin.

        :param url: A url belonging to the origin"""
        pass


class SelectorProtocolPart(ProtocolPart):
    """Protocol part for selecting elements on the page."""
    __metaclass__ = ABCMeta

    name = "select"

    def element_by_selector(self, element_selector):
        elements = self.elements_by_selector(element_selector)
        if len(elements) == 0:
            raise ValueError(f"Selector '{element_selector}' matches no elements")
        elif len(elements) > 1:
            raise ValueError(f"Selector '{element_selector}' matches multiple elements")
        return elements[0]

    @abstractmethod
    def elements_by_selector(self, selector):
        """Select elements matching a CSS selector

        :param str selector: The CSS selector
        :returns: A list of protocol-specific handles to elements"""
        pass


class ClickProtocolPart(ProtocolPart):
    """Protocol part for performing trusted clicks"""
    __metaclass__ = ABCMeta

    name = "click"

    @abstractmethod
    def element(self, element):
        """Perform a trusted click somewhere on a specific element.

        :param element: A protocol-specific handle to an element."""
        pass



class AccessibilityProtocolPart(ProtocolPart):
    """Protocol part for accessibility introspection"""
    __metaclass__ = ABCMeta

    name = "accessibility"

    @abstractmethod
    def get_computed_label(self, element):
        """Return the computed accessibility label for a specific element.

        :param element: A protocol-specific handle to an element."""
        pass

    def get_computed_role(self, element):
        """Return the computed accessibility role for a specific element.

        :param element: A protocol-specific handle to an element."""
        pass


class CookiesProtocolPart(ProtocolPart):
    """Protocol part for managing cookies"""
    __metaclass__ = ABCMeta

    name = "cookies"

    @abstractmethod
    def delete_all_cookies(self):
        """Delete all cookies."""
        pass

    @abstractmethod
    def get_all_cookies(self):
        """Get all cookies."""
        pass

    @abstractmethod
    def get_named_cookie(self, name):
        """Get named cookie.

        :param name: The name of the cookie to get."""
        pass


class SendKeysProtocolPart(ProtocolPart):
    """Protocol part for performing trusted clicks"""
    __metaclass__ = ABCMeta

    name = "send_keys"

    @abstractmethod
    def send_keys(self, element, keys):
        """Send keys to a specific element.

        :param element: A protocol-specific handle to an element.
        :param keys: A protocol-specific handle to a string of input keys."""
        pass

class WindowProtocolPart(ProtocolPart):
    """Protocol part for manipulating the window"""
    __metaclass__ = ABCMeta

    name = "window"

    @abstractmethod
    def set_rect(self, rect):
        """Restores the window to the given rect."""
        pass

    @abstractmethod
    def minimize(self):
        """Minimizes the window and returns the previous rect."""
        pass

class GenerateTestReportProtocolPart(ProtocolPart):
    """Protocol part for generating test reports"""
    __metaclass__ = ABCMeta

    name = "generate_test_report"

    @abstractmethod
    def generate_test_report(self, message):
        """Generate a test report.

        :param message: The message to be contained in the report."""
        pass


class SetPermissionProtocolPart(ProtocolPart):
    """Protocol part for setting permissions"""
    __metaclass__ = ABCMeta

    name = "set_permission"

    @abstractmethod
    def set_permission(self, descriptor, state):
        """Set permission state.

        :param descriptor: A PermissionDescriptor object.
        :param state: The state to set the permission to."""
        pass


class ActionSequenceProtocolPart(ProtocolPart):
    """Protocol part for performing trusted clicks"""
    __metaclass__ = ABCMeta

    name = "action_sequence"

    @abstractmethod
    def send_actions(self, actions):
        """Send a sequence of actions to the window.

        :param actions: A protocol-specific handle to an array of actions."""
        pass

    def release(self):
        pass


class TestDriverProtocolPart(ProtocolPart):
    """Protocol part that implements the basic functionality required for
    all testdriver-based tests."""
    __metaclass__ = ABCMeta

    name = "testdriver"

    @abstractmethod
    def send_message(self, cmd_id, message_type, status, message=None):
        """Send a testdriver message to the browser.

        :param int cmd_id: The id of the command to which we're responding
        :param str message_type: The kind of the message.
        :param str status: Either "failure" or "success" depending on whether the
                           previous command succeeded.
        :param str message: Additional data to add to the message."""
        pass

    def switch_to_window(self, wptrunner_id, initial_window=None):
        """Switch to a window given a wptrunner window id

        :param str wptrunner_id: Testdriver-specific id for the target window
        :param str initial_window: WebDriver window id for the test window"""
        if wptrunner_id is None:
            return

        if initial_window is None:
            initial_window = self.parent.base.current_window

        stack = [str(item) for item in self.parent.base.window_handles()]
        first = True
        while stack:
            item = stack.pop()

            if item is None:
                assert first is False
                self._switch_to_parent_frame()
                continue

            if isinstance(item, str):
                if not first or item != initial_window:
                    self.parent.base.set_window(item)
                first = False
            else:
                assert first is False
                try:
                    self._switch_to_frame(item)
                except ValueError:
                    # The frame no longer exists, or doesn't have a nested browsing context, so continue
                    continue

            try:
                # Get the window id and a list of elements containing nested browsing contexts.
                # For embed we can't tell fpr sure if there's a nested browsing context, so always return it
                # and fail later if there isn't
                result = self.parent.base.execute_script("""
                let contextParents = Array.from(document.querySelectorAll("frame, iframe, embed, object"))
                    .filter(elem => elem.localName !== "embed" ? (elem.contentWindow !== null) : true);
                return [window.__wptrunner_id, contextParents]""")
            except Exception:
                continue

            if result is None:
                # With marionette at least this is possible if the content process crashed. Not quite
                # sure how we want to handle that case.
                continue

            handle_window_id, nested_context_containers = result

            if handle_window_id and str(handle_window_id) == wptrunner_id:
                return

            for elem in reversed(nested_context_containers):
                # None here makes us switch back to the parent after we've processed the frame
                stack.append(None)
                stack.append(elem)

        raise Exception("Window with id %s not found" % wptrunner_id)

    @abstractmethod
    def _switch_to_frame(self, index_or_elem):
        """Switch to a frame in the current window

        :param int index_or_elem: Frame id or container element"""
        pass

    @abstractmethod
    def _switch_to_parent_frame(self):
        """Switch to the parent of the current frame"""
        pass


class AssertsProtocolPart(ProtocolPart):
    """ProtocolPart that implements the functionality required to get a count of non-fatal
    assertions triggered"""
    __metaclass__ = ABCMeta

    name = "asserts"

    @abstractmethod
    def get(self):
        """Get a count of assertions since the last browser start"""
        pass


class CoverageProtocolPart(ProtocolPart):
    """Protocol part for collecting per-test coverage data."""
    __metaclass__ = ABCMeta

    name = "coverage"

    @abstractmethod
    def reset(self):
        """Reset coverage counters"""
        pass

    @abstractmethod
    def dump(self):
        """Dump coverage counters"""
        pass


class VirtualAuthenticatorProtocolPart(ProtocolPart):
    """Protocol part for creating and manipulating virtual authenticators"""
    __metaclass__ = ABCMeta

    name = "virtual_authenticator"

    @abstractmethod
    def add_virtual_authenticator(self, config):
        """Add a virtual authenticator

        :param config: The Authenticator Configuration"""
        pass

    @abstractmethod
    def remove_virtual_authenticator(self, authenticator_id):
        """Remove a virtual authenticator

        :param str authenticator_id: The ID of the authenticator to remove"""
        pass

    @abstractmethod
    def add_credential(self, authenticator_id, credential):
        """Inject a credential onto an authenticator

        :param str authenticator_id: The ID of the authenticator to add the credential to
        :param credential: The credential to inject"""
        pass

    @abstractmethod
    def get_credentials(self, authenticator_id):
        """Get the credentials stored in an authenticator

        :param str authenticator_id: The ID of the authenticator
        :returns: An array with the credentials stored on the authenticator"""
        pass

    @abstractmethod
    def remove_credential(self, authenticator_id, credential_id):
        """Remove a credential stored in an authenticator

        :param str authenticator_id: The ID of the authenticator
        :param str credential_id: The ID of the credential"""
        pass

    @abstractmethod
    def remove_all_credentials(self, authenticator_id):
        """Remove all the credentials stored in an authenticator

        :param str authenticator_id: The ID of the authenticator"""
        pass

    @abstractmethod
    def set_user_verified(self, authenticator_id, uv):
        """Sets the user verified flag on an authenticator

        :param str authenticator_id: The ID of the authenticator
        :param bool uv: the user verified flag"""
        pass


class SPCTransactionsProtocolPart(ProtocolPart):
    """Protocol part for Secure Payment Confirmation transactions"""
    __metaclass__ = ABCMeta

    name = "spc_transactions"

    @abstractmethod
    def set_spc_transaction_mode(self, mode):
        """Set the SPC transaction automation mode

        :param str mode: The automation mode to set"""
        pass


class FedCMProtocolPart(ProtocolPart):
    """Protocol part for Federated Credential Management"""
    __metaclass__ = ABCMeta

    name = "fedcm"

    @abstractmethod
    def cancel_fedcm_dialog(self):
        """Cancel the FedCM dialog"""
        pass

    @abstractmethod
    def confirm_idp_login(self):
        """Confirm IDP login"""
        pass

    @abstractmethod
    def select_fedcm_account(self, account_index):
        """Select a FedCM account

        :param int account_index: The index of the account to select"""
        pass

    @abstractmethod
    def get_fedcm_account_list(self):
        """Get the FedCM account list"""
        pass

    @abstractmethod
    def get_fedcm_dialog_title(self):
        """Get the FedCM dialog title"""
        pass

    @abstractmethod
    def get_fedcm_dialog_type(self):
        """Get the FedCM dialog type"""
        pass

    @abstractmethod
    def set_fedcm_delay_enabled(self, enabled):
        """Sets the FedCM delay as enabled or disabled

        :param bool enabled: The delay to set"""
        pass

    @abstractmethod
    def reset_fedcm_cooldown(self):
        """Set the FedCM cooldown"""
        pass


class PrintProtocolPart(ProtocolPart):
    """Protocol part for rendering to a PDF."""
    __metaclass__ = ABCMeta

    name = "pdf_print"

    @abstractmethod
    def render_as_pdf(self, width, height):
        """Output document as PDF"""
        pass


class DebugProtocolPart(ProtocolPart):
    """Protocol part for debugging test failures."""
    __metaclass__ = ABCMeta

    name = "debug"

    @abstractmethod
    def load_devtools(self):
        """Load devtools in the current window"""
        pass

    def load_reftest_analyzer(self, test, result):
        import io
        import mozlog
        from urllib.parse import quote, urljoin

        debug_test_logger = mozlog.structuredlog.StructuredLogger("debug_test")
        output = io.StringIO()
        debug_test_logger.suite_start([])
        debug_test_logger.add_handler(mozlog.handlers.StreamHandler(output, formatter=mozlog.formatters.TbplFormatter()))
        debug_test_logger.test_start(test.id)
        # Always use PASS as the expected value so we get output even for expected failures
        debug_test_logger.test_end(test.id, result["status"], "PASS", extra=result.get("extra"))

        self.parent.base.load(urljoin(self.parent.executor.server_url("https"),
                              "/common/third_party/reftest-analyzer.xhtml#log=%s" %
                               quote(output.getvalue())))


class ConnectionlessBaseProtocolPart(BaseProtocolPart):
    def load(self, url):
        pass

    def execute_script(self, script, asynchronous=False):
        pass

    def set_timeout(self, timeout):
        pass

    def wait(self):
        return False

    def set_window(self, handle):
        pass

    def window_handles(self):
        return []


class ConnectionlessProtocol(Protocol):
    implements = [ConnectionlessBaseProtocolPart]

    def connect(self):
        pass

    def after_connect(self):
        pass


class WdspecProtocol(ConnectionlessProtocol):
    implements = [ConnectionlessBaseProtocolPart]

    def __init__(self, executor, browser):
        super().__init__(executor, browser)

    def is_alive(self):
        """Test that the connection is still alive.

        Because the remote communication happens over HTTP we need to
        make an explicit request to the remote.  It is allowed for
        WebDriver spec tests to not have a WebDriver session, since this
        may be what is tested.

        An HTTP request to an invalid path that results in a 404 is
        proof enough to us that the server is alive and kicking.
        """
        conn = HTTPConnection(self.browser.host, self.browser.port)
        conn.request("HEAD", "/invalid")
        res = conn.getresponse()
        return res.status == 404


class VirtualSensorProtocolPart(ProtocolPart):
    """Protocol part for Sensors"""
    __metaclass__ = ABCMeta

    name = "virtual_sensor"

    @abstractmethod
    def create_virtual_sensor(self, sensor_type, sensor_params):
        pass

    @abstractmethod
    def update_virtual_sensor(self, sensor_type, reading):
        pass

    @abstractmethod
    def remove_virtual_sensor(self, sensor_type):
        pass

    @abstractmethod
    def get_virtual_sensor_information(self, sensor_type):
        pass
