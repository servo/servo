import pytest
from webdriver import StaleElementReferenceException

from tests.support.inline import inline, iframe


def switch_to_parent_frame(session):
    return session.transport.send("POST", "session/%s/frame/parent" % session.session_id)


def test_stale_element_from_iframe(session):
    session.url = inline(iframe("<p>foo"))
    frame_element = session.find.css("iframe", all=False)
    session.switch_frame(frame_element)
    stale_element = session.find.css("p", all=False)
    switch_to_parent_frame(session)
    with pytest.raises(StaleElementReferenceException):
        stale_element.text
