import pytest

import webdriver.protocol as protocol

from webdriver import NoSuchElementException, StaleElementReferenceException
from webdriver.transport import Response

from tests.support.asserts import assert_error, assert_same_element, assert_success
from tests.support.inline import inline, iframe


def switch_to_frame(session, frame):
    return session.transport.send(
        "POST", "session/{session_id}/frame".format(**vars(session)),
        {"id": frame},
        encoder=protocol.Encoder, decoder=protocol.Decoder,
        session=session)


def frameset(*docs):
    frames = list(map(lambda doc: "<frame src='{}'></frame>".format(inline(doc)), docs))
    return "<frameset rows='{}'>\n{}</frameset>".format(len(frames) * "*,", "\n".join(frames))


def test_null_parameter_value(session, http):
    path = "/session/{session_id}/frame".format(**vars(session))
    with http.post(path, None) as response:
        assert_error(Response.from_http(response), "invalid argument")


def test_null_response_value(session):
    session.url = inline(iframe("<p>foo"))
    frame = session.find.css("iframe", all=False)

    response = switch_to_frame(session, frame)
    value = assert_success(response)
    assert value is None


@pytest.mark.parametrize("value", ["foo", True, [], {}])
def test_frame_id_invalid_types(session, value):
    response = switch_to_frame(session, value)
    assert_error(response, "invalid argument")


def test_no_browsing_context(session, closed_window):
    response = switch_to_frame(session, 1)
    assert_error(response, "no such window")


def test_frame_id_null(session):
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

    with pytest.raises(StaleElementReferenceException):
        element2.text
    with pytest.raises(StaleElementReferenceException):
        element1.text

    frame = session.find.css("iframe", all=False)
    assert_same_element(session, frame, frame1)


def test_frame_deleted(session, url):
    session.url = url("/webdriver/tests/support/html/frames.html")
    frame = session.find.css("iframe", all=False)

    response = switch_to_frame(session, frame)
    assert_success(response)

    input = session.find.css("input", all=False)
    input.click()

    response = switch_to_frame(session, None)
    assert_success(response)

    with pytest.raises(NoSuchElementException):
        session.find.css("iframe", all=False)

    session.find.css("#delete", all=False)
