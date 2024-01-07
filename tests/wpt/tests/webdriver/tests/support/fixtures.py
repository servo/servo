import copy
import json
import os

import pytest
import pytest_asyncio
import webdriver

from urllib.parse import urlunsplit

from tests.support import defaults
from tests.support.helpers import cleanup_session, deep_update
from tests.support.inline import build_inline
from tests.support.http_request import HTTPRequest
from tests.support.keys import Keys


SCRIPT_TIMEOUT = 1
PAGE_LOAD_TIMEOUT = 3
IMPLICIT_WAIT_TIMEOUT = 0

# The webdriver session can outlive a pytest session
_current_session = None


def pytest_configure(config):
    # register the capabilities marker
    config.addinivalue_line(
        "markers",
        "capabilities: mark test to use capabilities"
    )


def pytest_sessionfinish():
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


@pytest.fixture
def http(configuration):
    return HTTPRequest(configuration["host"], configuration["port"])


@pytest.fixture(scope="session")
def full_configuration():
    """Get test configuration information. Keys are:

    host - WebDriver server host.
    port -  WebDriver server port.
    capabilites - Capabilites passed when creating the WebDriver session
    timeout_multiplier - Multiplier for timeout values
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


@pytest.fixture()
def screen_size(session):
    """Return the size (width/height) of the screen."""
    return tuple(session.execute_script("""
        return [
            screen.width,
            screen.height,
        ];
        """))


@pytest.fixture()
def available_screen_size(session):
    """Return the effective available screen size (width/height).

    This is size which excludes any fixed window manager elements like menu
    bars, and the dock on MacOS.
    """
    return tuple(session.execute_script("""
        return [
            screen.availWidth,
            screen.availHeight,
        ];
        """))


@pytest.fixture()
def minimal_screen_position(session):
    """Return the minimal position (x/y) a window can be positioned at."""
    return tuple(session.execute_script("""
        return [
            screen.availLeft,
            screen.availTop,
        ];
        """))


@pytest_asyncio.fixture(scope="function")
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
        # Only resize and reposition if needed to workaround a bug for Chrome:
        # https://bugs.chromium.org/p/chromedriver/issues/detail?id=4642#c4
        if _current_session.window.size != defaults.WINDOW_SIZE:
            _current_session.window.size = defaults.WINDOW_SIZE
        if _current_session.window.position != defaults.WINDOW_POSITION:
            _current_session.window.position = defaults.WINDOW_POSITION

    # Set default timeouts
    multiplier = configuration["timeout_multiplier"]
    _current_session.timeouts.implicit = IMPLICIT_WAIT_TIMEOUT * multiplier
    _current_session.timeouts.page_load = PAGE_LOAD_TIMEOUT * multiplier
    _current_session.timeouts.script = SCRIPT_TIMEOUT * multiplier

    yield _current_session

    cleanup_session(_current_session)


@pytest_asyncio.fixture(scope="function")
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
        # Only resize and reposition if needed to workaround a bug for Chrome:
        # https://bugs.chromium.org/p/chromedriver/issues/detail?id=4642#c4
        if _current_session.window.size != defaults.WINDOW_SIZE:
            _current_session.window.size = defaults.WINDOW_SIZE
        if _current_session.window.position != defaults.WINDOW_POSITION:
            _current_session.window.position = defaults.WINDOW_POSITION

    yield _current_session.bidi_session

    await _current_session.bidi_session.end()
    cleanup_session(_current_session)


@pytest.fixture(scope="function")
def current_session():
    return _current_session


@pytest.fixture
def url(server_config):
    def url(path, protocol="https", domain="", subdomain="", query="", fragment=""):
        domain = server_config["domains"][domain][subdomain]
        port = server_config["ports"][protocol][0]
        host = "{0}:{1}".format(domain, port)
        return urlunsplit((protocol, host, path, query, fragment))

    return url


@pytest.fixture
def modifier_key(current_session):
    if current_session.capabilities["platformName"] == "mac":
        return Keys.META
    else:
        return Keys.CONTROL


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


@pytest.fixture
def get_actions_origin_page(inline):
    """Create a test pagefor action origin tests, recording mouse coordinates
    automatically on window.coords."""

    def get_actions_origin_page(inner_style, outer_style=""):
        return inline(
            f"""
          <div id="outer" style="{outer_style}"
               onmousemove="window.coords = {{x: event.clientX, y: event.clientY}}">
            <div id="inner" style="{inner_style}"></div>
          </div>
        """
        )

    return get_actions_origin_page


@pytest.fixture
def get_test_page(iframe, inline):
    def get_test_page(
        as_frame=False,
        frame_doc=None,
        shadow_doc=None,
        nested_shadow_dom=False,
        shadow_root_mode="open",
        **kwargs
    ):
        if frame_doc is None:
            frame_doc = """<div id="in-frame"><input type="checkbox"/></div>"""

        if shadow_doc is None:
            shadow_doc = """<div id="in-shadow-dom"><input type="checkbox"/></div>"""

        definition_inner_shadow_dom = ""
        if nested_shadow_dom:
            definition_inner_shadow_dom = f"""
                customElements.define('inner-custom-element',
                    class extends HTMLElement {{
                        constructor() {{
                            super();
                            this.attachShadow({{mode: "{shadow_root_mode}"}}).innerHTML = `
                                {shadow_doc}
                            `;
                        }}
                    }}
                );
            """
            shadow_doc = """
                <style>
                    inner-custom-element {
                        display:block; width:20px; height:20px;
                    }
                </style>
                <div id="in-nested-shadow-dom">
                    <inner-custom-element></inner-custom-element>
                </div>
                """

        page_data = f"""
            <style>
                custom-element {{
                    display:block; width:20px; height:20px;
                }}
            </style>
            <div id="with-children"><p><span></span></p><br/></div>
            <div id="with-text-node">Lorem</div>
            <div id="with-comment"><!-- Comment --></div>

            <input id="button" type="button"/>
            <input id="checkbox" type="checkbox"/>
            <input id="file" type="file"/>
            <input id="hidden" type="hidden"/>
            <input id="text" type="text"/>

            {iframe(frame_doc, **kwargs)}

            <img />
            <svg></svg>

            <custom-element id="custom-element"></custom-element>
            <script>
                var svg = document.querySelector("svg");
                svg.setAttributeNS("http://www.w3.org/2000/svg", "svg:foo", "bar");

                customElements.define("custom-element",
                    class extends HTMLElement {{
                        constructor() {{
                            super();
                            const shadowRoot = this.attachShadow({{mode: "{shadow_root_mode}"}});
                            shadowRoot.innerHTML = `{shadow_doc}`;

                            // Save shadow root on window to access it in case of `closed` mode.
                            window._shadowRoot = shadowRoot;
                        }}
                    }}
                );
                {definition_inner_shadow_dom}
            </script>"""

        if as_frame:
            iframe_data = iframe(page_data, **kwargs)
            return inline(iframe_data, **kwargs)
        else:
            return inline(page_data, **kwargs)

    return get_test_page


@pytest.fixture
def test_origin(url):
    return url("")


@pytest.fixture
def test_alt_origin(url):
    return url("", domain="alt")


@pytest.fixture
def test_page(inline):
    return inline("<div>foo</div>")


@pytest.fixture
def test_page2(inline):
    return inline("<div>bar</div>")


@pytest.fixture
def test_page_cross_origin(inline):
    return inline("<div>bar</div>", domain="alt")


@pytest.fixture
def test_page_multiple_frames(inline, test_page, test_page2):
    return inline(
        f"<iframe src='{test_page}'></iframe><iframe src='{test_page2}'></iframe>"
    )


@pytest.fixture
def test_page_nested_frames(inline, test_page_same_origin_frame):
    return inline(f"<iframe src='{test_page_same_origin_frame}'></iframe>")


@pytest.fixture
def test_page_cross_origin_frame(inline, test_page_cross_origin):
    return inline(f"<iframe src='{test_page_cross_origin}'></iframe>")


@pytest.fixture
def test_page_same_origin_frame(inline, test_page):
    return inline(f"<iframe src='{test_page}'></iframe>")


@pytest.fixture
def test_page_with_pdf_js(inline):
    """Prepare an url to load a PDF document in the browser using pdf.js"""
    def test_page_with_pdf_js(encoded_pdf_data):
        return inline("""
<!doctype html>
<script src="/_pdf_js/pdf.js"></script>
<canvas></canvas>
<script>
async function getText() {
  const pages = [];
  const loadingTask = pdfjsLib.getDocument({data: atob("%s")});
  const pdf = await loadingTask.promise;
  for (let pageNumber = 1; pageNumber <= pdf.numPages; pageNumber++) {
    const page = await pdf.getPage(pageNumber);
    const textContent = await page.getTextContent();
    const text = textContent.items.map(x => x.str).join("");
    pages.push(text);
  }
  return pages;
}
</script>
""" % encoded_pdf_data)

    return test_page_with_pdf_js


@pytest_asyncio.fixture
async def top_context(bidi_session):
    contexts = await bidi_session.browsing_context.get_tree()
    return contexts[0]
