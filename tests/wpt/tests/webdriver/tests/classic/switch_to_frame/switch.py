import pytest

import webdriver.protocol as protocol

from webdriver import NoSuchElementException
from webdriver.transport import Response

from tests.support.asserts import assert_error, assert_same_element, assert_success


def switch_to_frame(session, frame):
    return session.transport.send(
        "POST", "session/{session_id}/frame".format(**vars(session)),
        {"id": frame},
        encoder=protocol.Encoder, decoder=protocol.Decoder,
        session=session)


def test_null_parameter_value(session, http):
    path = "/session/{session_id}/frame".format(**vars(session))
    with http.post(path, None) as response:
        assert_error(Response.from_http(response), "invalid argument")


def test_null_response_value(session, inline, iframe):
    session.url = inline(iframe("<p>foo"))
    frame = session.find.css("iframe", all=False)

    response = switch_to_frame(session, frame)
    value = assert_success(response)
    assert value is None


@pytest.mark.parametrize("id", [
    None,
    0,
    {"element-6066-11e4-a52e-4f735466cecf": "foo"},
])
def test_no_top_browsing_context(session, url, id):
    session.window_handle = session.new_window()

    session.url = url("/webdriver/tests/support/html/frames.html")

    subframe = session.find.css("#sub-frame", all=False)
    session.switch_frame(subframe)

    session.window.close()

    response = switch_to_frame(session, id)
    assert_error(response, "no such window")


@pytest.mark.parametrize("id", [
    None,
    0,
    {"element-6066-11e4-a52e-4f735466cecf": "foo"},
])
def test_no_browsing_context(session, closed_frame, id):
    response = switch_to_frame(session, id)
    if id is None:
        assert_success(response)
        session.find.css("#delete", all=False)
    else:
        assert_error(response, "no such window")


def test_no_browsing_context_when_already_top_level(session, closed_window):
    response = switch_to_frame(session, None)
    assert_error(response, "no such window")


@pytest.mark.parametrize("value", ["foo", True, [], {}])
def test_frame_id_invalid_types(session, value):
    response = switch_to_frame(session, value)
    assert_error(response, "invalid argument")


def test_frame_id_shadow_root(session, get_test_page):
    session.url = get_test_page()

    element = session.find.css("custom-element", all=False)

    result = switch_to_frame(session, element.shadow_root)
    assert_error(result, "invalid argument")


def test_frame_id_null(session, inline, iframe):
    session.url = inline(iframe("{}<div>foo".format(iframe("<p>bar"))))

    frame1 = session.find.css("iframe", all=False)
    session.switch_frame(frame1)
    element1 = session.find.css("div", all=False)

    frame2 = session.find.css("iframe", all=False)
    session.switch_frame(frame2)
    element2 = session.find.css("p", all=False)

    # Switch to top-level browsing context
    response = switch_to_frame(session, None)
    assert_success(response)

    with pytest.raises(NoSuchElementException):
        element2.text
    with pytest.raises(NoSuchElementException):
        element1.text

    frame = session.find.css("iframe", all=False)
    assert_same_element(session, frame, frame1)


def test_find_element_while_frame_is_still_loading(session, url):
    session.timeouts.implicit = 5

    frame_url = url("/webdriver/tests/support/html/subframe.html?pipe=trickle(d2)")
    page_url = "<html><body><iframe src='{}'></iframe></body></html>".format(frame_url)

    session.execute_script(
        "document.documentElement.innerHTML = arguments[0];", args=[page_url])

    frame1 = session.find.css("iframe", all=False)
    session.switch_frame(frame1)

    # Ensure that the is always a valid browsing context, and the element
    # can be found eventually.
    session.find.css("#delete", all=False)
