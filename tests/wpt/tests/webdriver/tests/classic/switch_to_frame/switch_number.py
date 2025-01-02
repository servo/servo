import pytest

import webdriver.protocol as protocol

from tests.support.asserts import assert_error, assert_success


def switch_to_frame(session, frame):
    return session.transport.send(
        "POST", "session/{session_id}/frame".format(**vars(session)),
        {"id": frame},
        encoder=protocol.Encoder, decoder=protocol.Decoder,
        session=session)


@pytest.mark.parametrize("value", [-1, 2**16])
def test_frame_id_number_out_of_bounds(session, value):
    response = switch_to_frame(session, value)
    assert_error(response, "invalid argument")


@pytest.mark.parametrize("index", [1, 65535])
def test_frame_id_number_index_out_of_bounds(session, inline, iframe, index):
    session.url = inline(iframe("<p>foo"))

    response = switch_to_frame(session, index)
    assert_error(response, "no such frame")


@pytest.mark.parametrize("index, value", [[0, "foo"], [1, "bar"]])
def test_frame_id_number_index(session, inline, iframe, index, value):
    session.url = inline("{}{}".format(iframe("<p>foo"), iframe("<p>bar")))

    response = switch_to_frame(session, index)
    assert_success(response)

    element = session.find.css("p", all=False)
    assert element.text == value


def test_frame_id_number_index_nested(session, inline, iframe):
    session.url = inline(iframe("{}<p>foo".format(iframe("<p>bar"))))

    expected_text = ["foo", "bar"]
    for i in range(0, len(expected_text)):
        response = switch_to_frame(session, 0)
        assert_success(response)

        element = session.find.css("p", all=False)
        assert element.text == expected_text[i]
