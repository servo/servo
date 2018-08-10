import pytest

from webdriver import StaleElementReferenceException

from tests.support.asserts import assert_error, assert_success
from tests.support.inline import inline, iframe


def switch_to_parent_frame(session):
    return session.transport.send(
        "POST", "session/{session_id}/frame/parent".format(**vars(session)))


def test_null_response_value(session):
    session.url = inline(iframe("<p>foo"))
    frame_element = session.find.css("iframe", all=False)
    session.switch_frame(frame_element)

    response = switch_to_parent_frame(session)
    value = assert_success(response)
    assert value is None


def test_no_browsing_context(session, closed_window):
    response = switch_to_parent_frame(session)
    assert_error(response, "no such window")


def test_stale_element_from_iframe(session):
    session.url = inline(iframe("<p>foo"))
    frame_element = session.find.css("iframe", all=False)
    session.switch_frame(frame_element)
    stale_element = session.find.css("p", all=False)

    result = switch_to_parent_frame(session)
    assert_success(result)

    with pytest.raises(StaleElementReferenceException):
        stale_element.text
