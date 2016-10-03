# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this file,
# You can obtain one at http://mozilla.org/MPL/2.0/.

import pytest
import webdriver

import contextlib
import httplib


"""pytest fixtures for use in Python-based WPT tests.

The purpose of test fixtures is to provide a fixed baseline upon which
tests can reliably and repeatedly execute.
"""


class Session(object):
    """Fixture to allow access to wptrunner's existing WebDriver session
    in tests.

    The session is not created by default to enable testing of session
    creation.  However, a function-scoped session will be implicitly created
    at the first call to a WebDriver command.  This means methods such as
    `session.send_command` and `session.session_id` are possible to use
    without having a session.

    To illustrate implicit session creation::

        def test_session_scope(session):
            # at this point there is no session
            assert session.session_id is None

            # window_id is a WebDriver command,
            # and implicitly creates the session for us
            assert session.window_id is not None

            # we now have a session
            assert session.session_id is not None

    You can also access the session in custom fixtures defined in the
    tests, such as a setup function::

        @pytest.fixture(scope="function")
        def setup(request, session):
            session.url = "https://example.org"

        def test_something(setup, session):
            assert session.url == "https://example.org"

    When the test function goes out of scope, any remaining user prompts
    and opened windows are closed, and the current browsing context is
    switched back to the top-level browsing context.
    """

    def __init__(self, client):
        self.client = client

    @pytest.fixture(scope="function")
    def session(self, request):
        # finalisers are popped off a stack,
        # making their ordering reverse
        request.addfinalizer(self.switch_to_top_level_browsing_context)
        request.addfinalizer(self.restore_windows)
        request.addfinalizer(self.dismiss_user_prompts)

        return self.client

    def dismiss_user_prompts(self):
        """Dismisses any open user prompts in windows."""
        current_window = self.client.window_handle

        for window in self.windows():
            self.client.window_handle = window
            try:
                self.client.alert.dismiss()
            except webdriver.NoSuchAlertException:
                pass

        self.client.window_handle = current_window

    def restore_windows(self):
        """Closes superfluous windows opened by the test without ending
        the session implicitly by closing the last window.
        """
        current_window = self.client.window_handle

        for window in self.windows(exclude=[current_window]):
            self.client.window_handle = window
            if len(self.client.window_handles) > 1:
                self.client.close()

        self.client.window_handle = current_window

    def switch_to_top_level_browsing_context(self):
        """If the current browsing context selected by WebDriver is a
        `<frame>` or an `<iframe>`, switch it back to the top-level
        browsing context.
        """
        self.client.switch_frame(None)

    def windows(self, exclude=None):
        """Set of window handles, filtered by an `exclude` list if
        provided.
        """
        if exclude is None:
            exclude = []
        wins = [w for w in self.client.handles if w not in exclude]
        return set(wins)


class HTTPRequest(object):
    def __init__(self, host, port):
        self.host = host
        self.port = port

    def head(self, path):
        return self._request("HEAD", path)

    def get(self, path):
        return self._request("GET", path)

    @contextlib.contextmanager
    def _request(self, method, path):
        conn = httplib.HTTPConnection(self.host, self.port)
        try:
            conn.request(method, path)
            yield conn.getresponse()
        finally:
            conn.close()


@pytest.fixture(scope="module")
def http(session):
    return HTTPRequest(session.transport.host, session.transport.port)
