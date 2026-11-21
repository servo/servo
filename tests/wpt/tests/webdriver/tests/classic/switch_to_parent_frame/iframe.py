import pytest

from urllib.parse import urlparse
from tests.support.classic.asserts import assert_success
from tests.support.classic.helpers import document_location


def switch_to_parent_frame(session):
    return session.transport.send(
        "POST", "session/{session_id}/frame/parent".format(**vars(session)))


@pytest.mark.parametrize("domain", ["", "alt"], ids=["same_origin", "cross_origin"])
def test_switch_to_parent_from_frame(session, server_config, inline, iframe, domain):
    session.url = inline(iframe("<p id=child>foo</p>", domain=domain))

    frame = session.find.css("iframe", all=False)
    session.switch_to_frame(frame)

    response = switch_to_parent_frame(session)
    assert_success(response)

    # Back in the top document: it is on the browser host, and the child-only
    # element is no longer reachable from here.
    assert urlparse(document_location(session)).hostname == server_config["browser_host"]
    assert session.find.css("iframe", all=False)


def test_switch_to_parent_from_nested_cross_origin_frame(session, server_config, inline, iframe):
    frame2 = iframe("<p id=leaf>foo</p>", domain="alt", subdomain="www")
    frame1 = iframe(frame2, domain="alt")
    session.url = inline(frame1)

    top_level_host = urlparse(document_location(session)).hostname
    assert top_level_host == server_config["browser_host"]

    # Switch down: top -> alt -> www.alt.
    frame1_element = session.find.css("iframe", all=False)
    session.switch_to_frame(frame1_element)
    alt_host = urlparse(document_location(session)).hostname
    assert alt_host != server_config["browser_host"]

    frame2_element = session.find.css("iframe", all=False)
    session.switch_to_frame(frame2_element)
    assert urlparse(document_location(session)).hostname == "www.{}".format(alt_host)

    # Switch back up one level: www.alt -> alt.
    response = switch_to_parent_frame(session)
    assert_success(response)
    assert urlparse(document_location(session)).hostname == alt_host

    # And back up to the top document on the browser host.
    response = switch_to_parent_frame(session)
    assert_success(response)
    assert urlparse(document_location(session)).hostname == server_config["browser_host"]
