from urlparse import urlparse

import webdriver.protocol as protocol

from tests.support.asserts import assert_success
from tests.support.helpers import document_location
from tests.support.inline import iframe, inline


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
    session.url = inline(iframe("", domain="alt"))
    frame_element = session.find.css("iframe", all=False)

    response = switch_to_frame(session, frame_element)
    assert_success(response)

    parse_result = urlparse(document_location(session))
    assert parse_result.netloc != server_config["browser_host"]


def test_nested_cross_origin_iframe(session, server_config):
    frame2 = iframe("", domain="alt", subdomain="www")
    frame1 = iframe(frame2)
    top_doc = inline(frame1, domain="alt")

    session.url = top_doc

    parse_result = urlparse(document_location(session))
    top_level_host = parse_result.netloc
    assert not top_level_host.startswith(server_config["browser_host"])

    frame1_element = session.find.css("iframe", all=False)
    response = switch_to_frame(session, frame1_element)
    assert_success(response)

    parse_result = urlparse(document_location(session))
    assert parse_result.netloc.startswith(server_config["browser_host"])

    frame2_el = session.find.css("iframe", all=False)
    response = switch_to_frame(session, frame2_el)
    assert_success(response)

    parse_result = urlparse(document_location(session))
    assert parse_result.netloc == "www.{}".format(top_level_host)
