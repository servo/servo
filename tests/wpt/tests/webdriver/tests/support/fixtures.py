import copy
import json
import os

import pytest
import webdriver

from urllib.parse import urlunsplit

from tests.support.helpers import deep_update
from tests.support.web_extension import EXTENSION_DATA
from tests.support.inline import build_inline
from tests.support.http_request import HTTPRequest
from tests.support.keys import Keys

# The webdriver session can outlive a pytest session
_current_session = None


def get_current_session():
    return _current_session


def set_current_session(session):
    global _current_session
    _current_session = session


def pytest_configure(config):
    # register the capabilities marker
    config.addinivalue_line(
        "markers",
        "capabilities: mark test to use capabilities"
    )


def pytest_sessionfinish():
    # Cleanup at the end of a test run
    if get_current_session() is not None:
        get_current_session().end()
        set_current_session(None)


@pytest.fixture
def default_capabilities():
    """Default capabilities to use for a new WebDriver session."""
    return {}


@pytest.fixture
def capabilities(request, default_capabilities):
    """Merges default capabilities with any test-specific capabilities from a marker."""
    marker = request.node.get_closest_marker("capabilities")
    if marker and marker.args:
        # Ensure the first positional argument is a dictionary
        assert isinstance(
            marker.args[0], dict), "capabilities marker must use a dictionary"
        caps = copy.deepcopy(default_capabilities)
        deep_update(caps, marker.args[0])
        return caps

    return default_capabilities  # Use defaults if no marker is present


@pytest.fixture
def http(configuration):
    return HTTPRequest(configuration["host"], configuration["port"])


@pytest.fixture(scope="session")
def full_configuration():
    """Get test configuration information. Keys are:

    host - WebDriver server host.
    port -  WebDriver server port.
    capabilities - Capabilities passed when creating the WebDriver session
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
    # If there is a session with different requested capabilities active than
    # the one we would like to create, end it now.
    session = get_current_session()
    if session is not None:
        if not session.match(caps):
            is_bidi = isinstance(session, webdriver.BidiSession)
            if is_bidi:
                await session.end()
            else:
                session.end()
            set_current_session(None)


@pytest.fixture(scope="function")
def current_session():
    return get_current_session()


@pytest.fixture
def target_platform(configuration):
    return configuration["target_platform"]


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
def extension_data(current_session):
    browser_name = current_session.capabilities["browserName"]

    return EXTENSION_DATA[browser_name]


@pytest.fixture
def iframe(inline):
    """Inline document extract as the source document of an <iframe>."""
    def iframe(src, **kwargs):
        return "<iframe src='{}'></iframe>".format(inline(src, **kwargs))

    return iframe


@pytest.fixture
def get_actions_origin_page(inline):
    """Create a test page for action origin tests, recording mouse coordinates
    automatically on window.coords."""

    def get_actions_origin_page(inner_style, outer_style=""):
        return inline(
            f"""
          <meta name="viewport" content="width=device-width,initial-scale=1,minimum-scale=1">
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
