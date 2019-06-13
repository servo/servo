import webdriver.protocol as protocol

from tests.support.asserts import assert_success
from tests.support.helpers import document_location
from tests.support.inline import (
    iframe,
    inline,
)


"""
Tests that WebDriver can transcend site origins.

Many modern browsers impose strict cross-origin checks,
and WebDriver should be able to transcend these.

Although an implementation detail, certain browsers
also enforce process isolation based on site origin.
This is known to sometimes cause problems for WebDriver implementations.
"""


def switch_to_frame(session, frame):
    return session.transport.send(
        "POST", "/session/{session_id}/frame".format(**vars(session)),
        {"id": frame},
        encoder=protocol.Encoder, decoder=protocol.Decoder,
        session=session)


def test_cross_origin_iframe(session, server_config):
    session.url = inline(iframe("", subdomain="www"))
    frame_element = session.find.css("iframe", all=False)

    response = switch_to_frame(session, frame_element)
    value = assert_success(response)
    assert document_location(session).startswith(
        "http://www.{}".format(server_config["browser_host"]))


def test_nested_cross_origin_iframe(session, server_config):
    frame2 = iframe("", subdomain="www.www")
    frame1 = iframe(frame2, subdomain="www")
    top_doc = inline(frame1, subdomain="")

    session.url = top_doc
    assert document_location(session).startswith(
        "http://{}".format(server_config["browser_host"]))

    frame1_el = session.find.css("iframe", all=False)
    response = switch_to_frame(session, frame1_el)
    value = assert_success(response)
    assert document_location(session).startswith(
        "http://www.{}".format(server_config["browser_host"]))

    frame2_el = session.find.css("iframe", all=False)
    response = switch_to_frame(session, frame2_el)
    value = assert_success(response)
    assert document_location(session).startswith(
        "http://www.www.{}".format(server_config["browser_host"]))
