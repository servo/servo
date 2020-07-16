import pytest

import webdriver.protocol as protocol

from tests.support.asserts import assert_error, assert_success
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


@pytest.mark.parametrize("index, value", [[0, "foo"], [1, "bar"]])
def test_frame_id_webelement_frame(session, index, value):
    session.url = inline(frameset("<p>foo", "<p>bar"))
    frames = session.find.css("frame")
    assert len(frames) == 2

    response = switch_to_frame(session, frames[index])
    assert_success(response)

    element = session.find.css("p", all=False)
    assert element.text == value


@pytest.mark.parametrize("index, value", [[0, "foo"], [1, "bar"]])
def test_frame_id_webelement_iframe(session, index, value):
    session.url = inline("{}{}".format(iframe("<p>foo"), iframe("<p>bar")))
    frames = session.find.css("iframe")
    assert len(frames) == 2

    response = switch_to_frame(session, frames[index])
    assert_success(response)

    element = session.find.css("p", all=False)
    assert element.text == value


def test_frame_id_webelement_nested(session):
    session.url = inline(iframe("{}<p>foo".format(iframe("<p>bar"))))

    expected_text = ["foo", "bar"]
    for i in range(0, len(expected_text)):
        frame_element = session.find.css("iframe", all=False)
        response = switch_to_frame(session, frame_element)
        assert_success(response)

        element = session.find.css("p", all=False)
        assert element.text == expected_text[i]


def test_frame_id_webelement_no_element_reference(session):
    session.url = inline(iframe("<p>foo"))
    frame = session.find.css("iframe", all=False)
    frame.id = "bar"

    response = switch_to_frame(session, frame)
    assert_error(response, "no such element")


def test_frame_id_webelement_stale_reference(session):
    session.url = inline(iframe("<p>foo"))
    frame = session.find.css("iframe", all=False)

    session.refresh()

    response = switch_to_frame(session, frame)
    assert_error(response, "stale element reference")


def test_frame_id_webelement_no_frame_element(session):
    session.url = inline("<p>foo")
    no_frame = session.find.css("p", all=False)

    response = switch_to_frame(session, no_frame)
    assert_error(response, "no such frame")
