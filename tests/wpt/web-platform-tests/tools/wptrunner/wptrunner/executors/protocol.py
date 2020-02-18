import traceback

from abc import ABCMeta, abstractmethod


class Protocol(object):
    """Backend for a specific browser-control protocol.

    Each Protocol is composed of a set of ProtocolParts that implement
    the APIs required for specific interactions. This reflects the fact
    that not all implementaions will support exactly the same feature set.
    Each ProtocolPart is exposed directly on the protocol through an accessor
    attribute with a name given by its `name` property.

    :param Executor executor: The Executor instance that's using this Protocol
    :param Browser browser: The Browser using this protocol"""
    __metaclass__ = ABCMeta

    implements = []

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
            if msg is not None:
                self.logger.warning(msg)
            self.logger.warning(traceback.format_exc())
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


class ProtocolPart(object):
    """Base class  for all ProtocolParts.

    :param Protocol parent: The parent protocol"""
    __metaclass__ = ABCMeta

    name = None

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
        """Wait indefinitely for the browser to close"""
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

    def element_by_selector(self, element_selector, frame="window"):
        elements = self.elements_by_selector_and_frame(element_selector, frame)
        frame_name = "window"
        if (frame != "window"):
            frame_name = frame.id
        if len(elements) == 0:
            raise ValueError("Selector '%s' in frame '%s' matches no elements" % (element_selector, frame_name))
        elif len(elements) > 1:
            raise ValueError("Selector '%s' in frame '%s' matches multiple elements" % (element_selector, frame_name))
        return elements[0]

    @abstractmethod
    def elements_by_selector(self, selector):
        """Select elements matching a CSS selector

        :param str selector: The CSS selector
        :returns: A list of protocol-specific handles to elements"""
        pass

    @abstractmethod
    def elements_by_selector_and_frame(self, element_selector, frame):
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
    def set_permission(self, descriptor, state, one_realm=False):
        """Set permission state.

        :param descriptor: A PermissionDescriptor object.
        :param state: The state to set the permission to.
        :param one_realm: Whether to set the permission for only one realm."""
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


class TestDriverProtocolPart(ProtocolPart):
    """Protocol part that implements the basic functionality required for
    all testdriver-based tests."""
    __metaclass__ = ABCMeta

    name = "testdriver"

    @abstractmethod
    def send_message(self, message_type, status, message=None):
        """Send a testdriver message to the browser.

        :param str message_type: The kind of the message.
        :param str status: Either "failure" or "success" depending on whether the
                           previous command succeeded.
        :param str message: Additional data to add to the message."""
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
