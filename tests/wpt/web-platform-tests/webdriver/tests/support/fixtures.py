import copy
import json
import os

import asyncio
import pytest
import webdriver

from urllib.parse import urlunsplit

from tests.support import defaults
from tests.support.helpers import cleanup_session, deep_update
from tests.support.inline import build_inline
from tests.support.http_request import HTTPRequest


# The webdriver session can outlive a pytest session
_current_session = None

# The event loop needs to outlive the webdriver session
_event_loop = None

_custom_session = False


def pytest_configure(config):
    # register the capabilities marker
    config.addinivalue_line(
        "markers",
        "capabilities: mark test to use capabilities"
    )


def pytest_sessionfinish(session, exitstatus):
    # Cleanup at the end of a test run
    global _current_session

    if _current_session is not None:
        _current_session.end()
        _current_session = None


@pytest.fixture
def capabilities():
    """Default capabilities to use for a new WebDriver session."""
    return {}


def pytest_generate_tests(metafunc):
    if "capabilities" in metafunc.fixturenames:
        marker = metafunc.definition.get_closest_marker(name="capabilities")
        if marker:
            metafunc.parametrize("capabilities", marker.args, ids=None)


@pytest.fixture(scope="session")
def event_loop():
    """Change event_loop fixture to global."""
    global _event_loop

    if _event_loop is None:
        _event_loop = asyncio.get_event_loop_policy().new_event_loop()
    return _event_loop


@pytest.fixture
def http(configuration):
    return HTTPRequest(configuration["host"], configuration["port"])


@pytest.fixture(scope="session")
def full_configuration():
    """Get test configuration information. Keys are:

    host - WebDriver server host.
    port -  WebDriver server port.
    capabilites - Capabilites passed when creating the WebDriver session
    webdriver - Dict with keys `binary`: path to webdriver binary, and
                `args`: Additional command line arguments passed to the webdriver
                binary. This doesn't include all the required arguments e.g. the
                port.
    wptserve - Configuration of the wptserve servers."""

    with open(os.environ.get("WDSPEC_CONFIG_FILE"), "r") as f:
        return json.load(f)


@pytest.fixture(scope="session")
def server_config(full_configuration):
    return full_configuration["wptserve"]


@pytest.fixture(scope="session")
def configuration(full_configuration):
    """Configuation minus server config.

    This makes logging easier to read."""

    config = full_configuration.copy()
    del config["wptserve"]

    return config


async def reset_current_session_if_necessary(caps):
    global _current_session

    # If there is a session with different requested capabilities active than
    # the one we would like to create, end it now.
    if _current_session is not None:
        if not _current_session.match(caps):
            is_bidi = isinstance(_current_session, webdriver.BidiSession)
            if is_bidi:
                await _current_session.end()
            else:
                _current_session.end()
            _current_session = None


@pytest.fixture(scope="function")
async def session(capabilities, configuration):
    """Create and start a session for a test that does not itself test session creation.

    By default the session will stay open after each test, but we always try to start a
    new one and assume that if that fails there is already a valid session. This makes it
    possible to recover from some errors that might leave the session in a bad state, but
    does not demand that we start a new session per test.
    """
    global _current_session

    # Update configuration capabilities with custom ones from the
    # capabilities fixture, which can be set by tests
    caps = copy.deepcopy(configuration["capabilities"])
    deep_update(caps, capabilities)
    caps = {"alwaysMatch": caps}

    await reset_current_session_if_necessary(caps)

    if _current_session is None:
        _current_session = webdriver.Session(
            configuration["host"],
            configuration["port"],
            capabilities=caps)

    _current_session.start()

    # Enforce a fixed default window size and position
    if _current_session.capabilities.get("setWindowRect"):
        _current_session.window.size = defaults.WINDOW_SIZE
        _current_session.window.position = defaults.WINDOW_POSITION

    yield _current_session

    cleanup_session(_current_session)


@pytest.fixture(scope="function")
async def bidi_session(capabilities, configuration):
    """Create and start a bidi session.

    Can be used for a test that does not itself test bidi session creation.

    By default the session will stay open after each test, but we always try to start a
    new one and assume that if that fails there is already a valid session. This makes it
    possible to recover from some errors that might leave the session in a bad state, but
    does not demand that we start a new session per test.
    """
    global _current_session

    # Update configuration capabilities with custom ones from the
    # capabilities fixture, which can be set by tests
    caps = copy.deepcopy(configuration["capabilities"])
    caps.update({"webSocketUrl": True})
    deep_update(caps, capabilities)
    caps = {"alwaysMatch": caps}

    await reset_current_session_if_necessary(caps)

    if _current_session is None:
        _current_session = webdriver.Session(
            configuration["host"],
            configuration["port"],
            capabilities=caps,
            enable_bidi=True)

    _current_session.start()
    await _current_session.bidi_session.start()

    # Enforce a fixed default window size and position
    if _current_session.capabilities.get("setWindowRect"):
        _current_session.window.size = defaults.WINDOW_SIZE
        _current_session.window.position = defaults.WINDOW_POSITION

    yield _current_session.bidi_session

    await _current_session.bidi_session.end()
    cleanup_session(_current_session)


@pytest.fixture(scope="function")
def current_session():
    return _current_session


@pytest.fixture
def url(server_config):
    def url(path, protocol="http", domain="", subdomain="", query="", fragment=""):
        domain = server_config["domains"][domain][subdomain]
        port = server_config["ports"][protocol][0]
        host = "{0}:{1}".format(domain, port)
        return urlunsplit((protocol, host, path, query, fragment))

    return url


@pytest.fixture
def inline(url):
    """Take a source extract and produces well-formed documents.

    Based on the desired document type, the extract is embedded with
    predefined boilerplate in order to produce well-formed documents.
    The media type and character set may also be individually configured.

    This helper function originally used data URLs, but since these
    are not universally supported (or indeed standardised!) across
    browsers, it now delegates the serving of the document to wptserve.
    This file also acts as a wptserve handler (see the main function
    below) which configures the HTTP response using query parameters.

    This function returns a URL to the wptserve handler, which in turn
    will serve an HTTP response with the requested source extract
    inlined in a well-formed document, and the Content-Type header
    optionally configured using the desired media type and character set.

    Any additional keyword arguments are passed on to the build_url
    function, which comes from the url fixture.
    """
    def inline(src, **kwargs):
        return build_inline(url, src, **kwargs)

    return inline


@pytest.fixture
def iframe(inline):
    """Inline document extract as the source document of an <iframe>."""
    def iframe(src, **kwargs):
        return "<iframe src='{}'></iframe>".format(inline(src, **kwargs))

    return iframe
