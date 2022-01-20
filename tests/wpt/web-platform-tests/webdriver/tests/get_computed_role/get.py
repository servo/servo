import pytest

from webdriver.error import NoSuchAlertException

from tests.support.asserts import assert_error, assert_success


def get_computed_role(session, element):
    return session.transport.send(
        "GET", "session/{session_id}/element/{element_id}/computedrole".format(
            session_id=session.session_id,
            element_id=element))


def test_no_browsing_context(session, closed_window):
    response = get_computed_role(session, "id")
    assert_error(response, "no such window")


def test_no_user_prompt(session):
    response = get_computed_role(session, "id")
    assert_error(response, "no such alert")


@pytest.mark.parametrize("html,tag,expected", [
    ("<li role=menuitem>foo", "li", "menu"),
    ("<input role=searchbox>", "input", "searchbox"),
    ("<img role=presentation>", "img", "presentation")])
def test_computed_roles(session, inline, html, tag, expected):
    session.url = inline(html)
    element = session.find.css(tag, all=False)
    result = get_computed_role(session, element.id)
    assert_success(result, expected)
