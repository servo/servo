from __future__ import print_function

import copy
import json
import os
import urlparse
import re
import sys

import webdriver

from tests.support.http_request import HTTPRequest
from tests.support.wait import wait

default_host = "http://127.0.0.1"
default_port = "4444"

default_script_timeout = 30
default_page_load_timeout = 300
default_implicit_wait_timeout = 0


_current_session = None
_custom_session = False


def ignore_exceptions(f):
    def inner(*args, **kwargs):
        try:
            return f(*args, **kwargs)
        except webdriver.error.WebDriverException as e:
            print("Ignored exception %s" % e, file=sys.stderr)
    inner.__name__ = f.__name__
    return inner


def cleanup_session(session):
    """Clean-up the current session for a clean state."""
    @ignore_exceptions
    def _dismiss_user_prompts(session):
        """Dismiss any open user prompts in windows."""
        current_window = session.window_handle

        for window in _windows(session):
            session.window_handle = window
            try:
                session.alert.dismiss()
            except webdriver.NoSuchAlertException:
                pass

        session.window_handle = current_window

    @ignore_exceptions
    def _ensure_valid_window(session):
        """If current window was closed, ensure to have a valid one selected."""
        try:
            session.window_handle
        except webdriver.NoSuchWindowException:
            session.window_handle = session.handles[0]

    @ignore_exceptions
    def _restore_timeouts(session):
        """Restore modified timeouts to their default values."""
        session.timeouts.implicit = default_implicit_wait_timeout
        session.timeouts.page_load = default_page_load_timeout
        session.timeouts.script = default_script_timeout

    @ignore_exceptions
    def _restore_window_state(session):
        """Reset window to an acceptable size.

        This also includes bringing it out of maximized, minimized,
        or fullscreened state.
        """
        session.window.size = (800, 600)

    @ignore_exceptions
    def _restore_windows(session):
        """Close superfluous windows opened by the test.

        It will not end the session implicitly by closing the last window.
        """
        current_window = session.window_handle

        for window in _windows(session, exclude=[current_window]):
            session.window_handle = window
            if len(session.handles) > 1:
                session.close()

        session.window_handle = current_window

    _restore_timeouts(session)
    _ensure_valid_window(session)
    _dismiss_user_prompts(session)
    _restore_windows(session)
    _restore_window_state(session)
    _switch_to_top_level_browsing_context(session)


@ignore_exceptions
def _switch_to_top_level_browsing_context(session):
    """If the current browsing context selected by WebDriver is a
    `<frame>` or an `<iframe>`, switch it back to the top-level
    browsing context.
    """
    session.switch_frame(None)


def _windows(session, exclude=None):
    """Set of window handles, filtered by an `exclude` list if
    provided.
    """
    if exclude is None:
        exclude = []
    wins = [w for w in session.handles if w not in exclude]
    return set(wins)


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


def create_window(session):
    """Open new window and return the window handle."""
    def create_window():
        windows_before = session.handles
        name = session.execute_script("window.open()")
        assert len(session.handles) == len(windows_before) + 1
        new_windows = list(set(session.handles) - set(windows_before))
        return new_windows.pop()
    return create_window


def http(configuration):
    return HTTPRequest(configuration["host"], configuration["port"])


def server_config():
    return json.loads(os.environ.get("WD_SERVER_CONFIG"))


def configuration():
    host = os.environ.get("WD_HOST", default_host)
    port = int(os.environ.get("WD_PORT", default_port))
    capabilities = json.loads(os.environ.get("WD_CAPABILITIES", "{}"))

    return {
        "host": host,
        "port": port,
        "capabilities": capabilities
    }


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

    yield _current_session

    cleanup_session(_current_session)


def current_session():
    return _current_session


def url(server_config):
    def inner(path, protocol="http", query="", fragment=""):
        port = server_config["ports"][protocol][0]
        host = "%s:%s" % (server_config["browser_host"], port)
        return urlparse.urlunsplit((protocol, host, path, query, fragment))

    inner.__name__ = "url"
    return inner


def create_dialog(session):
    """Create a dialog (one of "alert", "prompt", or "confirm") and provide a
    function to validate that the dialog has been "handled" (either accepted or
    dismissed) by returning some value."""

    def create_dialog(dialog_type, text=None, result_var=None):
        assert dialog_type in ("alert", "confirm", "prompt"), (
            "Invalid dialog type: '%s'" % dialog_type)

        if text is None:
            text = ""

        assert isinstance(text, basestring), "`text` parameter must be a string"

        if result_var is None:
            result_var = "__WEBDRIVER"

        assert re.search(r"^[_$a-z$][_$a-z0-9]*$", result_var, re.IGNORECASE), (
            'The `result_var` must be a valid JavaScript identifier')

        # Script completes itself when the user prompt has been opened.
        session.execute_async_script("""
            setTimeout(function() {{
                window.{0} = window.{1}("{2}");
            }}, 0);
            """.format(result_var, dialog_type, text))

        wait(session,
             lambda s: s.alert.text == text,
             "No user prompt with text '{}' detected".format(text),
             timeout=15,
             ignored_exceptions=webdriver.NoSuchAlertException)

    return create_dialog


def clear_all_cookies(session):
    """Removes all cookies associated with the current active document"""
    session.transport.send("DELETE", "session/%s/cookie" % session.session_id)


def is_element_in_viewport(session, element):
    """Check if element is outside of the viewport"""
    return session.execute_script("""
        let el = arguments[0];

        let rect = el.getBoundingClientRect();
        let viewport = {
          height: window.innerHeight || document.documentElement.clientHeight,
          width: window.innerWidth || document.documentElement.clientWidth,
        };

        return !(rect.right < 0 || rect.bottom < 0 ||
            rect.left > viewport.width || rect.top > viewport.height)
    """, args=(element,))
