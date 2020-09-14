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

frame_doc = inline("<title>cheese</title><p>frame")
one_frame_doc = inline("<title>bar</title><iframe src='%s'></iframe>" % frame_doc)
nested_frames_doc = inline("<title>foo</title><iframe src='%s'></iframe>" % one_frame_doc)


def get_title(session):
    return session.transport.send(
        "GET", "session/{session_id}/title".format(**vars(session)))


def test_no_iframe(session):
    session.url = inline("<title>Foobar</title><h2>Hello</h2>")

    result = get_title(session)
    assert_success(result, "Foobar")


def test_iframe(session):
    session.url = one_frame_doc

    frame = session.find.css("iframe", all=False)
    session.switch_frame(frame)
    session.find.css("p", all=False)

    response = get_title(session)
    assert_success(response, "bar")


def test_nested_iframe(session):
    session.url = nested_frames_doc

    outer_frame = session.find.css("iframe", all=False)
    session.switch_frame(outer_frame)

    inner_frame = session.find.css("iframe", all=False)
    session.switch_frame(inner_frame)
    session.find.css("p", all=False)

    response = get_title(session)
    assert_success(response, "foo")


@pytest.mark.parametrize("domain", ["", "alt"], ids=["same_origin", "cross_origin"])
def test_origin(session, domain, url):
    session.url = inline("<title>foo</title>{}".format(
        iframe("<title>bar</title><p>frame", domain=domain)))

    frame = session.find.css("iframe", all=False)
    session.switch_frame(frame)
    session.find.css("p", all=False)

    response = get_title(session)
    assert_success(response, "foo")
