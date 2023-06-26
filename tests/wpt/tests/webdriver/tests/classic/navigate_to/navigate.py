import time

import pytest
from webdriver import error
from webdriver.transport import Response

from tests.support.asserts import assert_error, assert_success


def navigate_to(session, url):
    return session.transport.send(
        "POST", "session/{session_id}/url".format(**vars(session)),
        {"url": url})


def test_null_parameter_value(session, http):
    path = "/session/{session_id}/url".format(**vars(session))
    with http.post(path, None) as response:
        assert_error(Response.from_http(response), "invalid argument")


def test_null_response_value(session, inline):
    response = navigate_to(session, inline("<div/>"))
    value = assert_success(response)
    assert value is None


def test_no_top_browsing_context(session, closed_window):
    response = navigate_to(session, "foo")
    assert_error(response, "no such window")


def test_no_browsing_context(session, closed_frame, inline):
    doc = inline("<p>foo")

    response = navigate_to(session, doc)
    assert_success(response)

    assert session.url == doc


@pytest.mark.parametrize("protocol,parameters", [
    ("http", ""),
    ("https", ""),
    ("https", {"pipe": "header(Cross-Origin-Opener-Policy,same-origin)"})
], ids=[
    "http",
    "https",
    "https coop"
])
def test_seen_nodes(session, get_test_page, protocol, parameters):
    first_page = get_test_page(parameters=parameters, protocol=protocol)
    second_page = get_test_page(parameters=parameters, protocol=protocol, domain="alt")

    response = navigate_to(session, first_page)
    assert_success(response)

    assert session.url == first_page

    element = session.find.css("#custom-element", all=False)
    shadow_root = element.shadow_root

    response = navigate_to(session, second_page)
    assert_success(response)

    assert session.url == second_page

    with pytest.raises(error.StaleElementReferenceException):
        element.name
    with pytest.raises(error.DetachedShadowRootException):
        shadow_root.find_element("css selector", "in-shadow-dom")

    session.find.css("#custom-element", all=False)


@pytest.mark.capabilities({"pageLoadStrategy": "eager"})
def test_utf8_meta_tag_after_1024_bytes(session, url):
    page = url("/webdriver/tests/support/html/meta-utf8-after-1024-bytes.html")

    # Loading the page will cause a real parse commencing, and a renavigation
    # to the same URL getting triggered subsequently. Test that the navigate
    # command waits long enough.
    response = navigate_to(session, page)
    assert_success(response)

    # If the command returns too early the property will be reset due to the
    # subsequent page load.
    session.execute_script("window.foo = 'bar'")

    # Use delay to allow a possible missing subsequent navigation to start
    time.sleep(1)

    assert session.execute_script("return window.foo") == "bar"
