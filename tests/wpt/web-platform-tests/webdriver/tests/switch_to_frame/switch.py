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


@pytest.mark.parametrize("value", ["foo", True, [], {}])
def test_frame_id_invalid_types(session, value):
    response = switch_to_frame(session, value)
    assert_error(response, "invalid argument")


@pytest.mark.parametrize("value", [-1, 2**16])
def test_frame_id_out_of_bounds(session, value):
    response = switch_to_frame(session, value)
    assert_error(response, "invalid argument")


def test_no_browsing_context(session, closed_window):
    response = switch_to_frame(session, 1)
    assert_error(response, "no such window")


def test_frame_id_null(session):
    session.url = inline(iframe("{}<div>foo".format(iframe("<p>bar"))))

    frame1 = session.find.css("iframe", all=False)
    session.switch_frame(frame1)
    frame1_element = session.find.css("div", all=False)

    frame2 = session.find.css("iframe", all=False)
    session.switch_frame(frame2)
    frame2_element = session.find.css("p", all=False)

    # Switch to top-level browsing context
    response = switch_to_frame(session, None)
    assert_success(response)

    with pytest.raises(StaleElementReferenceException):
        frame2_element.text
    with pytest.raises(StaleElementReferenceException):
        frame1_element.text

    frame = session.find.css("iframe", all=False)
    assert_same_element(session, frame, frame1)


@pytest.mark.parametrize("index, value", [[0, "foo"], [1, "bar"]])
def test_frame_id_number_index(session, index, value):
    session.url = inline("{}{}".format(iframe("<p>foo"), iframe("<p>bar")))

    response = switch_to_frame(session, index)
    assert_success(response)

    frame_element = session.find.css("p", all=False)
    assert frame_element.text == value


def test_frame_id_number_index_out_of_bounds(session):
    session.url = inline(iframe("<p>foo"))

    response = switch_to_frame(session, 1)
    assert_error(response, "no such frame")


@pytest.mark.parametrize("index, value", [[0, "foo"], [1, "bar"]])
def test_frame_id_webelement_frame(session, index, value):
    session.url = inline(frameset("<p>foo", "<p>bar"))
    frames = session.find.css("frame")
    assert len(frames) == 2

    response = switch_to_frame(session, frames[index])
    assert_success(response)

    frame_element = session.find.css("p", all=False)
    assert frame_element.text == value


@pytest.mark.parametrize("index, value", [[0, "foo"], [1, "bar"]])
def test_frame_id_webelement_iframe(session, index, value):
    session.url = inline("{}{}".format(iframe("<p>foo"), iframe("<p>bar")))
    frames = session.find.css("iframe")
    assert len(frames) == 2

    response = switch_to_frame(session, frames[index])
    assert_success(response)

    frame_element = session.find.css("p", all=False)
    assert frame_element.text == value


def test_frame_id_webelement_no_element_reference(session):
    session.url = inline(iframe("<p>foo"))
    frame = session.find.css("iframe", all=False)
    frame.id = "bar"

    response = switch_to_frame(session, frame)
    assert_error(response, "no such element")


def test_frame_id_webelement_stale_reference(session):
    session.url = inline(iframe("<p>foo"))
    frame = session.find.css("iframe", all=False)

    session.switch_frame(frame)

    response = switch_to_frame(session, frame)
    assert_error(response, "stale element reference")


def test_frame_id_webelement_no_frame_element(session):
    session.url = inline("<p>foo")
    no_frame = session.find.css("p", all=False)

    response = switch_to_frame(session, no_frame)
    assert_error(response, "no such frame")
