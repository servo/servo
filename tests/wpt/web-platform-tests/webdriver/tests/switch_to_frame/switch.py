import pytest

import webdriver.protocol as protocol

from webdriver import StaleElementReferenceException
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


@pytest.mark.parametrize("id", [
    None,
    0,
    {"element-6066-11e4-a52e-4f735466cecf": "foo"},
])
def test_no_top_browsing_context(session, closed_window, id):
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


@pytest.mark.parametrize("value", ["foo", True, [], {}])
def test_frame_id_invalid_types(session, value):
    response = switch_to_frame(session, value)
    assert_error(response, "invalid argument")


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
