import pytest

from tests.support.asserts import assert_success
from tests.support.inline import iframe, inline


"""
Tests that WebDriver can transcend site origins.

Many modern browsers impose strict cross-origin checks,
and WebDriver should be able to transcend these.

Although an implementation detail, certain browsers
also enforce process isolation based on site origin.
This is known to sometimes cause problems for WebDriver implementations.
"""

frame_doc = inline("<p>frame")
one_frame_doc = inline("<iframe src='%s'></iframe>" % frame_doc)
nested_frames_doc = inline("<iframe src='%s'></iframe>" % one_frame_doc)


def get_current_url(session):
    return session.transport.send(
        "GET", "session/{session_id}/url".format(**vars(session)))


def test_iframe(session):
    top_level_doc = one_frame_doc
    session.url = top_level_doc

    frame = session.find.css("iframe", all=False)
    session.switch_frame(frame)
    session.find.css("p", all=False)

    response = get_current_url(session)
    assert_success(response, top_level_doc)


def test_nested_iframe(session):
    session.url = nested_frames_doc
    top_level_doc = session.url

    outer_frame = session.find.css("iframe", all=False)
    session.switch_frame(outer_frame)

    inner_frame = session.find.css("iframe", all=False)
    session.switch_frame(inner_frame)
    session.find.css("p", all=False)

    response = get_current_url(session)
    assert_success(response, top_level_doc)


@pytest.mark.parametrize("domain", ["", "alt"], ids=["same_origin", "cross_origin"])
def test_origin(session, domain, url):
    top_level_doc = inline(iframe("<p>frame", domain=domain))

    session.url = top_level_doc
    frame = session.find.css("iframe", all=False)
    session.switch_frame(frame)
    session.find.css("p", all=False)

    response = get_current_url(session)
    assert_success(response, top_level_doc)
