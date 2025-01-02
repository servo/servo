import pytest

from tests.support.asserts import assert_success


"""
Tests that WebDriver can transcend site origins.

Many modern browsers impose strict cross-origin checks,
and WebDriver should be able to transcend these.

Although an implementation detail, certain browsers
also enforce process isolation based on site origin.
This is known to sometimes cause problems for WebDriver implementations.
"""


@pytest.fixture
def frame_doc(inline):
    return inline("<title>cheese</title><p>frame")


@pytest.fixture
def one_frame_doc(inline, frame_doc):
    return inline("<title>bar</title><iframe src='%s'></iframe>" % frame_doc)


@pytest.fixture
def nested_frames_doc(inline, one_frame_doc):
    return inline("<title>foo</title><iframe src='%s'></iframe>" % one_frame_doc)


def get_title(session):
    return session.transport.send(
        "GET", "session/{session_id}/title".format(**vars(session)))


def test_no_iframe(session, inline):
    session.url = inline("<title>Foobar</title><h2>Hello</h2>")

    result = get_title(session)
    assert_success(result, "Foobar")


def test_iframe(session, one_frame_doc):
    session.url = one_frame_doc

    frame = session.find.css("iframe", all=False)
    session.switch_frame(frame)
    session.find.css("p", all=False)

    response = get_title(session)
    assert_success(response, "bar")


def test_nested_iframe(session, nested_frames_doc):
    session.url = nested_frames_doc

    outer_frame = session.find.css("iframe", all=False)
    session.switch_frame(outer_frame)

    inner_frame = session.find.css("iframe", all=False)
    session.switch_frame(inner_frame)
    session.find.css("p", all=False)

    response = get_title(session)
    assert_success(response, "foo")


@pytest.mark.parametrize("domain", ["", "alt"], ids=["same_origin", "cross_origin"])
def test_origin(session, inline, iframe, domain):
    session.url = inline("<title>foo</title>{}".format(
        iframe("<title>bar</title><p>frame", domain=domain)))

    frame = session.find.css("iframe", all=False)
    session.switch_frame(frame)
    session.find.css("p", all=False)

    response = get_title(session)
    assert_success(response, "foo")
