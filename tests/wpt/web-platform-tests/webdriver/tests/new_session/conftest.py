import sys

import pytest

from webdriver.transport import HTTPWireProtocol


def product(a, b):
    return [(a, item) for item in b]


def flatten(l):
    return [item for x in l for item in x]


@pytest.fixture(name="add_browser_capabilities")
def fixture_add_browser_capabilities(configuration):

    def add_browser_capabilities(capabilities):
        # Make sure there aren't keys in common.
        assert not set(configuration["capabilities"]).intersection(set(capabilities))
        result = dict(configuration["capabilities"])
        result.update(capabilities)

        return result

    return add_browser_capabilities


@pytest.fixture(name="new_session")
def fixture_new_session(request, configuration, current_session):
    """Start a new session for tests which themselves test creating new sessions.

    :param body: The content of the body for the new session POST request.

    :param delete_existing_session: Allows the fixture to delete an already
     created custom session before the new session is getting created. This
     is useful for tests which call this fixture multiple times within the
     same test.
    """
    custom_session = {}

    transport = HTTPWireProtocol(
        configuration["host"], configuration["port"], url_prefix="/",
    )

    def _delete_session(session_id):
        transport.send("DELETE", "session/{}".format(session_id))

    def new_session(body, delete_existing_session=False):
        # If there is an active session from the global session fixture,
        # delete that one first
        if current_session is not None:
            current_session.end()

        if delete_existing_session:
            _delete_session(custom_session["session"]["sessionId"])

        response = transport.send("POST", "session", body)
        if response.status == 200:
            custom_session["session"] = response.body["value"]
        return response, custom_session.get("session", None)

    yield new_session

    if custom_session.get("session") is not None:
        _delete_session(custom_session["session"]["sessionId"])
        custom_session = None


@pytest.fixture(scope="session")
def platform_name():
    return {
        "linux2": "linux",
        "win32": "windows",
        "cygwin": "windows",
        "darwin": "mac"
    }.get(sys.platform)
