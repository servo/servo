import webdriver.protocol as protocol
from webdriver.transport import Response

from tests.support.asserts import assert_error, assert_success
from tests.support.inline import inline, iframe


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


def test_null_response_value(session):
    session.url = inline(iframe("<p>foo"))
    frame_element = session.find.css("iframe", all=False)

    response = switch_to_frame(session, frame_element)
    value = assert_success(response)
    assert value is None


def test_no_browsing_context(session, closed_window):
    response = switch_to_frame(session, 1)
    assert_error(response, "no such window")
