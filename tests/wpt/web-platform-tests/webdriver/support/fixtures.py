import json
import os
import urlparse

import webdriver

from support.http_request import HTTPRequest
from support import merge_dictionaries

default_host = "http://127.0.0.1"
default_port = "4444"

def _ensure_valid_window(session):
    """If current window is not open anymore, ensure to have a valid one selected."""
    try:
        session.window_handle
    except webdriver.NoSuchWindowException:
        session.window_handle = session.handles[0]


def _dismiss_user_prompts(session):
    """Dismisses any open user prompts in windows."""
    current_window = session.window_handle

    for window in _windows(session):
        session.window_handle = window
        try:
            session.alert.dismiss()
        except webdriver.NoSuchAlertException:
            pass

    session.window_handle = current_window

def _restore_windows(session):
    """Closes superfluous windows opened by the test without ending
    the session implicitly by closing the last window.
    """
    current_window = session.window_handle

    for window in _windows(session, exclude=[current_window]):
        session.window_handle = window
        if len(session.window_handles) > 1:
            session.close()

    session.window_handle = current_window

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

def create_frame(session):
    """Create an `iframe` element in the current browsing context and insert it
    into the document. Return an element reference."""
    def create_frame():
        append = """
            var frame = document.createElement('iframe');
            document.body.appendChild(frame);
            return frame;
        """
        response = session.execute_script(append)

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

def http(session):
    return HTTPRequest(session.transport.host, session.transport.port)

def server_config():
    return json.loads(os.environ.get("WD_SERVER_CONFIG"))

def create_session(request):
    """Provide a factory function that produces wdclient `Session` instances.
    If the `WD_CAPABILITIES` environment variable is set, it will be parsed as
    JSON and the resulting object will be included in the WebDriver "Create
    Session" command. Additional capabilities may be specified as an optional
    argument to this function, but the operation will fail if any values
    conflict with those specified via the environment. If the session is still
    active at the completion of the test, it will be destroyed
    automatically."""

    def create_session(test_capabilities=None):
        host = os.environ.get("WD_HOST", default_host)
        port = int(os.environ.get("WD_PORT", default_port))
        if test_capabilities is None:
            test_capabilities = {}
        env_capabilities = json.loads(os.environ.get("WD_CAPABILITIES", "{}"))

        capabilities = merge_dictionaries(env_capabilities, test_capabilities)
        session = webdriver.Session(host, port, capabilities=capabilities)

        def destroy():
            if session.session_id is not None:
                session.end()

        # finalisers are popped off a stack, making their ordering reverse
        request.addfinalizer(destroy)
        request.addfinalizer(lambda: _switch_to_top_level_browsing_context(session))
        request.addfinalizer(lambda: _restore_windows(session))
        request.addfinalizer(lambda: _dismiss_user_prompts(session))
        request.addfinalizer(lambda: _ensure_valid_window(session))

        return session

    return create_session

# Create a wdclient `Session` object for each Pytest "session". If the
# `WD_CAPABILITIES` environment variable is set, it will be parsed as JSON and
# the resulting object will be included in the WebDriver "Create Session"
# command. If the session is still active at the completion of the test, it
# will be destroyed automatically.
def session(create_session):
    return create_session()

def url(server_config):
    def inner(path, query="", fragment=""):
        rv = urlparse.urlunsplit(("http",
                                  "%s:%s" % (server_config["host"],
                                             server_config["ports"]["http"][0]),
                                  path,
                                  query,
                                  fragment))
        return rv
    return inner
