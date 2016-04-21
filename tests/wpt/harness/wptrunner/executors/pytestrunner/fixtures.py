# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this file,
# You can obtain one at http://mozilla.org/MPL/2.0/.

import pytest


"""pytest fixtures for use in Python-based WPT tests.

The purpose of test fixtures is to provide a fixed baseline upon which
tests can reliably and repeatedly execute.
"""


class Session(object):
    """Fixture to allow access to wptrunner's existing WebDriver session
    in tests.

    The session is not created by default to enable testing of session
    creation.  However, a module-scoped session will be implicitly created
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

    The session is closed when the test module goes out of scope by an
    implicit call to `session.end`.
    """

    def __init__(self, client):
        self.client = client

    @pytest.fixture(scope="module")
    def session(self, request):
        request.addfinalizer(self.client.end)
        return self.client
