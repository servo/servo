import pytest

from tests.support.asserts import assert_error, assert_success
from tests.support.inline import inline

def element_click(session, element):
    return session.transport.send(
        "POST", "session/{session_id}/element/{element_id}/click".format(
            session_id=session.session_id,
            element_id=element.id))


def test_scroll_into_view(session):
    session.url = inline("""
        <input type=text value=Federer
        style="position: absolute; left: 0vh; top: 500vh">""")

    element = session.find.css("input", all=False)
    response = element_click(session, element)
    assert_success(response)

    # Check if element clicked is scrolled into view
    assert session.execute_script("""
        let input = arguments[0];
        rect = input.getBoundingClientRect();
        return rect["top"] >= 0 && rect["left"] >= 0 &&
            (rect["top"] + rect["height"]) <= window.innerHeight &&
            (rect["left"] + rect["width"]) <= window.innerWidth;
            """, args=(element,)) is True
