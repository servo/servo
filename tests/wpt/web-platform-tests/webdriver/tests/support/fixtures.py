import copy
import json
import os

import pytest
import webdriver

from six import string_types

from six.moves.urllib.parse import urlunsplit

from tests.support import defaults
from tests.support.helpers import cleanup_session
from tests.support.http_request import HTTPRequest
from tests.support.sync import Poll


_current_session = None
_custom_session = False


def pytest_configure(config):
    # register the capabilities marker
    config.addinivalue_line("markers",
        "capabilities: mark test to use capabilities")


@pytest.fixture
def capabilities():
    """Default capabilities to use for a new WebDriver session."""
    return {}


def pytest_generate_tests(metafunc):
    if "capabilities" in metafunc.fixturenames:
        marker = metafunc.definition.get_closest_marker(name="capabilities")
        if marker:
            metafunc.parametrize("capabilities", marker.args, ids=None)


@pytest.fixture
def add_event_listeners(session):
    """Register listeners for tracked events on element."""
    def add_event_listeners(element, tracked_events):
        element.session.execute_script("""
            let element = arguments[0];
            let trackedEvents = arguments[1];

            if (!("events" in window)) {
              window.events = [];
            }

            for (var i = 0; i < trackedEvents.length; i++) {
              element.addEventListener(trackedEvents[i], function (event) {
                window.events.push(event.type);
              });
            }
            """, args=(element, tracked_events))
    return add_event_listeners


@pytest.fixture
def create_cookie(session, url):
    """Create a cookie"""
    def create_cookie(name, value, **kwargs):
        if kwargs.get("path", None) is not None:
            session.url = url(kwargs["path"])

        session.set_cookie(name, value, **kwargs)
        return session.cookies(name)

    return create_cookie


@pytest.fixture
def create_frame(session):
    """Create an `iframe` element in the current browsing context and insert it
    into the document. Return a reference to the newly-created element."""
    def create_frame():
        append = """
            var frame = document.createElement('iframe');
            document.body.appendChild(frame);
            return frame;
        """
        return session.execute_script(append)

    return create_frame


@pytest.fixture
def http(configuration):
    return HTTPRequest(configuration["host"], configuration["port"])


@pytest.fixture
def server_config():
    return json.loads(os.environ.get("WD_SERVER_CONFIG"))


@pytest.fixture(scope="session")
def configuration():
    host = os.environ.get("WD_HOST", defaults.DRIVER_HOST)
    port = int(os.environ.get("WD_PORT", str(defaults.DRIVER_PORT)))
    capabilities = json.loads(os.environ.get("WD_CAPABILITIES", "{}"))

    return {
        "host": host,
        "port": port,
        "capabilities": capabilities
    }


@pytest.fixture(scope="function")
def session(capabilities, configuration, request):
    """Create and start a session for a test that does not itself test session creation.

    By default the session will stay open after each test, but we always try to start a
    new one and assume that if that fails there is already a valid session. This makes it
    possible to recover from some errors that might leave the session in a bad state, but
    does not demand that we start a new session per test."""
    global _current_session

    # Update configuration capabilities with custom ones from the
    # capabilities fixture, which can be set by tests
    caps = copy.deepcopy(configuration["capabilities"])
    caps.update(capabilities)
    caps = {"alwaysMatch": caps}

    # If there is a session with different capabilities active, end it now
    if _current_session is not None and (
            caps != _current_session.requested_capabilities):
        _current_session.end()
        _current_session = None

    if _current_session is None:
        _current_session = webdriver.Session(
            configuration["host"],
            configuration["port"],
            capabilities=caps)
    try:
        _current_session.start()
    except webdriver.error.SessionNotCreatedException:
        if not _current_session.session_id:
            raise

    # Enforce a fixed default window size and position
    _current_session.window.size = defaults.WINDOW_SIZE
    _current_session.window.position = defaults.WINDOW_POSITION

    yield _current_session

    cleanup_session(_current_session)


@pytest.fixture(scope="function")
def current_session():
    return _current_session


@pytest.fixture
def url(server_config):
    def inner(path, protocol="http", domain="", subdomain="", query="", fragment=""):
        domain = server_config["domains"][domain][subdomain]
        port = server_config["ports"][protocol][0]
        host = "{0}:{1}".format(domain, port)
        return urlunsplit((protocol, host, path, query, fragment))

    inner.__name__ = "url"
    return inner


@pytest.fixture
def create_dialog(session):
    """Create a dialog (one of "alert", "prompt", or "confirm") and provide a
    function to validate that the dialog has been "handled" (either accepted or
    dismissed) by returning some value."""

    def create_dialog(dialog_type, text=None):
        assert dialog_type in ("alert", "confirm", "prompt"), (
            "Invalid dialog type: '%s'" % dialog_type)

        if text is None:
            text = ""

        assert isinstance(text, string_types), "`text` parameter must be a string"

        # Script completes itself when the user prompt has been opened.
        # For prompt() dialogs, add a value for the 'default' argument,
        # as some user agents (IE, for example) do not produce consistent
        # values for the default.
        session.execute_async_script("""
            let dialog_type = arguments[0];
            let text = arguments[1];

            setTimeout(function() {
              if (dialog_type == 'prompt') {
                window.dialog_return_value = window[dialog_type](text, '');
              } else {
                window.dialog_return_value = window[dialog_type](text);
              }
            }, 0);
            """, args=(dialog_type, text))

        wait = Poll(
            session,
            timeout=15,
            ignored_exceptions=webdriver.NoSuchAlertException,
            message="No user prompt with text '{}' detected".format(text))
        wait.until(lambda s: s.alert.text == text)

    return create_dialog


@pytest.fixture
def closed_frame(session, url):
    original_handle = session.window_handle
    new_handle = session.new_window()

    session.window_handle = new_handle

    session.url = url("/webdriver/tests/support/html/frames.html")

    subframe = session.find.css("#sub-frame", all=False)
    session.switch_frame(subframe)

    deleteframe = session.find.css("#delete-frame", all=False)
    session.switch_frame(deleteframe)

    button = session.find.css("#remove-parent", all=False)
    button.click()

    yield

    session.window.close()
    assert new_handle not in session.handles, "Unable to close window {}".format(new_handle)

    session.window_handle = original_handle


@pytest.fixture
def closed_window(session):
    original_handle = session.window_handle
    new_handle = session.new_window()

    session.window_handle = new_handle

    session.window.close()
    assert new_handle not in session.handles, "Unable to close window {}".format(new_handle)

    yield new_handle

    session.window_handle = original_handle
